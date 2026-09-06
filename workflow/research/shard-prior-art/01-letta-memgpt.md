Letta (formerly MemGPT) — Prior-Art Dossier
Pillar: memory governance
Compiled: 2026-06-01 (oracle-mode grounded research for ELAI / SHARD comparison)
Grounding floor met: 11 distinct primary-source fetches (paper abstract, ar5iv full method, GitHub repo metadata + file tree + function_sets listing + raw base.py source + cpacker rename-redirect, PyPI JSON, two docs.letta.com pages, one docs WebSearch). No architecture/API/version/benchmark claim below relies on training memory.

========================================================================
1. IDENTITY & PROVENANCE
========================================================================

Names: "MemGPT" was the research-paper / original-project name. The project rebranded to "Letta" (the company is Letta AI). The PyPI package self-describes as "Letta (formerly MemGPT)". [T1: PyPI summary, GitHub description]

Verified rename: the GitHub repo cpacker/MemGPT now resolves (via GitHub API transparent redirect) to letta-ai/letta — same repository, same created_at 2023-10-11T07:38:37Z. This confirms continuity: one repo, renamed org/name, not a fork. [T1: api.github.com/repos/cpacker/MemGPT returns full_name=letta-ai/letta; api.github.com/repos/letta-ai/letta created_at 2023-10-11]

Paper authors / org: Charles Packer, Sarah Wooders, Kevin Lin, Vivian Fang, Shishir G. Patil, Ion Stoica, Joseph E. Gonzalez. (UC Berkeley Sky Computing lab affiliation per the author roster — Stoica/Gonzalez are Berkeley faculty; the affiliation line was not rendered in the fetched abstract page, so org is inferred from the author set, see UNGROUNDED note). [T1: arXiv:2310.08560 abstract page author list]

First release / submission: paper v1 submitted 2023-10-12 (arXiv:2310.08560), revised v2 2024-02-12. Repo created 2023-10-11. [T1: arXiv abstract page; GitHub created_at]

Current version: PyPI letta 0.16.8, uploaded 2026-05-14 17:14:47 UTC. requires_python: >=3.11,<3.14. Recent line: 0.16.8 (2026-05-14), 0.16.7 (2026-03-31), 0.16.6 (2026-03-04), 0.16.5 (2026-02-24), 0.16.4 (2026-01-29). [T1: PyPI JSON]

License: Apache-2.0 (GitHub license.spdx_id=Apache-2.0; PyPI license field "Apache License"). No license mismatch detected between repo and package. [T1: GitHub repo + PyPI]

Verified links:
- Paper: https://arxiv.org/abs/2310.08560 (resolves; full text https://ar5iv.labs.arxiv.org/abs/2310.08560)
- Repo: https://github.com/letta-ai/letta (resolves; 23,064 stars as of 2026-06-01)
- Docs: https://docs.letta.com (resolves)
- PyPI: https://pypi.org/project/letta/ (resolves; SDK client package is "letta_client", imported in docs snippets)

Version/identity flags:
- The pip install package is "letta"; the SDK client import shown in current docs is "letta_client" (from letta_client import Letta). These are distinct distributions — the server/framework (letta) vs the client SDK (letta_client). Do not conflate. [T1: docs memory-blocks snippet imports letta_client; PyPI package name is letta]
- Docs <title> currently markets Letta as "the memory-first coding agent" (a 2026 product pivot toward a coding-agent surface, "Letta Code"), distinct from the 2023 research framing. The underlying memory architecture below is unchanged. [T1: docs.letta.com title; WebSearch surfaced docs.letta.com/letta-code/memory/]

========================================================================
2. PROBLEM IT SOLVES
========================================================================

Target failure mode (verbatim abstract): LLMs "are constrained by limited context windows, hindering their utility in tasks like extended conversations and document analysis." MemGPT's claim is "virtual context management … drawing inspiration from hierarchical memory systems in traditional operating systems that provide the appearance of large memory resources through data movement between fast and slow memory." [T1: arXiv:2310.08560 abstract verbatim]

In one line: it makes a fixed context window behave like a paged virtual-memory hierarchy so an agent can carry unbounded conversational/document state across a bounded window, with the LLM itself deciding what to page in/out. The governance angle (relevant to SHARD): memory edits and retrieval are LLM-self-directed, not externally orchestrated.

========================================================================
3. HOW IT WORKS — ARCHITECTURE & KEY MECHANISMS
========================================================================

