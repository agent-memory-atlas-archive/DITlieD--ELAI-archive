# ELAI spec-time critique lenses (native, re-authored — not imported)
Authored 2026-06-08. Origin: adapted from the "Automated Doubt" workflow (alexself.dev/blog/automated-doubt) into ELAI's doctrine. These are RE-AUTHORED prompts, not copies of github.com/aself101/agents-and-pipelines (importing that repo would trip .claude/rules/skill-import-policy.md E-43 mcp-scan whole-directory gate first).

## Doctrine (read before using any lens)
1. ADVISORY, NEVER LOAD-BEARING. These are LLM-judged critiques. LLM-judged critique is noisy and bypassable (MSR-2026: 60.2% of LLM-review PRs sit in the 0-30% signal band; NAACL Mar-2025: all 8 inline LLM defenses bypassed >50% ASR). They SURFACE issues to a human or feed the plan-improvement protocol. They NEVER emit an accept/block verdict. ELAI's off-agent deterministic gates (F-4 stdout_hash, coverage, SSB rows, mutable=0 rules, SAFEDELTA) remain the only thing that decides DONE. This is the §6 hard rule of ELAI-IMPROVEMENT-PLAN.md: ban LLM-judges from the load-bearing path.
2. WHERE THEY FIRE. DESIGN / SPEC time only — PLANNING, idea-decomposition (B-40 idea-critic), and the planning-workflow plan-improvement protocol (the 9-step adversarial injection). This is the front of the pipe, where the artifact is still prose/spec and a deterministic gate has nothing to hash. They do NOT run on shipped code (that is the auditor + the off-agent spine).
3. GROUNDING REQUIRED (drop unfounded critique). Every finding MUST cite a concrete location: a spec section heading, a file:line, a named symbol, or a quoted phrase from the artifact. A finding with no location is DROPPED, not surfaced — this is the anti-noise discipline (R5: an AI comment with no failing test / trace / reproducer is inadmissible; here, no concrete anchor = inadmissible). Mirrors ELAI's missing-evidence discipline applied to critique.
4. PARALLAX, BOUNDED. Run the relevant lenses in parallel (parallax coverage) but cap output: max 8 findings per lens per pass, deduped across lenses by (location, claim). Token cost is real (the source workflow admits 40+ agents is "not cheap"); scale lens count to artifact risk (SIMPLE spec: 2-3 lenses; HEAVY: full set).
5. CONVERGENCE, NOT PERFECTION. Iterate until findings drop to NITs or the operator accepts the residual. The operator is the judge (negotiation between artifact, lenses, operator).

## Finding schema (every lens emits a list of these)
- lens: the lens name
- severity: BLOCKER-CANDIDATE | CONCERN | NIT  (BLOCKER-CANDIDATE routes to the human; it NEVER auto-blocks)
- location: REQUIRED concrete anchor (spec section / file:line / symbol / quoted phrase). No location -> finding dropped.
- claim: one sentence, the specific defect.
- why_it_matters: the concrete downstream failure if unaddressed.
- suggested_resolution: the smallest change that resolves it (or "needs operator decision").
- confidence: HIGH (anchored in a quoted contradiction) | MED | LOW.

================================================================================
## THE LENSES
================================================================================

### L1 — Assumption Excavator  [universal; run on every spec]
FIRES_WHEN: any spec or plan that references an external interface, schema, API, or another module's behavior.
INSPECTS: the gap between what the spec ASSUMES an interface returns/does and what it ACTUALLY does.
PROMPT:
"You are the Assumption Excavator. Read the artifact and surface every place it silently ASSUMES the shape, return value, ordering, nullability, units, or behavior of something it does not control (an API, a schema, another module, a library, a model output). For each, quote the assuming phrase AND state what you would need to verify against source to confirm it. Prioritize spec-vs-real-interface drift. Example pattern: the spec assumes a schema returns {avgScore, passRate, lastRunDate} but the real schema returns {totalCount, recentCount, windowMinutes}. Do NOT speculate without an anchor — if you cannot point to the assuming phrase, drop it. Output the finding schema. Every assumption you surface that touches an unread interface is a candidate for ELAI's grounding-sweep (classify GROUNDED / VERIFY / INGEST)."
ELAI_NOTE: this lens is the human-readable front end of ELAI's grounding-sweep. Its VERIFY/INGEST findings feed the existing grounding sweep; its job is to NAME the assumption, not to verify it (verification is the sweep's deterministic job).

