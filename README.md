<div align="center">

# Chronicle Launcher

**Dark Cloud and Dark Chronicle on PC, from your own discs, with MoonMod.**

</div>

The launcher for The Moon People's native ports,
[Chronicle](https://github.com/TheMoonPeople/Chronicle) (Dark Cloud) and
[Chronicle Two](https://github.com/TheMoonPeople/ChronicleTwo) (Dark Chronicle), with mods from
[MoonMod](https://github.com/Jeffiest/MoonMod). The two games are the Chronicle's two chapters: pick one, and its
page says the one next step: **Get the game**, **Set up from your disc**, **Play**.

Nothing from the games is included or downloaded. You supply your own disc.

| The Chronicle                                          | A chapter                                                             |
| ------------------------------------------------------ | --------------------------------------------------------------------- |
| ![The two chapters' covers](docs/screenshots/home.png) | ![Dark Cloud's page, with its steps](docs/screenshots/dark-cloud.png) |

## Where it stands

Phase 0 of [the plan](docs/PLAN.md): the two games, the Chronicle home and each game's page. **Get the game**
opens the port's GitHub (the launcher ships no builds yet), and **Choose a build** remembers the folder with your
build of the port. Setting up from your disc and playing come next.

## Building

Tauri 2: Rust does every file and process operation (`core/`, `src-tauri/`), Svelte 5 draws the UI (`src/`).
Linux needs Tauri's libraries (`libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev`).

```sh
npm install
npm run dev          # the UI alone in a browser, on mock data (src/lib/api.ts)
npx tauri dev        # the app
npx tauri build      # an installer
npm run verify       # typecheck, format, UI tests
cargo test --workspace
```

`CHRONICLE_LAUNCHER_HOME=<folder>` keeps the launcher's settings and game folders somewhere other than its usual
data folder.

| Read                                   | For                                       |
| -------------------------------------- | ----------------------------------------- |
| [docs/PLAN.md](docs/PLAN.md)           | what we are building, in what order       |
| [docs/REFERENCE.md](docs/REFERENCE.md) | what we learned from the OpenRAC launcher |
| [games/README.md](games/README.md)     | how a game is described                   |
