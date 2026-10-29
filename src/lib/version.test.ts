import { describe, expect, it } from "vitest";
// @ts-expect-error: a plain Node script, without type declarations
import { versions } from "../../scripts/version.mjs";

describe("the launcher's version", () => {
  it("is the same in package.json, tauri.conf.json and Cargo.toml", () => {
    const all = versions() as Record<string, string>;
    expect(new Set(Object.values(all)).size).toBe(1);
    expect(Object.values(all)[0]).toMatch(/^\d+\.\d+\.\d+$/);
  });
});
