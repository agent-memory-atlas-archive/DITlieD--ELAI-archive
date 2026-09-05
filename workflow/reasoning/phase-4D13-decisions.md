4D.13 Source Quality Table — Decision Doc

What: SourceQualityTable in crates/web_search/src/source_quality.rs.
Wraps elai_core::SourceTierConfig to provide domain-to-tier lookup and f64 scoring.
Bridges the 4-tier SourceTier system (elai-core) with the 5-tier SourceQuality system (research_tree).

Key decisions:
1. Option A (wrap core): SourceQualityTable owns a SourceTierConfig. No domain list duplication.
2. 5-tier score bridge: T1Auth=1.0, T2Strong=0.8, T3Weak=0.6, T4Rejected=0.4. Named constants.
   T3Weak is the default for unknown domains (not T4Rejected). Score 0.2 reserved for Tier5 but not produced by score() since SourceTier has no 5th variant.
3. Domain extraction: url::Url::parse (not parse_domain from lib.rs which has query-string bug).
4. classify_source(domain, config) works for bare domains — extract_domain handles no-scheme input.
5. Subdomain globs: *.gov/T1, *.edu/T1, *.ac.uk/T2, *.ac.jp/T2 — already in source-tiers.toml patterns (\.gov$, \.edu$, \.ac\.[a-z]{2}$). No new pattern infra needed; classify_source handles them via tier1_patterns.
6. TOML config: [research.trusted_domains] in elai_cc.toml with 200+ domains. Constructor validates after deser.
7. Two constructors: new_from_elai_cc_toml(path) for prod, new_from_config(SourceTierConfig) for tests/Oracle in-memory.

Expected test outcomes:
- lookup("arxiv.org") -> T1Authoritative
- lookup("w3schools.com") -> T4Rejected
- lookup("unknown-site.com") -> T3Weak
- score("arxiv.org") -> 1.0
- score("docs.rs") -> 0.8 (T2 in source-tiers.toml, NOT T1 per core config even though research_tree has it as T2)
- Subdomain *.gov -> T1Authoritative (via pattern)

Unknowns:
- research_tree classifies docs.rs as T2 and arxiv.org as T1. source-tiers.toml has docs.rs as T1 and arxiv.org as T1. Score for docs.rs via SourceQualityTable=1.0 (T1 in core config) vs research_tree=0.8 (T2). This divergence is by design — core config is broader, research_tree is narrower.
