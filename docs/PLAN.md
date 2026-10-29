# Chronicle Launcher: the plan

A launcher for the Dark Cloud ports, built from the
[OpenRAC launcher](https://github.com/OpenRAC/OpenRAC/tree/main/launcher). It
sets a game up from your own disc, plays its native PC port, and manages
[MoonMod](https://github.com/Jeffiest/MoonMod) (MoonFactory) mods. It never
downloads, uploads or bundles anything from the games.

| Game | Port | Launcher id | Executable | Extractor | Discs |
| --- | --- | --- | --- | --- | --- |
| Dark Cloud | [TheMoonPeople/Chronicle](https://github.com/TheMoonPeople/Chronicle) | `chronicle` | `darkcloud` | `dcdata` | PAL, NTSC 1.02 |
| Dark Chronicle / Dark Cloud 2 | [TheMoonPeople/ChronicleTwo](https://github.com/TheMoonPeople/ChronicleTwo) | `chronicle-two` | `darkchronicle` | `dc2data` | PAL |

The ids match MoonMod's manifest `games` enum (`chronicle`, `chronicle-two`), so a
mod's `mod.json` says directly which game pages list it.

## 1. What we take from OpenRAC, and what we leave

Base: OpenRAC `main` at `93ac558` (2026-10-29). It is Tauri 2: Rust does every
file and process operation (`core/`, `src-tauri/`), Svelte 5 draws the UI
(`src/`). It is GPL-3.0-or-later, so **this launcher is GPL-3.0-or-later too**,
keeps OpenRAC's copyright notices on the files it carries, and lists them in
`THIRD_PARTY_NOTICES.md`. Record the OpenRAC commit we last synced from in
`UPSTREAM.md`, the way DC_Web records its Chronicle import.

**Keep (rename and re-point):**

- The shell: header bar, game list on the left, one page per game whose main
  button names the next step (**Set up from your disc → Play**), a menu beside it
  for the rarer actions, toasts, the jobs/progress stream (`jobs.rs`,
  `JobProgress.svelte`), Tasks.
- Disc finding (`discs.rs`, `detect.rs`, `chd.rs`): scan a folder two levels deep,
  know a disc by its `SYSTEM.CNF`, accept `.chd`.
- Install/remove set-up (`install.rs`, `reset.rs`): **Remove the set-up** keeps the
  disc and the saves.
- Saves list with backups and restore (`saves.rs`, `SaveManagerBlock.svelte`).
- Crash notice, logs and **Copy report**/**Save report** with home paths shortened
  (`reports.rs`, `CrashNotice.svelte`, `ReportsBlock.svelte`).
- The Mods tab structure: overview of rows → a mod's own page with its settings
  (`PluginsPanel.svelte`, `ModPage.svelte`, `ModSetting.svelte`, `AddMod.svelte`,
  `remote.rs` for "add from a GitHub link").
- Settings rows, switches, segmented controls, folded Advanced (`.setting`,
  `details.more` in `app.css`), themes (`palettes.json`, `theme.rs`).
- Self-update (`update.rs`, `UpdateModal.svelte`), Discord presence (`discord.rs`),
  decomp progress band (`progress_sync.rs`, `ProgressBar.svelte`).
- The stylesheet tests (`stylesheets.test.ts`: no hex outside `:root`, hovers only
  in `@media (hover: hover)`) and the generated TS types from Rust (`ts-rs`).

**Leave out (Ratchet & Clank only):** the RAC save editor and its field tables
(planets, bolts, nanotech, gold bolts), Enhanced graphics and its light baking,
the Godot level editor and asset packs (`editor_state.rs`, `EditorFlow.svelte`,
`AssetsPanel.svelte`, `PacksPanel.svelte`), PCSX2 memory-card import, multiplayer,
Android, the six RAC game themes and pictures, `actions.json`'s OpenRAC repository
actions, and Luau plugin presets. Each can come back later as its Dark Cloud
equivalent (see phase 6).

## 2. The look: dark, moonlit, gold

Keep in step with what we already made. **DC_Web** (`port/web/site/index.html`)
set the palette: near-black ink, a slightly lifted panel, soft white text, and
**Dark Cloud gold** for the one main action. MoonMod's tools use the same family
(`#1c1c24` canvas, `#aaaabb` labels). The launcher keeps OpenRAC's *structure*
(rules in its `docs/STYLE.md`: one main action per page in a fixed place, rows not
boxes, secondary actions in a menu, removing in `--err`) and swaps its *colours*.

`src/app.css` on `:root`, OpenRAC token → Chronicle value:

| Token | OpenRAC | Chronicle | Source / use |
| --- | --- | --- | --- |
| `--ink` | `#181820` | `#0d0d12` | DC_Web `--bg`: the page |
| `--ink-deep` | `#12121a` | `#08080c` | inputs, job output |
| `--panel` | `#1d1d28` | `#17171f` | DC_Web `--panel` |
| `--panel-hi` | `#22222f` | `#1f1f2a` | cards in panels; DC_Web's secondary button is `#2a2a33` |
| `--brand` | `#d58a00` | `#a8842f` | section numbers, the selected job |
| `--amber` | `#ffb443` | `#c9a24a` | DC_Web `--accent`: links, focus rings, blockers |
| `--gold` | `#ffd08a` | `#e6cf8f` | headings |
| `--lav`, `--soft`, `--dim` | blue-lavender | `#c9c9d6`, `#aaaabb`, `#8a8a96` | DC_Web `--dim`, MoonMod's label grey |
| `--text`, `--body` | `#e8f0ff`, `#c3cdea` | `#e6e6ea`, `#c8c8d0` | DC_Web `--fg` |
| `--bar-top`, `--bar-bottom` | orange | `#d4b05a`, `#b38a32` | header bar, primary buttons: gold, not orange |
| `--bar-ink` | `#2a1800` | `#111111` | text on gold (DC_Web buttons use `#111`) |
| `--ok`, `--warn`, `--err` | | `#8fd694`, `--amber`, `#ff8a7a` | unchanged |

A **moon** accent ties it to The Moon People and MoonFactory: `--moon: #b9c4e6`
(a cool pale blue), used only for MoonMod things: the Mods tab's mark, the
"MoonMod" badge on a mod row, the Mods switch under Play.

Per-game colours (`src/lib/themes.ts`, same shape as OpenRAC's `--box`, `--bd`,
`--from`, `--to`, `--tx`):

- **Dark Cloud**: night indigo to black (`#1b1f3a → #0d0d12`), gold border. The
  Dark Genie's night over Norune.
- **Dark Chronicle**: warm sepia to dusk (`#3a2a1b → #0d0d12`), brass border. The
  Moon Flower/Palm Brinks storybook tone.

Type: **Cinzel** (headings, game titles: a storybook serif, fits both games' title
cards), **Inter** (body, as OpenRAC), system monospace (paths, logs). Bundle from
npm (`@fontsource/cinzel`, `@fontsource-variable/inter`); nothing loads from the
network. Dark only, as DC_Web is.

Logo: a crescent-moon mark plus "Chronicle" in Cinzel, drawn as SVG in
`Logo.svelte` (OpenRAC's component, new paths). Ask TheMoonPeople whether we may
use the Chronicle README logo (`i.imgur.com/pCMkgRM.png`) before using it.
Game pictures (`public/img/<id>-bg.webp`): stills from our own play, made with
OpenRAC's `scripts/media.py still`, each credited in `THIRD_PARTY_NOTICES.md`.

Write all of this into our own `docs/STYLE.md` in phase 1, as OpenRAC does.

## 3. How the launcher drives each game

Everything below comes from the ports' own docs (Chronicle `docs/PC.md`,
ChronicleTwo `docs/PC.md`); the launcher only calls documented commands and flags.

Launcher data folder: `%LOCALAPPDATA%\chronicle-launcher` (Windows),
`$XDG_DATA_HOME/chronicle-launcher` (Linux), `~/Library/Application
Support/Chronicle Launcher` (macOS). Under it, per game:

```
games/<id>/build/      the port build (executable, extractor, runtime DLLs)
games/<id>/data/       extracted data (+ data.zip, data.json for chronicle-two)
games/<id>/save/       saves, config.json, moonmod/profile.json
games/<id>/mods/       MoonMod packages, one folder per mod id
```

| Step | Dark Cloud (`chronicle`) | Dark Chronicle (`chronicle-two`) |
| --- | --- | --- |
| Identify the disc | `SYSTEM.CNF` boot serial, then `dcdata`'s release fingerprint of `DATA.HD2` | `SYSTEM.CNF`, then `dc2data list` |
| Set up | `dcdata extract <iso> data`, then `dcdata pack data` | `dc2data extract <iso> <root>/data` |
| Is set-up current? | `data/languages.json` present; re-extract after a port update | `dc2data current <iso> <root>/data` (exit status) |
| Check | none yet: show "Check" only when the port has one | `dc2data check <root>/data` |
| Play | `darkcloud --data <root>/data --save <root>/save` | `darkchronicle --data <root>/data --save <root>/save` |
| Settings file | `<save>/config.json` (documented keys and defaults) | `config.json` beside the save root (its `docs/PC.md`) |
| Exit status | 0 ok, 1 window/renderer, 2 bad args, 3 no data, 4 game assertion | 0 ok, 3 no data, crash report per `CRASH_REPORTS.md` |

Rules: the launcher passes explicit `--data` and `--save`, so the ports' own
fallbacks (`DC_DATA`, `XDG_DATA_HOME/chronicle`, …) never pick a different folder;
it never writes `config.json` or `profile.json` while the game runs; exit status 3
sends the player back to **Set up**, 4 and crashes open the crash notice.

**Where the builds come from.** Neither port publishes GitHub releases today (only
CI artifacts). Two modes, as OpenRAC has:

1. **Installed (players).** The launcher's installer, or a first-run download from
   *this* repository's releases, carries a port build for the player's system. Our
   CI (`.github/workflows/ports.yml`) builds each port at a pinned commit, with
   MoonFactory applied (`ports/<game>/apply-moonfactory.ps1` from MoonMod at its
   pinned commit), and publishes `chronicle-<sha8>-<platform>.zip`. The pin for
   Chronicle is MoonMod's `ports/chronicle/CHRONICLE_COMMIT`, so the build and the
   mod framework always agree. Builds contain no disc data.
2. **Checkout (contributors).** Point the launcher at a Chronicle/ChronicleTwo
   checkout; **Build the game** runs its `build.sh`/`build.bat <preset>-release`
   (Kazarov, Docker/Podman on Linux) and plays `port/build/...`. Mirrors OpenRAC's
   `port_build.rs`.

**Blocker to clear first:** neither Chronicle repository has a LICENSE file.
Redistributing builds (mode 1) needs TheMoonPeople's written OK, or a license added
upstream. Until then ship mode 2 only, plus "bring your own build" (pick a folder
with `darkcloud`/`darkchronicle` in it).

## 4. MoonMod in the launcher

The game's in-game Mod Manager stays the source of truth; the launcher reads and
writes the same files, so both always agree.

- **Discover**: every `<mods>/<id>/mod.json`, validated against MoonMod's
  `manifest.schema.json` (vendored, pinned). A mod whose `games` does not list this
  game is not shown on its page. A mod that fails validation shows why
  (`Reason.svelte`).
- **Enable, order, settings**: read and write `<save>/moonmod/profile.json`
  (`format_version: 1`, `enabled`, `settings`), keeping keys we do not know. Settings
  rows render from the manifest's typed `settings` (`boolean` → switch, `choice` →
  segmented/radios, `integer`/`number` with `min`/`max` → slider + field, `string`
  → field), the same rows OpenRAC's `ModSetting.svelte` draws. Dependencies,
  `load_after`/`load_before` and conflicts show on the mod's page.
- **Permissions**: list the manifest's `permissions` in plain words on the mod's
  page ("reads and changes the game", "keeps its own small save").
- **Native plugins** (`native_extension`): the launcher shows them, but **does not
  set `native_trusted`**. Trust stays the player's confirmation inside the game, as
  MoonMod's security model says. The row says "Needs trusting in game".
- **Install**: drop a `.zip` or folder, or **Add from a link** (a GitHub repository
  or release, as OpenRAC's `remote.rs`): preview id, author, version, games and
  permissions before anything is written. The folder is named by the manifest `id`.
- **Bundled mods**: the MoonMod release's `mods/` (`moonmod.rpg`,
  `moonmod.roguelike`, …) come with the build, all disabled, as MoonMod does.
- **Mods switch under Play**: off by default (the game as the disc plays it). Off
  starts the game with every mod disabled for that run without forgetting the
  player's choices. *Needs a MoonFactory flag such as `--no-mods`; propose it in
  MoonMod.*
- **Create**: a "New mod" button runs `tools/moonfactory.py new` into the mods
  folder and opens it; developer view only.

## 5. Repository layout

```
core/            Rust: discs, install, play, saves, reports, moonmod (profile, manifests)
src-tauri/       Tauri app, commands, icons, tauri.conf.json (dev.themoonpeople.chronicle-launcher)
src/             Svelte 5 UI, app.css, themes.ts, components/, pages/
games/           chronicle.json, chronicle-two.json: discs, commands, paths (the only per-game code)
vendor/moonmod/  manifest.schema.json at MoonMod's pinned commit
docs/            PLAN.md, STYLE.md, INTEGRATION.md, ARCHITECTURE.md, RELEASING.md
.github/         checks.yml (yarn verify, cargo test), ports.yml, release.yml
UPSTREAM.md      OpenRAC, Chronicle, ChronicleTwo and MoonMod commits we follow
```

The aim is that adding a third game (or a dc1tk/DC2-Toolkit tool) is one
`games/<id>.json` and a theme, not new Rust.

## 6. Phases

| # | Deliverable | Done when |
| --- | --- | --- |
| 0 | Import OpenRAC `launcher/` at `93ac558`, rename (product, identifier, data folder), strip the RAC-only parts, GPL notices, `UPSTREAM.md` | `yarn verify` and `cargo test` pass; the app opens with an empty game list |
| 1 | The look: tokens, Cinzel/Inter, logo, two game themes, `docs/STYLE.md`, stylesheet tests | screenshots of the Library and a game page reviewed |
| 2 | Dark Cloud end to end: find disc, set up with `dcdata`, play, saves with backups, crash/log reports, "bring your own build" and checkout build | a PAL and an NTSC 1.02 disc each go from folder to playing |
| 3 | MoonMod: discover, enable, settings, install from zip/link, bundled mods, Mods switch | every bundled mod toggles in the launcher and the in-game manager agrees |
| 4 | Dark Chronicle: `dc2data` set-up and check, play, saves, MoonMod on `chronicle-two` | a PAL disc goes from folder to playing with a mod on |
| 5 | Builds and releases: `ports.yml` (Windows x64, Linux x64; macOS arm64 after), signed launcher installers, self-update | depends on the license answer in §3 |
| 6 | Later: settings pages from each port's `config.json`, save viewer then editor (Chronicle's `saves/1/save.dat`, ChronicleTwo's `dc2Ver4` JSON), decomp.dev progress band, Discord presence, DC2-Toolkit/dc1tk tools | each its own PR |

Platforms: Windows and Linux first (MoonMod's scripts are PowerShell today; the
Linux Flatpaks the ports build are a second source to support). macOS when the
ports' macOS builds are packaged. No Switch, iOS or Android.

## 7. Open questions

1. **Licensing of the ports**: may we redistribute Chronicle/ChronicleTwo builds?
   (Blocks phase 5.)
2. **Logo**: use the Chronicle README logo, or our own moon mark?
3. **Name**: "Chronicle Launcher" or a Moon People name (e.g. "Moon Gate")?
4. **`--no-mods`** (or equivalent) in MoonFactory, for the Mods switch.
5. **One build or two** per game: a MoonFactory build only (mods off = vanilla), or
   also an unpatched build for players who want none of it?
