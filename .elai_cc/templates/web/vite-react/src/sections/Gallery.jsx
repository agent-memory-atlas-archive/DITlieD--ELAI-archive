// FIM_PREFIX: Gallery section skeleton — do not modify above this line
export default function Gallery({ images = [] }) {
  return (
    <section className="gallery" data-section="gallery">
      {/* FIM_FILL_START */}
      {images.map((img, i) => (
        <figure key={i}>
          <img src={img.src} alt={img.alt} loading="lazy" />
        </figure>
      ))}
      {/* FIM_FILL_END */}
      <style>{`
        .gallery { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; padding: 4rem 1.5rem; background: hsl(var(--bg)); }
        figure { overflow: hidden; border-radius: 0.5rem; }
        img { width: 100%; height: auto; display: block; }
      `}</style>
    </section>
  );
}
// FIM_SUFFIX: Gallery section skeleton — do not modify below this line
