# DEEP-G-002 exhaustive feature discovery

## Scope, method, and evidence contract

- **Exact assigned chunk:** `DEEP-G-002`, Family G, five `numbered/index original` files, 1,539 logical lines, 137,029 bytes; manifest target status `under 1,800`.
- **Permitted orientation:** `docs/thefinalglance/feature-discovery/07-G-SUMMARY_INDEX.discovery.md` was used only to locate summary additions/corrections. It is not source evidence.
- **Isolation:** all five assigned originals were read completely. No other G original, referenced implementation source, rationale, research paper, strategy document, or plan chunk was read. Consequently, `SHIPPED`, `ABSENT`, `grep-confirmed`, legal, benchmark, and paper claims below are documentary assertions of these five sources, not independent verification.
- **Atomicity/deduplication:** one row captures one semantic capability, state rule, schema contract, configuration rule, integration, failure policy, evidence requirement, test gate, dependency, constraint, or non-goal. Repeated Task/WIRING/ACCEPTANCE/Test wording is collapsed and all material anchors are retained. Registries/interactions reference rows and are not counted again.
- **Evidence codes:** `E-S` = explicit existing/shipped-state assertion; `E-P` = explicit proposed/normative behavior; `E-D` = explicit deferred/optional/non-goal; `E-G` = explicit acceptance/done gate; `E-R` = explicit risk/failure/fallback/recovery rule. `I`, `A`, and `C` occur only in their separated ledgers.
- **Anchor keys:** `G2`, `G6`, `G8`, `G9`, and `G10` mean the correspondingly numbered assigned file. `SUM` means the permitted orientation discovery.

## Exact processed manifest and integrity verification

| Chunk | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---|---:|---:|---|---|---|
| DEEP-G-002 | numbered/index original | 279 | 22,953 | `b70c014def1b95932dc4d7a5c0d0a795bd249aab130b05140d7be82d7b5b28d8` | `docs/thefinalglance/G-plan/G-2_reproducible_run_artifact.md` | exact |
| DEEP-G-002 | numbered/index original | 301 | 27,117 | `d3ad5003f857dd354915857497bf1d215d5154a585763a32a2736df340342d07` | `docs/thefinalglance/G-plan/G-6_safedelta_benchmark.md` | exact |
| DEEP-G-002 | numbered/index original | 310 | 29,442 | `0e8b82f6d21e5e715debb93e7f8a7e4378e87a468c8039b3e42e52cfa311fce4` | `docs/thefinalglance/G-plan/G-8_intoto_attestation_article50.md` | exact |
| DEEP-G-002 | numbered/index original | 308 | 28,308 | `709b3f6ab79dee2354470dda33a9078c7d701dd56daf5ce2196027894a126ae3` | `docs/thefinalglance/G-plan/G-9_journal_redaction_ttl.md` | exact |
| DEEP-G-002 | numbered/index original | 341 | 29,209 | `b04b65f60259d84979c537bf310366b83b0a55bfb65d3fe8e1823d87c9ea2f56` | `docs/thefinalglance/G-plan/G-10_model_drift_regression_canary.md` | exact |
| **Aggregate** | **5 originals** | **1,539** | **137,029** | — | exact manifest aggregate | exact |

Verification used `sha256sum` and `wc -l -c` on exactly the five paths above. Every digest, per-file count, and aggregate count has zero variance from `DEEP-SOURCE-MANIFEST.md:L654-L664`.

## Per-source exhaustive atomic ledgers

### G-2 — Reproducible-Run Artifact (RRA)

**Status boundary:** the header remains `PLANNED`, but this file contains a 2026-06-04 correction to its production emit route; G-8 later asserts that G-2 landed “this run,” whereas G-6/G-9 still call `rra.rs` absent and G-10 alternates between shipped/co-shipped and planned. Those cross-source status differences are not normalized.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DG002-G2-001 | E-P | G-2 is the planned Wave-2 keystone and explicit anchor for G-5, G-6, G-8, and G-9. | G2:L1-L3, L272-L275 |
| DG002-G2-002 | E-P | The product outcome is one portable, off-machine `.rra` trust artifact rather than internal journal plumbing. | G2:L5-L10, L29-L36 |
| DG002-G2-003 | E-P | The artifact bundles the SSB action stream, `ELAI_SEED`, `HardwareClass`, `system_fingerprint`, and model pin. | G2:L6-L8, L38-L40, L74-L80 |
| DG002-G2-004 | E-P | Its evidence manifest carries every consumed F-gate verdict hash, explicitly F-1 `source_blake3`, F-4 `stdout_hash`, F-9 `dep_name_hash`, and F-10 `delta_blake3` when available. | G2:L7-L8, L18-L20, L52-L53, L79-L81 |
| DG002-G2-005 | E-P | Bundle integrity is a BLAKE3 Merkle root over stream, evidence manifest, and pins. | G2:L6, L81-L82, L114-L118, L268-L269 |
| DG002-G2-006 | E-P | `elai replay --verify` replays the recorded action stream and rechecks all available F-gate verdict hashes, returning success or the first divergence. | G2:L8-L9, L84-L90 |
| DG002-G2-007 | E-P | The honest headline is reproducible action stream plus divergence verification, not unconditional bit-identical token replay. | G2:L10-L12, L38-L44 |
| DG002-G2-008 | E-S | The source asserts an append-only SQLite-WAL SSB with typed `write_payload`, action-type golden lock, and hashed tool I/O already ships. | G2:L15-L16, L35-L36, L46-L48 |
| DG002-G2-009 | E-S | `ELAI_SEED`, `HardwareClass`, and `system_fingerprint` primitives are asserted shipped, but their canonical module path must be confirmed at authoring. | G2:L16-L18, L49-L51, L64 |
| DG002-G2-010 | E-S | F-1/F-4/F-9 verdict fields are asserted shipped; F-10 is consumed when it lands rather than reimplemented. | G2:L18-L20, L52-L53 |
| DG002-G2-011 | E-S | `recover_from_checkpoint` is a stub; the refuted claim that BSP has no production caller must not be revived because `task_runner.rs:5018` is asserted non-test. | G2:L32, L56-L57, L114-L118, L253-L257 |
| DG002-G2-012 | E-P | G-2 may proceed in parallel with G-1 and G-4, and may ingest G-1 `taint_flow` rows. | G2:L22-L23 |
| DG002-G2-013 | E-P | G-2 must coordinate rather than overlap with any plan editing the `ssb.rs` write path in the same window. | G2:L22-L23 |
| DG002-G2-014 | E-P | The local seeded guarantee reproduces tool order plus tool input/output hashes in the action stream. | G2:L38-L40 |
| DG002-G2-015 | E-P | Every manifest F-gate verdict hash is intended to be offline-recheckable. | G2:L40-L41 |
| DG002-G2-016 | E-R | Cloud-provider fingerprint opacity and garbage-collected response bodies downgrade affected steps to `VerifyDivergence`; they may not be represented as token re-execution. | G2:L40-L43, L88-L90, L260-L265 |
| DG002-G2-017 | E-P | A green replay proves reproduction, not correctness, and the artifact records its actual guarantee level per run. | G2:L42-L44, L114-L118 |
| DG002-G2-018 | E-P | `RunArtifact` fields are `schema_version`, `elai_seed`, `hardware_class`, `system_fingerprint`, `model_pin`, `action_stream`, `evidence_manifest`, `root_blake3`, and `guarantee_level`. | G2:L74-L78 |
| DG002-G2-019 | E-P | `GuaranteeLevel` has the explicit states `ReproducibleTokens`, `ReproducibleActionStream`, and `VerifyDivergence`. | G2:L77-L79 |
| DG002-G2-020 | E-P | `GateVerdictHash` contains `gate`, `action_type`, `verdict_blake3`, and `ssb_event_id`. | G2:L78-L80 |
| DG002-G2-021 | E-P | `build_rra(&Ssb, seed, pins)` reads the session SSB rows and extracts the F-gate manifest. | G2:L79-L81 |
| DG002-G2-022 | E-P | `write_rra` serializes with `rmp-serde` plus a confirmed in-tree compression codec. | G2:L58-L59, L81-L82 |
| DG002-G2-023 | E-P | `ReplayVerdict` has `Verified`, `Divergence { first_event_id, expected_blake3, got_blake3 }`, and `Tampered`. | G2:L84-L88 |
| DG002-G2-024 | E-P | Verification first rereads the artifact and recomputes the Merkle root as the tamper check. | G2:L84-L85 |
| DG002-G2-025 | E-P | Verification freshly recomputes each gate verdict where its inputs are present. | G2:L85-L86 |
| DG002-G2-026 | E-P | Action replay runs under the pinned `ELAI_SEED` and reports the first mismatching event and hashes. | G2:L86-L87 |
| DG002-G2-027 | E-R | A cloud-opaque or body-GC'd step that cannot be reexecuted must never produce a fabricated `Verified`; it reports the lower guarantee explicitly. | G2:L88-L90, L151-L154 |
| DG002-G2-028 | E-P | The reconciled production emission point is `run_subcommand.rs::run_async` exit via `emit_run_artifact`, mirroring `--export-ssb`, using a clone of the live `wiring.ssb` Arc. | G2:L141-L150, L176, L189-L191 |
| DG002-G2-029 | E-P | A completed run writes `workflow/sessions/{id}/run.rra` and exactly one `run_artifact_emitted` row. | G2:L92-L95, L146-L150 |
| DG002-G2-030 | E-P | RRA emission is default-on; the sole runtime switch is kill switch `ELAI_RRA_DISABLE`, and `ELAI_RRA_ENABLE` is forbidden. | G2:L94-L95, L151-L155, L188 |
| DG002-G2-031 | E-P | CLI surfaces are `elai run --emit-rra` (force emit), `elai rra show <path>`, and `elai replay --verify <path>`, all supporting harness-convention `--json`. | G2:L97-L100 |
| DG002-G2-032 | E-P | `rra show` renders seed, pins, gate hashes, guarantee level, and root; replay prints `Verified`, `Divergence@event`, or `Tampered`. | G2:L97-L100, L166-L168 |
| DG002-G2-033 | E-P | `RunArtifactEmittedPayload` contains `root_blake3`, string `guarantee_level`, `gate_count`, `seed`, and `session_id`. | G2:L102-L104 |
| DG002-G2-034 | E-P | `RunArtifactEmitted` requires SQL spelling/parsing/CHECK, payload round trip, golden/action-type lock, and a named baseline-plus-one schema bump. | G2:L102-L104, L198 |
| DG002-G2-035 | E-P | `RraViewerPanel.svelte` plus `commands_g2.rs::get_run_artifact` renders manifest fields and the replay verdict and must pass cross-language drift checking. | G2:L106-L108, L160-L162 |
| DG002-G2-036 | E-D | Resume from the RRA stream is an optional, separately gated follow-on that closes `recover_from_checkpoint`; it is not part of G-2 DONE. | G2:L110-L112, L217-L218, L270 |
| DG002-G2-037 | E-P | `G-2-decisions.md` records guarantee levels, cloud/body capture limits, Merkle coverage, “reproduced not correct,” and the resume-vs-BSP correction. | G2:L114-L118 |
| DG002-G2-038 | E-P | Produced APIs are `RunArtifact`, `GuaranteeLevel`, `build_rra`, `write_rra`, `verify_rra`, `ReplayVerdict`, `RunArtifactEmitted`, `RunArtifactEmittedPayload`, CLI `rra_cmd::run`, `get_run_artifact`, and `RraViewerPanel`. | G2:L120-L132 |
| DG002-G2-039 | E-P | Consumed production surfaces are live `Ssb`, three named F payloads, `run_async`, determinism pins, and CLI dispatch. | G2:L133-L150 |
| DG002-G2-040 | E-P | Production code must use BLAKE3 only for roots/event hashes and contain no `.unwrap()`/`.expect()` on RRA paths. | G2:L70, L82, L155-L159 |
| DG002-G2-041 | E-P | No dependency is assumed beyond in-tree `rmp-serde`, compression, and BLAKE3; zstd requires explicit verification and user confirmation. | G2:L58-L59, L157-L158, L200, L266-L267 |
| DG002-G2-042 | E-G | The live observable runs a fixture with `run --emit-rra`, then verifies the generated session artifact. | G2:L163-L168 |
| DG002-G2-043 | E-G | That run must leave a live `run_artifact_emitted` row carrying root, guarantee level, and gate count. | G2:L164-L165 |
| DG002-G2-044 | E-G | The artifact must exist, show seed/pins/verdicts, return verified or first divergence as appropriate, and change to `Tampered` after a one-byte mutation. | G2:L166-L168 |
| DG002-G2-045 | E-G | DONE requires `wiring_contract_check.py --run-trigger` and a real artifact/event/verdict; cargo-green alone is insufficient. | G2:L170-L183, L185-L187 |
| DG002-G2-046 | E-G | Default-on behavior must be proven with no override and suppression with `ELAI_RRA_DISABLE=1`. | G2:L188 |
| DG002-G2-047 | E-G | Production-entry acceptance is the reconciled `run_async` exit path plus real CLI paths, not test-only and not the stale dispatcher anchor. | G2:L189-L191, L202-L207 |
| DG002-G2-048 | E-G | A local seeded fixture must verify, a cloud fixture must remain `VerifyDivergence`-checked, and a byte tamper must be `Tampered`. | G2:L192-L193 |
| DG002-G2-049 | E-G | Dashboard-smoker must prove the manifest and verdict through `RraViewerPanel`. | G2:L194, L208-L210 |
| DG002-G2-050 | E-G | Composition acceptance requires reading, not reimplementing, F-1/F-4/F-9 verdict fields for a run that exercised them. | G2:L195-L196, L219-L220 |
| DG002-G2-051 | E-G | The root must recompute byte-identically from the SSB stream under the pinned seed. | G2:L197 |
| DG002-G2-052 | E-G | Completion also requires payload round trip/golden lock, workspace cargo check, auditor passes for orchestrator/CLI/contracts, and no unverified dependency. | G2:L198-L200 |
| DG002-G2-053 | E-G | The round-trip contract builds from a seeded fixture, recomputes the root, round-trips the file, and confirms F-1/F-4/F-9 manifest entries. | G2:L222-L225 |
| DG002-G2-054 | E-G | Replay contract cases are local `Verified`, byte-tampered `Tampered`, cloud `VerifyDivergence`-checked, and injected event-N `Divergence`. | G2:L226-L228 |
| DG002-G2-055 | E-G | A smoke test must drive the real run-end hook and observe both `run.rra` and the event row. | G2:L229-L230 |
| DG002-G2-056 | E-R | Capture completeness is explicitly unresolved for response bodies retained outside hashed SSB rows and bounds re-execution claims. | G2:L260-L263 |
| DG002-G2-057 | E-R | The root must cover every stream/manifest/pin byte so any mutation is detectable. | G2:L268-L269 |
| DG002-G2-058 | E-P | Downstream composition is G-5 per-hunk evidence, G-6 replayable benchmarks, G-8 signed predicates, and G-9 export-safety precondition. | G2:L272-L276 |

