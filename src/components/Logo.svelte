<!-- The crescent: The Moon People's ports and MoonFactory's mods. With `name`, "Chronicle" in the heading face beside
     it. Drawn in the current colour. -->
<script lang="ts">
  let { size = 26, name = true }: { size?: number; name?: boolean } = $props();
</script>

<span class="logo" aria-label="Chronicle Launcher">
  <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
    <path d="M15.5 2.6A10 10 0 1 0 21.4 15 8 8 0 0 1 15.5 2.6Z" fill="currentColor" />
  </svg>
  {#if name}<span class="name">Chronicle</span>{/if}
</span>

<style>
  .logo {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    color: var(--amber);
  }
  .name {
    font-family: var(--font-head);
    font-weight: 600;
    font-size: 20px;
    letter-spacing: 0.08em;
    color: var(--gold);
  }
</style>
