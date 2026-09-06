SHARD PRIOR-ART DOSSIER 04 — Semantic Consensus Framework (SCF)
Pillar: multi-agent coordination / semantic conflict detection
Compiled: 2026-06-01 (oracle-mode grounded research)
Grounding floor: 11 distinct primary-source fetches (paper HTML, paper PDF, GitHub API tree, README, 4 source files, requirements.txt, Google Scholar). Every architecture/number/API claim below traces to a fetched source indexed in section 8.

=========================================================================
1. IDENTITY & PROVENANCE
=========================================================================

Full name: Semantic Consensus Framework (SCF) — paper title "Semantic Consensus: Process-Aware Conflict Detection and Resolution for Enterprise Multi-Agent LLM Systems".
Author / org: Vivek Acharya. Affiliation: Boston University (verified email bu.edu per Google Scholar [8]). Research areas listed: Agentic AI, AI Governance, Multi-Agent Systems, Healthcare AI, AI Security, MCP. h-index 4, i10 2, 83 total citations; the SCF paper itself has 0 citations as of fetch [8].
arXiv: 2604.16339 — VERIFIED, resolves to a real CS.AI submission, submitted 2026-03-13 [1][2].
Repo: github.com/curiosityexplorer/semantic-consensus — VERIFIED via GitHub API [4]. created_at 2026-03-10T17:12:26Z, pushed_at 2026-03-10T18:12:35Z (single ~1h push window, no subsequent commits). 0 stars, 0 forks, 0 open issues. Language: Python. License: NONE specified on the repo [4].
License (paper): CC BY-NC-ND 4.0 per arXiv PDF metadata [3]. NoDerivatives + NonCommercial — restrictive; cannot legally fork-and-modify the paper, and the repo has no license at all (default = all rights reserved, NOT open-source despite being public). FLAG for ELAI: code is not reusable under any OSS license.
Current version: no release tags; single push HEAD. "Version" is effectively the 2026-03-10 snapshot. Paper claims ~4,200 lines Python [2]; the visible tree (section 3) is consistent with a small codebase.

PROVENANCE MISMATCHES (flagged):
  - README citation block instructs citing the work as "Acharya, Vivek. Applied Sciences, 2026, MDPI" [5], but the canonical artifact fetched is an arXiv preprint [1] under CC BY-NC-ND. No MDPI Applied Sciences DOI was located in any fetched source. Treat the MDPI claim as UNCONFIRMED — possibly a planned/intended venue, possibly aspirational. Do not cite an MDPI publication that I could not resolve.
  - README headline metrics ("74.3% reduction in semantic conflict incidence", "145ms median governance overhead (<3% of interaction time)") [5] are CONTRADICTED by the paper body: the arXiv HTML states the paper does NOT provide a governance-overhead-ms figure and does NOT state an explicit conflict-reduction percentage (it reports a 25.9% baseline conflict rate constant across approaches) [7]. The README numbers do not appear in the fetched paper text. Treat the README's 74.3% / 145ms as UNRECONCILED author-side claims.

=========================================================================
2. PROBLEM IT SOLVES
=========================================================================

Target failure mode: "Semantic Intent Divergence" — cooperating LLM agents develop inconsistent interpretations of a shared objective because of siloed context and the absence of a shared process model, leading to coordination failure in enterprise workflows [2][6]. The paper frames this as a "primary yet formally unaddressed root cause" of multi-agent failure and cites enterprise multi-agent failure rates of 41-86.7% [1][2].
Scope qualifier (load-bearing): the problem is framed for ENTERPRISE workflows with formal process models (BPMN, state machines, policy docs). SCF assumes the existence of a machine-readable process/state model to resolve against. It is a governance/middleware concern, not a general agent-reasoning improvement.

=========================================================================
3. HOW IT WORKS — ARCHITECTURE + KEY MECHANISMS
=========================================================================

SCF is a middleware layer inserted between agent decision and agent execution via an interceptor. Six named components [2][6], each mapped to a real module in the repo [4]:

