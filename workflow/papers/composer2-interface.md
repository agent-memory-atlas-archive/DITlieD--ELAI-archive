Paper: Composer 2 Technical Report
ArXiv: 2603.24477 (v1 2026-03-25, v2 2026-03-26)
Authors: Cursor Research team led by Aaron Chan + 53 co-authors
Source URL: https://arxiv.org/abs/2603.24477
Source URL (HTML): https://arxiv.org/html/2603.24477v2
Verification date: 2026-05-24
Verification method: WebFetch on arxiv.org/abs/2603.24477 + 2 WebSearch cross-checks
              (huggingface.co/papers/2603.24477 + cometapi.com/cursor-2-0
              article both confirm same title/team/date).

CITE STATUS: VALID (paper exists, title/authors/date match the C-44 claim).
GROUNDING FIT FOR C-44: PARTIAL — see "Grounding fit analysis" below.

Topic of the paper:
  This is a model technical report (training methodology + evaluation
  + benchmark introduction). The paper describes how Composer 2 was
  built (two-phase training: continued pretraining + large-scale RL
  on a Cursor-environment-matched harness), introduces CursorBench
  (a benchmark from real codebase problems), and reports headline
  numbers: 61.3 accuracy improvement vs prior Composer, 61.7 on
  Terminal-Bench, 73.7 on SWE-bench Multilingual.

  Categories on arXiv: cs.SE (Software Engineering) + cs.LG (Machine
  Learning). The paper is about domain-specialized model training,
  NOT about multi-backend runtime architecture, tiered LLM
  orchestration, cost accrual, or fallback policies.

Grounding fit analysis (relative to C-44 plan claims):

  C-44 claims paper grounding (arXiv:2603.24477) for an integration
  task: "add Composer 2 as a third runtime backend tier alongside
  local Qwen and cloud Claude." The plan needs grounding for:
    a) backend taxonomy (which tier, what role)
    b) tier model (cost / latency / context envelope)
    c) cost accrual policy (per-token pricing, rate limits)
    d) fallback policy (when to fall back to local Qwen / cloud Claude)
    e) public API surface (endpoint, auth, model id, token-count
       semantics)

  The paper grounds ONLY:
    - "what is the Composer 2 model" (a coding model, trained by
      Cursor Research, evaluated on CursorBench / Terminal-Bench /
      SWE-bench Multilingual)
    - the model's positioning ("frontier-level coding capabilities"
      for agentic software engineering)

  The paper does NOT ground:
    - public API endpoint, auth model, or rate limits
    - per-token pricing or cost accrual semantics
    - token counting / tokenizer details usable at integration time
    - fallback / retry / streaming protocol
    - any "tier" framing — the paper is about ONE model in isolation

  Conclusion: the paper grounds the existence and identity of
  Composer 2 the model. It does NOT ground any integration detail
  C-44 needs. The actual ground-truth for C-44 is the Cursor SDK
  + cursor.com/docs (operational source-grounding, not paper
  grounding).

Public API existence status (C-44 deferred-infeasibility check):

  PUBLIC API EXISTS — Composer 2 is reachable as a service.
    - SDK: @cursor/sdk (TypeScript), public beta available to all
      Cursor users
    - Install: npm install @cursor/sdk
    - Auth: env var CURSOR_API_KEY (NOT "COMPOSER2_API_KEY" as the
      C-44 plan currently specifies — see "Plan corrections needed"
      below)
    - Model identifier: "composer-2" (NOT "cursor-composer-2" as
      the C-44 plan currently specifies)
    - Docs: cursor.com/docs/api + cursor.com/docs/models/cursor-
      composer-2 + cursor.com/docs/models/cursor-composer-2-5
    - A successor model "Composer 2.5" also exists (cursor.com/docs/
      models/cursor-composer-2-5) — C-44 should decide whether to
      ship Composer 2 or 2.5 or both.
    - Verified via 2 WebSearch cross-checks 2026-05-24.

  C-44 does NOT close as deferred-infeasibility. The integration
  is viable.

