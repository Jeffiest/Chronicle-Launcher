// The shapes the Rust core sends (core/src/games.rs, core/src/port.rs, core/src/library.rs). Keep them in step.

export interface Disc {
  region: string;
  file: string;
}

export interface Game {
  id: string;
  title: string;
  chapter: number;
  port: string;
  repository: string;
  get_url: string;
  executable: string;
  extractor: string;
  hero: string;
  discs: Disc[];
}

export type Next = "get-the-game" | "set-up" | "play";

export interface Build {
  dir: string;
  executable: string;
  extractor: string;
}

export interface GameView {
  game: Game;
  next: Next;
  build: Build | null;
  build_problem: string | null;
  data: string;
  save: string;
}
