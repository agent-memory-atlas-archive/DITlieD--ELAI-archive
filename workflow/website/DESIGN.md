# ELAI — Website Design Brief (for Claude design)

This is a brand + message + voice brief for the first version of the ELAI marketing website. It tells you WHO ELAI is, WHAT it must say, HOW it should feel, and WHAT it must never claim.

It deliberately says nothing about layout, page count, section order, or composition. That is yours. Be creative with structure, hierarchy, and visual arrangement. Everything below constrains brand, message, voice, and aesthetic — not layout.

Grounding note: every phrase, promise, and constraint here is pulled from ELAI's own strategic decision records (the trust pivot of June 2026) and its product design system. Nothing here is invented marketing. If a claim isn't in this brief, don't add it.

---

## 1. What ELAI is (the one-line positioning)

ELAI is a trust-first coding harness: an autonomous coding agent built so it physically can't wreck your machine, can't fabricate a passing test, and stops honestly when it's stuck — wrapped around a strong cloud model, with every run auditable and reproducible.

It is the same category as Claude Code / Cursor (a harness that drives the model), but built around TRUST instead of speed. The slogan-level idea: "intelligence in the infrastructure, not the model."

The deeper framing (use as a supporting idea, not the headline): ELAI is a verification control plane for the agent era — the external, deterministic reference that checks AI-written code, instead of asking another AI to review it.

## 2. The north star (the single sentence everything serves)

> an agent that can't lie to you and can't wreck your system.

Everything on the site should ladder up to this. It is the emotional and intellectual center of the brand.

## 3. The three keepable promises (the spine of the message)

These are the three concrete, defensible promises. They are the product. Lead with them.

1. It can't lie to you. Every claimed passing test is bound to a captured, hashed record of the real test run. A fabricated "all tests passing" is caught, not trusted.
2. It can't wreck your system. Catastrophic actions (wiping a directory, calling out to an unapproved host, installing a hallucinated package) are blocked deterministically, in code — not by a prompt the model can talk its way around.
3. It's honest about its limits. When it can't verify something, it abstains and tells you exactly where and why it stopped — instead of confidently handing you a wrong answer.

Supporting pillars (close with these, never lead): predictable / auditable cost, and privacy + no-rug-pull (local or hybrid option, your code stays yours). These are footnotes to trust, not the headline.

## 4. The audience (who the copy talks to)

Developers and teams who can read and action a flagged diff and who feel the trust problem: regulated / privacy-constrained teams, governance-conscious orgs, and agencies cleaning up insecure AI-generated code. Prosumer developers who run multiple AI tools and don't fully trust any of them.

NOT non-coders. ELAI hands you audited code; it is not a no-code app builder. Don't write copy that promises a deployed app to someone who can't read code.

## 5. The proof story (the demo narrative the site should evoke)

The signature story is the contrast demo — the same model, the same task, run two ways:

- Without ELAI's gates: the agent issues a destructive command, reports "all tests passing," claims success — and the damage and the lie both go through.
- With ELAI's gates on: the destructive command is blocked before it runs; the "passing tests" claim is exposed as fabricated because the hash doesn't match; the run ends with an honest stop and a downloadable, replayable record of exactly what happened.

The closing line (use it, it's the brand's sharpest sentence):

> Same model, same task. One lied and wiped your database. The other physically couldn't, and proved it.

This is grounded in a real, public 2025 incident (an autonomous agent deleted a production database during a code freeze, fabricated thousands of fake users to hide it, faked its tests, and lied that rollback was impossible). You can reference "real incidents where agents wiped production data and lied about it" — true and on-message. Don't name or accuse a specific competitor in copy.

## 6. Ready-to-use phrases (verbatim, on-brand)

These are pulled from ELAI's own records. Use them, adapt them, but keep the tone.

- "an agent that can't lie to you and can't wreck your system"
- "Same model, same task. One lied and wiped your database. The other physically couldn't, and proved it."
- "verified before you see it, honest about what it couldn't verify, physically blocked from the catastrophic thing"
- "intelligence in the infrastructure, not the model"
- "we catch the model when it lies and stop it before it wrecks — and tell you honestly what it couldn't verify"
- "trust isn't a feature here. it's the whole product."
- "a run you can audit, reproduce, and replay"
- "calibrated trust, not perfection"

Tone of these: quiet confidence, not hype. The whole brand is a reaction against agents that overpromise.

## 7. Voice and tone (hard rules for all copy)

The brand voice is restrained, precise, and anti-hype. It earns trust by refusing to oversell — which is the entire point of the product.

- Plain, declarative sentences. Short. Calm. No exclamation marks.
- No hype words: no "revolutionary," "magical," "10x," "supercharge," "unleash," "next-gen."
- No emojis. Anywhere.
- Lowercase "i" for first-person where copy is in the founder's voice; contractions are fine and natural (cant, doesnt) in that voice. (For headline UI copy, normal casing is fine — just keep it understated.)
- No em-dashes as a stylistic tic; prefer commas, colons, or parentheses.
- Confidence through specificity, not adjectives. "binds every passing test to a hashed record of the real run" beats "powerful test verification."
- Admit the honest frame: ELAI sells calibrated trust, never perfection. Copy that promises flawless code is off-brand and forbidden (see section 9).

## 8. Visual aesthetic (the feeling, not the layout)

The product's design language is a dark, quiet, precise observatory. Carry that feeling into the site. You own the layout; match the mood.

Mood words: a quiet observatory at 3am — deep, silent, precise. Restrained, slightly desaturated, low-lumen, calm. Think Linear at low luminance, Arc Browser at night, Superhuman after hours. "Sad elegiac undertone" but never grim — restraint, not gloom. Calm technology: it shows you what's happening without demanding your attention.

