# Orchestrator-Authoritative GRC Budget (Phase 4A BLOCK-4)

- **ELAI ids:** phase-4A / P4A-E007 / P4A-E014 / P4A-E015 / P4A-E026 / P4A-E027 / P4A-E029 / P4A-E046 / BLOCK-4
- **Spec status (as designed):** WIRED core / PARTIAL completeness (`ELAI-FEATURE-MAP.md:747-753`)
- **Crate / module (spec):** `crates/orchestrator` IPC gateway + GRC counter; `crates/ipc-types` 15-message enum; `crates/agent` (`elai_cc_agent`) advisory loop only
- **Purpose of this file:** enough architecture to rebuild **orchestrator-side ThinkStep counting**, agent-budget demotion, and the **GroundingRequest** proxy on the 15-message IPC. Not a full isolation encyclopedia.

## 1. What it is

After Tier C isolation, agents are OS processes with **no** LLM client, **no** network, **no** tools (`phase-4A.txt:7-9`, `phase-4A.txt:62-63`, BLOCK-3 `phase-4A.txt:22-23`). The orchestrator is the authority for GRC budget, inference, tools, and grounding, all proxied over a **bounded 15-message IPC** (`ELAI-SPEC-OVERVIEW.md:103-105`, `phase-4A.txt:26-27`). **BLOCK-4:** the orchestrator **counts `ThinkStep` messages externally**. Agent-local `GrcBudget` is **advisory only** (`phase-4A.txt:26-27`, `phase-index-MASTER.txt:701-702`). Phase 2C’s in-process enforcement is demoted, not deleted (`phase-2C.txt:373-380`).

## 2. Why it exists

An isolated agent that still **enforces** its own budget can lie: local `calls_used=0` after a restart, or a burst of ThinkSteps the engine never saw. Audit BLOCK-4 exists because agent-side limits are not a security boundary (`phase-4A.txt:26-27`, `phase-4A.txt:402-403`). Grounding methods that need files/web/tests cannot run inside a netless, tool-less binary — they must be `GroundingRequest` → orchestrator (`phase-4A.txt:168-176`, `phase-2B.txt:25-27`). If the agent budget wins a disagreement, 2A/2C invariants are theater.

## 3. How it works (runtime)

1. **Spawn.** Orchestrator creates `0700` `/tmp/elai_cc_{session}/`, listens, then spawns `elai_cc_agent` with the socket path. `SO_PEERCRED` must match child PID. Version handshake; mismatch → kill (`phase-4A.txt:81-87`). `agent_id` UUID v4 at spawn (`phase-4A.txt:80`).
2. **Wire.** `1-byte version + 4-byte length + postcard payload`. Check length **before** alloc. `MAX_FRAME_SIZE = 10MB` (`phase-4A.txt:52-54`, `phase-4A.txt:274`).
3. **Agent loop (advisory).** Receives `AgentRequest`. Runs a GRC reasoning loop **inside the binary** and may emit `InferRequest`, `ToolRequest`, `BashRequest`, `GroundingRequest`, `ThinkStep`. Heartbeat every **5s**. `AgentKill` → 1s flush (`phase-4A.txt:264-269`). Local `GrcBudget` may decrement; it **cannot** authorize a step the orchestrator refuses.
4. **Orchestrator GRC counter (authoritative, deterministic):**
   - Count `ThinkStep` messages **per topic per agent** (`phase-4A.txt:159-160`, `phase-4A.txt:278`).
   - After `max_calls`, inject a **convergence signal via IPC** (`phase-4A.txt:161`).
   - Per-topic independence: topic A exhausted does not close topic B (`phase-4A.txt:163`).
   - **Disagreement rule:** if agent local count ≠ 0 but orchestrator count == 0 at phase start → **orchestrator wins**; agent must obey convergence regardless of local state (`phase-4A.txt:164-166`).
   - `AgentDispatchConfig` (not `AgentRole` alone) is the authoritative spawn config, including `grc_config` overrides (`phase-4A.txt:71-74`). Feasibility executor override example: **GRC 3/600** (`phase-index-MASTER.txt:711-714`).
