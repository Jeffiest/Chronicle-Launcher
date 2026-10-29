<!-- The night-dark bar along the top, with a gold rule under it. The crescent goes home. -->
<script lang="ts">
  import Logo from "./Logo.svelte";

  let { onHome, version }: { onHome: () => void; version: string } = $props();
</script>

<header>
  <button class="home" onclick={onHome} title="The Chronicle"><Logo /></button>
  <span class="version mono">{version}</span>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px var(--page-x);
    background: var(--ink-deep);
    border-bottom: 1px solid var(--brand);
  }
  .home {
    background: none;
    border: 0;
    padding: 4px 0;
  }
  .version {
    color: var(--dim);
    font-size: 12px;
  }
</style>
