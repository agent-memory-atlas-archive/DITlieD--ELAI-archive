# E-4 chat UX critique — seam-design proposal (doc-only)

session: opuswalk-E-4-PLAN-1786040439-ca1411  date: 2026-08-07
actor: synthesizer-elai on posthog-fable-5  role: PLAN  writes: this doc only
base: worktree HEAD a9bef08 (branch conductor/opuswalk-E-4-PLAN-1786040439-ca1411-3d5cd3b4)
plan: docs/thefinalglance/E-plan/E-4_chat_ux_critique.md (230 lines, read in full)

QUEUE REASON (cited from TaskSpec): "SET_ASIDE: greenfield-undesigned WAVE-B producer seam; no frozen
acceptance contract (e4_chat_ux_critique_contract.rs absent) + net-new commands_e4.rs producer; substrate
present/serde-wired, prod consumer deferred. K-27 owner/design-gated."
GATE_JSON_REF: conductor/opus_walk/evidence/E-4/S3-disposition.md (SET_ASIDE 2026-07-12, plan-read CONFIRMED same day).

## 0. Drift since the S3 disposition (all re-verified by grep/read in this tree)

| Prior claim (S3 / substrate header / allowlist) | Verified reality at HEAD a9bef08 |
|---|---|
| `pub mod chat_ux` at lib.rs:117 | orchestrator/src/lib.rs:136 |
| ChatUxCritiqueEvent parse at action_type.rs:6283 | action_type.rs:6470 (variant :3064, as_str :5536) |
| registry row ssb.rs:949 | ssb.rs:955 |
| payloads.rs:4356 as_action_type; payload struct :12749 | payloads.rs:4615; ChatUxCritiqueKind :13277; ChatUxCritiqueEventPayload :13368; env const :13388 |
| allowlist E-4 row at wire-dark-allowlist.toml:948-961 | :959-972, still a COMMENTED-OUT `[[flag]]` block; its "Const ssb.rs:25174" note is stale (ssb.rs is now 1741 lines, split into ssb/) |
| chat_ux/mod.rs header: doom-loop anchor = task_runner.rs `derive_autonomous_evidence` defn :11379, call :6438 | defn :17846, call :9004. Line 6438 is now B-10 dispatch_context/ContextBudgetComputed code. The header comment is itself stale — third drift of this anchor |
| e4_chat_ux_critique_contract.rs absent | still absent (ls verified). `focus_tunnel_e4_contract.rs` exists but is B-28's internal "E-4" (StreamingValidator), a different namespace — do not confuse |
| commands_e4.rs absent | still absent (only commands_e40/e41/e46.rs) |
| ChatUxCritiqueEventPayload constructed only in tests | still true: sole constructor ssb/tests.rs:3263 |

Unchanged: substrate PRESENT (13-arm chat_ux/mod.rs), SSB envelope serde-wired for round-trip, zero
production emit. The queue reason survives drift; the line numbers did not.

## 1. The architectural fact the plan body missed (verified, load-bearing)

`elai-orchestrator` DEPENDS ON `elai-reasoning` (orchestrator/Cargo.toml:15). chat_reflect.rs:249-253
documents it: the `ReflectEventSink` trait exists precisely because the reverse dependency would be
circular. Therefore the plan's Task 2/6/7 instruction "ship in chat_reflect.rs AUDITOR/TRIAGE stage"
CANNOT literally use the chat_ux types or emit ChatUxCritiqueEvent from inside the reasoning crate —
`SycophancyGuard`, `CompletionClaimCheck`, and the SSB payloads all live in the orchestrator crate.

The production join point ALREADY EXISTS and is ALREADY WIRED:

- `impl ReflectEventSink for Ssb` — orchestrator/src/ssb/store.rs:1179. Fire-and-forget, maps each
  ReflectEvent to an SSB row. Its BudgetSoftWarn arm is the exact wire-dark precedent: emit-time
  `env_gate_truthy(ELAI_B39_SOFTWARN_ENABLE_ENV)` check (store.rs:~1228).
- ReflectEvent arms (chat_reflect.rs:255): Triaged / Candidate / AuditVerdict / Refined / BudgetSoftWarn.
  `ReflectEvent::Refined { total_tokens_used, synthesis }` is emitted at chat_reflect.rs:535 AFTER Stage-4
  winner selection — `synthesis` carries the final reply text.
- Production callers of the pipeline (real entrypoints, verified):
  - CLI: live_cli.rs:1793 constructs `ChatReflectPipeline::new(cfg, sink)` with the Ssb sink; :1808 runs it;
    :1844 pushes the winner as the assistant reply. Reached via the `/reason` toggle on `elai` live chat.
  - Dashboard: dashboard-v2/src-tauri/src/commands.rs:7235 (+ reflect_bridge.rs:121 TauriEventSink) — same
    pipeline, Tauri side. NOT touched by this proposal (owner UI hold).
- AUDITOR refine loop at chat_reflect.rs:410-480 (initial call :421, verdict parse :427, AuditVerdict emit
  :452, winner select :475). The substrate header's "379-393" points at the Chat short-circuit just above.

