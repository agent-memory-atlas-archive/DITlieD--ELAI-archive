DETERMINISTIC GUIDANCE VS REMEDIATION — and can a system code AND improve code deterministically
Date: 2026-07-05
Grounding sources: docs/thefinalglance/ELAI-SYSTEMS-OVERVIEW.md (read); I-SUMMARY_INDEX.md + J-SUMMARY_INDEX.md (grepped); C:\Users\user\Downloads\eiffel_negative_space_programming.md (read); WebSearch (grammar-constrained decoding; metamorphic testing). Every claim below is tagged with its source; ungrounded conjecture is labelled.


0. THE TWO QUESTIONS

Q1. Can we build a system that not only writes code but improves it, DETERMINISTICALLY?
Q2. The rules problem. When a harness enforces many prose rules ("do X or it is REJECTED"), a rule the model is not currently attending to gets violated, the violation is rejected, and the model burns context re-fixing. That is REMEDIATION. How does ELAI avoid pure remediation? Does it just clear the context / spawn a fresh model + a fix-prompt (still remediation)? Or do the deterministic rules actually GUIDE the model up front, before it goes wrong?

Q2 is the load-bearing one and it is answered first, because Q1 turns out to be the same doctrine applied one level down (to code state instead of to model actions).


1. THE CORE ANSWER TO Q2 — THIS IS NOT A BINARY

"Remediate after wrong" vs "guide before wrong" is not two options. ELAI's design (the whole I-plan and J-plan thesis, ELAI-SYSTEMS-OVERVIEW §17) is a THREE-REGIME spectrum, ordered by how early the wrongness is killed:

REGIME 1 — CONSTRAIN. Make the wrong output physically unemittable. The model is not asked to obey; the invalid action has no path.
REGIME 2 — GUIDE. Bias the generation toward correct up front, without any rejection, by putting the right constraint in the generation path at the moment it is needed.
REGIME 3 — REMEDIATE-AS-GUIDANCE. When a failure DOES slip through, do NOT blind-retry and do NOT just re-prompt a fresh model with "you failed, fix it." Convert the failure into a forward constraint and strip the failure noise from context.

The organising doctrine under all three (overview §1 opening + §17): "push decisions out of model discretion into deterministic harness machinery, leave the model only generation/selection, and gate even those against frozen, off-agent evidence." Restated for Q2: do not make rule-following the model's job. Make the rule the harness's job, and give the model the narrowest generative surface that can still produce the work.

The user's feared mechanism ("clears context or spawns another model and feeds it a fix prompt") IS one of ELAI's moves — but it lives in Regime 3, as a LAST resort gated on evidence, not as the first line of defence. Most of the leverage is in Regimes 1 and 2, which the prose-rule architecture of this workflow root barely uses.


2. REGIME 1 — CONSTRAIN (the wrong output cannot be produced)

