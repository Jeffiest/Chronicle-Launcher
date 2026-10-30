<!-- Dark Chronicle's set-up progress: a ring of the Options backdrop's gold line, filling, a knot at its end. -->
<script lang="ts">
  let { lit, size = 200, label }: { lit: number; size?: number; label: string } = $props();
  const id = $props.id();
  const R = 150;
  const LENGTH = 2 * Math.PI * R;
  const f = $derived(Math.min(1, Math.max(0, lit)));
  const end = $derived({ x: R * Math.sin(2 * Math.PI * f), y: -R * Math.cos(2 * Math.PI * f) });
</script>

<svg
  width={size}
  height={size}
  viewBox="-180 -180 360 360"
  role="progressbar"
  aria-label={label}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={Math.round(f * 100)}
>
  <defs>
    <linearGradient id="ring-{id}" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" style="stop-color: var(--gold-hi)"></stop>
      <stop offset="0.5" style="stop-color: var(--gold)"></stop>
      <stop offset="1" style="stop-color: var(--gold-lo)"></stop>
    </linearGradient>
    <radialGradient id="knot-{id}" cx="0.38" cy="0.32" r="0.75">
      <stop offset="0" style="stop-color: var(--gold-knot-hi)"></stop>
      <stop offset="0.55" style="stop-color: var(--gold-knot)"></stop>
      <stop offset="1" style="stop-color: var(--gold-knot-lo)"></stop>
    </radialGradient>
    <filter id="emboss-{id}" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow
        dx="2.5"
        dy="3.5"
        stdDeviation="1.4"
        style="flood-color: var(--emboss)"
        flood-opacity="0.95"
      ></feDropShadow>
    </filter>
  </defs>
  <circle r={R} fill="none" style="stroke: var(--leather-dark)" stroke-width="10"></circle>
  <g filter="url(#emboss-{id})">
    <circle
      class="arc"
      r={R}
      fill="none"
      stroke="url(#ring-{id})"
      stroke-width="10"
      stroke-linecap="round"
      stroke-dasharray="{f * LENGTH} {LENGTH}"
      transform="rotate(-90)"
    ></circle>
    <circle r="124" fill="none" stroke="url(#ring-{id})" stroke-width="3"></circle>
    <circle cy={-R} r="15" fill="url(#knot-{id})"></circle>
    <circle cx={end.x} cy={end.y} r="13" fill="url(#knot-{id})"></circle>
  </g>
</svg>

<style>
  svg {
    display: block;
    flex: none;
  }
  .arc {
    transition: stroke-dasharray var(--slow) var(--ease);
  }
</style>
