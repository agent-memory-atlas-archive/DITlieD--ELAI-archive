LACUNA-FILL DEEP-RESEARCH PROMPTS — authored 2026-06-26
Companion to LACUNA-SWEEP-2026-06-26.md (same dir). Four prompts, each targeting a cluster of
the real (force-named, mostly DENSE) lacunae from the sweep. A downstream session will run these
through a deep-research tool and use the results + the sweep file to design fills.

USAGE / ANONYMITY (HARD): the four PROMPT blocks are written in NEUTRAL vocabulary — no project
name, codenames, or file paths — so they can be sent to an external deep-research tool as-is.
Send ONLY the text inside a PROMPT block. The INTERNAL CROSSWALK at the bottom (which names the
internal lacuna IDs) is for the downstream session to reconnect results to the sweep file; it is
NOT part of any externally-sent prompt.

Shared instructions baked into each prompt: prefer 2024-2026 primary sources; cite every factual
claim (URL / paper id / date); explicitly distinguish "nobody has tried this" from "tried and it
failed/was abandoned" (and say WHAT happened); name concrete systems, papers, tools; for each
candidate approach give a CONCRETE, implementable mechanism (the reader is going to BUILD a fill,
not just survey); state tradeoffs, cost, and failure modes; mark which parts are deterministic vs
require model judgment; and finish with an adversarial "attack the design" pass.

