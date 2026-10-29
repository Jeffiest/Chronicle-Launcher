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
    <div class="items" role="menu">
      {#each items as item (item.label)}
        <button
          role="menuitem"
          class:removes={item.removes}
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
  .menu > button {
    padding: 10px 16px;
  }
  .items {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 2;
    min-width: 230px;
    display: grid;
    padding: 6px;
    background: var(--panel-hi);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-small);
  }
  .items button {
    border: 0;
    border-radius: 6px;
    background: none;
    text-align: left;
  }
  .removes {
    color: var(--err);
    border-top: 1px solid var(--line);
  }
  @media (hover: hover) {
    .items button:hover {
      background: var(--panel);
    }
  }
</style>
