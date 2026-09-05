# P-7 S1 Context Package — Amended Tier-1 Fork

## TASK
Build the per-user OperatingProfile (Tier-1 fork): manual asymmetry-guarded change function, three default data profiles (P-5/P-2/P-8 composition deferred), monotone tighten/loosen evidence guard, H-17/H-18 evidence path, observe-first journal and revert, SSB row, CLI surface. No product code edits — context only.

## FORK ACTIVATION (binding)
N-3 is BLOCKED (escalated, two failed recon attempts). The executor MUST follow the FORK clause (P-7:41): ship Tier-1 with the MANUAL change function. Tighten applies directly. Loosen refuses without explicit measured own-run FP evidence supplied by the caller, then routes through H-18 evaluate_deflation for significance. The automatic learning loop (Tier-2) is deferred behind N-3's confirmation.

## M3 DEFERRAL (binding)
P-5, P-2, P-8 are ALL unbuilt (0 refs in codebase). The three default profiles (toy/connected/team) ship as DATA (struct instances with intended posture values). The mapping to sibling units is journaled as intent, NOT wired. Actual composition deferred until those units land.

## FILES TO EDIT (exact existing paths, ALL inside .elai_cc/crates/)

### 1. NEW FILE: orchestrator/src/operating_profile.rs
The OperatingProfile type + three default profiles + asymmetry-guarded change function.
- OperatingProfile struct: { per_domain: BTreeMap<String, DomainProfile> }
- DomainProfile struct: { autonomy: f64, gate_strictness: f64, explanation_verbosity: f64 }
- Three const/default profiles: toy(), connected(), team() — return OperatingProfile with intended posture values
- change_function: tighten(profile, domain, field, new_value) -> applies directly (monotone-tighten: new_value must be >= old for gate_strictness, or <= old for autonomy — tighten direction depends on field)
- change_function: loosen(profile, domain, field, new_value, evidence: Option<OwnRunFPEvidence>) -> REFUSES without evidence; with evidence, calls evaluate_deflation; promotes only on GateVerdictKind::Allow
- revert: restore prior value from journaled old value
- Observe-first: emit SSB row BEFORE applying (GD-4)

### 2. EDIT: orchestrator/src/lib.rs
- Add `pub mod operating_profile;` (near line 1367, after `pub mod deflation_gate;`)

### 3. EDIT: orchestrator/src/ssb/action_type.rs
- Add `OperatingProfileChange` variant to `SsbActionType` enum (after `ReasoningCheckpointEvent`, line ~4923)
- Add `Self::OperatingProfileChange => "operating_profile_change"` to `as_sql_str()` (after line 5905)
- Add `"operating_profile_change" => Ok(SsbActionType::OperatingProfileChange)` to `ssb_action_type_from_str()` (after line 6827)
- NOTE: This is a FLAT variant (no Kind discriminator) — the change direction (tighten/loosen/revert) rides inside the payload

### 4. EDIT: orchestrator/src/ssb/payloads.rs
- Add `OperatingProfileChange(OperatingProfileChangePayload)` variant to `SsbPayload` enum (after line 4029)
- Add `Self::OperatingProfileChange(_) => SsbActionType::OperatingProfileChange` to `action_type()` match (after line 4887)
- Add `OperatingProfileChangePayload` struct (near end of file, after line 22108):
  { domain: String, field: String, old_value: String, new_value: String, evidence_ref: Option<String>, change_kind: ProfileChangeKind }
- Add `ProfileChangeKind` enum: { Tighten, Loosen, Revert }

### 5. EDIT: orchestrator/src/ssb.rs
- Add `SsbActionType::OperatingProfileChange` to `SSB_ACTION_TYPES` array (after line 1666, before closing `];`)
- SCHEMA_VERSION auto-derives from SSB_ACTION_TYPES.len() — NO manual bump needed (Tier-B Move 2)

### 6. EDIT: orchestrator/src/ssb/schema.rs
- Add `'operating_profile_change'` to the CHECK constraint action_type IN (...) list (after line 5599, before closing `))`)

### 7. EDIT: orchestrator/tests/action_type_lock.rs
- Add `SsbActionType::OperatingProfileChange => (),` to `exhaustive_match_guard()` (after line 1607)
- TOTAL_VARIANTS and golden auto-derive from SSB_ACTION_TYPES (no manual count bump needed)
- MUST regen golden file: `cargo test --test action_type_lock -- --ignored --nocapture` (regen_golden_file test)

