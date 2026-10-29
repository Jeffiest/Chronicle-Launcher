<!-- A chapter's page: its sky and name, the one next step, and the steps to get there. -->
<script lang="ts">
  import Clouds from "../components/Clouds.svelte";
  import Menu, { type Item } from "../components/Menu.svelte";
  import { chapterName, material } from "../lib/games";
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

<main data-window={material(game.chapter)}>
  <section class="band">
    <div class="inner">
      <button class="link back" onclick={onBack}>‹ The Chronicle</button>
      <span class="chapter">{chapterName(game.chapter)}</span>
      <h1 class="gold-text">{game.title}</h1>
      <p class="hero">{game.hero}</p>
      <div class="actions">
        <button class="primary" disabled={waiting} onclick={onGet}>{MAIN_BUTTON[view.next]}</button>
        {#if view.next === "get-the-game"}
          <button onclick={onChoose}>Choose a build</button>
        {/if}
        <Menu {items} />
      </div>
      <p class="where">
        <span class="state {state.tone}">{state.words}</span>
        {#if waiting}<span class="note">Setting up from your disc arrives in the launcher's next update.</span
          >{/if}
      </p>
    </div>
  </section>
  <Clouds height={110} />

  <section class="body">
    <div class="paper steps-panel rise">
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
      <details class="stone about rise">
        <summary>About this build</summary>
        <p>Build: <code>{view.build.dir}</code></p>
        <p>Game data: <code>{view.data}</code></p>
        <p>Saves: <code>{view.save}</code></p>
      </details>
    {/if}
  </section>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
  }
  .inner {
    max-width: 1200px;
    width: 100%;
    margin: 0 auto;
    padding: 16px var(--page-x) 8px;
    display: grid;
    gap: 6px;
    justify-items: start;
  }
  .back {
    margin-bottom: 22px;
  }
  .chapter {
    font: 700 15px/1 var(--font-title);
    color: var(--gold-hi);
    letter-spacing: 0.24em;
    text-transform: uppercase;
    text-shadow: 0 1px 3px var(--cloud-deep);
  }
  h1 {
    font-size: clamp(40px, 6vw, 64px);
    line-height: 1.05;
  }
  .hero {
    font-family: var(--font-game);
    color: var(--sky-haze);
    font-size: 17px;
    text-shadow: 0 1px 3px var(--cloud-deep);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px;
    margin-top: 20px;
  }
  .where {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    align-items: center;
    margin-top: 12px;
    text-shadow: 0 1px 3px var(--cloud-deep);
  }
  .where .state.warn {
    color: var(--gold-hi);
  }
  .note {
    color: var(--sky-haze);
    font-size: 13px;
  }
  .body {
    flex: 1;
    margin-top: -2px;
    background: var(--cloud-deep);
    width: 100%;
    padding: 8px var(--page-x) 48px;
    display: grid;
    gap: var(--page-gap);
    align-content: start;
  }
  .body > * {
    width: 100%;
    max-width: calc(1200px - 2 * var(--page-x));
    margin: 0 auto;
  }
  .steps-panel {
    padding: 20px 24px 10px;
  }
  .steps-panel h2 {
    font-size: 22px;
    color: var(--bronze-deep);
    margin-bottom: 4px;
  }
  strong {
    font-family: var(--font-title);
    font-size: 17px;
    letter-spacing: 0.02em;
    color: var(--ink);
  }
  .line {
    color: var(--ink-soft);
    font-size: 14px;
  }
  .line code {
    color: var(--ink);
  }
  .line .state.err {
    color: var(--terracotta-rim);
  }
  .steps-panel button {
    white-space: nowrap;
  }
  .soon {
    white-space: nowrap;
    color: var(--ink-soft);
    font-style: italic;
    font-size: 13px;
  }
  .about {
    padding: 14px 20px;
  }
  .about summary {
    cursor: pointer;
    font-family: var(--font-title);
    color: var(--gold-hi);
  }
  .about p {
    margin-top: 8px;
    color: var(--on-stone-soft);
    font-size: 13px;
    overflow-wrap: anywhere;
  }
</style>