So the seam is NOT the vapor `commands_e4.rs` (dashboard Tauri surface, held) and NOT an in-reasoning-crate
emit (circular). It is the Ssb sink impl, which serves both production callers with zero dashboard edits.

## 2. Owner UI hold (2026-08-06) applied to the 14 tasks

E-4 is not on the untouchable list, but tasks whose PRIMARY deliverable is Svelte/webview stay held:
Tasks 1, 3, 5(card), 10(button), 11, 12, 13 — all name CommandWindow/ActivityWindow/new .svelte components.
Backend-only candidates that emit SSB a future UI can consume: Task 2 (sycophancy guard), Task 6
(completion-claim guard), Task 7 (clarifier), Task 4 (re-anchor). Per the S3 disposition, Tasks 2 and 6 are
the most self-contained; Task 6 is the more deterministic of the two (pure string-trigger + evidence-count
check, no model-behavior dependency), so it is slice 1.

## 3. Slice 1 (the ONE vertical slice): observe-only completion-claim guard

Substrate API already shipped + unit-tested, test-only callers today:
`CompletionClaimCheck::evaluate(&self, output_text, evidence_count) -> CompletionClaimVerdict`
(chat_ux/mod.rs:340; verdicts AllowedWithEvidence / RejectedNoEvidence / NotApplicable), EvidenceRef 3-arm
enum (mod.rs:113: FileRef / SsbSeq / DetCheck). Kind arm exists: `ChatUxCritiqueKind::
CompletionClaimRejectedNoEvidence` -> wire label `completion_claim_rejected_no_evidence` (payloads.rs:13277+).

PRODUCTION_HOOK (file:line, this tree):
- orchestrator/src/ssb/store.rs:1179 `impl ReflectEventSink for Ssb`, the `ReflectEvent::Refined` arm:
  after mapping to ReflectRefined, ALSO run `CompletionClaimCheck::evaluate(&synthesis, evidence_count)`;
  on RejectedNoEvidence and `env_gate_truthy(ELAI_CHAT_UX_CRITIQUE_ENABLE_ENV)` (payloads.rs:13388), write
  a `ChatUxCritiqueEventPayload { kind: CompletionClaimRejectedNoEvidence, session_id, turn_id,
  details_json: {trigger, evidence_count, claim_blake3}, ts_ms }` row. Exactly the BudgetSoftWarn pattern.
- evidence_count source, slice 1: a new `pub fn extract_evidence_refs(&str) -> Vec<EvidenceRef>` in
  chat_ux/mod.rs (deterministic parse: `path:line` file refs, `ssb#<seq>`, det-check tokens), called from
  the store.rs hook in the SAME changeset (D-030 clean, no dark symbol). Upgrade path (slice 2): a new
  ReflectEvent arm threading real AUDITOR evidence, which Task 2's SycophancyGuard needs anyway.
- Reasoning crate: ZERO edits in slice 1. Dashboard: ZERO edits (sink serves it for free).

Frozen acceptance contract (auditor-authored BEFORE executor starts; K-27 executor never authors its gate):
`crates/contracts/tests/e4_chat_ux_critique_contract.rs` asserting (1) env=1 + winner "Done." with zero
evidence refs => exactly one `chat_ux_critique_event` row with kind `completion_claim_rejected_no_evidence`;
(2) winner containing a `file:line` citation => no reject row; (3) env unset => zero rows (wire-dark
honored); (4) payload serde round-trip (pattern exists at ssb/tests.rs:3263).

Same-changeset registration: un-comment the wire-dark-allowlist.toml:966-972 `[[flag]]
ELAI_CHAT_UX_CRITIQUE_ENABLE` row with smoke_test = the now-real contract path. This kills the row's own
R1 fail-open deferral reason ("would point at a vapor path"). Refresh the stale `ssb.rs:25174` comment.

Size: ~60-100 LOC production (store.rs arm + chat_ux extractor) + ~150 LOC contract + allowlist row.
Explicitly OUT of slice 1: any reply mutation/blocking, all Svelte surfaces, commands_e4.rs, doom-loop
task_runner hook (its anchor has drifted 3 times; re-anchor at its own slice), Tasks 1/3-5/7-13.

Pre-existing surfaces stay per workflow/reasoning/E-4-decisions.md:80-95 (E-4 ADDS, never replaces):
SsbActionType::SycophancyFlag (action_type.rs:1750), output_postprocessor::strip_sycophancy
(output_postprocessor.rs:29/55), the three dashboard sycophancy panels. NOTE: E-4-decisions.md and the
whole root `workflow/` tree are UNTRACKED in git — a scoped-mount executor cannot read them; the binding
content is pre-digested in conductor/opus_walk/evidence/E-4/design-dispatch-notes.md.

OPEN_QUESTION_FOR_OWNER (one): confirm slice-1 semantics = OBSERVE-ONLY (emit the reject row; never mutate
or block the reply). Enforcement changes chat output behavior and needs a UI affordance that is under the
2026-08-06 hold; recommend enforcement waits for the hold to lift.

VERDICT: BUILD-WITH-SEAM
