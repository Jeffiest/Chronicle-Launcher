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

/** The game's own window a chapter wears (app.css `.stone`, `.paper`): Dark Cloud's Blue Stone menus, Dark
 * Chronicle's parchment. */
export function material(chapter: number): "stone" | "paper" {
  return chapter === 1 ? "stone" : "paper";
}
