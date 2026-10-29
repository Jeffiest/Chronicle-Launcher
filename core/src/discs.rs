//! Finding a game's disc: images in a folder (two folders deep), each known by what its SYSTEM.CNF boots, and its
//! release told from its index file the way the port's extractor tells it.

use crate::games::{Game, Release};
use crate::iso::Iso;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FoundDisc {
    pub path: PathBuf,
    /// The release's name, or `None` for one the extractor's table lacks (it still sets it up, by its layout).
    pub release: Option<String>,
}

/// What a folder held: this game's discs, and the files the launcher saw but cannot use, with why.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct Scan {
    pub discs: Vec<FoundDisc>,
    pub skipped: Vec<Skipped>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Skipped {
    pub path: PathBuf,
    pub why: String,
}

/// FNV-1a, 64-bit, as `dcdata.hpp`'s `Fnv1a64`.
pub fn fnv1a64(data: &[u8]) -> u64 {
    data.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ *byte as u64).wrapping_mul(0x100000001b3)
    })
}

/// The program `BOOT2` names: `cdrom0:\SCUS_971.11;1` is `SCUS_971.11`.
pub fn boot_program(system_cnf: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(system_cnf);
    let line = text
        .lines()
        .find(|line| line.trim_start().to_ascii_uppercase().starts_with("BOOT2"))?;
    let value = line.split_once('=')?.1.trim();
    let name = value.rsplit(['\\', '/', ':']).next()?;
    Some(name.split(';').next()?.trim().to_string())
}

pub fn release_of<'a>(releases: &'a [Release], index: &[u8]) -> Option<&'a Release> {
    let fnv = fnv1a64(index);
    releases.iter().find(|release| {
        release.index_size == index.len() as u64 && u64::from_str_radix(&release.index_fnv, 16) == Ok(fnv)
    })
}

/// Whether `path` is this game's disc image, and which release. An error says, in words, why it is not.
pub fn identify(game: &Game, path: &Path) -> Result<FoundDisc, String> {
    let boot = game
        .boot
        .as_deref()
        .ok_or_else(|| format!("{} cannot be set up yet", game.title))?;
    let mut iso = Iso::open(path).map_err(|e| e.to_string())?;
    let cnf = iso
        .root_file("SYSTEM.CNF")
        .map_err(|e| e.to_string())?
        .ok_or("not a PlayStation 2 disc (no SYSTEM.CNF)")?;
    match boot_program(&cnf) {
        Some(found) if found.eq_ignore_ascii_case(boot) => {}
        Some(found) => return Err(format!("another game's disc (it boots {found})")),
        None => return Err("not a PlayStation 2 disc (SYSTEM.CNF names no program)".into()),
    }
    let release = match game.index_file.as_deref() {
        Some(name) => iso
            .root_file(name)
            .map_err(|e| e.to_string())?
            .and_then(|index| release_of(&game.releases, &index).map(|r| r.name.clone())),
        None => None,
    };
    Ok(FoundDisc {
        path: path.to_path_buf(),
        release,
    })
}

fn images(folder: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if depth > 0 {
                images(&path, depth - 1, out);
            }
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("iso") || e.eq_ignore_ascii_case("chd"))
        {
            out.push(path);
        }
    }
}

