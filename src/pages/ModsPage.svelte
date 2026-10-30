<!-- Moon Factory: each game's MoonMod mods, laid out as the in-game Mod Manager is (MoonMod src/manager_ui.cpp): the
     list on the left with each mod's state, the chosen mod on the right with its tabs, the control hints below. The
     choices go to the same profile.json that manager reads, so the two always agree. -->
<script lang="ts">
  import Backdrop from "../components/Backdrop.svelte";
  import ModSettingRow from "../components/ModSettingRow.svelte";
  import { api } from "../lib/api";
  import { GAMES } from "../lib/games";
  import { permissionWords } from "../lib/permissions";
  import { attempt, store, toast } from "../lib/store.svelte";
  import type { GameView, Mod, ModPreview } from "../lib/types";

  let { game: initial, onGame }: { game: string; onGame: (id: string) => void } = $props();
  let mods = $state<Mod[] | null>(null);
  let chosen = $state<string | null>(null);
  let tab = $state<"settings" | "about" | "can">("settings");
  let preview = $state<ModPreview | null>(null);
  let removing = $state(false);

  const id = $derived(initial);
  const view = $derived(store.views.find((v) => v.game.id === id) as GameView | undefined);
  const playing = $derived(view?.activity.kind === "playing");
  const current = $derived(mods?.find((m) => m.folder === chosen) ?? mods?.[0] ?? null);

  async function load() {
    mods = (await attempt(() => api.mods(id))) ?? null;
  }

  $effect(() => {
    void store.savesChanged;
    void id;
    load();
  });

  function stateOf(mod: Mod): string {
    if (mod.problem) return "BROKEN";
    if (mod.native && !mod.native_trusted && mod.enabled) return "TRUST";
    return mod.enabled ? "ON" : "OFF";
  }

  async function toggle(mod: Mod) {
    if (mod.problem) return;
    await attempt(() => api.setModEnabled(id, mod.id, !mod.enabled));
    load();
  }

  async function change(mod: Mod, setting: string, value: unknown) {
    const done = await attempt(async () => {
      await api.setModSetting(id, mod.id, setting, value);
      return true;
    });
    if (done)
      toast(
        value === null ? "Put back to its default." : "Saved. It applies from the game's next start.",
        "ok",
      );
    load();
  }

  async function pick(folder: boolean) {
    preview = (await attempt(() => api.pickMod(id, folder))) ?? null;
  }

  async function install() {
    const installed = await attempt(() => api.installMod(id));
    preview = null;
    if (installed) {
      toast("Added. Turn it on in the list.", "ok");
      chosen = installed;
    }
    load();
  }

  async function remove(mod: Mod) {
    removing = false;
    const done = await attempt(async () => {
      await api.removeMod(id, mod.folder);
      return true;
    });
    if (done) {
      toast(`${mod.name} removed. Its settings are kept.`, "ok");
      chosen = null;
    }
    load();
  }
</script>

