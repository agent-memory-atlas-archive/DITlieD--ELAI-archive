E-47 LoRA ship-gate -- decisions log

Session: s-22300 tranche-B1-E47
Status: GATE LAYER SHIPPED; producer-side LoRA training stack
RUNPOD-PENDING per CLAUDE.md (Unsloth + QLoRA-NF4 + PiSSA + DoRA on
RTX 3090 at pythondev). The gate ships locally regardless because the
deferral was unenforced before this session (zero call sites for
`DEFERRED_PER_E47_REPORT` in the repo prior to this commit).

DECISION-1: Pattern-2 collapse vs 3 sibling SsbActionType variants.
  Chosen: ONE SsbActionType::LoraShipGateEvent + 3-arm
  LoraShipGateEventPhase {Refused, CalibrationMissing,
  CalibrationPassed}.
  Rejected: 3 sibling variants (lora_ship_gate_refused /
  lora_calibration_gate_passed / lora_calibration_gate_refused) as
  literally named in the plan body.
  Rationale: matches the E-43 SkillSystemEvent 3-arm precedent +
  the documented 43-application Pattern-2 ledger in ssb.rs. Saves
  2 SCHEMA slots + 2 ALL_VARIANTS rows + 2 alphabetical inserts.
  Bumps SCHEMA_VERSION 191 -> 192 and TOTAL_VARIANTS 464 -> 465 as
  a single delta.

DECISION-2: Deferral-token semantic (triple-AND, not double).
  Chosen: triple-AND gate (enabled + confirm_user_understanding +
  EITHER deferral-token-flipped OR calibration-passes).
  Rejected: simple two-flag gate (enabled + confirm).
  Rationale: the source report
  workflow/improvement-sweep-2026-05-19/reports/47-lora-finetuning.md
  cites 5-19pp catastrophic forgetting (arXiv:2405.09673 Biderman et
  al.). A two-flag gate cannot enforce no-regression. The calibration
  baseline is the third leg.

DECISION-3: Wire-dark default OFF for env gate
ELAI_LORA_SHIP_GATE_ENABLE.
  Chosen: env var unset = SSB row NOT emitted; pure-logic gate
  still evaluates and returns the outcome to the caller.
  Rejected: producer always emits (would pollute SSB for operators
  who never enabled the gate).
  Rationale: mirrors the precedent for
  ELAI_SKILL_SYSTEM_ENABLE / ELAI_ROUTE_DECISION_SSB_ENABLE /
  ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE. Wire-dark posture is the
  canonical ship pattern for substrate variants per ssb.rs ledger.

DECISION-4: CI marker placement -- TOML field, not env var.
  Chosen: `ship_gate = "DEFERRED_PER_E47_REPORT"` literal in
  .elai_cc/elai_cc.toml [finetune.lora] block. Lint script greps
  the toml file directly.
  Rejected: env-var marker (would not survive a fresh checkout
  without the operator setting it manually) or sentinel file
  (additional state to track).
  Rationale: the toml is checked into git; the deferral token
  survives every checkout and is grep-able by CI without any
  runtime state. Mirrors the source report's "string literal in
  TOML" recommendation.

DECISION-5: Manual action_type_lock.golden regen.
  Chosen: stdlib-only Python script regenerates the golden file
  in-place by appending `lora_ship_gate_event` to the tag list,
  re-sorting alphabetically, and bumping the "Total variants:"
  header line.
  Rejected: `cargo test regen_golden_file -- --ignored` per the
  memory rule feedback_no_cargo_test_unless_permitted (auditor runs
  cargo with explicit permission, not the main LLM).
  Rationale: the regen logic is deterministic; the auditor's later
  cargo check / cargo test pass against the same golden file will
  catch any drift from the alphabetical-sort invariant.

DECISION-6: No new Cargo deps.
  Chosen: reuse serde_json + toml (both workspace-pinned) for the
  CLI handler.
  Rejected: any new crate (peft / unsloth / etc. would be Python,
  not Cargo; not needed at the gate layer).
  Rationale: plan stub_ban on peft/unsloth Cargo deps; anti-
  slopsquatting hard rule requires verifying every new dep.
  Reusing existing workspace pins avoids both gates.

DECISION-7: CLI subcommand `elai lora-gate-check` instead of boot-path
hook in `elai run`.
  Chosen: dedicated subcommand reading the toml block + evaluating
  the gate + printing the outcome.
  Rejected: hook in `elai run` boot path that refuses to start when
  the gate fires.
  Rationale: the producer-side adapter loader does not exist yet
  (RUNPOD-PENDING). Hooking `elai run` would force a noisy refusal
  on every CLI invocation without a clear adapter target. A
  dedicated subcommand surfaces the gate intent without changing
  unrelated CLI flows. The boot-path hook is the E-47.1 follow-on
  patch.

DECISION-8: Test naming convention mirrors plan literal names.
  Chosen: contract test fns named per the executor brief literal
  (test_lora_ship_gate_default_deferred_returns_refused / etc.),
  with snake_case wire-label alignment to the
  LoraShipGateEventPhase as_str labels.
  Rejected: paraphrased names.
  Rationale: future cross-check sessions read both the executor
  brief and the contract test file; literal-name alignment removes
  one source of drift between the two.

Revisit triggers (when to consider lifting the deferral)
  T1: measured prompt-rules+few-shot saturation gap of 5+pp vs the
      base model on the project-style probe (the Q1 gate in the
      flip-runbook).
  T2: a new Anthropic/Qwen measurement showing fine-tune > RAG for
      code-style transfer (replaces the CodeRAG-Bench finding).
  T3: a specific user request for a voice LoRA + explicit
      --consent-yes (Q4 gate in the flip-runbook). Voice LoRA at
      rank=8 only; not project-LoRA.

References
  Source report: workflow/improvement-sweep-2026-05-19/reports/47-lora-finetuning.md
  Plan: docs/thefinalglance/E-plan/E-47_lora_finetuning.md
  Cross-check: workflow/sessions/s-night-2026-05-30/cross-check/E/E-16_e47_e48_e49.md
  Memory: project_b40_shipped.md (Pattern-2 + SCHEMA bump precedent)
  Paper: arXiv:2405.09673 Biderman et al. (5-19pp forgetting)
