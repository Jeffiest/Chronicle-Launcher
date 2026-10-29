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

/// What an exit means for the player (Chronicle `port/src/app/exitcodes.hpp`, ChronicleTwo `port/src/exitcodes.hpp`).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Outcome {
    /// Closed, or quit from the game.
    Closed,
    /// The data is missing, holds nothing, or was rejected as made by an older port (Dark Chronicle's status 5): set up
    /// again.
    NoData,
    /// Stopped unexpectedly: the game's saves are kept, and a report can be copied. `what` is the cause, as a
    /// sentence about the game ("It stopped on signal 11").
    Crashed { what: String },
}

pub fn outcome(ended: &Ended) -> Outcome {
    let what = match (ended.code, ended.signal) {
        (Some(0), _) => return Outcome::Closed,
        (Some(3) | Some(5), _) => return Outcome::NoData,
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

/// The game's own crash report, when it writes one: Chronicle Two keeps the newest bundle's folder in
/// `<save>/crashes/last_crash.json`, with `report.txt` in it, personal paths already removed (`docs/CRASH_REPORTS.md`).
/// Bundles are dated by their folder names, so the newest is also found without that file.
pub fn game_crash_report(save: &Path, since: u64) -> Option<String> {
    let crashes = save.join("crashes");
    let mut newest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    for entry in std::fs::read_dir(&crashes).ok()?.flatten() {
        let report = entry.path().join("report.txt");
        let Ok(modified) = std::fs::metadata(&report).and_then(|m| m.modified()) else {
            continue;
        };
        if newest.as_ref().is_none_or(|(at, _)| modified > *at) {
            newest = Some((modified, report));
        }
    }
    let (modified, report) = newest?;
    let at = modified.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    if at + 60 < since {
        return None;
    }
    let text = std::fs::read_to_string(report).ok()?;
    Some(text.chars().take(64 * 1024).collect())
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
        assert_eq!(outcome(&ended(Some(5), None)), Outcome::NoData);
        assert!(matches!(outcome(&ended(Some(4), None)), Outcome::Crashed { .. }));
        let Outcome::Crashed { what } = outcome(&ended(None, Some(11))) else {
            panic!()
        };
        assert!(what.contains("signal 11"));
    }

    #[test]
    fn finds_the_games_own_crash_report() {
        let save = tempfile::tempdir().unwrap();
        assert_eq!(game_crash_report(save.path(), 0), None);
        for (session, text) in [("20261018-132124-6c54", "old"), ("20261029-101500-aaaa", "new")] {
            let bundle = save.path().join("crashes").join(session);
            std::fs::create_dir_all(&bundle).unwrap();
            std::fs::write(bundle.join("report.txt"), text).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let now = crate::saves::now();
        assert_eq!(game_crash_report(save.path(), now).as_deref(), Some("new"));
        assert_eq!(
            game_crash_report(save.path(), now + 3600),
            None,
            "a report from before this crash is not its own"
        );
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
