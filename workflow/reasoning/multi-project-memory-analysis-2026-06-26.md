MULTI-PROJECT / MULTI-REPO MEMORY + CONTEXT — ELAI ANALYSIS
2026-06-26. Evaluation mode: AS-SPECIFIED (spec-eval). Grounded-by-default: every
ELAI claim cites the feature map (docs/thefinalglance/ELAI-FEATURE-MAP.md) or live
code (crates/core/src/paths.rs). Comparator (Polygraph) grounded via web research.

THE QUESTION (verbatim intent)
Now that ELAI has multi-project surfaces like canvas, what about its MEMORY? Can it
work on all connected OR disconnected nodes/projects at the same time — handling
multi-project memory/context, including multi-repo support? Polygraph.com ships a
multi-repo memory system (closed-source) — compare.

================================================================================
1. WHAT ELAI ACTUALLY IS TODAY (code-grounded, not aspirational)
================================================================================

ELAI is architecturally a SINGLE-ROOT system. Every live knowledge and memory store
is a pure function of one `project_root`. This is not a config detail; it is the
spine of the whole data layer. From crates/core/src/paths.rs:

  - DKG (dependency knowledge graph):  <project_root>/.elai_cc/indexes/dkg.db
  - BM25 index:                        <project_root>/.elai_cc/indexes/projects/default/bm25
                                       (BM25_PROJECT_ID is the literal "default" —
                                       ONE project per root, paths.rs:173)
  - vector index:                      <project_root>/.elai_cc/indexes/vector_index.json
  - vector cache:                      <project_root>/.elai_cc/indexes/vector_cache.db
  - user memory facts (crates/memory): <project_root>/.elai_cc/<data-dir>/memory_facts.sqlite
  - held-out task store:               per-project sqlite
  - sessions (SSB journals live here): <project_root>/.elai_cc/sessions
  - metrics:                           <project_root>/.elai_cc/metrics

The SSB (Shared State Board) — the append-only single-writer journal that is the
load-bearing record for everything (FSM transitions, agent lifecycle, grounding,
memory governance, cost) — is a per-session SQLite WAL under that per-root sessions
dir (feature-map 1.4; ssb.rs). Retrieval (BM25 + vector + PageRank + CatRAG/A-RAG)
is per-project (feature-map 1.1). User memory (Mem0 ADD/UPDATE/DELETE dispatcher +
Zep bi-temporal + MINJA poisoning defense, crates/memory) is a per-project sqlite
keyed to one root (feature-map 1.5, E-5).

CONCLUSION OF FACT: ELAI's "memory" is a stack of per-project silos. Switch repos
and you switch to a different, isolated DKG / BM25 / vector / memory_facts / SSB.
There is no shared index spanning two repos, and no retrieval query that returns
results from repo B while you are rooted in repo A.

THE ONE CROSS-PROJECT MECHANISM THAT EXISTS
COSMOS ARTIFACT TIER [WIRED] (feature-map 1.1, B-42 + C-16): a GLOBAL store at
~/.elai/cosmos/ (NOT under any project_root) holding ~12 default artifacts, a
briefing compiler, a CosmosStage in the pipeline, a VerificationMethod::CosmosCheck
grounding method (the 7th grounding method), cosmos_worker (~700 LOC), CosmosDrawer.
This is the only thing in ELAI that deliberately crosses project boundaries.

What Cosmos IS: a cross-project ARTIFACT and briefing store — durable distilled
notes/templates/briefings that any project can read and that can ground a claim
(CosmosCheck). It is artifact-grained, curated, and small (~12 default artifacts).
What Cosmos is NOT: it is not a unified code knowledge graph across repos. It does
not index repo B's symbols, call graph, or files into a queryable structure usable
from repo A. It carries distilled artifacts, not live cross-repo code intelligence.

================================================================================
2. DIRECT ANSWERS TO THE LITERAL QUESTIONS
================================================================================

Q: Can ELAI work on connected OR disconnected projects AT THE SAME TIME?
A: No, not as one unified memory/context. A live ELAI session is bound to one
   project_root. The dashboard/canvas can LIST and SWITCH projects (canvas H-11
   activates the orphaned list_projects / create_project commands, feature-map 1.10)
   — but switching means tearing down one single-project context and standing up
   another. There is no simultaneous, cross-root reasoning surface where an agent
   queries two repos' indexes in one retrieval pass.

Q: Does ELAI handle multi-project MEMORY/CONTEXT?
A: Partially, and only at the artifact tier. Cosmos is the cross-project bridge
   (distilled artifacts + briefings + CosmosCheck grounding). Everything else —
   code retrieval, the DKG, user-preference memory, the SSB journal, sessions — is
   single-root. So "shared knowledge across projects" today = "a curated global
   artifact shelf", not "a queryable cross-repo knowledge graph or shared episodic
   memory".

