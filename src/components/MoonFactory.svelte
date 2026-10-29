<!-- Moon Factory: the game's MoonMod mods. Each is a row with its switch; a mod with settings opens a page of its own.
     The choices go to the same profile the in-game Mod Manager uses, so the two always agree. -->
<script lang="ts">
  import ModPage from "./ModPage.svelte";
  import { api } from "../lib/api";
  import { permissionWords } from "../lib/permissions";
  import { attempt, store, toast } from "../lib/store.svelte";
  import type { GameView, Mod, ModPreview } from "../lib/types";

  let { view }: { view: GameView } = $props();
  let mods = $state<Mod[] | null>(null);
  let open = $state<string | null>(null);
  let preview = $state<ModPreview | null>(null);

  const id = $derived(view.game.id);
  const playing = $derived(view.activity.kind === "playing");
  const current = $derived(mods?.find((m) => m.folder === open) ?? null);
  const usable = $derived(mods?.filter((m) => !m.problem) ?? []);
  const unusable = $derived(mods?.filter((m) => m.problem) ?? []);

  async function load() {
    mods = (await attempt(() => api.mods(id))) ?? null;
  }

  $effect(() => {
    void store.savesChanged;
    void id;
    void view.mods_dir;
    load();
  });

  async function toggle(mod: Mod, on: boolean) {
    await attempt(() => api.setModEnabled(id, mod.id, on));
    load();
  }

  async function pick(folder: boolean) {
    preview = (await attempt(() => api.pickMod(id, folder))) ?? null;
  }

  async function install() {
    const installed = await attempt(() => api.installMod(id));
    preview = null;
    if (installed) toast(`Added. Turn it on below.`, "ok");
    load();
  }

  async function cancel() {
    preview = null;
    await attempt(() => api.discardMod(id));
  }
</script>

<section class="stone factory rise">
  {#if current}
    <ModPage {id} mod={current} {playing} onBack={() => (open = null)} onChanged={load} />
  {:else}
    <div class="head">
      <h2><span class="mark" aria-hidden="true"></span><span class="gold-text">Moon Factory</span></h2>
      <div class="actions">
        <button onclick={() => pick(false)} disabled={playing}>Add a mod (.zip)</button>
        <button onclick={() => pick(true)} disabled={playing}>Add a folder</button>
        <button class="link" onclick={() => attempt(() => api.openFolder(id, "mods"))}
          >Open mods folder</button
        >
      </div>
    </div>
    <p class="line">
      MoonMod mods for {view.game.title}. They load only with <strong>Mods</strong> on under Play, and changes apply
      from the game's next start.
    </p>

    {#if preview}
      <div class="preview paper">
        <div>
          <h3>{preview.manifest.name} <span class="version">{preview.manifest.version}</span></h3>
          {#if preview.manifest.author}<p>By {preview.manifest.author}</p>{/if}
          {#if preview.manifest.description}<p>{preview.manifest.description}</p>{/if}
          {#if preview.manifest.permissions.length}
            <p class="can">It can: {permissionWords(preview.manifest.permissions).join("; ")}.</p>
          {/if}
          {#if !preview.for_this_game}<p class="warn">
              It is made for the other game, so this one will not load it.
            </p>{/if}
          {#if preview.replaces}<p class="warn">
              It replaces the one already here; its settings are kept.
            </p>{/if}
        </div>
        <div class="confirm">
          <button class="primary" onclick={install}>Add it</button>
          <button onclick={cancel}>Cancel</button>
        </div>
      </div>
    {/if}

    {#if mods && mods.length === 0}
      <p class="line empty">No mods yet. Add one, or put its folder in the mods folder.</p>
    {/if}
    <ul class="rows">
      {#each usable as mod (mod.folder)}
        <li>
          <input
            class="switch"
            type="checkbox"
            aria-label="{mod.name} on"
            checked={mod.enabled}
            disabled={playing}
            onchange={(e) => toggle(mod, e.currentTarget.checked)}
          />
          <div class="what">
            <strong>{mod.name}</strong>
            <span class="line">{[mod.version, mod.author].filter(Boolean).join(" · ")}</span>
          </div>
          {#if mod.native}
            <span class="badge" class:ok={mod.native_trusted}
              >{mod.native_trusted ? "Trusted" : "Needs trusting in the game"}</span
            >
          {:else}
            <span></span>
          {/if}
          <button class="link more" onclick={() => (open = mod.folder)}>
            {mod.settings.length ? "Settings ›" : "About ›"}
          </button>
        </li>
      {/each}
    </ul>
    {#if unusable.length}
      <details class="unusable">
        <summary>Not loaded ({unusable.length})</summary>
        <ul>
          {#each unusable as mod (mod.folder)}
            <li><span class="mono">{mod.folder}</span>: {mod.problem}</li>
          {/each}
        </ul>
      </details>
    {/if}
  {/if}
</section>

<style>
  .factory {
    padding: 20px 24px;
    display: grid;
    gap: 10px;
    box-shadow:
      inset 2px 2px 0 var(--bevel-light),
      inset -2px -2px 0 var(--bevel-dark),
      0 0 0 2px var(--stone-rim),
      0 0 0 4px var(--moon-blue),
      0 10px 24px var(--shadow);
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  h2 {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 22px;
  }
  .mark {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--moon-blue);
    box-shadow:
      inset -5px -2px 0 var(--slate-rim),
      0 0 10px var(--moon-blue);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }
  .line {
    color: var(--on-stone-soft);
    font-size: 14px;
  }
  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .rows li {
    display: grid;
    grid-template-columns: auto 1fr auto auto;
    gap: 16px;
    align-items: center;
    padding: 12px 0;
    border-top: 1px solid var(--stone-lo);
  }
  .what {
    display: grid;
  }
  .what strong {
    font-family: var(--font-game);
    font-size: 17px;
  }
  .what .line {
    font-size: 13px;
  }
  .badge {
    font-size: 12px;
    padding: 3px 10px;
    border-radius: 999px;
    color: var(--gold-hi);
    border: 1px solid var(--gold-mid);
  }
  .badge.ok {
    color: var(--green-light);
    border-color: var(--green);
  }
  .more {
    white-space: nowrap;
  }
  .preview {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 20px;
  }
  .preview h3 {
    font-size: 20px;
    color: var(--bronze-deep);
  }
  .preview p {
    color: var(--ink-soft);
    font-size: 14px;
  }
  .preview .version {
    font: 14px var(--font-sans);
    color: var(--ink-soft);
  }
  .preview .warn {
    color: var(--terracotta-rim);
  }
  .confirm {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .unusable summary {
    cursor: pointer;
    color: var(--on-stone-soft);
  }
  .unusable ul {
    margin: 8px 0 0;
    padding-left: 18px;
    color: var(--on-stone-soft);
    font-size: 13px;
  }
  @media (max-width: 560px) {
    .rows li {
      grid-template-columns: auto 1fr;
    }
  }
</style>