3.1 Two-tier memory (the OS analogy)
- Main context (= the prompt tokens; "in-context", directly accessible to the LLM). [T1: ar5iv method]
- External context (= out-of-context data, requires explicit retrieval). [T1: ar5iv method]

3.2 Main context is divided into three sections [T1: ar5iv method, verbatim component names]
  (a) System instructions — read-only; "contains control flow and function usage guidance".
  (b) Working context — a "fixed-size read/write block for storing key facts, preferences, and other important information". (In current Letta this is generalized to labeled "memory blocks" / "core memory".)
  (c) FIFO Queue — rolling message history; "the first index in the FIFO queue stores a system message containing a recursive summary of messages that have been evicted".

3.3 External context storage [T1: ar5iv method]
  - Archival storage — "a read/write database storing arbitrary length text objects" (semantic search).
  - Recall storage — the MemGPT message database (full conversation history, hybrid text+semantic search).

3.4 Self-directed memory editing (the core governance mechanism)
The paper: "Memory edits and retrieval are entirely self-directed: MemGPT autonomously updates and searches through its own memory based on the current context." Implemented by giving the LLM explicit instructions + a function schema in the system instructions. [T1: ar5iv method]

The real, current function set (verbatim from source, letta/functions/function_sets/base.py on main) [T1: raw.githubusercontent.com source]:
  - memory(agent_state, command, ...) — umbrella memory-management tool with sub-commands.
  - core_memory_append(agent_state, label, content) -> str — append to a core-memory block.
  - core_memory_replace(agent_state, label, old_content, new_content) -> str — replace within a block.
  - memory_replace(agent_state, label, old_string, new_string) -> str — string replace in a block.
  - memory_insert(agent_state, label, new_string, insert_line=-1) -> str — insert at a line.
  - memory_apply_patch(agent_state, label, patch) -> str — apply a "simplified unified-diff style patch" to one or more blocks.
  - memory_rethink(agent_state, label, new_memory) -> str — completely rewrite a block.
  - rethink_memory(agent_state, new_memory, target_block_label) -> None — rewrite main-agent block with integrated info.
  - memory_finish_edits(agent_state) -> None — signal edits complete.
  - conversation_search(self, query, ...) — hybrid (text + semantic) search over conversation history (recall).
  - archival_memory_insert(self, content, tags=None) -> Optional[str] — write to long-term archival memory.
  - archival_memory_search(self, query, ...) — semantic search over archival memory.
  - send_message(self, message) -> Optional[str] — message the human user.

Note the patch/diff lineage: memory_apply_patch (unified-diff) and memory_replace (old_string -> new_string) are code-edit-style operations applied to prose memory blocks — a 2026 generalization of the 2023 core_memory_replace primitive.

3.5 Control flow — queue manager, eviction, heartbeats [T1: ar5iv method]
- Queue manager: appends messages, triggers LLM inference, writes inputs/outputs to recall storage.
- Eviction policy: at a "warning token count" (e.g. 70% of window) a system message tells the LLM to persist important info; at the "flush token count" (e.g. 100%) the queue manager "evicts a specific count of messages (e.g. 50% of the context window), generates a new recursive summary" and prepends it. This is the paging-out step of the OS analogy.
- Function chaining / heartbeats: "Functions can be called with a special flag that requests control be immediately returned to the processor after the requested function completes execution." The flag is request_heartbeat=true (paper Fig 3), enabling multi-step retrieve-then-act chains without yielding to the user. [T1: ar5iv, Fig 3 caption]

3.6 Memory blocks as rendered today (current product) [T1: docs memory-blocks]
A block has four fields: label (unique id), description (tells the agent how to read/write it), value (contents), limit (size limit in characters). Blocks are "always visible — no retrieval needed" and are prepended to the prompt in an XML-like form carrying live char counters:
  <memory_blocks><persona><description>...</description><metadata>- chars_current=128 / - chars_limit=5000</metadata><value>...</value></persona></memory_blocks>

3.7 Sleep-time agents (the asynchronous memory-maintenance subsystem) [T1: docs sleeptime]
- A background agent that "share[s] the memory of your primary agents … and can modify the memory asynchronously".
- It "reflect[s] on the original context to iteratively derive a learned context" reflecting the most important info, and writes that learned context into a (possibly shared) memory block.
- Triggered every N steps (default 5), config sleeptime_agent_frequency; enabled via enable_sleeptime: true at agent creation; Letta auto-creates the primary+sleep-time pair as a managed group.
- The docs explicitly DECLINE to call this "self-improvement"; they frame it as a functional division of labor (background context distillation), not a scored/gated improvement loop. [T1: docs sleeptime — "documentation does not explicitly characterize this as self-improvement"]

