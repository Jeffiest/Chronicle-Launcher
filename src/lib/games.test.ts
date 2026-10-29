import { describe, expect, it } from "vitest";
import { GAMES, chapterName, material } from "./games";
import { mockApi } from "./api";

describe("games", () => {
  it("are the two chapters, in order, under MoonMod's names", () => {
    expect(GAMES.map((g) => g.id)).toEqual(["chronicle", "chronicle-two"]);
    expect(GAMES.map((g) => chapterName(g.chapter))).toEqual(["Chapter I", "Chapter II"]);
  });

  it("each wear one of the game's windows, defined in app.css", async () => {
    const { readFileSync } = await import("node:fs");
    const css = readFileSync(new URL("../app.css", import.meta.url), "utf8");
    expect(GAMES.map((g) => material(g.chapter))).toEqual(["stone", "paper"]);
    for (const game of GAMES) expect(css).toContain(`.${material(game.chapter)} {`);
  });

  it("go to GitHub for Get the game", () => {
    for (const game of GAMES) expect(game.get_url).toMatch(/^https:\/\/github\.com\//);
  });
});

describe("the browser's mock", () => {
  it("moves a game from Get the game to Set up and back", async () => {
    const api = mockApi();
    expect((await api.library()).map((v) => v.next)).toEqual(["get-the-game", "get-the-game"]);
    expect((await api.chooseBuild("chronicle"))?.next).toBe("set-up");
    expect((await api.library())[1].next).toBe("get-the-game");
    expect((await api.forgetBuild("chronicle")).next).toBe("get-the-game");
  });
});
