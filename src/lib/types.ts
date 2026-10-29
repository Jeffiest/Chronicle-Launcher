// The shapes the Rust core sends (core/src/games.rs, core/src/port.rs, core/src/library.rs). Keep them in step.

export interface Disc {
  region: string;
  file: string;
}

export interface Release {
  name: string;
  index_size: number;
  index_fnv: string;
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
  boot?: string;
  index_file?: string;
  releases?: Release[];
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
  can_set_up: boolean;
  set_up: SetUp | null;
  disc_folder: string | null;
  last_crash: Crash | null;
  activity: Activity;
  data: string;
  save: string;
}

export type Stage = "starting" | "reading-disc" | "sorting" | "checking" | "preparing" | "done";

export type Activity =
  { kind: "idle" } | { kind: "setting-up"; stage: Stage; fraction: number } | { kind: "playing" };

export interface SetUp {
  disc: string;
  release: string | null;
  build: string;
  finished: number;
}

export interface Crash {
  what: string;
  when: number;
  log: string;
  ended_code: number | null;
  ended_signal: number | null;
}

export interface FoundDisc {
  path: string;
  release: string | null;
}

export interface Scan {
  discs: FoundDisc[];
  skipped: { path: string; why: string }[];
}

export interface Save {
  name: string;
  number: number | null;
  play_seconds: number | null;
  day: number | null;
  map_no: number | null;
  modified: number | null;
  readable: boolean;
}

export interface Backup {
  id: string;
  made: number;
  saves: number;
}

export interface SavesView {
  saves: Save[];
  backups: Backup[];
}

export type Outcome = { kind: "closed" } | { kind: "no-data" } | { kind: "crashed"; what: string };

export interface Events {
  activity: GameView;
  "setup-line": { id: string; line: string };
  "setup-finished": { id: string; ok: boolean; message: string };
  "game-ended": { id: string; outcome: Outcome };
}
