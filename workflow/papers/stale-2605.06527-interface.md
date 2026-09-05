Algorithm: STALE Benchmark + CUPMem prototype (memory staleness / supersession evaluation)
ArXiv: 2605.06527
Paper: STALE: Can LLM Agents Know When Their Memories Are No Longer Valid?
Authors: Hanxiang Chao et al. (Wuhan University, CUHK, HKUST)
Reliability: VERIFIED (arXiv ID resolves, title/numbers confirmed via arxiv.org/abs/2605.06527 and arxiv.org/html/2605.06527, 2026-06-15)

---
SECTION 1: IMPLICIT CONFLICT -- PRECISE DEFINITION
---

Formal definition (paper verbatim):
"An implicit conflict is introduced when a new observation m_n renders a previously supported
belief invalid under world knowledge K, without this invalidation being explicitly communicated
in the dialogue."

Two-axiom formal characterization:
  Axiom 1 (Belief Incompatibility): m_n |=_K neg(v_o(a))
    -- the new observation logically entails the negation of the old belief value v_o for attribute a
  Axiom 2 (Non-explicit Invalidation): for all m_j in {m_{o+1},...,m_n}: not ExplicitInv(m_j, v_o(a))
    -- no utterance between old and new explicitly marks the old belief as obsolete

Key distinction from explicit contradiction:
  Explicit: "I no longer commute by bike."
  Implicit: user mentions setting up a bus pass (implies changed commute without stating it)
  The implicit form requires commonsense inference and world knowledge K to detect the invalidation.
  Explicit contradictions are excluded from the STALE benchmark by construction (Step 3 quality
  control rejects syntactically obvious candidate pairs).

Two structural subtypes:
  Type I (Co-referential): m_n and m_o address the SAME attribute a with incompatible values.
    Example: user earlier lives in Seattle; later mentions "setting up utilities for a new apartment
    in Portland" without saying "I no longer live in Seattle."
  Type II (Propagated): m_n updates attribute A, which cascades through a causal/logical dependency
    to invalidate a distinct attribute B that was never explicitly mentioned in m_n.
    Example: user described acclimatizing to Portland's pace (attribute B = location-derived habit);
    later mentions "finding a bark scorpion in their boot, driven indoors by relentless dry heat"
    (attribute A = climate/location updated to Arizona); belief about Portland habits is invalidated
    by inference, not by mention.
  Performance gap: Type II is consistently harder than Type I across all evaluated systems.
    Gemini-3.1-pro: Type I-SR = 92.0% vs Type II-SR = 69.0% (drop of 23pp).

---
SECTION 2: THREE-AXIS PROBING FRAMEWORK
---

Dimension 1 -- State Resolution (SR):
  Operational definition: "directly tests whether the model recognizes that a prior belief is no
  longer valid. The query explicitly asks about the prior belief."
  Example query: "Based on the conversation history, does the user still commute by cycling?"
  What it measures: recognition tier -- model is asked to flag staleness DIRECTLY
  Scoring: model must produce a response demonstrating awareness that the prior belief is outdated
  This is the EASIEST dimension for most models.

Dimension 2 -- Premise Resistance (PR):
  Operational definition: adversarial probing with false presuppositions. "We present a misleading
  query that presupposes m_o remains true, without mentioning new entities from m_n."
  Example query: "Since the user rides a bike every day, can you create a maintenance plan?"
  What it measures: model must REJECT the false premise rather than comply with it
  Critical: the misleading query deliberately omits all cues from m_n to prevent surface-level
  matching; model must have internalized the updated state to catch the false presupposition.
  This is the WEAKEST dimension even for top models.
  Evidence: Gemini-3.1-pro: Type I-SR = 92.0% but Type I-PR = 30.0% (gap = 62pp)
             Qwen3.5-27B:   Type I-SR = 76.0% but Type I-PR = 4.0%  (gap = 72pp)

Dimension 3 -- Implicit Policy Adaptation (IPA):
  Operational definition: "mimicking natural interaction, we pose a user-perspective query that
  mentions neither m_o nor m_n, but whose safe execution depends on the updated belief."
  Example query: "Can you suggest a commute plan for this week?"
  What it measures: application tier -- model must USE the updated state in downstream behavior
  without being cued to check for staleness
  Finding 1 (verbatim): "Recognition does not imply application. SR measures whether a model can
  invalidate an outdated belief under direct questioning; IPA tests whether the updated state is
  integrated into realistic downstream behavior. Success on one does not transfer to the other."

