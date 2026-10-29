<!-- The page's rarer actions, beside its main button. Anything that removes comes last, in --err. -->
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
  <button aria-haspopup="menu" aria-expanded={open} onclick={() => (open = !open)} title="More">⋯</button>
  {#if open}
    <div class="items stone" role="menu">
      {#each items as item (item.label)}
        <button
          role="menuitem"
          class:removes={item.removes}
          class:plain={!item.removes}
          onclick={() => {
            open = false;
            item.run();
          }}>{item.label}</button
        >
      {/each}
    </div>
  {/if}
</div>

<style>
  .menu {
    position: relative;
  }
  .items {
    position: absolute;
    top: calc(100% + 10px);
    left: 0;
    z-index: 4;
    min-width: 240px;
    display: grid;
    gap: 6px;
    padding: 10px;
  }
  .items .plain {
    background: none;
    box-shadow: none;
    text-align: left;
    color: var(--on-stone);
    padding: 8px 10px;
  }
  .items .removes {
    margin-top: 6px;
    text-align: left;
  }
  @media (hover: hover) {
    .items .plain:hover {
      background: var(--stone-lo);
    }
  }
</style>
