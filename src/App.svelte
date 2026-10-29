<script lang="ts">
  import Header from "./components/Header.svelte";
  import Toasts from "./components/Toasts.svelte";
  import Home from "./pages/Home.svelte";
  import GamePage from "./pages/GamePage.svelte";
  import { start, store } from "./lib/store.svelte";
  import { version } from "../package.json";

  let open: string | null = $state(null);
  const current = $derived(store.views.find((v) => v.game.id === open) ?? null);

  start();
</script>

<Header version={`v${version}`} showLogo={current !== null} onHome={() => (open = null)} />

{#if current}
  <GamePage view={current} onBack={() => (open = null)} />
{:else}
  <Home views={store.views} onOpen={(id) => (open = id)} />
{/if}

<Toasts />
