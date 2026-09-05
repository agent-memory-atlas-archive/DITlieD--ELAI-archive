# J-109: Learned-artifact admission hardening — immutable-surface span dedup + counterfactual replay fitness at the write seams

STATUS: PLANNED (WAVE STEAL-JUL04; S-M; EXTENSION(E-5/I-12/I-13) — one admission pipeline at the memory/rule write seams; from the AgenticSTS steal, steal-agenticsts-paw-abstention-2026-07-04.md S2+S3. Wargame-refit 2026-08-27 to flip every failing gate point (9/9 scored PASS; P8 stays auditor-owned). Design RATIFIED via K-27 seam proposal — carve restored in text, not redesigned: leg A (StaticSpanIndex + AD-2 span-dedup, AD-3 dup/merge band) at the TWO verified production write seams, observe-only default-OFF; leg B (AD-4 counterfactual replay) DEFERRED-GATED on two absent substrates. Baseline gate reproduced this refit at HEAD cc8ac315: FAIL P1 P3 P4 P5 P6 P7 P10 / P2 vacuous-pass / P9 pass. Refit work-product + grounding probes archived in conductor/opus_walk/evidence/J-109/refit-notes.md; slice basis conductor/opus_walk/evidence/J-109/J-109-seam-proposal.md §7, defer gates §8.)

Objective: every learned artifact ELAI admits through a write seam (a memory fact E-5, an intake rule I-12) passes its existing legs — but NO leg checks the candidate against the IMMUTABLE instruction surface ("don't re-learn what the rules already say") and no leg bands new candidates against live store entries at admission time. AgenticSTS ships both mechanisms with named thresholds and logged verdicts (write_gate.py:43-52/323/627; prewrite_ab.py:19/244, Apache-2.0 tree); J-109 ports the MECHANISMS — not code — as one AdmissionPipeline in front of the existing write seams: leg A = StaticSpanIndex over the J-102 admission-manifest surface AS ENUMERATED TODAY (.claude/rules+agents *.md, .claude/skills SKILL.md, AND cwd CLAUDE.md + AGENTS.md seeded at core/config.rs:316 since the manifest correction), two-threshold duplicate/Merge/Coexist band keyed on trigger context; BOUND by K-19 SF-1..SF-4 (execution-grounded fitness only — binds deferred leg B), H-31 auditor firewall (nothing admitted here ever reaches auditor/critic context), TD-3 (any LLM judge is ADVISORY; the decision fn here is fully deterministic), ID-5 (AgenticSTS thresholds are STARTING values ELAI recalibrates from observe-only rows). Default-OFF, observe-only first (GD-4): verdicts journaled, writes NEVER blocked in this walk.
GD-1 do-NOT-re-implement: J-102's `admission_scan()` enumeration (the index is BUILT FROM its rows — never a second path list); E-5 Mem0 `dispatch_one` ADD/supersede/noop decision arms (AD-1 is DELEGATION, not a second exact-match arm); I-13 write-time injection scan (ordered sibling: injection scan -> trust tier -> J-109 legs); the dual-ONNX embed stack + EmbeddingCircuitBreaker (consume ort_session embed); the SSB writer/golden/TOTAL_VARIANTS lockstep infra; ALL leg-B machinery (a byte-exact originating-prompt journal and a re-decide executor DO NOT EXIST — building them reverses a recorded CTIM-Rover security omission and needs its own owner-gated unit).

