# L-1 decisions — typed claim extraction substrate

Unit: L-1, `docs/thefinalglance/L-plan/L-1_typed_claim_extraction_substrate.md`
Slice 1 landed and pushed at `68d5ad9` (2026-08-10).
Seam design ratified at D-226; seam re-verified live before the build.

## DECISION-1 — the production seam is the IPC accept loop, not a new report hook

The plan named "the report/claim seam (report_fns)". The ratified seam design (D-226) pointed
instead at `crates/orchestrator/src/ipc_accept.rs`, and the live code agreed: the terminal
`FrameRouteResult::Response(inner)` arm produced the agent's answer and threw it away with
`let _ = inner`. `AgentSessionOutcome` carried only `final_state` and `frames_handled`, so the
single production consumer (`task_runner.rs:1959`) could see Passed or Failed and nothing else.

We hold the text on `AgentSessionOutcome.final_content` and extract from the task-runner side.
Rationale: this is the orchestrator's own loop, it already runs on every real `elai` baby step,
and it keeps extraction OFF-AGENT by construction. The worker cannot reach the extractor; it never
types its own claims (TD-3, wiring R8).

## DECISION-2 — compose with the D-10 `claim.rs`, never merge with it

`crates/orchestrator/src/claim.rs` already defines `ClaimType`
(Factual / Procedural / Normative / Speculative / Hypothetical / Computational / MetaQuery) and a
`Claim { claim_id, text, claim_type, evidence, ts_ms }` that `select_grounding_mode` reads.

That taxonomy answers "what does this claim MEAN". L-1 answers "what does this claim REFER TO".
They are orthogonal, so L-1 ships as the submodule `claim/extract.rs` with its own `ClaimKind` and
an `ExtractedClaim` type. Nothing in `claim.rs` changed except the submodule declaration and the
re-export. This satisfies GD-1: the D-10 grounding methods are consumed downstream, not
re-implemented.

## DECISION-3 — resolution is a trait, so extraction stays pure

The plan wants an unresolved ref kept with `resolved = false`, and a resolved one marked true. A
filesystem probe inside the extractor would make it impure and untestable, so resolution is a
`ClaimRefResolver` trait with three defaulting methods. `extract_claims` uses `UnresolvedResolver`,
which resolves nothing; `extract_claims_with` takes any resolver.

Consequence stated plainly: in production today every ref is `resolved: false`, because the
production path passes the default resolver. That is honest bookkeeping, not a coverage claim. A
real resolver (path existence, LSP symbol lookup, lockfile probe) is a later slice.

## DECISION-4 — no enable flag

Plan clause W-2 says extraction is always-on intake with no behaviour gate, because the gate is
L-7's job. So the slice ships always-on and observe-only: it logs typed counts and changes no
control flow. This deliberately avoids a wire-dark-allowlist row and the out-of-band gate re-lock
that several sibling units are stuck behind.

## DECISION-5 — the cue lexicon, and its known imprecision

Backtick tokens are typed in this order, first match wins: `Api` (contains `(` and `->`), `Symbol`
(contains `::` or ends `()`), `Dependency` (`name@version` or `name = "version"`), `File`
(contains `/` or a known extension). Sentences are typed in this order: `Reachability`, `Causal`,
`Behavior`, by verb lexicon.

Order matters and is asserted by test. The lexicon will mis-type prose that mixes cues; that is
why the slice is observe-only first (GD-4). Bounding the mis-typing rate on real runs comes before
anything gates on it.

## Falsification observed, both directions

The wiring mutation is deleting the `crate::claim::extract_claims(...)` call in `task_runner.rs`.
Run off-agent by the controller before landing:

- mutated (call site removed): `production_caller_check` returned BLOCK, rc 1, with
  `extract_claims`, `unresolved_ref_count` and `count_of_kind` all DARK.
- restored (file byte-identical, sha256 `79feb443dc9e...`): PASS, 0 dark, 7 wired.

The three cargo gates were green in both states, which is the point: only the wiring checker sees
this mutation, so it is the checker that earns the verdict here.

## Named residuals — what this slice does NOT deliver

1. `taint: Option<Provenance>` (G-1) is absent. Four different `Provenance` types exist in the
   workspace and the accept seam carries no ingest provenance handle, so stamping one would be a
   decorative field. Slice 2 must first decide which `Provenance` and where the taint enters.
2. Plan tests (e) taint tag and (f) off-agent verification are not written, for the same reason.
3. No SSB row. The plan allows prefer-reuse or +1; the slice emits tracing only.
4. No frontend pair. Deferred to the L-2 risk tile per the plan's W-4 clause.
