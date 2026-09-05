# How the 9 repo code-maps + Compiled AI change "how do we improve ELAI"

Date: 2026-06-15. Author: main LLM (Opus 4.8). Mode: spec-eval (AS-SPECIFIED;
design judged on merit, shipped-vs-planned tracked separately and never used to
discount the design).

Grounding: code-level maps of 9 repos written this session
(workflow/papers/repomap-{xgrammar,syncode,agentless,kimi-dev,meta-harness,
agentic-harness-eng,life-agentbench,life-taubench,live-swe-agent}.md) +
the Compiled AI paper (workflow/papers/paper-2604.05150-summary.md) + the
4-pass reasoning doc (elai-local-reasoning-and-context-2026-06-14.md) +
ELAI-FEATURE-MAP.md (full read incl. Section 4 rejections ledger).
Compose-not-restate (GD-1): this builds ON the 4-pass doc; it does not repeat it.

The difference vs the 4-pass doc: that doc reasoned over these same repos at the
PAPER-ABSTRACT level. This pass reasoned over their actual SOURCE CODE. The code
either (a) hands ELAI a concrete reference implementation for a planned-but-unbuilt
item, or (b) sharpens an honest design critique the abstracts could not.

================================================================================
## A. THE HEADLINE: the code triangulates the design; the lever is SEQUENCING, not invention
================================================================================
All 9 repos + Compiled AI independently converge on the SAME three primitives,
and all three are already ELAI's central design bets. This is triangulation, not
coincidence, and it is the strongest evidence the architecture is aimed right:

1. UN-GAMEABLE OFF-AGENT VERIFICATION is the load-bearing primitive everywhere.
   - Agentless: a reproduction test is trusted ONLY if it FAILS on the original
     (buggy) code first (test/generate_reproduction_tests.py). Selection is by
     test-signal + AST-normalized majority vote, never an LLM judge.
   - Kimi-Dev: generator x verifier selection (40 patches x 40 self-generated
     tests), cheap fail-closed pre-test gates (ast + flake8-fatal-delta + no-op
     + git-apply) before the Docker harness grades.
   - AHE (evolve.py): every harness edit carries a FALSIFIABLE prediction
     (predicted_fixes + at_risk_regressions) checked against the next eval's
     ACTUAL flips, auto-rollback to best-ever on a wrong prediction.
   - meta-harness: mutator/judge separation enforced by tool DENY-LIST (the
     proposer literally cannot invoke the gate command); off-agent fitness;
     Pareto frontier over (accuracy, COST).
   - Life-Harness/tau2: end-state DB-hash equality via write-only replay vs a
     gold run; pass^k reliability (all k must pass).
   - Compiled AI: four-stage Security->Syntax->Execution->Accuracy with
     error-context regeneration; "validation first-pass rate" as a metric.
   ELAI already has this as doctrine (wiring-enforcement R1-R9, F-4 cant-lie
   gate, D-3 >=2-verifier, production-caller gate). The repos prove the bet.

2. CONSTRAIN THE MODEL'S OUTPUT FORMAT MECHANICALLY, not by prompting.
   - xgrammar / syncode: grammar-constrained decoding at the logits layer.
   - Kimi-Dev / Agentless: a strict SEARCH/REPLACE edit DSL applied literally
     via difflib (no tool-calling needed to edit code).
   - Compiled AI: Pydantic schema on every bounded LLM call, auto-retry on
     schema violation.
   ELAI plans all of this (E-35 STRUCTURED OUTPUT, C-26, I-34) but it is the
   single most under-shipped high-leverage area (see B.2).

3. FREEZE THE MODEL, EVOLVE THE HARNESS. AHE, meta-harness, and Life-Harness
   ALL state this explicitly; Compiled AI's compile-once is the extreme form.
   This IS ELAI's spine (Focus Tunnel + DGM + rules-as-data). Confirmed.

Implication: with ~550 SSB variants and ~100 planned units already covering this
space, ELAI does NOT have a missing-capability problem. It has a SEQUENCING +
AFFORDABILITY problem. The repos' value is that they de-risk the few load-bearing
unbuilt items and they sharpen one honest design critique (Section D).

================================================================================
## B. THE THREE LOAD-BEARING DE-RISKS (concrete reference code for ELAI's top unbuilt items)
================================================================================

