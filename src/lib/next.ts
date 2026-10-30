// What a game's page says about where it stands, and what its main button does.

import type { GameView } from "./types";

export interface State {
  words: string;
  tone: "ok" | "warn" | "err" | "";
}

export function stateOf(view: GameView): State {
  if (view.activity.kind === "setting-up") return { words: "Setting up", tone: "warn" };
  if (view.activity.kind === "playing") return { words: "Playing", tone: "ok" };
  if (view.activity.kind === "rendering") return { words: "Taking a picture of the game", tone: "warn" };
  if (view.build_problem) return { words: "Its build has moved", tone: "err" };
  switch (view.next) {
    case "get-the-game":
      return { words: "Needs the game", tone: "warn" };
    case "set-up":
      return { words: "Needs your disc", tone: "warn" };
    case "play":
      return view.last_crash
        ? { words: "Stopped unexpectedly last time", tone: "err" }
        : { words: "Ready to play", tone: "ok" };
  }
}

/** The main button's words, which always name the next step. */
export function mainButton(view: GameView): string {
  if (view.activity.kind === "setting-up") return "Setting up…";
  if (view.activity.kind === "playing") return "Playing";
  if (view.activity.kind === "rendering") return "Play";
  switch (view.next) {
    case "get-the-game":
      return "Get the game";
    case "set-up":
      return "Set up from your disc";
    case "play":
      return "Play";
  }
}
