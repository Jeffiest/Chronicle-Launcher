<!-- Notices along the bottom, on stone: what an action did, or why it could not. -->
<script lang="ts">
  import { store } from "../lib/store.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each store.toasts as t (t.id)}
    <p class="toast stone rise"><span class="state {t.tone}">{t.text}</span></p>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: var(--page-x);
    bottom: 20px;
    z-index: 10;
    display: grid;
    gap: 10px;
    max-width: min(440px, calc(100vw - 32px));
  }
  .toast {
    padding: 12px 16px;
  }
</style>
