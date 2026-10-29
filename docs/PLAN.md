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
| Dark Cloud | [TheMoonPeople/Chronicle](https://github.com/TheMoonPeople/Chronicle) | `chronicle` | `darkcloud` | `dcdata` | PAL, NTSC 1.02 |
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
- **The moon is the through-line.** The Moon People made the ports; MoonFactory
  makes the mods. Progress (extraction, checks) is a **moon waxing** from a thin
  crescent to full, not a bar. The logo is a crescent. The Mods tab is called
  **Moon Factory** and wears the moon accent.
- **Each chapter has its own world.** Dark Cloud's page is night over Norune:
  indigo, starlight, gold. Dark Chronicle's is Palm Brinks at dusk: sepia, brass,
  warm light. Same layout, different light.
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

## 2. The look: dark, moonlit, gold

Keep in step with what we already made. **DC_Web** (`port/web/site/index.html`)
set the palette: near-black ink, a slightly lifted panel, soft white text, and
**Dark Cloud gold** for the one main action. MoonMod's tools use the same family
(`#1c1c24` canvas, `#aaaabb` labels). The launcher follows the reference's *structure rules*
(its `docs/STYLE.md`: one main action per page in a fixed place, rows not
boxes, secondary actions in a menu, removing in `--err`) and swaps its *colours*.

`src/app.css` on `:root`, OpenRAC token → Chronicle value:

| Token | OpenRAC (reference) | Chronicle | Source / use |
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
| `--bar-top`, `--bar-bottom` | orange | `#d4b05a`, `#b38a32` | primary buttons: gold, not orange. The header stays night-dark with a gold rule |
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
`Logo.svelte`. Ask TheMoonPeople whether we may
use the Chronicle README logo (`i.imgur.com/pCMkgRM.png`) before using it.
Game pictures (`public/img/<id>-bg.webp`): stills from our own play (the port's
`--screenshot`), each credited in `THIRD_PARTY_NOTICES.md`.

Write all of this into our own `docs/STYLE.md` in phase 1.

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
  player's choices. Needs a `--no-mods` flag in MoonFactory (agreed 2026-10-29):
  the game starts with no mod loaded and leaves `profile.json` as it is. It is a
  MoonMod change, added to both ports' patches, and lands before phase 3.
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
| 1 | The look: tokens, Cinzel/Inter, logo, two game themes, `docs/STYLE.md`, stylesheet tests | screenshots of the Library and a game page reviewed |
| 2 | Dark Cloud end to end: find disc, set up with `dcdata`, play, saves with backups, crash/log reports, **Get the game** link and **Choose a build**, checkout build | a PAL and an NTSC 1.02 disc each go from folder to playing |
| 3 | MoonMod: discover, enable, settings, install from zip/link, bundled mods, Mods switch | every bundled mod toggles in the launcher and the in-game manager agrees |
| 4 | Dark Chronicle: `dc2data` set-up and check, play, saves, MoonMod on `chronicle-two` | a PAL disc goes from folder to playing with a mod on |
| 5 | Builds and releases: `ports.yml` (Windows x64, Linux x64; macOS arm64 after), signed launcher installers, self-update | held for later; needs the ports' license OK (§3) |
| 6 | Later: settings pages from each port's `config.json`, save viewer then editor (Chronicle's `saves/1/save.dat`, ChronicleTwo's `dc2Ver4` JSON), decomp.dev progress band, Discord presence, DC2-Toolkit/dc1tk tools | each its own PR |

Platforms: Windows and Linux first (MoonMod's scripts are PowerShell today; the
Linux Flatpaks the ports build are a second source to support). macOS when the
ports' macOS builds are packaged. No Switch, iOS or Android.

## 7. Decisions and open questions

Decided (2026-10-29):

- **Name**: Chronicle Launcher, for now.
- **OpenRAC is a reference, not a base**: our own code and our own Dark Cloud take (§1, §1a).
- **Builds**: we plan to ship them, but not yet. Until then **Get the game** links to
  the port's GitHub (§3).
- **One build per game**: MoonFactory built in; mods off plays as the disc does.
- **`--no-mods`**: added to MoonFactory for the Mods switch (§4).

Open:

1. **Licensing of the ports**: may we redistribute Chronicle/ChronicleTwo builds?
   (Needed before phase 5.)
2. **Logo**: use the Chronicle README logo, or our own moon mark?
3. **License**: GPL-3.0-or-later is recommended. It matches the reference, so we
   can adapt its pieces (CHD reader, disc detection) with their notices. MIT would
   mean writing those from scratch.
