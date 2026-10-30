<!-- Dark Cloud's set-up progress: the gold moon of the Chronicle logo, waxing. -->
<script lang="ts">
  import { moonPath } from "../lib/format";

  let { lit, size = 120, label }: { lit: number; size?: number; label: string } = $props();
  const id = $props.id();
</script>

<svg
  width={size}
  height={size}
  viewBox="-1.25 -1.25 2.5 2.5"
  role="progressbar"
  aria-label={label}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={Math.round(lit * 100)}
>
  <defs>
    <radialGradient id="lit-{id}" cx="0.35" cy="0.35" r="0.8">
      <stop offset="0" style="stop-color: var(--gold-knot-hi)"></stop>
      <stop offset="0.6" style="stop-color: var(--dc1-accent-bright)"></stop>
      <stop offset="1" style="stop-color: var(--dc1-accent)"></stop>
    </radialGradient>
  </defs>
  <circle r="1" style="fill: var(--footer); stroke: var(--frame)" stroke-width="0.06"></circle>
  <path class="lit" d={moonPath(lit)} fill="url(#lit-{id})"></path>
</svg>

<style>
  svg {
    display: block;
    flex: none;
  }
  .lit {
    transition: d var(--slow) var(--ease);
  }
</style>
