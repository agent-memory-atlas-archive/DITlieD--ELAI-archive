# Agent protocols + MCP for ELAI (2024-2026)

## TL;DR
- Adopt MCP partially, not fully. Keep ELAI's binary postcard IPC for orchestrator-agent (hot path, latency-bound, internal trust boundary). Use MCP only at the external boundary (dashboard tools, Designer-as-MCP-server for Empire D2, ingestion of third-party servers). Grounded in: MCP's tool-definitions inflate context by 15-80k tokens per turn at multi-server scale ([MCP Token Trap, OnlyCLI 2025](https://onlycli.github.io/OnlyCLI/blog/mcp-token-cost-benchmark/), [MindStudio 2025](https://www.mindstudio.ai/blog/claude-code-mcp-server-token-overhead)), and postcard at ~70% of bincode size and 60ns serialize is unbeatable for internal hot path ([users.rust-lang.org 2024-2025](https://users.rust-lang.org/t/is-it-better-to-use-bincode-or-postcard/88740)).
- MCP is NOT enough on its own for ELAI's needs: it has no agent identity, no agent-to-agent spawning, no declared multi-step plans, no replanning primitive, no observation back-pressure ([Scola, Medium Mar 2026](https://medium.com/@thomas.scola/mcp-is-not-enough-the-missing-gaps-in-open-agent-standards-3bc31e7b4e59), [WorkOS Nov 2025](https://workos.com/blog/mcp-2025-11-25-spec-update)). Nov 2025 spec added Tasks (async/cancel), Sampling-with-Tools, URL-mode elicitation — but ELAI's FSM/scope-planner/agent-pool already does what MCP Tasks does, in-process, with no JSON-RPC overhead.
- Anthropic-compatible API (/v1/messages) is now the de-facto cross-provider standard: AWS Claude Platform, Cloudflare AI Gateway, Google Vertex, OpenAI's REST surface speaks both. ELAI's sidecar choice (Anthropic protocol) is strategically correct for 2026, but adding a LiteLLM/OpenRouter/Portkey gateway in front would add 10-50ms per call and break prompt caching ([Pkgpulse 2026](https://www.pkgpulse.com/guides/portkey-vs-litellm-vs-openrouter-llm-gateway-2026), [decodethefuture Apr 2026](https://decodethefuture.org/en/anthropic-blocks-third-party-tools/)). Do NOT add a gateway layer.
- Tool poisoning + rug-pull are real, demonstrated attack vectors with 7.2% of public MCP servers vulnerable (Queen's Univ. study via [Practical DevSecOps](https://www.practical-devsecops.com/mcp-tool-poisoning/), Trail of Bits "line jumping", OWASP MCP03:2025, postmark-mcp BCC backdoor Sept 2025). Any MCP server ELAI ingests must be hash-pinned and tool-description-sanitized before exposure to the model.
- ELAI's MCP bridge (HTTP JSON-RPC, deny-list + sanitization, dashboard-consumed) is already on the right track. The needed additions: tool-schema hash pinning (anti-rug-pull), tool-description sanitization on every list (anti-line-jumping), per-tool token-budget enforcement (anti-context-bloat), and supply-chain signing per OWASP MCP03:2025 ([owasp.org](https://owasp.org/www-project-mcp-top-10/2025/MCP03-2025%E2%80%93Tool-Poisoning)).

## Key sources

| # | URL | what | takeaway |
|---|---|---|---|
| 1 | https://modelcontextprotocol.io/specification/2025-11-25 | Canonical MCP spec | JSON-RPC 2.0, stateful, host/client/server triad, capability negotiation, resources/prompts/tools server-side, sampling/roots/elicitation client-side, security explicitly left to implementor |
| 2 | https://blog.modelcontextprotocol.io/posts/2025-11-25-first-mcp-anniversary/ | MCP 1yr post | ~2,000 servers in registry, 407% growth since Sept 2025; Tasks (SEP-1686) experimental; OAuth simplified (SEP-991); Sampling-with-Tools (SEP-1577) |
| 3 | https://blog.trailofbits.com/2025/04/21/jumping-the-line-how-mcp-servers-can-attack-you-before-you-ever-use-them/ | "Line jumping" prompt-injection-via-tools-list | tools/list descriptions are unsanitized model input; PoC exfiltrates chat history, suppresses alerts; no CVE |
| 4 | https://owasp.org/www-project-mcp-top-10/2025/MCP03-2025%E2%80%93Tool-Poisoning | OWASP MCP Top 10 #3 | Mitigations: sign schemas, immutable registry, RBAC, OPA/Rego policy-as-code, provenance metadata |
| 5 | https://thehackernews.com/2025/09/first-malicious-mcp-server-found.html | First in-wild malicious MCP (postmark-mcp v1.0.16 Sept 17 2025) | 1-line BCC backdoor to phan@giftshop.club; 1,643 downloads; ~300 orgs compromised; rug-pull from a previously-trusted package |
| 6 | https://venturebeat.com/security/mcp-stacks-have-a-92-exploit-probability-how-10-plugins-became-enterprise | MCP stack exploit probability | 92% exploit probability across surveyed enterprise MCP stacks |
| 7 | https://onlycli.github.io/OnlyCLI/blog/mcp-token-cost-benchmark/ | MCP token trap benchmark | MCP burns ~35x more tokens than equivalent CLI invocation; tool defs live in every-message context |
| 8 | https://www.mindstudio.ai/blog/claude-code-mcp-server-token-overhead | Claude Code MCP overhead | 93-tool GitHub MCP = 55,000 tokens before any work; 5 servers × 20 tools = ~80,000 tokens schema overhead |
| 9 | https://www.anthropic.com/engineering/code-execution-with-mcp | Anthropic "Code Execution with MCP" | Solution: expose tools as typed API, model writes program; Tool Search (late 2025) defers schema loading |
| 10 | https://a2a-protocol.org/latest/specification/ | A2A 1.0.0 spec | JSON-RPC 2.0 + gRPC + HTTP/REST bindings; Agent Card discovery; Task lifecycle states (submitted/working/input_required/completed/failed/canceled/auth_required); Push notifications |
| 11 | https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability/ | A2A launch Apr 9 2025 | 50+ partners, Apache 2.0, donated to Linux Foundation, v0.3 then v1.0.0 |
| 12 | https://www.microsoft.com/en-us/research/blog/autogen-v0-4-reimagining-the-foundation-of-agentic-ai-for-scale-extensibility-and-robustness/ | AutoGen v0.4 Jan 2025 | Actor model, async messaging, 30% latency reduction, layered (Core / AgentChat / Extensions) |
| 13 | https://openai.github.io/openai-agents-python/ | OpenAI Agents SDK | Function-tool wrap, hosted tools, ComputerTool/ApplyPatchTool, MCP server integration baked in; production successor to Swarm |
| 14 | https://github.com/openai/swarm | OpenAI Swarm | Educational/experimental only; handoff = return Agent object; stateless; not for production |
| 15 | https://users.rust-lang.org/t/is-it-better-to-use-bincode-or-postcard/88740 | postcard vs bincode | postcard 41B at 60ns serialize/180ns deserialize, ~70% of bincode size, ~1.5x slower |
| 16 | https://david.kolo.ski/blog/rkyv-is-faster-than/ | rkyv beats {bincode, capnp, cbor, flatbuffers, postcard, prost, serde_json} | rkyv zero-copy is fastest deserialize; capnp serialize ~32ns/14GB/s, deserialize ~344ns |
| 17 | https://blog.fka.dev/blog/2025-06-06-why-mcp-deprecated-sse-and-go-with-streamable-http/ | MCP deprecated SSE Mar 2025 | Replaced with Streamable HTTP (single POST endpoint + optional SSE response) for resumability + cancellation |
| 18 | https://medium.com/@thomas.scola/mcp-is-not-enough-the-missing-gaps-in-open-agent-standards-3bc31e7b4e59 | MCP gaps | "Agent is not a tool" — 7 gaps: identity, manifests, federated discovery, cross-framework portability, policy-as-primitive, audit trails, separation of duties |
| 19 | https://workos.com/blog/mcp-2025-11-25-spec-update | Nov 25 2025 spec update | Async Tasks (cancellable), URL-mode elicitation, simplified OAuth, extensions framework |
| 20 | https://www.pkgpulse.com/guides/portkey-vs-litellm-vs-openrouter-llm-gateway-2026 | Gateway shootout | LiteLLM +10-20ms overhead, struggles >300-500 RPS; OpenRouter +<50ms; semantic-cache only on Portkey |
| 21 | https://decodethefuture.org/en/anthropic-blocks-third-party-tools/ | Anthropic gateway breakage Apr 2026 | Third-party tools bypass Anthropic's internal prompt cache layer — every request full cost; gateways break caching |
| 22 | https://docs.cursor.com/context/model-context-protocol | Cursor MCP integration | Cursor client launches stdio child or HTTP; ~40 tool ceiling before degradation; resources in v1.6, elicitation in v1.5 |
| 23 | https://code.claude.com/docs/en/mcp | Claude Code MCP docs | `claude mcp add` with --transport stdio/http; project scope via .mcp.json; ~/.claude.json personal |
| 24 | https://github.com/modelcontextprotocol/inspector | MCP Inspector | React UI + Node proxy; JSON-RPC traffic viewer; CVE-2025-49596 RCE in older versions |
| 25 | https://www.reversinglabs.com/blog/mcp-rug-pull-attack-worries | Rug-pull mechanism | No version-pinning, no hash verification for tool schemas; silent updates |
| 26 | https://en.wikipedia.org/wiki/Model_Context_Protocol | MCP Wikipedia | Nov 2024 launch; OpenAI adopted Mar 2025; Anthropic donated to Linux Foundation (AAIF) Dec 2025; 97M+ monthly SDK downloads |

## Protocol landscape

| protocol | shape | transport | strengths | weaknesses | adoption | citation |
|---|---|---|---|---|---|---|
| MCP (Anthropic, 2024-11) | host/client/server, capability negotiation | JSON-RPC 2.0 over stdio or Streamable HTTP | Universal tool integration, 10k+ servers, Linux Foundation governance | No agent identity, no plans, no back-pressure, tool-poisoning surface, schema bloat | ~2k registry servers, 97M SDK DL/mo, Claude Code + Cursor + ChatGPT native | [1](https://modelcontextprotocol.io/specification/2025-11-25), [2](https://blog.modelcontextprotocol.io/posts/2025-11-25-first-mcp-anniversary/) |
| A2A (Google, 2025-04) | agent-to-agent, Agent Card discovery | JSON-RPC 2.0 + gRPC + HTTP/REST | Multi-binding; explicit Task lifecycle (8 states); push notifications; mTLS/OAuth2 | Early adoption; opaque agents only (no internal observability); heavyweight for in-process | 50+ partners, S&P Global, ServiceNow Fabric; v1.0.0 | [10](https://a2a-protocol.org/latest/specification/), [11](https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability/) |
| OpenAI Agents SDK | function-tool wrap + handoff + hosted tools | Anthropic-style messages + function-call JSON | Production-grade, MCP built in, ComputerTool/ApplyPatch hosted | OpenAI-centric, no inter-process wire format | Production successor to Swarm | [13](https://openai.github.io/openai-agents-python/) |
| AutoGen v0.4 | actor model, async messages | In-proc + cross-process via gRPC/Orleans | 30% latency cut, modular, observable, scales | Complex; Microsoft-ecosystem-leaning | Jan 2025 launch | [12](https://www.microsoft.com/en-us/research/blog/autogen-v0-4-reimagining-the-foundation-of-agentic-ai-for-scale-extensibility-and-robustness/) |
| CrewAI | tasks + crews + Pydantic outputs | Python in-proc + MCP integration | Easy to start, typed outputs | Wire format unspecified externally | Strong tutorial mass | [CrewAI docs](https://docs.crewai.com/en/concepts/crews) |
| LangGraph | state machine with reducers + checkpoints | Python in-proc; checkpoint persistence | Time-travel, parallel branches, explicit state | Python-only; not a wire protocol | Industry standard for agent state | [Sider 2025](https://sider.ai/blog/ai-tools/langgraph-review-is-the-agentic-state-machine-worth-your-stack-in-2025) |
| OpenAI Swarm | Agents + handoffs + routines | Chat Completions API | Minimal, observable | Experimental, not for prod | Educational only | [14](https://github.com/openai/swarm) |
| ELAI IPC | 15 msg types, postcard | 1-byte ver + 4-byte len + postcard | In-process latency, Rust-native, zero schema bloat | Internal-only, no external interop | Internal | code |
| ELAI MCP bridge | HTTP JSON-RPC | http_initialize/http_list_tools/http_call_tool | Deny-list + sanitization; dashboard-consumed | Not yet hash-pinned (rug-pull risk) | Internal | code |
| Anthropic /v1/messages | sidecar API | HTTP+JSON | De-facto standard; AWS/Cloudflare/Google support | None for ELAI's use case | Multi-provider | [Cloudflare AI Gateway docs](https://developers.cloudflare.com/ai-gateway/usage/providers/anthropic/) |

Binary wire formats (for IPC):

| format | serialize | deserialize | size vs bincode | zero-copy | citation |
|---|---|---|---|---|---|
| bincode | fastest baseline | fast | 100% | no | [15](https://users.rust-lang.org/t/is-it-better-to-use-bincode-or-postcard/88740) |
| postcard | 60ns | 180ns | ~70% (smaller) | no | [15](https://users.rust-lang.org/t/is-it-better-to-use-bincode-or-postcard/88740) |
| msgpack (rmp-serde) | 784-922ns | 2234-2686ns | similar to postcard | no | [djkoloski bench](https://github.com/djkoloski/rust_serialization_benchmark) |
| capnp | ~32ns | ~344ns | larger | yes | [16](https://david.kolo.ski/blog/rkyv-is-faster-than/) |
| flatbuffers | poor on structured data | yes (zero-copy) | larger | yes | [16](https://david.kolo.ski/blog/rkyv-is-faster-than/) |
| rkyv | fastest | fastest (zero-copy) | small | yes | [16](https://david.kolo.ski/blog/rkyv-is-faster-than/) |
| protobuf (prost) | mid | mid | mid | no | [djkoloski bench](https://github.com/djkoloski/rust_serialization_benchmark) |

## MCP deep-dive

**What it does well**
- Universal tool integration: 10k+ active servers in production, 97M+ monthly SDK downloads, Linux Foundation governance under AAIF since Dec 2025 ([Wikipedia](https://en.wikipedia.org/wiki/Model_Context_Protocol)).
- Capability negotiation per JSON-RPC handshake; clean host/client/server triad.
- Nov 2025 spec added Tasks abstraction with states (working/input_required/completed/failed/canceled) — finally addresses long-running operations ([WorkOS](https://workos.com/blog/mcp-2025-11-25-spec-update)).
- Sampling-with-Tools (SEP-1577) lets server run agent loops using client tokens.
- Streamable HTTP (Mar 2025) deprecated SSE — single POST endpoint, optional SSE response, resumable, cancellable ([fka.dev](https://blog.fka.dev/blog/2025-06-06-why-mcp-deprecated-sse-and-go-with-streamable-http/)).
- MCP Inspector ([github.com/modelcontextprotocol/inspector](https://github.com/modelcontextprotocol/inspector)) gives Chrome-DevTools-style JSON-RPC traffic view; runs at http://localhost:6274.

**What it does badly**
- **Token explosion**: tool defs are injected on EVERY message. 93-tool GitHub MCP = 55,000 tokens before any work; 5 servers × 20 tools ≈ 80,000 tokens schema overhead per turn ([MindStudio](https://www.mindstudio.ai/blog/claude-code-mcp-server-token-overhead)). OnlyCLI measured 35x more tokens than equivalent CLI ([OnlyCLI](https://onlycli.github.io/OnlyCLI/blog/mcp-token-cost-benchmark/)). Anthropic's mitigation: Tool Search + "code execution with MCP" — expose tools as typed API, let model write a program ([Anthropic Engineering](https://www.anthropic.com/engineering/code-execution-with-mcp)).
- **No agent semantics**: per Scola ([Medium](https://medium.com/@thomas.scola/mcp-is-not-enough-the-missing-gaps-in-open-agent-standards-3bc31e7b4e59)), "An agent is not a tool. An agent has identity, persistent state, declared capabilities, a lifecycle, and the ability to spawn or delegate to other agents. MCP does not specify any of that."
- **Lowest-common-denominator**: per Sanjeev Mohan critical analysis ([Medium](https://sanjmo.medium.com/to-mcp-or-not-to-mcp-part-1-a-critical-analysis-of-anthropics-model-context-protocol-571a51cb9f05)), "MCP, as middleware aiming to abstract various software APIs, faces the 'lowest common denominator' problem; it cannot fully support all features of underlying tools."
- **Proxy hop latency**: same critique — "could introduce latency... may not work for time-sensitive real-time AI applications" and risks "race conditions and failures."
- **Cursor's ~40-tool ceiling**: agent selection degrades past that ([Cursor docs](https://docs.cursor.com/context/model-context-protocol)).
- **No back-pressure** for streamed observations; no native multi-step plan primitive; no native replanning hook ([Scola](https://medium.com/@thomas.scola/mcp-is-not-enough-the-missing-gaps-in-open-agent-standards-3bc31e7b4e59)).

**Attack surface (cited)**
- **Tool poisoning / "line jumping"** ([Trail of Bits Apr 21 2025](https://blog.trailofbits.com/2025/04/21/jumping-the-line-how-mcp-servers-can-attack-you-before-you-ever-use-them/)): tools/list descriptions are unsanitized model input; PoC exfiltrates chat history, plants `chmod -R 0666 ~;` prefixes, suppresses security alerts.
- **OWASP MCP03:2025 Tool Poisoning** ([owasp.org](https://owasp.org/www-project-mcp-top-10/2025/MCP03-2025%E2%80%93Tool-Poisoning)): formal threat category with 4 scenarios (CI pipeline poisoning, trojaned dep, insider escalation, MITM rewrite). Mitigations: sign schemas, immutable registry, OPA/Rego policy-as-code, provenance metadata.
- **postmark-mcp rug-pull (Sept 17 2025)** ([Hacker News](https://thehackernews.com/2025/09/first-malicious-mcp-server-found.html), [Koi Security](https://www.koi.ai/blog/postmark-mcp-npm-malicious-backdoor-email-theft)): first in-wild malicious MCP. v1.0.16 added 1-line BCC backdoor to phan@giftshop.club. 1,643 downloads; ~300 orgs compromised; estimated 3,000-15,000 emails/org/day exfiltrated.
- **OX Security registry poisoning**: 9 of 11 MCP registries poisoned with test payload, command execution confirmed on 6 live production platforms with paying customers ([ReversingLabs](https://www.reversinglabs.com/blog/mcp-rug-pull-attack-worries)).
- **Clawdbot exposure (Jan 2026)**: 2,000+ exposed MCP instances leaking API keys + conversation histories, indexed in Shodan ([ReversingLabs](https://www.reversinglabs.com/blog/mcp-rug-pull-attack-worries)).
- **CVE-2025-49596**: RCE in older MCP Inspector versions — malicious MCP server can execute arbitrary code on the Inspector host ([Snyk](https://snyk.io/articles/how-to-debug-mcp-server-with-anthropic-inspector/)).
- **Queen's Univ. analysis of 1,899 OSS MCP servers** (via [Practical DevSecOps](https://www.practical-devsecops.com/mcp-tool-poisoning/)): 7.2% have general vulns, 5.5% exhibit MCP-specific tool poisoning.
- **MCP stack exploit probability**: 92% across surveyed enterprise stacks ([VentureBeat](https://venturebeat.com/security/mcp-stacks-have-a-92-exploit-probability-how-10-plugins-became-enterprise)).

## ELAI integration options

**Option A: Full MCP adoption (orchestrator-agent IPC replaced with MCP)**
- Pros: external interop, debugger ecosystem (MCP Inspector), free third-party tool ingestion.
- Cons:
  - Token explosion in internal flows that don't need to go through a model context anyway. ELAI's IPC is process-to-process, not model-context — there's no benefit to JSON-RPC schemas living in any context window. ([MindStudio](https://www.mindstudio.ai/blog/claude-code-mcp-server-token-overhead) confirms the schema-injection-per-message problem applies to MCP-as-tool-channel.)
  - Latency: postcard ~60ns serialize vs JSON-RPC over HTTP ~ms range ([users.rust-lang.org](https://users.rust-lang.org/t/is-it-better-to-use-bincode-or-postcard/88740)).
  - Loses ELAI's 15-message-type discipline; MCP is open-ended.
  - Adds rug-pull surface to internal trust boundary.
- Verdict: REJECT.

**Option B: MCP for external tools only (current state)**
- Pros: matches Cursor's pattern ([Cursor docs](https://docs.cursor.com/context/model-context-protocol)) and Claude Code's pattern ([Claude Code docs](https://code.claude.com/docs/en/mcp)). Dashboard consumes ELAI's MCP bridge; Designer can be exposed as MCP server to Empire D2.
- Cons: missing (1) tool-schema hash pinning per OWASP MCP03:2025; (2) tool-description sanitization on every list response per Trail of Bits line-jumping; (3) per-tool token-budget enforcement.
- Verdict: KEEP + HARDEN. This is the recommended path.

**Option C: MCP for ALL tool calls including internal**
- Pros: uniform debugging via MCP Inspector.
- Cons: same as Option A but worse — every internal step pays the JSON-RPC tax. AutoGen v0.4 explicitly chose actor-model in-process messaging precisely because of 30% latency reduction over older request/response patterns ([Microsoft Research](https://www.microsoft.com/en-us/research/blog/autogen-v0-4-reimagining-the-foundation-of-agentic-ai-for-scale-extensibility-and-robustness/)).
- Verdict: REJECT.

## Wire format choices

For ELAI's orchestrator-agent IPC (per-baby-step latency budget, internal trust boundary, Rust-only):

- **postcard (current)**: 60ns serialize, 180ns deserialize, ~70% bincode size, embedded-grade ([users.rust-lang.org](https://users.rust-lang.org/t/is-it-better-to-use-bincode-or-postcard/88740)). Already shipped. No reason to move.
- **bincode**: faster (~40-50ns) but 30% larger payloads. Marginal gain.
- **msgpack (rmp-serde)**: 784-922ns serialize, 2234-2686ns deserialize ([djkoloski/rust_serialization_benchmark](https://github.com/djkoloski/rust_serialization_benchmark)). 13-15x slower than postcard. REJECT for hot path. Acceptable if cross-language interop demanded.
- **capnp**: 32ns serialize (zero-copy), 344ns deserialize, 14GB/s ([rkyv comparison](https://david.kolo.ski/blog/rkyv-is-faster-than/)). Schema-driven, cross-language. Win if Python sidecar ever needs zero-copy. Cost: extra schema file + codegen step + bigger payloads.
- **flatbuffers**: "very poor serialization performance on highly-structured data, sometimes even worse than serde_json" per Koloski ([rkyv blog](https://david.kolo.ski/blog/rkyv-is-faster-than/)). REJECT.
- **rkyv**: fastest deserialize (zero-copy without schema file), Rust-native ([rkyv blog](https://david.kolo.ski/blog/rkyv-is-faster-than/)). Best theoretical replacement for postcard IF latency ever becomes a measured bottleneck. Migration risk: rkyv's safe-mutation story is newer than postcard's stability.
- **protobuf (prost)**: middle of the road. Use only if A2A interop required ([a2a-protocol.org](https://a2a-protocol.org/latest/specification/)).

Recommendation: STAY ON postcard. Bench-then-migrate to rkyv only if a measured baby-step boundary shows IPC > 5% of total step latency.

For sidecar I/O: keep Anthropic /v1/messages + postcard internal. Do NOT add LiteLLM/OpenRouter/Portkey — LiteLLM adds 10-20ms + breaks past 300-500 RPS ([PkgPulse](https://www.pkgpulse.com/guides/portkey-vs-litellm-vs-openrouter-llm-gateway-2026)); third-party gateways bypass Anthropic's prompt cache and pay full token cost on every call ([decodethefuture](https://decodethefuture.org/en/anthropic-blocks-third-party-tools/)).

## Anti-patterns

- **Loading all tool schemas upfront** — burns 15-80k tokens per turn ([MindStudio](https://www.mindstudio.ai/blog/claude-code-mcp-server-token-overhead), [OnlyCLI](https://onlycli.github.io/OnlyCLI/blog/mcp-token-cost-benchmark/)). Use Tool Search / code-execution-with-MCP pattern ([Anthropic Engineering](https://www.anthropic.com/engineering/code-execution-with-mcp)).
- **Trusting tool descriptions** — they ARE prompt-injection vectors ([Trail of Bits "line jumping"](https://blog.trailofbits.com/2025/04/21/jumping-the-line-how-mcp-servers-can-attack-you-before-you-ever-use-them/)). Sanitize on every list.
- **No version pinning on MCP servers** — rug-pull happened in the wild (postmark-mcp Sept 2025, [Hacker News](https://thehackernews.com/2025/09/first-malicious-mcp-server-found.html)). Pin by hash, alert on change.
- **Putting an LLM gateway in front of Anthropic** — breaks prompt caching, adds 10-50ms, will be enforcement-blocked by Anthropic starting Apr 2026 ([decodethefuture](https://decodethefuture.org/en/anthropic-blocks-third-party-tools/)).
- **Treating MCP as an agent protocol** — it isn't ([Scola](https://medium.com/@thomas.scola/mcp-is-not-enough-the-missing-gaps-in-open-agent-standards-3bc31e7b4e59)). For agent-to-agent, A2A is the candidate ([a2a-protocol.org](https://a2a-protocol.org/latest/specification/)) — but only when ELAI needs to talk to external agents.
- **Auto-approving MCP tool calls** — Cursor 2025 CVEs ([TrueFoundry MCP Cursor guide](https://www.truefoundry.com/blog/mcp-servers-in-cursor-setup-configuration-and-security-guide)) and CVE-2025-49596 in MCP Inspector show this is the routine RCE vector.
- **Using OpenAI Swarm in production** — repo explicitly says "experimental and not intended to be used in production" ([github.com/openai/swarm](https://github.com/openai/swarm)).
- **Putting MCP traffic on the orchestrator hot path** — see Anti-pattern #1; ELAI's per-baby-step latency budget cannot absorb 80k-token schema injection per call.

## Open questions

1. **Empire D2 / Designer-as-MCP-server**: which subset of Designer methods is safe to expose? Need a deny-list audit before shipping. Measurable: count of Designer methods × token cost of each schema × expected concurrent D2 sessions vs. Cursor's measured 40-tool ceiling ([Cursor docs](https://docs.cursor.com/context/model-context-protocol)).
2. **Should ELAI ingest external MCP servers (e.g., from mcp.so registry)?** If yes, mandatory: hash-pin + tool-description-sanitize + per-call OPA policy (per OWASP MCP03:2025). Measurable: time-to-first-rug-pull-detection on a controlled test server.
3. **Is A2A worth implementing for Designer-to-external-agent flows?** Measurable: does any existing ELAI consumer speak A2A (S&P Global, ServiceNow Fabric do; in ELAI's user base — likely none).
4. **rkyv migration**: bench IPC as % of baby-step latency. Threshold for migration: > 5% of total step latency. Measurable: cargo bench on `agent_message_roundtrip` benchmark before and after.
5. **MCP Tasks (SEP-1686) for ELAI's external MCP bridge**: ELAI's FSM already does what Tasks does. Should the bridge expose ELAI's FSM substates as MCP Task states so external MCP clients can poll long-running ELAI agent runs? Measurable: prototype against MCP Inspector, count of state-conversion edge cases.
