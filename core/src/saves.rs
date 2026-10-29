//! The games' saves, and backups of the whole `<save>/saves/` folder. Dark Cloud keeps `saves/<n>/save.dat` and
//! `saves/autosave/save.dat`, each retail's card image (Chronicle `docs/PC.md`, "Saves and host files"). Dark Chronicle
//! keeps `saves/BESCES-51190dkcl<n>.json` (File n + 1) and `saves/autosave.json`, JSON with a header the save list
//! reads (ChronicleTwo `docs/PC.md`, "Saves").

use crate::games::SaveFormat;
use serde::Serialize;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Offsets in `CSaveData` (Chronicle `ps2/include/savedata.hpp`): after `config[18]` and `chara_name[6][32]`.
const MAP_NO: usize = 0x1C8;
const PLAY_TIME: usize = 0x1D4;
const DAY: usize = 0x1D8;
/// The version string follows `CSaveData`'s 0x131C0 bytes.
const VERSION: usize = 0x131C0;
const VERSION_PREFIX: &[u8] = b"darkcloudVer";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Save {
    /// The folder's name: the number the save board shows, or `autosave`.
    pub name: String,
    pub number: Option<u32>,
    /// Play time in seconds (the game counts frames, 60 a second).
    pub play_seconds: Option<u64>,
    pub day: Option<i32>,
    pub map_no: Option<i32>,
    /// Seconds since 1970 when the save was last written.
    pub modified: Option<u64>,
    /// False when the file is not a Dark Cloud save the launcher can read; the game decides what it does with it.
    pub readable: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Backup {
    /// The backup's folder name: seconds since 1970 when it was made.
    pub id: String,
    pub made: u64,
    pub saves: usize,
}

fn le_i32(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn unix(time: io::Result<SystemTime>) -> Option<u64> {
    time.ok()?.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())
}

pub fn now() -> u64 {
    unix(Ok(SystemTime::now())).unwrap_or(0)
}

/// What Dark Cloud's save board shows, from a `save.dat`.
pub fn read(name: &str, file: &Path) -> Save {
    let number = name.parse::<u32>().ok().filter(|_| !name.starts_with('0'));
    let modified = std::fs::metadata(file).ok().and_then(|m| unix(m.modified()));
    let mut save = Save {
        name: name.to_string(),
        number,
        play_seconds: None,
        day: None,
        map_no: None,
        modified,
        readable: false,
    };
    let Ok(bytes) = std::fs::read(file) else {
        return save;
    };
    if bytes.len() < VERSION + VERSION_PREFIX.len() || !bytes[VERSION..].starts_with(VERSION_PREFIX) {
        return save;
    }
    save.readable = true;
    save.play_seconds = Some(le_i32(&bytes, PLAY_TIME).max(0) as u64 / 60);
    save.day = Some(le_i32(&bytes, DAY));
    save.map_no = Some(le_i32(&bytes, MAP_NO));
    save
}

/// Dark Chronicle's save files are named after the card files they replace.
const DC2_PREFIX: &str = "BESCES-51190dkcl";
/// Dark Chronicle counts play time at PAL's 50 a second (ChronicleTwo `port/src/menu_save.cpp`).
const DC2_TICKS: u64 = 50;

/// What Dark Chronicle's save list shows, from a save's JSON header.
pub fn read_dark_chronicle(name: &str, number: Option<u32>, file: &Path) -> Save {
    let metadata = std::fs::metadata(file).ok();
    let mut save = Save {
        name: name.to_string(),
        number,
        play_seconds: None,
        day: None,
        map_no: None,
        modified: metadata.as_ref().and_then(|m| unix(m.modified())),
        readable: false,
    };
    if metadata.is_none_or(|m| m.len() > 64 << 20) {
        return save;
    }
    let Some(document) = std::fs::read(file)
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
    else {
        return save;
    };
    let header = &document["header"];
    if document["format"] != "chronicletwo-save" || header["version"] != "dc2Ver4" {
        return save;
    }
    save.readable = true;
    save.play_seconds = header["play_time"].as_u64().map(|t| t / DC2_TICKS);
    save.map_no = header["map_no"].as_i64().map(|m| m as i32);
    save
}

/// The autosave first, then the numbered saves in order. Anything else in the folder is left out.
pub fn list(format: SaveFormat, save_root: &Path) -> Vec<Save> {
    let Ok(entries) = std::fs::read_dir(save_root.join("saves")) else {
        return Vec::new();
    };
    let mut saves: Vec<Save> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            match format {
                SaveFormat::DarkCloud => {
                    let file = e.path().join("save.dat");
                    file.is_file().then(|| read(&name, &file))
                }
                SaveFormat::DarkChronicle => {
                    if !e.path().is_file() {
                        return None;
                    }
                    if name == "autosave.json" {
                        return Some(read_dark_chronicle("autosave", None, &e.path()));
                    }
                    let n = name.strip_prefix(DC2_PREFIX)?.strip_suffix(".json")?;
                    let number = n
                        .parse::<u32>()
                        .ok()
                        .filter(|_| !n.starts_with('0') || n == "0")?;
                    Some(read_dark_chronicle(
                        &number.saturating_add(1).to_string(),
                        Some(number + 1),
                        &e.path(),
                    ))
                }
            }
        })
        .collect();
    saves.sort_by_key(|s| (s.name != "autosave", s.number.unwrap_or(u32::MAX), s.name.clone()));
    saves
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<usize> {
    std::fs::create_dir_all(to)?;
    let mut files = 0;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            files += copy_dir(&entry.path(), &to.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), to.join(entry.file_name()))?;
            files += 1;
        }
    }
    Ok(files)
}

