<!-- The gold moon of the Chronicle logo, waxing as a long job goes: the launcher's progress. Its shadowed part is
     the icon's cloud purple. -->
<script lang="ts">
  import { moonPath } from "../lib/format";

  let { lit, size = 96, label }: { lit: number; size?: number; label: string } = $props();
  const id = $props.id();
</script>

<svg
  class="moon"
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
      <stop offset="0" stop-color="var(--gold-hi)" />
      <stop offset="0.6" stop-color="var(--moon-gold)" />
      <stop offset="1" stop-color="var(--gold-mid)" />
    </radialGradient>
    <filter id="glow-{id}" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="0.12" />
    </filter>
  </defs>
  <circle r="1.08" fill="var(--moon-gold)" opacity={0.18 + lit * 0.3} filter="url(#glow-{id})" />
  <circle r="1" fill="var(--cloud)" />
  <path class="lit" d={moonPath(lit)} fill="url(#lit-{id})" />
</svg>

<style>
  .moon {
    display: block;
    flex: none;
  }
  .lit {
    transition: d var(--slow) var(--ease);
  }
</style>
