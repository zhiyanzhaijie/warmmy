use std::path::{Path, PathBuf};
use std::sync::Arc;

use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use zeroize::Zeroizing;

use super::SecretCipher;

const ENV_SECRET_KEY: &str = "WARMMY_SECRET_KEY";
const KEY_FILE_NAME: &str = ".warmmy-secret-key";
const ENVELOPE_PREFIX: &str = "warmmy-secret:v1";
const KDF_SALT: &[u8] = b"warmmy:local-first:secret-store:v1";
const NONCE_LEN: usize = 12;
const ROOT_KEY_LEN: usize = 32;
const DERIVED_KEY_LEN: usize = 32;

#[derive(Clone)]
pub struct Argon2SecretCipher {
    key: Arc<Zeroizing<[u8; DERIVED_KEY_LEN]>>,
}

impl Argon2SecretCipher {
    pub fn from_database_url(database_url: &str) -> Result<Self, String> {
        let root_key = load_or_create_root_key(database_url)?;
        let key = derive_encryption_key(root_key.as_slice())?;
        Ok(Self { key: Arc::new(key) })
    }

    fn less_safe_key(&self) -> Result<LessSafeKey, String> {
        let unbound = UnboundKey::new(&AES_256_GCM, self.key.as_ref().as_slice())
            .map_err(|_| "failed to initialize secret cipher".to_string())?;
        Ok(LessSafeKey::new(unbound))
    }
}

impl SecretCipher for Argon2SecretCipher {
    fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| "failed to generate secret nonce".to_string())?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = plaintext.as_bytes().to_vec();
        self.less_safe_key()?
            .seal_in_place_append_tag(nonce, Aad::from(ENVELOPE_PREFIX.as_bytes()), &mut in_out)
            .map_err(|_| "failed to encrypt secret".to_string())?;

        Ok(format!(
            "{}:{}:{}",
            ENVELOPE_PREFIX,
            STANDARD.encode(nonce_bytes),
            STANDARD.encode(in_out)
        ))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<String, String> {
        if !ciphertext.starts_with(ENVELOPE_PREFIX) {
            // Development compatibility for rows written before encrypted secrets existed.
            return Ok(ciphertext.to_string());
        }

        let parts = ciphertext.split(':').collect::<Vec<_>>();
        if parts.len() != 4 || parts[0] != "warmmy-secret" || parts[1] != "v1" {
            return Err("invalid secret envelope".to_string());
        }

        let nonce_bytes = STANDARD
            .decode(parts[2])
            .map_err(|err| format!("invalid secret nonce: {err}"))?;
        let nonce_bytes: [u8; NONCE_LEN] = nonce_bytes
            .try_into()
            .map_err(|_| "invalid secret nonce length".to_string())?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = STANDARD
            .decode(parts[3])
            .map_err(|err| format!("invalid secret ciphertext: {err}"))?;
        let plaintext = self
            .less_safe_key()?
            .open_in_place(nonce, Aad::from(ENVELOPE_PREFIX.as_bytes()), &mut in_out)
            .map_err(|_| "failed to decrypt secret".to_string())?;

        String::from_utf8(plaintext.to_vec()).map_err(|err| format!("invalid secret utf8: {err}"))
    }
}

fn load_or_create_root_key(database_url: &str) -> Result<Zeroizing<[u8; ROOT_KEY_LEN]>, String> {
    if let Ok(value) = std::env::var(ENV_SECRET_KEY) {
        return normalize_root_key(value.as_bytes());
    }

    let path = key_file_path(database_url);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create secret key directory: {err}"))?;
    }

    if path.exists() {
        let value = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read secret key file: {err}"))?;
        let raw = STANDARD
            .decode(value.trim())
            .map_err(|err| format!("invalid secret key file: {err}"))?;
        return normalize_root_key(&raw);
    }

    let rng = SystemRandom::new();
    let mut key = Zeroizing::new([0u8; ROOT_KEY_LEN]);
    rng.fill(key.as_mut_slice())
        .map_err(|_| "failed to generate local secret key".to_string())?;
    std::fs::write(&path, STANDARD.encode(key.as_slice()))
        .map_err(|err| format!("failed to write secret key file: {err}"))?;
    Ok(key)
}

fn normalize_root_key(input: &[u8]) -> Result<Zeroizing<[u8; ROOT_KEY_LEN]>, String> {
    if input.is_empty() {
        return Err(format!("{ENV_SECRET_KEY} is empty"));
    }

    let mut root = Zeroizing::new([0u8; ROOT_KEY_LEN]);
    if input.len() == ROOT_KEY_LEN {
        root.copy_from_slice(input);
        return Ok(root);
    }

    derive_encryption_key(input)
}

fn derive_encryption_key(input: &[u8]) -> Result<Zeroizing<[u8; DERIVED_KEY_LEN]>, String> {
    let params = Params::new(19_456, 2, 1, Some(DERIVED_KEY_LEN))
        .map_err(|err| format!("invalid argon2 params: {err}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; DERIVED_KEY_LEN]);
    argon2
        .hash_password_into(input, KDF_SALT, key.as_mut_slice())
        .map_err(|err| format!("failed to derive secret key: {err}"))?;
    Ok(key)
}

fn key_file_path(database_url: &str) -> PathBuf {
    sqlite_path_from_url(database_url)
        .and_then(|path| {
            Path::new(path)
                .parent()
                .map(|parent| parent.join(KEY_FILE_NAME))
        })
        .unwrap_or_else(|| PathBuf::from("data").join(KEY_FILE_NAME))
}

fn sqlite_path_from_url(database_url: &str) -> Option<&str> {
    if database_url == "sqlite::memory:" {
        return None;
    }

    if let Some(path) = database_url.strip_prefix("sqlite://") {
        return Some(path);
    }

    if let Some(path) = database_url.strip_prefix("sqlite:") {
        return Some(path);
    }

    Some(database_url)
}
