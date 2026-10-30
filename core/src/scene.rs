//! The game page's backdrop: a frame of the game itself, rendered by the port from the player's own data, or a
//! screenshot the player chooses. The launcher ships no picture of either game; each player's backdrop is made on
//! their computer, kept as `<game>/scene.png`.

use crate::games::Game;
use crate::paths::GameDirs;
use crate::port::Build;
use base64::Engine;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn file(dirs: &GameDirs) -> PathBuf {
    dirs.root.join("scene.png")
}

/// The port run headless to a place in the game (`games/*.json`, `scene_args`), on a scratch save folder so the
/// player's saves and settings are never touched, with every mod off, writing one frame.
pub fn command(game: &Game, build: &Build, dirs: &GameDirs, out: &Path) -> Command {
    let mut command = Command::new(&build.executable);
    command
        .arg("--data")
        .arg(&dirs.data)
        .arg("--save")
        .arg(dirs.root.join("scene-save"))
        .args(&game.scene_args)
        .arg("--screenshot")
        .arg(out)
        .env("MOONMOD_NO_MODS", "1")
        .current_dir(&dirs.root);
    command
}

/// Where a run writes before the frame replaces the backdrop: a failed run leaves the old one.
pub fn pending(dirs: &GameDirs) -> PathBuf {
    dirs.root.join("scene.new.png")
}

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";
const LIMIT: u64 = 24 << 20;

/// The picture's type, from its first bytes: the port writes PNG; a player's screenshot may be PNG, JPEG or WebP.
fn mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(PNG) {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Makes `from` the backdrop once it is a PNG, JPEG or WebP of a sane size; anything else leaves the backdrop as it
/// was. The file keeps the name `scene.png` whatever its type; `data_url` reads the type from its bytes.
pub fn adopt(dirs: &GameDirs, from: &Path) -> Result<(), String> {
    let size = std::fs::metadata(from)
        .map_err(|_| "No picture was written.".to_string())?
        .len();
    if size > LIMIT {
        return Err("The picture is larger than 24 MB.".into());
    }
    let bytes = std::fs::read(from).map_err(|e| e.to_string())?;
    if mime(&bytes).is_none() {
        return Err("The picture is not a PNG, JPEG or WebP.".into());
    }
    let temporary = dirs.root.join("scene.png.tmp");
    std::fs::write(&temporary, &bytes).map_err(|e| e.to_string())?;
    std::fs::rename(temporary, file(dirs)).map_err(|e| e.to_string())
}

/// The backdrop as a `data:` URL the page can show, if there is one.
pub fn data_url(dirs: &GameDirs) -> Option<String> {
    let bytes = std::fs::read(file(dirs)).ok()?;
    let mime = mime(&bytes)?;
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

pub fn remove(dirs: &GameDirs) -> io::Result<()> {
    match std::fs::remove_file(file(dirs)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{games, paths};

    fn png() -> Vec<u8> {
        let mut bytes = PNG.to_vec();
        bytes.extend_from_slice(b"rest of a picture");
        bytes
    }

    #[test]
    fn the_command_renders_one_frame_away_from_the_players_saves() {
        let root = tempfile::tempdir().unwrap();
        let dirs = paths::game_dirs(root.path(), "chronicle-two");
        let build = Build {
            dir: "/b".into(),
            executable: "/b/darkchronicle".into(),
            extractor: "/b/dc2data".into(),
        };
        let game = games::find("chronicle-two").unwrap();
        let command = command(game, &build, &dirs, &pending(&dirs));
        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().into()).collect();
        assert!(args.windows(2).any(|w| w == ["--jump", "town:10"]));
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--save" && w[1].ends_with("scene-save")));
        assert!(
            !args.iter().any(|a| a == &dirs.save.to_string_lossy()),
            "never the player's own saves"
        );
        assert_eq!(args.iter().rev().nth(1).map(String::as_str), Some("--screenshot"));
        assert!(command
            .get_envs()
            .any(|(k, v)| k == "MOONMOD_NO_MODS" && v == Some("1".as_ref())));
        for game in games::all() {
            assert!(game.scene_args.iter().any(|a| a == "--headless"), "{}", game.id);
            assert!(game.scene_args.iter().any(|a| a == "--frames"), "{}", game.id);
        }
    }

    #[test]
    fn only_a_png_becomes_the_backdrop() {
        let root = tempfile::tempdir().unwrap();
        let dirs = paths::game_dirs(root.path(), "chronicle");
        std::fs::create_dir_all(&dirs.root).unwrap();
        assert_eq!(data_url(&dirs), None);
        let bad = dirs.root.join("not.png");
        std::fs::write(&bad, b"GIF89a").unwrap();
        assert!(adopt(&dirs, &bad).is_err());
        assert!(adopt(&dirs, &dirs.root.join("missing.png")).is_err());
        assert_eq!(data_url(&dirs), None);
        let good = dirs.root.join("mine.png");
        std::fs::write(&good, png()).unwrap();
        adopt(&dirs, &good).unwrap();
        assert!(data_url(&dirs)
            .unwrap()
            .starts_with("data:image/png;base64,iVBORw0KGg"));
        let jpeg = dirs.root.join("phone.jpg");
        std::fs::write(&jpeg, [0xFF, 0xD8, 0xFF, 0xE0, 1, 2, 3]).unwrap();
        adopt(&dirs, &jpeg).unwrap();
        assert!(data_url(&dirs).unwrap().starts_with("data:image/jpeg;base64,"));
        remove(&dirs).unwrap();
        assert_eq!(data_url(&dirs), None);
    }
}