### G-6 — SAFEDELTA competitive safety-delta benchmark

**Status boundary:** G-6 is planned/deferred behind F-2b LIVE and G-2. Its in-process static ablation substrate is asserted shipped but never run live; dynamic/live-model/competitor claims remain RunPod-gated.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DG002-G6-001 | E-D | G-6 is a planned Wave-3, multi-session benchmark deferred behind F-2b LIVE and G-2; its live model/task arm is RunPod-gated. | G6:L1-L3, L14-L16 |
| DG002-G6-002 | E-P | SAFEDELTA’s primary design is same-model gate-ON versus gate-OFF ablation on git-stripped held-out tasks. | G6:L5-L6, L31-L33 |
| DG002-G6-003 | E-P | The target metric is the gate’s marginal residual-insecure rate that reaches a human reviewer. | G6:L5-L6, L31-L32 |
| DG002-G6-004 | E-P | Competitor-harness head-to-head comparison is secondary to the ELAI ablation. | G6:L6, L11-L16 |
| DG002-G6-005 | E-S | `SecCodePltRunner` is asserted to already compute `insecure_rate_before`, residual `insecure_rate_after`, and `gate_recall` over an embedded CWE corpus. | G6:L7-L10, L18-L21, L33-L38 |
| DG002-G6-006 | E-S | The existing `InsecureRateMeasured` static-gate beacon fires only inside a real `elai bench seccodeplt` invocation and is explicitly asserted never to have run live. | G6:L9-L11, L20-L21, L69-L74 |
| DG002-G6-007 | E-P | G-6 generalizes that static fixed-corpus ablation into a live model/task, dynamic-exploit, anti-gaming, multi-suite matrix. | G6:L11-L13, L41-L46 |
| DG002-G6-008 | E-P | The report headline is “fewer dynamically exploitable diffs per 100 tasks reach a reviewer,” with 95% bootstrap CI and per-CWE breakdown. | G6:L12-L14, L46-L48 |
| DG002-G6-009 | E-P | SAFEDELTA must never be framed as a SWE-bench/model-capability rank, a “beats/#1” claim, or a perfect-code promise. | G6:L13-L16, L47-L48, L164-L170, L207-L208 |
| DG002-G6-010 | E-P | G-6 consumes F-2b’s static/dynamic floor and does not redefine F-2. | G6:L18-L24, L69-L72, L237-L238 |
| DG002-G6-011 | E-P | Each benchmark run is wrapped by G-2’s RRA so the delta and signed manifest are independently replayable. | G6:L23-L24, L81, L123-L127 |
| DG002-G6-012 | E-P | G-6 may proceed in parallel with G-3/G-4 but not concurrently with edits to `seccodeplt_runner.rs` or bench CLI dispatch. | G6:L25-L26 |
| DG002-G6-013 | E-S | The real existing bench route is `args_bench.rs` → `bench_cmd.rs` → per-suite files; `crates/cli/src/benchmark.rs` does not exist. | G6:L50-L58, L75-L77 |
| DG002-G6-014 | E-S | `parse_bench_seccodeplt_args` accepts only `--json`, not `--suite`; only `run_tier_eval` supplies an existing `--suites` pattern. | G6:L53-L64, L75-L77 |
| DG002-G6-015 | E-S | `InsecureRateMeasured` is a FLAT beacon, while `run_bench_ablation` supplies an existing gate-on/off dashboard precedent. | G6:L58-L64, L73-L80 |
| DG002-G6-016 | E-P | Matrix comparisons pin `ELAI_SEED` and `HardwareClass` and are valid only within the same hardware class. | G6:L22, L43, L61-L62, L78 |
| DG002-G6-017 | E-S | The source records live baselines `SCHEMA_VERSION=205` and `TOTAL_VARIANTS=482`; an E-45 assertion of 394 is pre-existing stale state, not a G-6 regression. | G6:L65-L67, L82, L277-L278 |
| DG002-G6-018 | E-P | `run_safedelta(suites, arms, seed, hw) -> SafeDeltaReport` is the new runner API. | G6:L94-L103 |
| DG002-G6-019 | E-P | The suite portfolio explicitly includes SUSVIBES-200. | G6:L264-L267 |
| DG002-G6-020 | E-P | The suite portfolio explicitly includes BaxBench-392 and treats it as dynamic exploit execution. | G6:L41-L42, L264-L267 |
| DG002-G6-021 | E-P | The suite portfolio reuses the embedded SeCodePLT family for its static tier. | G6:L99-L103, L264-L267 |
| DG002-G6-022 | E-P | The suite portfolio explicitly includes AgentDojo. | G6:L264-L267 |
| DG002-G6-023 | E-D | MSB is an optional suite rather than a required done-gate suite. | G6:L264-L267 |
| DG002-G6-024 | E-P | `Arm::ElaiGateOn` is a matrix arm. | G6:L96-L99 |
| DG002-G6-025 | E-P | `Arm::ElaiGateOff` is a matrix arm. | G6:L96-L99 |
| DG002-G6-026 | E-P | `Arm::CursorFrozenModel` is a matrix arm. | G6:L96-L99 |
| DG002-G6-027 | E-P | `Arm::ClaudeCode` is a matrix arm. | G6:L96-L99 |
| DG002-G6-028 | E-P | `Arm::Copilot` is a matrix arm. | G6:L96-L99 |
| DG002-G6-029 | E-P | `Arm::Cline` is a matrix arm. | G6:L96-L99 |
| DG002-G6-030 | E-P | `SafeDeltaReport` carries per-suite and per-CWE cells, delta/CI, and before/after dynamically exploitable counts. | G6:L100-L102 |
| DG002-G6-031 | E-P | `safety_delta` is `(before - after)` exploitable diffs per 100 tasks and is paired with a 95% bootstrap interval. | G6:L100-L103 |
| DG002-G6-032 | E-P | Tier 1 is the in-process static gate over embedded/held-out corpus; Tier 2 is dynamic exploit execution plus live model and competitor harnesses behind RunPod. | G6:L102-L103, L235-L236 |
| DG002-G6-033 | E-P | Task checkouts remove `.git` so models cannot inspect git history or the known fix. | G6:L105-L106 |
| DG002-G6-034 | E-P | A cheat detector flags completions that read git history or held-out answers. | G6:L105-L107 |
| DG002-G6-035 | E-P | Train and evaluation CWEs must be disjoint held-out splits. | G6:L107-L108 |
| DG002-G6-036 | E-P | A dynamic-suite diff counts insecure only when its exploit actually fires in the sandbox; static-pattern-only scoring is forbidden. | G6:L107-L109, L164-L167 |
| DG002-G6-037 | E-P | SAFEDELTA’s own self-test PASS is bound to F-4 `TestOutcomePayload.stdout_hash` so fabricated PASS claims are detectable. | G6:L111-L113, L168 |
| DG002-G6-038 | E-P | The report must state that a successful replay is `REPRODUCED not RIGHT`. | G6:L111-L113, L182-L183 |
| DG002-G6-039 | E-P | `safety_delta_measured` may follow the FLAT-beacon precedent or a two-arm Pattern-2 if a discriminator is needed; the choice is explicitly deferred to `G-6-decisions.md`. | G6:L115-L119, L133-L139 |
| DG002-G6-040 | E-P | `SafetyDeltaMeasuredPayload` fields are `suite_set`, `model_id`, `safety_delta`, `ci_low`, `ci_high`, `n_tasks`, `seed`, `corpus_blake3`, and `gate_recall`. | G6:L118-L120 |
| DG002-G6-041 | E-P | If added on the stated baseline, the event performs SQL/parse/CHECK/round-trip/golden work and bumps schema 205→206 and total variants 482→483. | G6:L120-L121 |
| DG002-G6-042 | E-P | `elai bench safedelta` is a new bench sub-verb with its own parser, `CliAction`, command wrapper, and main dispatch, not an option on SeCodePLT. | G6:L123-L127 |
| DG002-G6-043 | E-P | The SAFEDELTA parser accepts a suite list following `run_tier_eval` convention. | G6:L123-L126 |
| DG002-G6-044 | E-P | The production bench route emits the measurement row and a signed replayable G-2 manifest. | G6:L126-L127, L160-L163 |
| DG002-G6-045 | E-P | `SafeDeltaPanel.svelte` plus `commands_g6.rs::get_safety_delta_latest` displays per-suite/per-CWE deltas and 95% CI and passes cross-language checking. | G6:L129-L131, L174-L176 |
| DG002-G6-046 | E-P | The decision record must preserve ablation framing, suite/citation roles, anti-gaming controls, the CyberSecEval correction, event-shape choice, deferred claims/CI, and stale-E45 status. | G6:L133-L139 |
| DG002-G6-047 | E-P | Produced identifiers are `run_safedelta`, `SafeDeltaReport`, `SafetyDeltaMeasured`/payload, `bench_safedelta::run`, `parse_bench_safedelta_args`, `get_safety_delta_latest`, and `SafeDeltaPanel`. | G6:L141-L150 |
| DG002-G6-048 | E-P | Consumed identifiers are `gate_blocks`, SeCodePLT `run/from_embedded`, determinism seed/class, F-4 stdout hash, the static event precedent, bench parsers, and the future G-2 wrapper. | G6:L151-L159 |
| DG002-G6-049 | E-R | Live/dynamic/competitor arms must carry `runpod_pending` rather than an estimated or advertised live-safety result. | G6:L169-L170, L209-L210 |
| DG002-G6-050 | E-P | BLAKE3 is the only allowed corpus hash; BLAKE2/SHA2 are forbidden. | G6:L171, L192-L193 |
| DG002-G6-051 | E-P | The insecure-code citation must use CyberSecEval 1/2 arXiv:2404.13161, not offensive-cyber CyberSecEval 3 arXiv:2408.01605. | G6:L172-L173, L269-L271 |
| DG002-G6-052 | E-G | The Tier-1 observable is `elai bench safedelta --suites seccodeplt --seed 42` with no environment override. | G6:L177-L179 |
| DG002-G6-053 | E-G | The live row must carry delta, CI, task count, BLAKE3 corpus digest, seed, and recall, and the delta must recompute from per-task before/after rows. | G6:L180-L181 |
| DG002-G6-054 | E-G | The artifact must be signed/replayable once G-2 exists, and the panel/report must expose suite/CWE results, CI, and reproduced-not-right framing. | G6:L182-L183 |
| DG002-G6-055 | E-G | Tier-1 DONE requires the wiring checker to run the static ablation and emit a real measurement; cargo-green alone is insufficient. | G6:L185-L201 |
| DG002-G6-056 | E-G | The measured gate is default-on with no enable flag even though the benchmark itself is explicitly invoked. | G6:L202-L203, L220-L223 |
| DG002-G6-057 | E-G | The CLI must be a real production sub-verb and the panel must pass dashboard-smoker. | G6:L204-L205 |
| DG002-G6-058 | E-G | The delta must be journal-recomputable under the pinned seed. | G6:L206 |
| DG002-G6-059 | E-G | Report serialization is contract-tested to contain no `rank`, `#1`, or `beats` framing. | G6:L207-L208 |
| DG002-G6-060 | E-G | Tier-1 must be explicitly labelled static/synthetic, while pending dynamic/competitor arms preserve honest status. | G6:L209-L210 |
| DG002-G6-061 | E-G | Anti-gaming acceptance proves `.git` removal, cheat detection, and disjoint CWE splits. | G6:L211-L212 |
| DG002-G6-062 | E-G | SSB acceptance requires payload round-trip/action lock and the stated 205→206/482→483 bump. | G6:L213-L214 |
| DG002-G6-063 | E-G | Completion includes workspace cargo check and auditor passes; Tier-1 adds no dependency and every Tier-2 dependency is verified and feature-gated. | G6:L215-L217 |
| DG002-G6-064 | E-G | Static-contract cases compare against SeCodePLT before/after, recompute under seed 42, reject rank framing, and round-trip the chosen event shape. | G6:L240-L243 |
| DG002-G6-065 | E-G | Anti-gaming contracts cover `.git` removal, cheat detection, disjoint splits, and the rule that an unfired dynamic exploit is not insecure. | G6:L244-L247 |
| DG002-G6-066 | E-R | Bootstrap methodology, adequate power for 200–392 tasks/small CWE cells, suite pooling, and cross-harness comparability remain unresolved before external claims. | G6:L281-L285 |
| DG002-G6-067 | E-R | Competitor arms require matched conditions and must record inaccessible harnesses as `not-run`, never estimated. | G6:L286-L287 |
| DG002-G6-068 | E-D | No external safety-delta claim may ship before the F-2b single-harness dynamic floor is measured. | G6:L288-L289 |
| DG002-G6-069 | E-P | G-6’s residual-CWE output feeds G-5, its PASS binds through F-4, and its publishable replayability composes with G-2. | G6:L295-L299 |

