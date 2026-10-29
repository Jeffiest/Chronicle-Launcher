// The launcher's commands and events. In the app they go to Rust (src-tauri/src/lib.rs); in a plain browser
// (npm run dev) they are mocked here, so the pages can be worked on without a build of the app or a disc.

import type { Backup, Events, GameView, Save, SavesView, Scan, Stage } from "./types";
import { GAMES } from "./games";

const inApp = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

type Listener<K extends keyof Events> = (payload: Events[K]) => void;

export interface Api {
  library(): Promise<GameView[]>;
  /** Asks for a folder, then remembers it as the game's build. Null when the player cancelled. */
  chooseBuild(id: string): Promise<GameView | null>;
  forgetBuild(id: string): Promise<GameView>;
  /** Asks for a folder (starting from `from`) and looks in it for the game's disc. Null when cancelled. */
  pickDiscFolder(id: string, from: string | null): Promise<{ folder: string; scan: Scan } | null>;
  scanDiscs(id: string, folder: string): Promise<Scan>;
  setUp(id: string, disc: string): Promise<GameView>;
  cancelSetUp(id: string): Promise<void>;
  removeSetUp(id: string): Promise<GameView>;
  play(id: string): Promise<GameView>;
  saves(id: string): Promise<SavesView>;
  backUpSaves(id: string): Promise<Backup | null>;
  restoreSaves(id: string, backup: string): Promise<void>;
  crashReport(id: string): Promise<string>;
  /** Writes the report to Downloads and says where. */
  saveReport(id: string): Promise<string>;
  openFolder(id: string, which: "saves" | "logs" | "game"): Promise<void>;
  openLink(url: string): Promise<void>;
  on<K extends keyof Events>(event: K, listener: Listener<K>): Promise<() => void>;
}

async function pickFolder(title: string, from: string | null): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const folder = await open({ directory: true, title, defaultPath: from ?? undefined });
  return typeof folder === "string" ? folder : null;
}

const app: Api = {
  library: () => call("library"),
  async chooseBuild(id) {
    const folder = await pickFolder("Choose the folder with the game's build", null);
    return folder ? call("choose_build", { id, folder }) : null;
  },
  forgetBuild: (id) => call("forget_build", { id }),
  async pickDiscFolder(id, from) {
    const folder = await pickFolder("Choose the folder with your disc image", from);
    return folder ? { folder, scan: await call<Scan>("scan_discs", { id, folder }) } : null;
  },
  scanDiscs: (id, folder) => call("scan_discs", { id, folder }),
  setUp: (id, disc) => call("set_up", { id, disc }),
  cancelSetUp: (id) => call("cancel_set_up", { id }),
  removeSetUp: (id) => call("remove_set_up", { id }),
  play: (id) => call("play", { id }),
  saves: (id) => call("saves", { id }),
  backUpSaves: (id) => call("back_up_saves", { id }),
  restoreSaves: (id, backup) => call("restore_saves", { id, backup }),
  crashReport: (id) => call("crash_report", { id }),
  saveReport: (id) => call("save_report", { id }),
  openFolder: (id, which) => call("open_folder", { id, which }),
  async openLink(url) {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(url);
  },
  async on(event, listener) {
    const { listen } = await import("@tauri-apps/api/event");
    return listen(event, (e) => listener(e.payload as never));
  },
};

/** The browser's stand-in: a build and a disc for each game, a set-up that waxes the moon in a few seconds, and a
 * game that "crashes" the second time it is played. `tick` is how long a set-up stage lasts. */
