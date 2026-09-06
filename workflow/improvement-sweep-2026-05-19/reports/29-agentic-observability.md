# Agent observability for ELAI (2024-2026)

## TL;DR
- Emit OTel GenAI semantic-convention attributes (`gen_ai.operation.name`, `gen_ai.agent.name`, `gen_ai.tool.name`, `gen_ai.usage.input_tokens|output_tokens|cache_*`, `gen_ai.response.finish_reasons`) on every SSB row that mirrors an LLM/tool/agent op so SSB becomes OTLP-exportable without a rewrite. SSB stays canonical; OTel becomes an export view.
- Add a Laminar-style "transcript view" drawer next to the span/timeline drawers. Long agent runs (BSP supersteps, 74-action_type SSB) are unreadable as flat span trees; transcripts (what the agent said + tool calls + tool results, rendered as a chat) plus filters on action_type are the SOTA for "where did it go wrong" debugging.
- Make replay a first-class SSB action_type. Record model_id, decode params, seed, system clock, tool I/O, and safety/guardrail config per LLM call. Temperature=0 alone is insufficient; full I/O capture is the only reliable reproduction primitive.
- Promote SSB checkpoint rows to a LangGraph-style time-travel surface: a Tauri command `ssb_resume_from(checkpoint_id)` that re-runs from any checkpoint with patched prompts/models. This is what LangGraph Studio ships and what Laminar's rollout debugger does.
- Add MAST failure taxonomy (14 modes, 3 categories: Specification 41.8%, Coordination 36.9%, Verification 21.3%) as a classifier over closed traces so failed runs auto-tag a root-cause category, surfaced in the MAP/RESEARCH orbit modes.

