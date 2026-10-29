<!-- The Chronicle: the logo under its two moons, and the two games as its two chapters. -->
<script lang="ts">
  import Clouds from "../components/Clouds.svelte";
  import Cover from "../components/Cover.svelte";
  import Logo from "../components/Logo.svelte";
  import type { GameView } from "../lib/types";

  let { views, onOpen }: { views: GameView[]; onOpen: (id: string) => void } = $props();
</script>

<main>
  <section class="sky">
    <div class="logo rise"><Logo height={190} /></div>
    <p class="tagline rise">Dark Cloud and Dark Chronicle on PC, from your own discs, with MoonMod.</p>
  </section>
  <Clouds height={150} />
  <section class="covers">
    {#each views as view, i (view.game.id)}
      <div class="rise" style="animation-delay: {100 + i * 90}ms">
        <Cover {view} onOpen={() => onOpen(view.game.id)} />
      </div>
    {/each}
  </section>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
  }
  .sky {
    display: grid;
    justify-items: center;
    gap: 6px;
    padding: 36px var(--page-x) 0;
    text-align: center;
  }
  .logo {
    max-width: 100%;
  }
  .logo :global(img) {
    max-width: 100%;
    height: auto !important;
    width: min(560px, 90vw);
  }
  .tagline {
    font-family: var(--font-game);
    color: var(--sky-haze);
    text-shadow: 0 1px 3px var(--cloud-deep);
    font-size: 17px;
  }
  .covers {
    flex: 1;
    margin-top: -2px;
    padding: 6px var(--page-x) 48px;
    background: var(--cloud-deep);
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(290px, 1fr));
    align-content: start;
    gap: 24px;
  }
  .covers > div {
    max-width: 560px;
    width: 100%;
    justify-self: center;
  }
</style>
