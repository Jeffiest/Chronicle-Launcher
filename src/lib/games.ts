// The games, from games/*.json: the same files the Rust core reads.

import chronicle from "../../games/chronicle.json";
import chronicleTwo from "../../games/chronicle-two.json";
import type { Game } from "./types";

// JSON imports widen string unions; the core's tests (core/src/games.rs) hold the files to the exact values.
export const GAMES: Game[] = ([chronicle, chronicleTwo] as Game[]).sort((a, b) => a.chapter - b.chapter);

const ROMAN = ["", "I", "II", "III", "IV", "V"];

/** "Chapter I", "Chapter II": how the Chronicle names a game. */
export function chapterName(chapter: number): string {
  return `Chapter ${ROMAN[chapter] ?? chapter}`;
}
