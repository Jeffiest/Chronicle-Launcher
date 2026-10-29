//! The launcher's own folder: `%LOCALAPPDATA%\chronicle-launcher`, `$XDG_DATA_HOME/chronicle-launcher` (or
//! `~/.local/share/chronicle-launcher`), `~/Library/Application Support/Chronicle Launcher`.

use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
const NAME: &str = "Chronicle Launcher";
#[cfg(not(target_os = "macos"))]
const NAME: &str = "chronicle-launcher";

/// `CHRONICLE_LAUNCHER_HOME` overrides the folder, for tests and for keeping a second set apart.
pub fn root() -> PathBuf {
    if let Some(home) = std::env::var_os("CHRONICLE_LAUNCHER_HOME") {
        return PathBuf::from(home);
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(NAME)
}

/// One game's folders under the root. Data and saves are passed to the port explicitly (`--data`, `--save`), so
/// its own fallbacks never pick another folder.
#[derive(Debug, Clone, PartialEq)]
pub struct GameDirs {
    /// The game's own folder, holding all the others.
    pub root: PathBuf,
    pub data: PathBuf,
    pub save: PathBuf,
    pub mods: PathBuf,
    pub logs: PathBuf,
}

pub fn game_dirs(root: &Path, id: &str) -> GameDirs {
    let game = root.join("games").join(id);
    GameDirs {
        data: game.join("data"),
        save: game.join("save"),
        mods: game.join("mods"),
        logs: game.join("logs"),
        root: game,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_game_has_its_own_folders() {
        let dirs = game_dirs(Path::new("/r"), "chronicle-two");
        assert_eq!(dirs.data, Path::new("/r/games/chronicle-two/data"));
        assert_eq!(dirs.save, Path::new("/r/games/chronicle-two/save"));
        assert_eq!(dirs.mods, Path::new("/r/games/chronicle-two/mods"));
        assert_eq!(dirs.logs, Path::new("/r/games/chronicle-two/logs"));
        assert_eq!(dirs.root, Path::new("/r/games/chronicle-two"));
    }
}