========================================================================
4. CORE API / USAGE (verbatim-in-spirit from docs)
========================================================================

Create a stateful agent with two core-memory blocks (current Python SDK, from docs.letta.com/guides/agents/memory-blocks) [T1]:

  from letta_client import Letta
  import os

  client = Letta(api_key=os.getenv("LETTA_API_KEY"))

  agent_state = client.agents.create(
      memory_blocks=[
          {"label": "human",   "value": "The human's name is Bob the Builder.", "limit": 5000},
          {"label": "persona", "value": "My name is Sam, the all-knowing sentient AI.", "limit": 5000},
      ],
      model="openai/gpt-4o-mini",
  )

Memory-edit surface the agent calls autonomously (real names, see 3.4): core_memory_append / core_memory_replace / memory_replace / memory_insert / memory_apply_patch / memory_rethink / archival_memory_insert / archival_memory_search / conversation_search / memory_finish_edits. Sleep-time variant: client.agents.create(..., enable_sleeptime=True, sleeptime_agent_frequency=5).

========================================================================
5. MEASURED NUMBERS / BENCHMARKS
========================================================================

All figures below are vendor/author-reported from arXiv:2310.08560 (the authors' own eval harness, 2023-24). Not independently reproduced here. [T1: ar5iv eval section]

Deep Memory Retrieval (DMR) accuracy:
  - GPT-4 + MemGPT: 92.5%
  - GPT-4 Turbo + MemGPT: 93.4%
  - GPT-3.5 Turbo + MemGPT: 66.9%
Conversation Opener (SIM-1 metric): MemGPT (GPT-4 Turbo) 0.857.
Document QA: MemGPT (GPT-4) "maintained accuracy across [increasing] document counts" (vs context-truncation baselines that degrade) — qualitative trend, exact baseline deltas not captured in fetch.
Nested KV Retrieval: MemGPT (GPT-4) reported 0% accuracy drop through 4 nesting levels.

Caveats: sample sizes / exact N per task not surfaced in the fetched method/eval excerpt; DMR/Conversation-Opener are author-defined tasks (not a community-standard public leaderboard at publication). Model versions (GPT-4 / GPT-4 Turbo / GPT-3.5 Turbo) are 2023-era and now superseded — figures do not reflect 2026 base models. Treat all as VENDOR-REPORTED, UNREPRODUCED.

========================================================================
6. MATURITY SIGNALS
========================================================================

- GitHub stars: 23,064; forks: 2,457; open issues: 54. [T1: GitHub API, 2026-06-01]
- Activity: pushed_at 2026-05-14, updated_at 2026-06-01 — actively developed. [T1: GitHub API]
- Releases: PyPI 0.16.8 on 2026-05-14, with monthly-ish cadence (5 releases Jan–May 2026). [T1: PyPI]
- License: Apache-2.0 (permissive, commercial-friendly). [T1]
- Commercial entity: Letta AI exists as a company (contact@letta.com PyPI author; "Letta Cloud" / hosted ADE referenced in docs). Funding amounts NOT fetched — see UNGROUNDED.
- Production users: the repo/docs assert production usage but I did NOT fetch a verifiable named-customer page. Treat "production users" as UNGROUNDED beyond the project's own marketing.

========================================================================
7. VERSUS SHARD'S MEMORY-GOVERNANCE PILLAR
========================================================================

SHARD reference specifics (from requester): staleness = (now - last_verified)/expected_lifetime, flag>1.0 / quarantine>2.0; DECLARE-REVIEW-EXECUTE coordination with semantic conflict detection; T1–T4 skill trust tiers; safety-constrained self-improvement with .bak rollback + 0–3 scoring; composition.py cross-subsystem self-healing cascade. Caveat (requester-supplied, treated as ground truth here): SHARD's repo implements only SYNTACTIC conflict detection and has NO self-improvement loop.

Where Letta is STRONGER (grounded):
1. Shipped & battle-tested. 23k stars, Apache-2.0, monthly releases, hosted platform. SHARD is a paper-stage design whose repo under-implements its own spec (syntactic-only conflict detection, no self-improvement loop). Letta's memory model is running in production today. [T1: GitHub/PyPI vs requester note on SHARD repo]
2. Real eviction/paging policy. Letta's FIFO + warning(70%)/flush(100%) + recursive-summarization gives a concrete, tested mechanism for what to drop when the window fills. SHARD's pillar (as described) governs validity/trust but has no described context-window paging policy — a gap Letta covers. [T1: ar5iv eviction policy]
3. Rich self-edit surface + semantic retrieval. core_memory_* / memory_apply_patch (unified-diff) / archival_memory_search (semantic) / conversation_search (hybrid) are a mature, code-edit-grade toolset. SHARD's memory ops are not described at this primitive granularity. [T1: base.py]
4. Asynchronous maintenance loop already exists. Sleep-time agents distill "learned context" every N=5 steps — a working background-consolidation mechanism. SHARD has no described async consolidation agent. [T1: docs sleeptime]

Where SHARD is STRONGER (i.e. mechanisms Letta LACKS — grounded by absence in fetched Letta sources):
1. Temporal validity / staleness. Letta memory blocks carry only {label, description, value, limit} — limit is a CHARACTER cap, NOT a temporal one. There is no last_verified timestamp, no expected_lifetime, no staleness ratio, no flag/quarantine threshold. Letta memory never "expires" or gets quarantined for age; it only gets overwritten or evicted by recency (FIFO). SHARD's staleness=(now-last_verified)/expected_lifetime with flag>1.0/quarantine>2.0 is a governance dimension Letta simply does not model. [Grounded-by-absence: docs memory-blocks block schema has no temporal field; base.py edit funcs take no validity arg]
2. Conflict-coordinated writes. Letta self-edits and sleep-time writes are unsynchronized: sleep-time "modif[ies] the memory asynchronously" and shares blocks with the primary agent, but the docs describe NO conflict-detection/resolution gate — it is effectively last-writer-wins on a shared block. SHARD's DECLARE-REVIEW-EXECUTE with conflict detection is a write-coordination protocol Letta has no analog for. (Honest symmetry: SHARD's shipped conflict detection is SYNTACTIC-only per requester, so SHARD's advantage here is design-level, not a fully-realized semantic gate.) [Grounded-by-absence: docs sleeptime describes async shared-block writes, no coordination protocol named]
3. Trust tiers. Letta has no T1–T4 skill/memory trust stratification; all memory blocks are equally authoritative within their labels. SHARD's tiering is absent in Letta. [Grounded-by-absence: no trust/tier field in block schema or tool set]
4. Scored, rollback-gated self-improvement. Letta's memory edits are not versioned with .bak snapshots, not scored 0–3, not gated on a safety score before commit. memory_apply_patch/memory_rethink mutate in place; rollback is not a described primitive. Letta's docs explicitly DO NOT frame sleep-time as self-improvement. SHARD's .bak rollback + 0–3 scoring loop is a governance layer Letta omits. (Symmetry: per requester, SHARD's self-improvement loop is also NOT implemented in its repo — so on shipped reality the two are closer than on paper.) [T1: docs sleeptime declines "self-improvement"; grounded-by-absence on versioning/scoring in base.py]
5. Cross-subsystem self-healing. No Letta analog to SHARD's composition.py self-healing cascade was found. [Grounded-by-absence]

