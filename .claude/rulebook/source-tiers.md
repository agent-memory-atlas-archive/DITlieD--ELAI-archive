Source quality tiers (HARD RULE)

Loaded by: any task spawning the researcher agent, or web-searching to verify a claim, API contract, library behavior, or framework pattern.

Deterministic classification of web sources by trustworthiness. Researcher stamps every fact with the tier of its source. Plans grounded in tier-3 / tier-4 sources only must be flagged. Tier-1 wins on contradictions unless explicitly overridden by a more recent tier-2 finding.


TIER 1 — official / canonical (highest trust)

- developer.apple.com (all paths)
- swift.org (Swift evolution proposals, Swift Forums announcements from core-team accounts only)
- reactnative.dev (official RN docs)
- docs.expo.dev (Expo docs)
- docs.flutter.dev / api.flutter.dev
- developer.android.com (when iOS gig has Android sibling)
- spec docs from RFC editor (rfc-editor.org), W3C, IETF
- arxiv.org papers (when peer-reviewed venue cited or DOI matches)
- npmjs.com (for package existence + version metadata only, not for "is this good")
- pypi.org (same — metadata only)
- crates.io (same)
- swiftpackageindex.com (verified packages with documented release history)
- official GitHub repos owned by the org publishing the framework (e.g. github.com/apple, github.com/facebook/react-native, github.com/expo/expo). The org-owned repo is tier 1; forks and third-party mirrors are NOT.
- WWDC session transcripts on developer.apple.com
- App Store Review Guidelines on developer.apple.com


TIER 2 — high-quality community / vendor blogs

- github.com (third-party libraries with verified maintainer activity, release tags, license)
- stackoverflow.com (answers with score >= 10 AND accepted OR score >= 25 AND from user with 10K+ rep)
- official engineering blogs of large vendors (eng.uber.com, engineering.fb.com, netflix tech blog, shopify.engineering, github.blog)
- pointfree.co, swiftbysundell.com, kavsoft.dev, hackingwithswift.com (well-known Swift teaching sites with active maintenance)
- expo.dev/blog (Expo company blog)
- vercel.com/blog (Next.js / general infra)
- fly.io blog (infra patterns)
- ray wenderlich / kodeco (verify article date — anything pre-iOS 16 is stale)


TIER 3 — general blogs, medium, dev.to, personal sites

- medium.com posts (treat as opinion, never as truth, useful for "is this approach sane")
- dev.to articles
- personal blogs (named author, dated, no obvious agenda)
- reddit r/swift / r/iOSProgramming / r/reactnative top-comment chains
- youtube tutorials from named creators (use for orientation, never as the only source)
- substack tech publications


TIER 4 — low trust / avoid as primary source

- forum posts without attribution
- AI-generated content farms (medium aggregators with low-effort posts, fake author bios)
- pirated copies of paid courses
- outdated tutorials (anything pre-iOS 14 / pre-RN 0.65 / pre-Swift 5.5 — verify date)
- SEO spam ("top 10 X frameworks 2024" listicles)
- LLM-summarized blog posts published as articles
- StackOverflow answers from pre-2018 referencing deprecated APIs


CLASSIFICATION RULES

1. Default: every URL the researcher fetches gets tagged with its tier.
2. Mismatch detection: if a tier-3 fact contradicts a tier-1 source, tier-1 wins. Contradiction surfaced in research report's REFLECT section.
3. Single-source claim: any claim grounded in a single tier-3 or tier-4 source flagged "weak grounding — verify before plan integration."
4. No tier-4 in plans: tier-4 may inform exploration but NEVER cited in a baby step's GROUNDING block. Replace with tier-1 or tier-2 before integration.
5. Date sensitivity: tier-2 / tier-3 article older than 18 months touching iOS / SwiftUI / React Native / Expo SDK requires verification against current docs (annual releases break training assumptions).


WHEN A TIER-1 SOURCE IS UNAVAILABLE

If a topic has no tier-1 source (rare, happens for cross-stack integrations or recent additions), researcher must:
1. Triangulate at least 2 tier-2 sources that agree.
2. Note in report: "no tier-1 source available, triangulated from [s1] and [s2]."
3. Mark decision DecisionSource=RESEARCH, confidence=MEDIUM.

If only tier-3 sources exist: mark DecisionSource=FALLBACK and alert user before plan integration.


REGISTRY UPDATES

List above is seeded. New domains added by editing this file. Adding a tier-1 entry requires user approval (one per request, dont batch). Removing a tier-1 or tier-2 entry requires justification (e.g. domain ownership changed, content quality dropped).
