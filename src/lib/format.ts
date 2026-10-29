// How numbers, times and paths read on a page.

/** Play time as the save board shows it: hours, minutes and seconds. */
export function playTime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

/** A moment, in the player's own date and time format. */
export function when(unix: number): string {
  return new Date(unix * 1000).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

/** The last part of a path, whichever slash it uses. */
export function fileName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}

/** The lit part of a waxing moon of radius 1 at the origin, lit from the right, `lit` from 0 to 1. */
export function moonPath(lit: number): string {
  const f = Math.min(1, Math.max(0, lit));
  const rx = Math.abs(1 - 2 * f).toFixed(4);
  return `M0,-1 A1,1 0 0 1 0,1 A${rx},1 0 0 ${f < 0.5 ? 0 : 1} 0,-1 Z`;
}
