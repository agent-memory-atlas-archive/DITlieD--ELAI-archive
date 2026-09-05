Phase 3F.6 decisions

What: detect_domains(file_path) -> Vec<String> free function + DOMAIN_KEYWORDS const table.

Key decision: const array of (&str, &[&str]) pairs, no new dependencies.
Rationale: pure string matching on lowercased path. Zero allocation beyond output Vec.
No regex needed — contains() on static keywords is sufficient and faster.

Known caveats (spec-mandated, not bugs):
- "mod" keyword matches all Rust mod.rs paths, "model", "module", etc.
- "ui" keyword (2 chars) matches "circuit", "equity", "suite", etc.
- "pipeline" matches knowledge pipeline, CI pipeline paths.
- "hook" matches plugin hook files (triggers RE domain).
- "component" matches ECS component paths (triggers dashboard domain).
All are documented in detect_domains docstring. Spec mandates these keywords;
domain override system (3F.6 optional, stub) can mitigate in practice.

Expected test outcome: 9 unit + 7 contract = 16 new tests, all green.
Unknowns: false positive rate in practice — mitigated by domain_overrides extension.
