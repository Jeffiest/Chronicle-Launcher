<!-- The Chronicle: the two games as its two chapters. -->
<script lang="ts">
  import Cover from "../components/Cover.svelte";
  import type { GameView } from "../lib/types";

  let { views, onOpen }: { views: GameView[]; onOpen: (id: string) => void } = $props();
</script>

<main class="page">
  <section class="head rise">
    <h1>The Chronicle</h1>
    <p>Dark Cloud and Dark Chronicle on PC, from your own discs, with MoonMod.</p>
  </section>
  <section class="covers">
    {#each views as view, i (view.game.id)}
      <div class="rise" style="animation-delay: {80 + i * 80}ms">
        <Cover {view} onOpen={() => onOpen(view.game.id)} />
      </div>
    {/each}
  </section>
</main>

<style>
  .page {
    padding: var(--page-top) var(--page-x) 48px;
    display: grid;
    gap: var(--page-gap);
    max-width: 1200px;
    margin: 0 auto;
  }
  .head h1 {
    font-size: 30px;
    margin-bottom: 4px;
  }
  .head p {
    color: var(--soft);
  }
  .covers {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 20px;
  }
</style>