### B.1 H-43 DGM execution-grounded fitness <- AHE + meta-harness (HIGHEST LEVERAGE)
The 4-pass doc names ELAI's DGM fitness placeholder (derived_alpha =
blake3(config_bytes) at darwin_admit_loop.rs:186; trap = keyword grep at
trap_tasks.rs:123) as THE caveat on the whole autonomous-loop capability gain
(VD-1/VD-2). H-43/H-44 are PLANNED to replace it. AHE + meta-harness hand ELAI
the exact, working, un-gameable mechanism:
- AHE's staggered-generation FALSIFICATION (evolve.py): a harness edit predicts
  which tasks will flip; the next eval verifies the prediction against actual
  flips and auto-rolls-back wrong predictions to the best-ever state. This is a
  real stronger-than-generator filter (the prediction is checked against
  off-agent ground truth, not self-confidence). It directly satisfies VD-1
  (execution-grounded), VD-2 (measured behavioral check), and H-45's worst-case
  /lineage promotion.
- meta-harness's STRUCTURAL enforcement: separate the mutator from the judge by
  TOOL DENY-LIST, not by social rule. The agent proposing a change cannot run
  the gate that scores it. This hardens H-43/H-47 eval-firewall and the
  production-caller gate from "the agent shouldn't" to "the agent can't."
Both are config/loop-level ports (no new model, no RunPod). This is the single
highest-leverage steal because it un-blocks the convergence theorem that gates
the entire self-improvement and flywheel track (I-30/I-31).
ELAI map: H-43, H-44, H-45, H-47; VD-1..VD-5. Status: PARTIAL/PLANNED.

### B.2 Grammar-constrained decoding INTO THE SIDECAR NOW <- xgrammar (+ syncode pattern)
E-35 carries "XGrammarLogitsProcessor + tool_parser.py absent (RUNPOD-PENDING)",
yet the sidecar is ALREADY model.generate() + a LogitsProcessor chain that LISTS
"XGrammar mask" as a slot (feature map 1.9). xgrammar is pip-installable
(Apache-2.0) and ships:
- contrib/hf.py: a ready HuggingFace LogitsProcessor (drops into the existing
  chain directly).
- get_model_structural_tag: consumes an OpenAI-shape tool catalog and emits the
  per-model wrapper grammar so a small Qwen returns a STRUCTURALLY-VALID,
  schema-correct tool call every time. This DELETES the tool-use failure
  category the 4-pass doc identifies as sub-32B models' #1 failure axis (42% of
  errors) -- it is the decoder-level embodiment of the Focus Tunnel thesis.
- FindJumpForwardString: emit forced literal continuations (JSON keys, braces,
  wrappers) with NO forward pass -- a free latency/token win on the slow sidecar.
syncode adds the fallback pattern if a from-scratch build is ever needed: an
OFFLINE precomputed mask store keyed by (tokenizer, grammar, mode) + a byte-level
tokenizer abstraction (works across any HF BPE family).
Crucial nuance (W-10 doctrine): RUNPOD is only needed for the EVAL. The
integration can ship NOW behind a deterministic fixture harness; the live
benchmark stays a separately-gated checkbox. So E-35's "RUNPOD-PENDING" is not a
reason to leave this dark.
HARD CAVEATS (do not oversell): masking is LOCAL-LANE ONLY (the cloud Anthropic
API takes no logits processor; only grammar->schema mapping transfers to cloud).
Masking shapes SAMPLING, never SEMANTIC correctness -- ELAI's grounding/wiring
gates stay orthogonal. And HD-3 stands: NEVER grammar-constrain reasoning/coder/
planner roles, only pure op-emission (tool calls, JSON, edit-DSL).
ELAI map: E-35, E-18, C-26, I-34, J-15 (llguidance Rust leg). Status: PARTIAL.

### B.3 Deterministic stagnation counters for anti-spiral / J-12 H4 <- AgentBench
C-10 anti-spiral is PARTIAL with phase_4G.toml [stagnation] section MISSING, and
J-12's H4 layer is planned. AgentBench (Life-Harness leg) ships the exact
CONTENT-AGNOSTIC counters that map 1:1 onto ELAI's StagnationPattern variants:
- exact-repeat counter -> Spinning
- A<->B oscillation counter -> Oscillation
- empty-error streak counter -> NoDrift
- budget-remaining force (force-submit only a PLAUSIBLE candidate, else escalate)
  -> the GRC "budget overflow forces convergence or escalation"
These are deterministic counts over the executor+auditor history (no LLM, no new
model). They replace ELAI's softer heuristics with exact triggers and fill the
missing config section. KEY DISCIPLINE the repo teaches: apply binary counters
ONLY to deterministic seams (build/test/wiring/dup-action), NEVER to NL taste/
maintainability judgements where they over-fire.
ELAI map: C-10, J-12 (H4 annotators). Status: PARTIAL.

