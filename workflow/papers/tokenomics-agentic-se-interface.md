Algorithm: Tokenomics (empirical token-consumption analysis of LLM multi-agent SE systems)
ArXiv: 2601.14470 (v1)
Paper: Tokenomics: Quantifying Where Tokens Are Used in Agentic Software Engineering
Authors: Salim, Latendresse, Khatoonabadi, Shihab (DAS Lab, Concordia University)
Venue: MSR 2026 (Mining Software Repositories), work-in-progress / preliminary study
Type: EMPIRICAL MEASUREMENT STUDY (not an algorithm). No code algorithm to implement; the
  "interface" here is the measurement methodology + the grounded findings/numbers.
Reliability: MEDIUM. Peer-track WIP, single framework (ChatDev), single model (GPT-5
  reasoning), n=30 tasks, several stages on small subsamples (Code Completion n=6, Testing
  n=12). Authors themselves flag generalizability limits. Replication pkg: zenodo 17430187.

What it measures (study inputs):
  System under study | ChatDev LLM-MA framework (chat-chain waterfall: design->coding->testing)
  Backbone model     | gpt-5-2025-08-07, reasoning model, temp=1.0 (immutable), 400K ctx, 128K max-out, cutoff 2024-09-30
  Tasks              | 30 prompts from the ProgramDev dataset (MAST study), Fibonacci..chess-game range
  Task-complexity proxy | reasoning-tokens per task, observed 17,280..40,000 (cites Shojaee "Illusion of Thinking")
  Instrumentation    | full execution trace per task: every LLM call's prompt, response, input/output/reasoning token counts

Outputs (what the study produces):
  Phase-mapping table | maps ChatDev internal phases -> 6 universal SDLC stages (the methodological contribution / "Rosetta Stone")
  Token-distribution  | % of total tokens per stage (avg over 30) + input/output/reasoning ratio per stage

ChatDev -> SDLC stage mapping (Table 2, the reusable abstraction):
  Design          <- DemandAnalysis, LanguageChoose
  Coding          <- Coding
  Code Completion <- CodeComplete
  Code Review     <- CodeReview  (iterative programmer<->reviewer dialogue)
  Testing         <- Test
  Documentation   <- EnvironmentDoc, Reflection, Manual

Key grounded findings (cite these, do NOT round/invent):
  F1 Code Review DOMINATES total token consumption: avg 59.4% of all tokens across 30 tasks.
     Code Completion 26.8% (only in the 6 tasks where it ran), Documentation 20.1%, Testing 10.3%
     (n=12), Coding 8.6%, Design 2.4%. (these are per-stage shares averaged over tasks where the
     stage occurred, so they do not sum to 100%.) Cost is in refinement/verification, NOT initial codegen.
  F2 Input tokens dominate overall: avg per-task split = 53.9% input / 24.4% output / 21.6% reasoning.
     ~2:1 input:output ratio = empirical evidence for the "communication tax" (AgentTaxo, Wang 2025);
     full context repeatedly passed in collaborative dialogue.
  F3 Per-stage tokenomic profiles differ (Table 3, avg input/output/reasoning %):
     Design          60.4 / 3.6  / 36.0   (input+reasoning heavy, tiny output)
     Coding           6.9 / 58.0 / 35.1   (OUTPUT-heavy outlier: verbose code from concise spec)
     Code Completion 47.7 / 41.7 / 10.5
     Code Review     51.4 / 24.7 / 23.9   (input-heavy verification)
     Testing         60.8 / 20.7 / 18.4
     Documentation   80.2 / 8.3  / 11.5   (most input-heavy: consumes code, emits little)
     Overall/task    53.9 / 24.4 / 21.6

Critical invariants (testable assertions over a parsed trace, Python syntax):
  assert split == ('input','output','reasoning') and abs(sum(per_task_split.values()) - 1.0) < 1e-6  # the 3 token classes partition each task's tokens
  assert per_task_split['input'] > per_task_split['output'] and per_task_split['input'] > per_task_split['reasoning']  # F2: input dominates
  assert coding_stage['output'] > coding_stage['input']  # F3: Coding is the only output-heavy stage
  assert documentation_stage['input'] > 0.75  # F3: Documentation ~80% input
  assert stage_total_share['Code Review'] == max(stage_total_share.values())  # F1: Code Review is the largest total consumer

Complexity / scale notes:
  Analysis pipeline = 3 deterministic steps: Trace Collection -> Phase Mapping -> Token Aggregation
  (Python scripts parse traces, bucket token counts per mapped stage). O(total_LLM_calls) parse.
  No model training; cost is the 30 ChatDev GPT-5 runs themselves.

Failure modes / threats the authors warn about:
  - Single system (ChatDev) + single model (GPT-5 reasoning): patterns may not transfer to MetaGPT-style
    SOP/assembly-line frameworks or to non-reasoning / differently-priced models.
  - n=30 small; Code Completion (n=6) and Testing (n=12) stage profiles are small-sample, low generalizability.
  - The phase->SDLC mapping is ONE of several valid abstractions, not canonical.
  - Stages are agent-chosen per task, so "n" per phase varies; per-stage shares are conditional averages.

Implementation pitfalls (do NOT do this when applying to ELAI):
  - Do NOT treat 59.4% / 53.9% as universal constants. They are ChatDev+GPT-5-specific, single small study.
  - Do NOT assume the input-heavy "communication tax" is fixed; it is hypothesized to be a property of
    conversational full-context-passing protocols, i.e. exactly the thing ELAI's caching / wire-dark /
    sub-agent-context-GC strategy aims to cut. Treat as a HYPOTHESIS-to-beat, not a target to hit.
  - Do NOT conflate "reasoning tokens" with output; the paper tracks 3 distinct classes.
  - Do NOT cite this as proof Code Review is expensive in ELAI's auditor flow; ELAI's auditor runs in a
    fresh GC'd context (not full-history re-passing), which is precisely the inefficiency this paper flags.

ELAI relevance (why this was ingested):
  Directly grounds the efficiency-direction memos (project_efficiency_trust_engineering,
  project_efficiency_deepdive_2026_06_03) and the H-plan Track-1 efficiency + Track-9 context-lifecycle
  work: empirical external evidence that (a) input/context-passing is the dominant token cost in
  conversational multi-agent SE, and (b) verification/refinement loops, not codegen, are where tokens go.
  Supports the "sub-agents save the main LLM's context" doctrine and the cloud prompt-caching push.
