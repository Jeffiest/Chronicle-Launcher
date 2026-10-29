<!-- The game stopped unexpectedly: say so, say the saves are kept, and give a report to share. Stays until the
     game next closes cleanly. -->
<script lang="ts">
  import { api } from "../lib/api";
  import { when } from "../lib/format";
  import { attempt, toast } from "../lib/store.svelte";
  import type { Crash } from "../lib/types";

  let { id, crash }: { id: string; crash: Crash } = $props();

  async function copy() {
    const text = await attempt(() => api.crashReport(id));
    if (text === undefined) return;
    await attempt(() => navigator.clipboard.writeText(text));
    toast("Report copied. Paste it into a GitHub issue or on Discord.", "ok");
  }

  async function save() {
    const where = await attempt(() => api.saveReport(id));
    if (where) toast(`Report saved: ${where}`, "ok");
  }
</script>

<section class="crash stone rise" role="alert">
  <div>
    <h2>The game stopped unexpectedly</h2>
    <p>{when(crash.when)}. {crash.what}. Your saves are kept.</p>
  </div>
  <div class="actions">
    <button onclick={copy}>Copy report</button>
    <button onclick={save}>Save report</button>
    <button class="link" onclick={() => attempt(() => api.openFolder(id, "logs"))}>Logs</button>
  </div>
</section>

<style>
  .crash {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 14px 24px;
    padding: 16px 22px;
    box-shadow:
      inset 2px 2px 0 var(--bevel-light),
      inset -2px -2px 0 var(--bevel-dark),
      0 0 0 3px var(--terracotta),
      0 10px 24px var(--shadow);
  }
  h2 {
    font-size: 19px;
    color: var(--terracotta-light);
  }
  p {
    color: var(--on-stone-soft);
    font-size: 14px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
  }
</style>
