H-47 decisions — DGM eval firewall (date-gate + template-regen + holdout-budget binding)
Session: s-10612 | 2026-06-11

WHAT
H-47 composes three SHIPPED mechanisms into a single pre-score gate for
darwin_admit_loop::run_one_generation. Net-new wiring only (GD-1 P0).

COMPOSED (do not re-implement)
- D-6: is_task_after_cutoff + ModelTrainingCutoff (held_out.rs:879/862)
- E-54: ngram_filter + jaccard_overlap (held_out.rs:597/633)
- H-19: HoldoutBudgetLedger::query_holdout (holdout_budget.rs:520)
  The H-19 budget leg emits HoldoutBudgetEvent INSIDE query_holdout.
  H-47 does NOT add a new HoldoutBudgetKind arm (GD-1 P0).

NET-NEW (H-47 authors)
- eval_firewall.rs: date_gated_sample, regenerate_instance,
  bind_holdout_budget, indistinguishability_probe, FirewallSummary
- EvalFirewallContext in darwin_admit_loop.rs (optional injection bundle)
- EvalFirewallEvent SSB variant (Pattern-2 3-arm: PreUpdateItemRejected /
  TemplateRegenerated / IndistinguishabilityProbe)
- emit_eval_firewall_row in darwin_admit_loop.rs

DATE-GATE DESIGN (LiveCodeBench 2403.07974)
LiveCodeBench timestamps held-out items at release date; eligibility gate
scores only items NEWER than the last training cutoff (S3.SS1 temporal
eligibility predicate). ELAI mirrors this via ConfigVariant.created_at *1000
as the cutoff.date_ms anchor. Date proxy: HeldOutTask.ingested_ts_ms
(ingest time as a release proxy for synthesized tasks; ArxivRepro uses
fetched_at_ms when available — verified: held_out.rs:882).

Decision: gate uses STRICT inequality (task.release_date > cutoff.date_ms),
matching LiveCodeBench's own predicate. Equal means "possibly in training."

TEMPLATE-REGEN DESIGN (GSM-Symbolic 2410.05229)
GSM-Symbolic shows that regenerating instances from templates at score time
reduces overfitting to memorized instances (up to 65% score drop when one
irrelevant clause is added — Apple's finding). ELAI regenerates by applying
a blake3-seeded transformation per generation.

RISK-2 RESOLUTION: HeldOutTask.template_id (Option<String>)
HeldOutTask had no template_id or is_templated field. Two options:
  (a) Add optional template_id: Option<String> to HeldOutTask (nullable,
      backward-compatible via #[serde(default)])
  (b) Wrapper type at the call site (templated vs non-templated)

Chose (a): least-invasive, backward-compatible (existing serialized tasks
deserialize with template_id=None), no schema break on the SQLite body
column. Trade-off: wider struct for all callers, but Option<String> is 24
bytes and callers are read-heavy. The serde(default) guarantee means no
migration needed for existing rows.

THRESHOLDOUT BINDING (Dwork 1506.02629)
Thresholdout (Science 2015) bounds adaptive holdout overfitting to O(1)
INDEPENDENT of query count. A DGM loop issuing thousands of "did my score
go up?" queries is EXACTLY the adaptive analysis scenario the paper addresses.
H-19 already ships the ledger + DP noise. H-47 routes DGM score queries
through it. Budget exhaustion -> eval REFUSED (fail-closed R1/R2).

RISK-3 RESOLUTION: budget provisioning in tests
HoldoutBudgetLedger.budget_remaining() returns BudgetMissing (fail-closed)
without a provisioned row. Test fixture calls provision_fresh_holdout
(the only public, non-agent provision path) before running firewall.
budget_remaining(BudgetMissing).must_refuse() = true; the firewall propagates
that as FirewallError::BudgetExhausted.

RISK-1: DATE-GATE STARVATION
If ingested_ts_ms grows slower than ConfigVariant.created_at, date_gated_sample
returns None continuously, disabling fitness eval (fail-open on eval starvation).
Mitigation: run_one_generation emits WARN + continues without held-out eval
(execution-grounded fitness still runs). NEVER falls back to scoring a
pre-update item. Tier-1 frozen fixture avoids this; Tier-2 live path is the
separately-gated checkbox.

FIREWALL INSERT POINT
run_one_generation parameter: firewall_ctx: Option<&EvalFirewallContext>.
None in all existing test calls (backward-compatible). The production CLI
(darwin_loop_cmd.rs) passes None for now (Tier-2 live wiring is the separately-
gated checkbox per W-10 SPLIT). The firewall path IS reachable from the REAL
run_one_generation signature (R3/R4 compliance).

INDISTINGUISHABILITY PROBE (eval-awareness 2505.23836 / 2510.20487)
Gemini-2.5-Pro detects eval vs deploy with AUC 0.83 (2505.23836).
Sonnet 4.5 verbalizes "being evaluated" >80% of the time (2510.20487).
The probe measures Jaccard distinguishability of eval items from workload
traces. HIGH score = model CAN detect eval context (sandbagging risk).
VD-5 HONEST RESIDUE: the probe SURFACES the risk and does not eliminate it.
A strong model can detect evals from semantic cues not captured by n-gram
similarity. Never claim immunity; always name the residue.

COMPOSED vs NET-NEW SUMMARY
COMPOSED:
  - is_task_after_cutoff (D-6) - used verbatim
  - ngram_filter / jaccard_overlap (E-54) - used verbatim
  - query_holdout / HoldoutBudgetEvent (H-19) - called, not re-implemented
NET-NEW:
  - EvalFirewallContext struct (wiring bundle)
  - date_gated_sample wrapper (adds the cutoff predicate around sample_held_out_task)
  - regenerate_instance (blake3-seeded body transformation per generation)
  - indistinguishability_probe (1.0 - jaccard_overlap adapter)
  - FirewallSummary aggregate struct
  - emit_eval_firewall_row SSB helper
  - EvalFirewallEvent / EvalFirewallEventKind / EvalFirewallPayload structs
  - EvalFirewallContext injection in run_one_generation
  - HeldOutTask.template_id optional field
