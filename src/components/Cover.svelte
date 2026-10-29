<!-- A chapter's cover on the Chronicle, in its game's window: Blue Stone for Dark Cloud, Paper for Dark Chronicle,
     framed in the logo's gold. -->
<script lang="ts">
  import { chapterName, material } from "../lib/games";
  import { stateOf } from "../lib/next";
  import type { GameView } from "../lib/types";

  let { view, onOpen }: { view: GameView; onOpen: () => void } = $props();
  const state = $derived(stateOf(view));
  const kind = $derived(material(view.game.chapter));
</script>

<button class="cover {kind}" onclick={onOpen}>
  <span class="chapter">{chapterName(view.game.chapter)}</span>
  <span class="title gold-text">{view.game.title}</span>
  <span class="hero">{view.game.hero}</span>
  <span class="state {state.tone}">{state.words}</span>
</button>

<style>
  .cover {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    width: 100%;
    padding: 26px 28px 24px;
    text-align: left;
    font: inherit;
    text-shadow: none;
    border: 3px solid transparent;
    border-radius: var(--radius);
  }
  .cover.stone {
    background:
      var(--grain) padding-box,
      var(--mottle) padding-box,
      linear-gradient(180deg, var(--stone-hi), var(--stone) 40%, var(--stone-lo)) padding-box,
      linear-gradient(180deg, var(--gold-hi), var(--gold-mid) 60%, var(--bronze)) border-box;
    background-blend-mode: overlay, overlay, normal, normal;
    box-shadow:
      inset 2px 2px 0 var(--bevel-light),
      0 0 0 2px var(--bronze-deep),
      0 12px 28px var(--shadow);
    color: var(--on-stone);
  }
  .cover.paper {
    background:
      var(--grain) padding-box,
      var(--mottle) padding-box,
      linear-gradient(180deg, var(--paper-hi), var(--paper) 50%, var(--paper-lo)) padding-box,
      linear-gradient(180deg, var(--gold-hi), var(--gold-mid) 60%, var(--bronze)) border-box;
    background-blend-mode: overlay, overlay, normal, normal;
    box-shadow:
      inset 2px 2px 0 var(--bevel-light),
      0 0 0 2px var(--bronze-deep),
      0 12px 28px var(--shadow);
    color: var(--ink);
  }
  .chapter {
    font: 700 14px/1 var(--font-title);
    letter-spacing: 0.22em;
    text-transform: uppercase;
  }
  .stone .chapter {
    color: var(--gold-hi);
  }
  .paper .chapter {
    color: var(--bronze-deep);
  }
  .title {
    font: 700 clamp(30px, 4vw, 42px) / 1.1 var(--font-title);
    margin-top: 6px;
  }
  .hero {
    font-family: var(--font-game);
    margin-bottom: 12px;
  }
  .stone .hero {
    color: var(--on-stone-soft);
  }
  .paper .hero {
    color: var(--ink-soft);
  }
  .paper .state.warn {
    color: var(--bronze-deep);
  }
  @media (hover: hover) {
    .cover:hover {
      transform: translateY(-3px);
      filter: brightness(1.06);
    }
  }
</style>