5. **`GroundingRequest` proxy:**
   - Agent emits `GroundingRequest{FILE_CHECK, claim}` / `{WEB_CHECK, claim}` / `{SOURCE_CHECK, claim, file_path}` → orchestrator resolves (web search happens **here**, never in the agent) (`phase-4A.txt:168-172`).
   - Route: `GroundingRequest → knowledge+web` (`phase-4A.txt:276-277`).
   - Role-method **capability** matrix from reasoning.txt S9 (`phase-4A.txt:173-177`):
     - executor / auditor: FILE_CHECK, SOURCE_CHECK, WEB_CHECK, TEST_CHECK
     - context-compiler / prompt-engineer / memory-writer: FILE_CHECK, SOURCE_CHECK only
     - TEST_CHECK from context-compiler → **rejected**
   - GroundingRequest **counts toward per-topic budget** (`phase-4A.txt:178`).
   - GroundingRequest for a topic with **no active ThinkStep budget entry** → **rejected** (orphaned-topic bypass) (`phase-4A.txt:179-180`).
   - 2C `allowed_methods` remains the policy layer; 4A is the second enforcement layer (`phase-2C.txt:79-81`).
6. **Other routes (neighbors, required for the organ to function):** `InferRequest → providers`; `ToolRequest → tools+rules+visibility_filter` (`ToolFilter` whitelist; generic `"tool call rejected by policy"`); `BashRequest → sandbox` (`phase-4A.txt:276-277`, `phase-4A.txt:65-68`).
7. **Burst fail-closed.** 10,000 ThinkStep burst → budget enforced, **excess dropped** (`phase-4A.txt:207`). ThinkStep frame cap **< 64KB** (`phase-4A.txt:53`).
8. **Commit / abstain.** Orchestrator accepts a ThinkStep only if the external count allows it; otherwise inject converge or drop. Agent advisory remainder is ignored when it disagrees.
9. **Artifact.** IPC frames + orchestrator per-topic counters. SSB write is **synchronous with ToolResult** (write **before** sending result) (`phase-4A.txt:279`) — neighbor invariant, not the counter itself.

The model inside the agent may **generate** ThinkSteps. It may **not** be the accountant.

## 4. Types, enums, objects

**Exactly 15 message types** (`phase-4A.txt:55-59`, `P4A-E015`) — compile-time `const_assert` / variant count (`phase-4A.txt:92-93`):

```
AgentRequest, AgentResponse,
InferRequest, InferResponse,
ToolRequest, ToolResult,
BashRequest, BashResult,
GroundingRequest, GroundingResult,
ThinkStep,
CheckpointSync,
AgentHeartbeat, AgentKill,
ReplanRequest
```

**Per-type size caps** (`phase-4A.txt:53-54`):

| Type | Cap |
|---|---|
| AgentHeartbeat | < 1KB |
| ThinkStep | < 64KB |
| ToolResult | < 10MB |
| AgentResponse | < `context_limit` |
| any frame | `MAX_FRAME_SIZE` 10MB |

**`AgentDispatchConfig`:** `{ role, model_preference, tool_filter, rule_domains, grc_config, sandbox_config }` — **AUTHORITATIVE** (`phase-4A.txt:71-74`).

**`ToolFilter`:** `{ allowed_tools: Vec<ToolId> }` whitelist at IPC (`phase-4A.txt:65-68`).

**`ProcessConfig`:** `{ allowed_paths, seccomp_profile, resource_limits }` (`phase-4A.txt:64`).

**Heartbeat:** 5s interval / **15s** timeout → SIGKILL (`phase-4A.txt:137`, `phase-4A.txt:275`).

**ReplanRequest (neighbor, CSQ/2D uses MAJOR):** MINOR → auditor fast-track; MAJOR → human approval; **max 1 per agent per phase**; missing evidence rejected; evidence `>500` tokens truncated or rejected (`phase-4A.txt:213-218`).