- Phase-aware tool/rule filtering at the tool-registry level (overview §4): in a given FSM state a disallowed tool "returns an error, not advice." The model cannot call the wrong tool; there is nothing to reject after the fact because the call never resolves. This is the cheapest possible enforcement: zero remediation, because zero violation is reachable.
- Directive supervisor, I-1 (I-SUMMARY_INDEX I-1): the generator emits PURE Intent->Directive structs; a process-separated Supervisor is the ONLY executor of side effects and holds the credentials. "A generator-authored attempt to modify gate code or call a sink directly is REFUSED at the supervisor." The model literally cannot perform a destructive action — it can only PROPOSE one, which the deterministic supervisor then accepts or refuses. Wrongness is structurally impossible, not policed.
- Grammar-constrained / structured decoding (I/J: XGrammar native, J-SUMMARY line 502; Rust grammar compile+validate+blake3-pin, J line 419). WebSearch-grounded (zeroentropy.dev, mbrenndoerfer.com, XGrammar): the decoder holds a state machine (DFA / pushdown automaton / JSON-schema walker); at every token step, tokens that would make the partial output invalid are set to -inf logit BEFORE sampling. The output is "guaranteed valid — you never get a parse error, never have to retry" — it "eliminates the validate-retry loop entirely." This is the purest form of the answer to Q2: the remediation loop is not shortened, it is DELETED, because the invalid token had zero probability mass. (Caveat, WebSearch arXiv:2606.25605 "Constraint Tax": hard structured constraints can suppress tool-calling / degrade reasoning on open-weight models — constrain the SHAPE of output, not the model's freedom to think; this is why ELAI grammar-pins schemas, not prose.)
- Rules-as-DATA, not prose (overview §4 Rules engine): rules are rows in SQLite with predicate types AST_PATTERN / GREP_PATTERN / SCHEMA_REQUIRED / METRIC_GATE / COMPOSITE and verdicts ALLOW/WARN/BLOCK/ESCALATE, evaluated by the harness. A rule expressed as a deterministic predicate does not depend on the model "remembering" it from a 5000-line system prompt. This is the direct structural fix for the user's exact pain: "if it is not in context, the model won't do it." A predicate rule is never out of context because the model was never the one holding it.


3. REGIME 2 — GUIDE (bias generation up front, no rejection)

- Context budget allocator with a dedicated Rules slot + anchor pane, I-2 (overview §5; I-SUMMARY_INDEX I-2): context is fixed-slot budgeted (SystemPrompt, Rules, PhaseSpec, RepoMap, ...). The relevant rules for THIS phase are deterministically selected into the Rules slot — not the whole rulebook, the applicable subset. I-2 adds an AnchorPane appended LAST (recency-bias grounding) carrying the active constraints, and "the model must emit a short acknowledgement tag reflecting the active constraint before generating." So the rule that matters is made maximally salient exactly when it applies, and adherence is forced into the generation prefix. Contrast with a static CLAUDE.md the model half-attends to under context pressure.
- Constraints embedded in TOOL DESCRIPTIONS, re-sent each turn (J-SUMMARY line 364: "H3 constraints embedded in TOOL DESCRIPTIONS (re-sent)"). A constraint that rides the tool schema is present in the generation path on every call, not a one-time system-prompt statement that decays.
- Activation steering, J-24 (J-SUMMARY J-24, "non-prompt steering"): the DARK sidecar /v1/steering ActAdd path steers behaviour via ADDED ACTIVATION VECTORS, not prompt tokens. This guides the local model without spending a single context token on the instruction. (Local-lane only; needs logit/activation access — not available on cloud API models.)
- CSQ + inline feasibility (overview §3): functions touching files/network/money/auth get generated counterfactual-safety-question guards ("if X is wrong the code will Y — add a guard") DURING execution, and an inline feasibility check can emit CONTRADICTS_PLAN before the wrong code is finished. The guard is authored into the code as it is written, not caught after.

Regimes 1+2 are the substance of the user's question "can the rules GUIDE, not only fix after?" — yes, and that is where ELAI puts most of the design weight.


4. REGIME 3 — REMEDIATE-AS-GUIDANCE (when failure slips through)

This is where the user's guessed mechanism lives, and ELAI does it smarter than "clear context, re-prompt to fix":

- Pristine context injector, I-5 (overview §5; I-SUMMARY_INDEX I-5): on a retry, the failed sibling's traceback is HARD-EXCLUDED from context, kept ONLY as a "<=30-token contrastive constraint ('attempt A failed on line-42 type X — avoid X')." This is the precise correction to naive remediation: do not re-feed the model its own failure (which anchors and pollutes it); strip the noise, keep the LESSON, and inject the lesson as a forward constraint. So the answer to "does it just clear context and re-prompt?" is: it clears the NOISE, keeps the SIGNAL, and the signal becomes Regime-2 guidance on the next pass.
- Insight backpropagation, J-10 Arbor (overview §13): extract a 1-3 sentence insight from each audit/experiment, re-synthesize each ancestor's insight from its children, inject ancestor-lineage insights DOWN into executor prompts as a CONSTRAINTS BLOCK. Failures compound upward into priors instead of being re-paid each time.
- Rule self-discovery (overview §4): "self-discovery auto-proposes rules from repeated violations." A failure pattern that recurs is promoted into a NEW deterministic predicate (Regime 1) — so the class of failure is eliminated structurally and never remediated again. Remediation, done right, feeds back UP the regimes: a Regime-3 failure becomes a Regime-1 constraint.
- Fresh-context agent spawning (overview §4 Agent system; idea-intake "one agent per node, fresh per-invocation context"): each role runs in its own context window that is garbage-collected on return. This IS the user's "spawn another model" — but note its PRIMARY use is up-front context HYGIENE (decompose into small scoped tasks each with a clean window), not post-failure remediation. Small clean tasks fail less; that is prevention, not cure.
- SUBSTITUTE_MODEL / SPLIT_TASK (overview §4 Autonomous mode): escalating to a stronger/cloud model, or splitting the task, are AutonomousGate recovery moves fired only on HIGH-risk / missing-evidence / anti-spiral triggers. This is the user's "escalate to cloud models" guess — correct, but it is the gated last resort, not the default.

Net: ELAI does remediate, but (a) Regimes 1+2 minimise how often Regime 3 is reached, and (b) Regime 3 itself is engineered to convert a failure into forward guidance (I-5, J-10) or into a new hard constraint (self-discovery), rather than a blind fix-and-retry that pays the same context cost every time.


5. HONEST DIAGNOSIS OF THIS ROOT (the workflow harness, not .elai_cc)

The user's pain is real and correctly located. THIS root — Claude Code driving a ~5000-line prose CLAUDE.md + .claude/rules/*.md, enforced by PreToolUse / PostToolUse / Stop hooks — is largely the architecture ELAI's design exists to REPLACE. Grounded breakdown:

- Already Regime 1 here: the PreToolUse hook does path/tool gating per FSM state + cross-session file locks (CLAUDE.md "Hook enforcement"). A blocked path is refused BEFORE the edit — that is constrain-up-front, and it is the good part.
- Mostly Regime 3 here: the bulk of the rulebook is PROSE the model must hold in attention ("do X or it is rejected"). Enforcement is after-the-fact: the Stop hook blocks termination if a rule was violated, the auditor returns BLOCK and a fix cycle runs, the PostToolUse hook flags a lazy-code token after it was written. Every one of those is remediation, and every one costs the context the user is describing. The rules being PROSE is the root cause: prose rules are subject to exactly the "not in context -> not obeyed -> rejected -> re-fix" loop.
- The irony worth stating plainly: the workflow root enforces its rules the way its own product's design says NOT to. ELAI-SYSTEMS-OVERVIEW §4 ("rules are DATA in SQLite, not prompt text") is the fix, and the harness has not applied it to itself.

Two grounded directions to move THIS root from remediation toward guidance (both buildable, pick by cost):

APPROACH A (cheap, high-leverage): phase-scope the prose. Today every rule loads every session. Move to I-2's model: a small always-loaded skeleton + deterministic per-state rule selection, so only the rules applicable to the current tool/state/file-type are in context, made salient (an anchor line last). This is mostly already latent — conditional-routing.md is a manual version of it ("load only when true"). Making the router deterministic and enforced (the hook injects the applicable rule subset, the model doesn't self-select) captures most of I-2's win without a rules engine. Side effect: smaller context = the model actually attends to the few rules that matter, which is the whole point.
APPROACH B (expensive, complete): port a subset of the rulebook from prose to predicates. Any rule that is already a deterministic check (file-size boundary, banned tokens, forbidden imports, path gates, the wire-dark checkers) should NOT be prose at all — it should be a PreToolUse predicate that refuses the write, turning a Regime-3 audit-BLOCK into a Regime-1 constraint. Many of these checkers already exist (maintainability_check.py, production_caller_check.py, taste_check.py) but run at AUDIT time (remediation) rather than at WRITE time (constrain). Moving the cheap ones earlier in the hook chain is the single biggest remediation-to-guidance conversion available here.

