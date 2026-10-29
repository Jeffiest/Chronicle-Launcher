// The launcher's version is in three places: package.json, src-tauri/tauri.conf.json and the Cargo workspace.
//
//   node scripts/version.mjs              prints it, or fails if the three disagree
//   node scripts/version.mjs check v1.2.3 fails unless all three are 1.2.3 (the release workflow's tag check)
//   node scripts/version.mjs set 1.2.3    writes 1.2.3 into all three

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const files = {
  package: `${root}package.json`,
  tauri: `${root}src-tauri/tauri.conf.json`,
  cargo: `${root}Cargo.toml`,
};
const CARGO = /(\[workspace\.package\][^[]*?\nversion = ")([^"]+)(")/;

export function versions() {
  const cargo = readFileSync(files.cargo, "utf8").match(CARGO);
  return {
    package: JSON.parse(readFileSync(files.package, "utf8")).version,
    tauri: JSON.parse(readFileSync(files.tauri, "utf8")).version,
    cargo: cargo ? cargo[2] : null,
  };
}

function set(version) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error(`${version} is not MAJOR.MINOR.PATCH`);
  for (const key of ["package", "tauri"]) {
    const json = JSON.parse(readFileSync(files[key], "utf8"));
    json.version = version;
    writeFileSync(files[key], `${JSON.stringify(json, null, 2)}\n`);
  }
  const cargo = readFileSync(files.cargo, "utf8");
  writeFileSync(files.cargo, cargo.replace(CARGO, `$1${version}$3`));
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [command, value] = process.argv.slice(2);
  const all = versions();
  const distinct = [...new Set(Object.values(all))];
  if (command === "set") {
    set(value);
    console.log(`version ${value}`);
  } else if (distinct.length !== 1) {
    console.error(`the versions disagree: ${JSON.stringify(all)}`);
    process.exit(1);
  } else if (command === "check" && value?.replace(/^v/, "") !== distinct[0]) {
    console.error(
      `the tag ${value} is not the version ${distinct[0]}; run: node scripts/version.mjs set <version>`,
    );
    process.exit(1);
  } else {
    console.log(distinct[0]);
  }
}