**GroundingRequest fields:** the 4A tests name `{FILE_CHECK, claim}`, `{WEB_CHECK, claim}`, `{SOURCE_CHECK, claim, file_path}` (`phase-4A.txt:168-172`). Full struct field list is **UNCERTAIN** in `phase-4A.txt` (reasoning.txt L137 cited, file not inlined here).

Same `GrcBudget` struct as 2A/2C; **enforcer changes** (`phase-2C.txt:380`).

## 5. Control flow / state machine

```
agent local GrcEngine.think   →  advisory
     │
     │ ThinkStep / GroundingRequest IPC
     ▼
orchestrator count[agent][topic] += 1
     │
     ├─ over max_calls → inject converge / drop excess
     ├─ GroundingRequest, no topic entry → reject
     ├─ method not in role matrix → reject
     └─ ok → run 2B verifier in orchestrator → GroundingResult
```

**Who owns the loop:** orchestrator IPC gateway (`phase-4A.txt:271-278`). Agent owns generation. 2A/2C engine inside the agent may still **locally** reject malformed steps; it may not **grant** budget the orchestrator has exhausted.

**Forced:** agent binary must not link `reqwest` / `providers` (`phase-4A.txt:90-91`, `:321-322`). Linux production security target; Windows AppContainer preferred, fallback documented-weaker (`phase-4A.txt:33-35`).

## 6. Grounding, receipts, gates

Grounding evidence is whatever 2B returns, produced **in the orchestrator process**. Agent never sees enforcement logic or other agents (`phase-4A.txt:7-8`).

**Never allowed as a confidence signal:**
- Agent-local `remaining_calls`
- Agent self-report of PID / budget (`phase-4A.txt:132-133` host-side NSpid — analog: do not trust agent budget)
- InferResponse text that claims a file exists
- WEB_CHECK performed inside the agent namespace

**2C policy vs 4A capability:** 2C executor/auditor policy is **all six** methods (`phase-2C.txt:78`). 4A test matrix lists FILE/SOURCE/WEB/TEST only for those roles (`phase-4A.txt:174-176`). API_CHECK / API_REGISTRY_CHECK on the wire after 4A: **UNCERTAIN** (network stubs were “until 4A IPC”; 4A tests never name them). Rebuild: at minimum enforce the 4A matrix; do not silently enable API methods without a spec line.

## 7. Budgets, limits, ceilings

| Knob | Value | Owner |
|---|---|---|
| ThinkStep count | `AgentDispatchConfig` / role `GrcBudget.max_calls` | orchestrator |
| default role budgets | 2C tables (executor 8/1200, …) | config → dispatch |
| feasibility override | **3/600** | `AgentDispatchConfig` (`phase-index-MASTER.txt:714`) |
| GroundingRequest | counts as budget use | orchestrator |
| 10k ThinkStep burst | excess dropped | orchestrator |
| ThinkStep frame | < 64KB | frame reader |
| MAX_FRAME_SIZE | 10MB | frame reader |
| heartbeat timeout | 15s | orchestrator |
| IPC RTT target | <1ms p99 for <64KB | 4A.6 (UNGROUNDED target, `phase-4A.txt:376-378`) |
| spawn | <100ms | 4A.6 |
| memory / agent | <30MB (orchestrator.txt ~20MB) | 4A.6 |
| overhead vs in-process | <5% | 4A.6 |
| Replan | max 1 / agent / phase | orchestrator |

**Overflow:** inject converge; drop excess; never let the agent’s leftover calls extend the session.

**rlimits (isolation neighbor):** `RLIMIT_NPROC=1`, `RLIMIT_FSIZE=100MB`, `RLIMIT_AS=512MB` (`phase-4A.txt:294`).

## 8. Roles and neighbors

**Roles using the organ:** every isolated agent. Researcher is the only role with `NETWORK_EXTRA` (veth+iptables, **zero** capabilities, not `CAP_NET_RAW`) (`phase-4A.txt:19-21`). That network is for allowlisted HTTPS, still not a private GRC accountant.