================================================================================
## C. CHEAP NET-NEW WINS (small adds the repos surface that ELAI lacks or has weaker)
================================================================================
- live-swe-agent (it is README + 3 YAML configs, the contribution is the recipe):
  * Per-TRAJECTORY dollar cap ($3/task in their config). ELAI has a cost ceiling
    (E-39, blocks-not-warns) but no per-task $ cap on the routed model -- a tiny
    add that matters when Opus is the routed model. Folds into E-39 / H-9.
  * Head+tail output truncation (5000 head + 5000 tail, middle elided) on tool
    observations -- a clean, cheap local-context-window protector for
    runtime_client.rs. Composes H-50 window-minimization.
  * Step-level REFLECTION NUDGE: one sentence appended every step ("reflect on
    whether a helper tool would help here") + in-trajectory DISPOSABLE tool
    synthesis (the model writes a task-local Python helper, runs it, discards
    it). Off-wedge for the FIM kernel but a real explorer-phase lever; A/B it.
- Kimi-Dev: flake8-FATAL-only-DELTA gate -- block only NEW fatal lint errors the
  patch introduced, not pre-existing ones. Concrete, cheap auditor Layer-1
  refinement (the "delta not absolute" trick) for the maintainability checker.
- Compiled AI:
  * Token-amortization break-even n* = GENTOKENS / RUNTIMETOKENS_saved as a
    PRINCIPLED crystallize threshold for the TemplateCrystallizer (C-25 today
    crystallizes after an arbitrary 3 successes). Grounds "when to compile a
    pipeline" in economics, not a magic number.
  * "Validation first-pass rate per gate" as an AUDITING-state metric
    (security/syntax/execution/accuracy first-pass + compile-time failure rate).
    Distinguishes "does it work" from "is it production-ready." Feeds SAFEDELTA.
  * The SAFETY SANDWICH (bounded LLM invocation -> hallucination range-check ->
    deterministic decision -> audit trail) as the CANONICAL shape for ELAI's
    "bounded agentic invocation" (the Code Factory variant). This is the precise
    spec for the sidecar invocation protocol and the H-17..H-29 trust track.
  * DeBERTa-v3 prompt-injection classifier (95.8% recall / 100% precision in
    their eval) as ELAI's Input Gate -- E-37 PROMPT INJECTION DEFENSE is PARTIAL
    (output_sanitizer/classifier absent). It is another small ONNX model on the
    EXISTING vector/reranker ONNX infra (GraphCodeBERT/Granite/ms-marco pattern),
    so low integration risk. ANTI-SLOPSQUAT: verify the exact HF model card
    before adding -- do not assume a package/model id.

================================================================================
## D. THE HONEST DESIGN CRITIQUE (spec-eval anti-sycophancy -- the real improvement)
================================================================================
This is the most valuable thing the CODE (not the abstracts) surfaces, and it is
a DESIGN observation, so it is fair under spec-eval (not an implementation note).

EVERY harness in this batch that WORKS is RADICALLY SIMPLER than ELAI's spec.
- Agentless's entire published thesis is "simple, non-agentic, deterministic
  beats agentic" -- and it won SOTA-for-open-source on SWE-bench Lite at
  ~$0.34/issue with a fixed 3-phase pipeline.
- AHE's DISCOVERED winning harness was "one shell tool + one regex middleware +
  a verification-discipline system prompt." Its measured datum is a direct
  warning to ELAI: adding system-prompt instruction text alone REGRESSED -2.3pp
  (memory +5.6pp and tools +3.3pp were the wins; MORE INSTRUCTION MADE THE MODEL
  WORSE).
- meta-harness keeps a Pareto frontier over (accuracy, COST) -- complexity is a
  first-class cost it actively minimizes.
- Kimi-Dev's scaffold is a deliberately minimal TWO-CALL pipeline; its real win
  came from RL TRAINING a 72B model, not from harness breadth.

ELAI is the opposite shape: 550 SSB variants, ~100 planned units, a 1826-line
feature map, composed reasoning stacks (GRC + reasoning-templates + PPM +
consensus + cross-family adversary + receipt-ranked search + abstention + depth
budget). The 4-pass doc's own caveat #3 (the layers COMPOSE, they do not
MULTIPLY; each adds latency/cost) and the K-4 gate-tax ceiling acknowledge this.
The code maps make it sharp: the empirical evidence is that the PAYOFF is
concentrated in a SMALL number of load-bearing primitives executed well, and that
instruction/complexity past a point is NEGATIVE-yield on a small model.

THE ACTIONABLE DESIGN IMPROVEMENT (not a new feature -- a sequencing+pruning
discipline):
1. Ship the MINIMAL load-bearing kernel FIRST and MEASURE it before adding
   breadth: (a) one un-gameable off-agent verifier loop end-to-end [B.1], (b)
   grammar/edit-DSL output constraint on the local lane [B.2], (c) deterministic
   stagnation counters [B.3], (d) the Agentless-shape decompose->repair->validate
   pipeline that E-46 already plans. That is ELAI's Agentless-equivalent minimal
   spine.
