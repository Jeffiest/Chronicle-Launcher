// The launcher's commands. In the app they go to Rust (src-tauri/src/lib.rs); in a plain browser (npm run dev)
// they are mocked here, so the pages can be worked on without a build of the app.

import type { GameView } from "./types";
import { GAMES } from "./games";

const inApp = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

export interface Api {
  library(): Promise<GameView[]>;
  /** Asks for a folder, then remembers it as the game's build. Null when the player cancelled. */
  chooseBuild(id: string): Promise<GameView | null>;
  forgetBuild(id: string): Promise<GameView>;
  openLink(url: string): Promise<void>;
}

const app: Api = {
  library: () => call("library"),
  async chooseBuild(id) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const folder = await open({ directory: true, title: "Choose the folder with the game's build" });
    if (typeof folder !== "string") return null;
    return call("choose_build", { id, folder });
  },
  forgetBuild: (id) => call("forget_build", { id }),
  async openLink(url) {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(url);
  },
};

/** The browser's stand-in: a build "chosen" for a game moves it to Set up. */
export function mockApi(chosen = new Set<string>()): Api {
  const view = (id: string): GameView => {
    const game = GAMES.find((g) => g.id === id)!;
    const has = chosen.has(id);
    return {
      game,
      next: has ? "set-up" : "get-the-game",
      build: has ? { dir: `~/${game.port}`, executable: game.executable, extractor: game.extractor } : null,
      build_problem: null,
      data: `~/.local/share/chronicle-launcher/games/${id}/data`,
      save: `~/.local/share/chronicle-launcher/games/${id}/save`,
    };
  };
  return {
    library: async () => GAMES.map((g) => view(g.id)),
    async chooseBuild(id) {
      chosen.add(id);
      return view(id);
    },
    async forgetBuild(id) {
      chosen.delete(id);
      return view(id);
    },
    async openLink(url) {
      window.open(url, "_blank", "noopener");
    },
  };
}

export const api: Api = inApp ? app : mockApi();
