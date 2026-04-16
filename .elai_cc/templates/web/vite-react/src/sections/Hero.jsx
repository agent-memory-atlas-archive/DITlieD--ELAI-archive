// FIM_PREFIX: Hero section skeleton — do not modify above this line
export default function Hero({ title = "{{project_name}}", subtitle = "{{description}}" }) {
  return (
    <section className="hero" data-section="hero">
      {/* FIM_FILL_START */}
      <h1>{title}</h1>
      <p>{subtitle}</p>
      {/* FIM_FILL_END */}
      <style>{`
        .hero { padding: 6rem 1.5rem; text-align: center; background: hsl(var(--bg)); color: hsl(var(--fg)); }
        .hero h1 { font-size: 3rem; color: hsl(var(--primary)); }
        .hero p  { font-size: 1.15rem; opacity: 0.8; margin-top: 1rem; }
      `}</style>
    </section>
  );
}
// FIM_SUFFIX: Hero section skeleton — do not modify below this line