2. Make SAFEDELTA/G-6 + the convergence-curve instruments (I-27/I-38) the GATE
   that decides whether each ADDITIONAL reasoning layer earns its keep -- run the
   gate-ON-vs-OFF ablation per layer, drop layers that do not move the verified
   number. AHE's -2.3pp result says some layers WILL be negative-yield; ELAI must
   be willing to find and cut them. This is HD-1 (measure per config) promoted
   from a doctrine line to the primary sequencing rule.
3. Treat the feature map's breadth as a MENU to prune against measured outcomes,
   not a backlog to fully build. "It is designed" is not "it should ship."

THE CEILINGS (do not over-claim past them; the code confirms the doc's Pass-2/3):
- MODEL-CAPABILITY FLOOR: the harness is not a model substitute. SWE-Effi:
  GPT-4o-mini collapses to 5.1% where Qwen3-32B hits 21.8% on the SAME scaffold.
  Kimi-Dev confirms from the other side -- its win was RL on a 72B model. ELAI's
  local-lane bet rests on Qwen-7B/Coder being ABOVE the floor; plausible
  (measured 100% on the short task set) but UNPROVEN for ELAI's decompose shape.
- MULTI-FILE GLOBALITY: deterministic decompose has a hard ceiling exactly where
  the gap is worst. Agentless produced 6 multi-file patches out of 497 successes;
  agentic BEATS deterministic on cross-file (OpenHands 25.3% vs Agentless 18.3%,
  arXiv:2509.13941). The Focus Tunnel narrows the MODEL's job, not the TASK's
  intrinsic globality. ELAI's per-step verification + first-error truncation +
  rollback fight this, but "decomposition recovers the long-horizon gap for a
  sub-32B model" remains a DESIGN HYPOTHESIS this corpus supports mechanistically
  and does NOT measure. SAFEDELTA/I-27/I-38 are the falsifiers.
- GRAMMAR DECODING is local-only and shapes sampling not semantics (B.2 caveats).

================================================================================
## E. WHAT NOT TO DO (already in the rejections ledger -- do not re-propose)
================================================================================
- Temporal.io as a workflow substrate (Compiled AI uses it): already
  self-REJECTED for local (Section 4). The SessionDag/SSB substrate stands.
- The constrained-gen alternatives (outlines/guidance/lm-format-enforcer/syncode-
  as-framework): REJECTED in favor of settled XGrammar + planned llguidance Rust
  leg (I-34). syncode is a PATTERN reference (offline mask store, byte-tokenizer),
  not a dependency to adopt.
- Force-JSON/constrained-decode on reasoning roles: BANNED (HD-3).
- Same-model self-critique as a gate, blake3-of-bytes fitness, keyword-grep trap:
  BANNED (VD-1/VD-2/D-13) -- B.1 REPLACES these, never reintroduces them.
- live-swe-agent's "make the model tiny" extreme: refuted by the model floor
  (SWE-Effi). Keep the SMALLEST IRREDUCIBLE kernel, not an arbitrarily small one.

================================================================================
## BOTTOM LINE
================================================================================
The 9 code-maps + Compiled AI do not reveal a capability ELAI is missing -- they
CONFIRM, at the source-code level, that ELAI's design is aimed at the three
primitives every working harness in this batch converges on. The improvement is
therefore not "add features"; it is:
(1) De-risk the three load-bearing UNBUILT items using these repos' working
    reference code -- DGM fitness via AHE staggered-falsification + meta-harness
    tool-deny-list separation [B.1, top priority]; grammar/structural-tag
    decoding into the sidecar now [B.2]; deterministic stagnation counters [B.3].
(2) Add a handful of cheap wins [C] (per-trajectory $ cap, head+tail truncation,
    flake8-fatal-delta, n* crystallize threshold, first-pass-rate metric,
    DeBERTa-v3 input gate, safety-sandwich as the canonical bounded-invocation).
(3) Adopt the design discipline the code makes undeniable [D]: every successful
    harness here is MINIMAL; ship the minimal load-bearing kernel first, gate
    every additional layer on a measured SAFEDELTA/convergence-curve delta, and
    prune negative-yield layers (AHE's -2.3pp says they exist). Breadth is a
    menu to measure against, not a backlog to fully build.
Honest ceiling: the local-lane long-horizon/multi-file gap is real and
scaffold-resistant; ELAI's decompose-verify-rollback is the right answer but
recovering that gap for a sub-32B model is an unmeasured hypothesis -- the
believe-the-number SAFEDELTA run (G-6) is still the thing that turns this whole
design from "mechanism-sound bet" into "proven."
