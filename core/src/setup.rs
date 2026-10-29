//! Setting a game up from the player's disc with the port's own extractor, and taking that set-up away again.
//! Set-up is done when `setup.json` is beside the data: it is written only after the extractor succeeded.

use crate::games;
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

/// How far set-up has got: how much of the moon is lit, and what it is doing, in words for the player.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Progress {
    pub fraction: f32,
    pub doing: String,
}

impl Progress {
    pub fn starting() -> Progress {
        Progress {
            fraction: 0.02,
            doing: "Starting the extractor".into(),
        }
    }
}

/// `dcdata` prints a line as each stage starts (`tools/dcdata/main.cpp`, `ExtractRaw` then `BakeAssets`) and no count
/// while it reads, so its moon waxes by stage.
fn dcdata_stage(line: &str) -> Option<Progress> {
    let (fraction, doing) = if line.starts_with("source: ") {
        (0.15, "Reading your disc")
    } else if line.starts_with("release: ") {
        (0.55, "Sorting the game's files")
    } else if line.starts_with("index: ") {
        (0.65, "Checking every file")
    } else if line.starts_with("verified ") {
        (0.75, "Preparing textures, models and sound")
    } else {
        return None;
    };
    Some(Progress {
        fraction,
        doing: doing.into(),
    })
}

/// `dc2data`'s stages (Kazarov `src/rust/progress.rs`, `Stage::title`) and what each is doing, in words for the player.
const DC2DATA_STAGES: [(&str, &str); 18] = [
    ("Read ISO", "Reading your disc"),
    ("Index", "Planning the game's files"),
    ("Plan assets", "Planning the game's files"),
    ("Textures", "Converting textures, sound, movies and models"),
    ("Audio", "Converting textures, sound, movies and models"),
    ("Movies", "Converting textures, sound, movies and models"),
    ("Models", "Converting textures, sound, movies and models"),
    ("Cleanup", "Converting the game's files"),
    ("Gameplay", "Converting the game's files"),
    ("Scripts", "Converting the game's files"),
    ("Packs", "Converting the game's files"),
    ("Other assets", "Converting the game's files"),
    ("Verify scripts", "Checking every file"),
    ("Verify trees", "Checking every file"),
    ("Write archive", "Writing the game's archive"),
    ("Verify archive", "Checking every file"),
    ("Clean staged data", "Finishing"),
    ("Publish", "Finishing"),
];

fn dc2data_words(stage: &str) -> &'static str {
    DC2DATA_STAGES
        .iter()
        .filter(|(title, _)| stage == *title || stage.starts_with(&format!("{title} ")))
        .max_by_key(|(title, _)| title.len())
        .map_or("Setting up", |(_, words)| words)
}

/// `dc2data` prints, when its output is not a terminal (or with `DC2DATA_PROGRESS=plain`),
/// `dc2data: [ 42%] Models 42% (…), Audio 10% (…) - 1:23 elapsed, ~2:10 left`.
fn dc2data_percent(line: &str) -> Option<Progress> {
    let rest = line.strip_prefix("dc2data: [")?;
    let (percent, rest) = rest.split_once("%]")?;
    let percent: f32 = percent.trim().parse().ok()?;
    let (stages, timing) = rest.trim_start().split_once(" - ")?;
    let mut doing = dc2data_words(stages.split(", ").next()?).to_string();
    if let Some(left) = timing.split(", ~").nth(1).and_then(|t| t.strip_suffix(" left")) {
        doing.push_str(&format!(", about {left} left"));
    }
    Some(Progress {
        fraction: (percent / 100.0).clamp(0.02, 0.99),
        doing,
    })
}

/// The progress a line of the extractor's output reports, if it reports any.
pub fn progress_of_line(kind: games::Progress, line: &str) -> Option<Progress> {
    match kind {
        games::Progress::Stages => dcdata_stage(line),
        games::Progress::Percent => dc2data_percent(line),
    }
}

/// The release the extractor names (`release: PAL prototype (July 12, 2001)`): its word is final, even for a disc the launcher's own
/// table does not know.
pub fn release_in_line(line: &str) -> Option<&str> {
    line.strip_prefix("release: ")
        .map(str::trim)
        .filter(|r| !r.is_empty())
}

fn marker(dirs: &GameDirs) -> PathBuf {
    dirs.data.with_file_name("setup.json")
}

