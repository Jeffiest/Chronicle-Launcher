//! The games, read from `games/*.json` at the repository root, which the UI reads too.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Disc {
    pub region: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub chapter: u32,
    pub port: String,
    pub repository: String,
    pub get_url: String,
    pub executable: String,
    pub extractor: String,
    pub hero: String,
    pub discs: Vec<Disc>,
}

const SOURCES: [&str; 2] = [
    include_str!("../../games/chronicle.json"),
    include_str!("../../games/chronicle-two.json"),
];

/// Every game, in chapter order.
pub fn all() -> &'static [Game] {
    static GAMES: OnceLock<Vec<Game>> = OnceLock::new();
    GAMES.get_or_init(|| {
        let mut games: Vec<Game> = SOURCES
            .iter()
            .map(|source| serde_json::from_str(source).expect("games/*.json is checked by the tests"))
            .collect();
        games.sort_by_key(|game| game.chapter);
        games
    })
}

pub fn find(id: &str) -> Option<&'static Game> {
    all().iter().find(|game| game.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_chapters_in_order() {
        let ids: Vec<&str> = all().iter().map(|game| game.id.as_str()).collect();
        assert_eq!(ids, ["chronicle", "chronicle-two"]);
    }

    #[test]
    fn ids_are_moonmods_game_names() {
        // MoonMod's manifest.schema.json: "games" items are "chronicle" or "chronicle-two".
        for game in all() {
            assert!(
                ["chronicle", "chronicle-two"].contains(&game.id.as_str()),
                "{}",
                game.id
            );
        }
    }

    #[test]
    fn links_go_to_github() {
        for game in all() {
            assert!(
                game.get_url.starts_with("https://github.com/"),
                "{}",
                game.get_url
            );
            assert!(
                game.repository.starts_with("https://github.com/"),
                "{}",
                game.repository
            );
        }
    }
}