<main data-game={id}>
  <Backdrop
    chapter={GAMES.find((g) => g.id === id)?.chapter ?? 1}
    scene={store.scenes[id] ?? null}
    blur={true}
  />

  <section class="window manager rise" aria-label="Moon Factory">
    <div class="head">
      <h1 class="window-title">Moon Factory / Mod Manager</h1>
      <span class="choice" aria-label="Game">
        {#each GAMES as g (g.id)}
          <button aria-pressed={g.id === id} onclick={() => onGame(g.id)}>{g.title}</button>
        {/each}
      </span>
    </div>

    <div class="panes">
      <div class="pane">
        <h2 class="window-title">Mods</h2>
        <div class="rule"></div>
        <ul class="rows">
          {#if mods && mods.length === 0}<li class="row faint">No mods yet</li>{/if}
          {#each mods ?? [] as mod (mod.folder)}
            <li>
              <button
                class="row"
                class:chosen={current?.folder === mod.folder}
                class:faint={mod.problem !== null}
                onclick={() => ((chosen = mod.folder), (preview = null), (removing = false))}
                ondblclick={() => toggle(mod)}
              >
                <span class="grow">{mod.name}</span><span class:on={mod.enabled && !mod.problem}
                  >{stateOf(mod)}</span
                >
              </button>
            </li>
          {/each}
        </ul>
      </div>

      <div class="pane">
        {#if preview}
          <h2 class="window-title">Add {preview.manifest.name}?</h2>
          <div class="rule"></div>
          <div class="body">
            <p class="soft">
              Version {preview.manifest.version}{preview.manifest.author
                ? ` / ${preview.manifest.author}`
                : ""}
            </p>
            {#if preview.manifest.description}<p>{preview.manifest.description}</p>{/if}
            {#if preview.manifest.permissions.length}<p class="soft">
                It can: {permissionWords(preview.manifest.permissions).join("; ")}.
              </p>{/if}
            {#if !preview.for_this_game}<p class="warn">
                It is made for the other game, so this one will not load it.
              </p>{/if}
            {#if preview.replaces}<p class="warn">
                It replaces the one already here; its settings are kept.
              </p>{/if}
            <div class="buttons">
              <button class="key" onclick={install}>Add it</button>
              <button class="link" onclick={() => ((preview = null), attempt(() => api.discardMod(id)))}
                >Cancel</button
              >
            </div>
          </div>
        {:else if current}
          <h2 class="window-title">{current.name}</h2>
          <div class="tabs choice" aria-label="Mod">
            <button aria-pressed={tab === "settings"} onclick={() => (tab = "settings")}>Settings</button>
            <button aria-pressed={tab === "about"} onclick={() => (tab = "about")}>About</button>
            <button aria-pressed={tab === "can"} onclick={() => (tab = "can")}>It can</button>
          </div>
          <div class="body">
            {#if current.problem}
              <p class="warn">{current.problem}</p>
            {:else if tab === "settings"}
              <div class="toggle">
                <span>Turned on</span>
                <span class="choice">
                  <button
                    aria-pressed={current.enabled}
                    disabled={playing}
                    onclick={() => !current.enabled && toggle(current)}>On</button
                  >
                  <button
                    aria-pressed={!current.enabled}
                    disabled={playing}
                    onclick={() => current.enabled && toggle(current)}>Off</button
                  >
                </span>
              </div>
              {#each current.settings as setting (setting.id)}
                <ModSettingRow
                  {setting}
                  disabled={playing}
                  onChange={(value) => change(current, setting.id, value)}
                />
              {/each}
              {#if current.settings.length === 0}<p class="soft">This mod has no settings.</p>{/if}
              {#if current.native && !current.native_trusted}<p class="warn">
                  It runs program code of its own: trust it in the game's Mod Manager before it loads.
                </p>{/if}
            {:else if tab === "about"}
              <p class="soft">Version {current.version}{current.author ? ` / ${current.author}` : ""}</p>
              {#if current.description}<p>{current.description}</p>{/if}
              {#if current.dependencies.length}<p class="soft">
                  Needs: {current.dependencies.join(", ")}
                </p>{/if}
              {#if removing}
                <div class="buttons">
                  <button class="key" onclick={() => remove(current)}>Remove it</button><button
                    class="link"
                    onclick={() => (removing = false)}>Keep it</button
                  >
                </div>
              {:else}
                <div class="buttons">
                  <button class="link" disabled={playing} onclick={() => (removing = true)}
                    >Remove this mod</button
                  >
                </div>
              {/if}
            {:else}
              <ul class="can">
                {#each permissionWords(current.permissions) as words (words)}<li>{words}</li>{/each}
                {#if current.permissions.length === 0}<li>Nothing beyond running its script</li>{/if}
              </ul>
            {/if}
          </div>
        {:else}
          <h2 class="window-title">No mod chosen</h2>
          <div class="rule"></div>
          <p class="body soft">Add a mod, or put its folder in the mods folder.</p>
        {/if}
      </div>
    </div>

    <div class="footer">
      <span>Double-click on/off</span>
      <button class="key" disabled={playing} onclick={() => pick(false)}>Add a mod</button>
      <button class="link" disabled={playing} onclick={() => pick(true)}>Add a folder</button>
      <button class="link" onclick={() => attempt(() => api.openFolder(id, "mods"))}>Mods folder</button>
      <span class="end">{playing ? "Close the game to change mods" : "Applies next start"}</span>
    </div>
  </section>
</main>

<style>
  main {
    position: relative;
    height: 100%;
    overflow: hidden;
  }
  .manager {
    position: absolute;
    inset: 36px 40px 40px 40px;
    display: flex;
    flex-direction: column;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding-right: 18px;
  }
  .head h1 {
    font-size: calc(24px * var(--game-size));
  }
  .panes {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(240px, 2fr) minmax(0, 3fr);
    gap: 10px;
    padding: 0 12px 12px;
  }
  .pane {
    min-height: 0;
    overflow: auto;
    background: var(--win);
    box-shadow: inset 0 0 0 1px var(--footer);
  }
  .rows li {
    list-style: none;
  }
  .rows button.row {
    width: 100%;
    border: 0;
    background: var(--footer);
    text-align: left;
  }
  .rows button.row.chosen {
    background: var(--select);
  }
  .rows button.row.faint {
    background: none;
  }
  .grow {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .on {
    color: var(--accent);
  }
  .tabs {
    margin: 0 12px;
  }
  .body {
    padding: 10px 18px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-family: var(--font-game);
    font-size: calc(16px * var(--game-size));
  }
  .soft {
    color: var(--text-soft);
  }
  .warn {
    color: var(--err);
  }
  .toggle {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 0;
    box-shadow: inset 0 -2px 0 var(--frame);
  }
  .buttons {
    display: flex;
    gap: 16px;
    align-items: center;
    margin-top: 6px;
  }
  .can {
    margin: 0;
    padding-left: 20px;
    color: var(--text-soft);
  }
  .end {
    margin-left: auto;
    color: var(--text-dim);
  }
</style>