Edge/side-effect to watch (anti-sycophancy): you cannot predicate-ify everything. Rules that require judgement ("is this the simplest solution", "does this design smell") are irreducibly Regime 2 at best — you can surface them at the right moment but not mechanically enforce them. Over-constraining those with a crude predicate reproduces the Constraint Tax (§2 caveat): the model routes around a dumb gate and you lose more than you gain. The split is: mechanical rules -> Regime 1 predicate; judgement rules -> Regime 2 salience; never force a judgement rule into a brittle predicate.


6. Q1 — CAN A SYSTEM CODE AND IMPROVE CODE DETERMINISTICALLY, AND THE EIFFEL CONNECTION

The eiffel_negative_space_programming.md doc is the SAME doctrine as §1, applied one level down — to CODE STATE instead of to MODEL ACTIONS. Design-by-Contract (preconditions / postconditions / invariants) and "negative space programming" (assertions, fail-fast, making invalid states unrepresentable) do at the code level what ELAI's gates do at the harness level: carve away the invalid region so the remaining space has fewer ways to be wrong. The doc says this itself: contracts give "AI-generated code less room to be vaguely wrong." That is Regime-1 thinking for code: an invalid state does not get silently tolerated and blow up 20 files later; it is refused at the boundary where it is created.

So the composition is:
- Harness level (ELAI): deterministic gates carve invalid ACTIONS (wrong tool, wire-dark symbol, ungrounded claim, insecure write).
- Code level (contracts / negative space): assertions carve invalid STATES (foo.bar must be a number here; balance must decrease by amount).
Both are "define what must NOT be true, and make it structurally caught." The eiffel idea is not decoration — it is the code-level instance of ELAI's whole philosophy.

