//! Playing: the port started on the launcher's data and saves, and what its exit means.

use crate::games::Game;
use crate::paths::GameDirs;
use crate::port::Build;
use crate::process::Ended;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

/// `darkcloud --data <data> --save <save>`, from the game's folder, so nothing it writes beside itself lands in the
/// build. With `mods_on` false, `MOONMOD_NO_MODS=1` starts every mod off for this run, keeping the player's choices.
pub fn command(build: &Build, dirs: &GameDirs, mods_on: bool) -> Command {
    let mut command = Command::new(&build.executable);
    command
        .arg("--data")
        .arg(&dirs.data)
        .arg("--save")
        .arg(&dirs.save);
    if mods_on {
        command.env_remove("MOONMOD_NO_MODS");
    } else {
        command.env("MOONMOD_NO_MODS", "1");
    }
    if let Some(folder) = dirs.data.parent() {
        command.current_dir(folder);
    }
    command
}

/// What an exit means for the player (Chronicle `port/src/app/exitcodes.hpp`).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Outcome {
    /// Closed, or quit from the game.
    Closed,
    /// The data is missing or holds nothing: set up again.
    NoData,
    /// Stopped unexpectedly: the game's saves are kept, and a report can be copied. `what` is the cause, as a
    /// sentence about the game ("It stopped on signal 11").
    Crashed { what: String },
}

pub fn outcome(ended: &Ended) -> Outcome {
    let what = match (ended.code, ended.signal) {
        (Some(0), _) => return Outcome::Closed,
        (Some(3), _) => return Outcome::NoData,
        (Some(1), _) => "Its window or graphics could not start".to_string(),
        (Some(2), _) => "It did not accept how the launcher started it".to_string(),
        (Some(4), _) => "It stopped on one of its own checks".to_string(),
        (_, Some(signal)) => format!("It stopped on signal {signal}"),
        (Some(code), _) => format!("It stopped with exit code {code}"),
        (None, None) => "It was stopped".to_string(),
    };
    Outcome::Crashed { what }
}

/// The last time the game stopped unexpectedly, kept in `<game>/logs/last-crash.json` until the next clean exit, so
/// the page still says so after the launcher restarts.
#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq)]
pub struct Crash {
    pub what: String,
    /// Seconds since 1970.
    pub when: u64,
    /// The run's log, beside this file.
    pub log: String,
    pub ended_code: Option<i32>,
    pub ended_signal: Option<i32>,
}

fn crash_file(logs: &Path) -> std::path::PathBuf {
    logs.join("last-crash.json")
}

pub fn record_crash(logs: &Path, crash: &Crash) -> std::io::Result<()> {
    std::fs::create_dir_all(logs)?;
    std::fs::write(crash_file(logs), serde_json::to_vec_pretty(crash)?)
}

pub fn last_crash(logs: &Path) -> Option<Crash> {
    serde_json::from_slice(&std::fs::read(crash_file(logs)).ok()?).ok()
}

pub fn clear_crash(logs: &Path) {
    let _ = std::fs::remove_file(crash_file(logs));
}

/// Text for a GitHub issue or Discord: what ran and how it ended, and the end of its log, with the home folder
/// written `~` so nothing personal goes in.
pub fn report(game: &Game, build: &Path, ended: &Ended, log: &str, home: Option<&Path>) -> String {
    const LINES: usize = 150;
    let shorten = |text: &str| match home.and_then(|h| h.to_str()).filter(|h| h.len() > 1) {
        Some(home) => text.replace(home, "~"),
        None => text.to_string(),
    };
    let lines: Vec<&str> = log.lines().collect();
    let tail = &lines[lines.len().saturating_sub(LINES)..];
    let how = match outcome(ended) {
        Outcome::Crashed { what } => format!("The game stopped unexpectedly. {what}."),
        Outcome::NoData => "The game found no data".into(),
        Outcome::Closed => "The game closed".into(),
    };
    shorten(&format!(
        "{title} ({port}), Chronicle Launcher {version}\nBuild: {build}\nSystem: {os} {arch}\n{how}\n\nLast {n} lines of its log:\n```\n{tail}\n```\n",
        title = game.title,
        port = game.port,
        version = env!("CARGO_PKG_VERSION"),
        build = build.display(),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        n = tail.len(),
        tail = tail.join("\n"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games;

    fn ended(code: Option<i32>, signal: Option<i32>) -> Ended {
        Ended { code, signal }
    }

    #[test]
    fn exits_mean_what_the_port_says() {
        assert_eq!(outcome(&ended(Some(0), None)), Outcome::Closed);
        assert_eq!(outcome(&ended(Some(3), None)), Outcome::NoData);
        assert!(matches!(outcome(&ended(Some(4), None)), Outcome::Crashed { .. }));
        let Outcome::Crashed { what } = outcome(&ended(None, Some(11))) else {
            panic!()
        };
        assert!(what.contains("signal 11"));
    }

    #[test]
    fn a_crash_is_remembered_until_cleared() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(last_crash(dir.path()), None);
        let crash = Crash {
            what: "x".into(),
            when: 5,
            log: "play-5.log".into(),
            ended_code: None,
            ended_signal: Some(11),
        };
        record_crash(dir.path(), &crash).unwrap();
        assert_eq!(last_crash(dir.path()), Some(crash));
        clear_crash(dir.path());
        assert_eq!(last_crash(dir.path()), None);
    }

    #[test]
    fn a_report_hides_the_home_folder_and_keeps_the_end() {
        let log: String = (0..400).map(|i| format!("line {i} /home/toan/games\n")).collect();
        let text = report(
            games::find("chronicle").unwrap(),
            Path::new("/home/toan/Chronicle"),
            &ended(None, Some(6)),
            &log,
            Some(Path::new("/home/toan")),
        );
        assert!(!text.contains("/home/toan"));
        assert!(text.contains("Build: ~/Chronicle"));
        assert!(text.contains("line 399 ~/games"));
        assert!(!text.contains("line 249 "));
        assert!(text.contains("signal 6"));
    }
}
