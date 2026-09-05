G-10 decisions — Model-Drift Regression Canary (exogenous model-version behavioral-regression gate)
Session s-56068 (resume). Written before EXECUTING. Binding on the executor + auditor.

WHAT
Pin the (model, system_fingerprint, lane) identity tuple (input from G-2's LIVE RunArtifact); hold a fixed
canary set with baseline scores keyed to that identity; on a detected identity change (or schedule) re-run the
canary, compute a per-family regression delta vs the last-good baseline, and on regression FLAG
model_drift_suspected + carry the affected ReasoningTemplate ids for review. SURFACE, never auto-swap (GD-4).
REUSE D-7 canary.rs (families+delta+band); re-implement NONE of it (GD-1).

LIVE BASELINE (code is ground truth): SCHEMA_VERSION 217 (ssb.rs) -> 218. TOTAL_VARIANTS 494
(tests/action_type_lock.rs:612) -> 495.

DD1 — REUSE canary.rs, supply identity EXTERNALLY (GD-1 compose-not-restate, BLOCKING).
canary.rs is IDENTITY-AGNOSTIC (keys on CanaryFamily + HeldOutTask.task_id). Reuse run_canary + decide +
FamilyEvalResult.transfer_score/baseline_score for the per-family delta + the non-determinism BAND.
Re-implement no canary logic. New crates/orchestrator/src/model_drift.rs builds a ModelIdentity{model_pin,
system_fingerprint_pin, lane} from RunArtifact (rra.rs:211/214, both Option<String>) and DERIVES lane (no
stored field) via has_cloud_model_swap (rra.rs:354) / cloud_mode::RoutedProvider — do NOT read a non-existent
lane field.

DD2 — new SSB Pattern-2 single variant ModelDriftSuspected (W-9, SCHEMA bump named).
Single Kind discriminator field `kind: DriftSignal` (FingerprintChanged / CanaryRegressed /
MalformedOutputSpike) for the golden lock + a `signals: Vec<..>` detail field (resolves the Pattern-2-with-Kind
tension). Payload also carries: model_identity (the tuple), per_family_delta (map family->delta),
affected_template_ids (Vec<ReasoningTemplate id>), confirmed_by_fingerprint (bool, HONEST per-lane). MIRROR the
in-tree G-6 SafetyDeltaMeasured cascade EXACTLY (grep `SafetyDeltaMeasured` in ssb.rs + action_type_lock to
enumerate every arm) — ALL of: SsbActionType enum + as_sql_str + from_str + SsbPayload enum + action_type() map
+ payload struct + test_payload arm + all_test_payloads + test_typed_payloads census/count + THE action_type
IN(...) SQL CHECK list (ssb.rs:4775 — the G-8 P0 trap, runtime-only, cargo check blind) + action_type_lock.rs
ALL_VARIANTS + TOTAL_VARIANTS 494->495 + golden insert (alphabetical ordinal) + header comment + exhaustive
match + lib.rs re-export + e45_cross_language_drift_contract.rs:294 (494 -> 495).

DD3 — `elai model-drift baseline|check|show` CLI (new model_drift_cmd.rs).
MUST NOT collide with E-45's existing `elai drift` (args.rs:6133 / drift_cmd.rs:264 = cross-language SOURCE
drift). Use the distinct `model-drift` subcommand. `baseline` runs the canary under the current identity and
writes per-family baseline scores as a JSON file keyed to the identity tuple under benchmarks/model_drift/
(mirror E-45's commit-keyed JSON baseline + G-6's benchmarks/safedelta/ layout). `check` reads the last-good
baseline, re-runs the canary, computes the per-family delta within the band, emits a row ONLY on drift. `show`
displays the latest.

DD4 — run-end check at run_subcommand.rs run_async EXIT (449-488).
Add a 3rd/4th best-effort block beside the G-2 RRA emit + G-8 attestation, on the shared rra_ssb Arc<Ssb>
(the same shape G-2 used). On a detected identity change (fingerprint/model differs from the last-good baseline)
OR a schedule tick, run the check and emit ModelDriftSuspected on regression. NOT the stale runtime_setup.rs
PostToolUse anchor (that is per-tool-call, wrong granularity).

DD5 — SURFACE not auto-swap (GD-4) + scope discipline.
The model_drift_suspected row CARRIES affected_template_ids — that IS the review surface (a reviewer reads the
row + the ModelDriftPanel). Do NOT invent a new template-review-queue subsystem (none exists; review_surface.rs
keys on git hunk/file, not template id — out of Tier-1 scope). The auto-pin-to-last-good action is a SEPARATE
wire-dark-allowlisted human-gated follow-on (would give ssb ModelSwap/TierModelSwap their first emitter) and is
NOT in the done-gate. Default = detect + surface only.

DD6 — GD-5 honesty + GD-2 reproducibility + the LOAD-BEARING anti-FP test.
LOCAL lane = fingerprint-confirmed (confirmed_by_fingerprint=true); CLOUD lane = behavioral-suspected
(Anthropic system_fingerprint OPAQUE per G-2 section 6) -> confirmed_by_fingerprint=FALSE for cloud; NEVER
fabricate a cloud fingerprint match; the ModelDriftPanel shows a fingerprint-confirmed/behavioral badge.
GD-2: the per-family delta recomputes byte-identically from the SSB journal under ELAI_SEED.
THE LOAD-BEARING GD-4 FP-GUARD TEST (this is the done-gate's hardest requirement, and the floor-fake trap):
baseline under a fixture/mock model A -> `model-drift check` under a mock-provider model B returning a REGRESSED
output -> exactly ONE model_drift_suspected row (kind + per_family_delta + affected_template_ids +
confirmed_by_fingerprint); a CLEAN re-check under the SAME model A within the non-determinism band -> NoDrift +
ZERO emitted row. A producer that ALWAYS emits (even on clean A) is a floor-fake and a hard BLOCK (the G-7
lesson). The dual test (positive=drift on B, negative=NO row on clean A) is mandatory.

DD7 — ModelDriftPanel frontend pair (W-4), mirror G-6.
New get_model_drift_latest Tauri command (mirror commands_g6.rs::get_safety_delta_latest, register in
generate_handler!) + ModelDriftPanel.svelte (mirror SafeDeltaPanel.svelte) + DrawerHost arm + a types.ts DTO
field-for-field consistent with the Rust DTO (cross_lang_diff must PASS). Show before/after per-family scores +
the fingerprint-confirmed/behavioral badge.

HARD RULES: write only under .elai_cc/; blake3 only; no .unwrap()/.expect()/as!-narrow in prod (test cfg + main
entrypoint OK); tracing not log; named constants; NO new deps (STOP+report if one seems needed); zero lazy code;
cargo check ONLY (NOT cargo test). Prior art (verifier-grounded, cite honestly): "Why Is My Prompt Getting
Worse?" arXiv:2311.11123 (silent API upgrades degrade a family); RETAIN arXiv:2409.03928; AgentAssay
arXiv:2603.02601 (MOTIVATION — a silent provider update 93%->71%, NOT an ELAI target). REUSE D-7 canary.rs.

EXPECTED TEST OUTCOME: cargo check --workspace green; the new test_typed_payloads + action_type_lock golden arms
compile; the dual fixture test yields a row under B and ZERO row under clean A. UNKNOWNS: the mock-provider A/B
harness (find the existing mock-provider pattern); whether a scheduled (non-identity-change) check tick is in
Tier-1 scope (identity-change trigger is the minimum; a schedule tick can be a marked follow-on).
