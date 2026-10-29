<!-- Set up from your disc: find it in a folder, then the port's extractor makes the game's data from it while the
     gold moon waxes. -->
<script lang="ts">
  import Moon from "./Moon.svelte";
  import { api } from "../lib/api";
  import { fileName } from "../lib/format";
  import { attempt, replace, store } from "../lib/store.svelte";
  import type { GameView, Scan, Stage } from "../lib/types";

  let { view }: { view: GameView } = $props();

  let folder: string | null = $state(null);
  let scan: Scan | null = $state(null);
  let looking = $state(false);

  const STAGE_WORDS: Record<Stage, string> = {
    starting: "Starting the extractor",
    "reading-disc": "Reading your disc",
    sorting: "Sorting the game's files",
    checking: "Checking every file",
    preparing: "Preparing textures, models and sound",
    done: "Done",
  };

  const game = $derived(view.game);
  const activity = $derived(view.activity);
  const lines = $derived(store.lines[game.id] ?? []);

  // Look again where the player last looked, once, when the page opens.
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

<section class="stone panel rise" id="disc">
  {#if activity.kind === "setting-up"}
    <div class="progress">
      <Moon lit={activity.fraction} label="Setting up" />
      <div>
        <h2 class="gold-text">Setting up {game.title}</h2>
        <p class="stage">{STAGE_WORDS[activity.stage]}…</p>
        <p class="soft">This reads the whole disc once. It can take a few minutes.</p>
        <button class="removes" onclick={() => attempt(() => api.cancelSetUp(game.id))}>Cancel</button>
      </div>
    </div>
    {#if lines.length}
      <details class="output">
        <summary>What it is doing</summary>
        <pre>{lines.slice(-40).join("\n")}</pre>
      </details>
    {/if}
  {:else}
    <h2 class="gold-text">Your disc</h2>
    <p class="soft">
      Your own {game.discs.map((d) => d.region).join(" or ")} disc image of {game.title}, as an
      <code>.iso</code>. The launcher looks in the folder you choose and two folders down. Nothing is
      downloaded.
    </p>
    <div class="row">
      <button onclick={pick}>{folder ? "Look in another folder" : "Choose a folder"}</button>
      {#if folder}<span class="where mono">{folder}</span>{/if}
    </div>

    {#if looking}
      <p class="soft">Looking…</p>
    {:else if scan}
      {#if scan.discs.length === 0}
        <p class="state warn">No {game.title} disc image in this folder.</p>
      {/if}
      <ul class="found">
        {#each scan.discs as disc (disc.path)}
          <li>
            <div>
              <strong>{fileName(disc.path)}</strong>
              <p class="soft">
                {disc.release ?? "A release the extractor does not list; it sets it up by its layout"}
              </p>
            </div>
            <button class="primary" onclick={() => begin(disc.path)}>Set up from this disc</button>
          </li>
        {/each}
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
  {/if}
</section>

<style>
  .panel {
    padding: 20px 24px;
    display: grid;
    gap: 12px;
  }
  h2 {
    font-size: 22px;
  }
  .soft {
    color: var(--on-stone-soft);
    font-size: 14px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
  }
  .where {
    color: var(--on-stone-dim);
    overflow-wrap: anywhere;
  }
  .found {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .found li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 0;
    border-top: 1px solid var(--stone-lo);
  }
  .found strong {
    font-family: var(--font-game);
    font-size: 16px;
  }
  .progress {
    display: flex;
    align-items: center;
    gap: 24px;
  }
  .progress > div {
    display: grid;
    gap: 6px;
    justify-items: start;
  }
  .stage {
    font-family: var(--font-game);
    font-size: 17px;
    color: var(--gold-hi);
  }
  details summary {
    cursor: pointer;
    color: var(--on-stone-soft);
  }
  .output pre {
    margin: 8px 0 0;
    max-height: 240px;
    overflow: auto;
    padding: 10px 12px;
    border-radius: var(--radius-plate);
    background: var(--stone-rim);
    color: var(--on-stone-soft);
    font: 12px/1.45 var(--font-mono);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .skipped ul {
    margin: 8px 0 0;
    padding-left: 18px;
    color: var(--on-stone-soft);
    font-size: 13px;
  }
</style>
