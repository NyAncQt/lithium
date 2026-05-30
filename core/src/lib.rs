pub mod identity;
pub mod storage;

use identity::User;
use storage::Db;
use yosemite::{style::Stream, Session};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use snow::Builder;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

#[derive(Clone)]
pub struct App {
    pub user: Arc<AsyncMutex<User>>,
    pub db: Arc<Db>,
}

static PATTERN: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2b";

impl App {
    pub fn new(u: User, d: Db) -> Self {
        Self { 
            user: Arc::new(AsyncMutex::new(u)), 
            db: Arc::new(d) 
        }
    }

    pub async fn init_session(&self) -> Session<Stream> {
        let sess = Session::<Stream>::new(Default::default()).await.unwrap();
        let dest = sess.destination().to_string();
        
        let mut user = self.user.lock().await;
        user.dest = dest.clone();
        self.db.set_meta("dest", &dest);
        
        sess
    }

    pub async fn send(&self, cid: i32, addr: &str, txt: &str) {
        let mut sess = Session::<Stream>::new(Default::default()).await.unwrap();
        let mut stream = sess.connect(addr).await.unwrap();

        let key = self.user.lock().await.get_key_bytes();
        let builder = Builder::new(PATTERN.parse().unwrap());
        let mut noise = builder.local_private_key(&key).build_initiator().unwrap();

        let mut buf = vec![0u8; 65535];
        let len = noise.write_message(&[], &mut buf).unwrap();
        stream.write_all(&buf[..len]).await.unwrap();
        let n = stream.read(&mut buf).await.unwrap();
        noise.read_message(&buf[..n], &mut []).unwrap();
        let len = noise.write_message(&[], &mut buf).unwrap();
        stream.write_all(&buf[..len]).await.unwrap();

        let mut noise = noise.into_transport_mode().unwrap();
        let len = noise.write_message(txt.as_bytes(), &mut buf).unwrap();
        stream.write_all(&buf[..len]).await.unwrap();
        
        self.db.save_msg(cid, txt, true); 
    }

    pub async fn listen(&self) {
        let mut sess = self.init_session().await;
        loop {
            let mut stream = sess.accept().await.unwrap();
            let addr = stream.remote_destination().to_string();
            let mut buf = vec![0; 65535];
            
            let key = self.user.lock().await.get_key_bytes();
            let builder = Builder::new(PATTERN.parse().unwrap());
            let mut noise = builder.local_private_key(&key).build_responder().unwrap();

            let n = stream.read(&mut buf).await.unwrap();
            noise.read_message(&buf[..n], &mut []).unwrap();
            let len = noise.write_message(&[], &mut buf).unwrap();
            stream.write_all(&buf[..len]).await.unwrap();
            let n = stream.read(&mut buf).await.unwrap();
            noise.read_message(&buf[..n], &mut []).unwrap();

            let mut noise = noise.into_transport_mode().unwrap();
            let n = stream.read(&mut buf).await.unwrap();
            let mut msg_buf = vec![0u8; 65535];
            let len = noise.read_message(&buf[..n], &mut msg_buf).unwrap();
            let msg = String::from_utf8_lossy(&msg_buf[..len]).to_string();
            
            let mut cid = 0;
            for c in self.db.get_contacts() {
                if c.addr == addr { cid = c.id; break; }
            }
            if cid == 0 { cid = self.db.add_contact("Unknown", &addr); }

            self.db.save_msg(cid, &msg, false);
        }
    }
}