Now the honest answer to "improve code DETERMINISTICALLY":

Pure-deterministic (no LLM) improvement is real but NARROW: mechanical transforms — dead-code removal, AST-folding, formatting, lint-fix, the maintainability catalog's extract/split suggestions, superoptimization on tiny fragments. These are provably behaviour-preserving and need no model. ELAI already ships the detection half (overview §17 maintainability review; §12 wiring stack; failure taxonomy).

GENERATIVE improvement (a better algorithm, a cleaner design) still needs an LLM to PROPOSE candidates — you cannot deterministically invent a better structure. The determinism has to live somewhere else, and this is the key insight: put the determinism in SELECTION and VERIFICATION, not in generation. This is precisely ELAI's built pattern:
- I-4 evidence-ranked search (overview §13): the generator proposes K divergent candidates in isolated workspaces; the orchestrator selects BY RECEIPTS (tri-layer HARD VETO -> calibrated scorer -> adversary), never by an LLM judge. Deterministic selection over non-deterministic generation.
- I-20 budgeted mutation scheduler (overview §13): mutation testing inside the loop measures whether a change actually preserves behaviour; survivors become structured counterexamples fed back to the generator. This is a deterministic fitness signal on generative output.
- DGM self-modification (overview §9): improves ELAI's own configs/rules/data-tables from PipelineOutcome. Class A = additive data-table rows with NO LLM and no staged eval — genuinely deterministic self-improvement. The "convergence theorem" (§9) is the load-bearing caveat: a self-improvement loop WITHOUT a real, un-gameable, stronger-than-generator filter provably collapses (it Goodharts its own metric). So deterministic improvement is only sound if the fitness oracle is (a) execution-grounded, (b) frozen/read-only to the thing being improved, (c) harder to satisfy than to fake.

And here is where the eiffel doc closes the loop with Q1: CONTRACTS ARE THAT FITNESS ORACLE. If the code carries negative-space assertions / contracts, then "improve this function" acquires a deterministic, un-gameable success test: a refactor that preserves every contract and passes every metamorphic relation is PROVABLY behaviour-preserving. WebSearch-grounded (arXiv:2511.02108; ScienceDirect S0164121224003741; ACM 3787120.3787123): metamorphic testing is a property-based technique that "alleviates the oracle problem" — it verifies outputs via Metamorphic Relations WITHOUT needing a ground-truth oracle. That is exactly the oracle a deterministic improvement loop needs and usually lacks. Contracts + metamorphic relations turn "did the improvement break anything?" from an LLM judgement into a deterministic check.

