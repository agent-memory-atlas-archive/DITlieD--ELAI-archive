# DEEP-K-002 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1102-L1114`, heading `### DEEP-K-002`; the next chunk starts at L1116 (`### DEEP-K-003`).
- Declared chunk: family K, **7 files**, **1,303 logical lines**, **143,184 bytes**, classes `auxiliary/grounding/fixture` (2) + `numbered/index original` (5); target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all seven listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `11-K-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `KGM:Lx-Ly` (grounding map), `K-3:Lx-Ly`, `K-6:Lx-Ly`, `K-17:Lx-Ly`, `K-18:Lx-Ly`, `K-22:Lx-Ly`, and `KMB:Lx-Ly` (master brief).
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| auxiliary/grounding/fixture | 147 | 12,809 | 751afd17f314fa78bab4611187e8a3fe472e6cf32040f04f82273333bc1d003d | docs/thefinalglance/K-plan/_grounding/K-GROUNDING-MAP.md | exact |
| numbered/index original | 164 | 11,834 | f9c013b1d201c54d02f23f45ebbc38f74952133d063e50c91ae0f04171e58230 | docs/thefinalglance/K-plan/K-3_pr_ci_evidence_emitter.md | exact |
| numbered/index original | 156 | 11,039 | a4edbae783504430afc577a5e265e39dae30dcf8af1bc63424241c693dc7adbc | docs/thefinalglance/K-plan/K-6_compliance_export.md | exact |
| numbered/index original | 110 | 22,090 | d2061d2b11b4664e48f73583c5273553b1d171fa3a29b06a5586dd440b0e32fe | docs/thefinalglance/K-plan/K-17_context_management_measured_regime_aware_differentiator.md | exact |
| numbered/index original | 119 | 25,061 | ac4e1c618cbb6f9d8d2adcc8ddba67cfd4ca4b2611159bfd70b3af4544a2ca4d | docs/thefinalglance/K-plan/K-18_case_reuse_economics_compounding_state_differentiator.md | exact |
| numbered/index original | 124 | 25,398 | 3e74bc6363632ce1e9bef2372a5cdd8d46fe1765c35ce69e5e1cc8995b43942b | docs/thefinalglance/K-plan/K-22_doctrine_enforcement_coverage_ledger.md | exact |
| auxiliary/grounding/fixture | 483 | 34,953 | ff6d5bbc0eda55245d57441f2d734c64750ff49ef79298918a3d70ac14231f24 | docs/thefinalglance/K-plan/K-MASTER-BRIEF-2026-06-12.md | exact |
| **Aggregate** | **1,303** | **143,184** | — | **7 exact paths** | **exact** |

## Per-file atomic ledgers

### K-GROUNDING-MAP — live anchors, binding corrections, web verification

Source: `docs/thefinalglance/K-plan/_grounding/K-GROUNDING-MAP.md` (147/147 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| KGM-E001 | E/STATUS | Map authored 2026-06-12 as W-7 source for all K detail files; supersedes master-brief anchors where they go stale. | KGM:L1-L11 |
| KGM-E002 | E/DET | Canonical live baseline: `SCHEMA_VERSION=236`, `TOTAL_VARIANTS=545`; never hard-code; re-read `pub const SCHEMA_VERSION` and `TOTAL_VARIANTS` at edit (W-9). | KGM:L13-L22 |
| KGM-E003 | E/C | Master brief “234/543” is DEAD bait; every K SSB delta bumps sequentially from 236/545 one at a time with 9-tuple golden lockstep. | KGM:L20-L22, L84-L84 |
| KGM-E004 | E/CAP | KG-1: I-43 claim-scoper is SHIPPED with `ClaimScopeMeasured` (PilotDelta/CrossLaneDivergence/ClaimScoped), CLI `elai bench claim-scope [--suite] [--seed] [--json]`, emitter `bench_claim_scope::run_inner`, fixture contract. | KGM:L27-L33 |
| KGM-E005 | E/FLOW | Because KG-1 landed, A-1 is INVOKE-the-shipped-gate, not build-it (KC-1). | KGM:L33-L33, L67-L71 |
| KGM-E006 | E/CAP | KG-2: `github_pr` is a FULL WIRED module (review, body_composer, comment_watcher, adapter_trait, gh_cli, semgrep_gate, auto_merge_guard, package_verify, git_assist, clock, types, tests) plus CLI `pr_review_cmd`/`github_pr_cmd`, Tauri `commands_c40.rs`, frontend `github.svelte.ts`, two contracts. | KGM:L34-L40 |
| KGM-E007 | E/CAP | KG-3: review surface present via `review_surface.rs`, `review_cmd.rs`, G-5 contracts, H-52 anchor contract; `score_review_surface`/`review_anchor` names PLAN-asserted at function grain, re-grep at edit. | KGM:L41-L45 |
| KGM-E008 | E/CAP | KG-4: SAFEDELTA substrate present (`safedelta_runner.rs`, `bench_safedelta.rs`, g6 contracts). | KGM:L46-L48 |
| KGM-E009 | E/CAP | KG-5: QDELTA substrate present (`bench_qdelta.rs` + benchmarks registration); K-4 gate-tax mirrors this runner shape and shares corpus. | KGM:L49-L50 |
| KGM-E010 | E/CAP | KG-6: SeCodePLT substrate present; LIVE local-model run (W-10 RunPod) remains the missing leg. | KGM:L51-L53 |
| KGM-E011 | E/CAP | KG-7: RRA CLI present (`rra_cmd.rs`); `elai verify-attestation` referenced; precondition for A-4/K-6. | KGM:L54-L55 |
| KGM-E012 | E/DATA | KG-8: `wire-dark-allowlist.toml` has 105 entries; source for K-4 gate inventory and K-5 policy-table input. | KGM:L56-L58 |
| KGM-E013 | E/SURFACE | KG-9: bench sub-command dispatcher live at `args_bench.rs:75` with named subs list; K-2 and K-4 ADD subs here. | KGM:L59-L62 |
| KGM-E014 | E/STATUS | KC-1 accelerates K-T1: I-43 already exists so first measured number is harvested from a real run, not built. | KGM:L67-L71 |
| KGM-E015 | E/STATUS | KC-2: K-3 is a RENDERER over the wired github_pr adapter + G-5/H-52; re-implements neither. | KGM:L72-L74 |
| KGM-E016 | E/DEP | KC-3: A-8/J-28 is NOT shipped (no `UsageTurnEvent`); K-2 live cost-delta leg GATED on J-28; deterministic fixture leg is not. | KGM:L75-L78 |
| KGM-E017 | E/DEP | KC-4: A-4 (`elai wrap` + .rra) is PLANNED; K-3 does NOT wait on A-4 and renders own-run evidence first. | KGM:L80-L83 |
| KGM-E018 | E/EVIDENCE | KW-1 package-hallucination EXTERNAL figures: >=5.2% commercial / 21.7% open-source; 205,474 unique fakes; 58% recur across 10 / 43% recur in ALL 10; composition 38%/13%/51%. | KGM:L92-L98 |
| KGM-E019 | E/EVIDENCE | KW-2 patch-insecurity EXTERNAL: Llama-3.3-70B introduced 135 verified vulns vs 12 developer (=11x) over 20,000+ SWE-bench issues; agentic frameworks also introduce vulns; more autonomy = more vulns. | KGM:L99-L103 |
| KGM-E020 | E/EVIDENCE | KW-3 reward-hacking EXTERNAL: 73.8% Kernel-Bench and 46.8% ALE-Bench proxy-only gains; proxy-reality gap widens 26.4%->57.8% from 10 to 100 steps. | KGM:L104-L107 |
| KGM-E021 | E/EVIDENCE | KW-4 orchestration overhead EXTERNAL: CodeAct collapse cuts ~50% latency and >60% tokens; grounds K-4/K-5. | KGM:L108-L112 |
| KGM-E022 | E/EVIDENCE | KW-5 SpecBench EXTERNAL “+28pp per 10x code size” is PLAN-asserted at exact-number grain; dossier-build must quote precise table+page before use. | KGM:L113-L116 |
| KGM-E023 | E/EVIDENCE | KW-6 PENDING-VERIFY: DORA 2026, GitClear, METR, Uplevel, CodeRabbit figures stay `[EXTERNAL, unverified]` and do NOT enter plan files as asserted numbers until K-1 re-verifies. | KGM:L117-L122 |
| KGM-E024 | E/DEP | KS-1 master brief is authoritative for SCOPE; this map supersedes it on ANCHORS (KC-1..KC-5). | KGM:L127-L129 |
| KGM-E025 | E/DEP | KS-2 post-research scorecard + kill-switches; KS-3 Perplexity reports on-disk path unknown; KW verifications stand on primary sources. | KGM:L130-L134 |
| KGM-E026 | E/STATUS | KS-4 feature-map header stamp 234/543 is STALE per KC-5; re-grep at every read. | KGM:L135-L136 |
| KGM-E027 | E/STATUS | DC-K1..DC-K5 activation-vs-build corrections: A-1 ACTIVATE; K-3 BUILD renderer; K-2 live leg BLOCKED-ON-J-28; A-4 BUILD; K-4/K-5 BUILD over present substrate. | KGM:L141-L145 |
| KGM-E028 | E/LIMIT | Every external figure is tagged `[EXTERNAL]` and NEVER claimed as an ELAI measured number (KD-1). | KGM:L87-L91 |
| KGM-I001 | I/DEP | Detail-file authors that ignore KC corrections and re-assert brief anchors will fail W-7 validation twice (authoring + wiring-contract validator). | KGM:L5-L11, L65-L66 |
| KGM-A001 | A/DATA | Exact `score_review_surface` / `review_anchor` symbol names remain PLAN-asserted until executor re-grep. | KGM:L44-L45 |
| KGM-A002 | A/EVIDENCE | KW-5 exact SpecBench table value/page anchor unresolved until dossier-build. | KGM:L113-L116 |
| KGM-C001 | C/STATUS | Brief claimed I-43 PLANNED and github_pr possibly dark; map flips both to SHIPPED/WIRED (KC-1/KC-2). | KGM:L67-L74, L20-L22 |

Category coverage: capabilities/CLI ✓; status/corrections ✓; external evidence ✓; dependencies/gates ✓; no UI/privacy policy in this file; determinism/SSB baseline ✓; tests absent (map is anchor source); deferred live RunPod/J-28/A-4 ✓.

### K-3 — PR/CI evidence emitter

Source: `docs/thefinalglance/K-plan/K-3_pr_ci_evidence_emitter.md` (164/164 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| K3-E001 | E/STATUS | K-3 is PLANNED WAVE KW-2 BUILD-renderer over a WIRED adapter; KD-2/KD-5; single highest-leverage distribution leg; effort ~1.5 sessions (~6-7 h). | K-3:L1-L3, L16-L17 |
| K3-E002 | E/CAP | Builds a RENDERER that emits ELAI per-hunk evidence into GitHub check-runs + PR comments over the already-wired github_pr adapter. | K-3:L5-L14, L54-L61 |
| K3-E003 | E/DATA | Closed-set render source is the RRA evidence manifest (G-2); free-text/open-set evidence comments are banned. | K-3:L26-L29, L51-L51, L55-L57, L88-L88 |
| K3-E004 | E/DATA | Per-hunk evidence kinds: F-1/F-2/F-9 gate verdicts; F-4 stdout-hash-bound test outcomes; G-5 DSR + vouched/unvouched bands; H-52 anchored findings; G-8 attestation digest + offline-verify line; J-16 blast-radius cone when available. | K-3:L9-L12, L55-L57 |
| K3-E005 | E/SURFACE | One check-run summarizing gate verdicts + attestation digest + offline-verify line. | K-3:L58-L59, L95-L99 |
| K3-E006 | E/SURFACE | Per-hunk PR comments placed only via the H-52 finding-anchor resolver; hallucinated line numbers are structurally impossible. | K-3:L10-L11, L59-L61, L89-L89, L115-L115 |
| K3-E007 | E/FAIL | A hunk with NO evidence renders UNVOUCHED; never silently clean, never omitted (R1). | K-3:L12-L14, L29-L30, L60-L61, L87-L87, L113-L113 |
| K3-E008 | E/LIMIT | Hard boundary: renderer only; re-implements neither github_pr adapter nor any evidence source (GD-1). | K-3:L12-L14, L61-L61, L76-L77, L86-L86, L114-L114 |
| K3-E009 | E/SURFACE | CLI verb: `elai pr-review` evidence-emit mode; production trigger `elai pr-review <pr>` / fixture `elai pr-review <fixture-pr> --emit-evidence`. | K-3:L62-L63, L82-L84, L95-L96, L104-L104 |
| K3-E010 | E/EVENT | Prefer-reuse github_pr / review_surface / attestation SSB rows (net 0..+1); else +`PrEvidenceEmitted` with Kind CheckRunPosted / HunkVouched / HunkUnvouched. | K-3:L63-L64, L74-L74, L96-L97 |
| K3-E011 | E/FAIL | Negative arm: flag `--post-unvouched-as-clean` MUST be rejected/fail; closed-set render makes out-of-evidence “clean” structurally impossible (R2). | K-3:L65-L66, L99-L99, L106-L106, L131-L131 |
| K3-E012 | E/SURFACE | frontend_pair: surface emitted check-run status on `crates/dashboard-v2/src/lib/state/github.svelte.ts` (decide at edit, W-4); dashboard = DEEP view, PR = buyer view (KD-2). | K-3:L67-L67, L91-L93, L112-L112, L123-L124 |
| K3-E013 | E/INT | Production reachability: CLI evidence-emit mode -> github_pr adapter -> real GitHub check-run; `#[cfg(test)]`/contracts never satisfy R7. | K-3:L81-L84 |
| K3-E014 | E/DEP | Consumes github_pr adapter (KG-2/KC-2), review_surface G-5, H-52 resolver, F-4, G-8, J-16 when present, RRA manifest G-2. | K-3:L18-L21, L75-L80 |
| K3-E015 | E/DEP | A-4 foreign-agent evidence is optional later consumer; K-3 ships own-run evidence first and does not block on A-4. | K-3:L14-L14, L43-L43, L155-L156 |
| K3-E016 | E/DATA | Creates `evidence_emitter.rs` under github_pr/; modifies `pr_review_cmd.rs` (~40 LOC for `--emit-evidence`); optional SSB + github.svelte.ts; contract `k3_pr_evidence_emitter_contract.rs`; `workflow/reasoning/K-3-decisions.md`. | K-3:L134-L139 |
| K3-E017 | E/TEST | Contract arms: full-evidence check-run + vouched comments; no-evidence UNVOUCHED; H-52 placement; every claim has journal row; `--post-unvouched-as-clean` rejected. | K-3:L128-L131 |
| K3-E018 | E/TEST | Deterministic checks include wiring_contract_check, rg for emit-evidence mode and UNVOUCHED, broken-arm absence, cross_lang_diff on github_pr + github.svelte.ts. | K-3:L101-L109 |
| K3-E019 | E/LIMIT | No `.unwrap()`/`.expect()` on prod render path; no open-set free-text evidence; no hallucinated line numbers. | K-3:L86-L90 |
| K3-E020 | E/EVENT | SSB net 0..+1 sequential from live 236/545. | K-3:L117-L117 |
| K3-E021 | E/DEFER | GitHub-only v1 recommended; GitLab demand-gated later rider. | K-3:L151-L152 |
| K3-E022 | E/OBS | Expect artifact: real check-run whose every claim joins a journal row (GD-2); out-of-evidence comment structurally impossible. | K-3:L94-L99, L120-L120 |
| K3-E023 | E/CFG | W-2: emit mode is invoked, not default-OFF; IF gated, register in wire-dark-allowlist with promotion plan. | K-3:L121-L122 |
| K3-E024 | E/EVIDENCE | Market premise EXTERNAL: PR is standardized supervision (KS-3 16/18); deterministic/cryptographic/replayable PR evidence is unfilled slot. | K-3:L5-L7, L141-L145 |
| K3-E025 | E/STATUS | Parallel-safe with K-2/K-4/K-6/K-9; NOT parallel with edits to github_pr internals. | K-3:L22-L22 |
| K3-I001 | I/SURFACE | Tauri dashboard remains the deep operator surface while PR check-runs/comments are the meet-the-buyer surface under KD-2. | K-3:L30-L32, L91-L93 |
| K3-A001 | A/INT | Whether body_composer/comment_watcher seams are stable enough to render over without forking is unresolved until edit-time confirm. | K-3:L153-L154 |
| K3-A002 | A/EVENT | Whether to reuse existing rows or add `PrEvidenceEmitted` is decided at edit. | K-3:L63-L64, L74-L74 |
| K3-A003 | A/SURFACE | Exact check-run status fields on `github.svelte.ts` decided at edit (W-4). | K-3:L67-L67, L91-L93 |