export function mockApi(tick = 700): Api {
  const builds = new Set<string>();
  const setUp = new Map<string, { disc: string; release: string | null }>();
  const crashed = new Map<string, number>();
  const playing = new Set<string>();
  const stages = new Map<string, Stage>();
  const plays = new Map<string, number>();
  const listeners: { [K in keyof Events]?: Listener<K>[] } = {};
  const emit = <K extends keyof Events>(event: K, payload: Events[K]) =>
    (listeners[event] as Listener<K>[] | undefined)?.forEach((l) => l(payload));
  const fraction: Record<Stage, number> = {
    starting: 0.02,
    "reading-disc": 0.15,
    sorting: 0.55,
    checking: 0.65,
    preparing: 0.75,
    done: 1,
  };
  const saves: Save[] = [
    {
      name: "autosave",
      number: null,
      play_seconds: 9300,
      day: 12,
      map_no: 1,
      modified: 1793270000,
      readable: true,
    },
    { name: "1", number: 1, play_seconds: 8640, day: 11, map_no: 0, modified: 1793180000, readable: true },
  ];
  const backups: Backup[] = [];

  const view = (id: string): GameView => {
    const game = GAMES.find((g) => g.id === id)!;
    const build = builds.has(id);
    const done = setUp.get(id);
    const stage = stages.get(id);
    const when = crashed.get(id);
    return {
      game,
      next: !build ? "get-the-game" : done ? "play" : "set-up",
      build: build ? { dir: `~/${game.port}`, executable: game.executable, extractor: game.extractor } : null,
      build_problem: null,
      can_set_up: game.boot !== undefined,
      set_up: done ? { ...done, build: `~/${game.port}`, finished: 1793200000 } : null,
      disc_folder: build ? "~/Games/PS2" : null,
      last_crash:
        when === undefined
          ? null
          : {
              what: "It stopped on signal 11",
              when,
              log: "play.log",
              ended_code: null,
              ended_signal: 11,
            },
      activity: stage
        ? { kind: "setting-up", stage, fraction: fraction[stage] }
        : playing.has(id)
          ? { kind: "playing" }
          : { kind: "idle" },
      data: `~/.local/share/chronicle-launcher/games/${id}/data`,
      save: `~/.local/share/chronicle-launcher/games/${id}/save`,
    };
  };
  const scan = (id: string): Scan => {
    const game = GAMES.find((g) => g.id === id)!;
    return game.boot
      ? {
          discs: [{ path: `~/Games/PS2/${game.discs[0].file}`, release: game.releases?.[1]?.name ?? null }],
          skipped: [
            { path: "~/Games/PS2/Other Game.iso", why: "another game's disc (it boots SLUS_123.45)" },
          ],
        }
      : { discs: [], skipped: [] };
  };

  return {
    library: async () => GAMES.map((g) => view(g.id)),
    async chooseBuild(id) {
      builds.add(id);
      return view(id);
    },
    async forgetBuild(id) {
      builds.delete(id);
      return view(id);
    },
    pickDiscFolder: async (id) => ({ folder: "~/Games/PS2", scan: scan(id) }),
    scanDiscs: async (id) => scan(id),
    async setUp(id, disc) {
      const order: Stage[] = ["starting", "reading-disc", "sorting", "checking", "preparing"];
      order.forEach((stage, i) =>
        setTimeout(() => {
          if (!stages.has(id)) return;
          stages.set(id, stage);
          emit("setup-line", { id, line: `${stage}…` });
          emit("activity", view(id));
        }, i * tick),
      );
      setTimeout(() => {
        if (!stages.has(id)) return;
        stages.delete(id);
        setUp.set(id, { disc, release: scan(id).discs[0]?.release ?? null });
        emit("setup-finished", { id, ok: true, message: "Set up. Ready to play." });
        emit("activity", view(id));
      }, order.length * tick);
      stages.set(id, "starting");
      return view(id);
    },
    async cancelSetUp(id) {
      stages.delete(id);
      emit("setup-finished", { id, ok: false, message: "Set-up was stopped." });
      emit("activity", view(id));
    },
    async removeSetUp(id) {
      setUp.delete(id);
      return view(id);
    },
    async play(id) {
      playing.add(id);
      const n = (plays.get(id) ?? 0) + 1;
      plays.set(id, n);
      setTimeout(() => {
        playing.delete(id);
        if (n % 2 === 0) crashed.set(id, Math.floor(Date.now() / 1000));
        else crashed.delete(id);
        emit("game-ended", {
          id,
          outcome: n % 2 === 0 ? { kind: "crashed", what: "It stopped on signal 11" } : { kind: "closed" },
        });
        emit("activity", view(id));
      }, tick * 3);
      return view(id);
    },
    saves: async () => ({ saves, backups: [...backups] }),
    async backUpSaves() {
      const made = Math.floor(Date.now() / 1000);
      const backup = { id: String(made), made, saves: saves.length };
      backups.unshift(backup);
      return backup;
    },
    async restoreSaves() {},
    crashReport: async () =>
      "Dark Cloud (Chronicle), Chronicle Launcher 0.1.0\nThe game stopped unexpectedly. It stopped on signal 11.\n",
    saveReport: async (id) => `~/Downloads/${id}-report.txt`,
    async openFolder() {},
    async openLink(url) {
      window.open(url, "_blank", "noopener");
    },
    async on(event, listener) {
      const list = (listeners[event] ??= []) as Listener<typeof event>[];
      list.push(listener);
      return () => list.splice(list.indexOf(listener), 1);
    },
  };
}

export const api: Api = inApp ? app : mockApi();
