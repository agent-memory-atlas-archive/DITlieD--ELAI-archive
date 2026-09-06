HOW TO IMPLEMENT "D" (the merge) IN ELAI — grounded design
==========================================================
Date: 2026-06-26. Author: main LLM (Opus 4.8). Status: DESIGN SKETCH (not code; build via planning-workflow /
paper-to-plan phase 5, delegated to executor agents — main LLM does not write .elai_cc code).
Grounding: OWN MEASURED DATA (the experiment) + code JUST READ in this codebase (file:line cited, HIGH confidence)
+ a labelled design proposal for the task_runner integration (NOT a full read of task_runner's ~13k lines).
Companion: RLM-VS-ELAI-RESULTS-2026-06-26.md (the experiment), rlm-recursive-agents-grc-2026-06-26.md PART 7/8.

WHY D — THE MEASURED CASE (recap, so the build is justified not speculative)
- exp-1 (lexical recall): D ties ELAI retrieval at 100%, cheapest per-correct. Safe no-op where retrieval wins.
- exp-2 (multi-hop, where single-shot retrieval STRUCTURALLY fails): D is the ONLY arm that handles it without
  collapsing AND without hallucinating. C (single-shot) abstains (empty); B (free-code RLM) FABRICATES ("42" x7);
  D navigates deterministically and abstains-or-correct.
- budget-tuning (D max_steps 10->18, identical tasks): 2xW 25% -> 37%; the gains are ABSTENTIONS BECOMING CORRECT
  (gold 1309: empty->1309; gold 6245: empty->6245), never guesses turning right. So D's ceiling is a TUNABLE step
  budget, and raising it preserves the trust property (it resolves what it used to abstain on). [8xW band still
  completing at write time; trend up, mechanism already proven per-task.]
=> D is the right "both together" answer: RLM's adaptivity, ELAI's determinism + abstain-not-fabricate contract.

CAN WE GET 100% ON THE 8xW MULTI-HOP BAND? — YES, AND HERE IS HOW (measured 2026-06-26)
The question "how to get 8/8" was settled empirically, and it reframes the whole build:
- ORACLE arm (pure deterministic traversal, ZERO model calls): 8/8 = 100%. Every needle is deterministically
  REACHABLE; the answer is always in the context, findable by exact ops. So 100% is POSSIBLE; the gap is not
  reachability.
- Tuning the MODEL-DRIVEN navigator (D max_steps 10->18) PLATEAUS: 2xW 25%->37%, but 8xW 25%->25% (flat). More
  steps for a model that CHOOSES the ops hits a ceiling ~25% at 8xW — the bottleneck is navigation reliability
  (compounding per-step model error over a multi-hop chain), not budget. So you do NOT get to 100% by giving the
  model more navigation latitude.
- GRAPH-HOP HYBRID (deterministic hops, model ONLY for atomic entity + number extraction, never navigation) on a
  WEAK 8B: 7/8 = 87% at 8xW. The deterministic grep found hop2 on ALL 8 (the model read the bridge entity 8/8
  correctly); the single miss was a final number mis-read ("99" vs 9943), not a navigation failure. A
  deterministic-traversal harness made a WEAK 8B beat model-driven navigation on a STRONG 120B by 50+ points.
THE PATH TO ~100%: minimise the model's role to the genuinely-fuzzy step (entity recognition) and make EVERYTHING
ELSE deterministic. The more of {find hop1, identify bridge entity, hop to hop2, extract answer} is deterministic,
the closer to 100%. Concretely: deterministic candidate retrieval -> model atomic entity read -> DETERMINISTIC grep
of that entity -> deterministic/atomic final extraction (regex the number off the single grepped hop2 chunk, or a
3-sample vote). This is ELAI's harness>model thesis, measured: the harness does the navigation, the model only reads.

AND ELAI ALREADY SHIPS THE DETERMINISTIC MULTI-HOP RETRIEVER: crates/memory/src/hippo_rag.rs — HippoRAG-style
Personalized PageRank over an entity/note graph (Gutiérrez et al. NeurIPS 2024, arXiv:2405.14831). One PPR pass
propagates from the dept query THROUGH the shared person entity to BOTH hop1 and hop2 in a SINGLE deterministic
retrieval — no model navigation, no per-step compounding error. This is the production-grade version of the
graph-hop hybrid: the entity graph IS the deterministic traversal. So "get to ~100% on multi-hop" = route the
multi-hop case through HippoRAG (entity-graph PPR) + an atomic model extraction, NOT through model-chosen ops.

REVISED RECOMMENDATION (sharper than "build the navigator"):
1. PRIMARY: the multi-hop / long-horizon retrieval path should use DETERMINISTIC traversal — HippoRAG
   (hippo_rag.rs, already shipped, PPR over the entity graph) for the hop, model only for the atomic read. This
   is what gets you toward 100% (oracle proves reachable; 8B hybrid already 87%).
2. The RetrievalNavigator (model-driven named-op loop, below) is the FALLBACK for cases with NO clean entity
   linkage (where the graph has no edge to propagate along); it caps ~25-37% on hard multi-hop, so it is the
   safety net, not the primary path. Keep it deterministic-ops + abstain-on-budget (the trust property), but do
   not expect 100% from it.
3. Either way the model NEVER navigates the corpus by writing code (RLM) — that scored 0% and hallucinated.

THE KEY INSIGHT FOR IMPLEMENTATION: D IS NOT NEW INFRASTRUCTURE
D = three pieces ELAI ALREADY SHIPS, composed by one new bounded loop:
  1. STAGE-1 candidate set (deterministic pre-filter)
       crates/knowledge/src/pipeline.rs:486  `pub fn query_with_tier(query, QueryTier)`  (BM25+vector+rerank;
       the real retrieval entrypoint, called from cli/bench_rag.rs). This IS Arm D's BM25 candidate set, but
       production-grade (adds the vector+rerank channels the experiment's BM25-only Arm C lacked).
  2. JOURNALED NAMED-OP SUBSTRATE (the op vocabulary + replay)
       crates/tools/src/adaptive_tool.rs  — the `ToolDispatcher` trait with typed `dispatch_*` methods, EACH
       emitting SSB markers (ToolCallStart / Read / Branch / Finding / Converge / Escalate / ToolCallEnd, see the
       map at adaptive_tool.rs:101-107). `dispatch_file_read_outline` (adaptive_tool.rs:295) IS Arm D's READ op.
       One journaled named op per turn == exactly D's Stage-2 shape. Replay falls out of the SSB rows for free.
  3. THE WIDTH SIBLING (the architectural template to copy)
       crates/orchestrator/src/search/ladder.rs:173  `SearchLadder::run_best_of_n(candidates, session, mode)` —
       wired into PRODUCTION at task_runner.rs:12869+ (also escalation_waterfall.rs, structural_retry.rs). It
       branches K candidates, ranks by verifier receipts, prunes vetoed/quarantined, promotes best-of-N, and on an
       empty survivor set emits `BudgetCollapsed` + returns `winner: None` (ladder.rs:223-231). Env-gated by
       `search_ladder_enabled()` (ladder.rs:382).

THE NET-NEW: A DEPTH OPERATOR, SIBLING TO THE WIDTH LADDER
SearchLadder is WIDTH: branch N candidates in parallel, pick the best (this is the J-86 receipt-gated candidate
ladder). D is DEPTH: iterate sequentially, each step's op informed by the last, to CHAIN hops (this is the J-38
mid-task-retrieval-loop / J-42 within-step distill-reset grain). They are orthogonal and coexist (the GRC-v3 doc
already names both axes: J-86 width vs J-84 serial depth).

PROPOSED SHAPE (design — mirrors SearchLadder's struct/handle/journal idiom; confidence: HIGH on the substrate
calls cited above, PROPOSAL on the exact struct):

  // crates/orchestrator/src/search/navigator.rs  (sibling to ladder.rs)
  pub enum NavOp {                 // the FIXED, deterministic op vocabulary (no free code — the J-55-safe part)
      Grep(String),                // -> chunk/section ids matching a pattern over the FULL index (escapes the
                                   //    candidate set, the multi-hop-enabling move)
      Read(Vec<ChunkId>),          // -> reuse dispatch_file_read_outline's journaling
      Expand(ChunkId),             // -> neighbours of a hit (context around it)
      SubQuery { ids: Vec<ChunkId>, q: String },  // -> a BOUNDED child call over a sub-slice (the recursion,
                                   //    capped depth like rlm_query max_depth=2; NOT arbitrary REPL recursion)
      Answer(String),              // -> commit
  }
  pub struct RetrievalNavigator { pipeline: Arc<KnowledgePipeline>, dispatcher: Arc<dyn ToolDispatcher>,
                                  ssb: Arc<Ssb>, max_steps: usize }
  impl RetrievalNavigator {
      pub async fn navigate(&self, query: &str, session_id: &str, mode: BackendMode)
          -> Result<NavOutcome, NavError> {
          let candidates = self.pipeline.query_with_tier(query, QueryTier::Auto)?;   // STAGE-1 (piece 1)
          for step in 0..self.max_steps {
              let op = self.model_pick_one_op(&ctx)?;        // model proposes ONE named op (TD-3: it proposes)
              match op {                                     // deterministic executor adjudicates (TD-3)
                  NavOp::Answer(a) => { self.emit(Promoted, a); return Ok(answered(a)); }
                  _ => { let obs = self.exec(op).await; self.emit(ToolCall/SearchNode row); /* journal */ }
              }
          }
          self.emit(BudgetCollapsed); Ok(NavOutcome{ answer: None })   // ABSTAIN on budget — never fabricate
      }
  }

WHY THIS IS THE WHOLE POINT (the trust properties, vs RLM)
- DETERMINISTIC ops: the model PROPOSES one named op; a deterministic executor RUNS it (TD-3). No model-written
  code executes (the exact thing that made Arm B fabricate + the J-55 collapse risk). Measured: D never
  hallucinated; B answered "42" x7.
- REPLAYABLE: every op is an SSB row (piece 2 / ladder's SearchNodeEvent idiom). Re-run the recorded op stream =
  identical execution (the property RLM structurally lacks).
- ABSTAIN-ON-BUDGET == SearchLadder's `BudgetCollapsed -> winner:None` (ladder.rs:223). The operator returns "no
  answer", it does NOT emit a guess. This is the measured trust contract and it is ALREADY idiomatic in ELAI.
- BOUNDED recursion: SubQuery is a capped child call (depth<=2), not an open REPL — sits below the J-55 code-mode
  collapse threshold (exp-1 showed the constrained op-path, unlike free-code B, did not collapse on the weak lane).

WIRING (proposal — the seam, gated, wire-dark-aware per house rules)
- Construct `RetrievalNavigator` at the SAME dispatch seam in task_runner.rs where SearchLadder is built (handles
  Arc-cloned once at boot). Invoke it on the multi-hop / long-context retrieval path (the case query_with_tier's
  single shot under-serves — a needle the first top-k misses).
- ENV GATE default-OFF, mirroring `search_ladder_enabled()` (e.g. `ELAI_RETRIEVAL_NAVIGATOR_ENABLE`), with a
  wire-dark-allowlist.toml row (type=experiment, owner, expiry, promotion_plan_id) per wiring-enforcement R9.
- PRODUCTION_HOOK (D-030): the navigate() call must be reached from task_runner's retrieval path in the same
  changeset, not shipped dark. Off-agent production_caller_check.py confirms.

THE DONE-GATE / MEASUREMENT (ID-5 — ELAI-measured, never a paper constant)
- The step budget (max_steps) is the K-15-class measured knee: `elai bench reasoning-budget` / a new
  `elai bench longcot` arm (MIT harness LongHorizonReasoning/longcot + HF dataset) measures the per-(model,lane)
  point past which more steps stop converting abstentions to answers. The experiment already shows the SHAPE
  (10->18 recovers abstained-but-reachable cases); the bench sizes it per model.
- Two-arm gate: navigator ON vs single-shot query_with_tier, on a multi-hop / long-horizon task, measured by
  exact-match + tokens-per-correct + abstention-rate (abstention is GOOD when the answer is unreachable).

EXPLICIT NON-GOALS (rejected by data + house rules — do NOT drift into these)
- NO free-Python RLM operator (model writes code over its own context). Lost every cell; hallucinated on hard
  tasks; recursion never engaged even on a 120B. Rejected on every lane.
- NO model-CHOSEN orchestration (rubric G6 anti-goal): the navigator's op set is FIXED and the executor is
  deterministic; the model picks WHICH op, never authors the control flow. ELAI is deliberately Level-2/3.
- NO external trace store for the journal (Langfuse etc., GD-2): the SSB rows ARE the record.

BUILD PATH (house-rule compliant)
This doc is a DESIGN SKETCH. To build: route through paper-to-plan phase 5 / planning-workflow as a new J/K unit
(a J-42/J-86/J-38 sibling), decomposed into vertical slices (producer + consumer + smoke per wiring-enforcement
PRE_GATE), dispatched to executor agents (main LLM orchestrates, does not implement), audited off-agent. The
candidate-set (piece 1) and journaling (piece 2) are reuse; only navigator.rs + its task_runner wiring + the gate
+ the bench arm are new. Estimated surface: 1 new file (~250-400 LOC), 1 task_runner wiring site, 1 env gate, 1
allowlist row, 1 bench subcommand — small, because the substrate already exists.
