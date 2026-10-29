//! The launcher's settings, `<root>/config.json`. Unknown keys are kept, so an older launcher does not drop what
//! a newer one wrote.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GameConfig {
    /// The folder the player chose that holds the port's executable and extractor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<PathBuf>,
    /// The folder the player last looked for discs in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disc_folder: Option<PathBuf>,
    #[serde(flatten)]
    pub other: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub games: BTreeMap<String, GameConfig>,
    #[serde(flatten)]
    pub other: BTreeMap<String, serde_json::Value>,
}

fn file(root: &Path) -> PathBuf {
    root.join("config.json")
}

/// A missing file is the defaults; an unreadable one is an error, so it is never overwritten unseen.
pub fn load(root: &Path) -> io::Result<Config> {
    match std::fs::read(file(root)) {
        Ok(bytes) => {
            serde_json::from_slice(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(e),
    }
}

/// Written beside and renamed over, so a crash never leaves half a file.
pub fn save(root: &Path, config: &Config) -> io::Result<()> {
    std::fs::create_dir_all(root)?;
    let temporary = root.join("config.json.new");
    std::fs::write(&temporary, serde_json::to_vec_pretty(config)?)?;
    std::fs::rename(temporary, file(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_is_default() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()).unwrap(), Config::default());
    }

    #[test]
    fn round_trip_keeps_unknown_keys() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.json"),
            r#"{"future": 1, "games": {"chronicle": {"build": "/b", "later": true}}}"#,
        )
        .unwrap();
        let config = load(dir.path()).unwrap();
        save(dir.path(), &config).unwrap();
        let again: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.path().join("config.json")).unwrap()).unwrap();
        assert_eq!(again["future"], 1);
        assert_eq!(again["games"]["chronicle"]["build"], "/b");
        assert_eq!(again["games"]["chronicle"]["later"], true);
    }

    #[test]
    fn broken_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.json"), "{").unwrap();
        assert!(load(dir.path()).is_err());
    }
}
