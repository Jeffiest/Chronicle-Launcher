// What a mod's manifest permissions (MoonMod manifest.schema.json) let it do, in a player's words.

const WORDS: Record<string, string> = {
  "storage.read": "Keeps its own small save",
  "storage.write": "Keeps its own small save",
  "game.read": "Reads what is happening in the game",
  "game.write": "Changes the game",
  "content.register": "Adds things to the game",
  "scripts.override": "Replaces some of the game's own scripts",
  "debug.inspect": "Looks inside the game, for debugging",
  "native.extension": "Runs its own program code, outside MoonFactory's sandbox",
};

/** Each permission once in words, the same words once, in the manifest's order. */
export function permissionWords(permissions: string[]): string[] {
  const words = permissions.map((p) => WORDS[p] ?? p);
  return words.filter((w, i) => words.indexOf(w) === i);
}
