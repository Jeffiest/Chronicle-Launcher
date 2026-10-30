// The launcher's commands and events. In the app they go to Rust (src-tauri/src/lib.rs); in a plain browser
// (npm run dev) they are mocked here, so the pages can be worked on without a build of the app or a disc.

import type { Backup, Events, GameView, Mod, ModPreview, Save, SavesView, Scan } from "./types";
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
  openFolder(id: string, which: "saves" | "logs" | "game" | "mods"): Promise<void>;
  mods(id: string): Promise<Mod[]>;
  setModEnabled(id: string, mod: string, on: boolean): Promise<void>;
  /** A value for a setting, or null to put it back to its default. */
  setModSetting(id: string, mod: string, setting: string, value: unknown): Promise<void>;
  setModsOn(id: string, on: boolean): Promise<GameView>;
  /** Asks for a mod's `.zip` (or, with `folder`, its folder), then unpacks and checks it. Null when cancelled. */
  pickMod(id: string, folder: boolean): Promise<ModPreview | null>;
  installMod(id: string): Promise<string>;
  discardMod(id: string): Promise<void>;
  removeMod(id: string, folder: string): Promise<void>;
  /** The page's backdrop (a `data:` URL), or null before there is one. */
  sceneImage(id: string): Promise<string | null>;
  /** Renders a new backdrop with the port; `scene-ready` says how it went. */
  renderScene(id: string): Promise<void>;
  /** Asks for a picture and makes it the backdrop. False when cancelled. */
  pickScene(id: string): Promise<boolean>;
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
  mods: async (id) => (await call<{ mods: Mod[] }>("mods_list", { id })).mods,
  setModEnabled: (id, mod, on) => call("set_mod_enabled", { id, modId: mod, on }),
  setModSetting: (id, mod, setting, value) =>
    call("set_mod_setting", { id, modId: mod, setting, value: value ?? null }),
  setModsOn: (id, on) => call("set_mods_on", { id, on }),
  async pickMod(id, folder) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const source = await open(
      folder
        ? { directory: true, title: "Choose the mod's folder" }
        : { title: "Choose the mod's .zip", filters: [{ name: "Mod", extensions: ["zip"] }] },
    );
    return typeof source === "string" ? call("preview_mod", { id, source }) : null;
  },
  installMod: (id) => call("install_mod", { id }),
  discardMod: (id) => call("discard_mod", { id }),
  removeMod: (id, folder) => call("remove_mod", { id, folder }),
  sceneImage: (id) => call("scene_image", { id }),
  renderScene: (id) => call("render_scene", { id }),
  async pickScene(id) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const picture = await open({
      title: "Choose a screenshot of the game",
      filters: [{ name: "Picture", extensions: ["png", "jpg", "jpeg", "webp"] }],
    });
    if (typeof picture !== "string") return false;
    await call("use_scene", { id, picture });
    return true;
  },
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
  const stages = new Map<string, number>();
  const plays = new Map<string, number>();
  const listeners: { [K in keyof Events]?: Listener<K>[] } = {};
  const emit = <K extends keyof Events>(event: K, payload: Events[K]) =>
    (listeners[event] as Listener<K>[] | undefined)?.forEach((l) => l(payload));
  // What each game's extractor reports as it goes: dcdata by stage, dc2data by percentage with the time left.
  const steps: Record<string, { fraction: number; doing: string }[]> = {
    chronicle: [
      { fraction: 0.02, doing: "Starting the extractor" },
      { fraction: 0.15, doing: "Reading your disc" },
      { fraction: 0.55, doing: "Sorting the game's files" },
      { fraction: 0.65, doing: "Checking every file" },
      { fraction: 0.75, doing: "Preparing textures, models and sound" },
    ],
    "chronicle-two": [
      { fraction: 0.02, doing: "Starting the extractor" },
      { fraction: 0.07, doing: "Reading your disc" },
      { fraction: 0.42, doing: "Converting textures, sound, movies and models, about 2:10 left" },
      { fraction: 0.81, doing: "Checking every file, about 0:40 left" },
      { fraction: 0.97, doing: "Writing the game's archive" },
    ],
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
  const modsOn = new Set<string>();
  // Two of MoonMod's own bundled mods, as their manifests describe them, and one that is broken.
  const mods: Mod[] = [
    {
      folder: "moonmod.rpg",
      id: "moonmod.rpg",
      name: "RPG: Character Paths",
      version: "1.4.0",
      author: "MoonMod",
      description:
        "Kills give XP, levels give HP, damage, toughness and skill points; a skill tree per character.",
      permissions: ["storage.read", "storage.write", "game.read", "game.write"],
      dependencies: [],
      settings: [
        {
          id: "xp_rate",
          name: "XP rate",
          description: "How fast characters level.",
          type: "number",
          default: 1,
          min: 0.5,
          max: 3,
          step: 0.1,
          choices: [],
          value: 1.5,
        },
        {
          id: "show_levelups",
          name: "Show level-ups",
          description: null,
          type: "boolean",
          default: true,
          min: null,
          max: null,
          step: null,
          choices: [],
          value: true,
        },
        {
          id: "difficulty",
          name: "Difficulty",
          description: null,
          type: "choice",
          default: "normal",
          min: null,
          max: null,
          step: null,
          choices: ["easy", "normal", "hard"],
          value: "normal",
        },
      ],
      enabled: true,
      native: false,
      native_trusted: false,
      problem: null,
    },
    {
      folder: "moonmod.controls",
      id: "moonmod.controls",
      name: "Controls",
      version: "1.1.0",
      author: "MoonMod",
      description: "Dungeon quick-item hotkeys, party shortcuts and a first-person toggle.",
      permissions: ["game.read", "game.write", "native.extension"],
      dependencies: [],
      settings: [],
      enabled: false,
      native: true,
      native_trusted: false,
      problem: null,
    },
    {
      folder: "half-finished",
      id: "half-finished",
      name: "half-finished",
      version: null,
      author: null,
      description: null,
      permissions: [],
      dependencies: [],
      settings: [],
      enabled: false,
      native: false,
      native_trusted: false,
      problem: "mod.json is missing",
    },
  ];

  const view = (id: string): GameView => {
    const game = GAMES.find((g) => g.id === id)!;
    const build = builds.has(id);
    const done = setUp.get(id);
    const step = stages.get(id);
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
      activity:
        step !== undefined
          ? { kind: "setting-up", ...steps[id][step] }
          : playing.has(id)
            ? { kind: "playing" }
            : { kind: "idle" },
      mods_on: modsOn.has(id),
      mods_dir: `~/${game.port}/mods`,
      data: `~/.local/share/chronicle-launcher/games/${id}/data`,
      save: `~/.local/share/chronicle-launcher/games/${id}/save`,
    };
  };
  const scan = (id: string): Scan => {
    const game = GAMES.find((g) => g.id === id)!;
    return game.boot
      ? {
          discs: [
            {
              path: `~/Games/PS2/${game.discs[0].file}`,
              // As the core: a game told by its boot program alone is its disc's region.
              release: game.index_file
                ? (game.releases?.find((r) => !r.refused)?.name ?? null)
                : game.discs[0].region,
            },
          ],
          skipped: [
            { path: "~/Games/PS2/Other Game.iso", why: "another game's disc (it boots SLUS_123.45)" },
            id === "chronicle"
              ? {
                  path: "~/Games/PS2/Dark Cloud (USA).iso",
                  why: "the NTSC release: the launcher sets up the PAL disc only",
                }
              : {
                  path: "~/Games/PS2/Dark Cloud 2 (USA).iso",
                  why: "Dark Cloud 2, the NTSC release: the launcher sets up the PAL disc only",
                },
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
      const order = steps[id];
      order.forEach((step, i) =>
        setTimeout(() => {
          if (!stages.has(id)) return;
          stages.set(id, i);
          emit("setup-line", { id, line: `${step.doing}…` });
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
      stages.set(id, 0);
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
    mods: async (id) => (id === "chronicle" ? structuredClone(mods) : []),
    async setModEnabled(_id, mod, on) {
      mods.find((m) => m.id === mod)!.enabled = on;
    },
    async setModSetting(_id, mod, setting, value) {
      const found = mods.find((m) => m.id === mod)!.settings.find((s) => s.id === setting)!;
      found.value = value ?? found.default;
    },
    async setModsOn(id, on) {
      if (on) modsOn.add(id);
      else modsOn.delete(id);
      return view(id);
    },
    pickMod: async () => ({
      manifest: {
        id: "author.weather",
        name: "Weather Report",
        version: "0.3.0",
        author: "A Modder",
        description: "Says what the sky over Norune will do tomorrow.",
        games: ["chronicle"],
        permissions: ["game.read", "storage.write"],
        native_extension: null,
      },
      replaces: false,
      for_this_game: true,
    }),
    async installMod() {
      mods.push({
        ...mods[2],
        folder: "author.weather",
        id: "author.weather",
        name: "Weather Report",
        version: "0.3.0",
        author: "A Modder",
        problem: null,
      });
      return "author.weather";
    },
    async discardMod() {},
    // A preview can show a backdrop kept in this browser's storage under chronicle-launcher.mock-scene.<id>; the
    // repository holds no picture of either game.
    async sceneImage(id) {
      try {
        return localStorage.getItem(`chronicle-launcher.mock-scene.${id}`);
      } catch {
        return null;
      }
    },
    async renderScene(id) {
      setTimeout(
        () =>
          emit("scene-ready", {
            id,
            ok: false,
            message: "The browser preview cannot run the game.",
            quiet: false,
          }),
        tick,
      );
    },
    pickScene: async () => false,
    async removeMod(_id, folder) {
      mods.splice(
        mods.findIndex((m) => m.folder === folder),
        1,
      );
    },
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
