<!-- A chapter's page: its name, the one next step, and below, what that step needs: the disc, the saves, a crash
     to report, or the steps still to come. -->
<script lang="ts">
  import Clouds from "../components/Clouds.svelte";
  import CrashNotice from "../components/CrashNotice.svelte";
  import DiscPanel from "../components/DiscPanel.svelte";
  import Menu, { type Item } from "../components/Menu.svelte";
  import SavesPanel from "../components/SavesPanel.svelte";
  import { api } from "../lib/api";
  import { fileName, when } from "../lib/format";
  import { chapterName, material } from "../lib/games";
  import { mainButton, stateOf } from "../lib/next";
  import { attempt, replace, toast } from "../lib/store.svelte";
  import type { GameView } from "../lib/types";

  let { view, onBack }: { view: GameView; onBack: () => void } = $props();
  let removing = $state(false);

  const game = $derived(view.game);
  const standing = $derived(stateOf(view));
  const busy = $derived(view.activity.kind !== "idle");
  const regions = $derived(game.discs.map((d) => d.region).join(" or "));
  const settingUp = $derived(view.next === "set-up" || view.activity.kind === "setting-up");

  async function main() {
    if (view.next === "get-the-game") return attempt(() => api.openLink(game.get_url));
    if (view.next === "set-up") {
      document.getElementById("disc")?.scrollIntoView({ behavior: "smooth", block: "center" });
      return;
    }
    replace(await attempt(() => api.play(game.id)));
  }

  async function choose() {
    replace(await attempt(() => api.chooseBuild(game.id)));
  }

  async function removeSetUp() {
    removing = false;
    const after = await attempt(() => api.removeSetUp(game.id));
    if (after) {
      replace(after);
      toast("The set-up is removed. Your disc and saves are kept.", "ok");
    }
  }

  const items: Item[] = $derived([
    { label: view.build ? "Choose another build" : "Choose a build", run: choose },
    { label: `${game.port} on GitHub`, run: () => attempt(() => api.openLink(game.repository)) },
    ...(view.build
      ? [
          { label: "Open the game's folder", run: () => attempt(() => api.openFolder(game.id, "game")) },
          { label: "Logs", run: () => attempt(() => api.openFolder(game.id, "logs")) },
        ]
      : []),
    ...(view.set_up ? [{ label: "Remove the set-up", run: () => (removing = true), removes: true }] : []),
    ...(view.build && !view.set_up
      ? [
          {
            label: "Forget this build",
            run: async () => replace(await attempt(() => api.forgetBuild(game.id))),
            removes: true,
          },
        ]
      : []),
  ]);
</script>

