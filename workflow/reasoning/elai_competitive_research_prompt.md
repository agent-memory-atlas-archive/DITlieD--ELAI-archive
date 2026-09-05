ELAI COMPETITIVE + RECENT-RESEARCH GROUNDING PROMPT
=====================================================
Author: maintainer@example.invalid (ELAI maintainer)
Generated: 2026-05-17
Purpose: hand to a research-capable agent (Claude Code, ChatGPT Deep Research, Perplexity Pro, or human analyst). Self-contained — receiver does not need prior context.

Copy everything between the two BEGIN/END markers.

================== BEGIN PROMPT ==================

You are a senior research analyst. Your task: produce a grounded comparative + recent-research report on the ELAI development engine, with ZERO speculation.

## What ELAI is (one paragraph)

ELAI is a Rust-native local-first autonomous development engine. 16 crates. Runs Qwen2.5-Coder (3B/7B/32B) NF4+FA2 on consumer GPUs (12-48GB VRAM) with optional cloud Sonnet 4.6 escalation. Architecture: 7-layer knowledge engine (tree-sitter DKG → BM25 → dual ONNX vectors → PageRank → CatRAG → A-RAG → cross-encoder reranker), GRC reasoning with per-agent budgets + CSQ, FSM with 9 states + evidence gates, pipeline factory (Designer + FlashResearch → Builder + DAG repair → Executor through FSM), recursive decomposition (Idea→Concepts→Phases→Steps→BabySteps with 8-block specs), code completeness enforcement (FILE_PLAN + skeleton-first + MiniCritic + StructuralRetry), testing trust (mutation + metamorphic + wiring contracts + IntentPair), self-modification (Phase 4H: 3 layers with auditor-approval gates and MSI auto-revert), Cosmos artifact tier for non-code knowledge, 17-language support, MCP bridge, Tauri+Svelte dashboard.

## REQUIRED reading order (READ ALL, in order, before writing anything)

These are absolute Windows paths. If you cannot access them, request them or stop.

1. F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md
   — The architecture overview. 1037 lines. Read fully.

2. F:\.ELAI_workflow\docs\specs\guide\steps\phase-index-MASTER.txt
   — Phase-by-phase implementation index (pre-0 through 4M). Large (~37K tokens). Read with offsets if needed.

3. F:\.ELAI_workflow\docs\specs\guide\INDEX.txt
   — Design-doc index (18 design files summarized). Large (~37K tokens). Read with offsets if needed.

4. F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md
   — B-1 through B-42 punch list + status. Large (~31K tokens). Read with offsets if needed.

5. F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-SUMMARY_INDEX.md
   — C-1 through C-34 follow-up punch list with wave structure + B→C closure map.

6. F:\.ELAI_workflow\workflow\reasoning\elai_capability_honest_assessment.md
   — Maintainer's own honest self-assessment with [V]/[H]/[P]/[U] confidence labels. READ THIS LAST so it anchors your analysis without biasing your spec reads.

## Your deliverable

A single report with two parts. Each claim in your report must carry a grounding citation per the rules below. Claims without grounding will be discarded.

================================================================
PART A — COMPETITIVE: WHAT DOES X DO MEASURABLY BETTER THAN ELAI?
================================================================

For each named competitor below, identify ONE TO THREE capabilities they do MEASURABLY better than ELAI's "all-B-plan-and-C-plan-done" target state.

Competitors (treat this list as MINIMUM — add others if you find them dominating a dimension):
- Devin (Cognition Labs)
- Aider (paul-gauthier/aider)
- Sweep / SweepAI
- OpenInterpreter (KillianLucas/open-interpreter)
- Cursor Composer / Cursor Agent
- Cline (formerly Claude Dev)
- Roo Code
- Claude Code (Anthropic, anthropics/claude-code)
- GitHub Copilot Agent / Copilot Workspace
- Continue.dev
- Codeium / Windsurf Cascade
- Bolt.new / StackBlitz
- v0 (Vercel)
- Replit Agent
- Smol Developer
- AutoGPT / AutoGen (Microsoft)
- LangGraph / CrewAI agentic templates (only if a specific deployment uses them in prod)
- SWE-agent (Princeton)
- OpenHands (formerly OpenDevin)
- MetaGPT
- Tabby (TabbyML)
- Refact.ai