### G-8 — signed in-toto provenance attestation and Article-50 verifier

**Status boundary:** the header says `PLANNED`, but corrected blocks assert that G-2’s manifest, the trailer key, run-exit emitter, and verify CLI already shipped. Older reads/W-7 prose still says `rra.rs` is absent. The conflict ledger preserves both strata.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DG002-G8-001 | E-P | G-8 is planned Wave 2, depends on G-2’s shared manifest, and extends F-4 anti-fabrication evidence off-machine. | G8:L1-L3, L31-L36 |
| DG002-G8-002 | E-P | It emits a custom in-toto Statement-v1 predicate for a committed run. | G8:L5-L10, L31-L40 |
| DG002-G8-003 | E-P | Predicate data includes subject BLAKE3, model ID, `ELAI_SEED`, verified-level rollup, evidence hashes, and SSB event ID. | G8:L5-L7, L99-L105 |
| DG002-G8-004 | E-P | The `verifiedLevels` rollup follows SLSA-VSA `PASSED` semantics over every consumed F-gate verdict. | G8:L5-L7, L34-L36, L99-L104 |
| DG002-G8-005 | E-P | The predicate subject is the staged tree/files digest, not the commit SHA, to avoid a self-referential commit hash. | G8:L36-L38, L103-L105, L175, L296-L297 |
| DG002-G8-006 | E-P | The commit contains only an attestation digest trailer; the full predicate is not embedded in commit text. | G8:L7-L8, L37-L38, L108-L113 |
| DG002-G8-007 | E-P | The full JSON sidecar path is `workflow/sessions/{id}/attestation.intoto.json`. | G8:L108-L112 |
| DG002-G8-008 | E-P | Attestation emission produces one `provenance_attestation_emitted` SSB action. | G8:L111-L112, L121-L125 |
| DG002-G8-009 | E-P | Default signing is an offline local BLAKE3-Merkle proof with no new dependency. | G8:L8-L9, L39-L40, L105-L106 |
| DG002-G8-010 | E-D | Cosign-keyless/Rekor signing is an opt-in, feature-gated, anti-slopsquatting-verified follow-on rather than the default done gate. | G8:L8-L9, L23-L24, L127-L130, L242-L243 |
| DG002-G8-011 | E-P | `elai verify-attestation <commit>` verifies offline and prints `verified`/`not-verified` without rerunning ELAI. | G8:L9-L10, L115-L119 |
| DG002-G8-012 | E-P | Compliance wording is restricted to “designed to support Article-50 transparency marking”; “certified compliant” is forbidden. | G8:L10-L12, L42-L46, L137-L142 |
| DG002-G8-013 | E-P | The cited transparency penalty is Article-99 EUR 15M/3%, never Article-5 EUR 35M/7%. | G8:L12-L13, L42-L46 |
| DG002-G8-014 | E-R | Source code is not asserted to be an enumerated Article-50 example, and timeline/posture wording must be refreshed from the named posture authority. | G8:L11-L12, L42-L46, L273-L279 |
| DG002-G8-015 | E-S | Existing commit substrate is asserted as `run_git_commit`, `format_commit_message`, `create_commit`, and `latest_commit_hash`. | G8:L16-L17, L48-L50 |
| DG002-G8-016 | E-S | Existing evidence fields are F-1 `source_blake3`, F-4 `stdout_hash`, and F-9 `dep_name_hash`, all consumed rather than reimplemented. | G8:L17-L19, L54-L58, L73-L77 |
| DG002-G8-017 | E-S | BLAKE3 1.8 is asserted in the workspace; cosign/sigstore/Rekor/in-toto dependencies are absent from manifests. | G8:L19-L24, L59-L60, L78-L80 |
| DG002-G8-018 | E-S | Corrected 2026-06-04 state says G-2’s `RunArtifact`, `GateVerdictHash`, and `extract_gate_manifest` shipped and must be reused. | G8:L21-L24, L65-L66, L84-L85 |
| DG002-G8-019 | E-P | G-8 is parallel-safe with G-3/G-4/G-9 but not with G-2 or simultaneous edits to `git_commit.rs`/the SSB write path. | G8:L25-L26 |
| DG002-G8-020 | E-P | The feature lets an external security team verify recorded gate passage from one committed artifact without rerunning the original run. | G8:L31-L36 |
| DG002-G8-021 | E-S | At the older ground-truth stratum, commit formatting had only a negative attribution-trailer strip filter and no positive E-41 trailer writer. | G8:L48-L53, L68-L72 |
| DG002-G8-022 | E-P | The provenance trailer must be appended after that strip filter or explicitly allow-listed so it survives. | G8:L108-L110, L292-L295 |
| DG002-G8-023 | E-S | `TestIntegrityEvent` is the existing two-arm F-4 anti-fabrication precedent; `ProvenanceTagged` is the SSB provenance precedent. | G8:L54-L65, L73-L83 |
| DG002-G8-024 | E-S | Corrected live counters are `SCHEMA_VERSION=213` and `TOTAL_VARIANTS=490`, superseding earlier 205/482 estimates. | G8:L66, L84-L85 |
| DG002-G8-025 | E-P | `InTotoPredicate` fields are `subject_blake3`, `model_id`, `elai_seed`, `verified_levels`, `evidence_hashes`, and `ssb_event_id`. | G8:L99-L102 |
| DG002-G8-026 | E-P | Each `GateVerdict` carries `gate`, boolean `passed`, and `verdict_blake3`. | G8:L100-L102 |
| DG002-G8-027 | E-P | `build_attestation` reads the session’s G-2 manifest/F-gate payload rollup and binds the subject to staged content. | G8:L102-L105, L155-L160 |
| DG002-G8-028 | E-P | `sign_local_merkle` returns `MerkleProof` using BLAKE3 and no new dependency. | G8:L105-L106 |
| DG002-G8-029 | E-P | Attestation production code must have no `.unwrap()`/`.expect()`. | G8:L105-L106 |
| DG002-G8-030 | E-P | Run-end commit formatting adds a digest-only provenance trailer with no AI attribution text. | G8:L108-L113, L176 |
| DG002-G8-031 | E-P | Attestation emission is default-on; `ELAI_ATTESTATION_DISABLE` is the sole kill switch and an ENABLE flag is forbidden. | G8:L111-L113, L177, L209-L212 |
| DG002-G8-032 | E-P | Offline verification reads trailer and sidecar, recomputes the local Merkle root, and rechecks each gate hash against recorded evidence. | G8:L115-L119 |
| DG002-G8-033 | E-P | CLI wiring adds `CliAction::VerifyAttestation`, parser arm, command module, and main dispatch following the shipped dep-check precedent. | G8:L115-L119 |
| DG002-G8-034 | E-P | `ProvenanceAttestationEmittedPayload` fields are `subject_blake3`, `merkle_root`, `gate_count`, `all_passed`, `model_id`, `seed`, and `signing` (`local-merkle` or `cosign-keyless`). | G8:L121-L124 |
| DG002-G8-035 | E-P | The action requires SQL spelling/parsing/CHECK, payload round trip, action lock, all-variant/golden regeneration, and cross-language synchronization. | G8:L121-L125, L241 |
| DG002-G8-036 | E-P | A future signing dependency must be verified for name, repository, version, publication date, and license before any import and guarded by `cfg(feature="cosign-keyless")`. | G8:L127-L130 |
| DG002-G8-037 | E-P | `AttestationDrawer.svelte` plus `commands_g8.rs::get_attestations`, under drawer key `attestation`, shows subject, root, gate rollup, all-passed state, and signing method. | G8:L132-L135, L180-L182 |
| DG002-G8-038 | E-P | `G-8-decisions.md` records standard freedom, staged-subject choice, digest-only filter handling, signing tiers, exact compliance wording, and F-4/G-2 composition. | G8:L137-L142 |
| DG002-G8-039 | E-P | Produced identifiers are `build_attestation`, `sign_local_merkle`, `InTotoPredicate`, the event/payload, verify command, Tauri getter, and drawer. | G8:L144-L153 |
| DG002-G8-040 | E-S | Corrected consumed identifiers include shipped `PROVENANCE_TRAILER_KEY = "ELAI-Provenance-Digest"`, G-2 manifest types/extractor, three F payloads, provenance precedent, shipped verify action, and workspace BLAKE3. | G8:L154-L163 |
| DG002-G8-041 | E-S | Corrected production run path is `run_async` → `emit_attestation` → `build_attestation` → sidecar plus SSB event, using live `rra_ssb` and inherited project ID. | G8:L164-L167 |
| DG002-G8-042 | E-S | Corrected commit binding is `format_commit_message` → `ELAI-Provenance-Digest`, and corrected verifier binding is `VerifyAttestation` → `verify_attestation_against_trailer`. | G8:L168-L170 |
| DG002-G8-043 | E-P | BLAKE3 is the only permitted Merkle/verdict digest family; BLAKE2 and SHA2 are forbidden. | G8:L178, L202 |
| DG002-G8-044 | E-P | The feature must not cite an E-41 trailer writer or fork/reimplement G-2’s manifest/F-gate rollup. | G8:L179 |
| DG002-G8-045 | E-G | The observable drives a committing fixture and then `elai verify-attestation <commit>` with no environment override. | G8:L183-L184 |
| DG002-G8-046 | E-G | The live SSB row must contain subject digest, Merkle root, gate count, all-passed flag, and signing method. | G8:L185 |
| DG002-G8-047 | E-G | The commit trailer and sidecar must exist; clean verification is offline `verified`, a tampered sidecar is `not-verified`, and the drawer shows the rollup. | G8:L186-L188 |
| DG002-G8-048 | E-G | DONE requires the wiring checker plus a real emitted artifact/event and real offline verdict; cargo-green alone is insufficient. | G8:L190-L210 |
| DG002-G8-049 | E-G | Default-on acceptance proves emission without override and suppression via `ELAI_ATTESTATION_DISABLE=1`. | G8:L211 |
| DG002-G8-050 | E-G | Both run-end/commit emission and the production verify CLI must be live, not test-only. | G8:L212-L213 |
| DG002-G8-051 | E-G | Dashboard-smoker must verify the per-gate rollup through `AttestationDrawer`. | G8:L214 |
| DG002-G8-052 | E-G | Clean offline verification must not rerun ELAI, while sidecar or trailer digest tampering must return `not-verified`. | G8:L215-L216 |
| DG002-G8-053 | E-G | Composition acceptance proves F-1/F-4/F-9 hashes are read for an exercised run and none of their gates are reimplemented. | G8:L217-L218 |
| DG002-G8-054 | E-G | A compliance contract enforces “designed to support,” EUR 15M/3%, and absence of “certified” and 35M/7%. | G8:L219-L220 |
| DG002-G8-055 | E-G | The offline default must add no dependency; any cosign path requires a recorded verified-dependency row and the workspace must build without that feature. | G8:L221-L222 |
| DG002-G8-056 | E-G | Event acceptance uses the corrected 213/490 live baseline and requires SSB round trip/action lock. | G8:L223-L224 |
| DG002-G8-057 | E-G | Completion includes workspace cargo check and auditor passes for orchestrator, CLI, tools, and contracts. | G8:L225 |
| DG002-G8-058 | E-P | A verified attestation means evidence was reproduced/passed the gates, not that the produced code is correct. | G8:L244-L246 |
| DG002-G8-059 | E-G | Round-trip tests cover G-2/F-1/F-4/F-9 rollup content, deterministic local Merkle recomputation, sidecar JSON, and event round trip. | G8:L248-L251 |
| DG002-G8-060 | E-G | Offline tests cover clean/tampered results, exact compliance text, and survival of the digest-only trailer through commit filtering. | G8:L252-L255 |
| DG002-G8-061 | E-R | Trailer placement versus allow-listing remains an authoring choice; accidental stripping would leave only the sidecar. | G8:L292-L295 |
| DG002-G8-062 | E-R | The exact staged-tree digest source remains to be confirmed at authoring. | G8:L296-L297 |
| DG002-G8-063 | E-D | Offline local Merkle carries DONE; cosign-keyless is the separately gated future path. | G8:L298-L299 |
| DG002-G8-064 | E-P | G-8 joins G-2 replayability and G-6 measurement into the signed, offline-verifiable trust spine; G-5 may display status. | G8:L304-L308 |

### G-9 — write-time journal/log redaction and retention