Net read for ELAI: Letta is the right prior art for the "what to keep in-context vs page out, and how an agent self-edits prose memory" half of memory governance — borrow its paging policy (FIFO+warning/flush+recursive summary), its block schema, and its self-edit tool granularity (esp. memory_apply_patch as a safe diff primitive). SHARD's differentiation is the GOVERNANCE half Letta lacks: temporal staleness/quarantine, write-conflict coordination, trust tiers, and scored+rollback self-improvement. The strongest defensible SHARD claim vs Letta is the staleness/quarantine dimension, because it is cleanly absent from Letta's block schema (not just under-emphasized). The weakest is "self-improvement", since SHARD's own loop is unimplemented and Letta at least ships an async consolidation agent (sleep-time) that does part of the job.

========================================================================
8. CITATIONS (tier-marked)
========================================================================

[1] T1 — MemGPT: Towards LLMs as Operating Systems (abstract, authors, dates). arXiv:2310.08560. https://arxiv.org/abs/2310.08560 (fetched 2026-06-01)
[2] T1 — MemGPT full method + evaluation (memory tiers, eviction, heartbeats, DMR/KV numbers). https://ar5iv.labs.arxiv.org/abs/2310.08560 (fetched 2026-06-01)
[3] T1 — letta-ai/letta repo metadata (stars, license, dates, description). https://api.github.com/repos/letta-ai/letta (fetched 2026-06-01)
[4] T1 — cpacker/MemGPT -> letta-ai/letta rename confirmation (API redirect). https://api.github.com/repos/cpacker/MemGPT (fetched 2026-06-01)
[5] T1 — repo file tree (package layout). https://api.github.com/repos/letta-ai/letta/contents/letta (fetched 2026-06-01)
[6] T1 — function_sets listing. https://api.github.com/repos/letta-ai/letta/contents/letta/functions/function_sets (fetched 2026-06-01)
[7] T1 — base.py source (verbatim memory-edit/retrieval function names + signatures). https://raw.githubusercontent.com/letta-ai/letta/main/letta/functions/function_sets/base.py (fetched 2026-06-01)
[8] T1 — PyPI letta package (version 0.16.8, license, python req, release dates). https://pypi.org/pypi/letta/json (fetched 2026-06-01)
[9] T1 — Memory blocks (core memory) docs (block schema, XML render, SDK snippet). https://docs.letta.com/guides/agents/memory-blocks/ (fetched 2026-06-01)
[10] T1 — Sleep-time agents docs (async maintenance, N=5, enable_sleeptime). https://docs.letta.com/guides/agents/architectures/sleeptime/ (fetched 2026-06-01)
[11] T1 — docs.letta.com index + WebSearch over docs (product framing, archival/recall, navigation map). https://docs.letta.com ; WebSearch results listing /guides/agents/memory/, /core-concepts/, archival-memory (fetched 2026-06-01)