## Key sources
| URL | What | Takeaway |
|---|---|---|
| https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/ | OTel GenAI agent span semconv (semconv 1.40.0, April 2026) | Canonical attribute schema: `invoke_agent`, `create_agent`, `invoke_workflow`, `execute_tool` spans; required `gen_ai.operation.name` + `gen_ai.provider.name`; conditional `gen_ai.agent.*`, `gen_ai.request.model`; opt-in `gen_ai.input.messages`/`output.messages` |
| https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/ | OTel GenAI client spans | `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens`, `gen_ai.usage.cache_creation.input_tokens`, `gen_ai.usage.cache_read.input_tokens`, `gen_ai.response.finish_reasons` — what we already track but under different names |
| https://opentelemetry.io/blog/2026/genai-observability/ | OTel blog 2026: GenAI client spans exited experimental early 2026; agent spans still development | Stable enough to adopt; vendors (Datadog v1.37, Grafana/Loki) already ingest natively |
| https://github.com/open-telemetry/semantic-conventions/blob/main/docs/gen-ai/gen-ai-spans.md | OTel semconv repo | Source of truth, evolves monthly; track |
| https://www.traceloop.com/docs/openllmetry/introduction + https://github.com/traceloop/openllmetry | OpenLLMetry / Traceloop | Reference instrumentation library for Anthropic, MCP, LangChain, Ollama — proves OTel GenAI is the practical export format; `TRACELOOP_TRACE_CONTENT` env to gate prompt content |
| https://github.com/langfuse/langfuse + https://langfuse.com/docs/observability/overview | Langfuse v3.172 (May 2026) MIT | Typed observations (generation/span/event/score), OTLP ingest, dataset-based eval-in-CI, LLM-as-judge + heuristic + human-annotation scorers; self-host all features free |
| https://arize.com/docs/phoenix + https://github.com/Arize-ai/phoenix | Arize Phoenix | Built on OTel + OpenInference; Phoenix Evals harness; supports Claude Agent SDK, LangGraph, Vercel AI SDK out-of-box; Elastic License 2.0; experiments + prompt playground |
| https://laminar.sh/article/2026-04-23-top-6-agent-observability-platforms | Laminar 2026 platform ranking | Laminar's design choices: transcript-first (not span-tree-first) reading, Signals = NL-extracted structured events backfilled across history, SQL-over-traces, agent rollout debugger. Apache 2.0. |
| https://www.langchain.com/langsmith/observability + https://docs.langchain.com/langsmith/evaluation | LangSmith 2026 | Scheduled replay of golden traces vs current prod as early-warning system; offline eval as unit tests, online eval as drift detector; LangGraph Studio = breakpoints + state inspection + checkpoint resumption |
| https://wandb.ai/site/articles/ai-agent-observability/ | W&B Weave | `@weave.op()` decorator makes every logical step a traceable unit; eval shifts left into CI (PR-gated regression suite) |
| https://www.helicone.ai/blog/the-complete-guide-to-LLM-observability-platforms | Helicone | Proxy-first model: drop-in, single endpoint change captures everything; cost analytics + caching as side-effect of being on the request path |
| https://tianpan.co/blog/2026-04-12-deterministic-replay-debugging-non-deterministic-ai-agents | Deterministic replay guide 2026 | Temperature=0 + seed are insufficient (Anthropic + OpenAI explicitly state this); must capture: model_id, decode params, full I/O, tool responses, system clock, safety config |
| https://dev.to/sreeni5018/debugging-non-deterministic-llm-agents-implementing-checkpoint-based-state-replay-with-langgraph-5171 | LangGraph checkpoint replay | Checkpoint-based replay = state snapshot per superstep, resume with patched inputs. Direct analogue to SSB checkpoint rows + BSP supersteps |
| https://arxiv.org/pdf/2503.13657 | Cemri et al. "Why Do Multi-Agent LLM Systems Fail?" (NeurIPS 2025) MAST taxonomy | 14 failure modes across 3 categories (Specification 41.8%, Coordination 36.9%, Verification 21.3%) from 1600+ traces. Worst: task misinterpretation, context collapse, incorrect verification, premature termination |
| https://arxiv.org/html/2604.08906v1 | Empirical study of bugs across 5 agentic frameworks 2026 | 409 fixed bugs analyzed; 5-layer architectural abstraction; symptoms include unexpected execution sequences + model-related faults — argues for capturing tool-call ordering as first-class observable |
| https://www.microsoft.com/en-us/security/blog/2026/03/18/observability-ai-systems-strengthening-visibility-proactive-risk-detection/ | MS Security agent observability 2026 | Argues unified tracing + consistent telemetry to follow a single request end-to-end and locate the first mistake's blast radius |
| https://www.axoniq.io/blog/ai-agent-explainability-event-sourcing-infrastructure | Event-sourcing for agent explainability | Append-only event log = queryable audit trail; compliance angle. SSB already does this; the gap is exposing it as event sourcing semantics, not just rows |
| https://arxiv.org/pdf/2602.23193 | ESAA: Event Sourcing for Autonomous Agents | Agents emit only validated JSON intentions; orchestrator persists to append-only `activity.jsonl`, applies effects, projects materialized view. Pattern matches SSB+harness split |
| https://callsphere.ai/blog/vw3c-opentelemetry-genai-conventions-ai-agents-2026 | OTel GenAI for AI agents 2026 | Status: GenAI semconv 1.40.0 (April 2026), client stable, agent/framework experimental-but-stable |

