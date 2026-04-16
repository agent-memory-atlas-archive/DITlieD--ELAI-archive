// FIM_PREFIX: Features section skeleton — do not modify above this line
export default function Features({ items = [] }) {
  return (
    <section className="features" data-section="features">
      {/* FIM_FILL_START */}
      {items.map((f, i) => (
        <article className="feature" key={i}>
          <h3>{f.title}</h3>
          <p>{f.body}</p>
        </article>
      ))}
      {/* FIM_FILL_END */}
      <style>{`
        .features { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1.5rem; padding: 4rem 1.5rem; background: hsl(var(--bg)); }
        .feature { padding: 1.5rem; border: 1px solid hsl(var(--fg) / 0.15); border-radius: 0.5rem; }
        .feature h3 { color: hsl(var(--primary)); }
      `}</style>
    </section>
  );
}
// FIM_SUFFIX: Features section skeleton — do not modify below this line
