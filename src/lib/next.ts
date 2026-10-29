// What a game's page says about where it stands, and what its main button does.

import type { GameView } from "./types";

export interface State {
  words: string;
  tone: "ok" | "warn" | "err" | "";
}

export function stateOf(view: GameView): State {
  if (view.build_problem) return { words: "Its build has moved", tone: "err" };
  switch (view.next) {
    case "get-the-game":
      return { words: "Needs the game", tone: "warn" };
    case "set-up":
      return { words: "Needs your disc", tone: "warn" };
    case "play":
      return { words: "Ready to play", tone: "ok" };
  }
}

export const MAIN_BUTTON: Record<GameView["next"], string> = {
  "get-the-game": "Get the game",
  "set-up": "Set up from your disc",
  play: "Play",
};
