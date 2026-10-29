<!-- A chapter's page: its sky and name, the one next step, and the steps to get there. -->
<script lang="ts">
  import Menu, { type Item } from "../components/Menu.svelte";
  import { chapterName, themeClass } from "../lib/games";
  import { MAIN_BUTTON, stateOf } from "../lib/next";
  import type { GameView } from "../lib/types";

  let {
    view,
    onBack,
    onGet,
    onChoose,
    onForget,
  }: {
    view: GameView;
    onBack: () => void;
    onGet: () => void;
    onChoose: () => void;
    onForget: () => void;
  } = $props();

  const game = $derived(view.game);
  const state = $derived(stateOf(view));
  const regions = $derived(game.discs.map((d) => d.region).join(" or "));
  // Set-up and Play come in the next phase (docs/PLAN.md §6); until then their button waits.
  const waiting = $derived(view.next !== "get-the-game");

  const items: Item[] = $derived([
    { label: view.build ? "Choose another build" : "Choose a build", run: onChoose },
    { label: `${game.port} on GitHub`, run: onGet },
    ...(view.build ? [{ label: "Forget this build", run: onForget, removes: true }] : []),
  ]);
</script>

<main>
  <section class="band backdrop {themeClass(game.chapter)}">
    <div class="inner">
      <button class="link back" onclick={onBack}>‹ The Chronicle</button>
      <span class="chapter">{chapterName(game.chapter)}</span>
      <h1>{game.title}</h1>
      <p class="hero">{game.hero}</p>
      <div class="actions">
        <button class="primary" disabled={waiting} onclick={onGet}>{MAIN_BUTTON[view.next]}</button>
        {#if view.next === "get-the-game"}
          <button onclick={onChoose}>Choose a build</button>
        {/if}
        <Menu {items} />
        <span class="state {state.tone}">{state.words}</span>
      </div>
      {#if waiting}
        <p class="note">Setting up from your disc arrives in the launcher's next update.</p>
      {/if}
    </div>
  </section>

  <section class="body">
    <div class="panel rise">
      <h2>Getting there</h2>
      <ol class="steps">
        <li class:done={view.build !== null}>
          <div>
            <strong>Get the game</strong>
            <p class="line">Build {game.port} from its GitHub, or get a MoonMod release of it.</p>
          </div>
          <button onclick={onGet}>Open GitHub</button>
        </li>
        <li class:done={view.build !== null}>
          <div>
            <strong>Choose a build</strong>
            <p class="line">
              {#if view.build_problem}
                <span class="state err">{view.build_problem}</span>
              {:else}
                The folder with <code>{game.executable}</code> and <code>{game.extractor}</code> in it, or
                your checkout of {game.port}.
              {/if}
            </p>
          </div>
          <button onclick={onChoose}>{view.build ? "Choose another" : "Choose"}</button>
        </li>
        <li class:done={view.next === "play"}>
          <div>
            <strong>Set up from your disc</strong>
            <p class="line">Your own {regions} disc of {game.title}. Nothing is downloaded.</p>
          </div>
          <span class="soon">Next update</span>
        </li>
        <li>
          <div>
            <strong>Play</strong>
            <p class="line">With your MoonMod mods, or as the disc plays it.</p>
          </div>
        </li>
      </ol>
    </div>

    {#if view.build}
      <details class="panel about rise">
        <summary>About this build</summary>
        <p>Build: <code>{view.build.dir}</code></p>
        <p>Game data: <code>{view.data}</code></p>
        <p>Saves: <code>{view.save}</code></p>
      </details>
    {/if}
  </section>
</main>

<style>
  .band {
    border-bottom: 1px solid var(--line);
  }
  .inner {
    max-width: 1200px;
    margin: 0 auto;
    padding: 18px var(--page-x) 34px;
    display: grid;
    gap: 6px;
    justify-items: start;
  }
  .back {
    margin-bottom: 28px;
  }
  .chapter {
    font-family: var(--font-head);
    color: var(--border);
    letter-spacing: 0.2em;
    text-transform: uppercase;
    font-size: 13px;
  }
  h1 {
    font-size: clamp(34px, 5vw, 52px);
    line-height: 1.05;
  }
  .hero {
    color: var(--soft);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin-top: 18px;
  }
  .note {
    color: var(--dim);
    font-size: 13px;
  }
  .body {
    max-width: 1200px;
    margin: 0 auto;
    padding: var(--page-top) var(--page-x) 48px;
    display: grid;
    gap: var(--page-gap);
  }
  h2 {
    font-size: 18px;
    margin-bottom: 6px;
  }
  .line {
    color: var(--soft);
    font-size: 14px;
  }
  .steps button {
    white-space: nowrap;
  }
  .soon {
    white-space: nowrap;
    color: var(--dim);
    font-size: 13px;
  }
  .about summary {
    cursor: pointer;
    color: var(--lav);
  }
  .about p {
    margin-top: 8px;
    color: var(--soft);
    font-size: 13px;
    overflow-wrap: anywhere;
  }
</style>