## Observability stack landscape
| Tool | License | OTel GenAI semconv | Replay | Eval integration | Best feature | Citation |
|---|---|---|---|---|---|---|
| Laminar | Apache 2.0 | Native, OTel-first | Full re-execution from any span; prompt/model patching | Limited (debugging focus) | Transcript view + Signals (NL outcome extraction) + SQL over traces | laminar.sh ranking |
| Langfuse | MIT | Full OTLP ingest | Observation tree; weaker on deep agent runs | Strong: datasets, LLM-as-judge, CI harness | Prompt versioning + typed observations + free self-host of everything | github.com/langfuse |
| LangSmith | Closed | Full (March 2026+) | LangGraph Studio breakpoints + checkpoint resume | Online + offline eval, conversation clustering | Stateful debugging IDE for LangGraph | langchain.com/langsmith |
| Arize Phoenix | Elastic 2.0 | Native (OpenInference + OTel) | Span-tree; not agent-conversation-oriented | Phoenix Evals + Ragas/Deepeval/Cleanlab | Notebook workflow + experiments + framework breadth | arize.com/docs/phoenix |
| W&B Weave | Closed | Partial | Weak on long runs | Strong scorer framework | Native W&B integration; PR-gated CI eval | wandb.ai |
| Braintrust | Closed | Partial | Not primary use | Eval-first, comparison workflows | CI-style eval gates, prompt playground | braintrust.dev |
| Helicone | Open core | OTel-compatible | Logs only | Helicone Eval (add-on) | Drop-in proxy, one-line install, cost + cache | helicone.ai |
| Datadog LLM Obs | Closed | Native (v1.37, Feb 2026) | Logs/traces | LLM Experiments preview | Full-stack APM correlation | datadoghq.com |
| Honeycomb | Closed | OpenLLMetry-compat | Trace replay via BubbleUp | Limited | High-cardinality query + BubbleUp anomaly attribution | honeycomb.io |
| Lunary | Open core | Partial | No | Basic | Fastest zero-to-traces for RAG/chatbot | confident-ai.com |
| Portkey | Closed gateway | Partial | No | Limited | Gateway-first (routing, failover) with observability bolted on | portkey.ai |
| New Relic AI | Closed | Partial | No | Limited | APM correlation | New Relic docs |

## OpenTelemetry GenAI semantic conventions
What ELAI SSB should emit to be OTel-compat (semconv 1.40.0, semconv repo, gen-ai-agent-spans.md):

Required on every agent op:
- `gen_ai.operation.name` ∈ {`create_agent`, `invoke_agent`, `invoke_workflow`, `execute_tool`, `chat`, `generate_content`}
- `gen_ai.provider.name` (e.g. `anthropic`, `openai`, `huihui-ai/qwen2.5-coder` for local)

Conditional (set when known):
- `gen_ai.agent.id`, `gen_ai.agent.name`, `gen_ai.agent.description`, `gen_ai.agent.version`
- `gen_ai.request.model`, `gen_ai.request.seed`, `gen_ai.request.choice.count`
- `gen_ai.conversation.id` (maps to ELAI session_id), `gen_ai.data_source.id`
- `error.type` on failure
- `gen_ai.workflow.name` for BSP-level / phase-level spans

Recommended (we already have data for most):
- `gen_ai.request.temperature`, `gen_ai.request.max_tokens`, `gen_ai.request.top_p`, `gen_ai.request.stop_sequences`, `gen_ai.request.frequency_penalty`, `gen_ai.request.presence_penalty`
- `gen_ai.response.finish_reasons`
- `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens`
- `gen_ai.usage.cache_creation.input_tokens`, `gen_ai.usage.cache_read.input_tokens` (prompt caching — Anthropic-native)

Opt-in (content):
- `gen_ai.input.messages`, `gen_ai.output.messages`, `gen_ai.system_instructions`, `gen_ai.tool.definitions` — gate behind a `TRACELOOP_TRACE_CONTENT`-style env var, see OpenLLMetry pattern

Tool execution (children of invoke_agent):
- Span name `execute_tool {gen_ai.tool.name}`; tool types ∈ {Extension, Function, Datastore}. ELAI's 74 action_types map mostly to Function and Datastore.

Span hierarchy:
- `invoke_workflow {phase}` → `invoke_agent {agent_role}` → `chat {model}` + `execute_tool {tool}` (siblings under agent)

Sources: opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/, gen-ai-spans.md, semconv 1.40.0 repo.

## SSB vs event-log vs trace-tree pattern
Tradeoffs for ELAI's use case:

SSB (current): SQLite WAL, 74 action_types, queryable via SsbViewer + Tauri. Strengths: single source of truth, transactional, SQL access, survives crashes, doubles as inter-agent communication, doubles as audit log (ESAA-style — arxiv 2602.23193). Weaknesses: not OTel-native; no first-class span parent_id semantics; readers ad-hoc.

