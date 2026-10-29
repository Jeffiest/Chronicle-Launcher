<!-- One of a mod's settings, as its manifest types it: a switch, a few choices side by side or a list, a slider with
     its number when both ends are known, or a field. It saves as it changes, and has a Reset off its default. -->
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
  const slider = $derived(numeric && setting.min !== null && setting.max !== null);

  function number(text: string) {
    const value = setting.type === "integer" ? Math.round(Number(text)) : Number(text);
    if (text.trim() !== "" && Number.isFinite(value)) onChange(value);
  }
</script>

<div class="setting">
  <div>
    <label class="name" for="setting-{setting.id}">{setting.name}</label>
    {#if setting.description}<p class="line">{setting.description}</p>{/if}
  </div>
  <div class="control">
    {#if setting.type === "boolean"}
      <input
        id="setting-{setting.id}"
        class="switch"
        type="checkbox"
        checked={setting.value === true}
        {disabled}
        onchange={(e) => onChange(e.currentTarget.checked)}
      />
    {:else if setting.type === "choice" && setting.choices.length <= 3}
      <div class="segmented" role="radiogroup" aria-label={setting.name}>
        {#each setting.choices as choice (choice)}
          <button
            role="radio"
            aria-checked={setting.value === choice}
            class:on={setting.value === choice}
            {disabled}
            onclick={() => onChange(choice)}>{choice}</button
          >
        {/each}
      </div>
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
      {#if slider}
        <input
          class="slider"
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
    {#if changed}
      <button class="link reset" {disabled} onclick={() => onChange(null)}>Reset</button>
    {/if}
  </div>
</div>

<style>
  .name {
    font-family: var(--font-game);
    font-size: 16px;
  }
  .line {
    color: var(--on-stone-soft);
    font-size: 13px;
  }
  .control {
    display: flex;
    align-items: center;
    gap: 12px;
    justify-content: flex-end;
    flex-wrap: wrap;
  }
  .segmented {
    display: flex;
    gap: 6px;
  }
  .segmented button {
    text-transform: capitalize;
  }
  .segmented button.on {
    --face: var(--amber);
    --face-hi: var(--amber-hi);
    --rim: var(--amber-rim);
    color: var(--amber-ink);
    text-shadow: 0 1px 0 var(--amber-light);
  }
  .field,
  select {
    font: inherit;
    color: var(--on-stone);
    background: var(--stone-rim);
    border: 1px solid var(--stone-lo);
    border-radius: var(--radius-plate);
    padding: 7px 10px;
  }
  .number {
    width: 92px;
  }
  .slider {
    width: 180px;
    accent-color: var(--amber);
  }
  .reset {
    font-size: 13px;
  }
</style>
