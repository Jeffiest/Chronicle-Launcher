// What every page shares: each game's view, the extractor's latest lines per game, and the notices along the
// bottom. The views stay current from the app's events (lib/api.ts, Events).

import { api } from "./api";
import type { GameView } from "./types";

export interface Toast {
  id: number;
  text: string;
  tone: "ok" | "err" | "";
}

const LINES = 200;

export const store = $state({
  views: [] as GameView[],
  lines: {} as Record<string, string[]>,
  toasts: [] as Toast[],
  savesChanged: 0,
  scenes: {} as Record<string, string | null>,
});

/** Fetches a game's backdrop again (after set-up, a new picture, or the player's own). */
export async function loadScene(id: string) {
  store.scenes[id] = (await attempt(() => api.sceneImage(id))) ?? null;
}

let next = 1;

export function toast(text: string, tone: Toast["tone"] = "") {
  const id = next++;
  store.toasts.push({ id, text, tone });
  setTimeout(() => (store.toasts = store.toasts.filter((t) => t.id !== id)), tone === "err" ? 9000 : 5000);
}

export function replace(view: GameView | null | undefined) {
  if (!view) return;
  store.views = store.views.map((v) => (v.game.id === view.game.id ? view : v));
}

/** Runs an action and shows its error, if it has one, as a notice. */
export async function attempt<T>(run: () => Promise<T>): Promise<T | undefined> {
  try {
    return await run();
  } catch (e) {
    toast(String(e), "err");
    return undefined;
  }
}

export async function refresh() {
  const views = await attempt(() => api.library());
  if (views) store.views = views;
}

let started = false;

export async function start() {
  if (started) return;
  started = true;
  await api.on("activity", (view) => replace(view));
  await api.on("setup-line", ({ id, line }) => {
    const lines = [...(store.lines[id] ?? []), line];
    store.lines[id] = lines.slice(-LINES);
  });
  await api.on("setup-finished", ({ ok, message }) => toast(message, ok ? "ok" : "err"));
  await api.on("scene-ready", ({ id, ok, message, quiet }) => {
    if (!quiet) toast(message, ok ? "ok" : "err");
    if (ok) loadScene(id);
  });
  await api.on("game-ended", ({ id, outcome }) => {
    store.savesChanged++;
    const title = store.views.find((v) => v.game.id === id)?.game.title ?? "The game";
    if (outcome.kind === "crashed") toast(`${title} stopped unexpectedly. Your saves are kept.`, "err");
    if (outcome.kind === "no-data") toast("The game found no data: set it up from your disc again.", "err");
    refresh();
  });
  await refresh();
}
