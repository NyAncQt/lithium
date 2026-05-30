use ed25519_dalek::{SigningKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub key: [u8; 32],
    pub dest: String,
}

impl User {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let sk = SigningKey::generate(&mut rng);
        Self { key: sk.to_bytes(), dest: "".to_string() }
    }

    pub fn from_bytes(b: [u8; 32]) -> Self {
        Self { key: b, dest: "".to_string() }
    }

    pub fn get_key_bytes(&self) -> [u8; 32] {
        self.key
    }
}
