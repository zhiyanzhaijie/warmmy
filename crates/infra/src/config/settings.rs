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
    #[serde(skip)]
    pub lancedb_path: String,
    #[serde(default = "default_rag_top_k")]
    pub top_k: usize,
}

impl Default for RagSettings {
    fn default() -> Self {
        Self {
            lancedb_path: String::new(),
            top_k: default_rag_top_k(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| default_app_env().to_string());

        let builder = ConfigBuilder::builder();

        let builder = builder.add_source(File::from_str(
            embedded_config_toml(&env),
            config::FileFormat::Toml,
        ));

        let mut config: Self = builder
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()?;

        config.normalize_local_paths();
        Ok(config)
    }

    fn normalize_local_paths(&mut self) {
        let Some(data_dir) = app_data_dir() else {
            self.rag.lancedb_path = fixed_lancedb_path(Path::new("."))
                .to_string_lossy()
                .to_string();
            ensure_parent_dir(Path::new(&self.rag.lancedb_path));
            ensure_sqlite_parent_dir(&self.database.url);
            return;
        };

        self.rag.lancedb_path = fixed_lancedb_path(&data_dir)
            .to_string_lossy()
            .to_string();
        ensure_parent_dir(Path::new(&self.rag.lancedb_path));

        self.database.url = normalize_sqlite_path(&self.database.url, &data_dir);
        ensure_sqlite_parent_dir(&self.database.url);
    }
}

fn fixed_lancedb_path(data_dir: &Path) -> PathBuf {
    data_dir.join("vectordb")
}

fn default_rag_top_k() -> usize {
    3
}

fn default_app_env() -> &'static str {
    if cfg!(debug_assertions) {
        "development"
    } else {
        "production"
    }
}

fn embedded_config_toml(env: &str) -> &'static str {
    match env {
        "production" => include_str!("toml/production.toml"),
        "development" => include_str!("toml/development.toml"),
        _ => include_str!("toml/development.toml"),
    }
}

fn normalize_sqlite_path(url: &str, data_dir: &Path) -> String {
    if url == "sqlite::memory:" {
        return url.to_string();
    }

    if let Some(path) = url.strip_prefix("sqlite://") {
        return normalize_path(path, data_dir).to_string_lossy().to_string();
    }

    if let Some(path) = url.strip_prefix("sqlite:") {
        return normalize_path(path, data_dir).to_string_lossy().to_string();
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
    if let Ok(path) = std::env::var("WARMMY_DATA_DIR") {
        return Some(PathBuf::from(path));
    }

    directories::ProjectDirs::from("com", "zhiyanzhaijie", "Warmmy")
        .map(|dirs| dirs.data_dir().to_path_buf())
}