(a) Process Context Layer — module scf/core/pcl.py. Ingests enterprise workflow models (BPMN, state machines, policy documents) to establish a shared operational semantic foundation [2]. This is the source of truth the detection engine resolves against (synonym resolution, state-exclusivity checks).

(b) Semantic Intent Graph (SIG) — module scf/core/sig.py. A directed labeled graph G=(V,E,L) where V is the set of intent nodes [2]. GROUND-TRUTH from source [9]:
    - class IntentNode (dataclass) fields: id, agent_id, agent_role, action_type, target_entities: List[str], preconditions: Dict[str,str], postconditions: Dict[str,str], confidence, timestamp, status, dependencies, metadata.
    - class SemanticIntentGraph with register_intent(self, intent: IntentNode) -> str (auto-constructs edges to related nodes), get_pending_intents(), get_conflict_edges(), get_intents_for_entity(entity).
    - enum EdgeType { DEPENDENCY="dependency", POTENTIAL_CONFLICT="potential_conflict", CAUSAL_CHAIN="causal_chain" }.
    Note: preconditions/postconditions are Dict[str,str] (entity -> state-string). Detection is therefore string/set comparison over these maps, not embedding similarity.

(c) Conflict Detection Engine — module scf/detect/cde.py, class ConflictDetectionEngine [10]. Three detection methods, all SYMBOLIC/RULE-BASED [10][7]:
    - _detect_contradictory() (Type 1): compares postcondition strings per shared entity; post_a = a.postconditions.get(entity,""), post_b = b.postconditions.get(entity,""), flag if post_a != post_b. With a process model loaded it calls model.resolve_synonym(...) then check_state_exclusivity(); without a model it flags differing raw strings as potential conflicts.
    - _detect_resource_contention() (Type 2): for each (entity_a, required_a) in a.preconditions, looks up post_b = b.postconditions.get(entity_a); conflict if post_b and post_b != required_a. Also numeric: if amount_a + amount_b > available.
    - _detect_causal_violation() (Type 3): for each (entity, required_state) in b.preconditions, checks a_post = a.postconditions.get(entity); conflict if a_post and a_post != required_state (dependency's postcondition fails dependent's precondition).
    The paper's own wording: "The CDE is implemented using a hybrid approach combining rule-based analysis (for Type 1 contradictory intent detection) and precondition-postcondition analysis (for Type 2 resource contention and Type 3 causal violations)" [7]. A SentenceTransformer is loaded ONLY for optional similarity scoring, NOT for core detection [10]; sentence-transformers is a COMMENTED-OUT optional dep in requirements.txt [11].

(d) Consensus Resolution Protocol — module scf/resolve/crp.py. policy-authority-temporal 3-tier hierarchy [2][6]:
    - Tier 1 (policy): governance policies, compliance rules, NIST AI RMF controls.
    - Tier 2 (authority): agent role hierarchy, skill relevance, historical accuracy.
    - Tier 3 (temporal): first-registered intent retains priority (optimistic concurrency).

(e) Drift Monitor — module scf/drift/monitor.py. Detects gradual semantic divergence across long-running workflows by measuring semantic distance between agents' evolving working contexts [2]. Computes a "Semantic Alignment Score" from entity-state-model overlap, consistency with valid transitions, and divergence between agent confidence and historical base rates; when SAS falls below threshold theta it triggers proactive re-synchronization [6].

(f) Process-Aware Governance Integration — module scf/governance/pagi.py. Connects SCF's conflict resolution to enterprise governance control planes for organizational policy enforcement; maintains governance audit trails [2][6].

Control/data flow (interceptor pattern) [2][6][12]: framework adapter wraps the native message-passing / tool-calling mechanism -> (a) intercept outgoing agent action pre-execution -> (b) extract an intent representation -> register in SIG -> invoke CDE -> apply CRP resolution if conflict -> return approve/block. Main entry: class SemanticConsensusFramework.process_intent(intent: IntentNode) -> InteractionResult, plus mark_executed(intent_id) to commit state after execution [12].