### L2 — Implied Completeness Detector
FIRES_WHEN: specs that enumerate a happy path or a partial list.
INSPECTS: places where the spec implies "...and the rest is obvious / handled / standard" without saying how.
PROMPT:
"You are the Implied Completeness Detector. Find every place the artifact IMPLIES full coverage while only specifying a subset: phrases like 'etc.', 'and so on', 'handle errors appropriately', 'standard validation', 'the usual', an enum/list that looks open-ended, or a method described only by its success path. Quote the phrase. State the specific cases left unspecified. A spec that says 'validate the input' without naming the validation rules is a finding. Anchor every finding to a quoted phrase or section."

### L3 — Ambiguity Mapper
FIRES_WHEN: any natural-language requirement.
INSPECTS: terms/requirements with more than one reasonable engineering interpretation.
PROMPT:
"You are the Ambiguity Mapper. Find every requirement that two competent engineers could implement DIFFERENTLY and both believe they followed the spec. For each: quote it, give the >=2 divergent readings, and state which downstream artifacts diverge depending on the reading. Target undefined nouns ('the cache', 'the user', 'recent'), unquantified adjectives ('fast', 'large', 'soon'), and unscoped pronouns. Anchor to the quoted text. An ambiguity with only one real reading is not a finding."
ELAI_NOTE: composes with the shipped AmbiguityScorer (B-38); this lens produces the human-readable map, the scorer produces the gate-time number.

### L4 — Gap Analyzer
FIRES_WHEN: any spec defining an operation, state machine, or data flow.
INSPECTS: unspecified behavior on the unhappy/edge paths.
PROMPT:
"You are the Gap Analyzer. For each operation the artifact defines, name the behaviors it does NOT specify: malformed input, permission denied, partial write/failure, concurrency/locking, empty/oversized input, timeout, retry/idempotency, ordering under contention, resource exhaustion. Example: a config strategy defines read/merge/write but says nothing about malformed input, permission denied, partial write failure, or file locking. Quote the operation; list its specific missing cases. Do not invent operations the spec doesn't have."

### L5 — Pre-Implementation Architect
FIRES_WHEN: HEAVY specs / multi-module designs.
INSPECTS: design quality, coupling, boundary placement, and whether the structure will resist the change it claims to enable.
PROMPT:
"You are the Pre-Implementation Architect. Critique the design's structure, not its prose: misplaced boundaries, hidden coupling, a module that will need to know too much, an abstraction that leaks, a data flow that crosses a trust boundary without a gate, a plan unit that ships a producer without its consumer (the wiring-enforcement walking-skeleton rule). Name two design alternatives where the choice is non-obvious, with the side effect each carries. Anchor to the section/symbol. Defer style; target structure that will cost a rewrite."
ELAI_NOTE: enforce the wiring-enforcement PRE_GATE here — flag any plan unit whose deliverable is a producer without its in-plan consumer + smoke trigger.

### L6 — Public-Interface & API-Contract Validator
FIRES_WHEN: any artifact that adds/changes a public symbol, IPC command, schema, or cross-language boundary.
INSPECTS: contract stability and cross-side symmetry.
PROMPT:
"You are the Public-Interface & API-Contract Validator. For every public surface the artifact touches (pub fn, IPC handler, Tauri command <-> TS invoke, sidecar route <-> Rust client, SSB action_type, serialized schema), check: is the contract fully specified (types, errors, invariants, versioning)? Does a change here have a matching change on the other side of every cross-language boundary? Is a new action_type/variant flowing through the golden-file lock? Quote the surface; name the missing half or the unversioned breaking change."
ELAI_NOTE: this is the design-time mirror of cross_lang_diff.py + the action_type golden lock (E-45); it catches the drift on the SPEC before code exists.

