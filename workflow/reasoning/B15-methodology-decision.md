B-15 baseline-methodology: cfg(test)-vs-prod disambiguation required BEFORE any future .unwrap/HashMap/entropy burndown
Session: s-22600 sub-phase trancheI | Date: 2026-06-01

Problem statement

Four consecutive B-15 sub-tranches (Tranche E rules / Tranche H.2 knowledge entropy / Tranche I.1 knowledge unwrap / Tranche I.2 orchestrator unwrap) opened with stubs.md headline raw counts of 287 / 4 / 320 / 784 (1395 raw total). Each closed VERIFIED-COMPLIANT-NULL-RESULT after the context-compiler ran a per-file cfg(test) boundary check and discovered every site (or all-but-3 in orchestrator unwrap) was inside #[cfg(test)] modules or already-exempt under CLAUDE.md case-3 (const/static init with provably valid literals). Zero production migration work resulted across four full HEAVY-tier tranches.

Root cause

stubs.md transcription pattern quoted raw Grep --count results (e.g. "session_dag.rs 171", "reranker.rs 47") without filtering for #[cfg(test)] blocks. The headline numbers were CORRECT raw counts but MISLEADING as prod-migration scope estimates. Three failure modes contributed:
1. Raw count includes test-module unwraps which CLAUDE.md explicitly exempts.
2. Raw count includes doc-comment ban-text matches (e.g. `// no .unwrap() outside test code`) which are not real call sites.
3. Raw count cannot distinguish case-3 LazyLock-static-regex exemptions (orchestrator deterministic_auditor.rs L1219/L1269/L1318) from real prod-migration targets.

DI6 canonical methodology (going-forward HARD RULE)

Every future B-15 tranche (or any tranche touching .unwrap / .expect / HashMap / SystemTime / entropy) MUST run prod-side filter FIRST before declaring scope. Two-minute Bash one-liner:

  for f in path/*.rs; do
    cfg_test=$(grep -n "^#\[cfg(test)\]" "$f" | head -1 | cut -d: -f1)
    first=$(grep -n "\.unwrap()" "$f" | head -1 | cut -d: -f1)
    [ -z "$cfg_test" ] && echo "NO_CFG_TEST $f"  # then read manually
    [ "$first" -lt "$cfg_test" ] && echo "MIXED $f"  # then read pre-boundary sites
  done

Files reporting MIXED need manual classification of each pre-boundary site against the three CLAUDE.md exemption cases (test / entrypoint / const-static-literal). Files reporting NO_CFG_TEST need manual read to determine whether matches are doc-comments or real prod sites.

HEAVY-tier pre-check step (new gate)

Context-compiler now MUST verify raw-vs-prod split BEFORE recommending tranche scope. Output format: "raw=N / prod=M / exempt=K / migration_target=L" where L = M - K. Tranches with migration_target = 0 are NULL-RESULT and ship as documentation only (no source edits). This rule applies retroactively to any B-15 follow-on tranches against plugins / providers / contracts / cli / dashboard-v2.

Recommended rules-file amendments

This methodology belongs alongside the other anti-spiral / anti-stale-training rules. Two candidate locations:
- .claude/rules/anti-stale-training.md -- amend to add "B-15 substrate audits (.unwrap / .expect / HashMap / SystemTime / entropy) MUST run cfg(test) boundary check BEFORE declaring scope. See workflow/reasoning/B15-methodology-decision.md."
- .claude/rules/anti-spiral.md -- amend to add "Four consecutive NULL-RESULT tranches against the same substrate is the canonical anti-spiral signal that the baseline methodology is wrong, not that the substrate is uniquely clean. Force methodology audit before opening a fifth tranche."

Cross-references
- workflow/reasoning/B15-E2-rules-unwrap-null-result.md (Tranche E, rules crate, 287 raw / 0 prod)
- workflow/reasoning/B15-H2-knowledge-entropy-null-result.md (Tranche H.2, knowledge entropy, 4 raw / 0 prod)
- workflow/reasoning/B15-I1-knowledge-unwrap-null-result.md (Tranche I.1, knowledge unwrap, 320 raw / 0 prod)
- workflow/reasoning/B15-I2-orchestrator-unwrap-null-result.md (Tranche I.2, orchestrator unwrap, 784 raw / 3 prod / 3 already-exempt)
- workflow/reasoning/B15-P1-plugins-unwrap-null-result.md (Tranche P, plugins, 258 raw / 41 prod / 41 already-exempt)
- workflow/reasoning/B15-P2-providers-unwrap-null-result.md (Tranche P, providers, 221 raw / 1 prod / 1 already-exempt)
- workflow/reasoning/B15-burndown-decommission-recommendation.md (Tranche P, decommissioning recommendation after 5 consecutive NULL-RESULT)

DI6 V2 -- line-comment filter

Tranche P discovered that the original DI6 Bash one-liner does NOT distinguish "describes the rule" from "violates the rule". Five of the 479 raw matches in plugins + providers were ban-text doc-comments (e.g. `//!   no .unwrap() outside #[cfg(test)])`, `// Check for .unwrap()/.expect() calls`, `/// snake_case label ... No .unwrap()`). The scanner treated each as a real call site, inflating the prod-classification workload.

Amendment: append `| grep -v "^\s*//"` to the raw-count and per-site line-extraction one-liners. The filter excludes any line whose first non-whitespace token is `//`, covering line-comments `//`, item-doc `///`, and module-doc `//!` line forms. Updated pre-filter one-liner:

  rg --count "\.unwrap\(\)" path/ | ...           # raw count, unchanged
  rg -n "\.unwrap\(\)" path/file.rs | grep -v ":\s*//"  # per-site, line-comment filter (NB: tab/space tolerance)

  for f in path/*.rs; do
    cfg_test=$(grep -n "^#\[cfg(test)\]" "$f" | head -1 | cut -d: -f1)
    first=$(grep -n "\.unwrap()" "$f" | grep -v ":\s*//" | head -1 | cut -d: -f1)
    [ -z "$cfg_test" ] && echo "NO_CFG_TEST $f"
    [ -n "$first" ] && [ -n "$cfg_test" ] && [ "$first" -lt "$cfg_test" ] && echo "MIXED $f"
  done

Block-comment false positives (`/* ... */`) remain a manual-read step. The plugins/providers pre-check did not encounter any block-comment false positives, consistent with prior tranches; block-comment .unwrap mentions appear to be vanishingly rare in this codebase.

Outcome

Going forward: zero HEAVY-tier B-15 tranches open without prod-side filter in context-compiler output. Expected throughput win: 4 tranches saved per substrate-class (rules/knowledge/orchestrator/plugins/providers/contracts/cli/dashboard) at ~30-60 min each.

Post-Tranche-P update: B-15 substrate-burndown plan is recommended for DECOMMISSIONING after 5 consecutive NULL-RESULT tranches across 6 crates / 1874 raw sites / 42 prod sites / 42 already-exempt. See workflow/reasoning/B15-burndown-decommission-recommendation.md. Future B-15 enforcement migrates from retroactive burndown to forward gate (auditor flags new violations at PR time).