IMPLEMENTATION NOTE (load-bearing for SHARD comparison): the adapters directory (scf/adapters/) contains only __init__.py [4] — there is NO concrete AutoGen/CrewAI/LangGraph adapter shipped, and requirements.txt lists NONE of autogen/crewai/langgraph [11]. Combined with the paper's admission that experiments "simulate multi-agent coordination patterns rather than running actual AutoGen, CrewAI, and LangGraph agents with live LLM inference" [7], the framework's headline integration with three named frameworks is ARCHITECTURAL/DESIGN, not demonstrated in shipped code.

=========================================================================
4. CORE API / USAGE (verbatim-in-spirit from repo)
=========================================================================

Core API surface [9][12]:

    from scf.core.pcl import ProcessContextLayer
    from scf.core.sig import IntentNode
    from scf.middleware import SemanticConsensusFramework

    # IntentNode dataclass fields (real, from scf/core/sig.py):
    #   id, agent_id, agent_role, action_type,
    #   target_entities: List[str], preconditions: Dict[str,str],
    #   postconditions: Dict[str,str], confidence, timestamp,
    #   status, dependencies, metadata

    scf = SemanticConsensusFramework(pcl, config=None)   # SCFConfig optional
    result = scf.process_intent(intent)                  # -> InteractionResult (approve/block + reasons)
    # ... agent executes only if approved ...
    scf.mark_executed(intent.id)

Repro entry point (from README [5]):
    pip install -r requirements.txt
    python -m experiments.runner --scenario all --runs 50 --seed 42
    python -m experiments.runner --scenario financial --runs 50 --seed 42

(Signatures process_intent(self, intent: IntentNode) -> InteractionResult and mark_executed(self, intent_id: str) are quoted from scf/middleware.py [12]; register_intent / get_conflict_edges / EdgeType from scf/core/sig.py [9].)

=========================================================================
5. MEASURED NUMBERS / BENCHMARKS
=========================================================================

Eval setup [2][7]: 600 runs = 50 runs x 4 scenarios x 3 framework configs. Frameworks named: AutoGen v0.4, CrewAI v0.76, LangGraph v0.2. Scenarios: Financial Transaction Processing, Customer Support Resolution, Supply Chain Order Fulfillment, Software Development Workflow. Baselines: Ungoverned, Schema-Only, Judge-Agent, SCF-NoPCL. Seed = 42 [2][3].

Headline figures:
  - SCF: 100% workflow completion; next-best baseline (Judge-Agent): 25.1% [2][6][7]. ** CRITICAL CAVEAT: the runs are SIMULATED. Paper states verbatim: "Our experimental framework simulates multi-agent coordination patterns rather than running actual AutoGen, CrewAI, and LangGraph agents with live LLM inference" [7]. The 100% vs 25.1% is over a simulation harness, NOT live multi-agent LLM execution. UNREPRODUCED on real agents.
  - Baseline failure rate (motivation): 41-86.7% [1][2].
  - CDE detection quality (Table 2) [7]: Precision 27.9%, Recall 65.2%, F1 39.0%. Author caveat verbatim: "SCF's lower precision (27.9%) reflects its deliberately conservative detection strategy: it is better to block a potentially non-conflicting intent (which can be retried at low cost) than to miss a true conflict" and "SCF achieves a lower F1 (39.0%) but 100% workflow completion, because it operates pre-execution and employs conservative blocking" [7]. So the 100% completion is purchased with low-precision over-blocking, not high-precision detection.
  - Baseline conflict rate ~25.9% constant across approaches [7].
  - README-only claims NOT found in paper body: "74.3% reduction in semantic conflict incidence", "145ms median governance overhead (<3% of interaction time)" [5]. Paper HTML explicitly says no governance-overhead-ms figure and no explicit reduction percentage are provided [7]. -> author-reported, UNRECONCILED, do NOT cite as measured.

Sample size: n=600 simulated runs, single seed (42). No statistical-significance test, no multi-seed variance, no confidence intervals surfaced in the fetched sections. tests/ directory in the repo is EMPTY (only __init__.py) [4] — no unit tests ship to independently verify the modules.

