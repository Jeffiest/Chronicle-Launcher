<!-- The game's saves, as its own load screen lists them: each with its play time (and Dark Cloud's day). Backups of
     them all, and restoring one. -->
<script lang="ts">
  import { api } from "../lib/api";
  import { playTime, when } from "../lib/format";
  import { attempt, store, toast } from "../lib/store.svelte";
  import type { GameView, SavesView } from "../lib/types";

  let { view }: { view: GameView } = $props();
  let saves = $state<SavesView | null>(null);
  let restoring = $state(false);
  let confirming = $state<string | null>(null);

  const id = $derived(view.game.id);
  const playing = $derived(view.activity.kind === "playing");
  const word = $derived(view.game.saves === "dark-chronicle" ? "File" : "Save");

  async function load() {
    saves = (await attempt(() => api.saves(id))) ?? null;
  }

  $effect(() => {
    void store.savesChanged;
    void id;
    load();
  });

  async function backUp() {
    const made = await attempt(() => api.backUpSaves(id));
    if (made === null) toast("No saves to back up yet.");
    if (made) toast(`Backed up ${made.saves} ${made.saves === 1 ? "save" : "saves"}.`, "ok");
    load();
  }

  async function restore(backup: string) {
    confirming = null;
    const done = await attempt(async () => {
      await api.restoreSaves(id, backup);
      return true;
    });
    if (done) toast("Restored. The saves from before are kept as a backup too.", "ok");
    load();
  }
</script>

<section class="window rise">
  <h2 class="window-title">{restoring ? "Backups" : "Save files"}</h2>
  <div class="rule"></div>
  {#if !restoring}
    <ul class="rows">
      {#if saves && saves.saves.length === 0}
        <li class="row faint">No saves yet</li>
      {/if}
      {#each saves?.saves ?? [] as save, i (save.name)}
        <li class="row" class:chosen={i === 0}>
          <span class="name">{save.number === null ? "Auto" : `${word} ${save.number}`}</span>
          {#if save.readable}
            <span class="grow">{playTime(save.play_seconds ?? 0)}</span>
            {#if save.day !== null}<span class="small">Day {save.day}</span>{/if}
          {:else}
            <span class="grow small">not readable</span>
          {/if}
        </li>
      {/each}
    </ul>
    <div class="footer">
      <button class="key" onclick={backUp} disabled={playing}>Back up</button>
      <button class="link" onclick={() => attempt(() => api.openFolder(id, "saves"))}>Open folder</button>
      {#if saves?.backups.length}<button class="link" onclick={() => (restoring = true)}>Backups</button>{/if}
    </div>
  {:else}
    <ul class="rows">
      {#each saves?.backups ?? [] as backup (backup.id)}
        <li class="row" class:chosen={confirming === backup.id}>
          <span class="grow">{when(backup.made)}</span>
          <span class="small">{backup.saves} {backup.saves === 1 ? "save" : "saves"}</span>
          {#if confirming === backup.id}
            <button class="key" onclick={() => restore(backup.id)}>Replace mine</button>
          {:else}
            <button class="link" disabled={playing} onclick={() => (confirming = backup.id)}>Restore</button>
          {/if}
        </li>
      {/each}
    </ul>
    <div class="footer">
      <button class="key" onclick={() => ((restoring = false), (confirming = null))}>Back</button>
      <span>What is there now is backed up first</span>
    </div>
  {/if}
</section>

<style>
  .name {
    min-width: 5.2em;
  }
  .grow {
    flex: 1;
  }
  .small {
    font-size: 0.82em;
    color: var(--text-soft);
  }
  .chosen .small {
    color: var(--accent);
  }
</style>
