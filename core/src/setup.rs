//! Setting a game up from the player's disc with the port's own extractor, and taking that set-up away again.
//! Set-up is done when `setup.json` is beside the data: it is written only after the extractor succeeded.

use crate::paths::GameDirs;
use crate::port::Build;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetUp {
    pub disc: PathBuf,
    pub release: Option<String>,
    pub build: PathBuf,
    /// Seconds since 1970 when the extractor finished.
    pub finished: u64,
}

/// How far the extractor has got, from what it prints (`tools/dcdata/main.cpp`, `ExtractRaw` then `BakeAssets`).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    Starting,
    ReadingDisc,
    Sorting,
    Checking,
    Preparing,
    Done,
}

impl Stage {
    /// How much of the moon is lit: rough, by stage, since the extractor prints no count while it reads.
    pub fn fraction(self) -> f32 {
        match self {
            Stage::Starting => 0.02,
            Stage::ReadingDisc => 0.15,
            Stage::Sorting => 0.55,
            Stage::Checking => 0.65,
            Stage::Preparing => 0.75,
            Stage::Done => 1.0,
        }
    }

    /// The stage a line of `dcdata extract`'s output starts, if it starts one.
    pub fn of_line(line: &str) -> Option<Stage> {
        if line.starts_with("source: ") {
            Some(Stage::ReadingDisc)
        } else if line.starts_with("release: ") {
            Some(Stage::Sorting)
        } else if line.starts_with("index: ") {
            Some(Stage::Checking)
        } else if line.starts_with("verified ") {
            Some(Stage::Preparing)
        } else {
            None
        }
    }
}

/// The release the extractor names (`release: NTSC 1.02`): its word is final, even for a disc the launcher's own
/// table does not know.
pub fn release_in_line(line: &str) -> Option<&str> {
    line.strip_prefix("release: ")
        .map(str::trim)
        .filter(|r| !r.is_empty())
}

fn marker(dirs: &GameDirs) -> PathBuf {
    dirs.data.with_file_name("setup.json")
}

pub fn read(dirs: &GameDirs) -> Option<SetUp> {
    if !dirs.data.is_dir() {
        return None;
    }
    serde_json::from_slice(&std::fs::read(marker(dirs)).ok()?).ok()
}

/// `dcdata extract <disc> <data>`.
pub fn command(build: &Build, disc: &Path, dirs: &GameDirs) -> Command {
    let mut command = Command::new(&build.extractor);
    command
        .arg("extract")
        .arg(disc)
        .arg(&dirs.data)
        .current_dir(&build.dir);
    command
}

pub fn finish(dirs: &GameDirs, done: &SetUp) -> io::Result<()> {
    let file = marker(dirs);
    let temporary = file.with_extension("json.new");
    std::fs::write(&temporary, serde_json::to_vec_pretty(done)?)?;
    std::fs::rename(temporary, file)
}

/// Forgets that set-up is done, keeping the data: the next start sets up again over it (the extractor keeps
/// files already present).
pub fn forget(dirs: &GameDirs) {
    let _ = std::fs::remove_file(marker(dirs));
}

/// Takes away what set-up made: the data and the marker. The saves, the logs and the disc stay.
pub fn remove(dirs: &GameDirs) -> io::Result<()> {
    let _ = std::fs::remove_file(marker(dirs));
    for archive in ["data.zip", "data.json"] {
        let _ = std::fs::remove_file(dirs.data.with_file_name(archive));
    }
    match std::fs::remove_dir_all(&dirs.data) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::game_dirs;

    #[test]
    fn stages_follow_the_extractors_output() {
        let output = [
            "source: /d/Dark Cloud (NTSC).iso (ISO 9660 image)",
            "DATA.DAT: 1 bytes, DATA.HD2: 2 bytes",
            "release: NTSC 1.02",
            "index: 10 records, 9 files, 100 bytes",
            "wrote 9 files (100 bytes), kept 0 already present",
            "verified 9/9 files in /data",
        ];
        let stages: Vec<Stage> = output.iter().filter_map(|l| Stage::of_line(l)).collect();
        assert_eq!(
            stages,
            [
                Stage::ReadingDisc,
                Stage::Sorting,
                Stage::Checking,
                Stage::Preparing
            ]
        );
        assert!(stages.windows(2).all(|w| w[0].fraction() < w[1].fraction()));
        assert_eq!(output.iter().find_map(|l| release_in_line(l)), Some("NTSC 1.02"));
    }

    #[test]
    fn set_up_only_once_finished_and_removed_without_the_saves() {
        let root = tempfile::tempdir().unwrap();
        let dirs = game_dirs(root.path(), "chronicle");
        std::fs::create_dir_all(dirs.data.join("normalized")).unwrap();
        std::fs::create_dir_all(dirs.save.join("saves/1")).unwrap();
        assert_eq!(read(&dirs), None);
        let done = SetUp {
            disc: "/d.iso".into(),
            release: Some("NTSC 1.02".into()),
            build: "/b".into(),
            finished: 1,
        };
        finish(&dirs, &done).unwrap();
        assert_eq!(read(&dirs), Some(done));
        remove(&dirs).unwrap();
        assert_eq!(read(&dirs), None);
        assert!(!dirs.data.exists());
        assert!(dirs.save.join("saves/1").is_dir());
    }
}