=========================================================================
6. MATURITY SIGNALS
=========================================================================

  - GitHub: 0 stars, 0 forks, 0 open issues, single push 2026-03-10 (~1h window), no commits since [4]. No release tags, no CI, no license, empty tests/ [4].
  - Paper: arXiv preprint 2026-03-13, 0 citations [1][8]. No located peer-reviewed venue (README's MDPI Applied Sciences claim unresolved [5]).
  - Author: solo author, Boston University, h-index 4 (most-cited work unrelated: Oracle Blockchain guide, 48 cites) [8]. A related 2026 paper "Skill Poisoning..." has 1 citation [8].
  - Production users: NONE verifiable. No adopters, no dependents, no downstream forks [4].
  VERDICT: research prototype / single-author preprint with a companion code drop. Pre-adoption. Not a maintained or production-grade system. Maturity is LOW.

=========================================================================
7. VERSUS SHARD'S MULTI-AGENT COORDINATION / SEMANTIC CONFLICT PILLAR
=========================================================================

Both systems target multi-agent coordination with conflict detection, but they sit at different layers and make opposite tradeoffs.

WHERE SCF IS STRONGER / HAS A MECHANISM SHARD LACKS:
  - SEMANTIC (not syntactic) conflict detection is SCF's whole point. SCF detects conflicts at the level of intent postconditions vs preconditions resolved against a formal process model — contradictory states, resource contention, causal violations [10]. Per the requester's note, SHARD's repo implements only SYNTACTIC conflict detection. SCF's three-category symbolic scheme (Type 1/2/3) and its synonym-resolution + state-exclusivity check against a BPMN/state-model is a concrete mechanism SHARD does not have. If ELAI wants to move SHARD's DECLARE-REVIEW-EXECUTE beyond syntactic clash detection, SCF's IntentNode(pre/postconditions) + CDE rule set is the closest grounded prior art to borrow from.
  - PROCESS-MODEL GROUNDING: SCF resolves intents against an external machine-readable process model (PCL). SHARD's coordination has no equivalent shared process/state model to adjudicate semantic equivalence; SCF's check_state_exclusivity / resolve_synonym pattern is a mechanism SHARD simplifies away.
  - DRIFT MONITORING over long-running workflows (Semantic Alignment Score + threshold-triggered re-sync) [6] is a temporal coordination mechanism. SHARD's staleness model — staleness = (now - last_verified)/expected_lifetime, flag>1.0, quarantine>2.0 — is conceptually adjacent (both detect divergence-over-time and act on a threshold), but SHARD's staleness is about KNOWLEDGE/SKILL freshness, whereas SCF's Drift Monitor is about AGENT-CONTEXT semantic alignment within a live workflow. Different axis; SCF's per-workflow drift signal is something SHARD's coordination layer does not track.
  - GOVERNANCE INTEGRATION (PAGI, NIST AI RMF tier-1 resolution, audit trails) is a first-class layer in SCF. SHARD's trust tiers (T1-T4) are a trust/authority model but are not wired to an enterprise governance control plane the way PAGI is.

WHERE SHARD IS STRONGER / WHAT SCF LACKS:
  - SELF-IMPROVEMENT: SHARD has safety-constrained self-improvement (.bak rollback + 0-3 scoring) and composition.py cross-subsystem self-healing cascade. SCF has NO self-improvement, NO learning, NO self-healing — it is a static rule-based gate. Its only "recovery" is conservative blocking + retry. This is SCF's biggest gap relative to SHARD's pillar. (Note per requester: SHARD's repo as-shipped also has NO self-improvement loop and only SYNTACTIC detection, so this is a design-intent advantage of SHARD, not necessarily a shipped one — both systems' most ambitious claims outrun their shipped code.)
  - EVIDENCE QUALITY: SHARD's coordination is exercised by an actual repo implementation; SCF's headline 100% vs 25.1% is over a SIMULATION that does not run AutoGen/CrewAI/LangGraph or any live LLM [7], and ships no concrete framework adapter and no tests [4]. On grounded-evidence terms, SCF's coordination claims are weaker because they are unreproduced on real agents.
  - DETECTION PRECISION: SCF's CDE is deliberately low-precision (27.9%) over-blocking [7]. A SHARD-style coordination layer that wants usable throughput cannot simply adopt SCF's "block on any postcondition mismatch" rule without inheriting a high false-positive rate. SHARD's syntactic detection, while narrower, will have far fewer false positives.
  - TRUST TIERS: SHARD's T1-T4 skill trust tiers are a richer authority model than SCF's CRP Tier-2 ("agent role hierarchy, skill relevance, historical accuracy"), which is described but not shown to be learned or calibrated.

CONCRETE TAKEAWAY FOR ELAI: The borrowable idea from SCF is the IntentNode pre/postcondition data structure + the three-category symbolic CDE resolved against a shared process/state model — this is exactly the "semantic" upgrade SHARD's syntactic-only conflict detection is missing, and it is small/rule-based enough to port without an LLM. What is NOT borrowable: the simulated benchmark numbers (do not cite 100% vs 25.1% as evidence), the README's 74.3%/145ms figures (unreconciled), and the framework-adapter integration (vaporware in shipped code). SCF adds nothing to SHARD's self-improvement pillar (it has none).

=========================================================================
8. CITATIONS (tier-marked)
=========================================================================

[1] arXiv:2604.16339 abstract page — "Semantic Consensus: Process-Aware Conflict Detection and Resolution for Enterprise Multi-Agent LLM Systems", Vivek Acharya, submitted 2026-03-13. https://arxiv.org/abs/2604.16339 — T1 (primary, peer-pending preprint). Accessed 2026-06-01.
[2] arXiv:2604.16339 full HTML — six components, SIG graph definition, eval setup, 600-run / 100%-vs-25.1% claims, ~4,200 LOC. https://arxiv.org/html/2604.16339 — T1. Accessed 2026-06-01.
[3] arXiv:2604.16339 PDF — CC BY-NC-ND 4.0 license metadata, seed=42. https://arxiv.org/pdf/2604.16339 — T1. Accessed 2026-06-01.
[4] GitHub API repo metadata + recursive tree — curiosityexplorer/semantic-consensus, created/pushed 2026-03-10, 0 stars, no license, full file tree, empty tests/, adapters/ only __init__. https://api.github.com/repos/curiosityexplorer/semantic-consensus (+ /git/trees/HEAD?recursive=1) — T1 (source-of-truth repo state). Accessed 2026-06-01.
[5] README.md (repo) — middleware description, install/usage, module map, README-only metrics (74.3%, 145ms), MDPI Applied Sciences citation block. https://raw.githubusercontent.com/curiosityexplorer/semantic-consensus/HEAD/README.md — T1. Accessed 2026-06-01.
[6] WebSearch synthesis of [1][2] terminology — CRP tiers, Drift Monitor SAS, interceptor pattern. arxiv.org result cluster — T1-derived. Accessed 2026-06-01.
[7] arXiv:2604.16339 v1 HTML (results detail) — Table 2 P/R/F1 (27.9/65.2/39.0), conservative-blocking caveat, hybrid rule-based CDE statement, SIMULATION admission, no-overhead-figure statement. https://arxiv.org/html/2604.16339v1 — T1. Accessed 2026-06-01.
[8] Google Scholar profile, Vivek Acharya — Boston University affiliation, research areas, citation counts, 0 cites on SCF paper. https://scholar.google.com/citations?user=fZFYHfgAAAAJ&hl=en — T2 (reputable secondary identity source). Accessed 2026-06-01.
[9] scf/core/sig.py (repo source) — IntentNode dataclass fields, SemanticIntentGraph methods, EdgeType enum. https://raw.githubusercontent.com/curiosityexplorer/semantic-consensus/HEAD/scf/core/sig.py — T1 (source code). Accessed 2026-06-01.
[10] scf/detect/cde.py (repo source) — ConflictDetectionEngine, three _detect_* methods, rule-based symbolic logic, optional SentenceTransformer. https://raw.githubusercontent.com/curiosityexplorer/semantic-consensus/HEAD/scf/detect/cde.py — T1 (source code). Accessed 2026-06-01.
[11] requirements.txt (repo source) — pyyaml/numpy/pandas/matplotlib core; sentence-transformers+torch commented optional; NO autogen/crewai/langgraph/networkx. https://raw.githubusercontent.com/curiosityexplorer/semantic-consensus/HEAD/requirements.txt — T1 (source code). Accessed 2026-06-01.
[12] scf/middleware.py (repo source) — class SemanticConsensusFramework, __init__(pcl, config), process_intent(intent)->InteractionResult, mark_executed(intent_id). https://raw.githubusercontent.com/curiosityexplorer/semantic-consensus/HEAD/scf/middleware.py — T1 (source code). Accessed 2026-06-01.

=========================================================================
9. GROUNDING LEDGER
=========================================================================

  - arXiv 2604.16339 resolves to a real paper -> [1][2][3] (fetched, confirmed).
  - Title/author/date (Acharya, 2026-03-13) -> [1].
  - Author affiliation = Boston University -> [8].
  - Six named components + their roles -> [2][6], each mapped to a real module file -> [4].
  - SIG graph G=(V,E,L), IntentNode fields, EdgeType enum, register_intent/get_conflict_edges -> [9] (source code, verbatim names).
  - CDE rule-based symbolic detection, three _detect_* methods, postcondition/precondition string comparison -> [10] (source) + [7] (paper "hybrid rule-based" statement).
  - CRP policy-authority-temporal 3 tiers -> [2][6].
  - Drift Monitor SAS + threshold re-sync -> [6].
  - PAGI governance/audit -> [2][6].
  - Main API SemanticConsensusFramework.process_intent -> InteractionResult, mark_executed -> [12] (source, verbatim signatures).
  - 600 runs = 50x4x3, frameworks AutoGen v0.4/CrewAI v0.76/LangGraph v0.2, 4 scenarios, 4 baselines, seed 42 -> [2][3][7].
  - 100% vs 25.1% (Judge-Agent next-best) -> [2][6][7]; CAVEAT runs are SIMULATED -> [7] (verbatim).
  - 41-86.7% baseline failure motivation -> [1][2].
  - P/R/F1 27.9/65.2/39.0 + conservative-blocking caveat -> [7] (verbatim).
  - License: paper CC BY-NC-ND 4.0 [3]; repo NO license [4].
  - Maturity: 0 stars/forks/issues, single push, empty tests/, no framework adapter, no live-LLM deps -> [4][11].

  UNGROUNDED / UNRESOLVED (explicitly flagged):
  - UNGROUNDED: README's "74.3% reduction in semantic conflict incidence" and "145ms median governance overhead (<3% of interaction time)" [5] — NOT found in the fetched paper body, and the paper HTML explicitly states no governance-overhead-ms figure / no explicit reduction % is provided [7]. Treat as author-side, unreconciled. Do not cite as measured.
  - UNGROUNDED: README's "Applied Sciences, 2026, MDPI" venue [5] — no MDPI DOI/page located; the canonical fetched artifact is an arXiv preprint. Venue claim UNCONFIRMED.
  - PARTIALLY GROUNDED: Drift Monitor's exact SAS formula and theta threshold value — described qualitatively in [6]; the precise formula was not quoted from a fetched section of monitor.py (only sig.py/cde.py/middleware.py source were fetched). Mechanism direction is grounded; exact math is not.
  - NOT INDEPENDENTLY VERIFIED: all SHARD-side specifics (staleness formula, DECLARE-REVIEW-EXECUTE, T1-T4 tiers, composition.py cascade, syntactic-only detection, no self-improvement loop) were supplied by the ELAI requester and used as given for the comparison in section 7; this dossier did not fetch SHARD's repo/paper to re-confirm them.