The retrieve-vs-act gap (the 55.2% figure, precise meaning):
  55.2% is Gemini-3.1-pro's OVERALL accuracy averaged across all six evaluation settings:
  (Type I x SR) + (Type I x PR) + (Type I x IPA) + (Type II x SR) + (Type II x PR) + (Type II x IPA)
  all six summed and averaged over 1,200 total queries.
  This is the BEST result among all evaluated models, both frontier LLMs and memory frameworks.
  It is NOT "the retrieve-vs-act gap" as a named metric; the gap is illustrated by SR vs IPA
  per-model spreads (see recognition-vs-application gap table below).

Recognition-vs-application gap (exact numbers, do not generalize without citing model):
  Qwen3.5-27B:   Type I-SR = 76.0%, Type I-IPA = 39.0%, gap = 37pp
  Qwen3.5-27B:   Type II-SR = 42.0%, Type II-IPA = 23.0%, gap = 19pp
  Gemini-3.1-pro: Type I-SR = 92.0%, Type I-IPA = 71.0%, gap = 21pp
  DO NOT cite "76%/39%" as a general-population average -- it is Qwen3.5-27B-specific.
  DO NOT cite "55.2%" as the recognition metric -- it is overall accuracy across all dimensions.

---
SECTION 3: CUPMem PROTOTYPE -- ALGORITHM AND DATA STRUCTURES
---

CUPMem reframes memory as: explicit state tracking with write-side adjudication.
Core insight: "response generation [should be] a consequence of prior adjudication rather than
a last-minute reconciliation of conflicting fragments."

WRITE TIME vs READ TIME distinction (load-bearing for harness implementation):
  WRITE TIME (when new session m_n arrives):
    -- adjudicator processes new evidence
    -- each existing state entry receives a label from {KEEP, STALE, REPLACE, UNKNOWN}
    -- stale/replaced entries are marked BEFORE they can pollute a future query
    -- this is "write-side authority": new evidence can revise, retire, or block older assumptions
    -- no deferral to query time; adjudication is completed at ingestion
  READ TIME (when query q arrives):
    -- CUPMem does NOT pass a raw top-k memory list to the generator
    -- instead, consumes write-side status markers
    -- active items  => used as current grounding for generation
    -- stale items   => treated as historical context only (not authoritative)
    -- unresolved (UNKNOWN) slots => block unsafe reuse of old defaults as premises
    -- this is constrained readout: generation is gated by prior write-side verdicts

Three core algorithmic components:

Component A -- Write-Side Belief Updating (Adjudication):
  Formula: y_i = J_theta(i, Delta_t, x_t, Omega) in {KEEP, STALE, REPLACE, UNKNOWN}
  Where:
    i        = existing state entry index
    Delta_t  = new observations from session t
    x_t      = full new session content
    Omega    = two-level state schema (see data structures below)
    J_theta  = LLM-based adjudicator
  Process: for each old state entry i, adjudicator assigns one of four labels given the new
  session. KEEP = still valid; STALE = invalidated, old value retained as history;
  REPLACE = overwrite with new value from Delta_t; UNKNOWN = cannot determine without more context.

Component B -- Topology-Triggered Belief Propagation (Propagation-Aware Search):
  Formula: C_t = {i in A_{t-1} | z_i in Direct(Delta_t) union Affected_theta(Delta_t, Omega)}
                 union Global_k(Delta_t, A_{t-1})
  Where:
    C_t              = candidate set of state entries to adjudicate for session t
    A_{t-1}          = all active state entries from prior sessions
    z_i              = state domain/slot for entry i
    Direct(Delta_t)  = slots directly touched by new observations (co-referential match)
    Affected_theta(Delta_t, Omega) = slots structurally dependent on directly touched slots
                                     (Type II propagation; requires schema Omega to resolve)
    Global_k(Delta_t, A_{t-1}) = top-k fallback candidates from semantic similarity search
  Purpose: expands stale-state search beyond directly mentioned slots to structurally affected
  regions. This is the mechanism that handles Type II (propagated) conflicts.
  Note: exact computational definitions of Direct/Affected/Global_k are in Appendix F (not
  reproduced in main text); conceptually Direct = exact attribute match, Affected = schema-guided
  dependency traversal, Global_k = embedding-based fallback.

Component C -- Constrained Readout:
  At query time, generator receives:
    -- active entries as authoritative grounding
    -- stale entries as historical background (labeled as such)
    -- UNKNOWN slots as explicit uncertainty markers (generator cannot assume old defaults)
  This is not a retrieval step; it is a structured consumption of write-side verdicts.
  The read step is O(|active entries|) -- bounded by the state entry count, not context length.

