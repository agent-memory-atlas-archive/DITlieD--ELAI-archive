<script lang="ts">
  import Hero from "./sections/Hero.svelte";
  import Features from "./sections/Features.svelte";
  import Cta from "./sections/Cta.svelte";
  import Gallery from "./sections/Gallery.svelte";
  import Footer from "./sections/Footer.svelte";
</script>

<main>
  <Hero />
  <Features />
  <Gallery />
  <Cta />
  <Footer />
</main>

<style>
  main { display: flex; flex-direction: column; min-height: 100vh; }
</style>
