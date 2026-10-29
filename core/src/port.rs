//! Finding a port build in the folder the player chose: the folder itself (an unpacked build), or a checkout of
//! the port with a build in it. Release builds come before debug ones.

use crate::games::Game;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Where the ports' CMake presets put their builds, relative to a checkout (both ports use the same names).
const CHECKOUT_BUILDS: [&str; 6] = [
    "port/build/windows-x64-release",
    "port/build/pc-release",
    "port/build/macos-arm64-release",
    "port/build/windows-x64",
    "port/build/pc",
    "port/build/macos-arm64",
];

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Build {
    /// The folder holding the executable and the extractor.
    pub dir: PathBuf,
    pub executable: PathBuf,
    pub extractor: PathBuf,
}

/// Why a chosen folder is not a build, in words for the player.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum NotABuild {
    Missing { folder: PathBuf },
    NoProgram { program: String },
    NoExtractor { extractor: String },
}

impl std::fmt::Display for NotABuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { folder } => write!(f, "{} is not there any more", folder.display()),
            Self::NoProgram { program } => write!(f, "No {program} in this folder or a build inside it"),
            Self::NoExtractor { extractor } => write!(f, "The build has no {extractor} beside it"),
        }
    }
}

fn program(dir: &Path, name: &str) -> PathBuf {
    if cfg!(windows) {
        dir.join(format!("{name}.exe"))
    } else {
        dir.join(name)
    }
}

pub fn find(game: &Game, folder: &Path) -> Result<Build, NotABuild> {
    if !folder.is_dir() {
        return Err(NotABuild::Missing {
            folder: folder.to_path_buf(),
        });
    }
    let candidates =
        std::iter::once(folder.to_path_buf()).chain(CHECKOUT_BUILDS.iter().map(|b| folder.join(b)));
    for dir in candidates {
        let executable = program(&dir, &game.executable);
        if !executable.is_file() {
            continue;
        }
        let extractor = program(&dir, &game.extractor);
        if !extractor.is_file() {
            return Err(NotABuild::NoExtractor {
                extractor: game.extractor.clone(),
            });
        }
        return Ok(Build {
            dir,
            executable,
            extractor,
        });
    }
    Err(NotABuild::NoProgram {
        program: game.executable.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"").unwrap();
    }

    fn dark_cloud() -> &'static Game {
        games::find("chronicle").unwrap()
    }

    #[test]
    fn an_unpacked_build() {
        let dir = tempfile::tempdir().unwrap();
        touch(&program(dir.path(), "darkcloud"));
        touch(&program(dir.path(), "dcdata"));
        assert_eq!(find(dark_cloud(), dir.path()).unwrap().dir, dir.path());
    }

    #[test]
    fn a_checkout_prefers_release() {
        let dir = tempfile::tempdir().unwrap();
        for build in ["port/build/pc", "port/build/pc-release"] {
            touch(&program(&dir.path().join(build), "darkcloud"));
            touch(&program(&dir.path().join(build), "dcdata"));
        }
        assert_eq!(
            find(dark_cloud(), dir.path()).unwrap().dir,
            dir.path().join("port/build/pc-release")
        );
    }

    #[test]
    fn says_what_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            find(dark_cloud(), dir.path()),
            Err(NotABuild::NoProgram {
                program: "darkcloud".into()
            })
        );
        touch(&program(dir.path(), "darkcloud"));
        assert_eq!(
            find(dark_cloud(), dir.path()),
            Err(NotABuild::NoExtractor {
                extractor: "dcdata".into()
            })
        );
    }

    #[test]
    fn the_other_games_program_is_not_this_one() {
        let dir = tempfile::tempdir().unwrap();
        touch(&program(dir.path(), "darkcloud"));
        touch(&program(dir.path(), "dcdata"));
        assert!(find(games::find("chronicle-two").unwrap(), dir.path()).is_err());
    }
}
