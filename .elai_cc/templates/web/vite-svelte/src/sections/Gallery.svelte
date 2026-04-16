<!-- FIM_PREFIX: Gallery section skeleton — do not modify above this line -->
<script lang="ts">
  type Image = { src: string; alt: string };
  export let images: Image[] = [];
</script>

<section class="gallery" data-section="gallery">
  <!-- FIM_FILL_START -->
  {#each images as img}
    <figure>
      <img src={img.src} alt={img.alt} loading="lazy" />
    </figure>
  {/each}
  <!-- FIM_FILL_END -->
</section>

<style>
  .gallery {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    padding: 4rem 1.5rem;
    background: hsl(var(--bg));
  }
  figure { overflow: hidden; border-radius: 0.5rem; }
  img { width: 100%; height: auto; display: block; }
</style>
<!-- FIM_SUFFIX: Gallery section skeleton — do not modify below this line -->