Category coverage: capabilities/CLI/UI ✓; workflows ✓; schemas/data ✓; flags/negative flag ✓; integrations/events ✓; fail-closed evidence ✓; determinism/journal join ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/deferred ✓. No formal retry/idempotency key; privacy is deferred to G-9 composition for foreign evidence.

### K-6 — compliance export

Source: `docs/thefinalglance/K-plan/K-6_compliance_export.md` (156/156 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| K6-E001 | E/STATUS | K-6 is PLANNED WAVE KW-3 BUILD-composition; KD-5; no new attestation machinery; effort ~1 session (~4-5 h). | K-6:L1-L3, L16-L16 |
| K6-E002 | E/SURFACE | CLI verb: `elai export compliance [--out <dir\|archive>]`; production trigger `elai export compliance --out /tmp/elai-compliance-bundle`. | K-6:L6-L7, L54-L56, L79-L79, L90-L91 |
| K6-E003 | E/DATA | One bundle composing: redacted RRA (G-9 precondition) + G-8 attestation + audit-trail extract + I-44 retention statement + mapping doc. | K-6:L7-L12, L54-L56, L122-L125 |
| K6-E004 | E/SURFACE | Mapping document path: `docs/strategy/ELAI-COMPLIANCE-MAPPING.md` translating bundle fields onto procurement clauses. | K-6:L57-L61, L71-L71, L102-L102 |
| K6-E005 | E/COMP | Mapping covers: traceability req->code->build->release (I-18); audit-log requirements; OMB M-25-22-style independent-evaluation/reproducibility; EU AI Act Art-50 “designed to support” ONLY; PLD exposure; SOC2-adjacent control narratives. | K-6:L8-L12, L57-L61, L134-L138 |
| K6-E006 | E/LIMIT | Article-50-CERTIFIED claim is KILLED per GTM record; Art-50 wording is “designed to support” only. | K-6:L12-L14, L50-L51, L84-L84, L111-L111, L103-L103 |
| K6-E007 | E/LIMIT | NO org/SSO/RBAC multi-tenant build (non-goal). | K-6:L14-L14, L86-L86, L148-L149 |
| K6-E008 | E/LIMIT | NO new attestation/redaction machinery (composition only; GD-1). | K-6:L3-L3, L82-L82, L109-L109 |
| K6-E009 | E/PRIV | Secret-grep over the export must be EMPTY (G-9 composed). | K-6:L28-L29, L62-L63, L83-L83, L95-L96, L110-L110 |
| K6-E010 | E/TEST | Bundle verifies offline on a clean machine via `elai verify-attestation` + conformance checker. | K-6:L27-L28, L62-L63, L94-L96, L110-L110 |
| K6-E011 | E/FAIL | Negative: bundle missing attestation digest FAILS offline verify (R2). | K-6:L63-L63, K6:L96-L96, L124-L124 |
| K6-E012 | E/COMP | Every 2026 regulatory fact must be cited to E-38 posture doc `docs/strategy/ELAI-2026-POSTURE.md` or primary sources; never training memory. | K-6:L31-L32, L40-L41, L48-L49, L85-L85, L112-L112 |
| K6-E013 | E/EVENT | SSB: +`ComplianceExported` (Pattern-2; Kind BundleAssembled / OfflineVerified / ClauseMapped) OR reuse G-8 rows; sequential from 236/545. | K-6:L64-L65, L72-L72, L91-L93, L113-L113 |
| K6-E014 | E/INT | Production reachability is the real CLI export arm; test-only/contracts never satisfy R7. | K-6:L78-L80 |
| K6-E015 | E/DEP | Consumes rra_cmd, verify_attestation_cmd, G-9 redaction, I-44 zero-retention, I-18 chain, E-38 posture doc. | K-6:L17-L18, L34-L41, L73-L77 |
| K6-E016 | E/SURFACE | frontend_pair: N/A CLI+docs justified (W-6) OR optional compliance-export tile (decide at edit). | K-6:L87-L89, L117-L118 |
| K6-E017 | E/DATA | Files: `compliance_export_cmd.rs` + arm registration; mapping md; optional SSB; contract `k6_compliance_export_contract.rs`; `K-6-decisions.md` (bundle-format decision). | K-6:L127-L132 |
| K6-E018 | E/TEST | Contract arms: five-part assemble; offline verify; secret-grep empty; missing-digest fail; mapping has no “certified” and cites posture doc. | K-6:L121-L125 |
| K6-E019 | E/TEST | Deterministic checks ban “certified” strings in mapping and require E-38/E-38 posture citations. | K-6:L98-L106 |
| K6-E020 | E/DEP | Soft-depends K-1 numbers existing (bundle evidence references them). | K-6:L19-L20 |
| K6-E021 | E/STATUS | Parallel-safe with K-2/K-3/K-4/K-9. | K-6:L19-L19 |
| K6-E022 | E/EVIDENCE | Buyer is org-side AppSec/DevSecOps/GRC/platform/procurement (KS-3 report 16), not individual developer. | K-6:L5-L6, L23-L25 |
| K6-A001 | A/DATA | Bundle format single archive vs directory layout unresolved; recommend directory + optional archive flag after reading I-44/G-8 shapes. | K-6:L55-L56, L143-L145 |
| K6-A002 | A/COMP | E-38 posture 90-day staleness check: if stale at authoring, refresh before citing; EU/PLD dates must re-confirm. | K-6:L146-L147 |
| K6-A003 | A/SURFACE | Whether a compliance-export dashboard tile ships in v1 is edit-time choice. | K-6:L87-L89 |
| K6-I001 | I/COMP | Mapping doc is a procurement-language surface distinct from the machine-verifiable bundle itself. | K-6:L7-L12, L57-L61 |

Category coverage: CLI surface ✓; compliance/privacy ✓; composition/reuse ✓; offline verify ✓; negative arms ✓; no org-layer ✓; events ✓; UI optional ✓; deferred format/tile ✓.

### K-17 — measured regime-aware context-management differentiator

Source: `docs/thefinalglance/K-plan/K-17_context_management_measured_regime_aware_differentiator.md` (110/110 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| K17-E001 | E/STATUS | K-17 is PLANNED WAVE KW-3 PROVABLE+PACKAGE E-23-class; NO new bench; PACKAGES J-34; ZERO new SSB; effort ~0.5 session. | K-17:L1-L3, L7-L7 |
| K17-E002 | E/CAP | Banks differentiator: ELAI MEASURES per-(model,lane) context-degradation knee against gate verdicts and runs OPPOSITE cloud vs local context policies. | K-17:L4-L5, L12-L13, L30-L30 |
| K17-E003 | E/CFG | Cloud regime: cache-stable prefix + late compaction; local regime: window-minimization + unload-early. | K-17:L4-L5, L30-L30 |
| K17-E004 | E/CFG | Folklore window fractions (0.75 local / 0.85 cloud) and slot ORDER are replaced by a measured knee (after J-34). | K-17:L4-L5, L13-L13, L17-L17 |
| K17-E005 | E/DATA | K-1 dossier row tagged `[DESIGN]`/`[BENCH-PENDING]` with context_budget.rs + context_policy.rs code basis; no fabricated number (KD-1). | K-17:L30-L30, L38-L38, L60-L60, L72-L72 |
| K17-E006 | E/SURFACE | Claims-policy entry forces “best/optimal/never-loses context management” strings to carry measured-or-[DESIGN] qualifier + regime-split correction. | K-17:L30-L30, L32-L32 |
| K17-E007 | E/SURFACE | K-7 brief line for AppSec/platform “how do you keep the agent on-context across a long run”. | K-17:L30-L30, L92-L92 |
| K17-E008 | E/LIMIT | Does NOT author J-34 or IMP-2..IMP-8; does NOT build a second context bench; does NOT fork context_budget/context_policy; does NOT re-state CL-1..CL-4. | K-17:L5-L5, L49-L54, L76-L76 |
| K17-E009 | E/DEP | Number source is J-34 `elai bench context-fill` (BENCH-PENDING); measured lift waits on it. | K-17:L1-L1, L8-L8, L18-L18, L41-L41, L74-L74 |
| K17-E010 | E/STATUS | WIRED code basis: context_budget.rs 10 slots (8 + Memory H-31 + Evidence H-53) + floors/steal + 2 cache breakpoints + 3-level adaptive reduction. | K-17:L16-L16, L97-L97 |
| K17-E011 | E/STATUS | WIRED regime policy: context_policy.rs Local=floor(window*0.75) / Cloud=floor(window*0.85), `ELAI_CONTEXT_POLICY_DISABLE`, h50_context_policy_contract.rs; effective_context_window(Local) reads `ELAI_LOCAL_MAX_CONTEXT`. | K-17:L17-L17, L97-L97 |
| K17-E012 | E/DEP | Cites CL-1..CL-4 as binding discipline; ships NO new rule. | K-17:L20-L20, L31-L31, L43-L43 |
| K17-E013 | E/INT | EXTEND (not fork) K-1 claims_policy_lint.py recogniser for context-management claims and context-efficiency numbers without ELAI-measured journal pointer. | K-17:L32-L32, L39-L39, L44-L44, L90-L90 |
| K17-E014 | E/FAIL | Negative fixture fails: (a) “ELAI never loses context” untagged; (b) bare “0% degradation at full window” without measured pointer; (c) RULER/Chroma frontier knee as ELAI local-lane knee. | K-17:L33-L33, L66-L67, L86-L86 |
| K17-E015 | E/INT | Production reachability: auditor runs claims_policy_lint.py over positioning copy; dossier row points at J-34 knee once measured; test-only never satisfies R7. | K-17:L45-L48 |
| K17-E016 | E/EVENT | expect_ssb: none; measured number is [BENCH-PENDING] pointer to J-34 ContextCalibrationEvent. | K-17:L58-L59, L82-L82 |
| K17-E017 | E/SURFACE | frontend_pair: ContextStatePanel (DARK E-23 context telemetry) once mounted + H-9 cost cockpit (knee vs $/verified-task); else dossier/brief artifact; justified N/A (W-6). | K-17:L55-L56, L80-L80 |
| K17-E018 | E/LIMIT | KD-4: do NOT headline bare “we manage context/compact/cache” (provider-absorbed); headline measured knee + regime split + journal-recomputable receipt. | K-17:L12-L13, L83-L83, L102-L103 |
| K17-E019 | E/LIMIT | ID-5: any knee is ELAI-measured per-(model,lane), never RULER/Chroma curve. | K-17:L5-L5, L74-L74, L98-L98 |
| K17-E020 | E/TEST | Lint extension tests: tagged differentiator passes; three negative arms fail; empty input fails-loud (R1). | K-17:L85-L86 |
| K17-E021 | E/DATA | Files: dossier row; claims_policy_lint ~13 LOC; K-7 brief +1 line; `_fixtures/context_management_negative.md`; `K-17-decisions.md`. | K-17:L88-L93 |
| K17-E022 | E/DEFER | OUT-OF-K IMP-1..IMP-8 context-activation track (J-34 build, adaptive_reduce wire, BeliefState condense, pristine injector, blast-radius+retrieval, rotation pool, directive-supervisor, vLLM) recorded not authored. | K-17:L105-L105 |
| K17-E023 | E/DEP | Soft-depends K-1, K-7; sibling of K-15 (reasoning-budget PROVE+PACKAGE shape). | K-17:L8-L9, L21-L21 |
| K17-E024 | E/CFG | W-2 N/A: cited context_policy already carries `ELAI_CONTEXT_POLICY_DISABLE`. | K-17:L80-L80 |
| K17-I001 | I/OBS | Once J-34 lands, promotion path is [BENCH-PENDING] -> measured without a second claim machinery. | K-17:L47-L47, L60-L60 |
| K17-A001 | A/OBS | Exact measured knee magnitude/threshold remains unknown until J-34 runs. | K-17:L5-L5, L102-L102 |
| K17-A002 | A/INT | Lint recogniser overlap with K-15 must compose as one recogniser/multiple shapes, not forks. | K-17:L104-L104 |

Category coverage: differentiator/claims surfaces ✓; CLI bench owned by J-34 not K-17 ✓; UI panel reuse ✓; config flags cited ✓; fail-closed lint ✓; zero SSB ✓; OUT-OF-K deferred ✓; privacy N/A.

### K-18 — case-reuse economics + compounding-state differentiator

Source: `docs/thefinalglance/K-plan/K-18_case_reuse_economics_compounding_state_differentiator.md` (119/119 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| K18-E001 | E/STATUS | K-18 is PLANNED WAVE KW-3 PROVABLE+PACKAGE BUILD-instrument + dossier/brief; VCG J-track done-gate; +0..1 SSB prefer-reuse; effort ~1 session. | K-18:L1-L3, L7-L7 |
| K18-E002 | E/SURFACE | CLI verb: `elai bench case-reuse --suite default --seed N --json` as real production bench entrypoint. | K-18:L5-L5, L30-L30, L49-L49, L61-L61 |
| K18-E003 | E/CAP | Recurrence-hit-rate arm (SAFE, ships now): held-out corpus from F-4-proven case rows; measures how often NEW failing task matches existing trigger_signature over BM25+dual-vector+reranker+RRF; ELAI_SEED-recomputable. | K-18:L5-L5, L30-L30, L76-L76 |
| K18-E004 | E/CAP | Reuse-ON-vs-OFF economics arm (BENCH-PENDING on VCG J-track): ON injects top-matched F-4-PROVEN case as demonstration; OFF = cold; reports iteration-count delta + $/verified-task delta per (bug-class, model). | K-18:L5-L5, L31-L31, L76-L76 |
| K18-E005 | E/OBS | Y-axis is verified-OUTCOME receipts, never pass@k or self-report (GD-3); no LLM-judge scoring (TD-3). | K-18:L5-L5, L31-L31, L54-L54, L56-L56 |
| K18-E006 | E/FLOW | Observe-first: recurrence-hit-rate proves graph WOULD be useful with ZERO reuse-into-generation (GD-4 Slice-1). | K-18:L5-L5, L13-L13, L30-L30, L78-L78 |
| K18-E007 | E/DEP | Reuse arm OBEYS K-19 execution-grounded reuse-fitness (held-out pass); never “it was retrieved” or LLM-judged-helpful. | K-18:L5-L5, L31-L31, L55-L55, L78-L78, L112-L112 |
| K18-E008 | E/CFG | Reuse-into-generation path default-OFF behind wire-dark-allowlist row + promotion plan (W-2/GD-4). | K-18:L57-L57, L84-L84 |
| K18-E009 | E/DATA | Differentiator: “per-customer compounding state / gets faster-cheaper on YOUR recurring bugs over time” as K-1 `[DESIGN]`/`[BENCH-PENDING]` row + claims-policy entry + K-7 brief line. | K-18:L32-L32, L40-L40 |
| K18-E010 | E/SURFACE | Claims lint extension fails untagged “self-improving / learns from your codebase / gets better with use” and bare compounding numbers without journal pointer. | K-18:L33-L34, L41-L41, L97-L97 |
| K18-E011 | E/FAIL | Negative fixture: (a) untagged learns/improves; (b) bare “50% fewer iterations”; (c) self-graded LLM-helpful fitness design (K-19 catches (c)). | K-18:L34-L34, L71-L71, L91-L91 |
| K18-E012 | E/CAP | Produces `benchmarks/case_reuse_runner.rs` + `case-reuse` bench sub at args_bench.rs:75; composes I-27 ConvergenceCurveMeasured + safedelta runner shape. | K-18:L38-L39, L43-L44, L94-L95 |
| K18-E013 | E/EVENT | Prefer-reuse bench row family / ConvergenceCurveMeasured (+Kind CaseReuseArm / RecurrenceHit); net 0..+1 from LIVE ~267/577 (dated); decide at edit W-9. | K-18:L61-L62, L87-L87 |
| K18-E014 | E/SURFACE | frontend_pair: recurrence-hit-rate + iteration/$ delta on I-27 ConvergencePanel + H-9 CloudPanel; optional VCG canvas case-altitude when shipped; decide at edit W-4 / justified N/A. | K-18:L58-L59, L85-L85 |
| K18-E015 | E/INT | Production: real CLI bench + auditor lint; contracts never satisfy R7. | K-18:L48-L51 |
| K18-E016 | E/DEP | Case-birth substrate: F-4 red->green stdout_hash mint; G-2 RRA sealed payload; failure_taxonomy.py trigger_signature; B-29 blast_radius_cone. | K-18:L17-L17, L45-L45 |
| K18-E017 | E/DEP | Does NOT author VCG capability (mint/wire/canvas/reuse-loop); reuse-ON arm BENCH-PENDING on it. | K-18:L5-L5, L16-L16, L57-L57, L80-L80 |
| K18-E018 | E/LIMIT | No vendor/paper number as ELAI’s (ID-5); no fork of I-27/G-6/case-base/K-1 lint; no main-LLM self-run of lint. | K-18:L52-L56 |
| K18-E019 | E/TEST | Done = recurrence-hit-rate + FIXTURE arms actually run and produce recomputable numbers AND lint negative fails — not “runner compiles”. | K-18:L83-L83, L76-L76 |
| K18-E020 | E/TEST | Determinism test: recurrence-hit-rate byte-identical under fixed seed; lint tests include empty-input fail-loud. | K-18:L90-L91 |
| K18-E021 | E/DATA | Files: case_reuse_runner.rs; args_bench.rs; dossier row; claims_policy_lint ~14 LOC; K-7 line; compounding_state_negative.md; K-18-decisions.md; optional SSB. | K-18:L93-L101 |
| K18-E022 | E/DEFER | OUT-OF-K VCG J-track: mint-on-proof case node; deterministic self-wiring edges; MEMORY/CASE canvas altitude; gated retrieval reuse; episodic->skill->rule ladder. | K-18:L114-L114 |
| K18-E023 | E/EVIDENCE | Moat ranking premise: #2 per-customer compounding state has no PROVE leg in K until K-18; industry publishes no production number for agent improving on your codebase. | K-18:L5-L5, L12-L13, L105-L105 |
| K18-E024 | E/DEP | Soft-depends K-1/K-7; gated by K-19; sibling of K-15. | K-18:L8-L9 |
| K18-E025 | E/OBS | Live local-lane run is RunPod/remote-sidecar-gated (W-10). | K-18:L7-L7, L61-L61, L85-L85 |
| K18-I001 | I/STATE | FIXTURE tier shipping now is the only always-on measured arm until VCG enables reuse-ON. | K-18:L7-L7, L30-L31 |
| K18-A001 | A/EVENT | Exact prefer-reuse Kind vs new family decided at edit against live ~267/577 baseline. | K-18:L61-L62, L87-L87 |
| K18-A002 | A/SURFACE | Whether a dedicated case-reuse tile is required vs riding existing panels is edit-time. | K-18:L58-L59 |
| K18-A003 | A/INT | Lint recogniser overlaps K-19/K-15/K-17 and must compose, not fork. | K-18:L113-L113 |

Category coverage: CLI bench ✓; UI panels ✓; metrics/ablation ✓; observe-first gates ✓; fitness discipline ✓; events ✓; OUT-OF-K deferred ✓; privacy N/A; compliance N/A.

### K-22 — doctrine-enforcement-coverage ledger

Source: `docs/thefinalglance/K-plan/K-22_doctrine_enforcement_coverage_ledger.md` (124/124 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| K22-E001 | E/STATUS | K-22 is PLANNED WAVE KW-3 PROVABLE+DISCIPLINE audit+deterministic tool K-8-shaped; NO SSB; effort ~1 session (~4-5 h). | K-22:L1-L3, L7-L7 |
| K22-E002 | E/CAP | Build per-clause ENFORCEMENT-COVERAGE LEDGER tagging each doctrine clause {FROZEN-CHECKER / AUDITOR-ADVISORY / UNENFORCED} + coverage metric. | K-22:L4-L4, L31-L32 |
| K22-E003 | E/SURFACE | Tool: `.claude/tools/doctrine_coverage.py` stdlib-only deterministic `--json`; extracts clause IDs from CLAUDE.md + `.claude/rules/*.md`. | K-22:L7-L7, L32-L32, L41-L41, K22:L100-L100 |
| K22-E004 | E/DATA | Clause families: W/GD/HD/TD/RD/VD/RV/CL/ID/JD/KD/ND + per-file R-n/MR-n/JC-n/SF-n/RR-n/TZ-n. | K-22:L8-L8, L16-L16, L32-L32 |
| K22-E005 | E/FLOW | Join clause to enforcer via gate-manifest (R8) + `.claude/tools/` inventory; tagging is name+manifest join (TD-3); LLM does not grade enforcement. | K-22:L32-L32, L62-L62 |
| K22-E006 | E/STATE | FROZEN-CHECKER iff a manifest-frozen deterministic tool enforces it; AUDITOR-ADVISORY = auditor-applied no frozen check; UNENFORCED = prose only. | K-22:L4-L4, L17-L17, L32-L32 |
| K22-E007 | E/FAIL | UNKNOWN/unmappable clause is UNENFORCED, never silently FROZEN-CHECKER (R1/ND-3). | K-22:L32-L32, L59-L59, L85-L85 |
| K22-E008 | E/OBS | Emit coverage metric: % frozen-checker-enforced, % advisory, % unenforced; recomputable from rulebook+manifest (GD-2). | K-22:L32-L32, L67-L69, L84-L84 |
| K22-E009 | E/SURFACE | Ledger artifact: `docs/strategy/ELAI-DOCTRINE-ENFORCEMENT-COVERAGE.md` (or workflow/reasoning/) with per-clause table + coverage % + UNENFORCED compilation BACKLOG ranked by leverage. | K-22:L33-L33, L42-L42, L101-L101 |
| K22-E010 | E/SEC | Backlog is user-confirmed PROPOSAL; never auto-build checkers (no-removal/no-silent-build). | K-22:L4-L4, L33-L33, L60-L60, L86-L86, L117-L117 |
| K22-E011 | E/LIMIT | Does NOT modify rulebook clauses (read+tag only); does NOT introduce runtime gate over .elai_cc; does NOT claim every clause can be compiled (Rice/halting). | K-22:L4-L4, L61-L61, L88-L88 |
| K22-E012 | E/SURFACE | Doctrine rule: `.claude/rules/doctrine-enforcement.md` (~25-30 lines); any rulebook edit DECLARES clause enforcement tag. | K-22:L34-L34, L43-L43, L102-L102 |
| K22-E013 | E/INT | conditional-routing.md loads doctrine-enforcement.md on rulebook-editing tasks. | K-22:L34-L34, L53-L53, L78-L78, L103-L103 |
| K22-E014 | E/SURFACE | EXTEND claims_policy_lint so “fully enforced doctrine / every rule is checked / all deterministic gates” fails without measured coverage % + honest AUDITOR-ADVISORY/UNENFORCED split. | K-22:L35-L35, L44-L44, L104-L104 |
| K22-E015 | E/FAIL | Negative: (a) fixtures “every ELAI rule is a frozen deterministic gate” untagged fails lint; (b) clause asserting checker absent from manifest is tagged UNENFORCED and coverage reflects it. | K-22:L36-L36, L69-L69, L97-L97 |
| K22-E016 | E/INT | Production: auditor/harness runs doctrine_coverage.py; conditional-routing loads rule; lint over positioning copy; K-4 reads ledger for real vs advisory gates (KD-3). | K-22:L51-L55, L94-L94 |
| K22-E017 | E/EVENT | expect_ssb: none (harness leg; JD-1 = tool runs + recomputable ledger). | K-22:L63-L63, L68-L68 |
| K22-E018 | E/SURFACE | frontend_pair: N/A — tool + ledger doc + thin rule; no dashboard surface; coverage tile later optional (W-6 justified). | K-22:L64-L65, K22:L91-L92 |
| K22-E019 | E/DEP | Consumes gate_freeze_check.py + gate-manifest + checker inventory; composes J-70 DO-333 gate-side traceability; extends K-1 lint; mirrors K-8 audit+tool shape. | K-22:L17-L22, L45-L50 |
| K22-E020 | E/DEP | Checker inventory includes maintainability_check, production_caller_check, python_wire_check, traced_test_assert_check, wiring_contract_check, power_of_ten_check, claims_policy_lint, failure_taxonomy, cross_lang_diff, spec_link_check, elai_map_freshness. | K-22:L18-L18 |
| K22-E021 | E/TEST | Tool tests: matching tool->FROZEN; auditor-only->ADVISORY; prose->UNENFORCED; fake self-assert->UNENFORCED; coverage deterministic; empty input fails-loud. | K-22:L96-L97 |
| K22-E022 | E/TEST | W-1 done = tool RUNS and emits recomputable ledger + rule loads on live rulebook edit, not mere file existence. | K-22:L91-L91, L84-L88 |
| K22-E023 | E/DATA | Files: doctrine_coverage.py; ledger md; doctrine-enforcement.md; conditional-routing ~2 LOC; claims_policy_lint ~10 LOC; doctrine_coverage_negative.md; K-22-decisions.md. | K-22:L99-L106 |
| K22-E024 | E/LIMIT | FROZEN-CHECKER is necessary-not-sufficient (“tool CLAIMS this clause”); deeper proof is J-70 objective->technique->evidence; state this limit in ledger. | K-22:L116-L116 |
| K22-E025 | E/DET | Un-anchorable clause reported as UNANCHORED backlog row, never dropped silently (R1); stable clause id = file+clause-id or file+first-line-slug. | K-22:L115-L115 |
| K22-E026 | E/IDEM | K-22 must tag its own doctrine-enforcement.md honestly as AUDITOR-ADVISORY/harness-load in first ledger run (eat-your-own-dogfood). | K-22:L118-L118 |
| K22-E027 | E/EVIDENCE | EXTERNAL ContextCov ~50% prose-only is design rationale only; actual ELAI coverage % is measured by the tool (KD-1). | K-22:L4-L4, L111-L111 |
| K22-E028 | E/DEFER | OUT-OF-K: C9-PLANS feature-map mechanical half and J-70 gate-side matrix are separate J fills. | K-22:L119-L119 |
| K22-I001 | I/OBS | Coverage % becomes a KD-1-tagged measured-ELAI number only after the tool runs over the live rulebook. | K-22:L20-L20, L33-L33 |
| K22-A001 | A/DATA | Exact clause-extraction heuristics for heterogeneous HARD RULES vs numbered families remain implementer-specified beyond stable-id requirement. | K-22:L115-L115 |
| K22-A002 | A/DATA | Ledger path strategy vs workflow/reasoning alternative left open by “or”. | K-22:L33-L33, L76-L76 |

Category coverage: harness CLI tool ✓; ledger surface ✓; doctrine rule + routing ✓; claims lint surface ✓; fail-closed classification ✓; no product UI ✓; no SSB ✓; backlog authority ✓; self-tagging ✓.

### K-MASTER-BRIEF — strategy handoff for K-plan authoring

Source: `docs/thefinalglance/K-plan/K-MASTER-BRIEF-2026-06-12.md` (483/483 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| KMB-E001 | E/STATUS | Master brief dated 2026-06-12; status APPROVED-DIRECTION, folder not yet authored at write time; purpose is single comprehensive handoff for K-plan folder authoring. | KMB:L1-L7 |
| KMB-E002 | E/CAP | Strategy thesis (a)-(d): MEASURED trust number; PORTABLE open vendor-neutral evidence; CHEAP TO LIVE WITH composed friction budget; SOLD AS COMPLIANCE-READY EVIDENCE to org-side buyers. | KMB:L42-L59 |
| KMB-E003 | E/EVIDENCE | Unfilled market gap: vendor-neutral cryptographic chain of custody prompt->context->permissions->tool calls->diff->tests->deploy with independent attestation; labs ship containment+auditability not proof. | KMB:L76-L82 |
| KMB-E004 | E/COMP | Documented procurement demands: Tennessee RFI traceability/audit/third-party assessment/data-flow/retention/kill-switch; OMB M-25-22 independent evaluation + reproducible results; EU AI Act GPAI from 2026-08-02; Art-50; PLD from 2026-12-09; insurance AI sublimits. | KMB:L86-L94 |
| KMB-E005 | E/DEP | Moat ranking: 1 distribution, 2 per-customer compounding state, 3 proprietary verification, 4 enterprise trust/compliance, 5 cost engineering (weakest). ELAI strongest at 3, building 2, weakest at 1. | KMB:L109-L113 |
| KMB-E006 | E/EVIDENCE | Gate-tax EXTERNAL CodeAct ~-50% latency / >-60% tokens; adaptive gating beats gate-everything. | KMB:L114-L116 |
| KMB-E007 | E/DET | Replay honesty: cloud byte-replay fragile; honest guarantee is forensic reproducibility; local lane can claim more (J-13). | KMB:L117-L121 |
| KMB-E008 | E/LIMIT | Local autonomous floor ~24GB VRAM; ELAI 7B@12GB correct only as mechanical/privacy/latency lane (H-50). | KMB:L122-L127 |
| KMB-E009 | E/STATUS | Section A execution pointers (sequence, not re-author): A-1 I-43; A-2 F-2b live; A-3 G-6; A-4 I-40 wrap; A-5 G-5 review-time study; A-6 H-7/I-25; A-7 J-9; A-8 J-28/J-29. | KMB:L187-L203 |
| KMB-E010 | E/FLOW | Order: A-1 -> (go) -> A-2 -> A-3; A-4 parallel after G-9 live; A-5 after K-3 or cockpit-only if K-3 slips; A-6..A-8 parallel-safe before K-2/K-9. | KMB:L201-L203 |
| KMB-E011 | E/CAP | K-1 dossier + claims policy: no external claim without measured journal-recomputable number or EXTERNAL/DESIGN tag. | KMB:L208-L220 |
| KMB-E012 | E/CAP | K-2 `elai bench routing-economics` hybrid number; live cost leg needs A-8. | KMB:L222-L237 |
| KMB-E013 | E/CAP | K-3 PR/CI evidence emitter (distribution leg): check-runs + PR comments; UNVOUCHED no-evidence; closed-set from RRA; `--post-unvouched-as-clean` fails. | KMB:L239-L254 |
| KMB-E014 | E/CAP | K-4 composed gate-tax budget with locked ceiling; breach = release BLOCK; `elai bench gate-tax`. | KMB:L256-L270 |
| KMB-E015 | E/CAP | K-5 risk-tiered GateIntensity policy table R8-frozen; LLM never picks tier (TD-3). | KMB:L272-L287 |
| KMB-E016 | E/CAP | K-6 `elai export compliance` bundle + mapping; Art-50 designed-to-support only; no new attestation machinery. | KMB:L289-L302 |
| KMB-E017 | E/CAP | K-7 AppSec/insurance positioning brief; pure prose + K-1 numbers; no runtime/SSB. | KMB:L304-L310 |
| KMB-E018 | E/CAP | K-8 absorbable-unit audit + KD-4 provider pre-gate; kill list is user-confirmed proposal. | KMB:L312-L325 |
| KMB-E019 | E/CAP | K-9 quarterly model-refresh cadence; ADOPT/KEEP/RE-EVALUATE user-ratified. | KMB:L327-L341 |
| KMB-E020 | E/CAP | K-10 sequencing/cut doctrine: A-1..A-4+K-3 above everything; NO NEW PLAN LETTERS until those DONE; activation-over-authoring; canvas H-10..H-16 demoted behind K-3+A-5; I-31 last. | KMB:L343-L355 |
| KMB-E021 | E/CFG | Doctrine KD-1 number-before-claim; KD-2 meet-the-buyer-in-the-PR; KD-3 composed-friction ceiling; KD-4 provider-absorption pre-gate; KD-5 window-clock for wedge artifacts. | KMB:L365-L378 |
| KMB-E022 | E/LIMIT | Non-goals: no IDE play; no app-generator; no SWE-bench-Verified headline; no org/SSO/RBAC in K; no gate-everything defaults; no local expansion until 24GB candidate passes K-9; no new efficiency without KD-4; no overclaim of determinism tiers. | KMB:L381-L394 |
| KMB-E023 | E/RECOVERY | Kill-switches: null/negative A-1/A-3 safety delta -> stop K-T2/K-T4 headline trust; K-4 tax untolerable -> aggressive collapse; GitHub ships crypto evidence -> A-4 pivots compatibility, K-3 survives; A-5 no review-time win -> cockpit compliance artifact, K-3 remains; no 2x3090 -> K-9 still runs, 24GB adoption deferred. | KMB:L397-L410 |
| KMB-E024 | E/FLOW | Track map K-T1 PROVE; K-T2 WEDGE+DISTRIBUTION (A-4,K-3,A-5); K-T3 ADAPTIVE GATE; K-T4 BUYER-SIDE (K-6,K-7); K-T5 DEPRECIATION+REFRESH; K-T6 SEQUENCING. | KMB:L357-L363 |
| KMB-E025 | E/INT | Authoring produces K-0_INDEX, K-SUMMARY_INDEX, K-1..K-N, `_grounding/K-GROUNDING-MAP.md`; mirror J-plan format. | KMB:L420-L422 |
| KMB-E026 | E/DET | W-7 verify-at-authoring list: live SCHEMA/TV; github_pr real state; G-5/H-52; safedelta/seccodeplt/bench_cmd; I-43 code status; wire-dark allowlist; J-28 UsageTurnEvent. | KMB:L423-L435 |
| KMB-E027 | E/EVENT | SSB prefer-reuse before new variants; Pattern-2 + 9-tuple lockstep; bump from LIVE integers only. | KMB:L436-L438 |
| KMB-E028 | E/EVIDENCE | Every external number in K-1/K-7 re-verified via WebSearch at authoring (house rule after bad-cite incidents). | KMB:L439-L441 |
| KMB-E029 | E/LIMIT | Never amend old plan files (new files only); K-8 kill list is PROPOSAL; docs/ gitignored. | KMB:L445-L448 |
| KMB-E030 | E/A | Open questions: K-4 ceiling VALUE user call; K-3 GitHub-only vs GitLab; K-6 archive vs directory; A-5 study n/pilot framing; whether K-10 lives only in K-0 or also `.claude/rules/`; canvas demotion confirmation. | KMB:L455-L469 |
| KMB-E031 | E/STATUS | One-screen summary: prove, distribute, make affordable, package for buyer, install discipline; ~10 net-new sessions + 8 pointers + 5 doctrine + 5 kill-switches. | KMB:L472-L483 |
| KMB-E032 | E/EVIDENCE | Problem ammunition list (USENIX/SWE-bench/reward-hacking/SpecBench/DORA/METR/vendor telemetry) for K dossier with cite-verify duty. | KMB:L158-L176 |
| KMB-C001 | C/STATUS | Brief states I-43 “exists as PLANNED — confirm no code landed” and SCHEMA 234/543 bait; K-GROUNDING-MAP later flips I-43 SHIPPED and baseline 236/545 (KC-1/KC-5). Authors MUST prefer the map. | KMB:L426-L434 vs KGM-E002/KGM-E005 |
| KMB-C002 | C/STATUS | Brief treats github_pr as “VERIFY live… rename caused stale FAIL”; map confirms FULL WIRED (KC-2). | KMB:L249-L251, L428-L429 vs KGM-E006 |
| KMB-I001 | I/DEP | K-17/K-18/K-22 are later expansions beyond the brief’s original K-1..K-10 list; brief scope is the original wave framing, not the complete later K index. | KMB:L182-L185, L205-L355 |
| KMB-A001 | A/CFG | Locked composed-friction ceiling value is user-set after first measurement; brief forbids inventing one. | KMB:L457-L458 |

Category coverage: strategy/tracks ✓; Section A/B units ✓; KD doctrine ✓; non-goals/kill-switches ✓; authoring process ✓; open questions ✓; stale-vs-map conflicts explicit ✓; no runtime UI (handoff doc).

## Cross-source interactions and lifecycle

| IX ID | Class | Interaction | Source atoms |
|---|---|---|---|
| IX-001 | E | K-GROUNDING-MAP is the W-7 authority that corrects K-MASTER-BRIEF anchors (SSB baseline, I-43 status, github_pr status, J-28 gate, A-4 status). | KGM-E001–E003, E005–E006, E016–E017, E024, E027; KMB-C001–C002 |
| IX-002 | E | K-3 is the distribution renderer KC-2/KMB prescribe over KG-2 github_pr + KG-3 review/H-52 + KG-7 RRA/attestation. | K3-E002, E014; KGM-E006–E007, E011, E015; KMB-E013 |
| IX-003 | E | K-3 does not wait on A-4; foreign-agent evidence is a later compose when wrap lands (KC-4 shared by map and K-3). | K3-E015; KGM-E017 |
| IX-004 | E | K-6 offline-verify path is the same `elai verify-attestation` surface K-3 embeds as an offline-verify line in PR check-runs. | K3-E004–E005; K6-E010; KGM-E011 |
| IX-005 | E | K-6 G-9 secret-scrub and redacted RRA composition is the publication privacy leg complementary to K-3’s public PR render. | K6-E003, E009; K3-E003 |
| IX-006 | E | K-6 soft-depends K-1 measured numbers; K-17/K-18/K-22 all extend the same K-1 claims_policy_lint + dossier surface. | K6-E020; K17-E005–E006, E013; K18-E009–E010; K22-E014 |
| IX-007 | E | K-17 packages J-34 bench numbers; forbids a parallel context bench; points measured events at J-34 ContextCalibrationEvent rather than a K SSB. | K17-E008–E009, E016 |
| IX-008 | E | K-17 cites CL-1..CL-4 + WIRED context_budget/context_policy; K-18 cites F-4/G-2/B-29/I-27 substrate the same way (compose, never re-author). | K17-E010–E012; K18-E012, E016 |
| IX-009 | E | K-18 reuse arm is gated by K-19 fitness discipline and VCG J-track landing; observe-first recurrence-hit ships without generation influence. | K18-E003–E008, E017 |
| IX-010 | E | K-18 and master-brief moat #2 “per-customer compounding state” are the same open PROVE gap; K-18 is the measurement leg. | K18-E023; KMB-E005 |
| IX-011 | E | K-22 ledger feeds K-4’s view of which gates are REAL frozen-checkers vs advisory (KD-3), composing with gate-tax. | K22-E016; KMB-E014, E021 |
| IX-012 | E | K-22 mirrors K-8 audit+tool+proposal shape and composes J-70 gate-side DO-333 rather than forking it. | K22-E001, E019; KMB-E018 |
| IX-013 | E | KD-1/KD-2/KD-5 from the brief bind K-3 (PR render + window clock), K-6 (compliance time-box), and K-17/K-18/K-22 claim tags. | KMB-E021; K3-E001, E012; K6-E001; K17-E005; K18-E009; K22-E014 |
| IX-014 | E | External KW-1..KW-5 figures in the map are the verified ammunition the brief §2.4 and K-1/K-7 use; KW-6 remains pending. | KGM-E018–E023; KMB-E032, E028 |
| IX-015 | E | SSB discipline is shared: map sets 236/545 sequential lockstep; K-3/K-6 may +1; K-17/K-22 force 0; K-18 prefer-reuse 0..+1 from a later live baseline. | KGM-E002–E003; K3-E020; K6-E013; K17-E016; K18-E013; K22-E017 |
| IX-016 | E | User-facing buyer surfaces split: K-3 PR/GitHub check-run + dashboard deep view; K-6 offline compliance bundle + mapping doc; K-7 brief (out of this chunk’s numbered files but sequenced by brief). | K3-E005–E006, E012; K6-E002–E004; KMB-E016–E017 |
| IX-017 | E | Operator harness surfaces: K-17/K-18 auditor claims lint; K-18 `elai bench case-reuse`; K-22 `doctrine_coverage.py` + doctrine-enforcement rule load. | K17-E013–E015; K18-E002, E010; K22-E003, E012–E013 |
| IX-018 | I | If K-3 frontend_pair and K-6 optional compliance tile both land, both ride dashboard-v2 but remain distinct (GitHub state vs compliance export). | K3-E012; K6-E016 |
| IX-019 | E | Master-brief kill-switch “GitHub ships crypto evidence” pivots A-4 but keeps K-3 rendering; matches K-3’s renderer-only boundary. | KMB-E023; K3-E008 |
| IX-020 | E | Parallel-safety: K-3 // K-6 (and K-2/K-4/K-9) when files disjoint; K-17 // K-13..K-19; K-22 // K-8/K-13..K-21. | K3-E025; K6-E021; K17-E023; K22 source L9 |

## Explicit ambiguity and conflict register

| AC ID | Class | Issue | Atoms |
|---|---|---|---|
| AC-001 | C | Master brief SSB 234/543 and I-43 PLANNED vs grounding map 236/545 and I-43 SHIPPED. | KMB-C001; KGM-E002, E005 |
| AC-002 | C | Master brief github_pr verify-at-authoring caution vs map FULL WIRED. | KMB-C002; KGM-E006, E015 |
| AC-003 | A | K-3 `PrEvidenceEmitted` vs reuse existing rows. | K3-A002 |
| AC-004 | A | K-3 github.svelte.ts field shape and body_composer seam stability. | K3-A001, A003 |
| AC-005 | A | K-6 bundle directory vs archive; compliance tile yes/no; E-38 staleness. | K6-A001–A003 |
| AC-006 | A | K-17 measured knee magnitude pending J-34; lint compose with K-15. | K17-A001–A002 |
| AC-007 | A | K-18 SSB Kind choice; panel vs new tile; lint compose with K-19/K-15/K-17. | K18-A001–A003 |
| AC-008 | A | K-22 clause extraction heuristics; ledger path strategy vs workflow. | K22-A001–A002 |
| AC-009 | A | Map PLAN-asserted review_surface symbol names and KW-5 table page. | KGM-A001–A002 |
| AC-010 | A | Brief open questions: K-4 ceiling value; GitLab scope; K-10 rules-file placement; A-5 pilot n. | KMB-E030, A001 |
| AC-011 | C | Brief original K-1..K-10 scope vs later K-17/K-18/K-22 files in this chunk (later expansions). | KMB-I001 |

## Summary additions and corrections

Comparison target: `docs/thefinalglance/feature-discovery/11-K-SUMMARY_INDEX.discovery.md` only.

### Grounding-map deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-KGM-001 | **ADD** full KG-1..KG-9 live anchors (I-43 CLI, github_pr file inventory, review/H-52, SAFEDELTA/QDELTA/SeCodePLT/RRA, 105 allowlist rows, args_bench sub list). | 11-K:L942-L944, L980 | KGM-E004–E013 |
| SD-KGM-002 | **ADD/CORRECT** KC-1..KC-5 binding flips and DEAD 234/543 baseline. | 11-K:L41-L42, L944 | KGM-E002–E003, E014–E017, E027, C001 |
| SD-KGM-003 | **ADD** KW-1..KW-6 exact external figures, dual 58%/43% recurrence distinction, and PENDING-VERIFY exclusion for KW-6. | 11-K:L59-L70, L980 | KGM-E018–E023, E028 |
| SD-KGM-004 | **ADD** KS/DC design-doc corrections and map-over-brief anchor precedence. | 11-K:L942-L944 | KGM-E024–E027 |

### K-3 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-K3-001 | **ADD** exact `--emit-evidence` mode, production trigger string, and optional `PrEvidenceEmitted` Kind set. | 11-K:L105-L126 | K3-E009–E010, E022 |
| SD-K3-002 | **ADD** closed-set RRA-manifest reader task split (~120 LOC) + renderer (~180-240 LOC) + evidence_emitter.rs path. | 11-K:L118-L122 | K3-E002–E003, E016 |
| SD-K3-003 | **ADD** W-2 allowlist-if-gated clause, cross_lang_diff requirement, and 236/545 sequential net. | 11-K:L123-L126 | K3-E020, E023, E018 |
| SD-K3-004 | **ADD** GitLab demand-gate deferral, A-4 non-blocking, parallel-safety, and open seam-stability question. | 11-K:L105-L126 | K3-E015, E021, E025, A001 |
| SD-K3-005 | **QUALIFY** summary UI pair: source marks check-run status on github.svelte.ts as edit-time decision, not a fixed field contract. | 11-K:L124 | K3-E012, A003 |

### K-6 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-K6-001 | **ADD** exact CLI flags `--out`, offline-verify composition path, and ComplianceExported Kind triad. | 11-K:L180-L205 | K6-E002, E010, E013 |
| SD-K6-002 | **ADD** E-38 posture-doc citation HARD RULE, 90-day staleness risk, and GTM-killed certified wording checks in DETERMINISTIC_CHECK. | 11-K:L194, L112 | K6-E006, E012, E019, A002 |
| SD-K6-003 | **ADD** five-part contract arms including mapping no-certified test; soft K-1 dependency; parallel-safety. | 11-K:L180-L205 | K6-E017–E021 |
| SD-K6-004 | **EXPAND** format ambiguity (directory recommended + optional archive) beyond summary’s “one bundle”. | 11-K:L185 | K6-A001 |
| SD-K6-005 | **ADD** summary K-6.22 unredacted-seeded-secret arm is a summary elaboration; numbered source states secret-grep empty + missing-digest fail without that exact fixture wording. | 11-K:L205 | K6-E009, E011 |

### K-17 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-K17-001 | **ADD** exact folklore constants Local 0.75 / Cloud 0.85, `ELAI_CONTEXT_POLICY_DISABLE`, `ELAI_LOCAL_MAX_CONTEXT`, and 10-slot composition (Memory H-31 + Evidence H-53). | 11-K:L447-L452 | K17-E004, E010–E011 |
| SD-K17-002 | **ADD** zero-SSB / ContextCalibrationEvent pointer, W-2 N/A reason, and ContextStatePanel/H-9 frontend pair. | 11-K:L456-L457 | K17-E016–E017, E024 |
| SD-K17-003 | **ADD** full OUT-OF-K IMP-1..IMP-8 activation list and KD-4 anti-headline framing. | 11-K:L462 | K17-E018, E022 |
| SD-K17-004 | **ADD** lint compose-with-K-15 requirement and empty-input fail-loud test. | 11-K:L455-L461 | K17-E013, E020, A002 |
| SD-K17-005 | **ADD** dossier path `ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md` and fixture path `context_management_negative.md`. | 11-K:L444-L458 | K17-E021 |

### K-18 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-K18-001 | **ADD** exact CLI shape `--suite default --seed N --json`, retrieval stack BM25+dual-vector+reranker+RRF, and FIXTURE-tier-now sequencing. | 11-K:L468-L479 | K18-E002–E003, E006 |
| SD-K18-002 | **ADD** wire-dark default-OFF for reuse-into-generation, K-19 SF fitness bans, and dated LIVE ~267/577 prefer-reuse note. | 11-K:L480-L489 | K18-E007–E008, E013 |
| SD-K18-003 | **ADD** runner path `case_reuse_runner.rs`, ConvergencePanel/CloudPanel pair, and W-1 “must actually run” done definition. | 11-K:L485-L489 | K18-E012, E014, E019 |
| SD-K18-004 | **ADD** full OUT-OF-K VCG mint/wire/canvas/reuse/promotion companion scope. | 11-K:L493 | K18-E022 |
| SD-K18-005 | **ADD** lint/composes K-19/K-15/K-17 coordination and byte-identical seed determinism test. | 11-K:L490-L492 | K18-E010–E011, E020, A003 |

### K-22 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-K22-001 | **ADD** exact tool CLI `--json`, coverage triple percentages, UNANCHORED residual class, and self-tag of doctrine-enforcement.md as AUDITOR-ADVISORY. | 11-K:L585-L608 | K22-E003, E008, E025–E026 |
| SD-K22-002 | **ADD** necessary-not-sufficient FROZEN-CHECKER limit (J-70 deeper proof) and checker inventory enumeration. | 11-K:L589, L601 | K22-E020, E024 |
| SD-K22-003 | **ADD** K-4 consumer link, conditional-routing load, and W-1 JD-1 run-not-exist done gate. | 11-K:L602-L606 | K22-E013, E016, E022 |
| SD-K22-004 | **ADD** OUT-OF-K C9-PLANS / J-70 companion split and ContextCov EXTERNAL rationale caveat. | 11-K:L608, L822 | K22-E027–E028 |
| SD-K22-005 | **ADD** ledger path alternative workflow/reasoning and fixture `doctrine_coverage_negative.md`. | 11-K:L603 | K22-E009, E023, A002 |

### Master-brief deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-KMB-001 | **ADD** full strategy (a)-(d), procurement demands, absorption/moat ranking, kill-switches, non-goals, and KD-1..KD-5 text. | 11-K:L35-L45, L942-L943 | KMB-E002–E008, E021–E023 |
| SD-KMB-002 | **ADD** Section A pointer order/gating and original K-1..K-10 deliverable contracts including K-10 no-new-letters. | 11-K:L35, L942 | KMB-E009–E020, E024 |
| SD-KMB-003 | **ADD** authoring instructions 6.1-6.8, open questions §7, and one-screen summary. | 11-K:L943 | KMB-E025–E031 |
| SD-KMB-004 | **FLAG** stale brief anchors superseded by grounding map (I-43, SCHEMA, github_pr). | 11-K:L944 | KMB-C001–C002; SD-KGM-002 |
| SD-KMB-005 | **QUALIFY** brief as original-wave framing; K-17/K-18/K-22 are later letter expansions not specified in the brief’s K-1..K-10 list. | 11-K:L943 | KMB-I001 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **7/7 files**, **1,303/1,303 logical lines**, **143,184/143,184 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **210 unique atoms** = **184 explicit**, **007 inferred**, **016 ambiguous**, **003 conflicting/corrective**.
- Per-file atom counts: KGM **032**; K-3 **029**; K-6 **026**; K-17 **027**; K-18 **029**; K-22 **031**; KMB **036**. Sum **210**.
- Relational interaction records: **20** (`IX-001`–`IX-020`). Summary-delta records: **034** (`SD-*`). Ambiguity/conflict register: **11** (`AC-001`–`AC-011`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `KGM/K3/K6/K17/K18/K22/KMB-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them.
- User-facing surface audit: CLI verbs/flags (`elai pr-review --emit-evidence`, `elai export compliance`, `elai bench case-reuse`, `doctrine_coverage.py --json`, `elai verify-attestation` offline path), GitHub check-run + PR-comment surfaces, `github.svelte.ts` check-run status, compliance mapping doc, ContextStatePanel/H-9, ConvergencePanel/CloudPanel, doctrine ledger doc, and claims-lint negative fixtures each have dedicated atoms (not folded into backend-only bullets).
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Map-over-brief corrections remain conflict atoms, not silent rewrites of brief text.
- Manifest-boundary audit: the only original sources opened were the seven exact DEEP-K-002 manifest rows. The only orientation artifact consulted was `11-K-SUMMARY_INDEX.discovery.md`.
- Output metrics after finalization: `output_lines=0429`; `output_bytes=059361`.
- Canonical normalized output SHA-256: 26219e86037cfce284972e29cf61fe8253b26e328e4bf022cda5fe3350e356dc.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
