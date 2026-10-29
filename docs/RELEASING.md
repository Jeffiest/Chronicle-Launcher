# Releasing the launcher

The launcher's installers are built by `.github/workflows/release.yml` on Windows, Linux and macOS. They hold
nothing from the games.

1. Pick the version and write it into the three places it lives (`package.json`, `src-tauri/tauri.conf.json`,
   `Cargo.toml`):

   ```sh
   node scripts/version.mjs set 0.2.0
   cargo check -p chronicle-launcher    # updates Cargo.lock
   git commit -am "Chronicle Launcher 0.2.0"
   ```

2. Tag it and push the tag:

   ```sh
   git tag v0.2.0
   git push origin v0.2.0
   ```

   The workflow refuses a tag that is not the version in all three files.

3. It makes a **draft** release with the installers: Windows `-setup.exe` and `.msi`, Linux `.AppImage` and `.deb`,
   macOS `.dmg`. Try one, then publish the draft on GitHub.

To build the installers without a release, run the workflow by hand (Actions, **release**, **Run workflow**); they
are kept as the run's artifacts.

The installers are not signed yet: Windows SmartScreen warns, and macOS needs right-click, Open the first time.
Signing needs a certificate (Windows) and an Apple Developer ID (macOS), set up as the workflow's secrets.
