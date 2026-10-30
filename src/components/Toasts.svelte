<!-- Notices along the bottom, each a small menu window. -->
<script lang="ts">
  import { store } from "../lib/store.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each store.toasts as t (t.id)}
    <p class="window toast rise"><span class="state {t.tone}">{t.text}</span></p>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    left: 50%;
    top: calc(var(--bar) + 14px);
    z-index: 10;
    transform: translateX(-50%);
    display: grid;
    gap: 12px;
    width: min(520px, calc(100vw - 32px));
  }
  .toast {
    padding: 10px 16px;
    font-size: calc(16px * var(--game-size, 1));
  }
</style>