Estimated effort: ~1 session across three one-worker-one-turn units (UNIT A1 ~360 LOC carrying module + SSB arm + wiring + tests; A2 ~75; A3 ~200). Pure decision fn + observe-only wiring; zero inference dependency; embedder/breaker reachability touches only UNIT A3's labels, never the build or the other units.
Pre-requisites (re-grep per W-7): E-5 Mem0 dispatcher + capture flow [WIRED]; I-13 defense scan [WIRED]; I-12 rule lifecycle + CLI intake [WIRED]; J-102 config-ingress admission manifest [WIRED, observe-only]; H-43 FitnessEvaluated family [WIRED, darwin-gated] — future leg-B row family, untouched here; dual-ONNX embedders + circuit breakers [WIRED behind ELAI_CODE_VEC]; J-96 CaseNodeMinted + J-97 reuse gate [ABSENT — verified ZERO symbols at cc8ac315; drives the J-97-leg deferral]; SSB byte-exact prompt records [ABSENT BY DESIGN — CTIM-Rover omission]; re-decide executor over PerRoleRuntime [ABSENT — accessors only].
Parallel-safe with: J-110 (disjoint scope: eval/bench supply vs memory/self-improvement admission).
Source: workflow/papers/agenticsts-summary.md + workflow/reasoning/steal-agenticsts-paw-abstention-2026-07-04.md S2/S3; arXiv:2607.02255 verified live 2026-07-04. Doctrine: J-0_INDEX.md. Seam basis: conductor/opus_walk/evidence/J-109/J-109-seam-proposal.md (K-27 ratification; slices + defer gates are THIS plan's UNITS and deferred-work section).

## Context
The convergence theorem (H-Track-7) says a self-improvement loop without a real, un-gameable, stronger-than-generator filter collapses. ELAI's spec answers this at PROMOTION time (H-43 execution-grounded fitness, K-19 SF-1..SF-4), but the ADMISSION seams rely on dedup-vs-existing + injection scans, so a failure class slips through: RE-LEARNED RULES — an agent "discovers" a lesson that is verbatim doctrine already sitting in CLAUDE.md / AGENTS.md / .claude rules / agent files (since config.rs:316 BOTH cwd CLAUDE.md and AGENTS.md are J-102 manifest members today), bloating stores whose retrieval displaces genuinely new signal (AgenticSTS's StaticSpanIndex exists precisely because their agent kept re-learning its own system prompt). Near-duplicate crowding WITHIN stores is the second class; the dup/merge/coexist band handles it deterministically. Leg A attacks both classes with one pure function. Leg B ("did the candidate actually FLIP the originating failure on replay?") needs substrates that do not exist on this tree (byte-exact prompt persistence + a re-decide executor): it stays OUT of this walk with its opening gate named below — never silently skipped, never faked.
THE PIPELINE (one deterministic decision function, every decision journaled):
- AD-1 exact/near-exact vs an existing entry -> UPDATE: ALREADY OWNED upstream by the E-5 dispatch_one decision arms (ADD / supersede / noop). J-109 delegates; building a second exact-match arm would violate compose-never-duplicate. Permanent descope recorded in FORKS.
- AD-2 immutable-span dedup: candidate overlap >= threshold vs StaticSpanIndex spans built FROM elai_core::ConfigLoader::admission_scan() rows (the J-102 manifest; index fingerprint = blake3 over the rows' content_hash values; rebuild-if-stale checked before EVERY verdict batch) -> RejectImmutableSpan — the candidate is doctrine, not a lesson.
- AD-3 existing-store band: >= T_DUP -> RejectDuplicate; inside [T_IMMUTABLE, T_DUP) split on trigger-Jaccard >= T_CTX -> Merge (same trigger context) else Coexist (distinct variant).
- AD-4 counterfactual replay (originating failure + candidate injected, N re-decides, off-agent aggregate zero-harmful + supermajority, execution-grounded flip where a runnable gate exists, judge ADVISORY-only per TD-3): DEFERRED-GATED — see the deferred-work section for the two opening gates.
- Degrade honesty carried from the steal: embedder-unavailable/breaker-OPEN degrades AD-2/AD-3 to LEXICAL with the verdict row labeled with which leg ran — NEVER blind-accept on infrastructure failure.

## Existing state (coverage)
- E-5 USER MEMORY SUBSTRATE [WIRED]: crates/cli/src/user_memory_capture.rs STEP-3 `dispatch_one` (:256) inside `capture_remember_with_clock` (:170); production callers main.rs:1451 (`/remember` CLI arm; brief-cited :1419 — drift recorded, same single call) and repl_command.rs:146 (REPL SlashCommand::Remember); default-OFF behind ELAI_USER_MEMORY_ENABLE. THE seam UNIT A1 fronts.
- I-13 MEMORY GUARD [WIRED]: MINJA classify(&candidate) runs earlier in the same capture flow (~:190 region, re-read at edit) plus MemoryWriteScanner CLI chokepoints — ORDERED BEFORE the pipeline (injection scan -> trust tier -> J-109 AD-2..AD-3); compose, never duplicate.
- I-12 EVIDENCE-GATED PLAYBOOK CURATOR [WIRED]: promotion tick rule_curator.rs:180 run_promotion_tick (spawned via knowledge_stack.rs spawn_playbook_curator :918 -> :108 startup chain) PROMOTES; candidate INTAKE where a rule FIRST enters PROPOSED = crates/cli/src/rule_cmd.rs:52 insert_user_rule (definition crates/rules/src/lifecycle.rs:356), already running scan_instruction_surface (:370) on rule text. THE seam UNIT A2 fronts (intake, NOT the promotion tick).
- J-96 CASE GRAPH + J-97 REUSE GATE [ABSENT]: zero CaseNodeMinted/case_graph symbols at cc8ac315 (re-probed; bench_case_reuse.rs is a retrieval benchmark, not a case graph). Their admission seam cannot be fronted; the J-97 leg is deferred with its opening gate named below.
- H-43 EXECUTION-GROUNDED FITNESS [WIRED, darwin-gated]: FitnessEvaluated family (payload struct payloads.rs:19574; kind enums ExecutionPassRate/DualAgreement/FitnessRegression) — the verdict family deferred leg B will reuse; untouched in this walk.
- J-102 CONFIG-INGRESS ADMISSION MANIFEST [WIRED, OBSERVE-ONLY]: crates/core/src/config.rs:315 `pub fn admission_scan` seeds cwd CLAUDE.md + AGENTS.md (:316) then walks .claude/{rules,agents} *.md + .claude/skills SKILL.md (collect_admission_paths :334-349); rows {path, blake3 content_hash, scan verdict} land in RuntimeConfig.admission_findings with NO production consumer yet — the ONE index source for AD-2 (GD-1).
- 1.1 EMBED SUBSTRATE [WIRED, flag-gated]: knowledge crate DualVector + ort_session.rs:138 `pub fn embed` + EmbeddingCircuitBreaker (circuit_breaker.rs:98, independent per-model instances), reachable via the knowledge_stack attach path behind ELAI_CODE_VEC — consumed by UNIT A3's cosine leg WITH labeled lexical degrade.
- SSB INFRA [WIRED]: MemoryGovernanceEventPayload (struct payloads.rs:19159, deny_unknown_fields :19158) and FitnessEvaluatedPayload (:19574) are fixed-field records under the lockstep golden/TOTAL_VARIANTS convention — a receipts-bearing admission verdict structurally CANNOT reuse them; net +1 NEW payload + action-type arm pinned below (CLOSED fork; W-9 lockstep).

## Reads (mandatory before coding)
1. workflow/papers/agenticsts-summary.md (mechanisms + thresholds at write_gate.py:43-52/323/627, prewrite_ab.py:19/244) + workflow/reasoning/steal-agenticsts-paw-abstention-2026-07-04.md S2/S3.
2. crates/memory mem0_dispatcher.rs (dispatch_one decision arms — the AD-1 delegation target) + crates/cli/src/user_memory_capture.rs (capture flow :170 / STEP-3 :256 / emit helper :527).
3. crates/cli/src/rule_cmd.rs + crates/rules/src/lifecycle.rs:356/:370 (intake surface composition the pipeline reads) + rule_curator.rs:180 (to confirm you are NOT touching promotion).
4. crates/core/src/config.rs:315-349 (admission_scan + collect_admission_paths: the enumeration the index MUST reuse, including the :316 CLAUDE/AGENTS seeding) + crates/core/src/admission.rs (content_hash / scan_instruction_surface helpers).
5. crates/orchestrator/src/ssb/payloads.rs + ssb/action_type.rs around the MemoryGovernanceEvent block (struct :19158-19159, variant/action_type.rs:3923, stringify :5764, route-from-string :6657) + the golden/TOTAL_VARIANTS lockstep files (RE-READ baselines at edit, W-9).
6. .claude/tools/wire-dark-allowlist.toml ([[flag]] row shape precedent) + CLAUDE.md doctrine: TD-3 (LLM advisory only, never the gate), GD-4 observe-first, R8 frozen named thresholds, W-9 schema lockstep, no .unwrap in prod.
7. conductor/opus_walk/evidence/J-109/J-109-seam-proposal.md §7 (slice plan mirrored by the UNITS) + §8 (deferred-leg gates, binding carve-out) — probed evidence distilled in refit-notes.md next to it.

## GROUNDING (verified seams — recon complete, re-probed 2026-08-27)
RECON_NEEDED: none open — every load-bearing seam below probed directly at HEAD cc8ac315 (/home/user/teikoku/elai main tree; commands + raw outputs archived in conductor/opus_walk/evidence/J-109/refit-notes.md). Execute against these; do not re-plan them.
- PRODUCTION SEAM 1 (E-5 front, UNIT A1): `.elai_cc/crates/cli/src/user_memory_capture.rs:256` `let op = dispatch_one(&mut facts, &candidate, &ctx)?;` inside `pub(crate) fn capture_remember_with_clock` (:170); SSB emit helper same file :527 (`fn emit_ssb_row(...)`).
- LIVE CALLER 1a of 2: `.elai_cc/crates/cli/src/main.rs:1451` `user_memory_capture::capture_remember_with_clock(...)` — the `/remember` CLI arm, the ONLY non-test hit in main.rs. DESIGN-BRIEF line was :1419; HEAD probe pins :1451. Claim PRESERVED, drift recorded (W-7/W-9 handling precedent).
- LIVE CALLER 1b of 2: `.elai_cc/crates/cli/src/repl_command.rs:146` `match crate::user_memory_capture::capture_remember_with_clock(` (REPL remember arm). Line EXACT.
- PRODUCTION SEAM 2 (I-12 intake front, UNIT A2): `.elai_cc/crates/cli/src/rule_cmd.rs:52` `insert_user_rule(&conn, &UserRuleSpec { ... }, &OsClock)?;`; definition lifecycle.rs:356; dispatched via CliAction::RuleAdd (re-grep the dispatch line at edit).
- MANIFEST SOURCE (GD-1): `.elai_cc/crates/core/src/config.rs:315` `pub fn admission_scan(&self) -> Vec<AdmissionScanRow>`; :316 VERBATIM `let mut paths = vec![self.cwd.join("CLAUDE.md"), self.cwd.join("AGENTS.md")];` then rules/agents/skills collect calls (:317-:319); collect_admission_paths :334-349 filters .md extension (rules/agents) and SKILL.md filenames (skills). THE CORRECTION carried into design: cwd CLAUDE.md + AGENTS.md ARE manifest members TODAY, so UNIT A1 indexes them BY CONSTRUCTION (build-from-scan-rows) and MUST prove it with a dedicated contract arm.
- GREENFIELD: rg AdmissionPipeline | admission_pipeline | StaticSpanIndex | ADMISSION_HARDENING under crates/**/*.rs -> ZERO hits (2026-08-27); CaseNodeMinted/case_graph still ZERO. All J-109 symbols genuinely new; consequence accepted: the new pub symbols WILL trip production_caller_check unless wired at the pinned sites.
- SSB ANCHORS (drift-recorded, re-read at edit per W-9): MemoryGovernanceEventPayload payloads.rs:19159 + deny_unknown_fields :19158 (proposal-cited :19105 — DRIFTED, same struct); FitnessEvaluatedPayload payloads.rs:19574 (proposal-family cite :19459 — drifted); action-type pattern precedent MemoryGovernanceEvent variant :3923, stringify :5764 ("memory_governance_event"), route-from-string :6657.
- PATH NOTE: Rust workspaces live under `.elai_cc/`; bare `crates/...` paths throughout match prior plan convention and resolve under `.elai_cc/crates/...`.

## VERIFY (off-agent authority — the author NEVER self-certifies DONE)
Off-agent pass artifacts required; author-run outputs are iteration feedback, never authority:
```
python3 .claude/tools/plan_gate_check.py --file docs/thefinalglance/J-plan/J-109_learned_artifact_admission_hardening.md
#   off-agent verifier reruns this; PASS tail appended to conductor/opus_walk/evidence/J-109/refit-notes.md
python3 .claude/tools/production_caller_check.py --files crates/orchestrator/src/admission_pipeline.rs
#   EXECUTE-time auditor rerun per R-8; any DARK symbol = BLOCKED return
python .claude/tools/python_wire_check.py --files <changed .py>   # n/a unless a .py enters the changeset
cargo check -p orchestrator -p cli   # plain cargo depot shim ONLY (A5 build law); opus_land gate owns the authoritative GREEN
git diff --stat
```
- VERIFY artifact 1 (this refit): frozen plan_gate_check PASS tail stored by the OFF-AGENT verifier / opus controller in conductor/opus_walk/evidence/J-109/refit-notes.md. This author ran the checker only to iterate the text.
- VERIFY artifact 2 (EXECUTE waves): auditor-run production_caller_check output over crates/orchestrator/src/admission_pipeline.rs proving all new pub symbols wired (DARK-free); python_wire_check only if any .py enters the changeset (none planned).
- VERIFY artifact 3 (observe-mode proofs): per-unit smoke artifacts captured OFF-AGENT — admission_verdict SSB rows from REAL entrypoint drives (capture flow + rule add), each paired with proof the write still committed; plus the negative-arm transcript.

## REDTEAM (recorded attacks; auditors replay them)
- RT-1 falsify-coverage attack: attempted to keep AD-2 coverage wording at "rules-as-data ACTIVE rows, operator prompts, CLAUDE.md-class config" (original plan text). Result: the manifest enumerates ONLY the .md surfaces + cwd CLAUDE.md/AGENTS.md today (config.rs:315-349 re-probed) -> coverage statement narrowed to manifest-as-enumerated-today; widening waits on a named J-102-extension gate, never asserted by side effect. INCLUDED correction: the :316 seeding makes the two root files first-class index members, proven by a contract arm rather than assumed.
- RT-2 falsify-intake-site attack: claimed "I-12 curator intake" could mean the promotion tick (rule_curator.rs:180 run_promotion_tick, spawned from knowledge_stack). Rejected — that tick is PROMOTION (the k-counter domain); admission must precede intake. Seam pinned to rule_cmd.rs:52 insert_user_rule where a rule FIRST enters PROPOSED. Additional check: calling the pipeline from inside lifecycle.rs would also be dependency-impossible (RT-3).
- RT-3 falsify-placement attack: put check_candidate beside insert_user_rule inside crates/rules/lifecycle.rs. Broken dependency direction: orchestrator depends ON rules (orchestrator/Cargo.toml pulls elai-rules), so rules cannot import the pipeline. Resolved: pipeline lives in orchestrator; called from the two CLI call sites (cli depends on orchestrator + knowledge).
- RT-4 falsify-row-shape attack: force candidate-hash/leg/threshold receipts into MemoryGovernanceEventPayload or FitnessEvaluatedPayload. Both are fixed-field deny_unknown_fields records (payloads.rs:19158-19159 / :19574 probed) — impossible without schema abuse. Resolved as net +1 NEW AdmissionVerdict arm in golden/TOTAL_VARIANTS lockstep.
- RT-5 falsify-greenfield attack: hunted pre-existing AdmissionPipeline/StaticSpanIndex/ADMISSION_HARDENING symbols anywhere under crates/*.rs (a hit would silently convert UNIT A1 into a collision refactor). Zero hits at cc8ac315 -> greenfield HOLDS.
- RT-6 observe-bypass attack: pressure toward blocking writes early ("just reject while you're there"). Blocked by design: flag default-OFF, observe-only unit-wide (GD-4); the negative arm REQUIRES writes to proceed even when RejectImmutableSpan fires; block-mode is a separate orchestrator-gated phase after FP bounding on >=3 real runs.
- FORMAT redteam of this refit itself: baseline gate FAIL reproduced BEFORE any edit (P1 P3 P4 P5 P6 P7 P10; P2 vacuous, P9 pass) matching the queued claim exactly; the checker was treated as frozen throughout (never edited; R-3/A4).

## FORKS
- FORKS: none — design RATIFIED through the K-27 seam proposal; resolutions locked here so no defensible alternative remains open for an executor to re-litigate:
  1. Former open fork "which I-12 seam?" — CLOSED by RT-2: rule_cmd.rs:52 intake, never the promotion tick.
  2. Former open fork "reuse MemoryGovernanceEvent/FitnessEvaluated (+Kind) vs new row?" — CLOSED by RT-4: net +1 NEW AdmissionVerdict payload + action-type arm, golden lockstep (reuse-first failed structurally on deny_unknown_fields).
  3. Former open fork "pipeline residency?" — CLOSED by RT-3: orchestrator-resident module, CLI call-site fronts.
  4. Former open fork "how wide is AD-2 coverage?" — CLOSED by RT-1: manifest-as-enumerated-today INCLUDING cwd CLAUDE.md/AGENTS.md; expansion deferred with a named gate.
  5. Former open fork "second exact-match UPDATE arm for AD-1?" — CLOSED permanently for the E-5 seam: dispatch_one already owns ADD/supersede/noop; the pipeline delegates. No gate needed; recorded so no executor re-adds it.

## ABORT_CONDITIONS
- A1 SEAM REGRESSION: user_memory_capture.rs:256 dispatch_one, rule_cmd.rs:52 insert_user_rule, or either production caller (main.rs `/remember` arm, repl_command.rs:146) moves/renames/re-signatures such that a pre-action call site no longer exists -> STOP + BLOCKED handoff. Never invent entrypoint #3; production wiring IS the deliverable (R-8).
- A2 COMPOSE BREAK: ConfigLoader::admission_scan() unavailable or its row shape changes so the index cannot be built FROM manifest rows -> STOP; any private re-enumeration violates GD-1; stubbing/faking the manifest is forbidden.
- A3 SCHEMA LOCKSTEP FAILURE: the AdmissionVerdict action-type arm fights the golden/TOTAL_VARIANTS lockstep beyond a local fix within UNIT A1 -> STOP + handoff; baselines are re-read first (W-9), never guessed.
- A4 FROZEN-GATE DISAGREEMENT: plan_gate_check stays FAIL after bounded scoped iteration due to apparent checker disagreement -> STOP and report the exact failing points verbatim (this task's own stop rule). The checker is frozen to every worker forever.
- A5 BUILD LAW: cargo invoked ONLY as plain `cargo` (depot shim; TEIKOKU_DEPOT_OFFLOAD=remote). Absolute-path cargo/rustc, `rustup run`, and CARGO_TARGET_DIR overrides are banned and watchdog-killed ON SIGHT; a Killed build is NEVER retried (same command dies again); shim failure = BLOCKED with shim output. opus_land's gate owns the authoritative GREEN; local checks are best-effort feedback only.
- A6 GIT: any destructive-git temptation -> R-1 STOP (git is read-only for workers; only the orchestrator mutates git).
- A7 SCOPE: any demand to touch files outside this plan + conductor/opus_walk/evidence/J-109/refit-notes.md -> R-3 STOP + handoff; edits to checkers/tools/gates are included in that ban.
- A8 OBSERVE-MODE ESCALATION: any mid-flight request to enable block-mode (verdict actually refuses the write) or ship the governance tile render -> out of scope (GD-4 + UI hold); an ORCHESTRATOR decision via fresh TaskSpec, never a unilateral flip.

## UNITS (three one-worker-one-turn slices; shared flag ELAI_ADMISSION_HARDENING_ENABLE default-OFF; allowlist row lands in UNIT A1)

### UNIT A1 — StaticSpanIndex + lexical AD-2 + AdmissionVerdict row + E-5 seam (ONE worker, ONE turn)

- EXPECTED_OBSERVATION (A1.1 index core, ~150 LOC): `.elai_cc/crates/orchestrator/src/admission_pipeline.rs` exports `StaticSpanIndex` built STRICTLY from `elai_core::ConfigLoader::admission_scan()` rows (GD-1 — chunks/spans derive only from those rows' texts), fingerprint = blake3 over the rows' content_hash values, staleness checked and index rebuilt-if-stale BEFORE any verdict batch; pure `AdmissionPipeline::check_candidate(candidate_text) -> AdmissionCheckVerdict` runs the AD-2 lexical leg: token-set overlap vs manifest spans under NAMED CONSTANTS T_IMMUTABLE_LEXICAL and a max-span-size const; enum covers the full pipeline {AdmitClean, RejectImmutableSpan, RejectDuplicate, Merge, Coexist} with arms exercised progressively across units; outputs carry receipts {candidate_hash (blake3), surface_leg="lexical", threshold_used, verdict} sufficient to recompute the decision from the emitted row alone (GD-2).
  - ON_FAIL: whole-file row granularity proves too coarse to trip doctrine duplicates -> chunk within the module OVER row text only (still sourced solely from scan rows), record the chunking choice in workflow/reasoning/J-109-decisions.md; if acceptance still cannot be met without inventing data outside the manifest -> STOP per A2, never scrape extra sources.
- EXPECTED_OBSERVATION (A1.2 SSB arm net +1, ~60 LOC): new AdmissionVerdictPayload {candidate_hash, surface_leg, threshold, verdict, originating_seam, ts} + SsbActionType::AdmissionVerdict landed in lockstep: payload struct beside the MemoryGovernanceEvent block (pattern-2 fields per probed anchors), action-type variant + stringify + route-from-string mirroring action_type.rs:3923/:5764/:6657 precedents, TOTAL_VARIANTS bump + golden regen; SCHEMA_VERSION baseline re-read at edit (W-9), never assumed.
  - ON_FAIL: golden mismatch -> fix within the UNIT A1 file set; an upstream schema bump landed meanwhile -> re-read and re-pin the new baseline first (W-9), then fix forward; forcing receipts into an existing payload instead is the RT-4 violation, never attempted.
- EXPECTED_OBSERVATION (A1.3 E-5 seam wiring, ~40 LOC): flag-gated ELAI_ADMISSION_HARDENING_ENABLE (default-OFF) checked inside the capture flow IMMEDIATELY BEFORE user_memory_capture.rs:256 `dispatch_one`; when ON, emits the AdmissionVerdict row through the existing :527 emit_ssb_row helper; OBSERVE-ONLY: the write ALWAYS proceeds regardless of verdict; wire-dark-allowlist.toml gains ONE [[flag]] row following the established shape (name/type=experiment/owner/expiry_date/promotion_plan_id=J-109/smoke_test).
  - ON_FAIL: the call needs context absent at STEP-3 that would require out-of-scope plumbing churn -> STOP per A1 abort; half-wiring (module compiled, no call site) is definitionally DARK under production_caller_check and must never return green.
- EXPECTED_OBSERVATION (A1.4 tests, ~110 LOC): crates/contracts/tests/j109_admission_pipeline_contract.rs pure-fn arms: (a) doctrine-duplicate text vs a fixture manifest -> RejectImmutableSpan; (b) unrelated text -> AdmitClean; (c) stale fingerprint -> rebuild happens BEFORE any verdict; (d) CORRECTION ARM — a fixture span shaped like a cwd CLAUDE.md member trips the reject, proving config.rs:316 members are indexed; (e) receipts recomputable from emitted row fields (GD-2). PLUS an in-module #[cfg(test)] test in user_memory_capture.rs (existing test-block convention): flag ON + doctrine-duplicate `/remember` driven through capture_remember_with_clock in a temp project root asserts BOTH the AdmissionVerdict SSB row landed AND the fact persisted. FALSIFIABLE MUTATION hook: deleting the A1.3 call site turns exactly this in-module test RED while contract arms stay green — that asymmetry is why the seam-deletion test lives here, not in contracts.
  - ON_FAIL: a contract arm red -> fix producers within the A1 file set; the in-module test red while the call site LOOKS correct -> suspect flag ordering or emit-path divergence relative to :256 and re-trace STEP numbering; weakening assertions is forbidden.
- INTEGRATION_SMOKE (A1 — REAL entrypoint, observe-only): drive capture_remember_with_clock exactly as main.rs:1451 does (CLI arm), or via the repl_command.rs:146 path, with flag=1 against a temp project root containing one doctrine markdown; assert from the resulting SSB stream that an admission_verdict row exists AND the memory write completed — both together prove observe-only.

### UNIT A2 — I-12 intake front (rule-add arm)

- EXPECTED_OBSERVATION (A2.1 intake wiring, ~30 LOC): crates/cli/src/rule_cmd.rs gains the same flag-gated check_candidate call over the composed rule surface (title + description + predicate_json — the identical composition lifecycle feeds into UserRuleSpec) placed IMMEDIATELY BEFORE :52 insert_user_rule; verdict row emitted through the pipeline emit path; insert proceeds regardless (observe-only parity with A1).
  - ON_FAIL: surface pieces unavailable at that call point without churn -> STOP per A1 abort reasoning; duplicating lifecycle composition logic wholesale instead of reading the SAME fields is a GD-1 violation, not a fallback.
- EXPECTED_OBSERVATION (A2.2 smoke, ~45 LOC): crates/cli/tests rule-add smoke (l13_rule_cli_smoke.rs pattern or sibling new file) drives the REAL CliAction::RuleAdd handler with a doctrine-duplicate rule body, flag ON; asserts the admission_verdict row + unchanged `PROPOSED` status output.
  - ON_FAIL: red smoke alongside green contract tests -> the call site is ordered wrong relative to :52 (post-insert emission is NOT an admission gate); reorder, never relax assertions.
- INTEGRATION_SMOKE (A2): real CLI-command-level drive (not pure-fn only): invoke the RuleAdd arm end-to-end the way l13 does; verify row presence + insertion semantics in one transcript.

### UNIT A3 — cosine upgrade + AD-3 band + honest degrade

- EXPECTED_OBSERVATION (A3.1 cosine leg, ~90 LOC): upgrade AD-2 inside admission_pipeline.rs — embed spans + candidate via ort_session.rs:138 `embed` behind EmbeddingCircuitBreaker; NAMED CONSTANTS T_IMMUTABLE (0.70 start) / T_DUP (0.85) / T_CTX (0.60) (ID-5: STARTING values; recalibration data arrives from the observe-only rows themselves); breaker OPEN -> lexical leg RUNS with the row labeled surface_leg="lexical" (never blind-accept, never silent pass); healthy breaker -> row labeled cosine.
  - ON_FAIL: embed session unreachable outside the knowledge-stack attach context -> the labeled lexical degrade IS the shipped behavior for that environment; making embedding mandatory would break the observe-only guarantee. Escalate design-level only if breaker health cannot be observed programmatically -> STOP + handoff, never guess.
- EXPECTED_OBSERVATION (A3.2 AD-3 band, ~70 LOC): extend the decision fn with AD-3 vs EXISTING store entries: facts read via the elai_memory FactStore read API; active rules via lifecycle fetch_all_active_rules; >= T_DUP -> RejectDuplicate; inside the band split on trigger-Jaccard >= T_CTX -> Merge vs Coexist; every outcome journaled; still observe-only.
  - ON_FAIL: store read APIs differ from expectation (private/absent accessors) -> STOP + handoff naming the exact accessor gap; inventing a parallel store reader violates compose-never-duplicate.
- EXPECTED_OBSERVATION (A3.3 contract additions, ~40 LOC): j109_admission_pipeline_contract.rs gains arms: breaker-OPEN fixture -> lexical label present + verdict still issued; healthy-breaker fixture -> cosine label; near-duplicate existing entry -> RejectDuplicate; band + same-trigger -> Merge; band + distinct-trigger -> Coexist.
  - ON_FAIL: a labeled-degrade arm red -> roll back to the lexical-only build state within the A3 file set and re-attempt; shipping cosine WITHOUT the tested fallback recreates blind-accept on infrastructure failure and is forbidden.
- INTEGRATION_SMOKE (A3): re-drive BOTH real entrypoints (A1 capture flow + A2 rule add) under a forced breaker-OPEN fixture AND a healthy embedder; assert labeled rows land in both conditions and writes always proceed.

### DEFERRED-GATED WORK (binding carve-out — NOT built in this walk; each gate names what opens it)
- LEG B / AD-4 counterfactual replay: waits on TWO absent substrates: (a) byte-exact originating-prompt persistence — the SSB stores blake3 8-byte/16-hex prefixes BY DESIGN ("Raw prompt text is omitted by design", CTIM-Rover defense; zero fetch_prompt/prompt_journal symbols), reversing it reverses a recorded security decision needing owner sign-off with retention/redaction bounds; (b) a re-decide executor — PerRoleRuntime is a role->backend config map (get/set/list/priority accessors only, nothing executes). Both belong to their own planned unit first. Until opened, NO flip/replay verdict may be fabricated — the enum carries no ReplayRequired arm.
- J-97 case-reuse admission leg: waits on J-96 then J-97 shipping (zero case_graph symbols re-probed at cc8ac315). Cleared by: the walk executing those units.
- Full-immutable-surface expansion beyond the enumerated-today manifest (rules-as-data ACTIVE rows, operator prompts): waits on a J-102 extension unit adding them WITH hashes. Until then coverage wording STAYS manifest-as-enumerated-today (plus its seeded root members).
- Block-mode (the verdict actually refuses the write): waits on the FP rate bounded from >=3 real observe-mode runs — the calibration corpus is precisely the rows THIS walk produces (GD-4). Orchestrator-gated flip, never unilateral (A8).
- Governance tile rendering of verdict counts (MemoryDrawer/H-33 surface): waits on the owner lifting the ELAI UI hold.

## WIRING_CONTRACT
produces_symbols:
  - .elai_cc/crates/orchestrator/src/admission_pipeline.rs::{StaticSpanIndex, AdmissionPipeline::check_candidate, AdmissionCheckVerdict}
  - .elai_cc/crates/orchestrator/src/ssb: AdmissionVerdictPayload + SsbActionType::AdmissionVerdict (net +1, golden/TOTAL_VARIANTS lockstep)
  - flagged pre-action call sites at BOTH fronts: user_memory_capture.rs immediately before :256, rule_cmd.rs immediately before :52
consumes_symbols:
  - elai_core::ConfigLoader::admission_scan rows (J-102 manifest — the ONLY index source, GD-1, incl :316 cwd CLAUDE.md/AGENTS.md seeding)
  - crates/memory mem0 dispatch_one decision arms (E-5; AD-1 delegated, never duplicated)
  - crates/knowledge vector embed + EmbeddingCircuitBreaker (UNIT A3, labeled degrade)
  - elai_memory FactStore read API + rules lifecycle fetch_all_active_rules (AD-3 comparators)
  - SSB writer + golden/schema-lockstep infra; wire-dark-allowlist.toml [[flag]] convention
reachable_from:
  - .elai_cc/crates/cli/src/user_memory_capture.rs — capture_remember_with_clock STEP-3, callers main.rs:1451 (`/remember`) and repl_command.rs:146 (PRODUCTION front 1 of 2)
  - .elai_cc/crates/cli/src/rule_cmd.rs — insert_user_rule intake via CliAction::RuleAdd (PRODUCTION front 2 of 2)
  - a #[cfg(test)] caller NEVER satisfies this (R7); the falsifiable-mutation pair is the A1.4 in-module test
stub_ban:
  - NO LLM judge anywhere in the decision path (TD-3 — judge annotation belongs to deferred leg B only, ADVISORY, and its substrates do not exist yet)
  - NO blind-accept on embedder/breaker failure — labeled lexical leg or honest refusal, never a silent pass
  - NO admission decision without its journaled verdict row (observable_effect negative arm)
  - NO second exact-match/UPDATE arm beside dispatch_one (FORKS item 5 permanent descope)
  - NO inline thresholds — T_* NAMED CONSTANTS only; NO .unwrap in prod paths
  - NO fabricated replay/flip language (leg B deferred; no ReplayRequired arm exists in this walk)
frontend_pair:
  NONE shipped this walk — governance tile render is DEFERRED-GATED on the owner UI hold (deferred-work section); verdict counts stay readable straight from the SSB stream.
observable_effect:
  trigger: "flag=1 real run where an agent proposes a memory fact via /remember (or a rule via rule add) duplicating an ACTIVE rule span present in a manifest member"
  expect_ssb: "admission_verdict rows land carrying leg + threshold + verdict + candidate_hash; the write STILL commits (observe-only)"
  expect_artifact: "every decision recomputable from its row (GD-2); NEGATIVE arms that must FAIL: any write admitted behind flag=1 with zero verdict rows behind it, any unlabeled/undefined-tier verdict, ANY diff that blocks the write"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-109_learned_artifact_admission_hardening.md --run-trigger
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/J-plan/J-109_learned_artifact_admission_hardening.md   # PASS required; frozen tool
rg "T_IMMUTABLE|T_DUP|T_CTX|T_IMMUTABLE_LEXICAL" .elai_cc/crates/orchestrator/src/admission_pipeline.rs        # named constants present
rg -c "judge.*verdict|llm.*decides|judge_model.*admit" .elai_cc/crates/orchestrator/src/admission_pipeline.rs   # MUST be 0 (TD-3)
rg "lexical|breaker|fingerprint|surface_leg" .elai_cc/crates/orchestrator/src/admission_pipeline.rs             # labeled degrade + staleness legs present
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/admission_pipeline.rs  # 0-DARK required
test -f workflow/reasoning/J-109-decisions.md
```

## ACCEPTANCE
- [ ] AD-2 (+ AD-3 once A3 lands) run as ONE ordered decision function behind ONE flag at BOTH fronts (pre-:256, pre-:52); every decision journaled with leg + threshold + receipts recomputable from the row alone (GD-2).
- [ ] Index built FROM admission_scan() rows only; fingerprint rebuild-if-stale PROVEN before any verdict (contract arm c).
- [ ] cwd CLAUDE.md + AGENTS.md membership proven by the correction arm (config.rs:316 binding) — a span shaped like either root member trips RejectImmutableSpan.
- [ ] Degrade honesty: breaker-OPEN yields the labeled lexical leg; no blind-accept path exists anywhere.
- [ ] Default-OFF + single allowlist [[flag]] row; observe-only guaranteed (writes never blocked in this walk; negative-arm enforced); H-31 firewall intact: nothing admitted here ever reaches auditor/critic context (compositional order injection-scan -> trust tier -> J-109 preserved).
- [ ] TD-3: judge-decides grep count 0 on admission_pipeline.rs; production_caller_check 0-DARK on all new pub symbols (auditor rerun is the authority; test-root callers rejected per R7).
- [ ] No new dependency; plain `cargo` depot shim usage only (opus_land gate owns the authoritative GREEN verdict).

## WIRING DONE-GATE (clause by clause)
- W-1: done = admission_verdict rows from REAL entrypoint drives (capture flow + rule add), not file-exists; artifacts archived off-agent (VERIFY artifact 3).
- W-2/R9: ELAI_ADMISSION_HARDENING_ENABLE carries one allowlist [[flag]] row (type=experiment, owner, expiry_date, promotion_plan_id=J-109, smoke_test) landing in UNIT A1.
- W-3: mechanisms ported from summary file:line grounding (Apache-2.0 tree only; AGPL mod untouched; zero code copied).
- W-4: measures behavior (journaled rows + persisting writes), not stdout claims; per-unit smokes pin it.
- W-5: production_caller_check.py against the two pinned fronts; test roots rejected (RT-5 consequence).
- W-6: frontend deliberately NONE this walk (deferral stated honestly in frontend_pair); counts readable from raw stream meanwhile.
- W-7: anchors re-grepped at edit — E-5/I-12/J-102 anchors, callers, SSB blocks; drift re-read + re-pinned (main.rs :1419->:1451 precedent followed).
- W-8: gate/checkers frozen; the plan never edits them (A4).
- W-9: SCHEMA_VERSION + golden/TOTAL_VARIANTS baselines re-read at edit; AdmissionVerdict = re-read-baseline + 1.
- W-10: observe-only now; block-mode + leg B tiered behind their named gates — no partial shipping of either.
- GD-1: composes J-102/E-5/I-13/1.1/SSB-infra; re-implements none (closed forks above form the boundary map).
- GD-5 honest residue: threshold values remain AgenticSTS transfer starting points (ID-5); observe-only rows are the calibration source; leg-B absence documented with named gates, never silently skipped; grounding refreshed this refit with drifts recorded inline rather than carried silently.

## Files touched
[A1] CREATE: .elai_cc/crates/orchestrator/src/admission_pipeline.rs (~150 LOC; grows ~160 more in A3)
[A1] MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (module decl) (~2 LOC)
[A1] MODIFY: .elai_cc/crates/orchestrator/src/ssb/payloads.rs + ssb/action_type.rs + golden/TOTAL_VARIANTS lockstep (net +1 arm) (~60 LOC)
[A1] MODIFY: .elai_cc/crates/cli/src/user_memory_capture.rs (pre-:256 flagged call + cfg(test) seam test) (~50 LOC)
[A1] MODIFY: .elai_cc/wire-dark-allowlist.toml (+1 [[flag]] row)
[A1] CREATE: .elai_cc/crates/contracts/tests/j109_admission_pipeline_contract.rs (~110 LOC)
[A1] CREATE: workflow/reasoning/J-109-decisions.md (threshold provenance, descope record, chunking choice)
[A2] MODIFY: .elai_cc/crates/cli/src/rule_cmd.rs (pre-:52 flagged call) (~30 LOC)
[A2] CREATE: .elai_cc/crates/cli/tests/j109_rule_add_smoke.rs (or l13 sibling extension) (~45 LOC)
[A3] MODIFY: admission_pipeline.rs + contract additions (cosine, AD-3 band, labeled degrade; ~200 LOC total)
[none] Deferred legs above: NOTHING touched — the carve-out binds scope.

## Citations / Grounding
- workflow/papers/agenticsts-summary.md: write_gate.py:627 cascade, :323 StaticSpanIndex, :43-52 named thresholds; prewrite_ab.py:19 byte-exact fetch_prompt_a, :244 aggregate_strict zero-harmful + supermajority; embed-fail never-blind-accept rule — all Apache-2.0 tree (mechanisms ported, NOT code). arXiv:2607.02255 verified live 2026-07-04.
- ELAI ground truth probed 2026-08-27 @ HEAD cc8ac315: core/config.rs:315/:316/:334-349; cli/user_memory_capture.rs:170/:190-region/:256/:527; cli/main.rs:1451 (brief cite :1419 drift recorded); cli/repl_command.rs:146; cli/rule_cmd.rs:52; rules/lifecycle.rs:356/:370; orchestrator/rule_curator.rs:180; cli/knowledge_stack.rs:108/:918; knowledge vector module + ort_session.rs:138 + circuit_breaker.rs:98; orchestrator/ssb/payloads.rs:19158/:19159/:19574; ssb/action_type.rs:3923/:5764/:6657. Probe commands + outcomes: conductor/opus_walk/evidence/J-109/refit-notes.md; slice rationale: same dir J-109-seam-proposal.md §2/§4-§9.
- Doctrine: K-19 SF-1..SF-4 (binds deferred leg B); H-31 auditor firewall (arXiv:2510.09905 carried); H-43 family (feature-map 1.7:1486-1500); TD-3; GD-2/GD-4/GD-5; ID-5; feature-map refs 1.5:696 (E-5), :962 (I-13), :964 (I-12), 1.8 (J-102).

## Next
Leg A gives both learned-artifact seams the immutable-surface dedup leg the promotion-time filters assumed existed: nothing enters through the `/remember` or `rule add` fronts claiming novelty the instruction surface already states — observed honestly, journaled always, blocking nobody until the false-positive number says otherwise. The observe-only rows ARE the calibration corpus for threshold rebasing (ID-5) and the FP evidence gate for block-mode (GD-4). Leg B (counterfactual replay fitness) joins when its two substrates exist under owner sign-off; the J-97 reuse-admission leg plugs the same pipe when J-96 lands.

End of J-109.