Data structures:
  State schema Omega: two-level hierarchy
    Level 1 -- State Domains: broad life-area categories (e.g., location, health, commute habits)
    Level 2 -- Local Slots: specific attribute-value pairs within each domain
                (e.g., domain=commute, slot=primary_mode, value=bicycle)
    Omega provides the dependency graph for Affected_theta: which slots in domain D are implied
    by a change to another slot in domain D' (captures Type II causal chains)
  Temporal store: each entry carries {slot, value, session_timestamp, status: active|stale|unknown}
    Status is set at write time by the adjudicator; read time consumes status markers.
  Candidate set C_t: bounded set computed per-session; not a full-scan of all entries.

Diagnostic evidence (LightMem comparison, Table 3):
  LightMem retrieves new evidence in top-20 for 77.5% of SR/PR cases.
  Yet only 3.3% of old entries are judged as requiring update.
  "Stale and updated memories therefore coexist without adjudication."
  This confirms the core CUPMem hypothesis: retrieval alone is insufficient; adjudication is
  the missing layer.

---
SECTION 4: BENCHMARK INPUTS / OUTPUTS / METRICS
---

Inputs to the evaluation harness:
  conflict_scenario | text | context up to 150K tokens | structured haystack containing:
    Session_o: multi-turn dialogue establishing old belief m_o
    Session_n: multi-turn dialogue with new implicit invalidation m_n
    Distractor sessions: unrelated daily-life topics, conservatively filtered
  probe_dimension | enum SR/PR/IPA | three separate queries per scenario
  conflict_type | enum Type-I/Type-II | structural subtype of the conflict

Outputs:
  staleness_verdict | bool | model recognizes prior belief as invalid (SR task)
  premise_rejection | bool | model rejects false-presupposition query (PR task)
  application_response | text | model's downstream behavior using updated state (IPA task)
  score_per_cell | float [0,1] | accuracy per (conflict_type x probe_dimension) cell (6 cells)
  overall_accuracy | float [0,1] | mean across all 6 cells over all scenarios

Evaluation judge:
  LLM judge: Gemini-3.1-flash-lite
  Method: assess whether response demonstrates awareness of the conflict and updated user state
  Scoring: non-reference (scored against foundational state logic, not synthetic references)
  Human agreement: 95.8% (Appendix E.3)

---
SECTION 5: QUANTITATIVE RESULTS (GROUND TRUTH NUMBERS)
---

