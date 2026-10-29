import { describe, expect, it } from "vitest";
import { mockApi } from "./api";
import { mainButton, stateOf } from "./next";

describe("a game's next step", () => {
  it("goes Get the game → Set up → Play, through setting up and playing", async () => {
    const api = mockApi(5);
    const first = (await api.library())[0];
    expect(mainButton(first)).toBe("Get the game");
    expect(mainButton((await api.chooseBuild("chronicle"))!)).toBe("Set up from your disc");

    const disc = (await api.scanDiscs("chronicle", "~/Games/PS2")).discs[0];
    expect(disc.path).toContain("Dark Cloud");
    const settingUp = await api.setUp("chronicle", disc.path);
    expect(stateOf(settingUp).words).toBe("Setting up");
    expect(mainButton(settingUp)).toBe("Setting up…");

    const finished = new Promise<boolean>((done) => api.on("setup-finished", ({ ok }) => done(ok)));
    expect(await finished).toBe(true);
    const ready = (await api.library())[0];
    expect(mainButton(ready)).toBe("Play");
    expect(stateOf(ready)).toEqual({ words: "Ready to play", tone: "ok" });

    expect(mainButton(await api.play("chronicle"))).toBe("Playing");
  });

  it("takes Dark Chronicle's PAL disc to Play, with the time left as it goes", async () => {
    const api = mockApi(5);
    await api.chooseBuild("chronicle-two");
    const scan = await api.scanDiscs("chronicle-two", "~/Games/PS2");
    expect(scan.discs.map((d) => d.release)).toEqual(["PAL"]);
    expect(scan.skipped.some((s) => s.why.includes("Dark Cloud 2, the NTSC release"))).toBe(true);

    const seen: string[] = [];
    await api.on("activity", (v) => {
      if (v.activity.kind === "setting-up") seen.push(v.activity.doing);
    });
    const finished = new Promise<boolean>((done) => api.on("setup-finished", ({ ok }) => done(ok)));
    await api.setUp("chronicle-two", scan.discs[0].path);
    expect(await finished).toBe(true);
    expect(seen.some((doing) => doing.endsWith("left"))).toBe(true);
    expect(mainButton((await api.library())[1])).toBe("Play");
  });
});