### 8. NEW FILE: cli/src/profile_cmd.rs
- Handler for `elai profile show|tighten|loosen` subcommand
- Pattern follows deflation_check_cmd.rs: open Ssb + TrialLedger, call operating_profile change function, emit SSB row
- show: print current profile for domain
- tighten: call change_function::tighten, emit SSB
- loosen: call change_function::loosen with caller-supplied evidence; if no evidence -> refuse with error message; if evidence supplied -> call evaluate_deflation, promote on Allow, refuse on Block

### 9. EDIT: cli/src/args.rs
- Add `Profile { subcommand, domain, field, value, evidence }` variant to `CliAction` enum (near line 1686)
- Add `"profile" => parse_profile_args(&rest[1..])` to dispatch (after line 3684, near deflation-check)
- Add `parse_profile_args` function (near line 4967, after parse_deflation_check_args)
  - subcommands: "show", "tighten", "loosen"
  - args: --domain <string>, --field <string>, --value <string>, --evidence <string> (optional, for loosen)

### 10. EDIT: cli/src/main.rs
- Add `mod profile_cmd;` (near line 274, after `mod deflation_check_cmd;`)
- Add `CliAction::Profile { ... }` dispatch arm (near line 1292, after DeflationCheck)
  - tokio runtime, call profile_cmd::run

## DO NOT RECREATE (existing modules to consume, not re-implement)

- `orchestrator/src/trial_ledger.rs` — TrialLedger::open(db_path, session_id), register_trial(...), trial_count(...) — USE AS-IS
- `orchestrator/src/deflation_gate.rs` — evaluate_deflation(ledger, ssb, DeflationInput) -> DeflationGateVerdict — USE AS-IS for loosen significance check
- `orchestrator/src/ssb/` module (action_type.rs, payloads.rs, schema.rs, store.rs, ssb.rs) — EXTEND with new variant, do not recreate
- `cli/src/args.rs` dispatch pattern — EXTEND with new arm, do not recreate
- `cli/src/deflation_check_cmd.rs` — reference pattern for handler structure, do not copy

## COMPILE/TEST LOCKSTEP

The SSB action type addition requires SIMULTANEOUS edits in 5 files that must compile together:
1. action_type.rs — enum variant + as_sql_str + from_str
2. payloads.rs — SsbPayload variant + action_type() match + payload struct
3. ssb.rs — SSB_ACTION_TYPES array entry
4. schema.rs — CHECK constraint DDL
5. action_type_lock.rs — exhaustive_match_guard arm

If ANY of these 5 is missing, compilation FAILS:
- Missing enum variant -> payloads.rs action_type() match won't compile (unknown variant)
- Missing SSB_ACTION_TYPES entry -> count mismatch test fails
- Missing schema.rs CHECK entry -> runtime insert fails (CHECK constraint violation)
- Missing exhaustive_match_guard arm -> compile error (non-exhaustive match)

After editing, regen the golden file:
  cargo test --test action_type_lock -- --ignored --nocapture

Then full check:
  cargo check (plain, via depot shim — NEVER absolute-path cargo)

## H-18 EVALUATE_DEFLATION API (the loosen significance gate)

```rust
// deflation_gate.rs:313
pub async fn evaluate_deflation(
    ledger: &TrialLedger,
    ssb: &Arc<Ssb>,
    input: DeflationInput,
) -> Result<DeflationGateVerdict, DeflationGateError>

// DeflationInput fields (deflation_gate.rs:160):
//   objective: String, data_window: String, sr_observed: f64,
//   t_periods: u64, skew: f64, excess_kurt: f64, sharpe_std: f64,
//   sr_benchmark: f64, pbo: Option<f64>, seed: u64

// DeflationGateVerdict (deflation_gate.rs:135):
//   kind: GateVerdictKind { Allow, Block }
//   reason: String, dsr: f64, sigma_sr: f64, sr_0: f64, min_trl: f64, n_used: u64
```

The loosen path maps the user's own-run FP evidence to DeflationInput:
- objective = "process_fp_rate_<domain>" (P-7-specific session)
- data_window = caller-supplied evidence window
- sr_observed = the measured FP-rate improvement (per-period)
- t_periods = number of observation periods
- The caller supplies these via the --evidence CLI arg (JSON or structured)

## ASYMMETRY ENFORCEMENT (M-2 at process grain, implemented fresh — M-2 API does NOT exist)

