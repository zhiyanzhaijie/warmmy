//! This crate contains all shared UI for the workspace.
pub mod blocks;
pub mod components;
pub mod impls;
pub mod views;

pub fn today_session_id() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}