**Status boundary:** G-9 is planned and calls G-2 absent/forward-dependent, in conflict with G-8’s later “G-2 shipped” correction. Its own source corrects the summary’s per-session-database premise: only hook logs are per-session; `ssb.db` is one shared project database.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DG002-G9-001 | E-P | G-9 is a planned Wave-2, local-only, no-new-dependency redaction/retention feature and an export-safety precondition for G-2/G-8. | G9:L1-L3, L12-L15 |
| DG002-G9-002 | E-P | Redaction runs at both the sole SSB writer and the hook-log sink. | G9:L5-L7, L17-L21 |
| DG002-G9-003 | E-P | Every detected secret/PII span is transformed before persistence; post-write scrubbing is not the design. | G9:L5-L10, L30-L37 |
| DG002-G9-004 | E-P | Replacements are stable tokens so repeated occurrences correlate without revealing the original value. | G9:L6-L7, L35-L37 |
| DG002-G9-005 | E-P | The append-only `no UPDATE/DELETE` SSB invariant forces write-time mutation before INSERT. | G9:L7-L10, L32-L33 |
| DG002-G9-006 | E-R | A detector error fails closed by redacting the field instead of passing the raw value. | G9:L34-L36, L95-L101 |
| DG002-G9-007 | E-P | Detection reuses F-1 `vibegate_secrets.rs::SECRET_PATTERNS`/`SECRET_LABELS`; G-9 must not define a second regex set. | G9:L10-L12, L40-L44, L74-L75 |
| DG002-G9-008 | E-P | Presidio, ggshield, and cloud redaction services are forbidden dependencies/egress paths; the implementation is hand-rolled and local. | G9:L12-L15, L40-L44, L63-L64 |
| DG002-G9-009 | E-R | Regex/pattern detection is explicitly treated as high-recall/low-precision and its over-redaction rate is a release gate. | G9:L13-L15, L43-L44 |
| DG002-G9-010 | E-S | The asserted SSB chain is `write_payload` → `Ssb::write` → sole `write_loop` → `execute_write`, with payload bound between `BEGIN IMMEDIATE` and `COMMIT`. | G9:L17-L20, L46-L50 |
| DG002-G9-011 | E-S | The asserted hook sink is `log_skill_invocation`, appending JSON lines to per-session `hook-executions.log`; its read side is `get_hook_executions`. | G9:L20-L21, L52-L54 |
| DG002-G9-012 | E-S | Existing secret shapes include AWS keys, typed/generic PEM headers, and secret assignments; CWE mapping detail beyond those surfaces is verify-before-cite. | G9:L55-L58, L74-L75 |
| DG002-G9-013 | E-S | Stable-token precedents are `source_blake3_prefix` and `blake3::hash(label)`, with workspace BLAKE3 already present. | G9:L58-L60, L76 |
| DG002-G9-014 | E-S | The database is one shared `<base>/.elai_cc/sessions/ssb.db` with `session_id` as a column, not one database per session. | G9:L60-L63, L77-L78 |
| DG002-G9-015 | E-S | Only `hook-executions.log` is a per-session, file-granularity sink. | G9:L60-L63, L77-L78 |
| DG002-G9-016 | E-S | `run_migration` has a destructive schema-mismatch DROP, distinct from retention and ordinary append-only operation. | G9:L50-L52, L70-L71 |
| DG002-G9-017 | E-S | An optional wire-dark `secret_redacted` advisory follows existing env-gated event precedent; it is not required for the redaction itself. | G9:L62-L64, L79-L80 |
| DG002-G9-018 | E-P | G-9 may proceed alongside G-3/G-4/G-8 but must coordinate with G-2 and any edits to `execute_write`. | G9:L24-L25 |
| DG002-G9-019 | E-P | `redact(text) -> (String, u32)` returns transformed text plus redaction count. | G9:L93-L98 |
| DG002-G9-020 | E-P | Replacement syntax is `[REDACTED:<label>:<blake3-16>]`, derived from the secret value. | G9:L95-L99 |
| DG002-G9-021 | E-P | Identical secret values must map to identical replacement tokens. | G9:L97-L99 |
| DG002-G9-022 | E-P | `redact_payload(&mut SsbPayload)` walks and mutates all text-bearing payload fields in place. | G9:L99-L101, L299-L300 |
| DG002-G9-023 | E-P | Redaction production paths contain no `.unwrap()`/`.expect()`. | G9:L99-L101 |
| DG002-G9-024 | E-P | `execute_write` invokes `redact_payload` at its top, before payload binding/INSERT, so raw values never land in `ssb.db`. | G9:L103-L106 |
| DG002-G9-025 | E-P | Core redaction is default-on with the only kill switch `ELAI_REDACT_DISABLE`; `ELAI_REDACT_ENABLE` is forbidden. | G9:L105-L106, L126-L129 |
| DG002-G9-026 | E-P | The serialized hook `LogLine` is passed through `redact()` before append. | G9:L108-L109 |
| DG002-G9-027 | E-P | `purge_expired(base, ttl) -> PurgeReport` is the retention API. | G9:L111-L112 |
| DG002-G9-028 | E-P | An expired per-session hook log is deleted as a whole file. | G9:L111-L114 |
| DG002-G9-029 | E-P | Shared-SSB retention must choose and document either session-scoped maintenance-boundary row purge or whole-file TTL across all sessions. | G9:L113-L117, L292-L295 |
| DG002-G9-030 | E-P | Mid-session SSB UPDATE/DELETE is forbidden even for TTL. | G9:L115-L117, L166 |
| DG002-G9-031 | E-D | `SecretRedacted`/payload is optional and, if emitted, contains only count, sink, and session ID—never the secret. | G9:L119-L121, L143-L150 |
| DG002-G9-032 | E-D | Only the optional advisory may be wire-dark behind `ELAI_REDACT_ADVISORY_ENABLE`; core redaction cannot be env-enable-gated. | G9:L119-L124, L168-L169 |
| DG002-G9-033 | E-P | If the advisory ships on the stated baseline, it gets SQL/parse/CHECK/golden and schema 205→206/variants 482→483. | G9:L121-L124 |
| DG002-G9-034 | E-G | Over-redaction is measured across at least three real orchestrate runs before default-on rollout. | G9:L126-L129 |
| DG002-G9-035 | E-G | A ten-benign-payload corpus must show zero false redactions and byte-preserve benign payloads. | G9:L127-L129 |
| DG002-G9-036 | E-P | `RedactionPanel` plus `commands_g9.rs::get_redaction_stats` shows per-session counts/advisories but never secret values. | G9:L131-L133 |
| DG002-G9-037 | E-P | `G-9-decisions.md` records write-time necessity, detector reuse/no deps, stable tokens, honest shared-DB retention, citation/penalty corrections, FP rollout, and G-2/G-8 precondition status. | G9:L135-L141 |
| DG002-G9-038 | E-P | Produced identifiers are `redact`, `redact_payload`, `purge_expired`, the stats getter/panel, and optional `SecretRedacted` event/payload. | G9:L143-L150 |
| DG002-G9-039 | E-P | Consumed surfaces are `execute_write`, `write_payload`, F-1 patterns/labels, hash-prefix precedent, hook writer, DB-layout constants, and G-2 export path. | G9:L151-L158 |
| DG002-G9-040 | E-P | Production reachability is SSB writer → payload redaction → INSERT and hook writer → line redaction → append. | G9:L159-L162 |
| DG002-G9-041 | E-P | No redaction data may leave the machine because a redactor that phones home is itself considered a leak. | G9:L163-L165 |
| DG002-G9-042 | E-P | No new detector/regex set and no Presidio/ggshield dependency may be introduced. | G9:L164-L165 |
| DG002-G9-043 | E-P | A detection failure cannot silently pass through raw content. | G9:L167 |
| DG002-G9-044 | E-P | The implementation and documentation may not claim `ssb.db` is per-session. | G9:L170 |
| DG002-G9-045 | E-P | BLAKE3 is the only permitted redaction-token digest; BLAKE2/SHA2 are forbidden. | G9:L171, L195 |
| DG002-G9-046 | E-P | Refuted Presidio numerical claims must be omitted in favor of qualitative high-recall/low-precision wording. | G9:L172, L268-L275 |
| DG002-G9-047 | E-G | The observable fixture carries an AWS key and email through prompt/tool output, triggers hook logging, and runs with no override. | G9:L177-L179 |
| DG002-G9-048 | E-G | The persisted row must contain `[REDACTED:aws_key:<blake3-16>]`, and searching the DB must find no raw `AKIA` key. | G9:L180-L181 |
| DG002-G9-049 | E-G | The hook log must contain only tokens, expired hook logs must purge whole, the panel must show counts, and benign data must remain byte-identical. | G9:L182-L184 |
| DG002-G9-050 | E-G | DONE requires wiring-checker success plus real redaction at both sinks and an expired-log purge; cargo-green alone is insufficient. | G9:L186-L203 |
| DG002-G9-051 | E-G | Default-on acceptance proves no-override redaction and kill-switch suppression without any core ENABLE flag. | G9:L204-L205 |
| DG002-G9-052 | E-G | Production acceptance requires both `execute_write` and `log_skill_invocation`, not test-only paths. | G9:L206-L207 |
| DG002-G9-053 | E-G | Dashboard-smoker verifies count-only data through `RedactionPanel`. | G9:L208 |
| DG002-G9-054 | E-G | A database-after-commit proof must show the raw secret never landed, because later scrubbing is prohibited. | G9:L209-L210 |
| DG002-G9-055 | E-G | Stable-token tests require identical values to map identically and benign payloads to remain unchanged. | G9:L211-L212 |
| DG002-G9-056 | E-G | The FP gate requires zero errors on ten benign samples and a measured three-real-run rate. | G9:L213-L214 |
| DG002-G9-057 | E-G | TTL acceptance deletes an expired hook log whole and proves no live-row UPDATE/DELETE; shared-DB retention is labelled session-scoped/whole-file. | G9:L215-L216 |
| DG002-G9-058 | E-G | If shipped, the advisory must round-trip/golden-lock and perform the conditional 205→206/482→483 bump. | G9:L217-L218 |
| DG002-G9-059 | E-G | Completion includes workspace cargo check/auditor and zero new dependency. | G9:L219-L220 |
| DG002-G9-060 | E-P | The complete feature is Tier-1 in-process and genuinely fail-closed, with no network/subprocess/RunPod work. | G9:L239-L243 |
| DG002-G9-061 | E-G | Write-time tests cover AWS key plus email, DB raw-value absence, repeated-token stability, benign identity, and detector-error fail-closed behavior. | G9:L245-L249 |
| DG002-G9-062 | E-G | TTL contracts cover whole-hook-log purge, no mid-session row mutation, and the chosen shared-DB retention mode. | G9:L250-L252 |
| DG002-G9-063 | E-R | The primary failure mode is corrupting real benign payloads by over-redaction; anchored credential-shape patterns plus measured FP gates are the mitigation. | G9:L287-L291 |
| DG002-G9-064 | E-R | Missing any text-bearing field in any payload variant is explicitly a leak hole and requires cross-variant contract coverage. | G9:L299-L300 |
| DG002-G9-065 | E-P | Once present, G-9 makes RRA/attestation export safer and establishes F-1’s detector as the single detection source for block and persistence-redaction decisions. | G9:L302-L306 |

### G-10 — model-drift regression canary

