# Chronicle Launcher: the plan

A launcher for the Dark Cloud ports. It sets a game up from your own disc, plays
its native PC port, and manages [MoonMod](https://github.com/Jeffiest/MoonMod)
(MoonFactory) mods. It never downloads, uploads or bundles anything from the games.

The [OpenRAC launcher](https://github.com/OpenRAC/OpenRAC/tree/main/launcher) is
our **reference**, the way [OpenGOAL's launcher](https://github.com/open-goal/launcher)
was OpenRAC's: we learn from how it works and what players liked, and we write our
own launcher with its own Dark Cloud / Dark Chronicle character. It is not a fork.

| Game | Port | Launcher id | Executable | Extractor | Discs |
| --- | --- | --- | --- | --- | --- |
| Dark Cloud | [TheMoonPeople/Chronicle](https://github.com/TheMoonPeople/Chronicle) | `chronicle` | `darkcloud` | `dcdata` | PAL (the July 12, 2001 prototype); NTSC is not supported |
| Dark Chronicle / Dark Cloud 2 | [TheMoonPeople/ChronicleTwo](https://github.com/TheMoonPeople/ChronicleTwo) | `chronicle-two` | `darkchronicle` | `dc2data` | PAL |

The ids match MoonMod's manifest `games` enum (`chronicle`, `chronicle-two`), so a
mod's `mod.json` says directly which game pages list it.

## 1. What we learn from OpenRAC, and what is ours

Same stack as the reference, because it fits: **Tauri 2**, Rust for every file and
process operation (`core/`, `src-tauri/`), **Svelte 5** for the UI (`src/`). We
write our own code. Where we do adapt a specific OpenRAC file (for example its CHD
reader), that file keeps OpenRAC's GPL-3.0-or-later notice and is listed in
`THIRD_PARTY_NOTICES.md`, so the launcher's license has to allow that (see §7).
`docs/REFERENCE.md` records the OpenRAC commit we last studied (`93ac558`,
2026-10-29) and what we took as an idea from it.

**Ideas we take from the reference** (players liked them; we rebuild them our way):

- One page per game whose main button always names the next step (**Get the
  game → Set up from your disc → Play**), with a menu beside it for the rarer
  actions and anything that removes set apart at the end.
- Disc finding: scan a folder two levels deep, know a disc by its `SYSTEM.CNF`,
  accept `.chd`. **Remove the set-up** keeps the disc and the saves.
- Long jobs (extraction) stream progress into the page; a log for each run.
- Saves with backups and restore; crash notice with **Copy report**/**Save
  report**, home paths shortened to `~`.
- Mods as an overview of rows → each mod's own page with its settings; install
  from a zip, a folder or a GitHub link with a preview first.
- Settings as rows (name left, control right), applied as they change; folded
  Advanced.
- Style rules as tests: no hex colour outside `:root`, hover styles only in
  `@media (hover: hover)`, reduced motion respected. Rust types exported to TS.

**Ours, not the reference's:** Ratchet & Clank's save editor, Enhanced graphics,
the Godot level editor, PCSX2 card import, multiplayer and Android have no place
here. What replaces them is in §1a.

## 1a. A Dark Cloud take

The two games are told as **one chronicle in two chapters**, and the launcher
reads like it:

- **Home is the Chronicle.** Not a grid of tiles: two tall storybook covers side by
  side, *Chapter I: Dark Cloud* and *Chapter II: Dark Chronicle*, each with its
  backdrop, its state in a few words ("Ready to play", "Needs your disc"), and the
  last time played. A game page opens like turning to that chapter.
- **The two moons are the through-line.** The Chronicle logo and the ports' icon
  both show a gold moon and a blue moon over purple clouds. The logo heads the home
  page; the icon is the app's icon. Set-up progress can be the gold moon rising.
  The Mods tab is called **Moon Factory**.
- **It wears the games' own materials.** Windows are the game's Blue Stone and
  Paper; buttons are its stone plates. Dark Cloud's cover is Blue Stone, Dark
  Chronicle's is Paper (chosen by us: the Chronicle Two port has no menu
  materials of its own yet). Both are framed in the logo's gold.
- **Saves read like the game's own save screen**: the hero (Toan, or Max and
  Monica), where they are, play time, the date, instead of slot numbers and bytes.
  Chronicle's `saves/<n>/save.dat` and ChronicleTwo's JSON saves (`dc2Ver4`
  header: `map_no`, `play_time`) give us these.
- **Words stay plain where they act.** Buttons say exactly what they do (**Play**,
  **Set up from your disc**); the flavour lives in headings, art and motion, so
  nothing becomes a riddle.
- **Later, ours alone**: a Georama gallery (screenshots from the port's
  `--screenshot`, per town), a Dark Chronicle "Scoop" photo album, decomp
  progress shown as a chapter's pages filling in (decomp.dev).

## 2. The look: the games' own menus

The launcher looks like the games' own screens, after the OpenGOAL layout: a title bar, a rail of the games down the
left, and each game's page filling the window.

- **The menu window** (measured from captures of MoonMod's Mod Manager inside each port): near-black `#0c0e12`, a
  slate frame `#272c36` with a dark edge, `#4d5159` rules, a `#16191f` footer of control hints, and the chosen row on
  a brown bar. Every panel is one of these windows.
- **Each game's accent and lettering**: Dark Cloud's menu yellow `#bbbb40`, chosen row `#453624`, in Chronicle Sharp
  (Dark Cloud's own letters); Dark Chronicle's pale gold `#d0b16c`, chosen row `#433725`, in a condensed sans like
  its menus (Barlow Condensed, bundled). Titles in Chronicle Title, white with a dark edge as the games' headings.
- **The scene behind each page**: a frame of the game rendered by the port from the player's own data after set-up
  (`--headless --jump ... --screenshot`, `games/*.json` `scene_args`: populated Norune, Palm Brinks), or the player's
  own screenshot. Nothing from the games ships with the launcher. Before there is one, Dark Chronicle shows its
  Options backdrop of leather and embossed gold lines, drawn by the launcher; Dark Cloud its menu darkness.
- **Progress**: Dark Cloud's gold moon waxing; Dark Chronicle's gold ring filling, from the same ornament.
- **Moon Factory** is laid out as the in-game Mod Manager is, so the two read as one tool.

The frame counts in `scene_args` are first guesses; a run on a real build sets them.

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
| Identify the disc | `SYSTEM.CNF` boots `SCUS_971.11` (PAL and NTSC both), then the size and FNV-1a 64 of `DATA.HD2`, `dcdata`'s own table: only the PAL prototype is set up, NTSC 1.02 is refused by name, and an unlisted release is refused too | `SYSTEM.CNF` boots `SCES_511.90` (PAL); Dark Cloud 2's NTSC disc (`SCUS_972.13`) is turned away by name. `dc2data` has no release table, so the boot program decides |
| Set up | `dcdata extract <iso> data` (no `pack`: the loose `data/` plays, and packing would keep the game's files twice) | `dc2data extract <iso> <root>/data`, which writes `<root>/data.zip` and `data.json` (no `data/` folder) and, piped, prints `dc2data: [ 42%] … ~2:10 left` lines the moon follows |
| Is set-up current? | `setup.json` beside `data/`, written only after the extractor succeeded; exit status 3 from the game forgets it | `setup.json` beside `data.zip`; the game itself refuses older data (exit 3 or 5), which forgets it |
| Check | none yet: show "Check" only when the port has one | `dc2data check <root>/data` exists; not offered yet |
| Play | `darkcloud --data <root>/data --save <root>/save` | `darkchronicle --data <root>/data --save <root>/save` |
| Settings file | `<save>/config.json` (documented keys and defaults) | `config.json` beside the save root (its `docs/PC.md`) |
| Exit status | 0 ok, 1 window/renderer, 2 bad args, 3 no data, 4 game assertion | as Dark Cloud, plus 5: data rejected (set up again). A crash also leaves the game's own report in `<save>/crashes/`, which **Copy report** includes |

Rules: the launcher passes explicit `--data` and `--save`, so the ports' own
fallbacks (`DC_DATA`, `XDG_DATA_HOME/chronicle`, …) never pick a different folder;
it never writes `config.json` or `profile.json` while the game runs; exit status 3
sends the player back to **Set up**, 4 and crashes open the crash notice.

**Where the builds come from.** Neither port publishes GitHub releases today (only
CI artifacts). There is **one build per game**: the port with MoonFactory built in.
With every mod off (or the Mods switch off) it plays as the disc does, so there is
no separate unmodded build.

**For now (decided 2026-10-29):** the launcher does not ship or download builds.
When a game has no build, its main button is **Get the game** and opens the
port's GitHub page in the browser (Chronicle's, or ChronicleTwo's; once MoonMod
publishes `MoonMod-<commit>` releases for both, their release page instead).
Beside it, **Choose a build** picks a folder that holds `darkcloud`/`darkchronicle`
and its extractor, and the button becomes **Set up from your disc** → **Play**.
The link lives in `games/<id>.json` (`get_url`), so switching to shipped builds
later changes data, not code.

Planned, held for later:

1. **Shipped builds (players).** The launcher's installer, or a first-run download from
   *this* repository's releases, carries a port build for the player's system. Our
   CI (`.github/workflows/ports.yml`) builds each port at a pinned commit, with
   MoonFactory applied (`ports/<game>/apply-moonfactory.ps1` from MoonMod at its
   pinned commit), and publishes `chronicle-<sha8>-<platform>.zip`. The pin for
   Chronicle is MoonMod's `ports/chronicle/CHRONICLE_COMMIT`, so the build and the
   mod framework always agree. Builds contain no disc data.
2. **Checkout (contributors).** Point the launcher at a Chronicle/ChronicleTwo
   checkout; **Build the game** runs its `build.sh`/`build.bat <preset>-release`
   (Kazarov, Docker/Podman on Linux) and plays `port/build/...`.

Before shipped builds: neither Chronicle repository has a LICENSE file, so
redistributing builds needs TheMoonPeople's written OK, or a license added upstream.

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
  → field). Dependencies,
  `load_after`/`load_before` and conflicts show on the mod's page.
- **Permissions**: list the manifest's `permissions` in plain words on the mod's
  page ("reads and changes the game", "keeps its own small save").
- **Native plugins** (`native_extension`): the launcher shows them, but **does not
  set `native_trusted`**. Trust stays the player's confirmation inside the game, as
  MoonMod's security model says. The row says "Needs trusting in game".
- **Install**: drop a `.zip` or folder, or **Add from a link** (a GitHub repository
  or release): preview id, author, version, games and
  permissions before anything is written. The folder is named by the manifest `id`.
- **Bundled mods**: the MoonMod release's `mods/` (`moonmod.rpg`,
  `moonmod.roguelike`, …) come with the build, all disabled, as MoonMod does.
- **Mods switch under Play**: off by default (the game as the disc plays it). Off
  starts the game with every mod disabled for that run without forgetting the
  player's choices: the launcher sets `MOONMOD_NO_MODS=1`, which MoonFactory reads
  at start (MoonMod branch `launcher-no-mods`, in its runtime and both ports'
  patches). An environment variable, not a flag, so neither port's argument
  parser changes. A MoonMod build without that change ignores it and loads the
  mods the player turned on.
- **Where the mods are**: the folder the game reads, by its own rule
  (`mods_directory.hpp`): `mods/` at the checkout's root, else beside the
  executable, else `<save>/mods`.
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
UPSTREAM.md      Chronicle, ChronicleTwo and MoonMod commits we follow
docs/REFERENCE.md  the OpenRAC commit we studied and the ideas taken from it
```

The aim is that adding a third game (or a dc1tk/DC2-Toolkit tool) is one
`games/<id>.json` and a theme, not new Rust.

## 6. Phases

| # | Deliverable | Done when |
| --- | --- | --- |
| 0 | Our own skeleton: Tauri 2 + Svelte 5 + Rust core, the two games as data, the Chronicle home and a game page with **Get the game** / **Choose a build**, tokens and style tests, CI | `yarn verify` and `cargo test` pass; CI builds the app; the page opens in a browser with mock data |
| 1 | The look: the games' materials, fonts, logo and icon (started), backgrounds from our play, the gold-moon progress, `docs/STYLE.md` | screenshots of the Library and a game page reviewed |
| 2 | Dark Cloud end to end: find disc, set up with `dcdata`, play, saves with backups, crash/log reports, **Get the game** link and **Choose a build**, checkout build. **Built**; tested with stand-ins for the port's programs, not yet with a real disc | a PAL disc goes from folder to playing; an NTSC disc is turned away with its reason |
| 3 | MoonMod: discover, enable, settings, install from zip or folder, bundled mods, Mods switch. **Built**, but not install from a GitHub link yet; every bundled mod and example passes the launcher's manifest checks | every bundled mod toggles in the launcher and the in-game manager agrees |
| 4 | Dark Chronicle: `dc2data` set-up with its percentage, play, saves (`BESCES-51190dkcl<n>.json`, play time at 50 a second), MoonMod on `chronicle-two`. **Built**; tested with stand-ins for `dc2data` and `darkchronicle` | a PAL disc goes from folder to playing with a mod on |
| 5 | Builds and releases: `ports.yml` (Windows x64, Linux x64; macOS arm64 after), signed launcher installers, self-update | held for later; needs the ports' license OK (§3) |
| 6 | Later: settings pages from each port's `config.json`, save viewer then editor (Chronicle's `saves/1/save.dat`, ChronicleTwo's `dc2Ver4` JSON), decomp.dev progress band, Discord presence, DC2-Toolkit/dc1tk tools | each its own PR |

Platforms: Windows and Linux first (MoonMod's scripts are PowerShell today; the
Linux Flatpaks the ports build are a second source to support). macOS when the
ports' macOS builds are packaged. No Switch, iOS or Android.

## 7. Decisions and open questions

Decided (2026-10-29):

- **Name**: Chronicle Launcher, for now.
- **Dark Cloud is PAL only**: no NTSC.
- **The Moon People's assets** (the Chronicle logo, the ports' icon, Chronicle Title and Chronicle Sharp) stay in the launcher.
- **OpenRAC is a reference, not a base**: our own code and our own Dark Cloud take (§1, §1a).
- **Builds**: we plan to ship them, but not yet. Until then **Get the game** links to
  the port's GitHub (§3).
- **One build per game**: MoonFactory built in; mods off plays as the disc does.
- **Mods off**: MoonFactory reads `MOONMOD_NO_MODS=1` for the Mods switch (§4).

Open:

1. **Licensing of the ports**: may we redistribute Chronicle/ChronicleTwo builds?
   (Needed before phase 5.)
2. **License**: GPL-3.0-or-later is recommended. It matches the reference, so we
   can adapt its pieces (CHD reader, disc detection) with their notices. MIT would
   mean writing those from scratch.
