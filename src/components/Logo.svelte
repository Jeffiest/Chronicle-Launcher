<!-- The Chronicle logo: the sky in its letters, the gold trim, the two moons. The Moon People's, from the
     Chronicle README. -->
<script lang="ts">
  import logo from "../assets/chronicle-logo.webp";

  let { height = 40 }: { height?: number } = $props();
</script>

<img class="logo" src={logo} alt="Chronicle" style="height: {height}px" />

<style>
  .logo {
    display: block;
    width: auto;
    filter: drop-shadow(0 3px 6px var(--shadow));
  }
</style>
