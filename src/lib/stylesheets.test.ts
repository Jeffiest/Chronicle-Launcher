// docs/STYLE.md's rules, held for app.css and every component's <style>: no hex colour outside :root (a colour is
// a token), and hover styles only inside @media (hover: hover) (a tapped control would otherwise keep them).

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const SRC = join(__dirname, "..");

function files(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return files(path);
    return /\.(css|svelte)$/.test(entry.name) ? [path] : [];
  });
}

function css(path: string): string {
  const text = readFileSync(path, "utf8");
  if (path.endsWith(".css")) return text;
  return [...text.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]).join("\n");
}

/** Each rule's selector chain, outer at-rules included. */
function rules(text: string): { chain: string[]; body: string }[] {
  const out: { chain: string[]; body: string }[] = [];
  const stack: string[] = [];
  let start = 0;
  const clean = text.replace(/\/\*[\s\S]*?\*\//g, "");
  for (let i = 0; i < clean.length; i++) {
    const ch = clean[i];
    if (ch === "{") {
      stack.push(clean.slice(start, i).trim());
      start = i + 1;
    } else if (ch === "}") {
      out.push({ chain: [...stack], body: clean.slice(start, i) });
      stack.pop();
      start = i + 1;
    } else if (ch === ";" && stack.length === 0) {
      start = i + 1;
    }
  }
  return out;
}

describe("stylesheets", () => {
  for (const path of files(SRC)) {
    const name = path.slice(SRC.length + 1);
    const text = css(path);

    it(`${name}: no hex colour outside :root`, () => {
      const bad = rules(text)
        .filter((r) => !r.chain.includes(":root") && /#[0-9a-f]{3,8}\b/i.test(r.body))
        .map((r) => `${name}: ${r.chain.at(-1)}`);
      expect(bad).toEqual([]);
    });

    it(`${name}: hover only inside @media (hover: hover)`, () => {
      const bad = rules(text)
        .filter((r) => r.chain.some((s) => s.includes(":hover")))
        .filter((r) => !r.chain.some((s) => /@media\s*\(hover:\s*hover\)/.test(s)))
        .map((r) => `${name}: ${r.chain.at(-1)}`);
      expect(bad).toEqual([]);
    });
  }
});
