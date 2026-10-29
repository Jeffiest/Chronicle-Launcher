import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { permissionWords } from "./permissions";

describe("permissions", () => {
  it("has words for every permission MoonMod's schema allows", () => {
    const schema = JSON.parse(
      readFileSync(new URL("../../vendor/moonmod/manifest.schema.json", import.meta.url), "utf8"),
    );
    const all: string[] = schema.properties.permissions.items.enum;
    for (const permission of all) expect(permissionWords([permission])[0]).not.toBe(permission);
  });

  it("says the same thing once", () => {
    expect(permissionWords(["storage.read", "storage.write", "game.write"])).toEqual([
      "Keeps its own small save",
      "Changes the game",
    ]);
  });
});
