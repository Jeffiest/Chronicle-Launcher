<!-- The page's rarer actions, in a menu window beside its main one. Anything that removes comes last. -->
<script module lang="ts">
  export interface Item {
    label: string;
    run: () => void;
    removes?: boolean;
  }
</script>

<script lang="ts">
  let { items }: { items: Item[] } = $props();
  let open = $state(false);
</script>

<div class="menu">
  <button
    class="window opener"
    aria-haspopup="menu"
    aria-expanded={open}
    aria-label="More"
    onclick={() => (open = !open)}
  >
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
      ><circle cx="5" cy="12" r="2"></circle><circle cx="12" cy="12" r="2"></circle><circle
        cx="19"
        cy="12"
        r="2"
      ></circle></svg
    >
  </button>
  {#if open}
    <ul class="window items rows" role="menu">
      {#each items as item (item.label)}
        <li role="none">
          <button
            role="menuitem"
            class="row"
            class:removes={item.removes}
            onclick={() => {
              open = false;
              item.run();
            }}>{item.label}</button
          >
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .menu {
    position: relative;
  }
  .opener {
    height: 100%;
    min-height: 56px;
    padding: 0 18px;
    border: 0;
    color: var(--text-soft);
  }
  .items {
    position: absolute;
    left: 0;
    bottom: calc(100% + 14px);
    z-index: 4;
    min-width: 260px;
  }
  .items button.row {
    width: 100%;
    border: 0;
    background: none;
    text-align: left;
  }
  .items .removes {
    color: var(--err);
  }
</style>
