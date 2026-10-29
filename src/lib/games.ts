// The games, from games/*.json: the same files the Rust core reads.

import chronicle from "../../games/chronicle.json";
import chronicleTwo from "../../games/chronicle-two.json";
import type { Game } from "./types";

export const GAMES: Game[] = [chronicle, chronicleTwo].sort((a, b) => a.chapter - b.chapter);

const ROMAN = ["", "I", "II", "III", "IV", "V"];

/** "Chapter I", "Chapter II": how the Chronicle names a game. */
export function chapterName(chapter: number): string {
  return `Chapter ${ROMAN[chapter] ?? chapter}`;
}

/** The theme class a game's page and cover wear (app.css: --dc1-*, --dc2-*). */
export function themeClass(chapter: number): string {
  return `dc${chapter}`;
}
