# J-LACUNA-FILL grounding map — the lacuna-fill track (J-87..J-94)
Authored 2026-06-26 (main-LLM in-band per paper-to-plan phase 5). Source run:
workflow/reasoning/LACUNA-FILL-PROGRAM-2026-06-26.md (a synthesis of LACUNA-SWEEP-2026-06-26.md
+ four external deep-research reports 7-10 + a full ELAI-FEATURE-MAP read + a targeted grep over
the B..K plan summaries). The reasoning file IS the paper-to-plan PHASE-4 steal analysis; this
wave is PHASE-5 authoring. K-native half already shipped as the LFG wave K-20/K-21/K-22 (the
re-executable-RRA differentiator + the journal-completeness doctrine + the doctrine-enforcement
coverage ledger). This track is the TECHNICAL half ("recorded NOT authored... also author the J
track now", feature-map Section-4 LFG verdict block).

This map is the W-7 anchor: every file:line a J-87..J-94 detail file uses is listed here or marked
PLAN-asserted (the executor re-greps at edit per W-9). NO arXiv ID is introduced by this wave
(the reasoning doc cited deep-research REPORTS 7-10 + prior-art NAMES, not arXiv ids); external
prior-art is cited by name as DESIGN-RATIONALE only (HD-1/ID-5) — every done-gate is ELAI-measured.

================================================================================
CANONICAL CONSTANTS (LIVE, re-derived 2026-06-26; NEVER hard-code, re-read at edit per W-9)
================================================================================
- SCHEMA_VERSION = SCHEMA_VERSION_DDL_EPOCH + SSB_ACTION_TYPES.len(); epoch derived at
  crates/orchestrator/src/ssb.rs:1503/1508, public alias :1513.
- SSB_ACTION_TYPES array head ssb.rs:62; TOTAL_VARIANTS lock at
  crates/orchestrator/tests/action_type_lock.rs (the :705/:775 region across waves).
- BASELINE FOR THIS WAVE: SCHEMA ~268 / TOTAL_VARIANTS ~577 (the 5b0a66d drain stamp; the
  feature-map "241/550" header line is the 2026-06-12 number and is STALE). Re-grep at edit.
- This wave is PREFER-REUSE-FIRST: most fills reuse an existing action_type with a new *Kind arm
  (net 0); the genuinely net-new events are E1's reconcile/observer rows (J-87) and at most the
  L1 spec-gate rows (J-89). A partial 9-tuple add breaks the workspace (91-error incident) — full
  lockstep or reuse, never half.

================================================================================
FILL -> FILE MAPPING (the unlock-DAG; reasoning §2-§6 + §9-§10)
================================================================================
- J-87  E1 journal-completeness substrate  [WAVE 1, KEYSTONE, the ONE genuine net-new build]
- J-88  E2 runtime-observed DKG edges + E3 machine-derived-first audit lane  [WAVE 1, enablers]
- J-89  L1 objective-correctness gate + L8 plan-DAG type-completeness  [WAVE 1, prevention pair, ruthless-3]
- J-90  L5 cross-lang+dynamic blast cone (USER'S #1) + L4 usage-weighted reachability + L7 cant-lie x mutation floor  [WAVE 1 L5 / WAVE 2 enforcement]
- J-91  L3 re-executable RRA  (= a J-14 EXTENSION; K-20's technical companion)  [WAVE 2, the WEDGE leg]
- J-92  L6 orchestrator-FSM dispatch ordering  (the logged P0 made structurally illegal)  [WAVE 2]
- J-93  Tier-2 composition wiring fills (7 legs)  [WAVE 2]
- J-94  Tier-3 honesty fixes + THIN measure-first  [WAVE 3; THIN explicitly deferred/measure-first]

RUTHLESS-3 (reasoning §9, where leverage concentrates): J-87 (E1) + J-90's L5 leg + J-89 (L1+L8).

================================================================================
TWO SEQUENCING RULES THAT BIND EVERY FILE (reasoning §1)
================================================================================
- RULE 1 MECHANISM vs ENFORCEMENT SPLIT. Every GATED fill ships in two halves: MECHANISM (now,
  OBSERVE-ONLY, emits the signal, blocks nothing, GD-4) and ENFORCEMENT (gated on the
  J-34-MEASURED per-(model,lane) knee, never a paper constant ID-5, promoted only after FP/over-
  block bounded on >=3 real runs). J-34 (EMPIRICAL CONTEXT CALIBRATION, PLANNED, ABSENT today) is
  the implicit prerequisite for the ENFORCEMENT half of L1/L4/L7. Until J-34 lands they stay
  observe-only — still a real improvement (the signal is visible), not a blocking gate.
- RULE 2 BUILD BY THE UNLOCK-DAG. E1/E2/E3 are enablers that precede their consumers: E1 feeds
  L3/canvas/C5-RULES labels; E2 feeds L5; E3 hardens every binary gate. Build roots first.

================================================================================
ELAI SUBSTRATE ANCHORS (the seams the wave rides; grep-confirmed in the 2026-06-26 feature-map
read unless marked PLAN-asserted; executor re-greps at edit, W-7/W-9)
================================================================================
E1 (J-87):
- LA-1 TIER-C PROCESS ISOLATION [WIRED] minimal OS processes, seccomp 2 profiles, 15-msg postcard
  IPC; crates/agent + crates/ipc-types; phase-4A. The mediation seam EXTENDS this (seccomp profile
  add). PLAN-asserted call-site: the Tier-C tool-dispatch / sandbox seam.
- LA-2 BLAST-RADIUS RUNTIME GATES [WIRED] G-3 post-shell-EXPANSION FS gate (canonicalize ~/glob/
  symlink before verb) + egress IP-pin; Linux-only. The mediation leg extends THIS (it is the
  same Linux-only privileged-host posture).
- LA-3 SSB SHARED STATE BOARD [WIRED] single-writer SQLite WAL, append-only, broadcast-after-
  commit; ssb.rs. The reconcile gate reads the journal; the mediated row commits atomically here.
- LA-4 K-21 .claude/rules/journal-completeness.md JC-1..JC-4 (PLANNED, K-native companion) — the
  binding pre-gate for this sub-track. J-87 IS the E1 build K-21 pre-gates.
- LA-5 EXTERNAL PRIOR-ART (design-rationale only, WebSearch-confirmed 2026-06-26): SECCOMP_RET_
  USER_NOTIF (Linux 5.0+, userspace supervisor intercepts connect/openat via SECCOMP_IOCTL_NOTIF_
  RECV/SEND) — DOCUMENTED CAVEAT: it is NOT a TOCTOU-safe boundary on pointer-deref args (kernel.org
  seccomp_filter.html), so it mediates atomic-commit-with-row, the observer is the real backstop;
  eBPF/LSM observers Tetragon/Falco/Tracee (CNCF, in-kernel syscall+file+network provenance, 2.7-6%
  overhead class CamFlow/LPM) — Linux-only. report 8 §re-executable/§verifier-isolation.

E2/E3 (J-88):
- LA-6 DKG [WIRED] tree-sitter -> SQLite symbol graph, typed edges imports/contains/implements;
  crates/knowledge; phase-1A. E2 adds an observed_calls edge KIND (a DKG edge kind, NOT an SSB row).
- LA-7 SYMBOL-LEVEL IMPACT API (B-29) [WIRED] impact_radius/call_flow/symbol_context. The cone E2's
  runtime edges enrich (and J-90 consumes). RULE: runtime edges may NEVER SHRINK the static cone
  (report 9; static is the conservative backbone). DIVER's "35-50% reduction" is design-rationale
  NOT an ELAI number.
- LA-8 G-2 RRA action-stream [WIRED] + L3 entrypoint coverage + L4 traces = the runtime-edge SOURCE.
- LA-9 the AUDITOR (OS-isolated, reads executor-authored artifacts today) — E3 routes its BINARY
  VERDICT lane to machine-derived artifacts ONLY (coverage %, SARIF rule-ids, syscall classes,
  unmatched-observer counts, bundle digests); report 8's sharp finding: DO NOT SHOW SOURCE in that
  lane (AST stylometry deanonymizes). The LLM-auditor prose stays ADVISORY (TD-3, already doctrine).

L1/L8 (J-89):
- LA-10 OBJECTIVE-HASH SEALED RECEIPTS (I-45) [PLANNED] blake3(canonical objective bytes) into the
  rra.rs seal — L1 makes the SEALED objective a typed evidence-bound spec-bundle BEFORE code.
- LA-11 spec-grounder agent + grounding-sweep (ADVISORY pre-steps today) — promoted to a first-
  class taint-tracked artifact with DETERMINISTIC coverage checks.
- LA-12 ADVERSARY ROLE CROSS-FAMILY (D-13) [DARK] + gpt-bridge — the cross-family substrate for the
  spec-JURY (the ONE LLM entry in the whole program, legal because PLANNING-time + advisory-by-
  construction + DISTINCT from the auditor; TD-3 §8 R2). PoLL diverse-small-judge-panel is design-
  rationale.
- LA-13 wiring_contract_check.py producer/consumer symmetry (.claude/tools, WIRED) — L8 LIFTS this
  RUNTIME symmetry to PLAN-DAG time (wire-dark PREVENTION not detection). BabyStepSpec 8-block
  schema + C-29 contract extraction + Kahn cycle detection (phase-3G) are the typed plan substrate.

L4/L5/L7 (J-90):
- LA-14 BLAST-RADIUS / ANTI-REGRESSION GATE (J-16) [PLANNED, KEYSTONE, user's #1] blast_radius.rs
  reverse-dep cone + regression-selection. L5 FUSES cross-lang + E2 runtime edges into this cone.
- LA-15 elai.map (all-langs, SCIP-enriched) + cross_lang_diff.py (Rust<->Python, Tauri<->Svelte)
  [.claude harness, WIRED] — the cross-language edges. Seam extractors for Tauri generate_handler!
  <-> TS invoke() + sidecar route <-> Rust client (both golden-locked). OPAQUE SEAMS WIDEN NEVER
  NARROW the cone (report 9 safety rule); log() what was widened.
- LA-16 LIVING EVAL CORPUS (J-56) [PLANNED] + E-54/G-6 — the frozen WEIGHTED representative-workload
  corpus L4 fuses into the wiring gate (usage-weighted exercisedness; min weighted score AND >=2
  scenario families, report 9 anti-smoke rule).
- LA-17 TEST-INTEGRITY CANT-LIE GATE (F-4) [WIRED] AntiFabricationGate.check() blake3 stdout_hash +
  MUTATION TESTING GATE (C-28 PARTIAL) + I-20 budgeted mutation scheduler [WIRED] — L7 composes
  F-4 X C-28/I-20 scoped to J-16's diff cone (honest-pass AND meaningful-test).

L6 (J-92):
- LA-18 FULL FSM [WIRED] 9 states + DAIRA evidence gates + InteractiveGate/AutonomousGate
  EvidenceCheck; FsmSubstate (precedent Framing B-40, agent_substate AUDITING/MEMORIZING).
- LA-19 DIRECTIVE SUPERVISOR PLANE (I-1) [PLANNED] generator emits Intent->Directive, process-
  separated Supervisor is the ONLY executor — L6 moves orchestrate-protocol.md H-1..H-10 from PROSE
  into FSM substates + capability tokens. J-66 SSB temporal-monitor + J-68 ElaiFSM.tla prove the
  ordering invariant.
- LA-20 orchestrate-protocol.md H-1..H-10 + the Stop-hook (the live P0 origin: the main LLM
  rationalizing past dispatch order).

Tier-2 (J-93):
- LA-21 MAST 14-mode classifier (E-6) [WIRED] runs POST-HOC -> wire as an IN-LOOP DETERMINISTIC
  supervisor over the live SSB action_type stream (FM-2.6 stated-plan-vs-actual divergence, FM-1.3
  action_type repetition, StuckLoop at 3 identical; report 10 §2 "all rule-based", TD-3).
- LA-22 SESSION DAG / H-49 saga log [PARTIAL] + H-35 /rewind — the compensation REGISTRY (a
  compensation per invertible side-effecting tool; non-invertible surfaced-to-human, RV-4). Tier-C
  mediates+journals every side-effecting tool = the registry key.
- LA-23 G-1 TAINT PROPAGATION [WIRED] Provenanced<String> (4 ingest sites) + crates/memory (E-5
  PARTIAL) + J-21 memory poisoning + H-51 cross-session handoff — thread G-1 Provenance THROUGH the
  memory write path (taint survives a stored-then-retrieved fact).
- LA-24 GRC (phase-2A) per-step grounding tags + GROUNDING VERIFICATION 6 methods (D-10
  select_grounding_mode) — route GRC's per-step citation through the DETERMINISTIC SOURCE_CHECK at
  STEP grain (today only CLAIM grain). Home = the J-83 BeliefState per-step receipt gate.
- LA-25 GoalDriftCanary (C-10 / B-22) + I-45 sealed objective — preference-drift vs capability-drift:
  diff WHICH constraint dropped vs the sealed objective; a dropped SAFETY/PRIVACY/SECURITY
  constraint escalates to human (NO auto-retry, report 10 §4 hard rule).
- LA-26 RULES ENGINE (phase-3F) promotes on fpr/tpr/causal — source non-LLM labels from the SSB
  OUTCOME stream (a BLOCK is a false-positive if the SAME change later passed all deterministic
  gates + shipped clean). Consumes E1 completeness for the outcome record to be trustworthy.
- LA-27 I-39 derives plan STATUS from the journal — C9-PLANS auto-derive the feature-map's MECHANICAL
  half (status flags + grep SCHEMA/TV + elai.map/codegraph composition edges), leaving only the
  design-intent "why" hand-authored. (K-22's only cross-track tie OUT-OF-K.)

Tier-3 / THIN (J-94):
- LA-28 D-6 held-out store -> C1-GROUNDMODES route-by-measured-sufficiency (offline calibration loop).
- LA-29 J-73 should_retrieve gate (WHEN) + C-29 IntentPair -> C2-CASCADE which-modality router (WHICH).
- LA-30 QueryTrace/SearchMode degradation -> a "context-incomplete" SSB trust marker (J-54 non-RRA
  stamp precedent) raising audit strictness.
- LA-31 J-13 batch-invariance (mm/addmm, NOT attention on Ampere/FA2) -> bind the attention residue
  to the J-14 ReproPack tier (T1 token-faithful only when attention is batch-invariant, else honest
  T0 downgrade; CL-4/GD-5).
- LA-32 C8-DASHBOARD plan-vs-execution drift view (I-45 sealed skeleton vs journaled actions; safety-
  class drift red) — pure read-only SSB projection, consumes the C3-AUTONOMOUS fill.
- LA-33 THIN (measure-first, do NOT build on faith): C3-BSP intra-agent superstep barrier (H-49 edit-
  grain), C7-ROUTING phase-keyed mid-trace routing (HD-2 cache tension), C5-SELFMOD local-lane
  checker-as-cross-family substitute (H-43 fitness + J-68; ensure mutable=0 immune-tier MEMBERSHIP
  lives in the frozen gate-manifest), C8-CANVAS shadow-anomaly nodes (derivative of E1).

================================================================================
THE RESIDUE LEDGER (every fill names its residue + a fail-closed monitor, or it over-claims;
reasoning §7) — carried verbatim into each file's "Open questions / residue" section
================================================================================
- L1 spec gate: Rice/halting — never "correct objective", only "survived N independent challenges +
  frozen attestation"; abstain->human (TD-5) is a first-class EXIT; jury gameable by CORRELATED
  cross-family blind spots + position bias (PoLL mitigates never eliminates).
- E1 completeness: host-trust CEILING (a privileged host/kernel adversary defeats both logs ->
  Rekor/TPM-IMA anchoring, a RIDER on I-18/G-8, not claimed now) + schema-mismatch FP FLOOR (temp-
  file churn, path aliasing, retries -> a normalization layer is MANDATORY; the claim is "every
  COVERED effect class left a row", never universal completeness); observer leg Linux-only (Windows
  = mediation-only fallback). monitor-the-monitor: dropped-event counters fail closed; "observer
  impaired" is a FIRST-CLASS verdict.
- L3 re-executable: world-drift — a cassette proves the OLD world; surface as a fidelity downgrade,
  never a portal back to the live world; non-snapshottable external effects (payments/emails) stay
  action-faithful-only with a visible note.
- L4 usage-weighted: frozen-corpus OVERFIT -> hidden holdout slice + incident-derived rotation on a
  SEPARATE cadence; threshold MEASURED (J-34) never a paper constant.
- L5 blast cone: opaque-seam FALSE-NEGATIVES (ad-hoc shell-outs, string-built routes, unregistered
  event buses) -> WIDEN-never-narrow + fall-back-to-broader-tests + log() what was widened.
- L7 mutation: equivalent-mutant — a strong LOWER BOUND not an oracle; tier it (changed-line in PRs,
  changed-cone in merge queue, full nightly).
- L6 orchestrator: the regress bottoms out at PROVEN CODE or a HUMAN — honest, by design (L1-HardVeto).
- C3-AGENTS supervisor: monitor-overload -> conservative thresholds, warn-before-halt.
- C3-SESSIONDAG compensation: not all external effects invertible -> non-invertible surfaced-to-human
  (cannot un-send an email; a DELETE may be unrecoverable).

================================================================================
PROGRAM-LEVEL RISKS (reasoning §8; bind the WHOLE track)
================================================================================
- R1 GATE-PROLIFERATION / FP-FATIGUE: ~20 new fail-closed gates raise the over-block rate. Per-gate
  FP ceiling + per-changeset gate-traversal budget; every gate OBSERVE-ONLY until FP bounded on >=3
  runs (GD-4). Sequence by leverage; do NOT ship all gates at once.
- R2 TD-3 BOUNDARY: the in-loop MAST supervisor MUST be DETERMINISTIC over the action_type stream;
  the spec-jury is the ONE LLM entry, legal only because PLANNING-time + advisory + distinct from
  the auditor. Any fill smuggling an LLM into a hot-path deterministic gate violates TD-3.
- R3 PLATFORM ASYMMETRY: cloud gets strong-E1 (eBPF), the cross-family jury (gpt-bridge), the cheap
  cassette; Windows/~1.8GB-VRAM local gets DEGRADED forms (E1 mediation-only; jury -> deterministic-
  checker substitute; local cassette gated by the C7-SIDECAR attention residue). Document per lane
  (CL-4); never advertise the cloud guarantee on the local lane.
- R4 MEASUREMENT PREREQUISITE: the ENFORCEMENT half of L1/L4/L7 cannot ship honestly until J-34
  measures the per-(model,lane) knees.
- R5 CONFIRMATION-BIAS on the meta-finding: "ELAI occupies the empty cell" is externally corroborated
  (report 8 verbatim) but is a DESIGN claim; it is not a MARKET edge until G-6 SAFEDELTA measures.

================================================================================
BINDING CORRECTIONS (LFG-1..LFG-5; the anti-duplication + doctrine guards)
================================================================================
- LFG-1 K-native already shipped: K-20 (re-executable-RRA differentiator) / K-21 (journal-
  completeness doctrine JC-1..JC-4) / K-22 (doctrine-enforcement-coverage ledger) are the PROVE/
  PACKAGE/DISCIPLINE slivers. This J track is the TECHNICAL build ONLY. Re-authoring a K sliver
  here is a GD-1 P0.
- LFG-2 anti-duplication grep CONFIRMED (2026-06-26): a grep over every *SUMMARY* index + phase-
  index-MASTER returned ZERO for eBPF / USER_NOTIF / observed_calls / spec-jury / plan-DAG /
  usage-weighted / compensation-registry / capability-token. These fills are net-new/composition,
  not re-proposals. E1's observer+reconcile leg is the ONE genuine green-field substrate (the
  mediation leg EXTENDS the shipped G-3/Tier-C).
- LFG-3 NO arXiv id introduced. The reasoning doc grounded via deep-research REPORTS 7-10 + prior-
  art NAMES; this wave cites those names as DESIGN-RATIONALE only (HD-1/ID-5), never a load-bearing
  number, never a fabricated arXiv id. Every done-gate is ELAI-measured on an ELAI gold set.
- LFG-4 MECHANISM/ENFORCEMENT split (RULE 1) is mandatory on L1/L4/L7 — ship the MEASURE observe-
  only, gate only on the J-34 knee.
- LFG-5 each fill ships its NAMED RESIDUE + a FAIL-CLOSED MONITOR (the residue ledger above), or it
  over-claims. The program pushes untrust to a PROVEN-CODE-or-HUMAN floor; it does not eliminate it.

================================================================================
SOURCE OF RECORD
================================================================================
- workflow/reasoning/LACUNA-FILL-PROGRAM-2026-06-26.md  (the PHASE-4 synthesis; §0 frame, §1 two
  rules, §2 enablers, §3 composition gates, §4 Tier-2, §5 Tier-3, §6 THIN, §7 residue, §8 risks,
  §9 ruthless-3, §10 packaging, §11 readiness).
- workflow/reasoning/LACUNA-SWEEP-2026-06-26.md  (the 27-concept 6-step sweep upstream of it).
- the four deep-research reports 7-10 (downloaded 2026-06-26 02:22): report 7 objective/spec gate +
  DAG preflight; report 8 journal completeness + re-executable bundles + machine-derived audit;
  report 9 deterministic done-gates (usage-weighted reachability + cross-language blast + test
  meaningfulness); report 10 ordered orchestration + in-loop semantic supervision + citation
  entailment + preference-drift.
- ELAI-FEATURE-MAP.md Section-4 LACUNA-FILL-PROGRAM verdict block (the K-author's record of this
  split) + the 1.6/1.14 K-20/K-21/K-22 entries (the K-native companions).