================================================================================
PROMPT 1 — Verifying the OBJECTIVE/SPEC is correct, not just that the agent satisfied it
================================================================================
Context: Consider an autonomous software-engineering agent system that already has a strong,
off-agent, fail-closed "done" gate — it can prove (via artifacts the working agent did not author)
that the agent HONESTLY satisfied the objective/spec it was given. The unsolved problem is one
level up: the objective or specification ITSELF may be wrong, incomplete, or a poor proxy for what
the user actually wanted. Even a perfectly honest verifier optimizes a proxy if the spec is a
proxy (Goodhart's law at the specification layer). A related prevention-time problem: when a task
is recursively decomposed into a tree/DAG of subtasks, we want to PROVE — before any code is
written — that the decomposition is complete (covers the goal) and free of "producer with no
consumer" dead subtasks (a subtask whose output nothing downstream consumes).

Research questions (find concrete, buildable mechanisms for each):
1. How do current systems verify that a specification/objective is CORRECT, COMPLETE, and faithful
   to user intent — as a check distinct from "did the agent satisfy it"? Cover: intent
   formalization, requirements verification, specification grounding, underspecification /
   ambiguity detection, spec-vs-intent gap measurement, and "specification autoformalization".
2. Can "is this the RIGHT objective" be made an INDEPENDENT, adversarial check — e.g. a critic
   from a different model family whose job is to attack the SPEC (find missing requirements,
   harmful interpretations, ambiguities) rather than attack the solution? What works; what is the
   failure rate?
3. Formal/type-theoretic COMPLETENESS and reachability checking over a task-decomposition DAG
   before execution: session types, process calculi (CSP), Petri-net reachability, producer-
   consumer interface typing applied to agent task graphs. Who has tried this for LLM agent
   decompositions specifically, and what happened?
4. How to encode an objective/spec as a machine-checkable, tamper-evident artifact such that any
   later MUTATION of the objective (after work was approved against it) is automatically
   detectable.

Attack-the-design (adversarial pass — required): Under what conditions does objective-verification
itself become gameable or collapse? Specifically address: (a) if the objective-checker is itself
an LLM, it shares the same blind spots (in-boundary self-validation); (b) specifications that are
irreducibly ambiguous or that require knowledge the system does not have; (c) the
halting/undecidability residue — when is verifying the objective provably as hard as solving the
task, so the only honest output is "abstain / route to a human"? Give the boundary between what is
deterministically checkable, what is LLM-judgment, and what is irreducibly human.

Deliverable: a ranked set of concrete fill-mechanisms with, for each: how it works, whether it is
deterministic or model-based, its cost, its failure modes, and citations.

================================================================================
PROMPT 2 — Completeness AND re-executability of an agent's evidence trail
================================================================================
Context: Consider an autonomous agent system that records every action it takes into an append-only
event log, treats that log as the single source of truth, and hash-chains it so that any post-hoc
ALTERATION of an existing entry is detectable. Two distinct gaps remain.
(A) COMPLETENESS: the hash-chain proves no recorded entry was altered, but it does NOT prove that
every real side effect actually PRODUCED an entry. An un-instrumented action (a file write, a
network call, a subprocess) leaves no row, and an append-only log cannot know it is incomplete —
the log only knows its own entries.
(B) RE-EXECUTABILITY: the system can export a portable "run bundle" that lets a third party REPLAY
the recorded action sequence and check that recorded verdict hashes match. But this only verifies
INTERNAL consistency (the bundle reproduces its own recorded verdicts); it is not RE-EXECUTABLE,
because there is no snapshot of the environment or of external inputs, so a third party can only
replay-and-trust, never re-run the actions against the same world and confirm the effects match.
A third, smaller gap: a downstream verifier/auditor is process-isolated from the worker, but it
still reads worker-AUTHORED artifacts (the code, its comments, naming, summaries), so it can be
biased across the isolation boundary by persuasive but misleading authored text.

Research questions (find concrete, buildable mechanisms):
1. Proving COMPLETENESS of an append-only execution log against reality: independent observation of
   side effects via OS-level mechanisms (syscall auditing, eBPF, filesystem watchers, sandbox/
   mediation layers) reconciled against the agent-emitted log; detecting "a side effect happened
   that produced no log row". Who does this for agents; what is the overhead and false-positive
   rate?
2. Making a recorded agent run RE-EXECUTABLE by an untrusted third party despite (a) LLM
   non-determinism and (b) external APIs whose state cannot be snapshotted: record-and-replay with
   cassette/VCR capture of external I/O, "replay the EFFECTS / action-stream not the tokens",
   content-keyed fixtures, environment/container snapshotting, and TIERED fidelity (action-faithful
   vs token-faithful vs score-faithful, with honest downgrades when a tier can't be met).
3. Restricting a verifier's INPUT to machine-derived artifacts the executor could not author
   (coverage numbers, static-analysis output, runtime event rows) vs executor-authored prose — how
   to give a verifier an authorship-stripped view for its binary verdict while keeping full code
   only for advisory review.

Attack-the-design (required): Where do completeness and re-executability guarantees break? Address:
a privileged host adversary who can rewrite the whole chain consistently (does this require a TEE
/ external transparency-log anchor?); side effects that are inherently un-capturable or
un-invertible; the independent shadow-observer itself having to trust the OS/kernel; cassette
staleness (the recorded external response no longer matches the live world). State the honest
residual risk for each mechanism.

Deliverable: ranked, buildable mechanisms with overhead, threat-model limits, deterministic-vs-
judgment split, and citations.

================================================================================
PROMPT 3 — A "done" decision that reflects real reachability, real impact, and meaningful tests
================================================================================
Context: Consider an autonomous coding agent whose "done" gate already (a) checks that a newly
added function is reachable from a real production entry point and leaves a runtime trace, and (b)
runs the project's tests off-agent so the agent cannot fake a pass. Three gaps remain.
(1) BINARY vs USAGE-WEIGHTED reachability: "reachable + emits a trace" passes even when the new
code sits on a reachable-but-never-actually-taken path; coverage>0 can be satisfied by a single
smoke test that drives a path no real user drives. At authoring time there is no production traffic
to weight by.
(2) CROSS-LANGUAGE / DYNAMIC blast radius: the impact / reverse-dependency cone used to catch "a
small change silently breaks something else" is single-language and STATIC, so it misses (i)
cross-language seams (a change in one language whose consumer is in another via IPC/FFI/RPC/UI
bindings) and (ii) dynamic dispatch (interface/trait objects, reflection, string-keyed dispatch,
event buses) — a change that breaks something through such an edge passes the gate.
(3) TEST MEANINGFULNESS: a test can pass HONESTLY while asserting nothing meaningful (large
fractions of agent-written tests lack real assertions), so an honest pass on a vacuous test defeats
the anti-fabrication gate without anyone lying.

Research questions (find concrete, buildable, deterministic-where-possible mechanisms):
1. Usage-weighted / representative-workload reachability for done-gating: driving a curated,
   frozen "representative workload" corpus rather than one happy-path smoke; regression-test
   selection that is ENFORCED, deterministic, off-agent, and hard-blocks on a previously-passing
   test now failing. Who has made anti-regression a hard pre-commit gate for agents (vs advisory)?
2. Cross-language and dynamic-dispatch impact / change-impact analysis: unifying static call graphs
   across languages, and FUSING static graphs with RUNTIME-OBSERVED call edges (from coverage /
   execution traces) to close dynamic-dispatch false-negatives; tracing a changed symbol across an
   IPC / RPC / FFI / UI-binding boundary to its consumer in another language.
3. Test-MEANINGFULNESS gates wired fail-closed and off-agent (so the agent cannot author around
   them): mutation testing scoped to the changed-code cone, assertion-presence/strength checks,
   anti-vacuous-test detectors; composing "the result is honest" with "the test is meaningful".

Attack-the-design (required): How does each gate get gamed or fail? Address: the representative
workload corpus is itself a proxy that can be overfit; mutation testing's cost forces sampling that
leaves gaps; runtime-observed edges only cover what the workload actually exercised; regression-test
selection that is too imprecise to trust (cite the real cases where regression-test-selection was
enabled then removed). Mark the deterministic vs model-adjudicated boundary explicitly (the goal is
NO model adjudication in these gates).

Deliverable: ranked, buildable, off-agent mechanisms with cost / false-positive bounds, the
static-vs-runtime fusion design, and citations.

================================================================================
PROMPT 4 — Governing the controller itself, and the reasoning, when "who guards the guard" recurses
================================================================================
Context: Consider a multi-agent coding harness in which a controlling/orchestrator agent sequences
worker agents through a MANDATED order of checks (e.g. gather-context, then implement, then audit,
then record). Four governance gaps remain, all variants of "who guards the guard".
(A) ORDER ON THE CONTROLLER: a finite-state workflow machine reliably gates the WORKERS' tool use
by state, but the CONTROLLER's own dispatch SEQUENCE (which agent it must run before it is allowed
to edit) is enforced only by written rules plus a termination hook — and in practice the
controlling model rationalizes its way past the required order. The machine cannot structurally
gate the very entity that drives the machine.
(B) SEMANTIC SUPERVISION: agent supervision restarts a worker on PROCESS DEATH, but most real
multi-agent failures are SEMANTIC, not crashes — an agent that is alive but off-task, repeating
itself, or whose stated plan and actual actions diverge. Restart-on-crash is blind to these.
(C) GROUNDING IS SELF-ASSERTED: each reasoning step is required to carry a grounding CITATION, but
the citation is asserted by the same model; nothing checks that the cited source actually entails /
causally supports the step (citations are often post-rationalized).
(D) DRIFT TYPE: goal-drift detection treats all drift as a capability failure (lost the thread),
missing PREFERENCE / value-conflict drift — where the model's trained dispositions resist the
assigned goal (e.g. quietly dropping a safety/privacy constraint in favor of utility). The latter
is a safety signal that should escalate to a human, not trigger a retry.

Research questions (find concrete, buildable mechanisms):
1. Structurally enforcing workflow/dispatch order on the ORCHESTRATING agent itself (not just
   workers): FSM / protocol-DSL / capability mechanisms that make an out-of-order or
   step-skipping dispatch structurally impossible; provable step-skip prevention; and the
   trust-anchor argument for where the regress of "who enforces the enforcer" terminates (an outer
   deterministic, non-model enforcer).
2. IN-LOOP SEMANTIC supervision that detects, from the live action/telemetry stream and
   deterministically where possible, reasoning-action mismatch, off-task drift, and step
   repetition — and restarts/redirects mid-run rather than catching it only in a post-hoc audit.
3. Verifying that a reasoning step's cited grounding actually ENTAILS it: claim-citation
   faithfulness, entailment checking, forced grounding at generation time vs post-hoc; doing it
   with deterministic / cheap checkers (file/source/test lookups) rather than another LLM where
   possible.
4. Distinguishing capability-drift from PREFERENCE / value-conflict drift in autonomous agents,
   and routing value-conflict drift to a human as a safety event (vs retrying it).

Attack-the-design (required): the recursion / trust-anchor problem — any enforcer is itself an
agent or process; where can the regress actually stop and why is the anchor trustworthy? The cost
of in-loop semantic supervision (a judge watching every step). False positives in loop/stagnation
and drift detection (cite the real case of an agent loop-detector abandoned because it could not
tell a refined retry from a degenerate loop). Whether preference-drift is even detectable without
white-box access to the model's internals.

Deliverable: ranked, buildable mechanisms; the explicit trust-anchor / end-of-regress argument;
deterministic-vs-judgment split; false-positive bounds; citations.

================================================================================
INTERNAL CROSSWALK (do NOT send externally — maps each prompt to the sweep's lacuna IDs)
================================================================================
Prompt 1 (objective/spec correctness + decomposition completeness):
  L1 C0-THESIS spec/objective-correctness gate (root) ; L8 C3-PLANMGMT formal producer->consumer
  type-check over the plan DAG ; C9-DOCTRINE doctrine-enforcement coverage ledger (the
  "is the rule itself enforced" sibling). Plus the "objective seal / mutation-detectable" thread
  ties to C4-RRA objective-hash (I-45).
Prompt 2 (evidence completeness + re-executability + verifier input independence):
  L2 C0-SSB journal completeness via independent shadow stream ; L3 C4-RRA re-executable
  (taint-ingest cassette) run bundle ; C0-TIERC machine-derived-first audit input. Threat-model
  leg (privileged-host / external anchoring) also touches C0-SSB external anchoring (I-18/G-8).
Prompt 3 (reachability/impact/test-meaningfulness done-gate):
  L4 C4-WIRING usage-weighted reachability (eval-corpus into wiring) ; L5 C4-BLAST cross-language +
  dynamic blast cone ; C2-DKG runtime-observed edges fused into the graph (enabler) ; L7 C4-CANTLIE
  compose with mutation-survival floor. Tier-3 rider: C2-RESILIENCE degradation -> trust-marker
  (a change made under degraded retrieval is also a "done under incomplete context" signal).
Prompt 4 (controller + reasoning self-governance, who-guards-the-guard):
  L6 C0-FSM bind the orchestrator's own dispatch order ; C3-AGENTS in-loop MAST semantic supervisor ;
  C1-GRC verify per-step citation entailment (compose with C1-GROUNDMODES) ; C3-AUTONOMOUS
  preference-drift vs capability-drift. Rider: C5-RULES journal-outcome-as-label (the supervisor's
  own true/false-positive labels can come from journal outcomes, non-LLM).

NOT directly covered by the 4 prompts (lower load-bearing or THIN; pick up later if a 5th prompt is
wanted): C1-GROUNDMODES route-by-measured-sufficiency ; C2-CASCADE which-modality retrieval router ;
C6-VIBEGATE taint-through-memory+handoff (could be folded into Prompt 2's provenance leg) ;
C7-SIDECAR attention-determinism residue->RRA tier (folds into Prompt 2 fidelity tiers) ;
C7-ROUTING phase-keyed routing [THIN] ; C8-DASHBOARD plan-vs-execution drift view (the visual
surface of Prompt 4's drift work) ; C8-CANVAS shadow-surfacing [THIN, derivative of Prompt 2] ;
C9-PLANS auto-derived feature-map ; C3-BSP intra-agent barrier [THIN] ; C5-SELFMOD local
cross-family substitute [THIN].