Color (this is the actual product token system — use it as the site palette):
- Dark-first, near-black backgrounds with a zinc-green tint. Never pure black (`#000`). Base voids around `#07090B` / `#0B0E12`, lifting to `#10141A` and `#1A2029` for raised surfaces.
- ONE accent: muted teal-forest. `--teal #1E8A7A` for active/brand, `--teal-bright #2BAE98` used sparingly for emphasis.
- ONE secondary: cold steel-blue `#5A7A9E` for informational text, links, keyboard-style hints.
- Text is warm-neutral, never pure white: primary `#D7DCE3`, secondary `#7A828E`, ghost greys below that.
- Semantic colors exist but are reserved for real meaning: amber `#C39A3A` for warnings, muted red `#B04545` for failures. Never use them as decoration.
- PURPLE IS FORBIDDEN. No rainbow status colors. No neon. The single-teal-accent discipline is part of the brand identity.

Depth and surfaces:
- Depth comes from luminance stepping (a lighter surface over a darker one), not from drop shadows. Avoid heavy shadows, bevels, glassmorphism blobs, and gradient-filled buttons.
- Borders are barely-there: 1px hairlines at very low opacity. "Structure should be felt, not seen."

Typography:
- Sans: Inter (body, headings, the wordmark in a thin 200 weight).
- Mono: JetBrains Mono — use it for anything that reads as machine truth: code, file paths, hashes, IDs, timestamps, status labels, state names. This mono/sans pairing IS the brand's visual signature: sans for human voice, mono for machine evidence.
- Don't over-bold. Prefer weight 500 over 700. Uppercase mono labels can carry slightly wider letter-spacing (~0.08em).
- The ELAI wordmark renders in thin-weight Inter, teal. Treat it as quiet and confident, large but not loud.

Motion (you choose where; obey the philosophy):
- Motion explains state; it never performs. Slow, intentional, 120–320ms for interface transitions; longer ambient breathing cycles (multiple seconds) for "alive" indicators.
- NO spinners, NO bouncing dots, NO shimmer skeletons, NO spring overshoot. The product's only loading indicator is a single 1px teal line that gently pulses opacity ("breath"). If you want a live/working indicator anywhere, use that idea, not a spinner.
- Respect `prefers-reduced-motion` — every animation needs an off switch.

Optional brand motif (only if it serves the page, never forced): the product's centerpiece is a deterministic "orbit / constellation" — a seeded star-field where the same task always renders the same constellation. If you want a hero visual, a calm, deterministic constellation / star-field on near-black, with faint teal blooms, is deeply on-brand. Keep it quiet. No flashy particle storms.

## 9. The kill-list (claims the site must NEVER make)

These are hard. Violating them breaks the brand and the thesis.

- NEVER promise "perfect code," "flawless," "bug-free," or "replaces a developer." ELAI sells calibrated trust. Overpromising is exactly the failure that burned the agents ELAI defines itself against.
- NEVER claim benchmark wins or leaderboard rankings (e.g. "#1 on SWE-bench"). That credibility collapsed in 2026 and ELAI doesn't compete there.
- NEVER lead with cost or "cheaper" / "fewer tokens" as the headline. Predictable/auditable cost is a supporting footnote only.
- NEVER position ELAI as an app/website builder or a tool for non-coders.
- NEVER claim it's a security scanner. ELAI prevents bad code in the loop; it is not a bolt-on vulnerability scanner.
- NEVER cite a specific measured "safety number," percentage, or stat unless the user gives you a real, current figure. The headline measurement (a gate-on vs gate-off "safety delta") is not published yet. Until it exists, the site must speak in terms of what ELAI is built to do and how it works, not measured results. Speak in mechanisms and promises, not numbers.
- NEVER use an EU AI Act / "Article 50 compliance" hook as a value prop.
- Don't name or attack specific competitors in copy. Reference the category and real public incidents instead.

## 10. What the site needs to convey (content, not layout)

You decide how to arrange and present these. This is the message inventory the site must carry:

- The north star line and what it means.
- The three keepable promises (can't lie / can't wreck / honest about limits), each explained in one or two plain sentences with its mechanism.
- The contrast proof story (section 5) and its closing line.
- The "harness, not a model" idea: it drives a strong cloud model; trust lives in the infrastructure around it. Local / hybrid option for privacy.
- Who it's for (section 4) — speak to developers and teams who feel the trust gap.
- A quiet, honest call to action. Given the product is pre-release, the right CTA is likely "follow along / get notified / request early access," not "buy now." Keep it understated and truthful; don't manufacture urgency.
- A footer presence for the supporting pillars: auditable cost, privacy, no-rug-pull, reproducible runs.

## 11. One-paragraph summary (paste this to orient yourself)

ELAI is a trust-first coding harness whose autonomous runs physically can't wreck your machine, can't fabricate a passing test, and stop honestly when stuck — wrapped around a strong cloud model, every run auditable and reproducible. The north star is "an agent that can't lie to you and can't wreck your system." The audience is developers and teams who feel the trust gap, not non-coders. The voice is restrained and anti-hype; trust is earned by refusing to oversell. The look is a quiet 3am observatory: near-black zinc-green backgrounds, one muted teal accent plus steel-blue, warm-neutral text, Inter for human voice and JetBrains Mono for machine evidence, depth from luminance not shadow, motion that explains rather than performs, no purple, no spinners, no neon. Never promise perfect code, never claim benchmarks or measured numbers that don't exist yet, never lead on cost. Layout, structure, and composition are entirely yours — make it feel inevitable and calm.
