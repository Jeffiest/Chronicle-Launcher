<!-- The bar along the top, as a menu window's edge: the Chronicle logo, the launcher's version, and the game's build. -->
<script lang="ts">
  import logo from "../assets/chronicle-logo.webp";
  import type { GameView } from "../lib/types";

  let { version, view }: { version: string; view: GameView | null } = $props();
  const build = $derived(view?.build ? view.build.dir.split(/[\\/]/).filter(Boolean).pop() : null);
</script>

<header>
  <img src={logo} alt="Chronicle" />
  <span class="name">LAUNCHER</span>
  <span class="version mono">{version}</span>
  <span class="space"></span>
  {#if view}
    <span class="build">
      {view.game.port} build: <span class="value">{build ?? "not chosen"}</span>
    </span>
  {/if}
</header>

<style>
  header {
    position: relative;
    z-index: 3;
    height: var(--bar);
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 16px;
    background: var(--win-glass);
    box-shadow:
      inset 0 -2px 0 var(--frame),
      inset 0 -3px 0 var(--frame-dark);
    font-family: var(--font-game);
  }
  img {
    height: 26px;
    width: auto;
  }
  .name {
    font-size: calc(15px * var(--game-size));
    letter-spacing: 0.12em;
    color: var(--accent);
  }
  .version {
    font-size: 11px;
    color: var(--text-dim);
  }
  .space {
    flex: 1;
  }
  .build {
    font-size: calc(14px * var(--game-size));
    color: var(--text-soft);
  }
  .value {
    color: var(--accent);
  }
</style>