========================================================================
9. GROUNDING LEDGER
========================================================================

- Name "Letta (formerly MemGPT)" -> [8] PyPI summary + [3] GitHub description.
- cpacker/MemGPT renamed to letta-ai/letta -> [4] API of old slug returns full_name=letta-ai/letta; [3] created_at 2023-10-11 matches.
- Authors (Packer, Wooders, Lin, Fang, Patil, Stoica, Gonzalez) -> [1] abstract page roster.
- Paper dates v1 2023-10-12 / v2 2024-02-12 -> [1].
- Version 0.16.8, python >=3.11,<3.14, Apache license, release dates -> [8].
- Apache-2.0 license, 23,064 stars / 2,457 forks / 54 issues, push 2026-05-14 -> [3].
- Two-tier memory; main-context = system instr + working context + FIFO; external = archival + recall; recursive summary at queue head -> [2].
- Eviction warning(70%)/flush(100%), evict ~50%, recursive summary -> [2].
- request_heartbeat=true function chaining -> [2] Fig 3.
- DMR 92.5% / 93.4% / 66.9%; SIM-1 0.857; KV 0% drop x4 levels -> [2] (VENDOR-REPORTED, UNREPRODUCED).
- Real function names core_memory_append/replace, memory_replace/insert/apply_patch/rethink, archival_memory_insert/search, conversation_search, memory_finish_edits, send_message -> [7] source verbatim.
- Block schema {label, description, value, limit(chars)} + XML render -> [9].
- Python SDK create-agent snippet (letta_client) -> [9].
- Sleep-time: async shared-block writes, every N=5 (sleeptime_agent_frequency), enable_sleeptime, "learned context", docs decline "self-improvement" -> [10].
- "Memory-first coding agent" product framing -> [11] docs title.
- Letta blocks have NO staleness/temporal field (SHARD comparison pt 1) -> grounded-by-absence in [9] schema + [7] edit-func signatures (no last_verified/lifetime arg anywhere).
- Letta has NO conflict-coordination protocol for shared-block writes (pt 2) -> grounded-by-absence in [10] (async writes described, no gate named).
- Letta has NO trust tiers / NO scored+.bak rollback self-improvement (pts 3-4) -> grounded-by-absence in [7]/[9]/[10]; reinforced by [10] explicitly declining "self-improvement".

UNGROUNDED (could not verify with a fetched primary source):
- UNGROUNDED: author affiliation "UC Berkeley Sky Computing lab" — inferred from the author roster (Stoica/Gonzalez), NOT read off the fetched abstract page (affiliation line did not render). Verify against the PDF before citing institutionally.
- UNGROUNDED: Letta AI funding round / amount / investors — not fetched.
- UNGROUNDED: named production customers — project asserts production use; no verifiable customer page fetched. Do not cite specific users.
- UNGROUNDED: exact per-task sample sizes (N) for DMR / Conversation-Opener / KV evals — not surfaced in the fetched excerpt.
- UNGROUNDED: Document-QA exact baseline deltas — only the qualitative "maintained accuracy" trend was captured.
