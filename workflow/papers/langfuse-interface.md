Source: Langfuse
Reliability: HIGH (official README + official docs at langfuse.com/docs, open source repo reviewed)
Files examined:
  https://github.com/langfuse/langfuse (README)
  https://langfuse.com/docs/tracing-data-model
  https://langfuse.com/docs/opentelemetry/get-started
License: MIT (except ee/ folders which are Enterprise Edition / proprietary)
Stars: not captured (active project, actively maintained as of 2026)

WHAT IT IS

Langfuse is an open source LLM engineering platform for observability, tracing, evaluation,
and debugging of AI applications. It is an EXTERNAL remote trace store, not an in-process
journal: applications push telemetry to Langfuse; it is not embedded in the process.

"Proudly made with ClickHouse open source database" (ClickHouse is the storage backend).
Also uses Postgres for metadata / relational data (implied by self-host setup).

DATA MODEL (verified from docs)

Hierarchy:
  Session (optional grouping of related traces)
    Trace (one request or operation)
      Observation(s) (individual steps, nestable as a tree)
        Types: span, generation, event, toolcall, RAG retrieval step

Observation types:
  generation: an LLM call with input/output tokens, model, cost
  span: a timed block of work (non-LLM step)
  event: a point-in-time marker (no duration)
  toolcall: tool/function invocation
  RAG retrieval: document retrieval step

Trace attributes (propagated to all child observations):
  userId, sessionId, metadata (key-value), tags (string[]), release, version, environment
  environment values: production / staging / development

Scores: evaluations attached to traces or observations (e.g., human feedback, LLM-as-judge,
  automated evals). Separate from the trace tree.

Async ingestion model: observations are batched locally and sent in the background.
Short-lived processes MUST call flush() before exit or data is lost.

OTLP / OPENTELEMETRY SUPPORT (verified from docs)

Langfuse accepts OTLP traces directly. It operates as an OpenTelemetry backend.

OTLP endpoint:
  EU cloud:   https://cloud.langfuse.com/api/public/otel
  US cloud:   https://us.cloud.langfuse.com/api/public/otel
  Japan:      https://jp.cloud.langfuse.com/api/public/otel
  HIPAA:      https://hipaa.cloud.langfuse.com/api/public/otel
  Signal-specific: /api/public/otel/v1/traces

Transport: OTLP over HTTP (JSON and protobuf). gRPC NOT supported.

Semantic conventions: GenAI OTel semantic conventions + custom langfuse.* namespace
attributes. Custom langfuse.* attributes take precedence over generic OTel conventions.

OTel span -> Langfuse mapping:
  OTel spans -> Langfuse observations (span / generation / event)
  Classification by attributes: gen_ai.system, model fields -> generation type
  Trace-level attrs (userId, sessionId, metadata) must be propagated on ALL spans
  for accurate filtering/aggregation (not auto-inherited from root span via OTel baggage)

SELF-HOST OPTIONS

Docker Compose: local VM
Kubernetes: Helm chart
Terraform templates: AWS, Azure, GCP

CLOUD OPTIONS

Langfuse Cloud (managed by Langfuse team, generous free tier, no credit card required)
Multiple regions: EU, US, Japan, HIPAA-compliant

LANGUAGE SDKS

Python (typed)
JavaScript / TypeScript (typed)
Integrations: OpenAI SDK (drop-in), LangChain, LlamaIndex, LiteLLM, others

STATED LATENCY / RELIABILITY GUARANTEES

Not stated in captured documentation.
Async batching implies eventual-consistency model: traces are NOT synchronously persisted.
flush() required for short-lived processes.

VERDICT AGAINST ELAI (E-29 / SSB journal context)

ELAI's current design uses a LOCAL SSB (Secure Scuttlebutt-style) append-only journal
(crates/orchestrator/src/ssb/) as an in-process event store. E-29 plans an OTel export.

Langfuse is a COMPLEMENTARY external destination, not a replacement for the local SSB:

1. Architecture: Langfuse is remote (HTTP push) / external / async / eventually-consistent.
   ELAI SSB is local / in-process / synchronous / append-only. These are different layers.
   SSB = source of truth for local execution correctness (wiring gate, deterministic audit).
   Langfuse = visualization / eval / debugging surface for cross-run analytics.

2. OTel compatibility: Langfuse's OTLP endpoint (HTTP, no gRPC) is a plausible E-29 target
   for ELAI's OTel span export. The custom langfuse.* attributes give a migration path from
   ELAI's existing SSB action_type taxonomy to Langfuse's data model.

3. License: MIT (core) means self-hosted deployment is viable without license cost. The EE
   folders gate enterprise features (SSO, RBAC, etc.) -- community self-host is fully open.

4. Local SSB journal vs Langfuse: for ELAI's wiring-enforcement Layer-4 runtime assertions,
   the SSB journal MUST remain local and synchronous (missing evidence = FAIL must be
   evaluated in-process, not deferred to an async remote store). Langfuse cannot replace
   that layer. It could complement it for dashboarding, eval scoring, and cross-session
   trend analysis.

5. gRPC gap: ELAI's OTel export (E-29) must use OTLP/HTTP if Langfuse is the target.
   If the current OTel exporter in ELAI is wired for gRPC, that needs an endpoint swap.

UNVERIFIED: exact Postgres vs ClickHouse split in the self-host data model
UNVERIFIED: whether the self-hosted OTLP endpoint path (/api/public/otel) is identical to cloud
UNVERIFIED: Langfuse score API schema (not fetched)
UNVERIFIED: latency SLAs, ingestion rate limits on free tier