GROUNDING RULES for Part A (each "X beats ELAI at Y" claim must have AT LEAST ONE of):
- A specific benchmark number with version + date: e.g. "Aider claims 79.7% on aider-bench polyglot leaderboard run 2026-03-15, vs ELAI's untested-on-this-bench state."
- A specific reproducible test: e.g. "ran SWE-bench Verified subset 2026-Q1, Devin scored X%; ELAI has no published SWE-bench number."
- A specific public demo with link + timestamp: e.g. "Devin's Upwork-task demo (cognition-labs.com/blog/introducing-devin, 2026-Q1) shows autonomous PR through CI."
- A specific code path in open-source repo: e.g. "Aider repo paul-gauthier/aider/aider/repomap.py implements file-level PageRank; ELAI does symbol-level (DETAILED_SUMMARY_01 Phase 1D)."
- A specific paper-cited result that names the tool.

FORBIDDEN in Part A:
- "Devin is probably faster" — no measurement, discarded.
- "Cursor has better UX" — UX without rubric is opinion, discarded.
- "Aider is more mature" — maturity is a vibe, discarded.
- ANY claim sourced only from training data without web verification.

For EACH (competitor, capability) pair, output exactly:

```
COMPETITOR: <name>
CAPABILITY: <one sentence, specific>
EVIDENCE: <link or repo path + line + date>
ELAI EQUIVALENT (per files above): <which B-N/C-N/phase covers it, or "not addressed">
GAP CLASSIFICATION: [QUANTIFIED_WORSE | UNMEASURED | NOT_ADDRESSED]
SUGGESTED ELAI ACTION: <one sentence — what would close the gap, or "accept the gap because Z"; must reference a concrete file/B-N/C-N>
```

If no competitor genuinely beats ELAI on a given dimension, say so explicitly. Do not invent gaps to fill quota.

================================================================
PART B — RECENT PAPERS (LAST 2 MONTHS: 2026-03-17 TO 2026-05-17)
================================================================

Find papers published between 2026-03-17 and 2026-05-17 (inclusive) on arXiv, NeurIPS, ICML, ICLR, ACL, EMNLP, CHI, OSDI, SOSP, ASPLOS, PLDI, ICSE, FSE, MSR, or top-tier workshops, that would help ELAI.

Topic scope (search these areas):
- Code generation with small/local LLMs (3B-32B parameter class)
- Speculative / draft / parallel decoding for code
- RAG for code (improvements to BM25/vector/PageRank/CatRAG/A-RAG/reranker stacks)
- Cross-encoder rerankers ≤ 200M params with sub-50ms latency
- Constrained / grammar-based decoding (XGrammar, lm-format-enforcer, outlines, etc.)
- LLM-based AST refactoring / multi-file edits
- Mutation testing with LLMs (cargo-mutants, mutmut, PIT extensions)
- Metamorphic testing for ML/code
- SWE-bench and successor benchmarks (SWE-bench Verified, SWE-bench Lite, Multi-SWE-bench, etc.)
- Self-improvement / self-modification for LLM agents (DGM, DarwinNet, CEL, Self-Rewarding)
- Anti-hallucination / grounding for code generation
- Knowledge graphs over codebases (tree-sitter, LSP, SCIP, treegen)
- Long-context for code (RULER, NIAH-code, infini-attention for code)
- Agentic frameworks with evidence gates / formal verification
- Cost-efficient LLM serving (NF4, AWQ, TQ, GPTQ, SmoothQuant updates)
- Steering vectors / activation engineering
- DoLa, contrastive decoding, ETC, related logits processors
- Code-aware freshness / staleness detection
- Pipeline compilation / template caching for LLM workflows

