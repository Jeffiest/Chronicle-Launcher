<!-- Your disc: found in a folder, then made into the game's data by the port's own extractor, while Dark Cloud's
     gold moon waxes or Dark Chronicle's gold ring fills. -->
<script lang="ts">
  import Moon from "./Moon.svelte";
  import Ring from "./Ring.svelte";
  import { api } from "../lib/api";
  import { fileName } from "../lib/format";
  import { attempt, replace, store } from "../lib/store.svelte";
  import type { GameView, Scan } from "../lib/types";

  let { view }: { view: GameView } = $props();
  let folder = $state<string | null>(null);
  let scan = $state<Scan | null>(null);
  let looking = $state(false);

  const game = $derived(view.game);
  const activity = $derived(view.activity);
  const lines = $derived(store.lines[game.id] ?? []);

  $effect(() => {
    const remembered = view.disc_folder;
    if (remembered && scan === null && !looking) {
      looking = true;
      attempt(() => api.scanDiscs(game.id, remembered)).then((found) => {
        folder = remembered;
        scan = found ?? null;
        looking = false;
      });
    }
  });

  async function pick() {
    const picked = await attempt(() => api.pickDiscFolder(game.id, folder));
    if (picked) {
      folder = picked.folder;
      scan = picked.scan;
    }
  }

  async function begin(disc: string) {
    store.lines[game.id] = [];
    replace(await attempt(() => api.setUp(game.id, disc)));
  }
</script>

<section class="window rise" id="disc">
  {#if activity.kind === "setting-up"}
    <h2 class="window-title">Setting up</h2>
    <div class="rule"></div>
    <div class="progress">
      {#if game.chapter === 1}
        <Moon lit={activity.fraction} size={96} label="Setting up" />
      {:else}
        <Ring lit={activity.fraction} size={120} label="Setting up" />
      {/if}
      <div>
        <p class="percent">{Math.round(activity.fraction * 100)}%</p>
        <p class="doing">{activity.doing}</p>
      </div>
    </div>
    {#if lines.length}
      <details class="output">
        <summary>What it is doing</summary>
        <pre>{lines.slice(-40).join("\n")}</pre>
      </details>
    {/if}
    <div class="footer">
      <button class="key" onclick={() => attempt(() => api.cancelSetUp(game.id))}>Cancel</button>
      <span>It reads the whole disc once</span>
    </div>
  {:else}
    <h2 class="window-title">Your disc</h2>
    <div class="rule"></div>
    <p class="say">
      Your own {game.discs.map((d) => d.region).join(" or ")} disc image of {game.title}, as an .iso.
    </p>
    {#if looking}
      <p class="say">Looking…</p>
    {:else if scan}
      <ul class="rows">
        {#each scan.discs as disc, i (disc.path)}
          <li>
            <button class="row" class:chosen={i === 0} onclick={() => begin(disc.path)}>
              <span class="grow">{fileName(disc.path)}</span><span class="release">{disc.release}</span>
            </button>
          </li>
        {/each}
        {#if scan.discs.length === 0}
          <li class="row faint">No {game.title} disc image here</li>
        {/if}
      </ul>
      {#if scan.skipped.length}
        <details class="skipped">
          <summary>Not used ({scan.skipped.length})</summary>
          <ul>
            {#each scan.skipped as skip (skip.path)}
              <li><span class="mono">{fileName(skip.path)}</span>: {skip.why}</li>
            {/each}
          </ul>
        </details>
      {/if}
    {/if}
    <div class="footer">
      <button class="key" onclick={pick}>{folder ? "Look elsewhere" : "Choose a folder"}</button>
      <span>{scan?.discs.length ? "Enter on a disc to set it up" : "Looks two folders down"}</span>
    </div>
  {/if}
</section>

<style>
  .say {
    padding: 12px 18px 2px;
    font-size: calc(16px * var(--game-size));
    line-height: 1.5;
    color: var(--text-soft);
  }
  .rows li {
    list-style: none;
  }
  .rows button.row {
    width: 100%;
    border: 0;
    background: none;
    text-align: left;
  }
  .rows button.row.chosen {
    background: var(--select);
  }
  .grow {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .release {
    font-size: 0.8em;
    color: var(--text-soft);
  }
  .progress {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 16px 18px;
  }
  .percent {
    font-family: var(--font-title);
    font-size: 40px;
    line-height: 1;
    color: var(--title);
  }
  .doing {
    margin-top: 6px;
    font-size: calc(16px * var(--game-size));
    color: var(--accent);
  }
  details {
    padding: 0 18px 10px;
    font-size: calc(14px * var(--game-size));
    color: var(--text-soft);
  }
  summary {
    cursor: pointer;
  }
  pre {
    max-height: 160px;
    overflow: auto;
    margin: 8px 0 0;
    padding: 8px 10px;
    background: var(--frame-dark);
    font: 12px/1.45 var(--font-mono);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .skipped ul {
    margin: 6px 0 0;
    padding-left: 18px;
  }
</style>
