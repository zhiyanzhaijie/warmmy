//! This crate contains all shared UI for the workspace.
pub mod blocks;
pub mod components;
pub mod hooks;
pub mod impls;
pub mod platform;
pub mod providers;
pub mod views;

pub fn today_session_id() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}
