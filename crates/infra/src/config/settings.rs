use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    #[serde(default)]
    pub rag: RagSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RagSettings {
    #[serde(default = "default_lancedb_path")]
    pub lancedb_path: String,
    #[serde(default = "default_rag_top_k")]
    pub top_k: usize,
}

impl Default for RagSettings {
    fn default() -> Self {
        Self {
            lancedb_path: default_lancedb_path(),
            top_k: default_rag_top_k(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let builder = ConfigBuilder::builder();

        #[cfg(any(target_os = "ios", target_os = "android"))]
        let builder = builder.add_source(File::from_str(
            embedded_config_toml(&env),
            config::FileFormat::Toml,
        ));

        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        let builder = {
            let path = format!("{}/src/config/toml/{env}.toml", env!("CARGO_MANIFEST_DIR"));
            builder.add_source(File::new(&path, config::FileFormat::Toml))
        };

        let mut config: Self = builder
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()?;

        config.normalize_local_paths();
        Ok(config)
    }

    fn normalize_local_paths(&mut self) {
        let Some(data_dir) = app_data_dir() else {
            ensure_parent_dir(Path::new(&self.rag.lancedb_path));
            ensure_sqlite_parent_dir(&self.database.url);
            return;
        };

        self.rag.lancedb_path = normalize_path(&self.rag.lancedb_path, &data_dir)
            .to_string_lossy()
            .to_string();
        ensure_parent_dir(Path::new(&self.rag.lancedb_path));

        self.database.url = normalize_sqlite_url(&self.database.url, &data_dir);
        ensure_sqlite_parent_dir(&self.database.url);
    }
}

fn default_lancedb_path() -> String {
    "data/lancedb-store".to_string()
}

fn default_rag_top_k() -> usize {
    3
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn embedded_config_toml(env: &str) -> &'static str {
    match env {
        "mobile" => include_str!("toml/mobile.toml"),
        "development" => include_str!("toml/development.toml"),
        _ => include_str!("toml/development.toml"),
    }
}

fn normalize_sqlite_url(url: &str, data_dir: &Path) -> String {
    if url == "sqlite::memory:" {
        return url.to_string();
    }

    if let Some(path) = url.strip_prefix("sqlite://") {
        return format!(
            "sqlite://{}",
            normalize_path(path, data_dir).to_string_lossy()
        );
    }

    if let Some(path) = url.strip_prefix("sqlite:") {
        return format!(
            "sqlite:{}",
            normalize_path(path, data_dir).to_string_lossy()
        );
    }

    normalize_path(url, data_dir).to_string_lossy().to_string()
}

fn normalize_path(path: &str, data_dir: &Path) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        data_dir.join(path)
    }
}

fn ensure_sqlite_parent_dir(url: &str) {
    if url == "sqlite::memory:" {
        return;
    }

    let path = url
        .strip_prefix("sqlite://")
        .or_else(|| url.strip_prefix("sqlite:"))
        .unwrap_or(url);
    ensure_parent_dir(Path::new(path));
}

fn ensure_parent_dir(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn app_data_dir() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("WARMMY_DATA_DIR") {
        return Some(PathBuf::from(path));
    }

    #[cfg(target_os = "ios")]
    {
        return std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join("Documents").join("warmmy"));
    }

    #[cfg(target_os = "android")]
    {
        return Some(PathBuf::from(
            "/data/data/com.zhiyanzhaijie.warmmy/files/warmmy",
        ));
    }
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
fn app_data_dir() -> Option<PathBuf> {
    None
}