**Status boundary:** G-10 is explicitly a planned post-G-9 extension, not an original R1–R9 item. It depends on G-2, but one prerequisite sentence says G-2 is shipped/co-shipped while the existing-state block calls `RunArtifact` planned. Its proposed run-end anchor also collides with G-2’s correction that `build_production_hook_dispatcher` is not a run-exit artifact hook.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DG002-G10-001 | E-P | G-10 is a planned Wave-3 post-G-9 extension added 2026-06-02, not an R1–R9 roadmap initiative. | G10:L1-L3, L19-L26 |
| DG002-G10-002 | E-P | It monitors exogenous provider checkpoint/local-weight changes, a seam not covered by endogenous ELAI regressions. | G10:L5-L9, L42-L52 |
| DG002-G10-003 | E-P | Model identity is the `(model, fingerprint, lane)` tuple. | G10:L9-L10, L54-L58 |
| DG002-G10-004 | E-P | A small fixed canary set has baseline scores keyed to model identity. | G10:L9-L11 |
| DG002-G10-005 | E-P | Identity change or a schedule triggers a canary rerun. | G10:L10-L11 |
| DG002-G10-006 | E-P | Evaluation compares each canary family with the last-good baseline and computes a regression delta. | G10:L11-L12 |
| DG002-G10-007 | E-P | Regression emits `model_drift_suspected` and routes matching `ReasoningTemplate` IDs to review. | G10:L12-L13 |
| DG002-G10-008 | E-P | Detection and surfacing are the default product boundary; model auto-swap/revert is excluded. | G10:L13, L64-L67 |
| DG002-G10-009 | E-P | Local identities based on weights/version can be labelled fingerprint-confirmed when the canary also regresses. | G10:L13-L16, L60-L64 |
| DG002-G10-010 | E-P | Cloud identities with opaque `system_fingerprint` can only be labelled behavioral-drift-suspected. | G10:L14-L16, L60-L64 |
| DG002-G10-011 | E-P | A local false alarm must not automatically swap models because false remediation is considered worse than advisory drift. | G10:L64-L67 |
| DG002-G10-012 | E-S | Gap analysis asserts no model-version pin type exists and all existing canary/regression symbols key on endogenous changes. | G10:L19-L26, L42-L48 |
| DG002-G10-013 | E-S | `ModelSwap` and `TierModelSwap` exist but are wire-dark with no production emitter. | G10:L23-L25, L50-L52, L75-L77 |
| DG002-G10-014 | E-S | `autonomous_gate::substitute_model` is restart/VRAM-triggered, not regression-triggered. | G10:L24-L26, L49-L50, L78-L79 |
| DG002-G10-015 | E-P | External practitioner discourse is corroborative motivation only and not a requirement. | G10:L19-L23 |
| DG002-G10-016 | E-P | G-2 `RunArtifact` identity pins are a hard dependency or co-ship input. | G10:L28-L29, L73-L74 |
| DG002-G10-017 | E-P | D-7 `canary.rs` supplies `CanaryFamily`, `FamilyEvalResult`, delta, sample-count threshold, and non-determinism band and must be reused. | G10:L29-L31, L69-L72 |
| DG002-G10-018 | E-P | Existing `held_out::TaskId` fixtures supply the fixed canary tasks. | G10:L31, L84 |
| DG002-G10-019 | E-P | F-4 `TestOutcomePayload.stdout_hash`/deterministic pass rate supplies the malformed-output-spike signal. | G10:L31-L33, L82-L83 |
| DG002-G10-020 | E-P | D-1’s `ReasoningTemplate` archive/domain is the review/revert target. | G10:L33-L34, L80-L81 |
| DG002-G10-021 | E-P | G-10 may proceed with G-5/G-8/G-9, but not with G-2 or concurrent `canary.rs` edits. | G10:L35-L36 |
| DG002-G10-022 | E-S | Existing endogenous examples are F-10 security monotonicity, self-mod/staged eval, D-7 optimized-template transfer, and goal-Jaccard drift. | G10:L42-L48 |
| DG002-G10-023 | E-P | G-10 is composition: G-2 provides identity input, D-7 measurement, D-1 review target, and F-4 malformed-output evidence. | G10:L54-L58 |
| DG002-G10-024 | E-P | `ModelIdentity` fields are `model_pin`, optional `system_fingerprint`, `provider`, and `lane`. | G10:L102-L104 |
| DG002-G10-025 | E-P | `Lane` states are `Local` and `Cloud`. | G10:L103-L104 |
| DG002-G10-026 | E-P | `identity_key()` is BLAKE3 over the complete model/fingerprint/provider/lane tuple. | G10:L103-L105 |
| DG002-G10-027 | E-P | `DriftBaseline` stores identity key, per-family scores, capture time, and per-output BLAKE3 hashes. | G10:L105-L106 |
| DG002-G10-028 | E-P | `DriftSignal` variants are `FingerprintChanged`, `CanaryRegressed`, and `MalformedOutputSpike`. | G10:L106-L107 |
| DG002-G10-029 | E-P | `DriftVerdict` variants are `NoDrift`, `BehavioralDriftSuspected { signals, per_family_delta, affected_template_ids, confirmed_by_fingerprint }`, and `BaselineCaptured`. | G10:L107-L109 |
| DG002-G10-030 | E-P | `record_baseline` runs the fixed canary under the current model and `ELAI_SEED`, reuses family scoring, and persists scores/output hashes under the identity key. | G10:L111-L114 |
| DG002-G10-031 | E-P | `check_model_drift` compares current identity with baseline and records `FingerprintChanged`/confirmation only for the local lane. | G10:L111-L116 |
| DG002-G10-032 | E-P | The current canary reruns and uses D-7 sample-count/non-determinism logic; a delta within the band is not drift. | G10:L116-L117 |
| DG002-G10-033 | E-P | A seeded F-4 deterministic pass-rate window with malformed-output spike adds `MalformedOutputSpike`. | G10:L117-L118 |
| DG002-G10-034 | E-P | A beyond-band signal produces `BehavioralDriftSuspected`; affected templates are those whose domains match regressed families. | G10:L118-L120 |
| DG002-G10-035 | E-P | Cloud verdicts always set `confirmed_by_fingerprint=false`. | G10:L119-L120 |
| DG002-G10-036 | E-R | First observation of a model identity is a cold start returning `BaselineCaptured`, not drift. | G10:L326-L327 |
| DG002-G10-037 | E-P | At run completion, the runtime reads G-2 identity, compares provider/lane last-good state, calls the checker, and emits exactly one drift row on suspected drift. | G10:L122-L126 |
| DG002-G10-038 | E-P | Detection is default-on with only `ELAI_MODEL_DRIFT_DISABLE`; the auto-pin action remains off. | G10:L126-L127 |
| DG002-G10-039 | E-P | `elai model-drift baseline` captures or refreshes the current identity baseline. | G10:L129-L133 |
| DG002-G10-040 | E-P | `elai model-drift check` runs immediately and prints `NoDrift` or detailed suspected-drift output. | G10:L129-L133 |
| DG002-G10-041 | E-P | `elai model-drift show` displays stored baselines and the latest verdict. | G10:L129-L133 |
| DG002-G10-042 | E-P | All model-drift CLI surfaces support harness-convention `--json`. | G10:L133 |
| DG002-G10-043 | E-P | `ModelDriftSuspected` is described as one Pattern-2 action with `DriftSignal` as its discriminator. | G10:L135-L140 |
| DG002-G10-044 | E-P | `ModelDriftSuspectedPayload` contains `identity_key`, string signals, per-family deltas, affected-template IDs, fingerprint-confirmation flag, and baseline capture time. | G10:L135-L140 |
| DG002-G10-045 | E-P | The event requires SQL/parse/CHECK, payload round trip, golden/action lock, and a named baseline-plus-one schema bump. | G10:L139-L140 |
| DG002-G10-046 | E-P | `ModelDriftPanel.svelte` plus `get_model_drift_status` shows current identity, baseline/current bars, active signals, review queue, and honest lane badge. | G10:L142-L145 |
| DG002-G10-047 | E-D | Auto-pin-to-last-good is optional, outside DONE, default-off, human-gated, and only recommends feeding `substitute_model`. | G10:L147-L151 |
| DG002-G10-048 | E-D | Any auto-pin experiment requires wire-dark metadata `type=experiment`, owner, expiry, promotion-plan ID, and smoke test. | G10:L147-L151 |
| DG002-G10-049 | E-G | Auto-pin cannot enable until drift false positives are bounded on at least three real runs. | G10:L150-L151 |
| DG002-G10-050 | E-P | `G-10-decisions.md` records lane asymmetry, detection/action split, reused band, and compose-not-restate boundaries. | G10:L153-L157 |
| DG002-G10-051 | E-P | Produced identifiers are four model-drift types, baseline/check functions, event/payload, CLI command, status getter, and panel. | G10:L159-L171 |
| DG002-G10-052 | E-P | Consumed surfaces are D-7 family/result, held-out tasks, G-2 artifact, SSB/F-4 evidence, D-1 templates, runtime hook, and CLI dispatch. | G10:L172-L181 |
| DG002-G10-053 | E-P | Reachable CLI flows are baseline → `record_baseline` and check → verdict plus conditional event; production runtime and dashboard routes are also required. | G10:L182-L186 |
| DG002-G10-054 | E-P | Cloud drift may never fabricate `FingerprintConfirmed`; it remains behavioral with a false confirmation flag. | G10:L187-L190 |
| DG002-G10-055 | E-P | DONE contains no auto-swap/revert and no reimplementation of D-7 family/delta/band logic. | G10:L191-L193 |
| DG002-G10-056 | E-P | BLAKE3 is the only identity/output digest and production code contains no `.unwrap()`/`.expect()`. | G10:L194-L195 |
| DG002-G10-057 | E-P | No new dependency is allowed; canary, SSB, serialization, and BLAKE3 are all expected in-tree. | G10:L196, L238 |
| DG002-G10-058 | E-G | The observable first baselines fixture model A, then checks mock model B with a regressed canary output. | G10:L200-L203 |
| DG002-G10-059 | E-G | The drift row must include `CanaryRegressed`, local `FingerprintChanged`, family deltas, affected templates, and confirmation flag. | G10:L201-L203 |
| DG002-G10-060 | E-G | The CLI/panel must show suspected drift, while a clean model-A recheck inside the band returns `NoDrift` and emits no row. | G10:L202-L203 |
| DG002-G10-061 | E-G | Per-family delta must recompute byte-identically from the SSB stream under pinned `ELAI_SEED`. | G10:L203 |
| DG002-G10-062 | E-G | DONE requires the wiring checker, a real drift row, and a clean no-row FP guard; cargo-green alone is insufficient. | G10:L205-L222 |
| DG002-G10-063 | E-G | Default-on acceptance proves identity-change checking without override and suppression by `ELAI_MODEL_DRIFT_DISABLE=1`. | G10:L223-L224 |
| DG002-G10-064 | E-G | Production checking must fire through the named runtime path and the real model-drift CLI, not tests alone. | G10:L225-L226 |
| DG002-G10-065 | E-G | Local fixture acceptance requires both changed fingerprint and regressed canary with confirmation; cloud acceptance requires behavioral suspicion without confirmation. | G10:L227-L229 |
| DG002-G10-066 | E-G | Dashboard-smoker verifies deltas, signals, and the honest badge. | G10:L230 |
| DG002-G10-067 | E-G | Composition acceptance proves actual reuse of D-7, G-2, and F-4 identifiers rather than restatement. | G10:L231-L232 |
| DG002-G10-068 | E-G | The FP guard records rates on three real orchestrate runs before action enablement. | G10:L234-L235 |
| DG002-G10-069 | E-G | Event round trip/action lock, named schema bump, workspace cargo check, auditor passes, and zero new dependency are completion gates. | G10:L236-L238 |
| DG002-G10-070 | E-P | A drift delta is baseline-relative behavioral change, never an absolute model-capability rank. | G10:L256-L260 |
| DG002-G10-071 | E-G | Verdict contracts cover unchanged/within-band, local changed+regressed, cloud regressed, and F-4 malformed-spike cases. | G10:L262-L267 |
| DG002-G10-072 | E-G | FP contracts require no event within the band and exact seeded recomputation; a real-hook smoke test must emit on a swapped fixture identity. | G10:L268-L272 |
| DG002-G10-073 | E-R | The non-determinism band must be calibrated to avoid noise alarms; detection-only ships before any auto-pin. | G10:L318-L322 |
| DG002-G10-074 | E-R | Cloud scheduling needs a small affordable canary set; exact size trades cost against sensitivity. | G10:L323-L325 |
| DG002-G10-075 | E-R | If G-2 has not landed, identity pins may be read directly from determinism after path confirmation, though co-shipping after G-2 is preferred. | G10:L328-L330 |
| DG002-G10-076 | E-D | G-10 is not a model-selection/routing engine and not an automatic remediation loop. | G10:L331-L332 |
| DG002-G10-077 | E-D | A future bounded-FP auto-pin could become the first production emitter for wire-dark `ModelSwap`/`TierModelSwap`. | G10:L334-L339 |

## Exact identifier, schema, surface, flag, and trigger registry

This is a lossless index into the counted ledgers, not a second atomic inventory.

| Source | Produced types/functions/data | Consumed integration anchors | Events, commands, surfaces, flags, artifacts |
|---|---|---|---|
| G-2 | `RunArtifact`; `GuaranteeLevel::{ReproducibleTokens,ReproducibleActionStream,VerifyDivergence}`; `GateVerdictHash`; `build_rra`; `write_rra`; `verify_rra`; `ReplayVerdict::{Verified,Divergence,Tampered}`; `RunArtifactEmittedPayload`; `rra_cmd::run` | `Ssb`; `TestOutcomePayload.stdout_hash`; `VibeGateBlockedPayload.source_blake3`; `WriteTimeSlopsquatBlockedPayload.dep_name_hash`; eventual F-10 `delta_blake3`; `run_subcommand::run_async`; live `wiring.ssb`; determinism pins; CLI dispatch | `run_artifact_emitted`; `elai run --emit-rra`; `elai rra show`; `elai replay --verify`; `--json`; `RraViewerPanel`; `get_run_artifact`; `ELAI_RRA_DISABLE`; forbidden `ELAI_RRA_ENABLE`; `workflow/sessions/{id}/run.rra` |
| G-6 | `run_safedelta`; `SafeDeltaReport`; `SuiteResult`; `PerCweCell`; six `Arm` states; `SafetyDeltaMeasuredPayload`; `bench_safedelta::run`; `parse_bench_safedelta_args` | `SecCodePltRunner::{run,from_embedded,gate_blocks}`; `ELAI_SEED_ENV`; `HardwareClass`; `TestOutcomePayload.stdout_hash`; `InsecureRateMeasuredPayload`; `parse_bench_subcommand_args`; `run_tier_eval`; `run_bench_ablation`; G-2 RRA | `safety_delta_measured`; `elai bench safedelta --suites ... --seed ...`; `SafeDeltaPanel`; `get_safety_delta_latest`; signed replayable run manifest; honest `runpod_pending`; no gate ENABLE flag |
| G-8 | `InTotoPredicate`; `GateVerdict`; `build_attestation`; `sign_local_merkle`; `MerkleProof`; `ProvenanceAttestationEmittedPayload`; `verify_attestation_cmd::run` | G-2 `RunArtifact`/`GateVerdictHash`/`extract_gate_manifest`; F-1/F-4/F-9 hashes; `format_commit_message`; corrected `PROVENANCE_TRAILER_KEY`; `CliAction`; workspace BLAKE3; `ProvenanceTagged`/`TestIntegrityEvent` precedents | `provenance_attestation_emitted`; `ELAI-Provenance-Digest` (also stale `provenance-digest` spelling); `attestation.intoto.json`; `elai verify-attestation <commit>`; `verified`/`not-verified`; `AttestationDrawer`; `get_attestations`; `ELAI_ATTESTATION_DISABLE`; forbidden ENABLE flag; optional `cosign-keyless` feature |
| G-9 | `redact`; `redact_payload`; `purge_expired`; `PurgeReport`; optional `SecretRedacted`/payload; `get_redaction_stats` | `execute_write`; `write_payload`; `SECRET_PATTERNS`; `SECRET_LABELS`; `source_blake3_prefix`; `log_skill_invocation`; `SSB_SESSIONS_SUBDIR`; `default_ssb_path_for_base`; G-2 export path | optional `secret_redacted`; `[REDACTED:<label>:<blake3-16>]`; `RedactionPanel`; `ELAI_REDACT_DISABLE`; forbidden core ENABLE; optional `ELAI_REDACT_ADVISORY_ENABLE`; shared `ssb.db`; per-session `hook-executions.log` |
| G-10 | `ModelIdentity`; `Lane::{Local,Cloud}`; `DriftBaseline`; `DriftSignal::{FingerprintChanged,CanaryRegressed,MalformedOutputSpike}`; `DriftVerdict::{NoDrift,BehavioralDriftSuspected,BaselineCaptured}`; `record_baseline`; `check_model_drift`; `ModelDriftSuspectedPayload`; `model_drift_cmd::run` | `CanaryFamily`; `FamilyEvalResult`; `held_out::TaskId`; G-2 `RunArtifact`; `Ssb`; F-4 `TestOutcomePayload`; D-1 reasoning templates; named runtime hook; CLI dispatch; optional `substitute_model` | `model_drift_suspected`; `elai model-drift baseline/check/show`; `--json`; `ModelDriftPanel`; `get_model_drift_status`; `ELAI_MODEL_DRIFT_DISABLE`; forbidden detection ENABLE; optional wire-dark auto-pin experiment |

## Explicit cross-source interactions

These relationships are stated by the originals and reference counted atoms.

