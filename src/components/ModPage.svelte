<!-- One mod's own page: what it is, what it is allowed to do, and its settings. -->
<script lang="ts">
  import ModSettingRow from "./ModSettingRow.svelte";
  import { api } from "../lib/api";
  import { permissionWords } from "../lib/permissions";
  import { attempt, toast } from "../lib/store.svelte";
  import type { Mod } from "../lib/types";

  let {
    id,
    mod,
    playing,
    onBack,
    onChanged,
  }: { id: string; mod: Mod; playing: boolean; onBack: () => void; onChanged: () => void } = $props();
  let removing = $state(false);

  async function change(setting: string, value: unknown) {
    const done = await attempt(async () => {
      await api.setModSetting(id, mod.id, setting, value);
      return true;
    });
    if (done)
      toast(
        value === null ? "Put back to its default." : "Saved. The game uses it from its next start.",
        "ok",
      );
    onChanged();
  }

  async function remove() {
    removing = false;
    const done = await attempt(async () => {
      await api.removeMod(id, mod.folder);
      return true;
    });
    if (done) {
      toast(`${mod.name} removed.`, "ok");
      onChanged();
      onBack();
    }
  }
</script>

<div class="page">
  <button class="link back" onclick={onBack}>‹ Moon Factory</button>
  <h3 class="gold-text">{mod.name}</h3>
  <p class="by">{[mod.version && `Version ${mod.version}`, mod.author].filter(Boolean).join(" · ")}</p>
  {#if mod.description}<p class="description">{mod.description}</p>{/if}

  {#if mod.permissions.length}
    <h4>It can</h4>
    <ul class="can">
      {#each permissionWords(mod.permissions) as words (words)}<li>{words}</li>{/each}
    </ul>
  {/if}
  {#if mod.native}
    <p class="state {mod.native_trusted ? 'ok' : 'warn'}">
      {mod.native_trusted
        ? "Trusted in the game"
        : "Runs program code of its own: trust it in the game's Mod Manager before it loads"}
    </p>
  {/if}

  {#if mod.settings.length}
    <h4>Settings</h4>
    {#if playing}<p class="note">Close the game to change these.</p>{/if}
    {#each mod.settings as setting (setting.id)}
      <ModSettingRow {setting} disabled={playing} onChange={(value) => change(setting.id, value)} />
    {/each}
  {/if}

  <div class="end">
    {#if removing}
      <span>Remove {mod.name}? Its settings are kept, if you add it again.</span>
      <button class="removes" onclick={remove}>Remove</button>
      <button onclick={() => (removing = false)}>Keep it</button>
    {:else}
      <button class="removes" disabled={playing} onclick={() => (removing = true)}>Remove this mod</button>
    {/if}
  </div>
</div>

<style>
  .page {
    display: grid;
    gap: 6px;
  }
  .back {
    justify-self: start;
    margin-bottom: 8px;
  }
  h3 {
    font-size: 26px;
  }
  h4 {
    margin: 14px 0 2px;
    font: 700 15px/1 var(--font-title);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--moon-pale);
  }
  .by,
  .note {
    color: var(--on-stone-soft);
    font-size: 13px;
  }
  .description {
    color: var(--on-stone-soft);
  }
  .can {
    margin: 0;
    padding-left: 18px;
    color: var(--on-stone-soft);
    font-size: 14px;
  }
  .end {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    margin-top: 16px;
    padding-top: 14px;
    border-top: 1px solid var(--stone-lo);
    color: var(--on-stone-soft);
  }
</style>
