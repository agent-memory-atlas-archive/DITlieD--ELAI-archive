GROUNDING SWEEP -- phase B-39
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-39_chat_reflect_pipeline.md
Date: 2026-05-10
FSM: PLANNING | Session: s-30584

SPEC REFERENCES
(B-39 has no docs/specs/guide/steps/phase-{X}.txt citations; all references are external papers, a blog, and a memory file)

arXiv:2407.01489 (Agentless, 2024) | VERIFY_NEEDED | Paper confirmed real (title: "Agentless: Demystifying LLM-based Software Engineering Agents"). Abstract describes three-phase localization/repair/patch-validation on SWE-bench. Zero mention of "context-isolation pattern", "per-stage clean sidecar POST", or "KV cache reset between stages". B-39 extrapolates an isolation principle not stated in the paper. Claim partially supportable (agentless = no shared agent state) but KV-reset framing is architectural interpretation, not a direct spec quote.

arXiv:2502.06703 (Liu et al., 2025) | VERIFY_NEEDED | Paper confirmed real (title: "Can 1B LLM Surpass 405B LLM? Rethinking Compute-Optimal Test-Time Scaling"). Abstract confirms 7B beats DeepSeek-R1 on MATH-500 via compute-optimal TTS. Does NOT state "N=4 is the inflection point for 3B-7B class models" in the abstract; the strategy is PRM-and-difficulty-dependent, not a flat best-of-N=4 recommendation. The specific N=4 inflection claim requires reading the paper body (Table/Figure level data).

arXiv:2408.15240 (Generative Verifiers, 2024) | GROUNDED | Paper confirmed real (title: "Generative Verifiers: Reward Modeling as Next-Token Prediction"). Scoring candidate reasoning traces via next-token prediction matches the auditor design claim. TACL-level evidence.

arXiv:2406.01297 (Kamoi et al., TACL 2024) | GROUNDED | Paper confirmed real (title: "When Can LLMs Actually Correct Their Own Mistakes?"). Abstract explicitly: "self-correction works well in tasks that can use reliable external feedback". Supports mandatory grounded evidence input to auditor. Author match: Kamoi et al.

arXiv:2511.22173 (RefineBench, 2025) | VERIFY_NEEDED | Paper confirmed real (title: "RefineBench: Evaluating Refinement Capability of Language Models via Checklists"). Abstract confirms refinement fails to consistently improve across iterations (self-refinement: +1.8% or -0.1%). Does NOT state "diminishing returns past iteration 2" specifically. Abstract says guided refinement works "within five turns". The cap=2 claim is not directly derivable from the abstract; needs paper body section on iteration counts.

Memory: project_qwen3b_baseline.md (C:\Users\user\.claude\projects\d---ELAI-workflow\memory\) | GROUNDED | File exists. Content: "In-VRAM context ceiling: ~24K ctx tokens". 20K production cap with 5K headroom is consistent with measured 25K ceiling. Evidence: project_qwen3b_baseline.md line 13.

AISLE blog (https://aisle.com/blog/ai-cybersecurity-after-mythos-the-jagged-frontier) | VERIFY_NEEDED | URL returns HTTP 200. Content is Next.js SPA (client-side rendered); article body not accessible via curl. Cannot confirm "orchestration is the security moat" or "each stage must be isolated to prevent prompt injection" claims. Accessible but unverifiable without browser rendering.

CLAIM CLASSIFICATION

Agentless per-stage clean sidecar POST, no shared KV | INGEST | Abstract only; KV-reset isolation not stated -- ingest arxiv:2407.01489 to find section on context isolation

N=4 inflection point for 3B-7B best-of-N sampling | INGEST | Abstract confirms 7B>DeepSeek-R1 via TTS but N=4 specific inflection not in abstract -- ingest arxiv:2502.06703 to find N curve data

Auditor scores traces not re-solves (GenVerifiers pattern) | GROUNDED | arxiv:2408.15240 confirmed: reward modeling as next-token prediction on candidates. Consistent with scoring framing.

External feedback mandatory for effective self-correction | GROUNDED | arxiv:2406.01297 abstract line: "self-correction works well in tasks that can use reliable external feedback"

REFINE loop diminishing returns past iteration 2 | INGEST | RefineBench (2511.22173) shows iteration stagnation but "past iteration 2" specific threshold not in abstract -- ingest arxiv:2511.22173 to verify cap=2 data point

25K token ceiling / 20K production budget cap | GROUNDED | project_qwen3b_baseline.md:13 -- "~24K ctx tokens" measured; 20K cap with 5K headroom matches

AISLE blog: orchestration is security moat, per-stage isolation | UNGROUNDED | Blog URL 200 but SPA body unverifiable; security/isolation framing cannot be confirmed from available content

SCHEMA_VERSION currently 28, B-39 bumps to 29 | GROUNDED | ssb.rs:44 -- "const SCHEMA_VERSION: i64 = 28;" confirmed. B-39 increment to 29 is correct.

agent_grc_config_for_role exists with "media_validator" pattern | GROUNDED | config.rs:336-377 read directly -- function exists, "media_validator" arm at line 352-373, pattern matches B-39 Task 1 description exactly.

GroundingLevel enum: Uncertain/High/Verified only (no Medium) | GROUNDED | types.rs:126-133 read -- enum has Uncertain, High, Verified. B-39 note ("Medium does not exist") confirmed correct.

LiveCli::run_turn at live_cli.rs:1659 is chat dispatch entry | GROUNDED | live_cli.rs:1659 -- "pub(crate) fn run_turn(&mut self, input: &str)" confirmed. Insert point claim verified.

ConversationRuntime is field type, no send_message method | GROUNDED | live_cli.rs:141 -- "pub(crate) runtime: Option<ConversationRuntime<...>>" confirmed as field; no send_message in grep results.

slash_command_specs() count currently 146, target 147 | GROUNDED | commands.rs:4279 -- "assert_eq!(slash_command_specs().len(), 146)" confirmed. +1 for "reason" spec = 147 target is correct.

PipelineGrc and BasicGrcEngine exist as described | GROUNDED | grc.rs:3,17 -- "wraps elai_reasoning::BasicGrcEngine", struct PipelineGrc confirmed. reasoning/src/engine.rs confirmed present via glob.

ACTIONS REQUIRED
- ingest paper "Agentless" via paper-ingestor mode=paper with arxiv id 2407.01489 -- verify context-isolation / per-stage KV reset claim is supported in paper body (not just inferred from abstract)
- ingest paper "Compute-Optimal TTS" via paper-ingestor mode=paper with arxiv id 2502.06703 -- verify N=4 inflection point for 3B-7B class models in figures/tables
- ingest paper "RefineBench" via paper-ingestor mode=paper with arxiv id 2511.22173 -- verify diminishing returns past iteration 2 specific data point (cap=2 design decision)
- UNGROUNDED claim "AISLE blog: orchestration is security moat, per-stage isolation prevents prompt injection" -- blog body unverifiable via curl (SPA). Either fetch with browser tool or remove blog citation and rely on 2407.01489 alone for isolation justification.

VALIDATION WARNINGS
no docs/specs/guide/steps/phase-{X}.txt citations in source -- B-39 is a B-plan capability file, not tied to a phase spec; all citations are external papers and memory. This is expected per task input note.
RefineBench paper dated 2025-11-27 (future relative to assistant training cutoff Aug 2025) -- paper is real and confirmed on arxiv.org; treat as INGEST until body is verified.
