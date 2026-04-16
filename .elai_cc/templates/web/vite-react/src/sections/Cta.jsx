// FIM_PREFIX: Cta section skeleton — do not modify above this line
export default function Cta({ heading = "Get started", href = "#", label = "Begin" }) {
  return (
    <section className="cta" data-section="cta">
      {/* FIM_FILL_START */}
      <h2>{heading}</h2>
      <a className="cta-button" href={href}>{label}</a>
      {/* FIM_FILL_END */}
      <style>{`
        .cta { padding: 4rem 1.5rem; text-align: center; background: hsl(var(--accent) / 0.1); color: hsl(var(--fg)); }
        .cta-button { display: inline-block; margin-top: 1rem; padding: 0.75rem 1.75rem; background: hsl(var(--primary)); color: hsl(var(--bg)); border-radius: 0.5rem; font-weight: 600; }
      `}</style>
    </section>
  );
}
// FIM_SUFFIX: Cta section skeleton — do not modify below this line
