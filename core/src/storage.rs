use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct Contact {
    pub id: i32,
    pub name: String,
    pub addr: String,
}

pub struct Db {
    pub conn: Mutex<Connection>,
}

impl Db {
    pub fn init<P: AsRef<Path>>(p: P, pass: &str) -> Self {
        let conn = Connection::open(p).unwrap();
        conn.pragma_update(None, "key", pass).unwrap();
        let db = Self { conn: Mutex::new(conn) };
        db.setup();
        db
    }

    fn setup(&self) {
        let conn = self.conn.lock().unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS contacts (id INTEGER PRIMARY KEY, name TEXT NOT NULL, addr TEXT NOT NULL UNIQUE, pubkey TEXT NOT NULL)", []).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS msgs (id INTEGER PRIMARY KEY, cid INTEGER, txt TEXT NOT NULL, ts DATETIME DEFAULT CURRENT_TIMESTAMP, mine BOOLEAN NOT NULL)", []).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, val TEXT)", []).unwrap();
    }

    pub fn save_msg(&self, cid: i32, txt: &str, mine: bool) {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO msgs (cid, txt, mine) VALUES (?1, ?2, ?3)", (cid, txt, mine)).unwrap();
    }

    pub fn get_contacts(&self) -> Vec<Contact> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, addr FROM contacts").unwrap();
        let rows = stmt.query_map([], |row| Ok(Contact { id: row.get(0).unwrap(), name: row.get(1).unwrap(), addr: row.get(2).unwrap() })).unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    pub fn get_msgs_for(&self, cid: i32) -> Vec<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT txt, mine FROM msgs WHERE cid = ?1 ORDER BY ts ASC").unwrap();
        let rows = stmt.query_map([cid], |row| {
            let txt: String = row.get(0).unwrap();
            let mine: bool = row.get(1).unwrap();
            Ok(if mine { format!("Me: {}", txt) } else { format!("Them: {}", txt) })
        }).unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    pub fn add_contact(&self, name: &str, addr: &str) -> i32 {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT OR IGNORE INTO contacts (name, addr, pubkey) VALUES (?1, ?2, '')", (name, addr)).unwrap();
        conn.last_insert_rowid() as i32
    }

    pub fn set_meta(&self, k: &str, v: &str) {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT OR REPLACE INTO meta (key, val) VALUES (?1, ?2)", (k, v)).unwrap();
    }

    pub fn get_meta(&self, k: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT val FROM meta WHERE key = ?1").unwrap();
        stmt.query_row([k], |row| row.get(0)).ok()
    }
}