/// The folder and the folders in it, two deep. A `.chd` is named but not used: the extractor reads disc images.
pub fn scan(game: &Game, folder: &Path) -> Scan {
    let mut paths = Vec::new();
    images(folder, 2, &mut paths);
    let mut scan = Scan::default();
    for path in paths {
        if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("chd")) {
            scan.skipped.push(Skipped {
                path,
                why: "a CHD: make it a disc image first, with chdman extractdvd (or extractcd)".into(),
            });
            continue;
        }
        match identify(game, &path) {
            Ok(disc) => scan.discs.push(disc),
            Err(why) => scan.skipped.push(Skipped { path, why }),
        }
    }
    scan
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games;
    use crate::iso::tests::image;

    const CNF: &[u8] = b"BOOT2 = cdrom0:\\SCUS_971.11;1\r\nVER = 1.02\r\nVMODE = NTSC\r\n";

    /// Dark Cloud with a release whose fingerprint is `index`'s, since a real disc's index cannot be in a test.
    fn dark_cloud_knowing(index: &[u8]) -> Game {
        let mut game = games::find("chronicle").unwrap().clone();
        game.releases.push(Release {
            name: "Test release".into(),
            index_size: index.len() as u64,
            index_fnv: format!("{:x}", fnv1a64(index)),
        });
        game
    }

    #[test]
    fn fnv_matches_the_reference_values() {
        assert_eq!(fnv1a64(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a64(b"a"), 0xaf63dc4c8601ec8c);
    }

    #[test]
    fn reads_the_boot_program() {
        assert_eq!(boot_program(CNF).as_deref(), Some("SCUS_971.11"));
        assert_eq!(
            boot_program(b"boot2=cdrom0:\\SCES_511.90;1").as_deref(),
            Some("SCES_511.90")
        );
        assert_eq!(boot_program(b"VMODE = PAL"), None);
    }

    #[test]
    fn knows_the_release_by_its_index() {
        let dir = tempfile::tempdir().unwrap();
        let index = vec![3u8; 5000];
        std::fs::write(
            dir.path().join("dc.iso"),
            image(&[("SYSTEM.CNF", CNF), ("DATA.HD2", &index)]),
        )
        .unwrap();
        let disc = identify(&dark_cloud_knowing(&index), &dir.path().join("dc.iso")).unwrap();
        assert_eq!(disc.release.as_deref(), Some("Test release"));
    }

    #[test]
    fn an_unknown_release_is_still_dark_cloud() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("dc.iso"),
            image(&[("SYSTEM.CNF", CNF), ("DATA.HD2", &[1; 10])]),
        )
        .unwrap();
        let disc = identify(games::find("chronicle").unwrap(), &dir.path().join("dc.iso")).unwrap();
        assert_eq!(disc.release, None);
    }

    #[test]
    fn scans_two_folders_deep_and_says_what_it_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let deep = dir.path().join("ps2/dark cloud");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("Dark Cloud (NTSC).iso"), image(&[("SYSTEM.CNF", CNF)])).unwrap();
        std::fs::write(
            dir.path().join("other.iso"),
            image(&[("SYSTEM.CNF", b"BOOT2 = cdrom0:\\SCES_511.90;1")]),
        )
        .unwrap();
        std::fs::write(dir.path().join("dc.chd"), b"MComprHD").unwrap();
        std::fs::write(dir.path().join("notes.txt"), b"").unwrap();
        let too_deep = dir.path().join("a/b/c");
        std::fs::create_dir_all(&too_deep).unwrap();
        std::fs::write(too_deep.join("hidden.iso"), image(&[("SYSTEM.CNF", CNF)])).unwrap();

        let scan = scan(games::find("chronicle").unwrap(), dir.path());
        assert_eq!(scan.discs.len(), 1);
        assert!(scan.discs[0].path.ends_with("Dark Cloud (NTSC).iso"));
        let why: Vec<&str> = scan.skipped.iter().map(|s| s.why.as_str()).collect();
        assert_eq!(why.len(), 2, "{why:?}");
        assert!(why.iter().any(|w| w.contains("SCES_511.90")));
        assert!(why.iter().any(|w| w.contains("chdman")));
    }

    #[test]
    fn a_game_without_a_boot_program_cannot_be_set_up_yet() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("dc2.iso"), image(&[("SYSTEM.CNF", CNF)])).unwrap();
        let err = identify(games::find("chronicle-two").unwrap(), &dir.path().join("dc2.iso")).unwrap_err();
        assert!(err.contains("cannot be set up yet"));
    }
}
