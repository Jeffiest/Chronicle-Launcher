<!-- The game's saves, on Paper, as its save board lists them: the autosave, then each numbered save with its play
     time and day. Backups of them all, and restoring one. -->
<script lang="ts">
  import { api } from "../lib/api";
  import { playTime, when } from "../lib/format";
  import { attempt, store, toast } from "../lib/store.svelte";
  import type { GameView, SavesView } from "../lib/types";

  let { view }: { view: GameView } = $props();
  let saves: SavesView | null = $state(null);
  let confirming: string | null = $state(null);

  const id = $derived(view.game.id);
  const playing = $derived(view.activity.kind === "playing");

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

<section class="paper panel rise">
  <div class="head">
    <h2>Saves</h2>
    <div class="actions">
      <button onclick={backUp} disabled={playing}>Back up now</button>
      <button class="link" onclick={() => attempt(() => api.openFolder(id, "saves"))}>Open folder</button>
    </div>
  </div>

  {#if saves && saves.saves.length === 0}
    <p class="line">No saves yet. They appear here once you save in the game.</p>
  {:else if saves}
    <ul class="rows">
      {#each saves.saves as save (save.name)}
        <li>
          <span class="name">{save.number ?? "Auto"}</span>
          {#if save.readable}
            <span class="time">{playTime(save.play_seconds ?? 0)}</span>
            <span class="day">Day {save.day}</span>
          {:else}
            <span class="time unreadable">Not a save the launcher can read</span>
            <span></span>
          {/if}
          <span class="when">{save.modified ? when(save.modified) : ""}</span>
        </li>
      {/each}
    </ul>
  {/if}

  {#if saves && saves.backups.length}
    <details class="backups">
      <summary>Backups ({saves.backups.length})</summary>
      <ul class="rows">
        {#each saves.backups as backup (backup.id)}
          <li>
            <span class="when">{when(backup.made)}</span>
            <span class="line">{backup.saves} {backup.saves === 1 ? "save" : "saves"}</span>
            {#if confirming === backup.id}
              <span class="confirm">
                <button class="removes" onclick={() => restore(backup.id)}>Replace my saves</button>
                <button onclick={() => (confirming = null)}>Keep them</button>
              </span>
            {:else}
              <button onclick={() => (confirming = backup.id)} disabled={playing}>Restore</button>
            {/if}
          </li>
        {/each}
      </ul>
    </details>
  {/if}
</section>

<style>
  .panel {
    padding: 20px 24px 14px;
    display: grid;
    gap: 8px;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  h2 {
    font-size: 22px;
    color: var(--bronze-deep);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .actions .link,
  summary {
    color: var(--bronze-deep);
    text-shadow: none;
  }
  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .rows li {
    display: grid;
    grid-template-columns: 56px 1fr auto auto;
    gap: 16px;
    align-items: center;
    padding: 10px 0;
    border-top: 1px solid var(--paper-lo);
  }
  .backups .rows li {
    grid-template-columns: 1fr 1fr auto;
  }
  .name {
    font: 700 22px/1 var(--font-title);
    color: var(--bronze);
    text-align: center;
  }
  .time,
  .day {
    font-family: var(--font-game);
    font-size: 17px;
    color: var(--ink);
  }
  .unreadable {
    font-size: 14px;
    color: var(--ink-soft);
  }
  .when,
  .line {
    color: var(--ink-soft);
    font-size: 13px;
  }
  .confirm {
    display: flex;
    gap: 10px;
  }
  summary {
    cursor: pointer;
    font-family: var(--font-title);
    margin-top: 6px;
  }
  @media (max-width: 560px) {
    .rows li {
      grid-template-columns: 44px 1fr;
    }
  }
</style>
