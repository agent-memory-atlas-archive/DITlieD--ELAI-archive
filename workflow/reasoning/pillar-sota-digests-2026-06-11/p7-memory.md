DIGEST: p7-memory
SOURCE: 074c26b9-0157-4dc8-9787-ff3e1844b7ab.txt

TOPIC: Pillar P7 (cross-session agent memory for a local-first coding agent). Audits ELAI's design (Tier A = code-cited facts revalidated before injection; Tier B = bi-temporal supersession w/ hard filter of superseded preference rows; zero-memory auditor default; staleness instrumentation; TTL forgetting) vs 2026 memory SOTA. Verdict frame: HARDEN not replace.

REPORT VERDICTS:
- Overall: HARDEN. Keep core shape — bi-temporal supersession + hard retrieval filter, zero-memory auditor default, staleness measurement. ELAI is closer to the strongest 2026 research direction (provenance-grounded memory: TierMem/MemIR/Eywa) than to the Mem0/Zep/Letta product lineage.
- Do NOT swap for a generic Mem0/Zep/Letta layer and assume pillar solved — they remain vulnerable to provenance collapse, memory poisoning, freshness/cost tradeoffs; several mature ones are unattractive on the local-first cost frontier.
- KEEP unchanged: bi-temporal supersession + hard retrieval filter (matches Zep valid/transaction time + 2026 explicit-version-control work); zero-memory auditor default (indirect evidence: cold-start evaluators safer than memoryful by default) — but refine to fresh-thread + no-recall + typed-evidence-packet-only.
- DEMOTE TTL from "primary forgetting policy" to "maximum age guardrail"; add activation/heat score so revalidated/used facts live longer.
- Top 3 changes (by evidence strength): (1st) add provenance-linked raw-evidence tier w/ sufficiency-gated escalation; (2nd) replace write-time-only safety w/ typed memory representation + retrieval-time authorization; (3rd) upgrade code validation from file/symbol checks to commit-aware semantic validation.

