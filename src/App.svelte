<!-- The launcher: the bar along the top, the rail of games down the left, and the page open. -->
<script lang="ts">
  import Rail from "./components/Rail.svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import Toasts from "./components/Toasts.svelte";
  import GamePage from "./pages/GamePage.svelte";
  import ModsPage from "./pages/ModsPage.svelte";
  import { GAMES } from "./lib/games";
  import { start, store } from "./lib/store.svelte";
  import { version } from "../package.json";

  const KEY = "chronicle-launcher.open";
  function remembered(): string {
    try {
      return localStorage.getItem(KEY) ?? GAMES[0].id;
    } catch {
      return GAMES[0].id;
    }
  }

  let open = $state(remembered());
  let modsGame = $state(GAMES[0].id);
  const game = $derived(open === "mods" ? modsGame : open);
  const view = $derived(store.views.find((v) => v.game.id === game) ?? null);

  function go(where: string) {
    open = where;
    if (where !== "mods") modsGame = where;
    try {
      localStorage.setItem(KEY, where);
    } catch {
      // A private window keeps nothing; the launcher opens on Dark Cloud next time.
    }
  }

  start();
</script>

<div class="app" data-game={game}>
  <TitleBar version={`v${version}`} {view} />
  <div class="body">
    <Rail {open} onOpen={go} />
    <div class="page">
      {#if open === "mods"}
        <ModsPage game={modsGame} onGame={(id) => (modsGame = id)} />
      {:else if view}
        <GamePage {view} />
      {/if}
    </div>
  </div>
  <Toasts />
</div>

<style>
  .app {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--win);
  }
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .page {
    flex: 1;
    min-width: 0;
    position: relative;
  }
</style>
