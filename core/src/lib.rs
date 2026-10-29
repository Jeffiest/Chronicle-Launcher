//! Everything the launcher does to files and processes. The Tauri app (`src-tauri/`) only turns these into
//! commands; nothing here knows about windows or the UI.

pub mod config;
pub mod discs;
pub mod games;
pub mod iso;
pub mod library;
pub mod mods;
pub mod paths;
pub mod play;
pub mod port;
pub mod process;
pub mod saves;
pub mod setup;