fn backups_dir(game_dir: &Path) -> PathBuf {
    game_dir.join("backups")
}

/// Copies `saves/` into `backups/<now>/saves`. Nothing to back up is not an error: it makes no backup.
pub fn back_up(format: SaveFormat, game_dir: &Path, save_root: &Path) -> io::Result<Option<Backup>> {
    let saves = save_root.join("saves");
    if !saves.is_dir() || list(format, save_root).is_empty() {
        return Ok(None);
    }
    let mut made = now();
    while backups_dir(game_dir).join(made.to_string()).exists() {
        made += 1;
    }
    let folder = backups_dir(game_dir).join(made.to_string());
    copy_dir(&saves, &folder.join("saves"))?;
    Ok(Some(Backup {
        id: made.to_string(),
        made,
        saves: list(format, &folder).len(),
    }))
}

/// Newest first.
pub fn backups(format: SaveFormat, game_dir: &Path) -> Vec<Backup> {
    let Ok(entries) = std::fs::read_dir(backups_dir(game_dir)) else {
        return Vec::new();
    };
    let mut backups: Vec<Backup> = entries
        .flatten()
        .filter_map(|e| {
            let made = e.file_name().to_str()?.parse::<u64>().ok()?;
            Some(Backup {
                id: made.to_string(),
                made,
                saves: list(format, &e.path()).len(),
            })
        })
        .collect();
    backups.sort_by_key(|b| std::cmp::Reverse(b.made));
    backups
}