<main data-window={material(game.chapter)}>
  <section class="band">
    <div class="inner">
      <button class="link back" onclick={onBack}>‹ The Chronicle</button>
      <span class="chapter">{chapterName(game.chapter)}</span>
      <h1 class="gold-text">{game.title}</h1>
      <p class="hero">{game.hero}</p>
      <div class="actions">
        <button class="primary" disabled={busy || (view.next === "set-up" && !view.can_set_up)} onclick={main}
          >{mainButton(view)}</button
        >
        {#if view.next === "get-the-game"}
          <button onclick={choose}>Choose a build</button>
        {/if}
        {#if !busy}<Menu {items} />{/if}
      </div>
      <p class="where">
        <span class="state {standing.tone}">{standing.words}</span>
        {#if view.set_up?.release}<span class="note">{view.set_up.release}</span>{/if}
        {#if view.next === "set-up" && !view.can_set_up}
          <span class="note"
            >Setting {game.title} up from its disc comes in a later update of the launcher.</span
          >
        {/if}
      </p>
    </div>
  </section>
  <Clouds height={110} />

  <section class="body">
    {#if removing}
      <div class="stone confirm rise" role="alertdialog" aria-label="Remove the set-up">
        <p>
          Remove the set-up? This deletes the game data made from your disc. Your disc image and saves are
          kept.
        </p>
        <span>
          <button class="removes" onclick={removeSetUp}>Remove</button>
          <button onclick={() => (removing = false)}>Keep it</button>
        </span>
      </div>
    {/if}

    {#if view.last_crash && !busy}
      <CrashNotice id={game.id} crash={view.last_crash} />
    {/if}

    {#if settingUp && view.can_set_up && view.build}
      <DiscPanel {view} />
    {/if}

    {#if view.set_up}
      <SavesPanel {view} />
      <details class="stone about rise">
        <summary>About this set-up</summary>
        <p>Disc: <code>{fileName(view.set_up.disc)}</code>, {view.set_up.release ?? "release not listed"}</p>
        <p>Set up: {when(view.set_up.finished)}</p>
        <p>Build: <code>{view.build?.dir ?? view.set_up.build}</code></p>
        <p>Game data: <code>{view.data}</code></p>
        <p>Saves: <code>{view.save}</code></p>
      </details>
    {:else}
      <div class="paper steps-panel rise">
        <h2>Getting there</h2>
        <ol class="steps">
          <li class:done={view.build !== null}>
            <div>
              <strong>Get the game</strong>
              <p class="line">Build {game.port} from its GitHub, or get a MoonMod release of it.</p>
            </div>
            <button onclick={() => attempt(() => api.openLink(game.get_url))}>Open GitHub</button>
          </li>
          <li class:done={view.build !== null}>
            <div>
              <strong>Choose a build</strong>
              <p class="line">
                {#if view.build_problem}
                  <span class="state err">{view.build_problem}</span>
                {:else}
                  The folder with <code>{game.executable}</code> and <code>{game.extractor}</code> in it, or
                  your checkout of {game.port}.
                {/if}
              </p>
            </div>
            <button onclick={choose}>{view.build ? "Choose another" : "Choose"}</button>
          </li>
          <li>
            <div>
              <strong>Set up from your disc</strong>
              <p class="line">Your own {regions} disc of {game.title}. Nothing is downloaded.</p>
            </div>
            {#if !view.can_set_up}<span class="soon">Later update</span>{/if}
          </li>
          <li>
            <div>
              <strong>Play</strong>
              <p class="line">With your MoonMod mods, or as the disc plays it.</p>
            </div>
          </li>
        </ol>
      </div>
    {/if}
  </section>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
  }
  .inner {
    max-width: 1200px;
    width: 100%;
    margin: 0 auto;
    padding: 16px var(--page-x) 8px;
    display: grid;
    gap: 6px;
    justify-items: start;
  }
  .back {
    margin-bottom: 22px;
  }
  .chapter {
    font: 700 15px/1 var(--font-title);
    color: var(--gold-hi);
    letter-spacing: 0.24em;
    text-transform: uppercase;
    text-shadow: 0 1px 3px var(--cloud-deep);
  }
  h1 {
    font-size: clamp(40px, 6vw, 64px);
    line-height: 1.05;
  }
  .hero {
    font-family: var(--font-game);
    color: var(--sky-haze);
    font-size: 17px;
    text-shadow: 0 1px 3px var(--cloud-deep);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px;
    margin-top: 20px;
  }
  .where {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    align-items: center;
    margin-top: 12px;
    text-shadow: 0 1px 3px var(--cloud-deep);
  }
  .where .state.warn {
    color: var(--gold-hi);
  }
  .note {
    color: var(--sky-haze);
    font-size: 13px;
  }
  .body {
    flex: 1;
    margin-top: -2px;
    background: var(--cloud-deep);
    width: 100%;
    padding: 8px var(--page-x) 48px;
    display: grid;
    gap: var(--page-gap);
    align-content: start;
  }
  .body > * {
    width: 100%;
    max-width: calc(1200px - 2 * var(--page-x));
    margin: 0 auto;
  }
  .confirm {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px 20px;
    padding: 14px 20px;
  }
  .confirm span {
    display: flex;
    gap: 10px;
  }
  .steps-panel {
    padding: 20px 24px 10px;
  }
  .steps-panel h2 {
    font-size: 22px;
    color: var(--bronze-deep);
    margin-bottom: 4px;
  }
  strong {
    font-family: var(--font-title);
    font-size: 17px;
    letter-spacing: 0.02em;
    color: var(--ink);
  }
  .line {
    color: var(--ink-soft);
    font-size: 14px;
  }
  .line code {
    color: var(--ink);
  }
  .line .state.err {
    color: var(--terracotta-rim);
  }
  .steps-panel button {
    white-space: nowrap;
  }
  .soon {
    white-space: nowrap;
    color: var(--ink-soft);
    font-style: italic;
    font-size: 13px;
  }
  .about {
    padding: 14px 20px;
  }
  .about summary {
    cursor: pointer;
    font-family: var(--font-title);
    color: var(--gold-hi);
  }
  .about p {
    margin-top: 8px;
    color: var(--on-stone-soft);
    font-size: 13px;
    overflow-wrap: anywhere;
  }
</style>
