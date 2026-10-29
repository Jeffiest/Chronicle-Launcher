# MoonMod

`manifest.schema.json` is MoonMod's mod manifest schema, from [Jeffiest/MoonMod](https://github.com/Jeffiest/MoonMod)
at `7cbf45c`. `core/src/mods.rs` checks manifests by the same rules as MoonFactory's own reader (`src/manifest.cpp`),
and its tests fail if the schema gains a field the launcher does not know.
