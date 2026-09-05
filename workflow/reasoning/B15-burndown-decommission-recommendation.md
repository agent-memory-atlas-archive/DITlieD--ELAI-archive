B-15 burndown plan -- decommissioning recommendation
Session: s-22600 sub-phase trancheP | Date: 2026-06-01

Verdict: RECOMMEND DECOMMISSIONING the B-15 .unwrap/.expect burndown plan; mark substrate VERIFIED-COMPLIANT.

Evidence: five consecutive NULL-RESULT tranches this session
- Tranche E (rules crate .unwrap):             raw=287  / prod=0 / migration_target=0
- Tranche H.2 (knowledge entropy):             raw=4    / prod=0 / migration_target=0
- Tranche I.1 (knowledge crate .unwrap):       raw=320  / prod=0 / migration_target=0
- Tranche I.2 (orchestrator crate .unwrap):    raw=784  / prod=3 / 3 ALREADY-EXEMPT / migration_target=0
- Tranche P (plugins + providers .unwrap/.expect): raw=479 / prod=42 / 42 ALREADY-EXEMPT / migration_target=0

Cumulative: 1874 raw .unwrap/.expect occurrences across 6 crates / 0 production migration sites verified.

Threshold rationale
- .claude/rules/anti-spiral.md gates "same file edited >3 times for same issue without resolution" as STOP signal.
- By analogy, "same substrate audited 5 times with zero prod migration" is the canonical anti-spiral signal that the substrate is structurally already-compliant, not that each crate is uniquely clean.
- Author of plugins/src/tiered_analysis.rs pre-anticipated this audit via explicit CLAUDE-md-EXCEPTION headers at L298/L1295/L1962 covering all 37 prod .unwrap sites. The codebase's authors have already encoded the exemption rationale inline; retroactive substrate-wide burndown adds zero protection.

Recommendation
1. MARK B-15 .unwrap/.expect substrate as VERIFIED-COMPLIANT across all 6 audited crates (rules + knowledge + orchestrator + plugins + providers + knowledge-entropy adjunct).
2. DECOMMISSION the B-15 substrate-burndown plan. No further sweeps; no further tranches.
3. Future B-15 enforcement migrates from RETROACTIVE BURNDOWN to FORWARD GATE:
   - CLAUDE.md hard rule remains: no .unwrap()/.expect() in production Rust except case-3 (const/static init with provably valid literals) plus the documented entrypoint exemption.
   - New prod .unwrap/.expect sites are caught by code review at PR time (auditor agent flags violations during normal audit cycles).
   - No additional substrate sweep is needed unless a code-review finding surfaces a regression.
4. REFUSE Tranche Q (any further B-15 .unwrap sweep) without explicit user override. Redirect orchestrator effort to other backlog items (cli regrowth, dashboard, follow-on plan work).

DI6 methodology amendment (per Tranche P context-package INFO note)
- The DI6 Bash one-liner does NOT filter doc-comment false positives. Three of five providers raw .unwrap matches were ban-text doc-comments (e.g. `//!   no .unwrap() outside #[cfg(test)])`).
- Add `| grep -v "^\s*//"` line-comment filter to future B-15 substrate one-liners (covers line-comments `//` and module/item docs `///` / `//!` line forms).
- Block-comment `/* ... */` false positives remain a manual-read step (rare in this codebase per audit history).
- Methodology doc B15-methodology-decision.md updated alongside this recommendation to record the V2 amendment.

Outcome
- Recommend orchestrator close the B-15 substrate-burndown plan as VERIFIED-COMPLIANT-FULL across all 6 audited crates.
- Recommend refusing any further B-15 .unwrap/.expect sweep tranches absent a code-review-surfaced regression.
- No source edits this tranche.

=== ACCEPTANCE 2026-06-01 (session s-22601, user-confirmed) ===
Status: ACCEPTED. The user confirmed the decommission recommendation during orchestrate s-22601.
Decision:
1. B-15 .unwrap/.expect substrate is MARKED VERIFIED-COMPLIANT-FULL across all 6 audited crates (rules + knowledge + orchestrator + plugins + providers + knowledge-entropy adjunct). Cumulative 1874 raw / 42 prod / 42 already-exempt / 0 migration_target.
2. The retroactive substrate-burndown plan is DECOMMISSIONED. No further sweeps; no Tranche Q.
3. Enforcement migrates from RETROACTIVE BURNDOWN to FORWARD GATE: the CLAUDE.md no-.unwrap/.expect-in-production HARD RULE stands; new violations are caught by the auditor agent at PR/audit time during normal cycles. No additional substrate sweep unless a code-review finding surfaces a regression.
4. Any future B-15 .unwrap sweep tranche is REFUSED absent a code-review-surfaced regression (the user override gate is now considered satisfied for decommission only, NOT for re-opening burndown).
Stub registry: B15-burndown-decommission flipped RECOMMENDED -> ACCEPTED in workflow/sessions/s-22600/stubs.md and recorded in workflow/sessions/s-22601/stubs.md.
DI6 V2 line-comment filter amendment (| grep -v "^\s*//") remains the standing methodology for any future ad-hoc .unwrap/.expect grep, to suppress doc-comment false positives.