| Interaction | Explicit relationship | Atomic references |
|---|---|---|
| G-9 → G-2/G-8 | Write-time redaction is the export-safety precondition for handing RRA/attestation material to clients. | DG002-G9-001, DG002-G9-065, DG002-G2-058, DG002-G8-064 |
| G-2 → G-6 | Every SAFEDELTA run is wrapped in the RRA so the measured number is independently replayable. | DG002-G2-058, DG002-G6-011, DG002-G6-044 |
| G-2 → G-8 | G-8 reads and signs the shared `RunArtifact` evidence manifest rather than rebuilding F-gate evidence. | DG002-G2-058, DG002-G8-018, DG002-G8-027, DG002-G8-044 |
| G-2 → G-10 | G-10 reads `model_pin`/`system_fingerprint` from `RunArtifact` as model-identity input. | DG002-G2-003, DG002-G10-016, DG002-G10-023 |
| G-2 → G-5 | The RRA manifest is the explicit per-hunk evidence input to the reviewer cockpit. | DG002-G2-001, DG002-G2-058 |
| F-2b → G-6 | G-6 generalizes but does not replace the existing/static and pending/live dynamic floor. | DG002-G6-005, DG002-G6-010, DG002-G6-068 |
| F-4 → G-6 | SAFEDELTA binds its own PASS to captured stdout evidence. | DG002-G6-037, DG002-G6-069 |
| G-6 → G-5 | Residual per-CWE rates tell the cockpit which vulnerability classes still reach review. | DG002-G6-069 |
| F-1/F-4/F-9 → G-8 | The attestation rollup consumes their recorded BLAKE3 verdicts; F-4’s anti-fabrication evidence moves off-machine. | DG002-G8-016, DG002-G8-020, DG002-G8-027, DG002-G8-053 |
| G-8 → G-5 | The reviewer cockpit may show attestation verified/not-verified state for each change. | DG002-G8-064 |
| F-1 → G-9 | One `SECRET_PATTERNS` detector is reused for both pre-write blocking and persistence redaction. | DG002-G9-007, DG002-G9-065 |
| D-7 → G-10 | D-7 supplies the family scoring, delta, threshold, and non-determinism band. | DG002-G10-017, DG002-G10-023, DG002-G10-055 |
| F-4 → G-10 | F-4 pass-rate/stdout evidence contributes the malformed-output-spike signal. | DG002-G10-019, DG002-G10-033 |
| D-1 → G-10 | Regressed canary families map to ReasoningTemplates for review/revert rather than model auto-remediation. | DG002-G10-020, DG002-G10-034 |
| Shared SSB edit window | G-2/G-8/G-9 explicitly require coordination around SSB or commit/write-path changes; G-6 coordinates around benchmark files and G-10 around `canary.rs`. | DG002-G2-013, DG002-G6-012, DG002-G8-019, DG002-G9-018, DG002-G10-021 |

## Inference ledger (not explicit implementation facts)

| Atomic ID | Evidence | Inference | Basis / caution |
|---|---|---|---|
| DG002-I-001 | I | Delivery can implement G-2’s local artifact before G-9, but any external RRA release should wait for G-9. | G-2 is the keystone while both G-2 and G-9 call G-9 an export precondition; exact release sequencing is not specified. |
| DG002-I-002 | I | The intended signed SAFEDELTA manifest likely means a G-2 RRA plus G-8 attestation, because G-2 itself hashes but does not define a signature. | G6:L126-L127; G2:L74-L90; G8:L304-L308. Signing composition is not directly wired in G-6. |
| DG002-I-003 | I | G-2 roots, G-8 predicates, and G-10 baselines need a shared canonical identity/serialization convention to compare across machines. | Their schemas share seed/model/fingerprint/hash values, but no cross-artifact canonicalization contract is stated. |
| DG002-I-004 | I | G-9 should run before G-2/G-8 serialization at run exit, otherwise already-collected raw values could enter exported material despite later sink redaction. | The sources say redaction is pre-persistence and export-safe, but run-exit ordering is not explicitly diagrammed. |
| DG002-I-005 | I | Concurrent scheduled and run-end G-10 checks need coalescing/idempotency to avoid duplicate drift rows and canary cost. | Two trigger classes and one-row language are explicit; locking/deduplication is absent. |
| DG002-I-006 | I | The optional G-9 advisory cannot be the sole source for dashboard counts because it may be disabled while redaction remains on. | Core redaction is default-on and advisory wire-dark; the panel promises counts but the counter store is not fixed. |
| DG002-I-007 | I | A G-10 `FingerprintChanged` signal should probably be reserved for actual fingerprint change, not every `identity_key` change. | `identity_key` includes model/provider/lane, while the signal name is narrower; source does not resolve mapping. |
| DG002-I-008 | I | A local Merkle tree proves integrity only relative to a trusted root; calling it “signed” likely requires an unstated trust/key binding. | The source uses “signed” and `sign_local_merkle` but defines no secret/private key or external trust root. |

## Ambiguity and unresolved-contract ledger

| Atomic ID | Evidence | Ambiguous or unspecified contract | Exact source anchors |
|---|---|---|---|
| DG002-A-001 | A | G-2 does not define canonical msgpack ordering, schema migration/compatibility, stable float/string encoding, or event ordering needed for byte-identical roots. | G2:L74-L82, L197 |
| DG002-A-002 | A | The compression codec is intentionally unresolved; the source says in-tree codec and forbids assuming zstd without confirmation. | G2:L58-L59, L157-L158, L266-L267 |
| DG002-A-003 | A | Merkle leaf format, tree shape, domain separation, duplicate-leaf handling, and inclusion-proof format are absent. | G2:L81-L82, L268-L269 |
| DG002-A-004 | A | Criteria for `ReproducibleTokens` versus `ReproducibleActionStream` on a local run are not operationally defined. | G2:L38-L44, L77-L79 |
| DG002-A-005 | A | “Fresh recompute where inputs are present” does not define input discovery, unavailable-field status, or whether missing evidence fails verification. | G2:L84-L90 |
| DG002-A-006 | A | `SsbEventDigest` fields, action replay semantics, external side-effect sandboxing, and idempotent replay behavior are unspecified. | G2:L74-L87 |
| DG002-A-007 | A | The behavior for absent/empty model pins or provider fingerprints and guarantee-level downgrade rules is not defined. | G2:L74-L79, L260-L265 |
| DG002-A-008 | A | Default-on emission plus force-emit `--emit-rra` does not specify duplicate handling, overwrite policy, or what the force flag changes. | G2:L92-L100, L188 |
| DG002-A-009 | A | Artifact-write failure policy is absent: the source does not say whether the run fails, retries, leaves a partial file, or emits a failure event. | G2:L92-L95, L163-L168 |
| DG002-A-010 | A | The viewer’s replay verdict storage/source, refresh lifecycle, and behavior for an unverified artifact are unspecified. | G2:L106-L108 |
| DG002-A-011 | A | Resume-from-journal omits checkpoint mapping, irreversible-side-effect handling, exactly-once semantics, and crash recovery because it is deferred. | G2:L110-L112, L270 |
| DG002-A-012 | A | F-10’s `delta_blake3` is conditional “when it lands,” but manifest versioning and behavior for runs before/after that field are not stated. | G2:L18-L20, L52-L53 |
| DG002-A-013 | A | G-6 names suites but does not define the exact `Suite` enum, frozen corpus revision, task deduplication, or split manifest. | G6:L94-L103, L264-L267 |
| DG002-A-014 | A | `CursorFrozenModel` does not define the model/checkpoint/API version or how the freeze is enforced across competitor harnesses. | G6:L96-L99 |
| DG002-A-015 | A | Matched competitor conditions—budgets, prompts, tool permissions, retries, timeouts, hardware, and versions—remain unspecified. | G6:L286-L287 |
| DG002-A-016 | A | Dynamic exploit sandbox isolation, timeout, nondeterminism, success oracle, and safe handling of malicious exploits are not fixed. | G6:L41-L46, L105-L109 |
| DG002-A-017 | A | Cheat-detector algorithm, evidence, false-positive policy, and response to detected cheating are not defined. | G6:L105-L107, L244-L246 |
| DG002-A-018 | A | Bootstrap resampling unit, number of replicates, stratification, seeded determinism, pooling threshold, and multiplicity correction are unresolved. | G6:L100-L103, L281-L285 |
| DG002-A-019 | A | Event shape is explicitly open between FLAT beacon and two-arm Pattern-2. | G6:L115-L121, L233-L234 |
| DG002-A-020 | A | G-6 calls its manifest “signed,” but does not name the signer, key/trust root, signature schema, or whether G-8 supplies it. | G6:L123-L127, L177-L183 |
| DG002-A-021 | A | Tier-2 RunPod gate configuration, retry/cancellation, expense controls, credential scope, and partial-matrix representation are unspecified. | G6:L94-L103, L169-L170, L235-L236 |
| DG002-A-022 | A | `corpus_blake3` coverage/canonicalization and per-task row schema required for recomputation are not defined. | G6:L118-L120, L180-L181, L206 |
| DG002-A-023 | A | The exact condition constituting “F-2b LIVE” and the authority that releases external claims are not operationalized. | G6:L18-L24, L281-L289 |
| DG002-A-024 | A | How static and dynamic results coexist in one `SafeDeltaReport`/panel, including unavailable cells and `runpod_pending`, is unspecified. | G6:L100-L103, L129-L131, L209-L210 |
| DG002-A-025 | A | The headline denominator for incomplete/failed tasks and treatment of exploit/test infrastructure failures are not stated. | G6:L5-L13, L100-L103 |
| DG002-A-026 | A | G-8 does not define the complete in-toto Statement envelope, predicate type URI/version, JSON canonicalization, or subject-name schema. | G8:L31-L40, L99-L106 |
| DG002-A-027 | A | The proposed type is `InTotoPredicate`, while deterministic checking searches `InTotoStatement`; authoritative identifier is unclear. | G8:L99-L106, L194 |
| DG002-A-028 | A | “Signed local BLAKE3-Merkle proof” lacks a signing key, MAC/public-key primitive, trust root, or threat model beyond tamper-evident hashing. | G8:L5-L10, L39-L40, L105-L106 |
| DG002-A-029 | A | Local proof serialization, key/root storage, rotation, revocation, and verification across machines are unspecified. | G8:L105-L106, L115-L119 |
| DG002-A-030 | A | Sidecar lookup from a commit, naming collisions, multiple sessions/commits, relocation, and missing-sidecar behavior are undefined. | G8:L108-L119 |
| DG002-A-031 | A | Task prose spells `provenance-digest`, while corrected wiring names exact key `ELAI-Provenance-Digest`; casing and final serialized format conflict. | G8:L108-L113, L156-L168 |
| DG002-A-032 | A | Verify CLI exit codes, JSON mode, error classes, trust-root input, and offline missing-evidence semantics are absent. | G8:L115-L119, L183-L188 |
| DG002-A-033 | A | Rechecking verdict hashes “against recorded evidence” without rerunning ELAI does not state which independent data makes the check more than internal self-consistency. | G8:L115-L119, L215-L218 |
| DG002-A-034 | A | `all_passed` and `gate_count` behavior for absent, skipped, deferred, or failed-to-record F gates is not defined. | G8:L121-L125 |
| DG002-A-035 | A | It is unclear whether corrected 213/490 already includes `ProvenanceAttestationEmitted` or is the baseline to increment. | G8:L66, L121-L125, L223-L224 |
| DG002-A-036 | A | The exact optional cosign/sigstore crate, version, feature wiring, Rekor availability, and offline fallback behavior remain deferred. | G8:L23-L24, L127-L130, L298-L299 |
| DG002-A-037 | A | G-8’s overall implementation status is unclear because planned header/tasks coexist with assertions that key types/routes/CLI shipped. | G8:L3, L21-L24, L84-L85, L155-L170 |
| DG002-A-038 | A | Staged-tree digest scope for submodules, symlinks, file modes, ignored/untracked files, and dirty index changes is unspecified. | G8:L36-L38, L103-L105, L296-L297 |
| DG002-A-039 | A | Trailer insertion timing relative to the commit’s staged-tree digest and sidecar creation is not fully ordered, including failure rollback. | G8:L108-L119, L164-L170 |
| DG002-A-040 | A | Legal/posture text is required to be refreshed, but no pinned source revision or expiry gate is supplied. | G8:L87-L95, L273-L279, L300-L302 |
| DG002-A-041 | A | No retention/privacy/access-control rule is given for sidecars containing model and gate evidence. | G8:L99-L112, L132-L135 |
| DG002-A-042 | A | G-9 promises secret and PII detection and tests email, but reused `SECRET_PATTERNS` explicitly lists only AWS/PEM/secret assignments; the PII/email detector is not defined. | G9:L5-L14, L55-L58, L95-L101, L177-L181 |
| DG002-A-043 | A | “BLAKE3-keyed token” has no secret key source or keyed-hash API; the task text instead describes a plain BLAKE3 prefix of the secret. | G9:L5-L7, L35-L37, L95-L99 |
| DG002-A-044 | A | Token domain separation, label selection, collision handling for 16 hex characters, key rotation, and cross-project correlation are unspecified. | G9:L95-L99 |
| DG002-A-045 | A | Overlapping/multiple pattern matches and stable replacement order are not defined. | G9:L95-L101 |
| DG002-A-046 | A | “Detection error” lacks a specified error-producing detector interface or rule for how much of the containing field is redacted. | G9:L95-L101, L166-L167 |
| DG002-A-047 | A | Exhaustive traversal rules for nested JSON, arrays, optional strings, encoded/binary fields, spill files, and future `SsbPayload` variants are absent. | G9:L99-L101, L299-L300 |
| DG002-A-048 | A | Shared-DB retention is an explicit unresolved choice between maintenance row purge and whole-file TTL. | G9:L111-L117, L292-L295 |
| DG002-A-049 | A | Maintenance-purge scheduling, exclusive locking, vacuum/WAL cleanup, crash consistency, audit evidence, and reconciliation with “no DELETE” are unspecified. | G9:L111-L117, L250-L252 |
| DG002-A-050 | A | TTL duration, configuration source/default, clock basis, expiry timestamp, and retry/recovery behavior are not defined. | G9:L111-L118 |
| DG002-A-051 | A | Redaction-count persistence is not fixed when the optional advisory is disabled; the panel may read a counter or rows. | G9:L119-L133, L235-L236 |
| DG002-A-052 | A | The hook writer is asserted wire-dark behind `ELAI_SKILL_SYSTEM_ENABLE`, but the plan does not clarify whether/when its redaction done gate can run in production. | G9:L52-L54, L72-L73 |
| DG002-A-053 | A | Redaction failure handling around the SQLite transaction and log append does not specify retries, transaction rollback, partial logs, or surfaced errors. | G9:L103-L109 |
| DG002-A-054 | A | Permissions and audit requirements for `ELAI_REDACT_DISABLE` are absent even though it disables a privacy boundary. | G9:L126-L129, L204-L205 |
| DG002-A-055 | A | The optional advisory could itself traverse the redaction writer; recursion/suppression and exactly-one-event rules are unstated. | G9:L119-L124, L237-L238 |
| DG002-A-056 | A | Secret exposure in SQLite WAL/pages, backups, preexisting rows, crash dumps, or process memory is outside the stated pre-INSERT guarantee. | G9:L46-L50, L209-L210 |
| DG002-A-057 | A | The source does not specify whether a maintenance-boundary session-row purge is a permitted exception to append-only policy or a semantic conflict requiring a separate store. | G9:L113-L117, L292-L295 |
| DG002-A-058 | A | G-10 does not define baseline persistence format/location, atomic update, schema/version, retention, or recovery. | G10:L102-L120, L129-L133 |
| DG002-A-059 | A | Lookup is described both as identity-keyed and as “last-good for provider+lane”; selecting the comparison baseline after identity changes is not defined. | G10:L9-L12, L105-L120, L122-L125 |
| DG002-A-060 | A | G-2 uses `system_fingerprint: String`, whereas `ModelIdentity` uses `Option<String>`; normalization of missing/opaque values is absent. | G2:L74-L79; G10:L102-L105 |
| DG002-A-061 | A | Scheduled-check cadence, configuration, jitter, budget, and concurrency with run-end checks are unspecified. | G10:L10-L11, L323-L325 |
| DG002-A-062 | A | The fixed canary task set, size, family coverage, fixture version/hash, refresh governance, and leakage protection are undefined. | G10:L9-L11, L111-L120, L323-L325 |
| DG002-A-063 | A | The reused sample-count threshold/non-determinism band has no numeric values or model/provider-specific calibration process here. | G10:L29-L31, L116-L117, L318-L322 |
| DG002-A-064 | A | F-4 window length, malformed-output definition, spike threshold, and interaction with canary regression are not defined. | G10:L117-L120 |
| DG002-A-065 | A | Mapping a `CanaryFamily` to ReasoningTemplate domains/IDs, handling multiple domains, and review queue lifecycle are unspecified. | G10:L118-L120, L142-L145 |
| DG002-A-066 | A | Objective says flag “on regression,” while Task 2 says any beyond-band signal; whether fingerprint change alone emits drift is unclear. | G10:L10-L13, L114-L120 |
| DG002-A-067 | A | `FingerprintChanged` is set from a changed composite `identity_key`, which can also change for model/provider/lane reasons. | G10:L103-L105, L114-L116 |
| DG002-A-068 | A | Canary output canonicalization and ordering for `output_blake3` and byte-identical per-family deltas are unspecified. | G10:L105-L106, L233 |
| DG002-A-069 | A | `BaselineCaptured` has no named event/action, so its durable observability and dashboard/CLI source are unclear. | G10:L107-L114, L129-L145, L326-L327 |
| DG002-A-070 | A | The proposed production hook is called a run-end point, but exact lifecycle timing, run failure behavior, and relation to G-2’s corrected run-exit path are unresolved. | G10:L122-L127, L180-L185; G2:L141-L150 |
| DG002-A-071 | A | “Pattern-2 single-variant” with a vector of `DriftSignal` kinds does not define SQL discriminator mapping or whether one row represents multiple kinds. | G10:L135-L140 |
| DG002-A-072 | A | The per-task/per-family SSB rows required for journal recomputation are not named or schematized. | G10:L176-L178, L203, L233 |
| DG002-A-073 | A | Auto-pin does not define last-good model resolution, availability validation, rollback failure, human approval UI, or use of the existing swap event types. | G10:L147-L151, L334-L339 |
| DG002-A-074 | A | Cloud aliases with absent, unstable, or later-exposed fingerprints and transitions between behavioral/fingerprint confirmation are not covered. | G10:L60-L64, L119-L120 |
| DG002-A-075 | A | Duplicate/simultaneous checks, baseline capture races, exactly-once event emission, and rerun cancellation are unspecified. | G10:L10-L12, L122-L127, L182-L186 |
| DG002-A-076 | A | Promotion of a new model result to “last-good,” baseline history/versioning, and manual rejection/reversion are absent. | G10:L11-L12, L105-L120, L129-L145 |

