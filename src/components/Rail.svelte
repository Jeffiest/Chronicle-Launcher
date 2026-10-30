<!-- The rail down the left: each game, then Moon Factory. The open one sits on the brown bar, as a chosen menu row. -->
<script lang="ts">
  import { GAMES } from "../lib/games";

  let { open, onOpen }: { open: string; onOpen: (where: string) => void } = $props();
  const LABELS: Record<string, string> = { chronicle: "DARK CLOUD", "chronicle-two": "CHRONICLE" };
</script>

<nav aria-label="Games">
  {#each GAMES as game (game.id)}
    <button
      class:open={open === game.id}
      aria-current={open === game.id ? "page" : undefined}
      onclick={() => onOpen(game.id)}
    >
      {#if game.chapter === 1}
        <svg width="30" height="30" viewBox="0 0 24 24" aria-hidden="true"
          ><path d="M15.5 2.6A10 10 0 1 0 21.4 15 8 8 0 0 1 15.5 2.6Z" fill="currentColor"></path></svg
        >
      {:else}
        <svg width="30" height="30" viewBox="-32 -32 64 64" aria-hidden="true"
          ><path
            d="M0 -26 C 14 -14, 14 14, 0 26 C -14 14, -14 -14, 0 -26 Z"
            fill="none"
            stroke="currentColor"
            stroke-width="4"
          ></path><circle cy="-26" r="5" fill="currentColor"></circle><circle
            r="28"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
          ></circle></svg
        >
      {/if}
      <span>{LABELS[game.id] ?? game.title}</span>
    </button>
  {/each}
  <span class="rule" aria-hidden="true"></span>
  <button
    class:open={open === "mods"}
    aria-current={open === "mods" ? "page" : undefined}
    onclick={() => onOpen("mods")}
  >
    <span class="moon" aria-hidden="true"></span>
    <span>MODS</span>
  </button>
</nav>

<style>
  nav {
    width: var(--rail);
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 14px 8px;
    background: var(--win-glass);
    box-shadow:
      inset -2px 0 0 var(--frame),
      inset -3px 0 0 var(--frame-dark);
  }
  button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 10px 0 8px;
    border: 0;
    background: none;
    color: var(--text-soft);
    font-family: var(--font-game);
    font-size: calc(12px * var(--game-size));
    white-space: nowrap;
  }
  button.open {
    background: var(--select);
    box-shadow: inset 0 0 0 1px var(--select-edge);
    color: var(--accent);
  }
  .rule {
    height: 2px;
    margin: 6px 4px;
    background: var(--frame);
  }
  .moon {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: radial-gradient(circle at 35% 30%, var(--moon-light), var(--moon));
    box-shadow: inset -8px -3px 0 var(--win);
  }
  .open .moon {
    box-shadow: inset -8px -3px 0 var(--select);
  }
  @media (hover: hover) {
    button:not(.open):hover {
      color: var(--text);
    }
  }
</style>