The change function enforces:
- TIGHTEN: always allowed. gate_strictness increases, autonomy decreases, explanation_verbosity decreases. No evidence needed.
- LOOSEN: REFUSED unless caller supplies measured own-run FP evidence. If evidence supplied, routes through evaluate_deflation. Promotes ONLY on GateVerdictKind::Allow. On Block, refuses with the gate's reason.
- Cross-user or global FP evidence NEVER satisfies a per-user loosen (P-7:81 REDTEAM patch).
- User affect/frustration NEVER produces FP evidence (P-7:81 REDTEAM).

## OBSERVE-FIRST + REVERT (GD-4 + H-49)

- Every profile change emits the `operating_profile_change` SSB row BEFORE applying (observe-first).
- The SSB row carries {domain, field, old_value, new_value, evidence_ref, change_kind}.
- Revert restores the prior value from the journaled old_value.
- A revert is itself a journaled change (change_kind=Revert).

## CLI SURFACE

```
elai profile show [--domain <domain>]
elai profile tighten --domain <domain> --field <field> --value <value>
elai profile loosen --domain <domain> --field <field> --value <value> [--evidence <json>]
```

- tighten: applies directly, emits SSB, prints confirmation
- loosen without --evidence: REFUSED, prints error explaining evidence requirement
- loosen with --evidence: parses evidence, calls evaluate_deflation, promotes on Allow / refuses on Block
- show: prints current profile state

## PRODUCTION_HOOK (R-8 binding)

The production caller is the `elai profile` CLI subcommand dispatch in cli/src/main.rs.
This calls profile_cmd::run which calls the operating_profile change function + SSB emit.
The new pub symbols (OperatingProfile, DomainProfile, change functions) MUST be reachable
from this CLI entrypoint in the SAME changeset. A #[cfg(test)] caller does NOT satisfy R-8.

Self-verify before returning: `python .claude/tools/production_caller_check.py --files <changed .rs>`
A DARK new pub fn = BLOCKED return, never green DONE.

## SCOPE RISKS

1. SCHEMA_VERSION is DERIVED (Tier-B Move 2): no hand-bump needed. SSB_ACTION_TYPES.len() auto-increments. But the golden file MUST be regenerated.
2. The trial ledger is session-scoped and finance-keyed (strategy_trials table). P-7's manual path uses a P-7-specific session_id (e.g. "elai-profile-<domain>"). This is acceptable for Tier-1 — the evidence is caller-supplied, not auto-harvested.
3. P-5/P-2/P-8 composition is DEFERRED. The three defaults are DATA only. Do NOT attempt to wire to sibling units that don't exist.
4. M-2 monotone-tighten API does NOT exist. The asymmetry is implemented IN the change function itself.
5. N-3's candidate-family re-keying is NOT required. The manual path supplies evidence explicitly.
6. E-5 content adaptation is DISJOINT. P-7 is PROCESS only (autonomy, gate_strictness, explanation_verbosity). Do NOT touch E-5's path.

## CURRENT PLAN-STANDARD RISKS

- P-7 is PLANNED, not in elai-status.json. No prior code exists.
- The FORK clause (P-7:41) is explicitly anticipated by the plan. The amended Tier-1 is plan-compliant.
- P-AGGREGATE.md:87 confirms P-7's SSB action type is `operating_profile_change` — matches the plan.
- The drain card (cards-MNOPQ.md:439-445) confirms: state=dark, 0 refs, depends N-3, superseded_by=none.

## SUMMARY OF TOUCHPOINTS (10 files, 2 new + 8 edits)

NEW: orchestrator/src/operating_profile.rs
NEW: cli/src/profile_cmd.rs
EDIT: orchestrator/src/lib.rs (add mod declaration)
EDIT: orchestrator/src/ssb/action_type.rs (enum + as_sql_str + from_str)
EDIT: orchestrator/src/ssb/payloads.rs (SsbPayload variant + action_type() + payload struct + Kind enum)
EDIT: orchestrator/src/ssb.rs (SSB_ACTION_TYPES array)
EDIT: orchestrator/src/ssb/schema.rs (CHECK constraint DDL)
EDIT: orchestrator/tests/action_type_lock.rs (exhaustive_match_guard + regen golden)
EDIT: cli/src/args.rs (CliAction variant + dispatch + parse function)
EDIT: cli/src/main.rs (mod declaration + dispatch arm)

End of P-7 S1 Context Package.