## Conflict ledger

| Atomic ID | Evidence | Conflict retained without normalization | Exact anchors |
|---|---|---|---|
| DG002-C-001 | C | G-2’s own header says `PLANNED`, while G-8’s corrected state says the G-2 RRA landed “this run.” | G2:L3 versus G8:L21-L24, L84-L85 |
| DG002-C-002 | C | G-6 and G-9 call `rra.rs` absent/forward-dependent, while G-8 calls it shipped. | G6:L24, L65, L81; G9:L22-L23, L64, L79 versus G8:L21-L24, L84-L85 |
| DG002-C-003 | C | G-10’s prerequisite says G-2 is shipped or co-shipped, while its existing-state block says `RunArtifact` is planned. | G10:L28-L29 versus L73-L74 |
| DG002-C-004 | C | G-2’s reconciled contract says RRA emission occurs at `run_async` exit and explicitly not in `build_production_hook_dispatcher`; older Task 3 and files-touched still assign `runtime_setup.rs`. | G2:L92-L95, L141-L150, L189-L191 versus L232-L245 |
| DG002-C-005 | C | G-10 names `build_production_hook_dispatcher` as the run-end drift hook, while G-2’s correction says that dispatcher is a tool-use dispatcher, not the artifact run-exit hook. | G10:L95-L96, L122-L127, L180-L185 versus G2:L141-L150 |
| DG002-C-006 | C | G-8’s header remains `PLANNED`, while its corrected contract calls the digest trailer, verify CLI, manifest dependency, and run-exit emit path shipped/live. | G8:L3 versus L21-L24, L84-L85, L155-L170, L286-L288 |
| DG002-C-007 | C | G-8’s corrected blocks say `rra.rs` shipped, while older mandatory reads and W-7 prose still say it is a forward dependency/absent. | G8:L21-L24, L84-L85, L157, L179 versus L94-L95, L235-L238 |
| DG002-C-008 | C | G-8 Task 2 calls the trailer `provenance-digest`, whereas corrected consumed/reachable wiring names `ELAI-Provenance-Digest`. | G8:L108-L113 versus L156-L168 |
| DG002-C-009 | C | G-8 tasks produce `InTotoPredicate`, but the deterministic check requires `InTotoStatement`. | G8:L99-L106, L145-L149 versus L194 |
| DG002-C-010 | C | G-2 defines `RunArtifact.evidence_manifest` and `GateVerdictHash.ssb_event_id`; G-8’s corrected consumer refers to `RunArtifact.manifest` and `GateVerdictHash.ssb_event_seq`. | G2:L74-L80 versus G8:L156-L160 |
| DG002-C-011 | C | G-8 Task 1 says `build_attestation` reads SSB payloads directly, while corrected wiring says it reads the G-2 `RunArtifact` manifest and must not fork extraction. | G8:L99-L105 versus L155-L160, L179 |
| DG002-C-012 | C | G-9 initially says individual rows cannot be deleted, yet its honest shared-DB caveat permits a session-scoped row purge at a maintenance boundary. | G9:L7-L10, L37-L38 versus L113-L117, L292-L295 |
| DG002-C-013 | C | The permitted summary requires both BLAKE3 and SHA-256 subject digests for G-8, while the assigned original specifies BLAKE3-only and explicitly bans SHA2. | SUM:L307-L310 versus G8:L5-L8, L178, L202 |
| DG002-C-014 | C | The permitted summary treats G-9 TTL as whole-file deletion of an expired per-session database; the original says there is one shared database and that exact framing is false. | SUM:L349-L350, L366 versus G9:L60-L63, L111-L117, L170 |
| DG002-C-015 | C | The permitted summary fixes zstd compression for G-2; the original deliberately leaves codec choice at “already-vendored” and requires confirmation before zstd. | SUM:L97-L100 versus G2:L58-L59, L81-L82, L157-L158, L266-L267 |
| DG002-C-016 | C | The permitted summary uses `build_production_hook_dispatcher` as G-2’s production integration; the corrected original explicitly supersedes that anchor with `run_async` exit. | SUM:L121-L124 versus G2:L141-L150, L189-L191 |
| DG002-C-017 | C | G-2 describes G-9 as its export precondition while G-9 also calls G-2 a forward dependency, yielding a build-order cycle unless “build” and “safe external release” are separated. | G2:L272-L276 versus G9:L22-L25, L158 |
| DG002-C-018 | C | G-8’s older state says the commit path has no positive trailer writer; corrected blocks assert a shipped `PROVENANCE_TRAILER_KEY` and live binding. | G8:L48-L53, L68-L72 versus L155-L170 |

## Additions and corrections to `07-G-SUMMARY_INDEX.discovery.md`

`ADD` means source-explicit detail absent from the orientation ledger; `CORRECT` supersedes or contradicts it; `STATUS` adds implementation-state evidence. Rows group only tightly related source atoms, with no source behavior promoted from the summary itself.

