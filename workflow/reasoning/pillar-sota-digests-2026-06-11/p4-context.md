DIGEST: p4-context
SOURCE: 2cd3b7ba-63ea-4766-9d86-f267d159e7b4.txt

TOPIC: Pillar P4 (agent context-window management). Audits ELAI's design (8 named slot floors, prompt-cache byte-stable prefix discipline, deterministic tail-drop trim, session DAG + lossless journal, journal-derived handoff, cloud/local dual-lane policy, bi-temporal preference memory, code-grounded fact memory w/ TTL) vs 2024-2026 SOTA.

REPORT VERDICTS (per-component, from Part c table):
- Fixed-slot token-budget allocator: HARDEN — add observation-size pre-assessment (static floors insufficient per BACM-RL).
- 3-level adaptive reduction <50ms: KEEP (validated by Claude Code 5-layer pipeline; timing discipline correct).
- Prompt-cache discipline (byte-stable prefix, no prefix re-summarization): KEEP (fully validated by Manus 10x cost; JetBrains corroborates).
- Deterministic tail-drop: HARDEN — add goal-conditioned relevance scoring (SWE-Pruner goal-guided beats recency by 23-54%).
- Session DAG + lossless trim + journal: KEEP (most rigorous journal-derived architecture in literature; no published alternative matches lossless + branch/merge).
- Handoff digest from journal: HARDEN — add mid-session checkpoint write (session-end-only digest vulnerable to crash-on-overflow).
- Cloud lane (maximize cache-stable prefix, late compaction): KEEP (Manus + Claude Code microcompact).
- Local lane (minimize window, unload early, selective reload): KEEP (VRAM economics; no better published for <=32K).
- No named regime-aware cloud/local policy switching in literature: KEEP — ELAI design is AHEAD (AdaSwitch does capability-routing not window-regime routing).
- Preference memory bi-temporal supersession: HARDEN — add omission-constraint reinforcement (preference dilution under context pressure documented).
- Code-grounded fact memory TTL+revalidation: HARDEN — add semantic-drift detection (TTL catches structural staleness only).
Overall verdict: design is architecturally ahead of most published work on cache-prefix discipline, backend-regime awareness, lossless journal-backed trim; 3 hardening targets (static slot floors, deterministic tail-drop, two-tier memory lacks write-time graph linking).

