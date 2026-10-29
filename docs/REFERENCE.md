# The reference: OpenRAC's launcher

[OpenRAC's launcher](https://github.com/OpenRAC/OpenRAC/tree/main/launcher) is to us what
[OpenGOAL's launcher](https://github.com/open-goal/launcher) was to OpenRAC: a reference, not a base. We study
how it works and what players liked, and we write our own launcher with its own Dark Cloud take
([PLAN.md](PLAN.md) §1a). No OpenRAC file is copied here. If one ever is, it keeps its GPL-3.0-or-later notice
and is listed in `THIRD_PARTY_NOTICES.md`.

Last studied: OpenRAC `main` at `93ac558` (2026-10-29).

## Ideas taken, and where they are here

| Idea | Reference | Here |
| --- | --- | --- |
| Tauri 2, Rust for files and processes, Svelte 5 for the UI | `launcher/core`, `src-tauri`, `src` | the same split: `core/`, `src-tauri/`, `src/` |
| A page's main button names the next step; rarer actions in a menu, removing last | `GamePlayer.svelte`, `Menu.svelte` | `src/pages/GamePage.svelte`, `src/components/Menu.svelte` |
| A state is a dot and a few words | `docs/STYLE.md` | `.state` in `src/app.css` |
| Steps in order as a numbered list | `ol.steps` | `ol.steps` in `src/app.css` |
| Colours only as tokens; hover only in `@media (hover: hover)`, held by a test | `stylesheets.test.ts` | `src/lib/stylesheets.test.ts` |
| The UI runs in a browser on mock data | `mockEnhanced.ts` | `mockApi` in `src/lib/api.ts` |

## Not taken

Everything Ratchet & Clank's own: the save editor, Enhanced graphics, the Godot level editor, PCSX2 card import,
multiplayer, the game tiles and their orange. Our home is the Chronicle's two chapter covers instead of a game
list, and our look is the games' own: the Chronicle logo's sky and gold, the ports' icon, and the port's Blue
Stone, Paper and plate materials (PLAN.md §2).
