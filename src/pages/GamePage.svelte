<!-- A game's page, as the game's own screens look: its scene behind (`Backdrop`), its name and the one next step on
     the left, and on the right the menu window that step needs: getting the game, the disc, the saves. -->
<script lang="ts">
  import Backdrop from "../components/Backdrop.svelte";
  import CrashNotice from "../components/CrashNotice.svelte";
  import DiscPanel from "../components/DiscPanel.svelte";
  import Menu, { type Item } from "../components/Menu.svelte";
  import SavesPanel from "../components/SavesPanel.svelte";
  import { api } from "../lib/api";
  import { fileName, when } from "../lib/format";
  import { mainButton, stateOf } from "../lib/next";
  import { attempt, loadScene, replace, store, toast } from "../lib/store.svelte";
  import type { GameView } from "../lib/types";

  let { view }: { view: GameView } = $props();
  let removing = $state(false);

  const game = $derived(view.game);
  const standing = $derived(stateOf(view));
  const busy = $derived(view.activity.kind !== "idle");
  const settingUp = $derived(view.next === "set-up" || view.activity.kind === "setting-up");
  const scene = $derived(store.scenes[game.id] ?? null);

  $effect(() => {
    if (!(game.id in store.scenes)) loadScene(game.id);
  });

  async function main() {
    if (view.next === "get-the-game") return attempt(() => api.openLink(game.get_url));
    if (view.next === "set-up") {
      document.querySelector<HTMLButtonElement>("#disc button")?.focus();
      return;
    }
    replace(await attempt(() => api.play(game.id)));
  }

  async function choose() {
    replace(await attempt(() => api.chooseBuild(game.id)));
  }

  async function setMods(on: boolean) {
    replace(await attempt(() => api.setModsOn(game.id, on)));
  }

  async function removeSetUp() {
    removing = false;
    const after = await attempt(() => api.removeSetUp(game.id));
    if (after) {
      replace(after);
      toast("The set-up is removed. Your disc and saves are kept.", "ok");
    }
  }

  async function ownScene() {
    if (await attempt(() => api.pickScene(game.id))) {
      await loadScene(game.id);
      toast("Your screenshot is the background now.", "ok");
    }
  }

  const items: Item[] = $derived([
    { label: view.build ? "Choose another build" : "Choose a build", run: choose },
    { label: `${game.port} on GitHub`, run: () => attempt(() => api.openLink(game.repository)) },
    ...(view.set_up
      ? [
          { label: "Take a new background", run: () => attempt(() => api.renderScene(game.id)) },
          { label: "Use my screenshot", run: ownScene },
        ]
      : []),
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

<main data-game={game.id}>
  <Backdrop chapter={game.chapter} {scene} />

  <section class="intro rise">
    <span class="chapter">CHAPTER {game.chapter === 1 ? "I" : "II"}</span>
    <h1>{game.title}</h1>
    <p class="hero">{game.hero}{view.set_up?.release ? ` / ${view.set_up.release}` : ""}</p>
    <div class="actions">
      <button
        class="main-action"
        disabled={busy || (view.next === "set-up" && !view.can_set_up)}
        onclick={main}
      >
        <svg width="18" height="22" viewBox="0 0 18 22" aria-hidden="true"
          ><path d="M1 1 L17 11 L1 21 Z" fill="currentColor"></path></svg
        >
        {mainButton(view)}
      </button>
      {#if view.next === "play"}
        <div class="window mods" role="group" aria-label="Mods">
          <span>MODS</span>
          <span class="choice">
            <button aria-pressed={view.mods_on} disabled={busy} onclick={() => setMods(true)}>On</button>
            <button aria-pressed={!view.mods_on} disabled={busy} onclick={() => setMods(false)}>Off</button>
          </span>
        </div>
      {/if}
      {#if !busy}<Menu {items} />{/if}
    </div>
    <p class="where"><span class="state {standing.tone}">{standing.words}</span></p>
  </section>

  <aside class="side">
    {#if removing}
      <div class="window rise" role="alertdialog" aria-label="Remove the set-up">
        <h2 class="window-title">Remove the set-up?</h2>
        <div class="rule"></div>
        <p class="say">This deletes the game data made from your disc. Your disc image and saves are kept.</p>
        <div class="footer">
          <button class="key" onclick={removeSetUp}>Remove</button>
          <button class="link" onclick={() => (removing = false)}>Keep it</button>
        </div>
      </div>
    {/if}

    {#if view.last_crash && !busy}
      <CrashNotice id={game.id} crash={view.last_crash} />
    {/if}

    {#if view.activity.kind === "rendering"}
      <div class="window rise">
        <h2 class="window-title">Taking a picture</h2>
        <div class="rule"></div>
        <p class="say">
          The game is drawing a frame of {game.title} from your own disc, for this page's background.
        </p>
      </div>
    {/if}

    {#if settingUp && view.can_set_up && view.build}
      <DiscPanel {view} />
    {:else if view.set_up}
      <SavesPanel {view} />
    {:else if view.next === "get-the-game" || view.build_problem}
      <div class="window rise">
        <h2 class="window-title">Before you play</h2>
        <div class="rule"></div>
        <p class="say">
          The launcher plays The Moon People's {game.port} port, built from your own {game.discs[0].region} disc.
          Get the port, then show the launcher where your build is.
        </p>
        {#if view.build_problem}<p class="say problem">{view.build_problem}</p>{/if}
        <ul class="rows">
          <li>
            <button class="row chosen" onclick={() => attempt(() => api.openLink(game.get_url))}
              >Get the game on GitHub</button
            >
          </li>
          <li><button class="row" onclick={choose}>Choose a build</button></li>
          <li class="row faint">Set up from your disc</li>
          <li class="row faint">Play</li>
        </ul>
        <div class="footer"><span>Nothing from the game is downloaded</span></div>
      </div>
    {:else if !view.can_set_up}
      <div class="window rise">
        <h2 class="window-title">Coming later</h2>
        <div class="rule"></div>
        <p class="say">Setting {game.title} up from its disc comes in a later update of the launcher.</p>
      </div>
    {/if}

    {#if view.set_up}
      <details class="window about">
        <summary class="window-title">About this set-up</summary>
        <div class="rule"></div>
        <dl>
          <dt>Disc</dt>
          <dd>{fileName(view.set_up.disc)}</dd>
          <dt>Set up</dt>
          <dd>{when(view.set_up.finished)}</dd>
          <dt>Build</dt>
          <dd class="mono">{view.build?.dir ?? view.set_up.build}</dd>
          <dt>Data</dt>
          <dd class="mono">{view.data}</dd>
          <dt>Saves</dt>
          <dd class="mono">{view.save}</dd>
        </dl>
      </details>
    {/if}
  </aside>
</main>

<style>
  main {
    position: relative;
    height: 100%;
    overflow: hidden;
  }
  .intro {
    position: absolute;
    left: 48px;
    bottom: 64px;
    max-width: min(640px, 56%);
    display: flex;
    flex-direction: column;
    font-family: var(--font-game);
  }
  .chapter {
    font-size: calc(18px * var(--game-size));
    letter-spacing: 0.2em;
    color: var(--accent);
    text-shadow: 0 1px 2px var(--shadow);
  }
  h1 {
    margin-top: 6px;
    font-family: var(--font-title);
    font-size: clamp(44px, 5.8vw, 82px);
    white-space: nowrap;
    line-height: 0.95;
    color: var(--title);
    -webkit-text-stroke: 2px var(--title-edge);
    paint-order: stroke fill;
    text-shadow:
      0 3px 0 var(--title-edge),
      0 10px 24px var(--shadow);
  }
  .hero {
    margin-top: 12px;
    font-size: calc(20px * var(--game-size));
    color: var(--text);
    text-shadow: 0 1px 2px var(--shadow);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: stretch;
    gap: 14px;
    margin-top: 28px;
  }
  .mods {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px;
    font-size: calc(18px * var(--game-size));
    letter-spacing: 0.08em;
  }
  .where {
    margin-top: 20px;
    font-size: calc(16px * var(--game-size));
    text-shadow: 0 1px 2px var(--shadow);
  }
  .side {
    position: absolute;
    right: 40px;
    top: 28px;
    bottom: 64px;
    width: min(380px, 40%);
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 22px;
    overflow: auto;
    padding: 6px;
  }
  .say {
    padding: 12px 18px 4px;
    font-size: calc(16px * var(--game-size));
    line-height: 1.5;
    color: var(--text-soft);
  }
  .say.problem {
    color: var(--err);
  }
  .rows li {
    list-style: none;
  }
  .rows button.row {
    width: 100%;
    border: 0;
    background: none;
    text-align: left;
  }
  .rows button.row.chosen {
    background: var(--select);
  }
  .about summary {
    cursor: pointer;
    list-style: none;
  }
  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 4px 14px;
    margin: 0;
    padding: 10px 18px 14px;
    font-size: calc(14px * var(--game-size));
  }
  dt {
    color: var(--accent);
  }
  dd {
    margin: 0;
    color: var(--text-soft);
    overflow-wrap: anywhere;
  }
</style>
