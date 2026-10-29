//! Everything the launcher does to files and processes. The Tauri app (`src-tauri/`) only turns these into
//! commands; nothing here knows about windows or the UI.

pub mod config;
pub mod games;
pub mod library;
pub mod paths;
pub mod port;
