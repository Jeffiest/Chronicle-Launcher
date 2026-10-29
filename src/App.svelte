<script lang="ts">
  import Header from "./components/Header.svelte";
  import Home from "./pages/Home.svelte";
  import GamePage from "./pages/GamePage.svelte";
  import { api } from "./lib/api";
  import type { GameView } from "./lib/types";
  import { version } from "../package.json";

  let views: GameView[] = $state([]);
  let open: string | null = $state(null);
  let error: string | null = $state(null);

  const current = $derived(views.find((v) => v.game.id === open) ?? null);

  async function refresh() {
    try {
      views = await api.library();
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  function replace(view: GameView | null) {
    if (view) views = views.map((v) => (v.game.id === view.game.id ? view : v));
  }

  async function act(run: () => Promise<GameView | null | void>) {
    try {
      const view = await run();
      if (view) replace(view);
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  refresh();
</script>

<Header version={`v${version}`} onHome={() => (open = null)} />

{#if error}
  <p class="error" role="alert"><span class="state err">{error}</span></p>
{/if}

{#if current}
  <GamePage
    view={current}
    onBack={() => (open = null)}
    onGet={() => act(() => api.openLink(current.game.get_url))}
    onChoose={() => act(() => api.chooseBuild(current.game.id))}
    onForget={() => act(() => api.forgetBuild(current.game.id))}
  />
{:else}
  <Home {views} onOpen={(id) => (open = id)} />
{/if}

<style>
  .error {
    padding: 10px var(--page-x);
    background: var(--panel);
    border-bottom: 1px solid var(--line);
  }
</style>