Q: Multi-repo support (in the Polygraph sense)?
A: Not shipped and not planned. Grepping the full feature map + every plan
   SUMMARY_INDEX (B..K): there is NO entry for a cross-repo knowledge graph, a
   multi-repo retrieval router, or simultaneous multi-root indexing. The closest
   adjacent items are all a DIFFERENT axis (see section 3).

================================================================================
3. THE TWO THINGS PEOPLE CONFUSE WITH MULTI-REPO MEMORY (they are NOT it)
================================================================================

A. MULTI-TENANT / PER-SEAT (a USER axis, not a PROJECT axis).
   J-49 multi-seat admission + per-seat quota [PLANNED], J-59 per-seat memory +
   trust isolation [PLANNED] (feature-map 1.5/1.3). These namespace memory by
   SEAT-ID (different humans) for fair-share, spend isolation, and the auditor-
   objectivity firewall. This is about keeping User A's memory away from User B —
   the OPPOSITE of unifying repos. It actively isolates, it does not federate.
   The whole posture is "isolation-first, NO cross-seat sharing v1" (J-59). So this
   does not give multi-repo memory; if anything it is the wrong-direction precedent.

B. CANVAS MULTI-PROJECT UI (a VIEW axis, not a MEMORY axis).
   Canvas H-11 activates list_projects/create_project; J-31 system mode, J-60
   process mode, J-98 memory/case altitude (feature-map 1.10). These are projection
   surfaces over the SSB journal of the CURRENT root. J-98's "memory/CASE altitude"
   is genuinely a memory-visualization layer, but it projects the per-root verified
   case graph (J-96/J-97) and is explicitly "per-user via J-59 per-seat isolation" —
   again single-root, per-seat. Canvas makes one project's memory visible and
   steerable; it does not stitch repos together.

So the honest status: ELAI has invested heavily in WITHIN-project memory depth
(DKG, CatRAG, Mem0/Zep user memory, verified case graph, belief-state carriers,
context-lifecycle regimes) and in MULTI-USER isolation — but the cross-REPO axis
is a genuine whitespace. Cosmos is the only stone laid on it.

================================================================================
4. POLYGRAPH COMPARISON  (research-grounded; sonnet agent, web sources cited)
================================================================================

IDENTITY: the product is Nx's "Polygraph" at trypolygraph.com (NOT polygraph.com,
which is unrelated). Built by the Nx/nrwl team (Victor Savkin, Jeff Cross — ex-Angular).
It is a META-HARNESS that sits ON TOP of existing agents (Claude Code, Codex,
OpenCode) and supplies the two things a single-repo agent structurally lacks. Savkin
names them as two axes: SPATIAL (across repos) and TEMPORAL (across sessions). It
reframes a polyrepo org as a "synthetic monorepo" without moving code. Closed-source,
cloud-hosted on Nx Cloud (no self-hosted option). Sources: nx.dev/blog/announcing-
polygraph, docs.trypolygraph.com, Product Hunt maker Q&A, github.com/nrwl/polygraph-skills.

POLYGRAPH'S ARCHITECTURE (two graphs + LIVE code, NOT a unified embedding index):
  1. DEPENDENCY GRAPH (org-level, static-ish): inter-repo edges from package
     publish/consume + API producer/consumer relationships, harvested from Nx CI
     runs. Updated daily / on-demand (rationale: inter-repo deps change slowly). Used
     for PLANNING/ROUTING — which repos are relevant, in what order, which downstream
     consumers need updating — NOT for code retrieval.
  2. SESSION GRAPH (per-session, episodic, org-shared): every agent session captured
     — traces, reasoning, repo states, branches, commits, PRs, decision logs. "A meta
     graph of how the work came to be." Updated on commit / PR. Stored as NORMALIZED
     transcripts so a Claude Code session resumes in Codex (framework-portable). Shared
     org-wide: "every session feeds one shared memory" — your agent builds on what the
     whole TEAM did. Access-gated by repo permission (you can only touch a session if
     you can read all its repos).
  3. CODE ACCESS — the architecturally distinctive choice: NOT pre-built embeddings.
     Direct maker quote: "polygraph will actually check out the repo locally and
     delegate subagents to work inside them so we have the actual code in front of us,
     not just embeddings." Sequence: dep-graph plans -> check out the relevant repo
     locally -> delegate a `polygraph-delegate-subagent` to work inside it -> poll for
     completion. Delivered as an MCP server (@polygraph/mcp) + skills/subagents
     installed into each agent framework; agent-to-agent coordination over A2A.
  Staleness: dep graph daily; session graph on commit/PR; code is never pre-indexed,
  so code-staleness is a non-issue (live checkout).

