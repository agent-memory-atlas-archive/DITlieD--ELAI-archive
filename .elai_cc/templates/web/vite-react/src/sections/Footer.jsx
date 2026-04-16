// FIM_PREFIX: Footer section skeleton — do not modify above this line
export default function Footer({ owner = "{{author}}", year = new Date().getFullYear() }) {
  return (
    <footer className="site-footer" data-section="footer">
      {/* FIM_FILL_START */}
      <div className="columns" />
      <p className="copyright">&copy; {year} {owner}</p>
      {/* FIM_FILL_END */}
      <style>{`
        .site-footer { padding: 3rem 1.5rem; background: hsl(var(--bg)); color: hsl(var(--fg) / 0.7); border-top: 1px solid hsl(var(--fg) / 0.1); }
        .columns { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1.5rem; }
        .copyright { margin-top: 2rem; text-align: center; font-size: 0.9rem; }
      `}</style>
    </footer>
  );
}
// FIM_SUFFIX: Footer section skeleton — do not modify below this line
