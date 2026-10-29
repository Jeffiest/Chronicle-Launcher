<!-- A Blue Stone bar along the top, trimmed in gold. The logo goes home. -->
<script lang="ts">
  import Logo from "./Logo.svelte";

  let {
    onHome,
    version,
    showLogo = true,
  }: { onHome: () => void; version: string; showLogo?: boolean } = $props();
</script>

<header class="stone">
  <button class="home link" onclick={onHome} title="The Chronicle">
    {#if showLogo}<Logo height={34} />{:else}<span class="word gold-text">The Chronicle</span>{/if}
  </button>
  <span class="label">Launcher</span>
  <span class="version mono">{version}</span>
</header>

<style>
  header {
    position: relative;
    z-index: 3;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px var(--page-x);
    border-radius: 0;
    border-bottom: 3px solid var(--gold-mid);
  }
  .home {
    padding: 0;
  }
  .word {
    font: 700 20px/1 var(--font-title);
  }
  .label {
    font: 700 15px/1 var(--font-title);
    color: var(--on-stone-soft);
    letter-spacing: 0.08em;
  }
  .version {
    margin-left: auto;
    color: var(--on-stone-dim);
    font-size: 12px;
  }
</style>