| Atomic ID | Delta | Addition or correction | Deep anchor | Orientation anchor |
|---|---|---|---|---|
| DG002-SD-001 | STATUS | G-2 remains header-PLANNED, but G-8 later says its RRA/manifest shipped; G-6/G-9 say absent and G-10 is internally split. The summary’s single planned state is incomplete. | G2:L3; G6:L24; G8:L21-L24; G9:L22-L23; G10:L28-L29, L73-L74 | SUM:L97-L135 |
| DG002-SD-002 | CORRECT | RRA production emission is `run_async` exit using live `wiring.ssb`, not `build_production_hook_dispatcher`. | G2:L141-L150, L189-L191 | SUM:L121-L124 |
| DG002-SD-003 | CORRECT | Compression is an authoring-time in-tree-codec choice; zstd is not fixed and requires confirmation. | G2:L58-L59, L157-L158, L266-L267 | SUM:L97-L100 |
| DG002-SD-004 | ADD | Exact `RunArtifact` schema adds `schema_version` and per-run `GuaranteeLevel` with three named states. | G2:L74-L79 | SUM:L97-L117 |
| DG002-SD-005 | ADD | Exact `GateVerdictHash` schema is gate/action/hash/event ID, and F-10 is conditional “when it lands.” | G2:L18-L20, L78-L80 | SUM:L105-L109 |
| DG002-SD-006 | ADD | Exact replay result schema includes expected/got hashes and distinct `Tampered`. | G2:L84-L90 | SUM:L128-L133 |
| DG002-SD-007 | ADD | The Merkle root explicitly covers stream, manifest, and pins; one-byte mutation must fail. | G2:L81-L82, L268-L269 | SUM:L109-L110, L128-L135 |
| DG002-SD-008 | ADD | RRA is default-on with disable-only `ELAI_RRA_DISABLE`; an ENABLE flag is banned. | G2:L92-L95, L151-L155, L188 | SUM:L111-L135 |
| DG002-SD-009 | ADD | `RunArtifactEmittedPayload` exact fields and baseline+one/golden requirements are supplied. | G2:L102-L104, L198 | SUM:L128-L135 |
| DG002-SD-010 | ADD | CLI semantics distinguish default emission, force emit, show, verify, and `--json`; viewer fields and Tauri endpoint are exact. | G2:L97-L108 | SUM:L111-L127 |
| DG002-SD-011 | ADD | Production paths ban unwrap/expect and non-BLAKE3 hashes and forbid unverified dependencies. | G2:L70, L82, L151-L159, L200 | SUM:L93-L135 |
| DG002-SD-012 | ADD | Resume is a separate non-DONE capability; the BSP-no-caller assertion is explicitly refuted. | G2:L56-L57, L110-L118 | SUM:L124-L127 |
| DG002-SD-013 | ADD | Replay tests distinguish local, cloud-opaque, tampered, and injected-event divergence, plus a real-hook smoke. | G2:L222-L230 | SUM:L128-L135 |
| DG002-SD-014 | ADD | Same-window SSB editing is prohibited and G-1 `taint_flow` rows may join the manifest. | G2:L22-L23 | SUM:L117-L126 |
| DG002-SD-015 | ADD | Artifact-write failure, canonical serialization, event replay idempotency, and guarantee-level assignment remain unresolved. | DG002-A-001…A-010 | SUM:L500 |
| DG002-SD-016 | STATUS | G-6’s static SeCodePLT metrics/event ship but have never run live; dynamic/live competitor results are a separate Tier-2. | G6:L5-L16, L69-L81 | SUM:L234-L269 |
| DG002-SD-017 | CORRECT | Real bench plumbing is `args_bench.rs`/`bench_cmd.rs`/per-suite files; `benchmark.rs` is absent and SeCodePLT rejects `--suite`. | G6:L50-L64, L75-L77 | SUM:L262-L269 |
| DG002-SD-018 | ADD | Comparisons are pinned to both seed and same `HardwareClass`. | G6:L61-L62, L78 | SUM:L263-L264 |
| DG002-SD-019 | ADD | Exact runner/report schemas and six enum arms are supplied, including before/after exploitable counts. | G6:L94-L103 | SUM:L234-L269 |
| DG002-SD-020 | ADD | Tier-1 static is the done gate; dynamic exploit/live-model/competitors are separately RunPod-gated. | G6:L102-L103, L235-L236 | SUM:L258-L269 |
| DG002-SD-021 | ADD | Exact `SafetyDeltaMeasuredPayload` fields are supplied. | G6:L115-L121 | SUM:L262-L269 |
| DG002-SD-022 | ADD | Event modeling is still open between FLAT beacon and two-arm Pattern-2. | G6:L115-L121, L233-L234 | SUM:L264-L269, L504 |
| DG002-SD-023 | ADD | Current schema/variant baselines are 205/482 and stale E-45’s 394 assertion is explicitly not a G-6 regression. | G6:L65-L67, L277-L278 | SUM:L230-L269 |
| DG002-SD-024 | ADD | Honest Tier-2 state is `runpod_pending`/`not-run`, never estimated, with matched-condition methodology unresolved. | G6:L169-L170, L281-L289 | SUM:L267-L269 |
| DG002-SD-025 | ADD | Exact panel/Tauri pair, serialization claim-language contract, anti-gaming cases, and conditional dependency gate are supplied. | G6:L129-L131, L199-L217, L240-L247 | SUM:L230-L269 |
| DG002-SD-026 | ADD | Corpus hashing/per-task row schema, exploit sandbox, competitor freeze/fairness, and bootstrap unit remain open. | DG002-A-013…A-025 | SUM:L504 |
| DG002-SD-027 | STATUS | G-8 is header-PLANNED, but corrected text calls the G-2 manifest, trailer, verify CLI, and run-exit emitter shipped. | G8:L3, L21-L24, L155-L170 | SUM:L303-L333 |
| DG002-SD-028 | CORRECT | G-8’s production route is `run_async::emit_attestation`, not an unspecified `elai run`/dispatcher route. | G8:L164-L170 | SUM:L330-L333 |
| DG002-SD-029 | ADD | Exact predicate and per-gate schemas are supplied, with staged-tree/files as subject rather than commit SHA. | G8:L99-L106 | SUM:L307-L317 |
| DG002-SD-030 | CORRECT | SHA-256 subject binding in the summary is unsupported and conflicts with the original’s BLAKE3-only/SHA2 ban. | G8:L5-L8, L178, L202 | SUM:L307-L310 |
| DG002-SD-031 | ADD | Corrected exact trailer key is `ELAI-Provenance-Digest`; it must survive the existing negative trailer filter. | G8:L48-L53, L108-L113, L156-L168 | SUM:L315-L317 |
| DG002-SD-032 | ADD | Sidecar path, offline recompute steps, conditional cosign feature, and dependency verification fields are exact. | G8:L108-L130 | SUM:L315-L329 |
| DG002-SD-033 | ADD | Exact attestation event payload fields and corrected 213/490 live counters are supplied. | G8:L121-L125 | SUM:L317, L333 |
| DG002-SD-034 | ADD | Attestation emission is default-on behind only `ELAI_ATTESTATION_DISABLE`. | G8:L108-L113, L177, L209-L212 | SUM:L303-L333 |
| DG002-SD-035 | ADD | Drawer fields, tamper/strip-filter tests, and workspace-without-cosign gate are supplied. | G8:L132-L135, L208-L225, L248-L255 | SUM:L303-L333 |
| DG002-SD-036 | ADD | Compliance copy must be refreshed from the posture source; sidecar privacy/retention is not specified. | G8:L87-L95, L273-L302; DG002-A-041 | SUM:L323-L329, L506 |
| DG002-SD-037 | ADD | Predicate-vs-statement identifier, trailer spelling, direct-SSB-vs-manifest, and shipped-vs-absent strata conflict internally. | DG002-C-006…C-011, C-018 | SUM:L303-L333 |
| DG002-SD-038 | CORRECT | G-9 has one shared project `ssb.db`; the summary’s per-session DB/whole-session-file premise is false. | G9:L60-L63, L111-L117, L170 | SUM:L349-L350, L366 |
| DG002-SD-039 | ADD | SSB retention remains a choice between maintenance-boundary session rows and whole shared file; only hook logs are per-session files. | G9:L111-L117, L292-L295 | SUM:L349-L350, L366 |
| DG002-SD-040 | ADD | Exact write chokepoint is `execute_write` before INSERT within the sole-writer transaction; hook append is the second path. | G9:L46-L54, L103-L109 | SUM:L339-L348 |
| DG002-SD-041 | ADD | Redactor APIs, stable token syntax, return count, payload traversal, and detector-error fail-closed rule are supplied. | G9:L93-L101 | SUM:L339-L367 |
| DG002-SD-042 | ADD | Reused detector’s named shapes are AWS/PEM/secret assignments; email/PII detection promised by the fixture is not defined. | G9:L55-L58, L95-L101, L177-L181 | SUM:L344-L365 |
| DG002-SD-043 | ADD | Core redaction uses disable-only `ELAI_REDACT_DISABLE`; `SecretRedacted` is optional and alone may be wire-dark. | G9:L119-L129, L168-L169 | SUM:L339-L367 |
| DG002-SD-044 | ADD | Exact panel/count-only privacy rule, ten-benign corpus, three-real-run FP gate, cross-variant leak-hole test, and detector-error case are supplied. | G9:L126-L133, L201-L220, L245-L252, L287-L300 | SUM:L359-L367 |
| DG002-SD-045 | ADD | “BLAKE3-keyed” lacks a key and the shared-DB purge/append-only exception, TTL config, and error recovery remain open. | DG002-A-043…A-057 | SUM:L335-L367, L507 |
| DG002-SD-046 | ADD | G-9 is a mutual sequencing dependency with G-2: required for safe export but written against an absent forward RRA. | G2:L272-L276; G9:L22-L25, L302-L306 | SUM:L358-L367 |
| DG002-SD-047 | ADD | G-10 exact schemas include `ModelIdentity`, optional fingerprint/lane, `DriftBaseline`, three signals, three verdicts, and BLAKE3 identity/output hashes. | G10:L102-L120 | SUM:L377-L402 |
| DG002-SD-048 | ADD | A first-seen identity returns `BaselineCaptured`, not drift. | G10:L326-L327 | SUM:L373-L425 |
| DG002-SD-049 | ADD | F-4 malformed-output spike is a first-class third signal and exact event fields include identity/baseline time. | G10:L106-L120, L135-L140 | SUM:L398-L419 |
| DG002-SD-050 | ADD | Detection is default-on behind only `ELAI_MODEL_DRIFT_DISABLE`; action remains separately off. | G10:L122-L127, L187-L196, L223-L224 | SUM:L403-L412 |
| DG002-SD-051 | ADD | CLI `show`, `--json`, exact panel contents, and no-row clean behavior are fully specified. | G10:L129-L145, L200-L203 | SUM:L403-L425 |
| DG002-SD-052 | ADD | Optional auto-pin requires experiment owner/expiry/promotion/smoke metadata and three-run FP bounds. | G10:L147-L151, L234-L235 | SUM:L410-L412 |
| DG002-SD-053 | CORRECT | G-10’s named runtime dispatcher conflicts with G-2’s correction that this is not the run-exit hook. | G10:L122-L127, L180-L185; G2:L141-L150 | SUM:L403-L409 |
| DG002-SD-054 | ADD | Baseline persistence/lookup/promotion, schedule, canary size, thresholds, event Kind mapping, idempotency, and recomputation-row schemas remain open. | DG002-A-058…A-076 | SUM:L369-L425, L508 |
| DG002-SD-055 | ADD | G-10 explicitly forbids new dependencies, canary reimplementation, absolute capability ranking, and auto-remediation in DONE. | G10:L187-L196, L240-L260, L331-L332 | SUM:L369-L425 |

## Requested-dimension coverage matrix

| Requested dimension | Explicit coverage | Principal rows | Separated unresolved coverage |
|---|---|---|---|
| Capabilities/actions/surfaces | RRA build/show/replay/view; SAFEDELTA runner/CLI/panel; attestation emit/verify/drawer; redaction/purge/panel; drift baseline/check/show/panel | All five explicit ledgers; identifier registry | A-008…010, A-024, A-030…032, A-051, A-069, A-073 |
| Workflows/states | Guarantee/replay outcomes, benchmark tiers/arms, offline verification, write-before-persist and TTL, drift baseline/signal/verdict/review | G2-014…036; G6-024…045; G8-025…042; G9-019…040; G10-024…050 | A-004…011, A-019…025, A-034, A-048…050, A-058…076 |
| Schemas/data/lifecycle | Every named struct/enum/payload and artifact/file path is inventoried | G2-018…035; G6-018…045; G8-025…040; G9-019…039; G10-024…052 | A-001…007, A-013, A-022, A-026…035, A-042…051, A-058…060, A-068…072 |
| Flags/defaults/config | All disable-only switches, forbidden ENABLE flags, optional feature/advisory/action gates, seed/hardware inputs | G2-030/041; G6-016/032/049; G8-010/031/036; G9-025/032…035; G10-038/042/047…049 | A-002, A-014…015, A-021, A-036, A-050, A-054, A-061…063 |
| Integrations/events | Exact produced/consumed registry, five SSB actions, production/CLI/dashboard routes | Identifier registry; cross-source interactions | A-009, A-019…022, A-030…035, A-051…055, A-069…072, A-075 |
| Permissions/security/privacy/compliance | Fail-closed redaction; no cloud egress; anti-slopsquatting; digest-only trailer; exact Article-50/99 claim boundary; human-gated auto-pin | G8-005…014/030…043/054…063; G9-001…009/024…046/054…064; G10-008…011/047…057/065…076 | A-028…041, A-042…057, A-073…074 |
| Failure/fallback/retry/recovery | Tamper/divergence, missing-body downgrade, RunPod pending/not-run, redaction fail-closed, clean no-row/cold baseline, deferred resume/auto-pin | G2-016/023…027/036/044/048/054…057; G6-032/036/049/060/066…068; G8-047/052/061…063; G9-006/029…035/043/048…064; G10-029…038/047…049/060…076 | A-005…011, A-016…025, A-030…036, A-046…057, A-058…076 |
| Determinism/idempotency/concurrency | Seed/hardware pins, BLAKE3 roots/tokens/identity, byte recomputation, matched edit-window constraints, fixed canaries | G2-005/012…017/021…027/040…057; G6-012/016/030…038/050…065; G8-005/009/025…036/043; G9-004/010/013/018…030/045/055; G10-004…006/017/026…036/056/061/068…075 | A-001…009, A-013…025, A-028…029, A-043…050, A-058…068, A-071…076 |
| Observability/evidence | All artifact/event fields, live expected rows/files, panels, offline verdicts, exact claim evidence | G2-029…055; G6-039…065; G8-030…060; G9-031…062; G10-037…072 | A-010, A-020/022/024, A-030…041, A-051/055, A-069…072 |
| Tests/acceptance/done gates | Every deterministic check class, observable trigger, payload/golden/schema gate, dashboard smoker, cargo/auditor, FP/claim test | G2-042…055; G6-052…065; G8-045…060; G9-047…062; G10-058…072 | Ambiguities above identify missing mechanics, not omitted stated gates |
| Constraints/non-goals | Honest replay/claim boundaries; no rank/perfect-code; no certification; no cloud redactor; no auto-remediation; optional resume/cosign/advisory/action | G2-007/016/017/027/036/040/041; G6-009/032/036/049…051/068; G8-010/012…014/029/031/036/043/044/058/063; G9-006…009/023/030…035/041…046/060; G10-008…011/047…057/070/073…077 | None omitted; unresolved implementations are in A ledger |
| Dependencies/status/deferred | All prerequisite/status assertions, cross-source dependency edges, parallel-safety rules, optional/deferred phases, conflicting temporal strata | Per-ledger status boundaries; cross-source interactions; C-001…018 | Status conflicts remain deliberately unresolved |

## Coverage, zero-omission, and duplicate audit

- **Manifest coverage:** 5/5 assigned originals processed completely; 1,539/1,539 logical lines; 137,029/137,029 bytes; 5/5 SHA-256 values exact.
- **Source endings observed:** `End of G-2.` at G2:L279; `End of G-6.` at G6:L301; `End of G-8.` at G8:L310; `End of G-9.` at G9:L308; `End of G-10.` at G10:L341.
- **Explicit per-file atomic counts:** G-2 = 58; G-6 = 69; G-8 = 64; G-9 = 65; G-10 = 77; explicit total = 333.
- **Separated epistemic counts:** inference = 8; ambiguity = 76; conflict = 18; summary addition/correction = 55.
- **Unique enumerated atomic IDs:** 490; duplicate IDs: 0.
- **Zero-omission rule:** every named command, flag, environment default, type/variant/field set, event/action, artifact path, CLI/dashboard pair, production reachability statement, lane/tier/status, dependency, failure/fallback, privacy/compliance boundary, observable trigger, test/acceptance condition, risk, and explicit defer/non-goal in the five originals is represented by an explicit row or an epistemically separated I/A/C row.
- **Duplicate-content rule:** repeated Objective/Task/WIRING/ACCEPTANCE/W-clause/Test language was collapsed by semantic key. The registry, interactions, matrix, and audit reference counted atoms and add zero feature atoms.
- **Orientation isolation:** the summary discovery contributes only comparison anchors in `DG002-SD-*`/`DG002-C-*`; it is never the sole basis of a source-explicit feature.

## Output digest

Digest convention: payload SHA-256 covers every byte before the `## Output digest` heading (including the newline before it), avoiding a self-referential whole-file hash. The final whole-file SHA-256 is reported separately to the compiling agent.

- Payload SHA-256: `3a73d1cc7cc34071a7f153952d48fa320b94880ebb8606ea6a640710fb69313d`
- Output payload lines before digest heading: 635
- Final file lines: 644
- Unique enumerated atomic IDs: 490
- Duplicate atomic IDs: 0
