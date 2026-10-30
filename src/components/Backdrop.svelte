<!-- Behind a game's page: a frame of the game, rendered from the player's own data or their own screenshot
     (`scene.rs`). Until there is one, Dark Cloud's menu darkness or Dark Chronicle's Options backdrop of leather and
     embossed gold lines, drawn here. -->
<script lang="ts">
  let { chapter, scene, blur = false }: { chapter: number; scene: string | null; blur?: boolean } = $props();
  const id = $props.id();
</script>

<div class="backdrop" class:blur aria-hidden="true">
  {#if scene}
    <img src={scene} alt="" />
  {:else if chapter === 2}
    <svg viewBox="0 0 1200 760" preserveAspectRatio="xMidYMid slice">
      <defs>
        <filter id="leather-{id}" x="0" y="0" width="100%" height="100%">
          <feTurbulence type="fractalNoise" baseFrequency="0.012" numOctaves="4" seed="8" result="big"
          ></feTurbulence>
          <feColorMatrix
            in="big"
            type="matrix"
            values="0 0 0 0 0.24  0 0 0 0 0.12  0 0 0 0 0.06  0 0 0 1.1 -0.45"
            result="blotch"
          ></feColorMatrix>
          <feTurbulence type="fractalNoise" baseFrequency="0.7" numOctaves="2" seed="3" result="fine"
          ></feTurbulence>
          <feColorMatrix
            in="fine"
            type="matrix"
            values="0 0 0 0 0.62  0 0 0 0 0.44  0 0 0 0 0.3  0 0 0 0.22 0"
            result="grain"
          ></feColorMatrix>
          <feMerge><feMergeNode in="blotch"></feMergeNode><feMergeNode in="grain"></feMergeNode></feMerge>
        </filter>
        <linearGradient id="line-{id}" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" style="stop-color: var(--gold-hi)"></stop>
          <stop offset="0.5" style="stop-color: var(--gold)"></stop>
          <stop offset="1" style="stop-color: var(--gold-lo)"></stop>
        </linearGradient>
        <radialGradient id="knot-{id}" cx="0.38" cy="0.32" r="0.75">
          <stop offset="0" style="stop-color: var(--gold-knot-hi)"></stop>
          <stop offset="0.55" style="stop-color: var(--gold-knot)"></stop>
          <stop offset="1" style="stop-color: var(--gold-knot-lo)"></stop>
        </radialGradient>
        <filter id="emboss-{id}" x="-10%" y="-10%" width="120%" height="120%">
          <feDropShadow
            dx="2.5"
            dy="3.5"
            stdDeviation="1.4"
            style="flood-color: var(--emboss)"
            flood-opacity="0.95"
          ></feDropShadow>
        </filter>
      </defs>
      <rect width="1200" height="760" style="fill: var(--leather)"></rect>
      <rect width="1200" height="760" filter="url(#leather-{id})"></rect>
      <g fill="none" stroke="url(#line-{id})" stroke-linecap="round" filter="url(#emboss-{id})">
        <circle cx="930" cy="330" r="236" stroke-width="7"></circle>
        <circle cx="930" cy="330" r="206" stroke-width="3"></circle>
        <path d="M560 -20 L 760 150 L 930 94 L 1100 150 L 1240 40" stroke-width="6"></path>
        <path d="M760 150 L 620 420 L 930 566 L 1180 470" stroke-width="6"></path>
        <path d="M1100 150 L 1180 470 M620 420 L 470 780 M930 566 L 930 790" stroke-width="5"></path>
        <path d="M760 150 C 806 200, 806 300, 760 352 C 714 300, 714 200, 760 150 Z" stroke-width="5"></path>
        <path d="M1100 150 C 1146 200, 1146 300, 1100 352 C 1054 300, 1054 200, 1100 150 Z" stroke-width="5"
        ></path>
        <path d="M930 566 C 980 610, 980 690, 930 740 C 880 690, 880 610, 930 566 Z" stroke-width="5"></path>
        <circle cx="40" cy="700" r="160" stroke-width="5" opacity="0.7"></circle>
      </g>
      <g filter="url(#emboss-{id})" fill="url(#knot-{id})">
        <circle cx="760" cy="150" r="15"></circle><circle cx="930" cy="94" r="11"></circle><circle
          cx="1100"
          cy="150"
          r="15"
        ></circle>
        <circle cx="620" cy="420" r="11"></circle><circle cx="930" cy="566" r="15"></circle><circle
          cx="1180"
          cy="470"
          r="11"
        ></circle>
      </g>
    </svg>
  {/if}
  <div class="shade" class:leather={!scene && chapter === 2}></div>
</div>

<style>
  .backdrop {
    position: absolute;
    inset: 0;
    overflow: hidden;
    background: radial-gradient(ellipse at 65% 35%, var(--footer), var(--win) 70%);
  }
  img,
  svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .blur img,
  .blur svg {
    inset: -40px;
    width: calc(100% + 80px);
    height: calc(100% + 80px);
    filter: blur(14px) brightness(0.8);
  }
  .shade {
    position: absolute;
    inset: 0;
    background:
      linear-gradient(90deg, var(--shade) 0%, var(--shade-soft) 34%, var(--clear) 58%),
      linear-gradient(0deg, var(--shade) 0%, var(--clear) 34%);
  }
  .shade.leather {
    background:
      linear-gradient(
        90deg,
        var(--leather-shade) 0%,
        var(--leather-shade-soft) 40%,
        var(--leather-clear) 62%
      ),
      radial-gradient(ellipse at 58% 42%, var(--leather-clear) 55%, var(--leather-shade) 100%);
  }
</style>