**Consumes:** 2A `ThinkStep` semantics, 2C `AgentGrcConfig` / `allowed_methods`, 2B verifiers. **Produces:** convergence signals, `GroundingResult`, rejects.

**Must not fork:** a second budget in the agent that can overrule; an in-process 2C engine treated as final after isolation is on; counting `InferRequest` instead of `ThinkStep`.

**2D Blocked** still becomes `ReplanRequest` on this bus (`phase-2D.txt:70-73`, `phase-4A.txt:213-216`).

## 9. Config / flags / defaults

- `AgentDispatchConfig` overrides role defaults (feasibility 3/600, `stub_write+bash`, 25s/30s timeouts) (`phase-index-MASTER.txt:711-714`).
- Socket dir `/tmp/elai_cc_{session}/` mode `0700` (`phase-4A.txt:28`, `:82`).
- Env sanitization: only `LANG`, `TERM`, `PATH` (restricted), `TMPDIR`. No `HOME`/`USER`/API keys/`ELAI_CC_*` (`phase-4A.txt:145-146`).
- `enable_sandbox` is an **implementation** flag in one tree defaulting false (`audit-implementation/phase-4A.md:99`) — **not** a spec default. Spec is isolation on.

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `elai_cc` + `elai_cc_agent` | two binaries (`phase-4A.txt:89`) |
| `ipc_types` / `ipc-types` 15-variant enum | `phase-4A.txt:55-59` |
| orchestrator IPC gateway | count ThinkStep; route GroundingRequest (`phase-4A.txt:271-278`) |
| `AgentDispatchConfig` | `phase-4A.txt:71-74` |
| tests `test_4a_grc_budget_*`, `test_4a_grounding_*` | `phase-4A.txt:184-185` |
| `test_4a_adv_ipc_*` including 10k ThinkStep burst | `phase-4A.txt:207`, `:247` |
| metrics `.elai_cc/metrics/phase_4A.toml` | `phase-4A.txt:343` |

**Audited tree:** IpcGateway GRC budget **coded and unit-tested**, **not wired** to a live accept loop; agent `handle_message` ACKs without emitting ThinkStep/GroundingRequest; BLOCK-4 “satisfied in IpcGateway” but unenforced in production (`audit-implementation/phase-4A.md:26-31`, `:137`, `:157-158`). Rebuild requires the **gateway on the live path**, not the type-only cousin.

## 11. Journal / SSB events

