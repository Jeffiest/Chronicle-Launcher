import { describe, expect, it } from "vitest";
import { fileName, moonPath, playTime } from "./format";

describe("format", () => {
  it("writes play time as the save board does", () => {
    expect(playTime(0)).toBe("0:00:00");
    expect(playTime(3725)).toBe("1:02:05");
    expect(playTime(100 * 3600 + 59)).toBe("100:00:59");
  });

  it("names a file from either kind of path", () => {
    expect(fileName("/home/t/Dark Cloud (NTSC).iso")).toBe("Dark Cloud (NTSC).iso");
    expect(fileName("C:\\Games\\dc.iso")).toBe("dc.iso");
  });

  it("waxes the moon from a sliver to full", () => {
    expect(moonPath(0)).toBe("M0,-1 A1,1 0 0 1 0,1 A1.0000,1 0 0 0 0,-1 Z");
    expect(moonPath(0.5)).toContain("A0.0000,1");
    expect(moonPath(1)).toBe("M0,-1 A1,1 0 0 1 0,1 A1.0000,1 0 0 1 0,-1 Z");
    expect(moonPath(2)).toBe(moonPath(1));
  });
});