CANDIDATES (name | url-or-id | report's claim | replaces-or-hardens):
Claude Code | arXiv:2604.14228 (Dive into Claude Code) + mintlify source | only production agent w/ ctx mgmt reverse-engineered at publication quality; 5-layer compaction in query.ts:365-453 (Budget reduction, Snip, Microcompact, Context collapse, Auto-compact); microCompact avoids busting cache keys | reference for ELAI 3-level reduction; auto-compact (step 5) breaks cache-prefix = ELAI's gap-filler for semantic-density spikes
Manus AI | dev.to context-engineering-manus / manus.im blog | KV-cache hit rate as first-order metric; 100:1 input/output ratio; single-token diff busts cache; >80% hit-rate SLO; tool masking over removal; filesystem-as-memory; reversible compaction over summarization | validates ELAI cache-prefix discipline KEEP
SWE-Agent | emergentmind / dev.to / arXiv:2511.22729 | documented failure taxonomy: context explosion from noisy tools, stale context accumulation, multi-session boundary failures, hard autosubmission on overflow (ContextWindowExceededError) | grounds ELAI's static-slot + handoff failure modes
SWE-Pruner | arXiv:2601.16746 github yanami4/s-pruner | 0.6B task-goal-guided neural skimmer selects relevant LINES; 23-54% token reduction SWE-Bench Verified, up to 14.84x on LongCodeQA; success maintained/improved; runs local, no prefix mutation; open source | REPLACES ELAI deterministic tail-drop below cache boundary (Change 2, HIGH)
CAT (Context as a Tool) / SWE-Compressor | arXiv:2512.22087 | context maintenance as callable tool; SWE-Compressor 57.6% solve SWE-Bench-Verified; formalizes stable-semantics + condensed-LT-memory + high-fidelity-ST | hardens handoff/compaction (callable boundary)
Context-Folding + FoldGRPO | arXiv:2510.11967 | branch into sub-trajectories, fold/collapse-with-summary on completion via end-to-end RL; active context 10x smaller than ReAct, outperforms summarization | adjacent (requires fine-tune, not drop-in)
ACON (Microsoft) | arXiv:2510.00615 github microsoft/acon (MIT) | gradient-free compression guidelines via contrastive failure analysis; 26-54% token reduction w/ success improvements; distillable | adjacent (LLM-rewrite busts prefix cache, only post-cache-boundary)
JetBrains empirical study | blog.jetbrains.com 2025/12 | observation masking matched LLM summarization in cost+ability after tuning; summarization caused 15% more turns (Gemini 2.5 Flash); best masking window ~10 recent turns | validates KEEP cache discipline / masking
MemGPT / Letta | arXiv:2310.08560 / noze.it | OS-inspired memory hierarchy, agent issues own memory syscalls; Letta (UC Berkeley Sky Lab, rebrand Sep 2024) productionizes; 3 tiers; Apache 2.0 | memory architecture reference
A-MEM | arXiv:2502.12110 (NeurIPS 2025) | write-time Zettelkasten graph: Note Construction + Link Generation + Memory Evolution; 5.8x LoCoMo multi-hop, ~2x MemGPT temporal F1 | HARDEN ELAI code-grounded fact memory w/ write-time graph linking (Change 3, MODERATE-HIGH)
Mem0 | arXiv:2504.19413 | graph-based production memory, extract/consolidate ADD/UPDATE/DELETE; 92.5 LoCoMo, 94.4 LongMemEval @ ~6900 tok/query; best dev ergonomics (~20min) | memory baseline
NeuSymMS | semanticscholar 25153e93 (arXiv May 2026) | hybrid neuro-symbolic: neural extraction + CLIPS expert system lifecycle rules (classify/dedup/reconcile); subject-relation-value triples in RDBMS; dual-horizon | most relevant to ELAI code-grounded-fact-w/-TTL; symbolic rules > pure TTL for semantic invalidation
BACM / ContextBudget | arXiv:2604.01664 | context mgmt as sequential decision w/ budget; assesses remaining budget then NONE/SELECTIVE/ALL; BACM-RL curriculum 16K->4K modified GRPO; 1.6x gains high-complexity; 30B/8K beats 235B/128K on BrowseComp-Plus | core insight for ELAI deferred-observation-loading (Change 1, HIGH); no code released
CONF-KV | huggingface 2605.24786 (arXiv May 2026) | adjusts KV retention budget by model confidence (next-token entropy); retains more when uncertain, prunes when confident; plug-in | adjacent local-lane KV policy
SCOPE | aclanthology 2025.acl-long.529 (ACL 2025) | separate KV optimization for prefill vs decode; sliding-window heavy-hitter for decode; plug-in | adjacent KV compression
AdaSwitch | arXiv:2410.13181 | local agent handles less-complex steps, cloud handles complex; dynamic routing | closest published to ELAI cloud/local switch, but capability-routing NOT window-regime routing
OpenVINO prefix caching | docs.openvino.ai | --enable_prefix_caching + INT8 KV compression for local reusable prefixes | local-lane practice
Continuity Layer position paper | semanticscholar a6bb739a (arXiv:2604.10981/2604.06710 Apr 2026) | field has not built infra to carry understanding across sessions; 7 required characteristics incl write-time decomposition + read-time reconstruction | philosophical alignment w/ ELAI journal model (not competing impl)
"Contextual Agentic Memory is a Memo, Not True Memory" | semanticscholar 040e5f05 | lookup-based retrieval has provable generalization ceiling on compositionally novel tasks | criticism of recall-based continuity

ATTACK-THE-DESIGN (Part b, documented failure modes of ELAI):
- B1 Static slot floors -> observation-entropy mismatch: fixed floors underallocate when grep explodes (50 vs 50,000 tokens), overallocate when quiet; premature compaction under load (BACM-RL evidence).
- B2 Deterministic tail-drop -> semantic loss on non-temporal relevance: drops by recency not relevance; most diagnostic observations often older (early test failures, task spec); "lost-in-the-middle" blind spot grows with session length.
- B3 Ban on LLM re-summarization -> can't handle semantic-density spikes: deterministic trim reduces volume not density; hard ceiling when all content is decision-relevant (Claude Code auto-compact exists for exactly this).
- B4 Preference bi-temporal supersession -> preference dilution under context pressure: omission constraints ("never do Y") decay faster than commission ("always X"); attention favors recent tokens; hard constraints set early lose grip.
- B5 Code-grounded fact TTL without semantic-drift detection: re-validating file/symbol existence catches structural invalidation but not semantic drift (code present but now does different things via refactor/rename).
- B6 Two-cache-breakpoint -> cache invalidation on model/provider switches: byte-stable prefix is provider-specific; same bytes may not hit cache if model versions/tokenizers/cache-key-semantics differ between cloud and local Q4.
- B7 Handoff digest completeness -> "half-finished session" boundary: digest authored at session end, but overflow can trigger mid-task; crash-on-overflow may capture incomplete state (SWE-agent autosubmits partial patch).

TOP 3 CHANGES (Part d):
- Change 1 (HIGH): deferred observation loading w/ size pre-assessment — query token size before commit; if observed > slot_remaining x alpha trigger compaction before insert; 3-mode NONE/SELECTIVE/ALL (BACM). Compatible w/ <50ms (check not compression).
- Change 2 (HIGH): replace tail-drop w/ goal-conditioned structural pruning below cache boundary — pass (current_goal, code_block) to 0.6B SWE-Pruner skimmer; line selection not rewrite, doesn't touch prefix.
- Change 3 (MODERATE-HIGH): add write-time graph linking to code-grounded fact memory — A-MEM-style link generation + memory evolution; enables multi-hop queries.

NUMBERS (figure | claimed source):
- Manus: 100:1 input/output ratio; >80% KV-cache hit SLO; cached $0.30/MTok vs uncached $3.00/MTok (10x).
- SWE-Pruner: 23-54% token reduction SWE-Bench Verified, up to 14.84x LongCodeQA, ~40% avg (Reddit).
- CAT/SWE-Compressor: 57.6% solve SWE-Bench-Verified. ACON: 26-54% peak token reduction. Context-Folding: active ctx 10x smaller.
- JetBrains: LLM summarization +15% turns (Gemini 2.5 Flash); best masking window ~10 turns.
- Memory retention (10-session sim, practitioner bench NOT peer-reviewed): Letta 95/90/85% (sess 2/5/10); Mem0 92/80/65%; A-MEM 88/85/80%; no-memory 0/0/0%. p95 overhead: Letta 180ms, Mem0 80ms, A-MEM 120ms.
- A-MEM: LoCoMo 27.23 ROUGE-L vs 4.68 baseline (~5.8x); temporal 45.85 F1 vs MemGPT 25.52.
- Mem0: 92.5 LoCoMo, 94.4 LongMemEval @ ~6900 tok; +29.6pp temporal, +23.1 multi-hop.
- BACM: 1.6x gains high-complexity; stable 16K->4K; 30B/8K beats 235B/128K; light-load compresses 42% less, heavy-load 109% more scoring 143% higher.
- KV at 32K: 4-6GB extra VRAM on 8B model. Claude Code: 200K (older), 1M (Claude 4.6).
- Context quality degrades after ~32K (Databricks Mosaic). ~65% of 2025 enterprise AI failures attributed to context drift/memory loss (secondary source, unverified).

UNVERIFIED/FLAGGED by report: BACM (no code, preprint, not peer-reviewed); Context-Folding FoldGRPO (internal bench, needs RL fine-tune); Letta/Mem0/A-MEM retention figures (practitioner sim, methodology not described); "65% enterprise failures" (secondary source); NeuSymMS (May 2026 preprint, not peer-reviewed); Manus KV economics (blog post, not peer-reviewed).
