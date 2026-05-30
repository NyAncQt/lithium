use lithium_core::identity::User;
use lithium_core::storage::Db;
use lithium_core::App;
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
    layout::{Layout, Constraint, Direction},
};
use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let d = Db::init("msg.db", "pass123");
    
    let u = if let Some(k_str) = d.get_meta("sk") {
        let b = BASE64.decode(k_str).unwrap();
        let mut key = [0u8; 32];
        key.copy_from_slice(&b);
        User::from_bytes(key)
    } else {
        let new_user = User::new();
        d.set_meta("sk", &BASE64.encode(new_user.get_key_bytes()));
        new_user
    };

    let app = App::new(u, d);

    let a_listen = app.clone();
    tokio::spawn(async move {
        a_listen.listen().await;
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input = String::new();
    let mut nc_name = String::new();
    let mut nc_addr = String::new();
    let mut sel_idx = 0;
    let mut mode = 0;

    loop {
        let contacts = app.db.get_contacts();
        let sel_contact = contacts.get(sel_idx);
        let msgs = if let Some(c) = sel_contact {
            app.db.get_msgs_for(c.id)
        } else {
            vec![]
        };
        let my_dest = app.db.get_meta("dest").unwrap_or("Connecting...".to_string());
        let display_dest = if my_dest.len() > 60 {
            format!("{}...", &my_dest[..60])
        } else {
            my_dest.clone()
        };

        terminal.draw(|f| {
            let size = f.area();
            
            let root = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(size);

            let my_dest_p = Paragraph::new(display_dest.as_str()).block(Block::default().title("My I2P Address (Share this)").borders(Borders::ALL));
            f.render_widget(my_dest_p, root[1]);

            let main_area = root[0];

            if mode >= 2 {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
                    .split(main_area);
                
                let n_p = Paragraph::new(nc_name.as_str()).block(Block::default().title(if mode == 2 { "> Name" } else { "Name" }).borders(Borders::ALL));
                let a_p = Paragraph::new(nc_addr.as_str()).block(Block::default().title(if mode == 3 { "> I2P Addr" } else { "I2P Addr" }).borders(Borders::ALL));
                f.render_widget(n_p, chunks[0]);
                f.render_widget(a_p, chunks[1]);
                return;
            }

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(main_area);

            let c_items: Vec<ListItem> = contacts.iter().enumerate().map(|(i, c)| {
                let s = if i == sel_idx && mode == 0 { format!("> {}", c.name) } else { c.name.clone() };
                ListItem::new(s)
            }).collect();
            let c_list = List::new(c_items).block(Block::default().title("Lithium Contacts").borders(Borders::ALL));
            f.render_widget(c_list, main_chunks[0]);

            let chat_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(main_chunks[1]);

            let title = if let Some(c) = sel_contact { format!("Chat with {}", c.name) } else { "No contact selected".to_string() };
            let m_items: Vec<ListItem> = msgs.iter().map(|m| ListItem::new(m.clone())).collect();
            let m_list = List::new(m_items).block(Block::default().title(title).borders(Borders::ALL));
            f.render_widget(m_list, chat_chunks[0]);

            let i_block = Block::default().title(if mode == 1 { "Type message..." } else { "Message" }).borders(Borders::ALL);
            let i_p = Paragraph::new(input.as_str()).block(i_block);
            f.render_widget(i_p, chat_chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
                    KeyCode::Char('n') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        mode = 2;
                    }
                    KeyCode::Esc => {
                        mode = 0;
                        nc_name.clear();
                        nc_addr.clear();
                    }
                    KeyCode::Up if mode == 0 => {
                        if sel_idx > 0 { sel_idx -= 1; }
                    }
                    KeyCode::Down if mode == 0 => {
                        if !contacts.is_empty() && sel_idx < contacts.len() - 1 { sel_idx += 1; }
                    }
                    KeyCode::Char(c) => {
                        if mode == 1 { input.push(c); }
                        else if mode == 2 { nc_name.push(c); }
                        else if mode == 3 { nc_addr.push(c); }
                    }
                    KeyCode::Backspace => {
                        if mode == 1 { input.pop(); }
                        else if mode == 2 { nc_name.pop(); }
                        else if mode == 3 { nc_addr.pop(); }
                    }
                    KeyCode::Tab => {
                        if mode == 0 || mode == 1 { mode = 1 - mode; }
                        else if mode == 2 || mode == 3 { mode = 5 - mode; }
                    }
                    KeyCode::Enter => {
                        if mode == 1 && !input.is_empty() {
                            if let Some(c) = sel_contact {
                                let a = app.clone();
                                let addr = c.addr.clone();
                                let txt = input.clone();
                                let cid = c.id;
                                tokio::spawn(async move { a.send(cid, &addr, &txt).await; });
                                input.clear();
                            }
                        } else if mode == 0 {
                            mode = 1;
                        } else if mode == 2 {
                            mode = 3;
                        } else if mode == 3 && !nc_name.is_empty() && !nc_addr.is_empty() {
                            app.db.add_contact(&nc_name, &nc_addr);
                            nc_name.clear();
                            nc_addr.clear();
                            mode = 0;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
