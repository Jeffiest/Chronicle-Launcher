// The shapes the Rust core sends (core/src/games.rs, core/src/port.rs, core/src/library.rs). Keep them in step.

export interface Disc {
  region: string;
  file: string;
}

export interface Release {
  name: string;
  index_size: number;
  index_fnv: string;
  refused?: string;
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
  refused_boots?: Record<string, string>;
  progress?: "stages" | "percent";
  saves?: "dark-cloud" | "dark-chronicle";
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
  mods_on: boolean;
  mods_dir: string;
  activity: Activity;
  data: string;
  save: string;
}

export type Activity =
  | { kind: "idle" }
  | { kind: "setting-up"; fraction: number; doing: string }
  | { kind: "playing" }
  | { kind: "rendering" };

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
  "scene-ready": { id: string; ok: boolean; message: string; quiet: boolean };
}

export interface ModSetting {
  id: string;
  name: string;
  description: string | null;
  type: "boolean" | "integer" | "number" | "string" | "choice";
  default: unknown;
  min: number | null;
  max: number | null;
  step: number | null;
  choices: string[];
  value: unknown;
}

export interface Mod {
  folder: string;
  id: string;
  name: string;
  version: string | null;
  author: string | null;
  description: string | null;
  permissions: string[];
  dependencies: string[];
  settings: ModSetting[];
  enabled: boolean;
  native: boolean;
  native_trusted: boolean;
  problem: string | null;
}

export interface ModManifest {
  id: string;
  name: string;
  version: string;
  author: string | null;
  description: string | null;
  games: string[];
  permissions: string[];
  native_extension: { library: string } | null;
}

export interface ModPreview {
  manifest: ModManifest;
  replaces: boolean;
  for_this_game: boolean;
}