THE SHARP CONTRAST WITH ELAI (this is the important part):

  AXIS                 | POLYGRAPH                      | ELAI (today, grounded)
  ---------------------|--------------------------------|----------------------------------
  Cross-repo (spatial) | YES — org dep graph + live      | NO — single project_root; Cosmos
                       | checkout/delegate               | artifacts only (paths.rs; B-42)
  Cross-session (temp.)| YES — org-shared session graph, | WITHIN-project only: SSB journal +
                       | framework-portable transcripts  | session DAG + rewind + handoff +
                       |                                 | verified case graph (J-96). NOT
                       |                                 | org-shared, NOT framework-portable
  Code intelligence    | NONE pre-built ("not just       | DEEP per-repo: DKG + BM25 + vector +
                       | embeddings"); live checkout     | PageRank + CatRAG + symbol-impact
                       |                                 | cone (feature-map 1.1) — but
                       |                                 | single-root
  Store philosophy     | CENTRALIZED cloud (Nx Cloud),   | LOCAL journal-as-record (GD-2),
                       | closed, org-shared              | single-writer SSB, replayable,
                       |                                 | taint-tagged, single-user
  Delegation machinery | subagents check out + work in   | orchestrate/auto-orchestrate +
                       | each repo, poll completion (MCP)| Tier-C isolated subprocess agents
                       |                                 | + worktree pool (I-2/J-80) +
                       |                                 | poll/notify (J-2/H-36)

THE KEY INSIGHT FROM THE COMPARISON
Polygraph deliberately did NOT build the thing ELAI is strongest at (a pre-built
per-repo code knowledge graph). It bet that for CROSS-repo work, live checkout +
subagent delegation beats cross-repo embeddings — and used a lightweight DEPENDENCY
graph purely for planning/routing. That is striking because:
  - ELAI ALREADY HAS the delegation half. orchestrate/auto-orchestrate dispatch
    Tier-C-isolated subagents; J-80 worktree pool gives per-agent isolated checkouts;
    J-2 (ELAI-as-MCP) + H-36 notifications give the poll/notify loop. The Polygraph
    "delegate a live subagent into the repo" pattern is, structurally, an ELAI
    orchestrate run pointed at a different root.
  - What ELAI is MISSING is exactly Polygraph's two graphs: (a) a cross-repo
    DEPENDENCY graph for planning ("which repos, what order, which consumers"), and
    (b) an episodic session memory that is portable/shareable ACROSS roots (ELAI's
    SSB+case-graph is the right substrate but is single-root and single-user).
  - And ELAI's spine FORBIDS Polygraph's biggest design choice: a centralized cloud
    store. ELAI's GD-2 (journal is the record, never an external store) + F-4 (can't-
    lie) + single-user posture mean ELAI's multi-repo answer must be a LOCAL federation
    of per-root journals, not a cloud session graph. That is a real differentiator, not
    a deficiency — but it makes the org-SHARED-team-memory feature (Polygraph's headline)
    out of scope until/unless the per-seat work (J-49/J-59) is extended to sharing.

================================================================================
5. DESIGN REASONING — COULD / SHOULD ELAI DO UNIFIED MULTI-REPO MEMORY?
   (spec-eval IDEA/DESIGN-IMPROVEMENT: propose mechanism, attack it, 2 approaches)
================================================================================

WHY THIS MATTERS FOR ELAI SPECIFICALLY
Real work spans repos: a frontend repo + its backend API + a shared types/protocol
package; a service + its infra repo; an app + a vendored SDK. An agent fixing a
breaking change in the protocol package needs to see who consumes it across repos.
ELAI's single-root spine makes that a manual, lossy, copy-paste exercise today
(or a Cosmos artifact someone hand-curates). Polygraph's wedge is exactly this.

THE LOAD-BEARING CONSTRAINT (do not break the spine to get the feature)
ELAI's entire trust story rests on: (1) the SSB single-writer append-only journal
as THE record (GD-2: never an external store, re-derive byte-identically); (2)
deterministic, replayable, taint-tagged provenance (G-1); (3) per-project gates.
Any multi-repo memory design that introduces a second, external, mutable store
(a cloud graph DB, a shared vector service) directly violates GD-2 and the
can't-lie spine (F-4). That is the cheapest thing to get wrong, and it is the thing
a naive "just add a cross-repo vector DB" port WOULD get wrong. The design must keep
each repo's journal as its own source of truth and build the cross-repo layer as a
DERIVED, re-computable projection — never a primary store.

APPROACH 1 — FEDERATED, QUERY-TIME ROUTER (lighter, spine-preserving)
Keep every repo exactly as it is: its own DKG/BM25/vector/SSB under its own root.
Add a thin WORKSPACE layer above roots:
  - a workspace manifest (a list of member project_roots + their relationship, e.g.
    "consumes-types-from", "deploys"), itself a Cosmos-tier artifact (so it inherits
    CosmosCheck grounding and the global ~/.elai/cosmos/ store — no new store class).
  - a multi-root retrieval router: a query fans out to each member root's existing
    pipeline (BM25->vector->PageRank->CatRAG) in parallel, results merged by RRF
    (ELAI already runs RRF k=60, feature-map 1.1) with a per-root provenance tag so
    every hit says which repo it came from. No new index; reuse N existing ones.
  - cross-repo edges only where they are CHEAP and SOUND: symbol-name + import-path
    matching between repo A's exported symbols and repo B's imports (the protocol/
    types-package case), surfaced as a low-confidence DKG-adjacent edge, never
    asserted as a compiler-grade call edge (R5 sound-reference discipline).
Pros: preserves every gate, every journal stays single-writer, zero new store,
incremental, kills the "switch repo = lose context" failure for the common case.
Cons: query-time fan-out cost scales with member count; cross-repo edges are
name-heuristic (false positives) until an LSP/SCIP cross-index exists; no unified
graph traversal across repos (you get federated retrieval, not one graph).
This is the J-18 LSP channel + J-38 retrieval-loop + Cosmos, composed — mostly
wiring of already-planned pieces, which is the cheapest honest path.

APPROACH 2 — UNIFIED CROSS-REPO KNOWLEDGE GRAPH (heavier, the Polygraph shape)
Build one graph spanning member repos: a super-DKG that ingests each repo's symbol
graph and adds resolved cross-repo edges (export->import, API-route->client-call,
shared-type references) via the planned LSP/SCIP channel (J-18) rather than name
heuristics. Retrieval and impact analysis (B-29 symbol-impact API, the J-16 blast-
radius gate) then traverse across repo boundaries: "change this protocol field ->
these 3 consumers in 2 other repos."
Pros: this is the actual capability — cross-repo blast radius, the thing that makes
multi-repo agents trustworthy. It directly extends ELAI's strongest existing asset
(the DKG + symbol-impact cone) rather than bolting on a foreign system.
Cons: expensive and risky. (a) Provenance/journal: a unified graph derived from N
journals needs a clear ownership rule — keep it a DERIVED projection re-computable
from the N member journals (GD-2), with the graph itself NOT a source of truth. (b)
Staleness: N repos change independently; the blake3-incremental CASCADE that keeps
one DKG fresh now has to fan across roots. (c) The cross-repo edges are the whole
value AND the whole risk — a wrong cross-repo edge produces a confidently-wrong
blast radius, which is worse than no edge. This MUST gate on the LSP/SCIP sound-
reference channel (J-18), not tree-sitter name-grep, or it fails R5/R6.

APPROACH 3 — DEPENDENCY-GRAPH-PLANS + LIVE-DELEGATION (the Polygraph shape; and it
fits ELAI's existing machinery BEST). Do not pre-index code across repos at all. Add:
  - a cross-repo DEPENDENCY graph only (package publish/consume + API producer/consumer
    edges; ELAI can derive these from each repo's own DKG exports/imports). Used for
    PLANNING — which repos, what order, which downstream consumers — not retrieval.
  - reuse ELAI's ALREADY-BUILT delegation: an orchestrate/auto-orchestrate run treats
    each member repo as a target root, dispatches a Tier-C-isolated subagent into a
    J-80 worktree checkout of that repo (where the repo's OWN single-root DKG/retrieval
    already works perfectly), polls via J-2/H-36, merges results.
  - episodic memory stays each repo's own SSB journal; the cross-repo layer records a
    workspace-session row (which roots, which order, which PRs) as a Cosmos-tier
    artifact — the ELAI analog of Polygraph's session graph, but local and journal-backed.
Pros: this is ~80% composition of things ELAI already has or has planned (orchestrate +
Tier-C + J-80 worktree pool + per-root DKG + Cosmos). It sidesteps the cross-repo
embedding-staleness and the unified-graph provenance problems entirely (each repo is
read live by an agent that already has full single-root intelligence). It is the only
approach that is honest for DISCONNECTED repos (no forced edges; the dep graph is just
sparse). It keeps GD-2 intact (no external store; cross-repo state is derived).
Cons: no cross-repo semantic retrieval in a single pass (you delegate instead of
query); cross-repo blast radius is only as good as the dependency edges you can derive
soundly (needs the J-18 LSP/SCIP channel to be trustworthy, else it is name-heuristic);
coordination cost of N live subagents. But these are the same costs Polygraph accepts,
and Polygraph is shipping on exactly this bet.

ADVERSARIAL ATTACK ON ALL THREE (the cheapest falsifiers)
  - Disconnected projects: the user explicitly asked about DISCONNECTED nodes too.
    For genuinely unrelated repos there are NO cross-repo edges to find; the unified
    graph (Approach 2) buys nothing and only adds staleness surface. The honest
    design treats "connected" (shared symbols/imports, real edges) and "disconnected"
    (federated retrieval only, no edges) as two modes — and defaults to Approach-1
    federation, promoting to Approach-2 edges ONLY where a sound cross-repo reference
    is proven. A design that always builds the unified graph is over-engineered for
    the disconnected case.
  - The poisoning surface widens: cross-repo memory means a fact/edge sourced in
    repo A now influences a decision in repo B. ELAI already has the defense shape
    (J-21 commit-aware memory validation, MINJA, the auditor firewall) but it is
    per-project today; multi-repo memory needs that validation to carry the SOURCE
    REPO as part of provenance, or a poisoned low-trust repo contaminates a high-
    trust one. This is a real design requirement, not a nicety.
  - GD-2 trap: the single most likely way to ship this wrong is to stand up a shared
    external graph/vector service "because it's easier than fanning out." That breaks
    the journal-is-the-record spine and the replayability/taint story in one move.
    Both approaches above are constructed specifically to avoid it.

================================================================================
6. VERDICT + MATURITY NOTE
================================================================================
DESIGN VERDICT: Multi-repo memory is a real, well-motivated gap, and it is the RIGHT
shape for ELAI to want. The single most useful grounded finding is that the strongest
shipping comparator (Polygraph) does NOT solve this with cross-repo embeddings — it
uses a cheap dependency graph for PLANNING and LIVE SUBAGENT DELEGATION for code. That
is Approach 3, and it is the recommended primary for ELAI because it is ~80% a
composition of machinery ELAI already has (orchestrate/auto-orchestrate + Tier-C
isolation + J-80 worktree pool + each repo's own single-root DKG/retrieval + Cosmos as
the workspace-session record) and because it is the only approach that stays honest for
DISCONNECTED repos and keeps the GD-2 journal-as-record spine intact.

Recommended sequencing:
  1. WORKSPACE ABSTRACTION ABOVE ROOTS: a workspace manifest (member project_roots +
     relationships) as a Cosmos-tier artifact. Cheapest, unblocks everything.
  2. APPROACH 3 (delegation): wire orchestrate to treat workspace members as targets,
     delegating Tier-C subagents into J-80 worktree checkouts; record a workspace-session
     artifact. This is the Polygraph capability, built on ELAI's existing spine.
  3. CROSS-REPO DEPENDENCY GRAPH for planning: derive package/API edges from each repo's
     own DKG exports/imports — sound where the J-18 LSP/SCIP channel exists, name-
     heuristic (low-confidence, flagged) otherwise. This is what makes step 2's "which
     repos / what order / which consumers" planning real rather than manual.
  4. ONLY THEN, if measured demand: federated query-time retrieval (Approach 1) for the
     cases where you want a single cross-repo search pass, and sound cross-repo DKG edges
     (Approach 2) for true cross-repo blast radius — both gated behind J-18, both derived
     projections, never a new primary store.
Where ELAI deliberately DIVERGES from Polygraph (and should): no centralized cloud
store (GD-2), local journal-as-record, single-user-first. That means Polygraph's
headline "org-shared team memory" is out of scope until the per-seat work (J-49/J-59)
is extended from isolation to opt-in sharing — and that is a SEPARATE, later decision,
not a prerequisite for single-user multi-repo work.

IMPLEMENTATION-READINESS NOTE (build status, NOT part of the verdict; index-derived,
confirm against live code): today ELAI is single-root with Cosmos as the only cross-
project store; per-seat (J-49/J-59) and canvas multi-project (H-11/J-31/J-60/J-98)
are a different axis (multi-user / multi-view), not multi-repo memory. None of the
three approaches above is planned in any current B..K plan — this is genuine
whitespace, not a status lag on an existing plan.

================================================================================
7. CONCRETE MECHANISM SPEC  (designed on the AS-DONE substrate, per spec-eval)
================================================================================

7.0 THE SUBSTRATE ASSUMED FULLY SHIPPED (the as-specified baseline)
This spec is designed ON TOP of these ELAI capabilities treated as DONE+WIRED, not in
their current half-wired state: DKG + BM25 + vector + PageRank + CatRAG + RRF k=60
(per-root retrieval); B-29 symbol-impact cone + J-16 blast-radius auditor gate; J-18
LSP/SCIP sound cross-reference channel; orchestrate/auto-orchestrate + Tier-C process
isolation; J-80 worktree pool (per-agent isolated checkouts, lifecycle/lease/self-heal);
J-2 ELAI-as-MCP control plane (task_submit/status, session_fork); H-36 notifications +
H-51 handoff digest; J-59 per-seat memory + trust isolation; J-21 commit-aware memory
validation; C-48 TrustTier; Cosmos global artifact tier; the SSB single-writer journal +
FSM + phase-3G Kahn cycle detection + 3E context budget allocator. Everything below is
NEW work that COMPOSES these; it invents only the cross-repo layer itself.

7.1 NEW OBJECTS (three; everything else is reuse)

  WORKSPACE — the object above project roots. Concrete shape (TOML, the seats.toml /
  models.toml DATA convention, W-2, no per-member feature flag):
    workspace_id        = "acme-platform"
    seat                = "<J-59 seat-id>"          # owner; per-seat namespaced
    [[member]]
      root      = "F:/work/protocol-types"          # a project_root
      alias     = "types"
      role      = "produces-for"                    # produces-for | consumes-from | peer
    [[member]]
      root      = "F:/work/api-server"
      alias     = "api"
      role      = "consumes-from"
  Stored at ~/.elai/workspaces/<workspace_id>/workspace.toml (GLOBAL, beside Cosmos,
  NOT under any project_root — this is the "above the roots" home). Relationships are
  AUTHORED hints, not the derived edges (those are 7.3). Capability is gated by ONE
  default-OFF flag ELAI_WORKSPACE_ENABLE with a wire-dark-allowlist row + promotion plan
  (R9), never per-member flags.

  CROSS_REPO_DEP_GRAPH — a DERIVED projection (never a source of truth; GD-2). Edge:
    { from_repo, to_repo, kind: PackageDep | ApiDep | TypeRef,
      from_symbol, to_symbol, evidence: "file:line",
      confidence: Sound(J-18 LSP/SCIP) | Heuristic(name+import match) }
  Materialized as a cross_repo_edges table in the workspace store, keyed by
  blake3(concat of each member's DKG index-manifest hash). When ANY member's DKG changes,
  the key changes and the graph re-derives — exactly the blake3-incremental CASCADE
  pattern the single-root DKG already uses. Re-derivable from member DKGs => GD-2 safe.

  WORKSPACE_SESSION — ELAI's local, journal-backed analog of Polygraph's session graph.
  It is the workspace-grain SSB: a single-writer append-only WAL at
  ~/.elai/workspaces/<workspace_id>/workspace.ssb.db. It records ONLY coordination rows
  (7.8) — which members were delegated, in what order, the SSB seq-range each member's
  own journal produced, the PRs, the cross-repo impact computed. It NEVER duplicates
  per-repo content; resume re-derives detail from each member's own SSB + H-51 digest.

7.2 STORAGE LAYOUT (GD-2 posture made explicit)
  ~/.elai/workspaces/<workspace_id>/
    workspace.toml            # authored manifest (DATA)
    workspace.ssb.db          # workspace-grain journal (the ONLY new authoritative store,
                              #   and it is still an append-only single-writer journal —
                              #   GD-2 compliant, same class as a per-root SSB)
    derived/cross_repo_edges  # re-computable projection, blake3-keyed, disposable
  Each member repo is UNTOUCHED: its own <root>/.elai_cc/... stays authoritative for its
  own code intelligence and its own session journals. The workspace store holds only what
  is true ABOUT the workspace, not copies of member state.

7.3 CROSS-REPO DEPENDENCY GRAPH DERIVATION (concrete, sound-vs-heuristic)
  For each member repo, from its already-built DKG:
    PUBLIC SURFACE  = exported symbols + published package name(s) + declared API routes.
    EXTERNAL REFS   = imports of names not locally defined + HTTP client calls to
                      route-shaped strings + references to types not locally declared.
  Edge construction: for every (producer.public_symbol) that matches a
  (consumer.external_ref):
    - if BOTH sides resolve through the J-18 LSP/SCIP channel  -> confidence = Sound.
    - else (name + import-path match only)                     -> confidence = Heuristic.
  Sound edges may drive gates (blast radius, ordering). Heuristic edges are SURFACED with
  a warning badge and NEVER asserted as a real call edge (R5 sound-reference, R6 producer-
  without-consumer discipline). This is the single most important correctness rule in the
  whole design: a wrong cross-repo edge yields a confidently-wrong blast radius, worse than
  no edge — so the gate consumes Sound edges only.

7.4 THE DELEGATION FLOW + FSM (Approach 3, the primary capability)
  New orchestrate mode: `workspace orchestrate`. The controller session roots at the
  WORKSPACE (synthetic root); it never edits member code itself (main-llm-role holds).
  A lightweight workspace FSM rides ABOVE the normal per-root FSM:
    WORKSPACE_PLANNING   -> derive/refresh cross_repo_edges; compute the touched-member
                            set from the task + the dep graph; Kahn-order them over the
                            Sound edges (reuse phase-3G cycle detection; a cycle = tripwire).
    WORKSPACE_DELEGATING -> for each member in order, dispatch a Tier-C sub-agent into a
                            J-80 worktree checkout of that member, running that member's
                            OWN single-root FSM/pipeline (full local intelligence). The
                            sub-agent's work emits to the MEMBER's own SSB (authoritative).
                            Poll/notify via J-2 task_status + H-36. Cap = orchestrate's
                            parallel_cap (3); independent members run parallel, dependent
                            ones serialize by the Kahn order.
    WORKSPACE_MERGING    -> collect each member's PR/branch; run the cross-repo INTEGRATION
                            smoke (wiring-enforcement R10: an independent agent drives the
                            real entrypoints across repos on a HARD input — e.g. types repo
                            changed -> api repo built against the new types -> contract test
                            green). Missing integration evidence = FAIL.
    WORKSPACE_COMPLETE   -> one workspace-session summary row; per-member commits already
                            governed by each member's own commit-cadence.
  Orchestrate's 4 tripwires (irreversible op, 50-event bubble cap, dep-wait>4h, sweep-fail)
  apply unchanged at workspace grain. Worker git stays read-only per agent-dispatch-preamble.

7.5 FEDERATED RETRIEVAL PATH (Approach 1, the "search across all repos" surface)
  `elai workspace search <id> "<query>"`: fan the query to EACH member's existing pipeline
  (BM25->vector->PageRank->CatRAG, all single-root, all already built); each returns a
  ranked list tagged with its source repo; RRF-merge (k=60, already built) over the union;
  every hit carries per-repo provenance. No new index, no cross-repo embedding store — N
  existing indexes queried in parallel and merged. The 3E context budget allocator's
  RepoMap/CodeContext slots now carry multi-repo provenance; a workspace task's injected
  context is the RRF-merged top-k across members, each line labelled with its repo.

7.6 CROSS-REPO BLAST RADIUS (Approach 2, the trust-critical capability)
  Extend the B-29 symbol-impact cone + J-16 auditor gate to traverse cross_repo_edges:
  "change symbol S in repo A" -> cone within A (built) -> at each exported symbol of A that
  has a Sound cross_repo_edge, CONTINUE the cone into the consuming repo's DKG -> union the
  per-repo cones into a workspace blast radius. The J-16 gate becomes cross-repo-aware: a
  change in A that breaks a consumer in B is caught at the workspace gate, not after merge.
  HEURISTIC edges contribute a "possible consumer, unverified" advisory list, never the
  asserted radius. This is the single highest-value capability of the whole design and the
  direct answer to "an agent fixing a shared protocol must see who consumes it across repos."

7.7 EPISODIC CROSS-REPO MEMORY + PROVENANCE + POISONING
  Resume: a session re-opening a workspace reads workspace.ssb.db (what was done, in what
  order, which PRs) + each member's H-51 handoff digest -> reconstructs the cross-repo
  narrative without copying member state. This is the local, journal-backed equivalent of
  Polygraph's session graph.
  Provenance: every cross-repo fact/edge carries source_repo. J-21 commit-aware validation
  is extended so its provenance tuple includes source_repo + that repo's C-48 TrustTier.
  Poisoning firewall: a fact sourced in member A injected into a decision about member B must
  (a) pass J-21 validation against B's current commit+AST, and (b) carry the cross-repo
  provenance badge; a low-trust member cannot silently poison a high-trust one (the H-31
  auditor-objectivity firewall pattern, extended from per-seat to per-REPO trust).
  Per-seat: a workspace belongs to a J-59 seat and is namespaced by it; cross-seat workspace
  sharing is the SAME opt-in-later decision as cross-seat memory sharing — out of scope here.

7.8 NEW SSB ACTION_TYPE (the lockstep; net +1, Pattern-2)
  WorkspaceEvent { kind: WorkspaceEventKind, workspace_id, ... } with discriminator
    WorkspaceEventKind = Created | MemberAdded | MemberRemoved | DependencyGraphDerived
      | SessionStarted | MemberDelegated | MemberCompleted | CrossRepoImpactComputed
      | FederatedQueryRan | SessionMerged
  Emitted to the WORKSPACE ssb.db (not a member's). Full 9-tuple lockstep per the
  extension-points rule (SsbActionType + SsbPayload arm + payload struct + all_test_payloads
  + golden row + TOTAL_VARIANTS bump + SCHEMA_VERSION bump + match arms + ts-rs binding for
  the canvas surface). Payload NEVER carries code content or a seat secret — only ids, repo
  aliases, seq-ranges, hashes, verdicts (the journal-content-minimization rule).

7.9 CANVAS + CLI SURFACES
  CANVAS: a WORKSPACE altitude beside J-31 (system) / J-60 (process) / J-98 (memory) —
  nodes = member repos, edges = cross_repo_edges (colored Sound vs Heuristic), an in-flight
  delegation lights up the member node, the workspace-session + PRs are the temporal layer.
  Re-derived byte-identically from workspace.ssb.db (GD-2, no parallel state). This is the
  literal visual answer to "see all my projects and how they connect at once."
  CLI (each paired with its FSM gate): `elai workspace create <id> --members a,b`,
  `elai workspace add <id> <root>`, `elai workspace map <id>` (print the dep graph),
  `elai workspace search <id> "<q>"` (federated retrieval), `elai workspace orchestrate <id>
  "<task>"` (the delegation flow), `elai workspace impact <id> <repo>:<symbol>` (cross-repo
  blast radius).

7.10 DISCONNECTED-REPO BEHAVIOR (handled by construction, not a special case)
  If two members have no Sound or Heuristic edge, cross_repo_edges is simply empty between
  them. Then: planning has no forced order (delegate to whatever the task names);
  federated search still returns the union of independent results; blast radius does not
  cross (correctly — there is nothing to cross). The design never fabricates a connection to
  make the feature "work"; disconnected = federation-only, which is the honest answer.

7.11 VERTICAL SLICES + DONE-GATES (wiring-enforcement PRE_GATE; ELAI-measured, ID-5)
  Each unit ships producer + consumer + smoke together:
    U1 workspace object + `elai workspace create/add/map` consuming it + a 2-repo fixture.
    U2 dep-graph derivation + `elai workspace map` rendering Sound/Heuristic edges + a
       fixture where types-repo<-api-repo yields exactly one Sound edge, zero false edges.
    U3 federated search + the RRF merge + a fixture where the answer lives in repo B and is
       returned top-k while rooted at the workspace.
    U4 cross-repo blast radius + the J-16 gate extension + a fixture where changing an
       exported type in A surfaces the real consumer in B with ZERO false consumers
       (precision-first; a false consumer is worse than a miss).
    U5 workspace orchestrate (delegation) + WORKSPACE_MERGING integration smoke producing
       2 PRs with a cross-repo contract test green (R10).
    U6 episodic resume + provenance/poisoning extension + canvas workspace altitude.
  Done-gates are ELAI-measured on these fixtures, never a Polygraph or paper number.

7.12 WHAT IS GENUINELY NEW vs COMPOSED (honesty)
  NEW (must be built): the workspace object + manifest; the cross_repo_edges derivation
  (the producer/consumer matching + the Sound/Heuristic split); the workspace SSB +
  WorkspaceEvent; the workspace FSM ring + `workspace orchestrate` mode; the cross-repo
  cone traversal; the canvas workspace altitude.
  COMPOSED (reused as-is): every per-root index/retrieval, RRF, the symbol cone, J-16, the
  J-18 sound channel, orchestrate + Tier-C + J-80 worktrees, J-2/H-36 poll-notify, H-51
  resume, J-21 validation, C-48 trust, J-59 per-seat, Cosmos, the SSB/FSM/Kahn machinery.
  Roughly: the cross-repo LAYER is new; the depth under it is entirely reused. That ratio
  (thin new layer over deep reused substrate) is exactly why this is the right shape — and
  why, on the as-DONE substrate, it is a contained track rather than a from-scratch system.

PLAN-TRACK NOTE: authored as a spec here, not yet a numbered plan. If promoted, it is a NEW
plan letter/track (K is the newest existing -> this would be an L-track, or folded into K),
authored per the Section-5 extension rule: L-0_INDEX + L-SUMMARY_INDEX + per-unit files
(U1..U6 above as the unit seeds) + _grounding map, with ELAI_WORKSPACE_ENABLE in the wire-
dark-allowlist and the feature-map upsert (new "1.15 MULTI-REPO / WORKSPACE" subsystem group).