/// Puts a backup's saves back. What is there now is backed up first, so a restore can itself be undone.
pub fn restore(
    format: SaveFormat,
    game_dir: &Path,
    save_root: &Path,
    id: &str,
) -> io::Result<Option<Backup>> {
    if id.parse::<u64>().is_err() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no such backup"));
    }
    let source = backups_dir(game_dir).join(id).join("saves");
    if !source.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "no such backup"));
    }
    let before = back_up(format, game_dir, save_root)?;
    let saves = save_root.join("saves");
    let staging = save_root.join("saves.restoring");
    let _ = std::fs::remove_dir_all(&staging);
    copy_dir(&source, &staging)?;
    if saves.exists() {
        std::fs::remove_dir_all(&saves)?;
    }
    std::fs::rename(staging, saves)?;
    Ok(before)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A save image with the board's fields set, the size retail writes.
    fn save_dat(play_frames: i32, day: i32, map: i32) -> Vec<u8> {
        let mut bytes = vec![0u8; 0x136A7];
        bytes[PLAY_TIME..PLAY_TIME + 4].copy_from_slice(&play_frames.to_le_bytes());
        bytes[DAY..DAY + 4].copy_from_slice(&day.to_le_bytes());
        bytes[MAP_NO..MAP_NO + 4].copy_from_slice(&map.to_le_bytes());
        bytes[VERSION..VERSION + 15].copy_from_slice(b"darkcloudVer1.9");
        bytes
    }

    fn put(root: &Path, name: &str, bytes: &[u8]) {
        let dir = root.join("saves").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("save.dat"), bytes).unwrap();
    }

    #[test]
    fn reads_what_the_board_shows() {
        let root = tempfile::tempdir().unwrap();
        put(root.path(), "2", &save_dat(60 * 3725, 41, 3));
        put(root.path(), "10", &save_dat(60, 1, 0));
        put(root.path(), "autosave", &save_dat(0, 2, 1));
        put(root.path(), "1", b"not a save");
        std::fs::create_dir_all(root.path().join("saves/3")).unwrap();
        let saves = list(SaveFormat::DarkCloud, root.path());
        let names: Vec<&str> = saves.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["autosave", "1", "2", "10"]);
        assert!(!saves[1].readable);
        assert_eq!(saves[2].play_seconds, Some(3725));
        assert_eq!(saves[2].day, Some(41));
        assert_eq!(saves[2].number, Some(2));
        assert_eq!(saves[0].number, None);
    }

    #[test]
    fn restore_brings_saves_back_and_keeps_what_was_there() {
        let game = tempfile::tempdir().unwrap();
        let save_root = game.path().join("save");
        put(&save_root, "1", &save_dat(60, 1, 0));
        let first = back_up(SaveFormat::DarkCloud, game.path(), &save_root)
            .unwrap()
            .unwrap();
        assert_eq!(first.saves, 1);

        put(&save_root, "1", &save_dat(120, 9, 0));
        put(&save_root, "2", &save_dat(60, 1, 0));
        let before = restore(SaveFormat::DarkCloud, game.path(), &save_root, &first.id)
            .unwrap()
            .unwrap();
        assert_eq!(before.saves, 2);
        let now = list(SaveFormat::DarkCloud, &save_root);
        assert_eq!(now.len(), 1);
        assert_eq!(now[0].day, Some(1));
        assert_eq!(backups(SaveFormat::DarkCloud, game.path()).len(), 2);
        assert_eq!(backups(SaveFormat::DarkCloud, game.path())[0].id, before.id);
    }

    #[test]
    fn nothing_to_back_up_makes_no_backup() {
        let game = tempfile::tempdir().unwrap();
        assert_eq!(
            back_up(SaveFormat::DarkCloud, game.path(), &game.path().join("save")).unwrap(),
            None
        );
        assert!(restore(
            SaveFormat::DarkCloud,
            game.path(),
            &game.path().join("save"),
            "../x"
        )
        .is_err());
    }

    #[test]
    fn reads_dark_chronicles_save_list() {
        let root = tempfile::tempdir().unwrap();
        let saves = root.path().join("saves");
        std::fs::create_dir_all(&saves).unwrap();
        let save = |play_time: u64, map: i64| {
            serde_json::json!({
                "format": "chronicletwo-save", "schema_version": 1,
                "header": {"version": "dc2Ver4", "map_no": map, "play_time": play_time, "game_progress": 3},
                "save": {}, "checksum": "crc32:00000000"
            })
            .to_string()
        };
        std::fs::write(saves.join("BESCES-51190dkcl0.json"), save(50 * 3600, 10)).unwrap();
        std::fs::write(saves.join("BESCES-51190dkcl11.json"), save(50 * 61, 2)).unwrap();
        std::fs::write(saves.join("autosave.json"), save(50, 10)).unwrap();
        std::fs::write(saves.join("BESCES-51190dkcl2.json"), "{ not json").unwrap();
        std::fs::write(saves.join("BESCES-51190dkcl3.json.0123456789abcdef.tmp"), "").unwrap();
        std::fs::write(saves.join("notes.txt"), "").unwrap();

        let list = list(SaveFormat::DarkChronicle, root.path());
        let names: Vec<&str> = list.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["autosave", "1", "3", "12"]);
        assert_eq!(list[1].play_seconds, Some(3600));
        assert_eq!(list[1].map_no, Some(10));
        assert_eq!(list[1].day, None);
        assert!(!list[2].readable);
        assert_eq!(list[3].play_seconds, Some(61));
        assert_eq!(list[0].number, None);
        assert_eq!(
            back_up(SaveFormat::DarkChronicle, root.path(), root.path())
                .unwrap()
                .unwrap()
                .saves,
            4
        );
    }
}
