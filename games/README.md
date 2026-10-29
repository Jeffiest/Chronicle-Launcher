# Games

One file per game. The Rust core (`core/src/games.rs`) and the UI (`src/lib/games.ts`) both read these, so a
game is described in one place.

| Key                       | Meaning                                                                                                  |
| ------------------------- | -------------------------------------------------------------------------------------------------------- |
| `id`                      | the launcher's id; the same as MoonMod's manifest `games` value                                          |
| `title`, `chapter`        | how the home page names it: _Chapter `chapter`: `title`_                                                 |
| `port`, `repository`      | the native port and its source                                                                           |
| `get_url`                 | where **Get the game** goes while the launcher ships no builds (the port's GitHub, later a release page) |
| `executable`, `extractor` | the port's program and its disc extractor, without `.exe`                                                |
| `hero`                    | who the saves are about                                                                                  |
| `discs`                   | the releases the port accepts, and the file name the port's own scripts expect                           |