Overall accuracy (Table 2, all 6 cells averaged):
  Gemini-3.1-pro:       55.2%  (best overall)
  Qwen3.5-27B:          31.3%
  Gemini-3.1-flash-lite: 22.4%
  LightMem:             17.8%  (best memory framework, still worse than plain LLMs)
  mem-0:                 8.3%
  LiCoMemory:            7.6%
  Zep:                   6.0%
  A-mem:                 5.1%
  CUPMem (proposed):    68.0%  (NOT a baseline; this is the paper's own prototype)

Memory frameworks vs plain LLMs: memory frameworks score LOWER than vanilla frontier LLMs.
LightMem retrieval diagnostic:
  New evidence in top-20: 77.5% (SR/PR), 67.8% (IPA)
  Old + new both retrieved: 71.0% (SR/PR), 52.2% (IPA)
  Failure DESPITE new evidence present: 56.1% (SR), 99.0% (PR), 78.6% (IPA)
  -- confirms that retrieval success does not prevent stale-memory failures

Benchmark construction:
  400 expert-validated conflict scenarios
  1,200 total evaluation queries (3 per scenario)
  100+ everyday topic domains
  Contexts up to 150K tokens (haystack from LongMemEval distractors)
  Four-step pipeline: (1) base state formulation from topic ontology, (2) adversarial conflict
  generation, (3) LLM quality-control judge with implicitness check, (4) multi-turn dialogue
  packaging into long-context haystack

---
SECTION 6: FAILURE MODES (CITED IN PAPER)
---

Recognition-application gap: models flag staleness when asked directly (SR) but fail to
integrate the update when responding to realistic downstream queries (IPA). Gap can exceed 37pp.

Premise resistance collapse: PR is the weakest dimension universally. Models that achieve high
SR still comply with false-presupposition queries. Gemini: 92% SR but only 30% PR (Type I).

Type II propagation failure: models detect direct conflicts (Type I) but fail to propagate
through dependency chains (Type II). Gemini-3.1-pro: 92% SR Type I vs 69% SR Type II.

Memory framework inversion: specialized memory systems (LightMem, Zep, A-mem) score BELOW
plain frontier LLMs. Root cause: all frameworks retrieve but none adjudicate at write time.
LightMem retrieves correctly 77.5% of the time but updates only 3.3% of stale entries.

Coexistence without resolution: old and new memories coexist in the store without any system
marking one as superseding the other. At query time, the generator sees both and has no
principled way to choose.

---
SECTION 7: IMPLEMENTATION PITFALLS (DO NOT DO THIS)
---

Do NOT treat retrieval success as proof of staleness handling. LightMem retrieves the right
evidence 77.5% of the time but still fails because no adjudication step exists.

Do NOT defer staleness detection to query time. CUPMem's architecture explicitly moves
adjudication to write time; last-minute reconciliation at read time is the pattern that fails.

Do NOT cite "76% recognition / 39% application" as a general population statistic. These are
Qwen3.5-27B-specific numbers from Type I only. The Gemini gap (92% vs 71%) is the better
citation for "the best model's gap".

Do NOT conflate SR (recognition accuracy on one dimension) with overall accuracy. 55.2% is
overall accuracy across all six cells; SR alone for Gemini-3.1-pro Type I is 92.0%.

Do NOT treat CUPMem's 68.0% as a baseline. It is the proposed prototype in this paper.

Do NOT use explicit contradiction in conflict test fixtures. STALE explicitly excludes explicit
negation ("I no longer do X") by construction; implicit conflict requires commonsense inference.

Do NOT assume the UNKNOWN label is a failure mode. In CUPMem, UNKNOWN actively BLOCKS the
generator from reusing a stale default; it is a safety marker, not an error state.

Do NOT skip the propagation step for Type II conflicts. The Affected_theta component is what
handles causal dependency chains; omitting it reduces CUPMem to Type I only.

---
SECTION 8: WHAT AN AGENT MEMORY HARNESS CAN STEAL
---

Steal 1 -- Write-side adjudication contract:
  On every memory write (new observation ingested), run an adjudicator over existing entries in
  the candidate set C_t. Assign each a status: KEEP / STALE / REPLACE / UNKNOWN.
  Never leave old and new entries in the store without a verdict.

Steal 2 -- Status-gated read:
  At retrieval time, pass status markers to the generator. Stale = historical only.
  UNKNOWN = do not use as premise. Active = authoritative grounding.
  Do NOT pass raw top-k to the generator without status filtering.

Steal 3 -- Two-level schema for propagation:
  Define a domain/slot hierarchy (Omega) over the memory store. Use it to expand the candidate
  set beyond directly matched slots to structurally dependent slots (Type II coverage).

Steal 4 -- Probe dimension as a test oracle:
  SR / PR / IPA are independently testable. An agent memory system can self-test using SR
  (ask directly about a stored fact that was implicitly superseded). If SR passes but IPA fails,
  the adjudication step is not propagating to generation.

Steal 5 -- The 55.2% ceiling:
  55.2% is the best any model achieves without write-side adjudication. CUPMem at 68.0% shows
  ~12.8pp headroom from structured write-time adjudication alone. This is the expected gain
  from implementing Steal 1+2 above.

---
SECTION 9: CRITICAL ASSERTIONS (TESTABLE)
---

assert sr_score >= ipa_score or abs(sr_score - ipa_score) < 0.01, \
  "SR (recognition) should not be harder than IPA (application) in a well-formed fixture"

assert conflict_has_no_explicit_negation(m_o, m_n), \
  "implicit conflict requires no utterance explicitly marking m_o as invalid"

assert adjudicator_ran_at_write_time, \
  "CUPMem contract: staleness verdict must be assigned before query time, not during retrieval"

assert status in {"KEEP", "STALE", "REPLACE", "UNKNOWN"} for all entries after adjudication

assert unknown_entries_block_generator_premise_use, \
  "UNKNOWN status must prevent generator from treating old slot value as an authoritative premise"

assert candidate_set_includes_type2_affected_slots, \
  "propagation step (Affected_theta) must expand C_t beyond direct attribute matches"

assert memory_framework_accuracy < plain_llm_accuracy or \
  memory_framework_has_adjudication, \
  "memory frameworks without write-side adjudication underperform plain LLMs on implicit conflict"

---
SECTION 10: COMPLEXITY AND SCOPE
---

Evaluation complexity:
  O(N x 3) queries per N scenarios (one per probe dimension)
  Context per scenario: up to 150K tokens (long-context haystack)
  Judge: LLM-based (Gemini-3.1-flash-lite), non-reference scoring

CUPMem complexity:
  Write time: O(|C_t|) adjudicator calls per new session, where |C_t| is bounded candidate set
  Read time: O(|active entries|) status-filtered readout
  Propagation: O(|Affected_theta(Delta_t, Omega)|) schema traversal -- depends on schema depth
  No explicit complexity bounds stated for Affected_theta traversal in main text

Scope:
  Benchmark covers everyday personal domains (commute, location, health, diet, hobbies, work)
  Does NOT cover structured databases, code, or factual world knowledge (Wikipedia-style)
  Designed for long-context conversational memory agents, not retrieval-augmented generation
  over document corpora
