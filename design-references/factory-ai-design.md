# design.md — factory.ai

A reusable design reference extracted deterministically from factory.ai's own
compiled CSS (the two `/_next/static/css/*.css` bundles + page HTML), not from
visual inspection. Every token below is a literal value from the stylesheet
unless marked `[inferred]` (a name or intent deduced from how the value is used).

Source: https://factory.ai/  ·  extracted 2026-06-12  ·  stack: Next.js + Tailwind v4

---

## 1. Identity in one line

Warm near-black canvas, a single confident orange accent, Geist (sans + mono),
tight corners, and fast/subtle motion. Dark-first; reads as a serious developer
/ infra product, not a consumer app. The "brand" is restraint plus one hot accent.

## 2. Color

### Brand accent (the whole identity rides on this one hue)
```
--accent-100: #ef6f2e   /* lighter orange — hover / highlight        */
--accent-200: #ee6018   /* PRIMARY accent — CTAs, links, focal items  */
--accent-300: #d15010   /* darker orange — pressed / deep state       */
```
One accent only. No secondary brand color. Everything else is neutral. [inferred: 200 is the canonical brand orange because it's the mid-stop and maps to `--color-accent-200`.]

### Base surfaces (dark theme — the default)
```
--dark-base-primary:   #020202   /* page background — near-pure black */
--dark-base-secondary: #101010   /* raised surfaces / cards           */
```

### Base surfaces (light theme — defined but secondary)
```
--light-base-primary:   #eee
--light-base-secondary: #fafafa
```

### Neutrals — WARM grey, not pure grey (this is a deliberate tell)
The neutral ramp is tinted brown/warm (note the `r > g > b` skew), which is what
keeps the near-black from feeling cold/blue. Do not substitute pure greys.
```
--neutral-100:  #d6d3d2
--neutral-200:  #ccc9c7
--neutral-300:  #b8b3b0
--neutral-400:  #a49d9a
--neutral-500:  #8a8380
--neutral-600:  #5c5855
--neutral-700:  #4d4947
--neutral-800:  #3d3a39
--neutral-900:  #2e2c2b
--neutral-1000: #1f1d1c
```
Semantic aliases map onto this ramp and FLIP between themes (e.g. `--color-base-100`
= `neutral-100` in light, `neutral-1000` in dark) so component code stays
theme-agnostic. Replicate that pattern: name by role (base-100…base-1000),
not by literal shade.

### Status colors (Tailwind defaults, oklch — use only for state, never brand)
blue / emerald / green / orange / red / yellow at 400+500 stops, plus gray.
These are stock Tailwind; they signal success/error/info, not identity.
Absolutes: `#000`, `#fff`.

## 3. Typography

```
--font-sans: "Geist", "Geist Fallback"
--font-mono: "Geist Mono", "Geist Mono Fallback"
```
Geist (Vercel's typeface) for everything; Geist Mono for code/labels/technical
text. Mono is used deliberately (`!important` overrides exist) for a
developer-tool texture.

Weights in use: 300 / 400 / 500 / 600 / 700.
```
--font-weight-light:    300
--font-weight-normal:   400
--font-weight-medium:   500
--font-weight-semibold: 600
--font-weight-bold:     700
```

Type scale (rem):
```
xs .75 · sm .875 · base 1 · lg 1.125 · xl 1.25 · 2xl 1.5 · 3xl 1.875 · 4xl 2.25 · 5xl 3
```

Line-height: tight 1.25 · snug 1.375 · normal · relaxed 1.625 · loose 2
Letter-spacing: tight -.025em · normal 0 · wide .025em · wider .05em
[inferred convention: tight tracking pairs with the large display sizes (3xl–5xl);
body stays at normal. This is the standard Geist display pattern.]

## 4. Spacing & layout

Base spacing unit: `.25rem` (4px) — the Tailwind v4 multiplier; all spacing is a
multiple of this.

Container max-widths (rem): xs 20 · sm 24 · xl 36 · 2xl 42 · 5xl 64 · 6xl 72.
Content tops out around 64–72rem; comfortable, centered, not full-bleed text.

## 5. Radius — tight

```
xs .125 · sm .1875 · md .25 · DEFAULT .375 · lg .375 · xl .5 · 2xl .625 · 3xl .75  (rem)
```
Default corner is `.375rem` (~6px). Corners are crisp/tight, reinforcing the
serious-tooling feel. Nothing pill-shaped or playful.

## 6. Motion — fast and subtle

```
--default-transition-duration: .15s
--default-transition-timing:    cubic-bezier(.4,0,.2,1)   /* ease-in-out */
--ease-out:                     cubic-bezier(0,0,.2,1)
```
Transitions are short (150ms) and eased. Provided animations are minimal
(`pulse`, `spin`). Motion supports, never performs. Respect
`prefers-reduced-motion` when you rebuild.

## 7. Effects

Blur scale present (sm 8 · md 12 · xl 24 · 2xl 40 · 3xl 64 px) — used for
backdrop/glass surfaces over the near-black. No custom shadow tokens defined;
elevation comes from the `base-primary` → `base-secondary` surface step and blur,
not heavy drop-shadows.

## 8. Voice / tone (from page copy)

Confident, builder-facing, future-oriented. Sample headline:
"Ready to build the software of the future?" Nav/footer vocabulary is enterprise
+ developer: Enterprise, Pricing, Docs, Security, SLA, DPA, Open Source,
"Contact Sales", "Factory for Science". CTA voice is direct and sales-led
("Contact Sales"), not cute.

## 9. How to reuse this as a template (anti-generic checklist)

To build a *different* site that borrows this language without cloning it:
- Keep the structure: ONE hot accent over warm-tinted near-black neutrals. Swap
  the hue (e.g. cyan/violet) but keep the "single accent + warm dark neutrals" rule.
- Keep Geist OR pick one authored sans + its mono companion. Mono-for-technical
  text is doing real identity work — don't drop it for a system stack.
- Keep tight radii (~6px default) and 150ms eased motion. Loosening either makes
  it read consumer/generic.
- Theme by role-named tokens that flip per theme, not by literal hex in components.
- Tells that would make a clone read as "AI-default slop" and must be avoided:
  pure (cold) greys instead of the warm ramp; a second competing brand color;
  pill buttons; large soft drop-shadows; a system-font stack.

## 10. Token block (copy-paste starting point, dark theme)

```css
:root {
  /* accent */
  --accent-100:#ef6f2e; --accent-200:#ee6018; --accent-300:#d15010;
  /* surfaces (dark) */
  --bg:#020202; --surface:#101010;
  /* warm neutral ramp */
  --n-100:#d6d3d2; --n-200:#ccc9c7; --n-300:#b8b3b0; --n-400:#a49d9a;
  --n-500:#8a8380; --n-600:#5c5855; --n-700:#4d4947; --n-800:#3d3a39;
  --n-900:#2e2c2b; --n-1000:#1f1d1c;
  /* type */
  --font-sans:"Geist",system-ui,sans-serif;
  --font-mono:"Geist Mono",ui-monospace,monospace;
  /* radius / motion */
  --radius:.375rem;
  --dur:.15s; --ease:cubic-bezier(.4,0,.2,1);
}
```

---
Extraction method (reproducible): `curl` the page HTML, read the linked
`/_next/static/css/*.css` bundles, `grep` out `--*` custom properties,
`font-family`, the type/spacing/radius scales, and page copy. No screenshots,
no model vision — same input yields the same file.
