<!-- FIM_PREFIX: Hero section skeleton — do not modify above this line -->
<script lang="ts">
  export let title: string = "{{project_name}}";
  export let subtitle: string = "{{description}}";
</script>

<section class="hero" data-section="hero">
  <!-- FIM_FILL_START -->
  <h1>{title}</h1>
  <p>{subtitle}</p>
  <!-- FIM_FILL_END -->
</section>

<style>
  .hero {
    padding: 6rem 1.5rem;
    background: hsl(var(--bg));
    color: hsl(var(--fg));
    text-align: center;
  }
  .hero h1 { font-size: 3rem; color: hsl(var(--primary)); }
  .hero p  { font-size: 1.15rem; opacity: 0.8; margin-top: 1rem; }
</style>
<!-- FIM_SUFFIX: Hero section skeleton — do not modify below this line -->
