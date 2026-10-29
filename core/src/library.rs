//! Each game's state, and the one next step its page offers.

use crate::{config, games, paths, port};
use serde::Serialize;
use std::io;
use std::path::{Path, PathBuf};

/// The main button, in order: no build → Get the game; a build but no data → Set up; both → Play.
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Next {
    GetTheGame,
    SetUp,
    Play,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GameView {
    pub game: games::Game,
    pub next: Next,
    pub build: Option<port::Build>,
    /// Why the chosen build folder no longer works, when it does not.
    pub build_problem: Option<String>,
    pub data: PathBuf,
    pub save: PathBuf,
}

/// Set up is done when the extractor's output is there: Chronicle's `data.zip` or `data/`, Chronicle Two's
/// `data.json` marker beside them. The check that it is current comes with set-up itself.
fn has_data(game: &games::Game, data: &Path) -> bool {
    let parent = data.parent().unwrap_or(data);
    match game.id.as_str() {
        "chronicle-two" => parent.join("data.json").is_file(),
        _ => parent.join("data.zip").is_file() || data.join("languages.json").is_file(),
    }
}

pub fn view(root: &Path, config: &config::Config, game: &games::Game) -> GameView {
    let dirs = paths::game_dirs(root, &game.id);
    let chosen = config.games.get(&game.id).and_then(|c| c.build.as_deref());
    let (build, build_problem) = match chosen.map(|folder| port::find(game, folder)) {
        Some(Ok(build)) => (Some(build), None),
        Some(Err(problem)) => (None, Some(problem.to_string())),
        None => (None, None),
    };
    let next = match (&build, has_data(game, &dirs.data)) {
        (None, _) => Next::GetTheGame,
        (Some(_), false) => Next::SetUp,
        (Some(_), true) => Next::Play,
    };
    GameView {
        game: game.clone(),
        next,
        build,
        build_problem,
        data: dirs.data,
        save: dirs.save,
    }
}

pub fn library(root: &Path) -> io::Result<Vec<GameView>> {
    let config = config::load(root)?;
    Ok(games::all()
        .iter()
        .map(|game| view(root, &config, game))
        .collect())
}

/// Remembers the folder only when it holds a build, so a wrong pick changes nothing.
pub fn choose_build(root: &Path, id: &str, folder: &Path) -> Result<GameView, String> {
    let game = games::find(id).ok_or_else(|| format!("No game called {id}"))?;
    port::find(game, folder).map_err(|problem| problem.to_string())?;
    let mut config = config::load(root).map_err(|e| e.to_string())?;
    config.games.entry(id.to_string()).or_default().build = Some(folder.to_path_buf());
    config::save(root, &config).map_err(|e| e.to_string())?;
    Ok(view(root, &config, game))
}

/// Forgets the build folder; never touches the folder, the data or the saves.
pub fn forget_build(root: &Path, id: &str) -> Result<GameView, String> {
    let game = games::find(id).ok_or_else(|| format!("No game called {id}"))?;
    let mut config = config::load(root).map_err(|e| e.to_string())?;
    if let Some(entry) = config.games.get_mut(id) {
        entry.build = None;
    }
    config::save(root, &config).map_err(|e| e.to_string())?;
    Ok(view(root, &config, game))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_build(dir: &Path, executable: &str, extractor: &str) {
        let name = |n: &str| {
            if cfg!(windows) {
                format!("{n}.exe")
            } else {
                n.to_string()
            }
        };
        std::fs::write(dir.join(name(executable)), b"").unwrap();
        std::fs::write(dir.join(name(extractor)), b"").unwrap();
    }

    #[test]
    fn a_new_launcher_offers_get_the_game() {
        let root = tempfile::tempdir().unwrap();
        let views = library(root.path()).unwrap();
        assert!(views
            .iter()
            .all(|v| v.next == Next::GetTheGame && v.build_problem.is_none()));
    }

    #[test]
    fn get_the_game_then_set_up_then_play() {
        let root = tempfile::tempdir().unwrap();
        let build = tempfile::tempdir().unwrap();
        fake_build(build.path(), "darkchronicle", "dc2data");
        let view = choose_build(root.path(), "chronicle-two", build.path()).unwrap();
        assert_eq!(view.next, Next::SetUp);
        std::fs::create_dir_all(view.data.parent().unwrap()).unwrap();
        std::fs::write(view.data.parent().unwrap().join("data.json"), b"{}").unwrap();
        let views = library(root.path()).unwrap();
        assert_eq!(views[1].next, Next::Play);
        assert_eq!(views[0].next, Next::GetTheGame);
    }

    #[test]
    fn a_wrong_folder_is_not_remembered() {
        let root = tempfile::tempdir().unwrap();
        let empty = tempfile::tempdir().unwrap();
        assert!(choose_build(root.path(), "chronicle", empty.path()).is_err());
        assert!(!root.path().join("config.json").exists());
    }

    #[test]
    fn a_build_that_went_away_says_so() {
        let root = tempfile::tempdir().unwrap();
        let build = tempfile::tempdir().unwrap();
        fake_build(build.path(), "darkcloud", "dcdata");
        choose_build(root.path(), "chronicle", build.path()).unwrap();
        let gone = build.path().to_path_buf();
        drop(build);
        let view = &library(root.path()).unwrap()[0];
        assert_eq!(view.next, Next::GetTheGame);
        assert!(view
            .build_problem
            .as_deref()
            .unwrap()
            .contains(&gone.display().to_string()));
    }

    #[test]
    fn forgetting_keeps_the_folder() {
        let root = tempfile::tempdir().unwrap();
        let build = tempfile::tempdir().unwrap();
        fake_build(build.path(), "darkcloud", "dcdata");
        choose_build(root.path(), "chronicle", build.path()).unwrap();
        assert_eq!(
            forget_build(root.path(), "chronicle").unwrap().next,
            Next::GetTheGame
        );
        assert!(build
            .path()
            .join(if cfg!(windows) {
                "darkcloud.exe"
            } else {
                "darkcloud"
            })
            .exists());
    }
}
