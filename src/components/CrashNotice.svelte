<!-- The game stopped unexpectedly: said in a menu window, with the saves kept and a report to share. -->
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

<section class="window rise" role="alert">
  <h2 class="window-title crash">The game stopped</h2>
  <div class="rule"></div>
  <p>{when(crash.when)}. {crash.what}. Your saves are kept.</p>
  <div class="footer">
    <button class="key" onclick={copy}>Copy report</button>
    <button class="link" onclick={save}>Save report</button>
    <button class="link" onclick={() => attempt(() => api.openFolder(id, "logs"))}>Logs</button>
  </div>
</section>

<style>
  .crash {
    color: var(--err);
  }
  p {
    padding: 10px 18px 12px;
    font-size: calc(16px * var(--game-size));
    color: var(--text-soft);
  }
</style>