CANDIDATES (name | url-or-id | report's claim | replaces-or-hardens):
Zep / Graphiti | Apache-2.0, repo confirmed | strongest mature temporal-KG; closest product match to ELAI bi-temporal instincts; 94.8% vs 93.4% over MemGPT on DMR, up to 18.5% LongMemEval gain w/ 90% lower latency (vendor); independent shows strongest mainly on open-domain | validates ELAI bi-temporal supersession KEEP
Mem0 | repo confirmed (license unverified in sources) | fast read path, simple deploy, good ergonomics; update logic largely LLM-mediated, weaker on freshness/conflict; independent: 32.0% LongMemEval @ 2.2s latency @ 4.02h construction (strong speed/quality tradeoff not SOTA) | memory baseline (not a replacement)
Letta / Letta Code | Apache-2.0, repos active June 2026 | highest product maturity for stateful agents + memory-first coding harness; independent: 27.7% LongMemEval @ 14.36h construction, highest energy = poor local-first cost point | rejected as local-first swap
SimpleMem | MIT, active OSS research + MCP/multimodal | best current OSS for compression + adaptive retrieval w/o heavy graph stacks; 26.4% avg F1 improvement, up to 30x lower inference-time tokens; among strongest LoCoMo baselines, high query latency in systems studies | candidate compression/retrieval
MemMachine | Apache-2.0, mid/high maturity | strongest ground-truth-preserving episodic memory productized alt to Mem0-style extracted-fact stores; 0.9169 LoCoMo, 93.0% LongMemEvalS, ~80% fewer input tokens vs Mem0 (VENDOR, not independently replicated) | episodic-memory alt
TierMem | no-url (research-only, code unverified) | write-time abstraction alone creates write-before-query barrier; 2-tier defaults to cheap summary, escalates to immutable raw logs only when needed | ARCHITECTURAL PATTERN to steal: provenance-linked raw-evidence tier (Change 1)
MemIR | no-url (research-only) | memory must cleanly separate evidence / retrieval cues / truth-bearing claims; retrieval only authorizes still-supported claims | typed claim/evidence separation (Change 2)
Eywa | no-url (research-only) | immutable evidence first, cheap summary second, inference-time evidence sufficiency, deterministic retrieval w/ minimal/zero LLM on read path | provenance-first pattern (Changes 1+2)
Independent 2026 systems characterization (Stanford/MIT-adjacent) | no-url | LongMemEval favors simpler paradigms: BM25 47.0%, GraphRAG 46.0%, HippoRAG v2 44.3%, A-Mem 42.7%, Mem0 32.0%, Letta 27.7%, MIRIX 20.0%; CONSTRUCTION (not query) dominates energy+wall-time; LLM-bounded systems have widest worst-case latency tails | argues KEEP deterministic bounded retrieval, background-only mutation loop
2026 conflict-resolution paper | no-url | Mem0/Graphiti/Zep still struggle keeping up-to-date store when facts change; deterministic recipe w/ explicit version control + answer-time assembly: 97.7% per-fact conflict-tracking, 45.8% whole-store freshness, 98.1% retrieval precision | directly validates ELAI bi-temporal supersession + hard-filter > "narrate outdated memories"
Prometheus | no-url | turns repo into unified KG + working memory reusing explored contexts; 74.4% SWE-bench Verified, 33.8% SWE-PolyBench Verified, strong multilingual | code-structure memory > flat file retrieval; version/structure-aware
RIG / SPADE (Repository Intelligence Graph) | no-url | deterministic evidence-backed architectural map from build/test artifacts; +12.2% mean QA accuracy, -53.9% completion time, -57.8% sec/correct-answer over 8 repos; larger gains in multilingual | CLOSEST analogue to ELAI "facts cite code + revalidate"; stronger because edges trace to concrete build/test defs (Change 3)
Codebase-Memory | no-url | Tree-sitter + incremental sync + MCP; live code graph across 66 languages; deterministic indexing first-class | incremental structural substrate for Tier A validation
MPBench | no-url | most comprehensive 2026 memory-poisoning study; multiple write channels + structural vulns; poisoning persistent at avg 50.46% ASR / 41.05% response success across 2 agent systems; prompt-injection defenses do NOT transfer to memory poisoning (benign-at-write, harmful-after-retrieval) | attack evidence: write-time scanning insufficient (Change 2)
MemPoison | no-url | attacks SELECTIVE memory systems (not toy direct-write); ASR up to 0.95 via trigger/payload pairs surviving extraction+rewriting | attack evidence
State Contamination in Memory-Augmented LLM Agents | no-url | harmful influence "laundered" through summarization; cleaning BEFORE summarization reduces hidden propagation, cleaning only completed summary leaves influence intact | argues sanitize-before-abstraction (Change 2)
SSGM governance framework | no-url | read-path identity constraints + immutable episodic logs to prevent semantic drift + persona cross-contamination | layered-defense pattern
AMEL | no-url | prior conversation history biases later judgments when multiple judgments in one conversation | indirect support for zero-memory auditor
MemoryOS | no-url | FIFO short-to-mid transfer + heat score (frequency+recency) promoting into long-term; large LoCoMo gains but doesn't isolate forgetting as causal | TTL+heat forgetting upgrade
Oblivion | no-url | forgetting as decay-driven accessibility not hard deletion; on LongMemEval noisy split w/ GPT-4.1-mini: 90.60% vs 89.00% direct baseline vs 81.27% EverMemOS | strongest adaptive-forgetting evidence; TTL+activation-decay
HippoRAG v2 | no-url | 44.3% LongMemEval (independent) | retrieval baseline
MIRIX | no-url | 20.0% LongMemEval (independent) | memory baseline
A-Mem | no-url | 42.7% LongMemEval (independent) | memory baseline

ATTACK-THE-DESIGN (report's adversarial failure-mode table for ELAI):
- Citation-validity != semantic-validity: revalidating file/symbol existence doesn't prove the stored claim is still entailed; refactors/behavior-changes/changed-tests preserve paths while invalidating meaning. Fix: commit hashes + AST identities + build/test artifact edges + re-derive before injection (RIG/SPADE).
- Write-before-query omission: Tier A extraction can drop the one detail a later prompt needs even if every kept fact is valid. Fix: immutable raw-evidence tier, escalate only when fact layer insufficient (TierMem).
- Write-time-only poisoning defense misses semantic poison: MPBench/MemPoison show semantically-natural poison survives selective extraction + later biases behavior. Fix: retrieval-time provenance gating, typed claim/evidence separation, quarantine new writes.
- Sanitizing after summarization too late: harmful influence compressed into safe-seeming summary persists. Fix: safety checks BEFORE abstraction/compaction + raw immutable log.
- Fixed TTL forgets rare-but-important invariants: 28-day expiry can drop low-frequency high-value facts (project conventions, dormant architectural constraints). Fix: TTL hard-cap + usage/confidence activation.
- Zero-memory auditors can still be contaminated OR starved: if reviewer shares prior conversation/tool-traces/persona, "zero memory injection" doesn't guarantee objectivity; cutting all context can starve task-local evidence. Fix: stricter firewall = fresh thread + no recall + typed evidence packet only.
- Append-only superseded rows still create growth + attack surface: store grows monotonically by default; needs explicit compaction/namespaces/ABAC/scoped-forensic-query.
- Branch/version drift in code memory: "live revalidation over HEAD" can wrongly reject a fact valid for the task's target branch, or accept a fact changed between sessions. Fix: scope code memories by repo+branch+commit range, revalidate against relevant checkout not only HEAD.

NUMBERS (figure | claimed source):
- Zep: 94.8% vs 93.4% over MemGPT on DMR; up to 18.5% LongMemEval gain w/ 90% lower latency (vendor).
- Independent LongMemEval: BM25 47.0%, GraphRAG 46.0%, HippoRAG v2 44.3%, A-Mem 42.7%, Mem0 32.0%, Letta 27.7%, MIRIX 20.0%.
- Mem0 independent: 32.0% @ 2.2s query latency @ 4.02h construction. Letta independent: 27.7% @ 14.36h construction (highest energy).
- 2026 conflict-resolution: 97.7% per-fact conflict-tracking, 45.8% whole-store freshness, 98.1% retrieval precision.
- SimpleMem: 26.4% avg F1 improvement, up to 30x lower inference-time tokens.
- MemMachine (vendor): 0.9169 LoCoMo, 93.0% LongMemEvalS, ~80% fewer input tokens vs Mem0.
- Prometheus: 74.4% SWE-bench Verified, 33.8% SWE-PolyBench Verified.
- RIG: +12.2% mean QA accuracy, -53.9% completion time, -57.8% sec/correct-answer (8 repos).
- Codebase-Memory: 66 languages.
- MPBench: 50.46% avg ASR, 41.05% response success (2 agent systems). MemPoison: ASR up to 0.95.
- Oblivion (LongMemEval noisy, GPT-4.1-mini): 90.60% vs 89.00% direct vs 81.27% EverMemOS.

UNVERIFIED/FLAGGED by report: no apples-to-apples coding-agent-specific comparison of fixed-TTL vs usage-decay vs learned-forgetting under same repo tasks/model; no direct published ablation of memoryful vs zero-memory reviewer on code-review objectivity (support is INDIRECT); strongest-looking numbers (Mem0/Zep/MemMachine) are vendor-authored not independently replicated; TierMem/MemIR/Eywa code + production maturity NOT verified (treat as patterns not platforms); Mem0 license not directly verified in gathered sources.