GROUNDING RULES for Part B (each "Paper X would help ELAI" claim must have ALL of):
- arXiv ID OR DOI + verified publication date in window (cite the abs page).
- ONE OR MORE quoted empirical results from the paper (table number, figure number, or section). Example: "Table 3 shows +12.4% pass@1 on HumanEval-Mul vs baseline."
- A concrete mapping to ELAI: "Paper's technique X maps to ELAI's component Y (file/B-N/C-N) because the paper measures Z which addresses ELAI's gap W (cite the gap from DETAILED_SUMMARY or assessment file)."
- Reproducibility note: "Paper releases code at [repo], evaluated on [hardware], cost ~[$/h or hours]" OR "no code released, would need to reimplement."
- A risk note: "Paper's technique may NOT transfer to ELAI because [tested only on K=N, only English, only Python, requires capability ELAI doesn't have, etc.]."

FORBIDDEN in Part B:
- "This recent paper on Mixture-of-Experts could help" — vague, no result, discarded.
- "Authors claim improvement" — claims without numbers, discarded.
- Papers older than 2026-03-17 — out of window, exclude even if relevant.
- Papers behind paywalls without an arXiv preprint — cannot verify, exclude.
- ANY citation you have not actually retrieved (per memory feedback_paper_citation_verify.md: 2 bad citations were already caught in B-40/B-42 plans by exactly this kind of unverified citation pattern). VERIFY EACH arXiv ID by fetching the abstract page.

For EACH paper, output exactly:

```
PAPER: <title>
ARXIV: <ID> (or DOI for non-arXiv)
DATE: <YYYY-MM-DD> [must be within 2026-03-17..2026-05-17]
AUTHORS: <first author et al.>
QUOTED RESULT: "<exact quote from the paper>" (Table/Figure/Section reference)
ELAI COMPONENT TARGETED: <file path + B-N or C-N reference>
WHY IT HELPS ELAI: <one paragraph, max 5 sentences, naming the specific ELAI gap closed>
INTEGRATION COMPLEXITY: [TRIVIAL | MODERATE | RESEARCH-GRADE | INFEASIBLE-WITHOUT-32B]
REPRODUCIBILITY: <code link + hardware OR "no code">
RISK: <one sentence — why this might NOT transfer>
PROPOSED B-N OR C-N PATCH: <new B-43 / C-35 / etc. label and one-line scope>
```

================================================================
ANTI-SYCOPHANCY RULES (apply to entire report)
================================================================

- Do not write "ELAI's architecture is impressive." Do not praise. Do not validate maintainer's design choices unless the praise carries a citation.
- If a competitor or paper is BETTER than ELAI on a dimension, say so directly. Do not hedge with "ELAI takes a different approach which is also valid." If it's worse, it's worse — say so.
- If you cannot find evidence either way, say "UNKNOWN — needs benchmarking." Do not fill silence with optimism.
- "I think" / "this could help" / "this might be useful" → strip. Replace with measurement or remove.
- Quantify everything quantifiable. "Faster" without a number is removed.
- Hedge only when the source itself hedges: "Authors note the technique may not generalize to languages outside Python (Section 6.2)."

================================================================
OUTPUT FORMAT
================================================================

- Plain text or markdown, your call.
- Max 12,000 words. Concision over completeness. If you have 30 papers but only 8 with hard numbers, give me the 8.
- Top of report: 1-paragraph executive summary naming the SINGLE highest-leverage Part A gap and the SINGLE highest-leverage Part B paper.
- Mid: Part A grouped by competitor.
- Mid: Part B sorted by INTEGRATION COMPLEXITY ascending (TRIVIAL first).
- Bottom: PROPOSED FOLLOW-UPS — list of B-43+/C-35+ patch candidates synthesized from the above, ranked by leverage-per-effort.

================================================================
HARD STOPS
================================================================

- If you cannot read one of the 6 required files, stop and report which.
- If you cannot reach arXiv to verify a citation, stop and report.
- If your search returns ZERO papers in the 2026-03-17..2026-05-17 window, double-check the date filter; do not substitute older papers.
- If a competitor's website / repo is down, mark the (competitor, capability) pair UNVERIFIED rather than guessing from training data.

================== END PROMPT ==================