### L7 — Documentation Validator
FIRES_WHEN: specs accompanied by docs/comments, or plans that claim doc coverage.
INSPECTS: drift between what the doc says and what the spec/design actually does.
PROMPT:
"You are the Documentation Validator. Find every place the documentation, doc-comment, or README claim CONTRADICTS or under-describes the spec/design: a documented parameter that no longer exists, a described behavior the design changed, an undocumented public surface, a 'TODO/see below' that goes nowhere. Quote both sides of the contradiction. Missing docs on a public surface is a CONCERN; contradictory docs is a BLOCKER-CANDIDATE."

### L8 — Anxiety Reader  [run last; catch-all]
FIRES_WHEN: before declaring a spec ready, and in the Ship-equivalent review.
INSPECTS: the residual unease a careful engineer feels but the structured lenses didn't name.
PROMPT:
"You are the Anxiety Reader. You are the senior engineer who has to be on call for this. Read the whole artifact and answer one question: what would make you nervous to ship this, that the other lenses did NOT already catch? Surface the quiet worries — the irreversible step with no undo, the silent failure mode, the thing that works in the demo and breaks under real load/scale/adversary, the place where 'it should be fine' is doing too much work. Anchor each worry to a concrete location. If you have no anchored worry, say so explicitly rather than inventing one."

================================================================================
## CRITIQUE-ONLY MODE (the second deliverable: a selectable ELAI profile)
================================================================================
A new operating profile alongside the autonomous `orchestrate` default, mirroring the source workflow's "human writes / AI critiques only" stance and ELAI's own human-as-verifier conclusion (R5).

PROFILE: critique-only
- WRITES: performed by the OPERATOR (or a single non-spawned generation step the operator drives), NEVER by spawned executor agents. Rationale carried verbatim from the source: agent-spawned writes "caused more harm than good" — this is the conservative counterpart to orchestrate's autonomous delegation.
- AGENTS: spawned ONLY for critique (L1-L8 at design time; the existing auditor + off-agent gates at ship time). No executor/fixer agents touch disk.
- LOOP: Design (run lenses -> operator revises spec) -> Operator implements -> Ship (auditor + off-agent spine + L6/L8) -> iterate to convergence.
- WHEN TO PICK IT: high-stakes / regulated / low-trust-in-autonomy work; the human-as-verifier engagement R5 says is durable. orchestrate stays the default for autonomous throughput.
- INVARIANT: both profiles share ONE engine and ONE off-agent spine. The only difference is WHO writes and whether executor agents are spawned. The trust guarantee (off-agent deterministic evidence) is identical in both.

================================================================================
## INTEGRATION + CAVEATS
================================================================================
- WIRE-IN POINT: planning-workflow plan-improvement protocol (the 9-step adversarial injection) and the B-40 idea-critic loop. The lenses are advisory SURFACERS feeding those steps; they write findings to workflow/reasoning/ (or an advisory SSB row tagged advisory=true), NOT a gate verdict.
- OFF THE LOAD-BEARING PATH (hard rule): no lens output ever sets accept/block. Only deterministic off-agent gates decide DONE (ELAI-IMPROVEMENT-PLAN.md §6, §8).
- TOKEN BUDGET: scale lens count to artifact risk; cap findings per lens; dedup across lenses. Do not run the full 8 on a one-file SIMPLE change.
- EXTERNAL IMPORT IS GATED: do NOT import github.com/aself101/agents-and-pipelines directly — skill-import-policy.md (E-43) requires mcp-scan whole-directory scan + a workflow/skill-imports.log row with action=ALLOWED first. These re-authored lenses avoid that by being native.
- WHAT THIS IS NOT: not a replacement for the trust spine, not a new architecture. It thickens ELAI's comparatively-thin DESIGN-phase critique (ELAI's strength is post-implementation/wiring) and adds a conservative operating mode. The differentiator stays: ELAI makes doubt UN-FAKEABLE (off-agent evidence) where this layer only makes it PLURAL (more perspectives).
