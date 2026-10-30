<!-- One of a mod's settings, as the in-game Mod Manager draws it: its name, and ON / OFF, a strip of choices, a bar
     with its number, or a field. It saves as it changes; off its default, a Reset. -->
<script lang="ts">
  import type { ModSetting } from "../lib/types";

  let {
    setting,
    disabled,
    onChange,
  }: { setting: ModSetting; disabled: boolean; onChange: (value: unknown) => void } = $props();

  const changed = $derived(JSON.stringify(setting.value) !== JSON.stringify(setting.default));
  const numeric = $derived(setting.type === "integer" || setting.type === "number");
  const step = $derived(setting.step ?? (setting.type === "integer" ? 1 : 0.1));

  function number(text: string) {
    const value = setting.type === "integer" ? Math.round(Number(text)) : Number(text);
    if (text.trim() !== "" && Number.isFinite(value)) onChange(value);
  }
</script>

<div class="setting">
  <label for="setting-{setting.id}" class:changed>{setting.name}</label>
  <div class="control">
    {#if setting.type === "boolean"}
      <span class="choice" id="setting-{setting.id}">
        <button aria-pressed={setting.value === true} {disabled} onclick={() => onChange(true)}>On</button>
        <button aria-pressed={setting.value !== true} {disabled} onclick={() => onChange(false)}>Off</button>
      </span>
    {:else if setting.type === "choice" && setting.choices.length <= 4}
      <span class="choice" id="setting-{setting.id}">
        {#each setting.choices as choice (choice)}
          <button aria-pressed={setting.value === choice} {disabled} onclick={() => onChange(choice)}
            >{choice}</button
          >
        {/each}
      </span>
    {:else if setting.type === "choice"}
      <select
        id="setting-{setting.id}"
        value={setting.value}
        {disabled}
        onchange={(e) => onChange(e.currentTarget.value)}
      >
        {#each setting.choices as choice (choice)}<option>{choice}</option>{/each}
      </select>
    {:else if numeric}
      {#if setting.min !== null && setting.max !== null}
        <input
          class="bar"
          type="range"
          aria-label={setting.name}
          min={setting.min}
          max={setting.max}
          {step}
          value={setting.value}
          {disabled}
          onchange={(e) => number(e.currentTarget.value)}
        />
      {/if}
      <input
        id="setting-{setting.id}"
        class="field number"
        type="number"
        min={setting.min ?? undefined}
        max={setting.max ?? undefined}
        {step}
        value={setting.value}
        {disabled}
        onchange={(e) => number(e.currentTarget.value)}
      />
    {:else}
      <input
        id="setting-{setting.id}"
        class="field"
        type="text"
        maxlength="256"
        value={setting.value}
        {disabled}
        onchange={(e) => onChange(e.currentTarget.value)}
      />
    {/if}
    {#if changed}<button class="link" {disabled} onclick={() => onChange(null)}>Reset</button>{/if}
  </div>
  {#if setting.description}<p class="line">{setting.description}</p>{/if}
</div>

<style>
  .setting {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 2px 16px;
    align-items: center;
    padding: 8px 0;
    box-shadow: inset 0 -2px 0 var(--frame);
    font-family: var(--font-game);
    font-size: calc(17px * var(--game-size));
  }
  label.changed {
    color: var(--accent);
  }
  .control {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .line {
    grid-column: 1 / -1;
    font-size: 0.82em;
    color: var(--text-dim);
  }
  .field,
  select {
    padding: 3px 8px;
    border: 0;
    background: var(--frame-dark);
    box-shadow: inset 0 0 0 1px var(--rule);
    color: var(--text);
    font: inherit;
  }
  .number {
    width: 80px;
  }
  .bar {
    width: 150px;
    accent-color: var(--accent);
  }
</style>
