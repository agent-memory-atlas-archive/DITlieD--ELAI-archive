C-49 — Iterative-distillation prompt-policy (decisions)

Session: s-25120 session 2/4 of the C-25120 completion-drive cluster.
Plan: docs/thefinalglance/C-plan/C-49_iterative_distillation_prompt_policy.md
Paper: arXiv:2605.14443 (Sayana, Todi, Jash 2026 — "Prompting Policies
for Multi-step Reasoning and Tool-Use in Black-box LLMs with Iterative
Distillation of Experience"). Interface map:
workflow/papers/iterative-distillation-interface.md.

WHAT SHIPPED (this session)
- New crate module crates/orchestrator/src/experience/ with:
    mod.rs        (ENV_ENABLE gate + module re-exports)
    fragment.rs   (PromptFragment + rank_score + blake3 text_hash)
    trace.rs      (SuccessTrace fields)
    corpus.rs     (ExperienceCorpus + MAX_TRACES_PER_SKILL=100 LRU +
                  record_success free fn)
    distiller.rs  (heuristic counting distiller + DEFAULT_MIN_FREQ=2 +
                  DEFAULT_MIN_SUCCESS_RATE=0.6 + max_emit=32)
    lookup.rs     (lookup_distilled_fragments query-time entry point)
- task_runner.rs gained run_experience_distill_post_execute_hook
  invoked after run_crystallizer_post_execute_hook on every baby-step
  audit-pass. Wire-dark gate: ENV_ELAI_EXPERIENCE_DISTILL_ENABLE
  (default off). Emits a single PromptPolicyDistillationRun SSB beacon
  per pass.
- ssb.rs +3 variants atomically with SCHEMA_VERSION 122 -> 123:
    PromptPolicyDistillationRun(PromptPolicyDistillationRunPayload)
    PromptFragmentSelected(PromptFragmentSelectedPayload)
    PromptFragmentDiscarded(PromptFragmentDiscardedPayload)
  9-tuple cascade applied (enum + SQL CHECK + as_sql_str + from_str +
  SsbPayload + action_type() + test_payload + all_test_payloads +
  golden / TOTAL_VARIANTS 343 -> 346 / count 282 -> 285).
- SkillManifest extended with `distilled_fragments: Vec<PromptFragment>`
  (#[serde(default)]) + set_distilled_fragments write-through setter.
  All existing SkillManifest literals (7 in manifest.rs, 4 elsewhere,
  2 contract tests) updated to default-initialise the new field.
- CLI surface: `elai experience distill|list|export|reset` via
  crates/cli/src/experience_cmd.rs. Synthetic-corpus path (GPU-free);
  the live persistent store is RUNPOD-PENDING.
- Contract test crates/contracts/tests/c49_iterative_distillation_contract.rs
  (G1..G7 fences).

KEY DECISIONS + GROUNDING

1. HEURISTIC adaptation, NOT RL prompter.
   Paper Algorithm 1 (sec 4.2.3) trains a small prompter policy
   pi_theta via Group-Relative Policy Optimization (GRPO) + Adam +
   learning rate 1e-5 + KL anchor (alpha >= 0) to a reference policy
   pi_ref. Reproducing it requires a feedback model M_FB and a scalar
   reward signal we do not yet compute. C-49 ships a COUNTING-BASED
   heuristic distiller only — fragments are ranked by
   `freq_used * success_rate` over the per-skill trace bin. The
   RL-trained prompter is RUNPOD-PENDING and documented as such in
   every module header.

2. Black-box vs controllable worker — minor divergence, intentional.
   Paper assumes a black-box worker M(x, p) -> y_hat. ELAI's local
   sidecar IS controllable (we can read its logits, change generation
   config, etc). The heuristic distiller does not exploit that fact;
   it works equally well whether the worker is black-box or not. We
   accept the lost optimisation and document the divergence here.

3. Buffer is TASK-DEPENDENT. Do not claim "buffer makes it work".
   Paper Section 5.3 footnote: Web of Lies (the largest raw gain in
   the paper, +37.62pp) ran WITHOUT the experience buffer. Buffer is
   not the load-bearing component for every task. The contract test
   intentionally does NOT assert "buffer helps"; it asserts only the
   structural shape (LRU cap, distill filter, lookup return type).
   Callers who do not want fragment biasing simply skip
   `lookup_distilled_fragments` (the boundary case at min_freq=1 is a
   near-passthrough, which is the no-buffer regime equivalent).

4. PER-SKILL binning, NO cross-model transfer.
   Paper Section 5.5 Table 4: prompter trained on Flash, deployed on
   Pro yields +7.5pp; on Flash-lite yields +5.0pp; in-distrib Flash
   yields +28.0pp. Cross-model transfer is poor. C-49 keys traces by
   `skill_name` ONLY; there is no cross-model or cross-skill sharing.
   The PromptFragmentSelectedPayload carries `skill_name_hash` (no
   plaintext) so dashboards can never accidentally aggregate across
   skills.

5. Audit-pass = TRUE is the heuristic equivalent of Eq. 5.
   Paper Eq. 5: `r_t^(i) >= r_max - epsilon` is the buffer-insertion
   gate (scalar reward with epsilon band). C-49 simplifies to the
   binary `audit_pass` boolean because ELAI's existing audit
   infrastructure produces a binary verdict. The richer reward-with-
   epsilon gate would require the feedback model M_FB; that path is
   RUNPOD-PENDING. The contract test asserts negatives (audit_pass =
   false) are NOT counted by the distiller (g2_distill_filters_audit_
   pass_false_traces).

6. Do NOT cite paper headline aggregates as ELAI expected outcomes.
   Paper abstract: "55% -> 90% on logic reasoning, 74% -> 91% on
   tool-use". These are aggregate / best-case ranges across BBEH + Tau-
   bench, vary by task, and rely on the RL-trained prompter that we
   don't yet ship. Per-table-row citation only:
     Dyck Languages       63.33% -> 91.25% (+27.92, WITH buffer)
     Web of Lies          52.50% -> 90.12% (+37.62, WITHOUT buffer)
     Tau-bench Retail     73.91% -> 91.30% (+17.30)
     Tau-bench Airline    65.27% -> 83.33% (+18.06)
     Disambiguation QA    57.08% -> 65.41% (+8.33, p<0.25 NOT sig)

7. Wire-dark env gating matches D-10/D-11/D-12/B-37 precedent.
   ELAI_EXPERIENCE_DISTILL_ENABLE is default-OFF. The post-audit hook
   is a transparent no-op until an operator opts in. Matches the
   sibling 4-application precedent (env-gated post-execute hooks
   already shipped in production).

8. ELAI heuristic adaptation marker on every artifact header.
   Every module header + payload doc + contract test + CLI handler
   includes the literal phrase "ELAI heuristic adaptation of
   arXiv:2605.14443" and the "RL-trained prompter is RUNPOD-PENDING"
   marker. This is to prevent future readers from mistaking the
   counting distiller for the paper's RL trainer.

RUNPOD-PENDING
- iterative_distillation_prompter_rl_qwen3_8b — train a small Qwen3-8B
  prompter via GRPO + KL anchor against a Qwen2.5-Coder-7B worker on a
  feedback model loop. Requires: feedback model M_FB selection +
  scalar reward function design + epsilon hyperparameter sweep + 195+
  RL training steps per Dyck reference (paper sec 5.2 early-stop at
  10 non-improving steps).
- Persistent on-disk corpus store. Current implementation is
  per-process in-memory; cross-session learning needs a serialised
  store under the project's .elai/experience/ directory (filesystem
  layout described in the original plan §3).
- Live distillation projection at the prompt-assembly site (consult
  corpus AFTER skill_registry::top_k_skills, inject fragments into
  the prompt with a ~500-token budget cap). The heuristic distiller
  IS in place; the call-site that consumes its output to bias prompt
  construction is the next surface to wire.

VERIFICATION
- cargo check -p elai-orchestrator (target).
- cargo check -p elai_cc_contracts --tests --test
    c49_iterative_distillation_contract (target).
- action_type_lock.golden regenerated to 346 entries; per
  action_type_lock.rs TOTAL_VARIANTS const + ALL_VARIANTS array +
  exhaustive_match_guard arms all updated in lockstep.
- all_test_payloads count assertion 282 -> 285 (3 new SSB variants).
- SCHEMA_VERSION 122 -> 123; DESTRUCTIVE comment block lists v121 /
  v122 session DB drop precedent.