/// Set up when the marker is there with the extractor's output: Dark Cloud's `data/` folder, or Dark Chronicle's
/// `data.zip` beside it (`dc2data` makes no `data/` folder).
pub fn read(dirs: &GameDirs) -> Option<SetUp> {
    if !dirs.data.is_dir() && !dirs.data.with_file_name("data.zip").is_file() {
        return None;
    }
    serde_json::from_slice(&std::fs::read(marker(dirs)).ok()?).ok()
}

/// `dcdata extract <disc> <data>`, or `dc2data extract <disc> <game root>/data`: the same shape. `dc2data` is told to
/// print its plain progress lines, which it does anyway when its output is not a terminal.
pub fn command(build: &Build, disc: &Path, dirs: &GameDirs) -> Command {
    let mut command = Command::new(&build.extractor);
    command
        .arg("extract")
        .arg(disc)
        .arg(&dirs.data)
        .current_dir(&build.dir);
    command.env("DC2DATA_PROGRESS", "plain");
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
    fn dcdata_waxes_by_stage() {
        let output = [
            "source: /d/Dark Cloud (PAL).iso (ISO 9660 image)",
            "DATA.DAT: 1 bytes, DATA.HD2: 2 bytes",
            "release: PAL prototype (July 12, 2001)",
            "index: 10 records, 9 files, 100 bytes",
            "wrote 9 files (100 bytes), kept 0 already present",
            "verified 9/9 files in /data",
        ];
        let progress: Vec<Progress> = output
            .iter()
            .filter_map(|l| progress_of_line(games::Progress::Stages, l))
            .collect();
        assert_eq!(progress.len(), 4);
        assert_eq!(progress[0].doing, "Reading your disc");
        assert!(progress.windows(2).all(|w| w[0].fraction < w[1].fraction));
        assert_eq!(
            output.iter().find_map(|l| release_in_line(l)),
            Some("PAL prototype (July 12, 2001)")
        );
    }

    #[test]
    fn dc2data_reports_its_percentage_and_time_left() {
        let read = progress_of_line(
            games::Progress::Percent,
            "dc2data: [  7%] Read ISO 40% (1.2 GiB/3.0 GiB) - 0:12 elapsed, estimating time left",
        )
        .unwrap();
        assert_eq!(
            read,
            Progress {
                fraction: 0.07,
                doing: "Reading your disc".into()
            }
        );
        let models = progress_of_line(
            games::Progress::Percent,
            "dc2data: [ 42%] Models 42% (120/500), Audio 10% (3/30) - 1:23 elapsed, ~2:10 left",
        )
        .unwrap();
        assert!((models.fraction - 0.42).abs() < 1e-6);
        assert_eq!(
            models.doing,
            "Converting textures, sound, movies and models, about 2:10 left"
        );
        let seal = progress_of_line(
            games::Progress::Percent,
            "dc2data: [ 97%] Write archive 1.4 GiB written - 4:01 elapsed, finishing publication",
        )
        .unwrap();
        assert_eq!(seal.doing, "Writing the game's archive");
        assert_eq!(
            progress_of_line(games::Progress::Percent, "dc2data: some warning"),
            None
        );
        assert_eq!(
            progress_of_line(games::Progress::Percent, "converted 9 reachable files"),
            None
        );
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
            release: Some("PAL prototype (July 12, 2001)".into()),
            build: "/b".into(),
            finished: 1,
        };
        finish(&dirs, &done).unwrap();
        assert_eq!(read(&dirs), Some(done));
        remove(&dirs).unwrap();
        assert_eq!(read(&dirs), None);
        assert!(!dirs.data.exists());
        assert!(!dirs.data.exists());
        assert!(dirs.save.join("saves/1").is_dir());
    }

    #[test]
    fn dark_chronicles_archive_counts_as_set_up() {
        let root = tempfile::tempdir().unwrap();
        let dirs = game_dirs(root.path(), "chronicle-two");
        std::fs::create_dir_all(&dirs.root).unwrap();
        let done = SetUp {
            disc: "/d.iso".into(),
            release: Some("PAL".into()),
            build: "/b".into(),
            finished: 1,
        };
        finish(&dirs, &done).unwrap();
        assert_eq!(read(&dirs), None, "a marker without data is not a set-up");
        std::fs::write(dirs.data.with_file_name("data.zip"), b"").unwrap();
        assert_eq!(read(&dirs), Some(done));
        remove(&dirs).unwrap();
        assert!(!dirs.data.with_file_name("data.zip").exists());
        assert_eq!(read(&dirs), None);
    }
}