- `phase_complete` at phase boundaries: `{ phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes }`, fsync before model unload (`phase-4A.txt:75-78`).
- ToolResult: SSB write **before** dispatch (`phase-4A.txt:279`, `:404`).
- 3B: orchestrator writes after ThinkStep IPC (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` P3B-E025). `reasoning/` never calls SSB.

## 12. Tests and observables

**GRC budget (`phase-4A.txt:159-166`):**
- Orchestrator counts ThinkStep per topic
- After max_calls, convergence signal via IPC
- Agent count advisory; orchestrator authoritative on disagreement
- Topics independent
- Names: `test_4a_grc_budget_*` ; **exactly one assert per test** (`phase-4A.txt:184-186`)

**GroundingRequest (`phase-4A.txt:168-180`):**
- FILE_CHECK / WEB_CHECK / SOURCE_CHECK resolve on orchestrator
- Role-method matrix (TEST_CHECK from context-compiler rejected)
- Counts toward budget
- No active ThinkStep topic → rejected
- Names: `test_4a_grounding_*`

**Adversarial containing this organ (`phase-4A.txt:200-211`):**
- Oversized frame / `length=u32::MAX` rejected **before** alloc
- 10,000 ThinkStep burst: budget enforced, excess dropped
- Mismatched `agent_id` rejected; version mismatch kill; unsolicited AgentResponse rejected

**Agent binary:** `cargo tree -p elai_cc_agent | grep reqwest` count 0; same for `providers` (`phase-4A.txt:90-91`).

Done-gate: “GRC budget enforced by orchestrator, not agent” (`phase-4A.txt:423`).

## 13. Rebuild contract

- Agents are a separate binary with **zero** LLM/network/tool deps.
- IPC has **exactly** the 15 named messages, 1+4+postcard, 10MB cap, ThinkStep <64KB.
- Orchestrator maintains `count[agent_id][topic]` over **received ThinkStep frames**.
- On `max_calls`: inject converge; do not trust agent `GrcBudget`.
- At phase start, orchestrator count 0 beats a nonzero agent local count.
- `GroundingRequest` executed in the orchestrator; WEB_CHECK never in the agent.
- Role-method matrix as §3; GroundingRequest billed to the topic; reject if no ThinkStep budget entry exists for that topic.
- `AgentDispatchConfig` is the spawn authority (including GRC 3/600 feasibility override).
- 10k ThinkStep burst drops excess without OOM.
- 2C engine may still run **inside** the agent as a helper; it is not the accountant.

A process that still asks `engine.get_budget()` to decide whether the orchestrator will accept the next step is a cousin. In-process 2C (Phases 0–3) is the **pre-4A** organ, not a substitute for BLOCK-4.

## 14. Non-goals / banned designs

- Agent-side budget as authority after 4A (`phase-4A.txt:26-27`, `phase-2C.txt:373-380`).
- LLM client / `reqwest` / `providers` in `elai_cc_agent` (BLOCK-3) (`phase-4A.txt:22-23`).
- Agent performing WEB_CHECK on its own network (`phase-4A.txt:170-171`).
- GroundingRequest on a topic with no ThinkStep budget entry (`phase-4A.txt:179-180`).
- Allocating `u32::MAX` frame from the length prefix (`phase-4A.txt:201-202`).
- Counting agent-local `think()` returns instead of IPC ThinkStep messages.
- Role-only spawn that ignores `AgentDispatchConfig.grc_config` (`phase-4A.txt:71-74`).
- Letting 2C `allowed_methods` be the **only** check once IPC exists (need both policy and capability) (`phase-2C.txt:79-81`).
- Trusting agent self-reported isolation or budget (`phase-4A.txt:132-133` analog).

Out of **this** file’s organ (specified in 4A but not BLOCK-4): seccomp profiles, AppContainer, self-mod locks, auditor PROPOSED gate — neighbors; do not omit them from a full 4A rebuild, but they are not the GRC-budget organ.

## 15. Source citations

Primary: `docs/specs/guide/steps/phase-4A.txt` L26-27, L52-74, L159-180, L207, L264-278, L402-407, L423. Index: `phase-index-MASTER.txt:688-717`, `:1024`. Overview: `ELAI-SPEC-OVERVIEW.md:103-105`. Feature map: `ELAI-FEATURE-MAP.md:747-753`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:140-146`. Exhaustive: P4A-E007, E014, E015, E026, E027, E029, E046. 2C demotion: `phase-2C.txt:373-380`. 2B wrap: `phase-2B.txt:25-27`, `:141-143`. Audit: `audit-implementation/phase-4A.md` (BLOCK-4 coded, production-unenforced).

## 16. Gaps

- Full `GroundingRequest` / `GroundingResult` / `ThinkStep` **wire field lists** (4A names methods+claim+file_path only). reasoning.txt L137-152 cited (`phase-4A.txt:406`) but not copied into the phase file.
- Whether IPC `ThinkStep` is the same type as `reasoning::ThinkStep` (`phase-4A.txt:55-59` vs `phase-2A.txt:77-79`).
- Whether API_CHECK / API_REGISTRY_CHECK become live in 4A or stay stubbed (2B deferral vs 4A matrix omission).
- Exact bytes of the “convergence signal” message (new field on AgentRequest vs a ThinkStep Converge injection) — **UNCERTAIN** (`phase-4A.txt:161`).
- How GroundingRequest budget accounting combines with ThinkStep (1:1 increment vs shared max_calls) — specified only as “counts toward per-topic budget” (`phase-4A.txt:178`).
- Designer / researcher / media-validator rows in the 4A method matrix: not listed (`phase-4A.txt:174-177`).
