<!-- The ports' icon's purple clouds, as a band of brushed strokes: ellipses pulled sideways by noise, back to
     front from the lightest purple to the deepest. -->
<script lang="ts">
  let { height = 160, flip = false }: { height?: number; flip?: boolean } = $props();
  const id = $props.id();

  // Thin horizontal strokes in a 1200 x 200 box, as the icon's are painted: each layer darker and lower than the
  // last. A small fixed random keeps every draw the same.
  const layers = ["var(--cloud-grey)", "var(--cloud-hi)", "var(--cloud)", "var(--cloud-deep)"];
  let seed = 11;
  const random = () => ((seed = (seed * 16807) % 2147483647) - 1) / 2147483646;
  const strokes = layers.flatMap((fill, layer) =>
    Array.from({ length: 9 + layer * 3 }, () => ({
      fill,
      cx: random() * 1300 - 50,
      cy: 50 + layer * 40 + random() * 26,
      rx: 120 + random() * 220,
      ry: 7 + layer * 3 + random() * 10,
    })),
  );
  // The lowest layer runs into the solid band under the clouds.
  strokes.push({ fill: "var(--cloud-deep)", cx: 600, cy: 210, rx: 760, ry: 34 });
</script>

<svg
  class="clouds"
  class:flip
  viewBox="0 0 1200 200"
  preserveAspectRatio="xMidYMax slice"
  style="height: {height}px"
  aria-hidden="true"
>
  <defs>
    <filter id="brush-{id}" x="-10%" y="-40%" width="120%" height="180%">
      <feTurbulence type="fractalNoise" baseFrequency="0.006 0.06" numOctaves="4" seed="7" result="noise" />
      <feDisplacementMap
        in="SourceGraphic"
        in2="noise"
        scale="30"
        xChannelSelector="R"
        yChannelSelector="G"
      />
      <feGaussianBlur stdDeviation="0.8" />
    </filter>
  </defs>
  <g filter="url(#brush-{id})">
    {#each strokes as stroke, i (i)}
      <ellipse cx={stroke.cx} cy={stroke.cy} rx={stroke.rx} ry={stroke.ry} fill={stroke.fill} />
    {/each}
  </g>
</svg>

<style>
  .clouds {
    display: block;
    width: 100%;
    pointer-events: none;
  }
  .flip {
    transform: scaleY(-1);
  }
</style>
