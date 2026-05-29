pub mod argon2;

use std::sync::Arc;

pub trait SecretCipher: Send + Sync {
    fn encrypt(&self, plaintext: &str) -> Result<String, String>;
    fn decrypt(&self, ciphertext: &str) -> Result<String, String>;
}

pub type SharedSecretCipher = Arc<dyn SecretCipher>;