Recommendation:

  1. KEEP the C-44 plan but REWORK its grounding posture. The paper
     citation (arXiv:2603.24477) is a VALID cite for the sentence
     "Composer 2 exists and is a Cursor Research coding model" only.
     Remove any claim that the paper grounds the integration design
     (backend taxonomy, cost accrual, fallback policy, public API
     surface).

  2. ADD source-grounding (mode=source) for the actual integration
     ground truth:
       - paper-ingestor mode=source target=@cursor/sdk +
         cursor.com/docs/api + cursor.com/docs/models/cursor-
         composer-2
       - Produces workflow/papers/cursor-sdk-interface.md with
         (a) endpoint + auth, (b) model ids, (c) request/response
         schema, (d) cost / rate-limit table, (e) error / retry
         protocol. Reliability level: HIGH (official SDK).

  3. PLAN CORRECTIONS NEEDED in C-44_composer2_runtime_backend_tier.md:
       - line 55: api_key_env = "COMPOSER2_API_KEY"
                  -> api_key_env = "CURSOR_API_KEY"
       - line 56: model_id = "cursor-composer-2"
                  -> model_id = "composer-2"
       - line 39: "cursor.com/blog/composer-2 + cursor.com/docs"
                  -> add cursor.com/docs/models/cursor-composer-2
                  + cursor.com/docs/api specifically
       - line 15-25: reword the "Paper grounding" sentence to
         scope the cite to model-identity grounding only, and
         add a separate "Source grounding" sentence pointing at
         workflow/papers/cursor-sdk-interface.md (to be authored
         by paper-ingestor mode=source)

Cluster context (cite-quality cluster, per workflow/sessions/
s-25120/mistakes.md):

  This is the FOURTH grounding incident in the recent plan cluster:
    1. B-40: bad cite (arXiv:2407.21787 substituted with 2409.04109)
    2. B-42: bad cite (arXiv:2305.18290 substituted)
    3. C-43: bad cite (arXiv:2604.21725 — paper exists but topic
       totally wrong; cite removed; mechanism reframed as ELAI
       design hypothesis)
    4. C-44 (this report): paper VALID, topic SHALLOW MATCH. Not a
       hallucinated cite, but a misuse-of-cite. The paper is about
       Composer 2 the model, NOT about how to integrate Composer 2
       as a backend tier. Cite is reusable for narrow model-identity
       grounding only; integration design needs source-grounding.

  This is a DIFFERENT failure mode from incidents 1-3. Cite is real
  AND topically adjacent, but does not actually answer the plan's
  grounding need. Pattern name: "shallow-cite-fit" or "cite-of-
  proximity-not-of-utility".

Inputs (to the model itself, per paper):
  Not applicable — paper does not specify a public input schema.
  Operational inputs come from @cursor/sdk, not from the paper.

Outputs (of the model itself, per paper):
  Headline benchmarks (paper figures, NOT runtime contracts):
    - CursorBench: 61.3 accuracy improvement vs prior Composer
    - Terminal-Bench: 61.7
    - SWE-bench Multilingual: 73.7
  These are evaluation numbers, NOT integration-time invariants.

Hyperparameters / tunables:
  Not applicable at integration time — Cursor exposes a fixed
  model id, not training hyperparameters. The SDK exposes standard
  inference knobs (temperature, max tokens, streaming) per the
  Cursor docs — to be filled by source-grounding.

Critical invariants (testable assertions):
  None derivable from the paper. The paper is a training report;
  it does not specify integration-time invariants the runtime
  backend must uphold. Integration-time invariants must come from
  cursor.com/docs/api (source-grounding, separate document).

Complexity:
  Not applicable. Training-side compute is not relevant to a
  client-side runtime backend.

Failure modes cited in the paper:
  The paper discusses evaluation-time failure modes (long-horizon
  task failure, multi-step execution errors) but these are
  model-quality failure modes, NOT integration failure modes.
  Integration failure modes (timeout, rate-limit, 5xx, auth fail)
  must come from cursor.com/docs/api.

Implementation pitfalls (do NOT do this) — derived from cite-fit
analysis, not from the paper itself:
  - Do NOT cite arXiv:2603.24477 as grounding for the runtime
    backend integration design. The paper does not cover that.
  - Do NOT assume the API key env var is COMPOSER2_API_KEY — the
    SDK uses CURSOR_API_KEY.
  - Do NOT assume the model id is "cursor-composer-2" — the SDK
    uses "composer-2".
  - Do NOT skip the source-grounding step (cursor.com/docs/api +
    @cursor/sdk) — the paper alone is insufficient grounding.
  - Do NOT proceed with C-44 implementation until the source-
    grounding interface map (workflow/papers/cursor-sdk-
    interface.md) is authored.

RAG ingest status: NOT INGESTED.
  Rationale: ingesting a partial-fit paper into RAG risks
  cross-contamination (future queries for "composer 2 backend
  integration" would return training-methodology content). The
  paper's headline numbers + identity are documented above; that
  is sufficient for plan-author reference without polluting the
  retrieval namespace. If future work needs deep grounding on
  Cursor's training methodology specifically, ingest then under
  topic "paper-composer2-training-report".