So the buildable shape of "code AND improve, deterministically" is:
1. Model writes code AND its contracts (negative-space assertions carving the valid states) in the same unit — this is already ELAI's vertical-slice + CSQ discipline, just extended to emit executable contracts, not only guards.
2. The contract set is FROZEN and becomes the fitness oracle (like ELAI's frozen gates, overview §12 R8).
3. Model proposes K improvement candidates (Regime: generation stays probabilistic).
4. Harness deterministically selects: any candidate that violates a contract or fails a metamorphic relation or a mutation is vetoed (I-4 + I-20 pattern); survivors ranked by a measurable objective (fewer branches, smaller file, lower complexity — the maintainability checker's numbers).
5. Winner admitted only if execution-grounded-better, journaled, reversible (overview §7 time-travel, §9 auto-revert).
The determinism is total in steps 2, 4, 5; the irreducible non-determinism is confined to step 3, where it belongs. This is not speculative — every piece exists in the spec (I-4, I-20, DGM, maintainability review, contracts-as-CSQ); the only NEW idea is making the model emit executable contracts as first-class output and freezing them as the improvement oracle.


7. LIMITS AND CAVEATS (do not oversell)

- Constraint Tax (WebSearch arXiv:2606.25605): hard output constraints can degrade reasoning / suppress tool-calling on weaker models. Constrain output SHAPE and ACTIONS; do not straitjacket the model's reasoning. This is why the split in §5 (mechanical->predicate, judgement->salience) matters.
- Determinism cannot reach generation. You can make selection and verification deterministic and reproducible (ELAI_SEED, overview §1); you cannot make "invent a better design" deterministic. Anyone claiming fully-deterministic code IMPROVEMENT is either doing only mechanical transforms or hiding an LLM in the loop.
- The convergence theorem (overview §9) is a hard constraint on Q1: an improvement loop whose fitness oracle is weaker than or gameable by the generator collapses. Contracts help ONLY if they are frozen and execution-grounded; contracts the improver can also edit are worthless as an oracle.
- Confident-shared-wrong beliefs (overview §14 honest limit): the whole stack catches coverage-gap wrongness (the model didn't know), not confident-shared-wrong (the model and its checker are wrong the same way). Contracts written wrong carve the wrong negative space. Mitigated by cross-family adversary, never eliminated.
- This root's fix is mostly organisational, not novel: the mechanisms (phase-scoped rules, write-time predicates) already exist in pieces; the work is moving cheap checks earlier in the hook chain and predicate-ifying the mechanical rules. High leverage, low novelty. Say so.


8. ONE-LINE ANSWERS

Q2: ELAI does NOT primarily "clear context and re-prompt to fix." It kills wrongness as early as possible across three regimes — CONSTRAIN (unemittable: tool-registry gating, directive supervisor I-1, grammar decoding, rules-as-data), GUIDE (salient up front: anchor-pane I-2, constraints-in-tool-descriptions, activation steering J-24, inline CSQ), and only then REMEDIATE-AS-GUIDANCE (I-5 keeps the lesson not the noise; self-discovery promotes the failure into a new hard rule). The context-waste the user feels on THIS root is real and is a symptom of THIS harness enforcing rules as prose (Regime 3) instead of as predicates and salience (Regimes 1-2) — the exact thing ELAI's design fixes and this root has not applied to itself.

Q1: Yes, bounded. Code deterministically = only mechanical transforms without an LLM. Improve code with an LLM-in-the-loop but DETERMINISTIC selection/verification = yes, and it is already ELAI's I-4/I-20/DGM pattern. The missing piece that makes it clean is the eiffel idea: emit executable contracts (negative space) as first-class output and freeze them as the un-gameable fitness oracle — then a behaviour-preserving improvement is a deterministic check (metamorphic/contract), not an LLM judgement.