Event-log (append-only JSONL like ESAA's `activity.jsonl`): Strengths: stream-friendly, infinitely append, easy export, easy ingest into Kafka/ClickHouse for analytics. Weaknesses: no transactional integrity, query requires projection, no built-in indexing. ELAI's SSB already gives the audit-log properties without the JSONL fragility (axoniq.io event-sourcing arg).

Trace-tree (OTel spans, parent_id pointer): Strengths: industry-standard, every tool plugs in, visualizable in Jaeger/Tempo/Phoenix/Langfuse, span_kind discrimination. Weaknesses: spans are time-bounded windows + attribute bags; agent state lives outside the trace; reconstructing a multi-day session means correlating by trace_id across processes — fragile.

Verdict: SSB stays canonical. Add a thin OTel exporter that maps each SSB row to a span (parent_id resolved from existing checkpoint_id / phase_id columns). Best of both: SSB's atomicity + queryability, OTel's ecosystem reach. This is exactly what AgentFS does (turso.tech/agentfs): SQLite as backing store, structured observability as a view.

## Replay / debugging patterns
What enables time-travel for agent debugging (tianpan.co 2026, dev.to LangGraph checkpoint replay, debugg.ai):

1. **Full input capture**: model_id, decode params (temp/top_p/top_k/seed/stop), system_instructions, full message list, tool definitions, safety/guardrail config. Temperature=0 + seed is not deterministic — Anthropic and OpenAI both state this explicitly.
2. **Full output capture**: completion + finish_reason + token logprobs (if available) + tool_use blocks.
3. **Tool I/O capture**: tool input args, tool output, tool execution timestamp. Replay must substitute recorded tool outputs (network/file I/O is non-deterministic).
4. **Clock + RNG interception**: if agent reads `time.now()` or `random.*`, replay must inject recorded values. SSB action_type `clock_read` and `rng_read` would close this.
5. **Checkpoint per superstep / per agent transition**: LangGraph's pattern is to snapshot state at every node boundary so you can resume from any. ELAI's BSP supersteps + checkpoint rows are the natural unit — just need a `ssb_resume_from(checkpoint_id, patch={prompt, model, tool_overrides})` Tauri command.
6. **Golden trace replay**: LangSmith pattern — pick known-good production traces, replay against new model/prompt version, diff outputs at each span. Most reliable regression signal in 2026.
7. **Structural equivalence**: did agent call the same tools in the same order? Cheaper than output-equality check, catches most regressions.

## Best techniques to adopt
Ranked by ROI for ELAI:

| # | Technique | Grounding | Integration hook |
|---|---|---|---|
| 1 | OTel GenAI semconv emission as SSB column aliases + OTLP exporter | semconv 1.40.0 stable; Datadog/Grafana/Honeycomb/Phoenix/Langfuse all ingest. opentelemetry.io/blog/2026/genai-observability | New SSB columns `otel_op_name`, `otel_provider`, `otel_model`; new Rust crate `ssb-otel-export` reading SSB → OTLP gRPC. Zero new action_types — annotates existing rows. |
| 2 | Transcript view drawer (Laminar pattern) | laminar.sh 2026 ranking — transcript was their first build; span trees fail on long agents | New drawer `TranscriptDrawer.svelte` with new Tauri command `ssb_transcript(session_id, filter={role,action_type,window})` that joins prompt/tool_use/tool_result rows in temporal order |
| 3 | Replay primitive: `ssb_resume_from(checkpoint_id, patch)` | LangGraph checkpoint replay (dev.to/sreeni5018); LangSmith golden replay; tianpan.co 2026 | New action_type `replay_resume`; new Tauri command + new RESEARCH-mode orbit node "replay branches" showing parent run + replay deltas as siblings |
| 4 | MAST failure taxonomy auto-classifier on closed traces | Cemri et al. NeurIPS 2025 (arxiv 2503.13657); MS Security 2026 unified-tracing argument | New action_type `failure_classified` written by a post-run agent; new dashboard window "Failure modes" surfacing % per category over time; map mode highlights mode-tagged runs |
| 5 | Signals (NL outcome extraction backfilled across history) | Laminar Signals; SQL-over-traces | New action_type `signal_match`; user writes free-text query "agent asked user for clarification and got useful answer" → LLM classifier writes signal_match rows backfilled across SSB → filter UI in PLAN mode |
| 6 | Per-stage cost + latency rollup with cache_read attribution | OTel gen_ai.usage.cache_* (Anthropic prompt caching); traceloop granular cost-per-feature | Add `cost_usd`, `cache_read_input_tokens`, `cache_creation_input_tokens` to relevant SSB rows; new drawer "Spend" with sparkline per phase/agent/tool |
| 7 | Dataset-from-traces + offline eval harness | Langfuse datasets, LangSmith golden traces, Braintrust scorers | New SSB action_type `eval_dataset_member`; new Tauri command `ssb_promote_to_dataset(trace_ids)`; runs in COMMITTING substate before push |
| 8 | OTLP HTTP/gRPC export endpoint inside the harness so users can pipe SSB to their existing APM | OpenLLMetry; Datadog/Grafana ingestion | CLI flag `--otlp-endpoint`; no schema change beyond #1 |
| 9 | SQL-over-traces (SsbViewer already has this; expose in UI) | Laminar's flagship feature; Honeycomb high-cardinality | New "Query" drawer with prepared queries + free-form SQL bounded by read-only role |
| 10 | Continuous eval-gate before commit (PR-style for autonomous harness) | W&B Weave 2026 CI-shift; Braintrust gates | Pre-COMMITTING substate runs eval pack; tripwire on regression > threshold |

## Anti-patterns
- Span-tree-only UX for >500-span runs. Trace can have 2000 spans with failures multi-tool-calls-deep; transcript + filters beat flat tree (laminar.sh 2026).
- Trusting temperature=0 + seed as "deterministic replay." Anthropic + OpenAI explicitly document non-determinism. Must capture full I/O (tianpan.co 2026).
- Logging only final output without intermediate state. The first wrong tool arg silently corrupts every subsequent step — error cascades unseen unless every step is recorded (futureagi.com, microsoft.com/security 2026).
- Pre-aggregated metrics without raw events. Honeycomb's case: high-cardinality raw traces enable post-hoc question asking; pre-aggregation locks dashboards to questions you already knew to ask (honeycomb.io).
- Mixing observability platform vendor lock-in into the agent core. OpenLLMetry pattern: agent emits OTel spans; backend is swappable (traceloop.com).
- Storing prompt content unconditionally. PII / cost / disk. Gate behind explicit env flag, OpenLLMetry's `TRACELOOP_TRACE_CONTENT=false` default.
- Treating eval as a separate system from traces. Modern stacks (Langfuse, LangSmith, Phoenix, Braintrust) all fuse: production traces become eval datasets (digitalapplied.com 2026, langchain.com).

## Open questions
1. What is the actual span volume from a typical ELAI HEAVY-tier session? If >10k SSB rows, the OTLP export must batch + sample (semconv allows head/tail sampling). Measure on a B-N session.
2. Does MAST's 14-mode taxonomy generalize to ELAI's FSM-driven harness, or do FSM-specific modes need to be added (e.g. "Stuck-in-substate", "Anti-spiral-hit")? Run MAST classifier on last 10 closed sessions and report novel modes.
3. Replay fidelity ceiling on remote sidecar: tool outputs are recorded but the sidecar's KV-cache state isn't. Does cache-warmup before replay change outputs? Measure structural-equivalence rate on 20 replayed traces.
4. Token cost vs. observability cost: how much storage overhead does full `gen_ai.input.messages` capture add to SSB on a 100k-token session? Decide content-capture policy.
5. Continuous-eval gate: what's a useful regression threshold (% of golden traces drifting) before the autonomous harness should block COMMITTING? Needs 30+ trace baseline first.
