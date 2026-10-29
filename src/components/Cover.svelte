<!-- A chapter's cover on the Chronicle: its sky, its name, who it is about, and where it stands. -->
<script lang="ts">
  import { chapterName, themeClass } from "../lib/games";
  import { stateOf } from "../lib/next";
  import type { GameView } from "../lib/types";

  let { view, onOpen }: { view: GameView; onOpen: () => void } = $props();
  const state = $derived(stateOf(view));
</script>

<button class="cover backdrop {themeClass(view.game.chapter)}" onclick={onOpen}>
  <span class="chapter">{chapterName(view.game.chapter)}</span>
  <span class="title">{view.game.title}</span>
  <span class="hero">{view.game.hero}</span>
  <span class="state {state.tone}">{state.words}</span>
</button>

<style>
  .cover {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: flex-start;
    gap: 6px;
    width: 100%;
    min-height: 360px;
    padding: 28px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius);
    text-align: left;
    box-shadow: inset 0 0 0 1px var(--line);
  }
  .chapter {
    font-family: var(--font-head);
    color: var(--border);
    letter-spacing: 0.2em;
    text-transform: uppercase;
    font-size: 13px;
  }
  .title {
    font-family: var(--font-head);
    color: var(--gold);
    font-size: clamp(28px, 4vw, 40px);
    font-weight: 600;
    line-height: 1.1;
  }
  .hero {
    color: var(--soft);
    margin-bottom: 14px;
  }
  @media (hover: hover) {
    .cover:hover {
      border-color: var(--border);
      transform: translateY(-2px);
    }
  }
</style>
