# Tool use + structured output reliability for ELAI

## TL;DR
- Swap ELAI's diff edit-format for Aider-style SEARCH/REPLACE with 99%+ format adherence target (Claude-3.5-Sonnet hits 99.2% correct format / 84.2% pass on Aider leaderboard; Qwen2.5-Coder-7B only hits 57.9% pass on `whole` and is the weakest acceptable Aider-bench fit on diff). Search-replace beats unified-diff for both proprietary and open models on Diff-XYZ (arxiv:2510.12487), but is brittle to delimiter collisions — add a delimiter-escape pre-check before sending to the model.
- Replace any in-house JSON-mode/regex constrainer with XGrammar as a Tier-2 structured-output backend: <40 µs/token JSON, 97.1% schema-accurate on nested GitHub-issue schemas vs Outlines 76.4% with Qwen-2.5-32B (arxiv:2411.15100). XGrammar is now the default in vLLM, SGLang, and TensorRT-LLM (March 2026).
- For Qwen2.5-Coder-7B specifically (ELAI's local sidecar), enforce `<tool_call>...</tool_call>` Hermes-format wrapping + XGrammar schema mask: without it Qwen2.5-Coder silently emits ```json blocks that bypass parsers, and the documented `}}}` JSON-in-JSON escape bug surfaces on nested args. Qwen2.5 chat template is Hermes-compatible — no fine-tune needed.
- Add a CodeAct-style code-execution channel (arxiv:2402.01030) alongside JSON tool calls: +20% success rate vs JSON-only on 17-LLM benchmark; lets the model compose multiple ELAI tools in one call (e.g. read+grep+edit) instead of round-tripping the orchestrator 3 times via IPC.
- Cap recovery to ONE structured retry with the error message embedded; Claude-Sonnet-4 recovers >50% on tool-call errors, Qwen3-8B only ~20% (arxiv:2509.18847). Beyond one retry, escalate — don't loop. ELAI's current "agent retries with same prompt" pattern matches the documented anti-pattern.

## Key sources
URL or arxiv | what | takeaway
- https://gorilla.cs.berkeley.edu/leaderboard.html | BFCL v3/v4 board | GLM-4.5 0.778 leads BFCL-v3 multi-turn; Qwen3-Next-80B 0.720; Qwen2.5-Coder not on v3.
- https://llm-stats.com/benchmarks/bfcl-v3 | BFCL-v3 mirror | avg 0.699 across 18 evaluated models; multi-turn is the discriminator.
- arxiv:2411.15100 (XGrammar) | constrained decoding engine | <40µs/token JSON, 3.5x faster than baselines on JSON-schema, 10x on CFG, 97.1% accuracy vs Outlines 76.4% on Qwen-2.5-32B nested schemas.
- arxiv:2501.10868 (JSONSchemaBench, v3 2025) | 10K real schemas, 6 engines | Guidance wins overall efficiency+coverage; Outlines lowest compliance rate due to 40s–10min compile timeouts on complex schemas; XGrammar erratic on new complex schemas (CPU mask-gen bottleneck).
- arxiv:2402.01030 (CodeAct, ICML 2024) | code-as-action vs JSON-tool | +20% success across 17 LLMs on API-Bank, lets models compose tools in one Python action.
- arxiv:2510.12487 (Diff-XYZ, JetBrains) | diff-format ablation | search-replace > udiff for larger models; small open models gain little from format choice; udiff context lines disorient models.
- https://aider.chat/docs/leaderboards/edit.html | Aider edit-format board | Claude-3.5-Sonnet (diff) 84.2%/99.2% format; DeepSeek-Coder-V2 (diff) 72.9%/97.7%; Qwen2.5-Coder-7B (whole) 57.9%/100% — diff format degrades small open models.
- https://aider.chat/docs/unified-diffs.html | udiff ablation on GPT-4 Turbo | SEARCH/REPLACE 20% baseline → unified-diff 61% on GPT-4-Turbo, "3x less lazy"; cuts placeholder/elision behavior.
- arxiv:2406.10279 (USENIX Sec 2025 package hallucination) | 576K samples, 16 LLMs | 19.7% recommended packages don't exist; 58% repeatable across 10 runs; 38% conflations, 13% typos, 51% fabrications — exact threat ELAI's anti-slopsquatting rule mitigates.
- arxiv:2509.18847 (Structured Reflection 2025) | tool-call error recovery | Claude-Sonnet-4 >50% recovery, Qwen3-8B ~20%; explicit reflection step beats blind retry.
- arxiv:2505.10570 (LongFuncEval) | long-context tool use | 7–85% drop as tool count grows; 13–40% drop as multi-turn lengthens; 7–91% drop as tool-response length grows.
- arxiv:2509.23206 (PARL-MT) | multi-turn progress awareness | LLMs re-invoke same function / drop params under long horizons; progress-awareness training fixes it.
- arxiv:2511.01527 (TPS-Bench) | multi-step tool scheduling | open models fail to parallelize independent subtasks; only frontier models pick correct schedule.
- https://qwen.readthedocs.io/en/latest/framework/function_call.html | Qwen tool-call protocol | Hermes-format `<tool_call>` tags; Qwen2.5 chat template Hermes-compatible.
- https://github.com/hanXen/vllm-qwen2.5-coder-tool-parser | Qwen2.5-Coder parser | documents silent ```json fallback failure mode; `<tools>` tag fixes parser.
- https://blog.squeezebits.com/guided-decoding-performance-vllm-sglang | vLLM/SGLang guided decoding 2025 | XGrammar+SGLang mask-gen overlaps GPU compute, near-zero overhead vs sequential.
- https://github.com/djkoloski/rust_serialization_benchmark | Rust serde bench | Cap'n Proto 32ns serialize / 14000 MB/s; postcard 480µs; msgpack ~660µs/5ms — for ELAI's 15-type IPC, Cap'n Proto wins on throughput, postcard wins on simplicity+size.
- https://medium.com/data-science-collective/stop-trusting-your-agent-with-tool-arguments-dbe45fe158ad | Pydantic+LangGraph validation pattern | schema validation + pre-execution guards + one-shot self-repair, production pattern.
- arxiv:2603.11495 (Try-Check-Retry divide-and-conquer) | long-context tool calling | training-free pipeline beats single-shot retry on long-context tool fail recovery.

## BFCL v2/v3 landscape

Source: https://llm-stats.com/benchmarks/bfcl-v3 (May 2026 snapshot)

| Model | Overall (BFCL-v3) | Multi-turn-strong | Parallel-strong | Local-feasible on RTX 3090 24GB |
|---|---|---|---|---|
| GLM-4.5 (Zhipu) | 0.778 | yes | yes | no (too big) |
| GLM-4.5-Air | 0.764 | yes | yes | borderline (needs Q4) |
| LongCat-Flash-Thinking (Meituan) | 0.744 | yes | yes | no |
| Qwen3-Next-80B-A3B-Thinking | 0.720 | yes | partial | no (80B) |
| Qwen3-VL-235B-A22B-Thinking | 0.719 | yes | yes | no |
| (avg of 18 evaluated) | 0.699 | mixed | mixed | — |
| Qwen2.5-Coder-7B | not on BFCL-v3 board | — | — | yes (ELAI's current) |

Caveats: only 18 models on the v3 board, all self-reported, no verified runs. Qwen2.5-Coder is on Aider leaderboard but not BFCL-v3. BFCL-v3 multi-turn is the discriminator — single-turn scores cluster much tighter.

LongFuncEval (arxiv:2505.10570) is the more honest long-context tool-use eval — it shows 7–91% degradation that BFCL-v3's 1000 test cases don't capture.

## Constrained decoding comparison

| Tool | Schema expressiveness | Speed (per-token overhead) | Model compat (Qwen2.5-Coder-7B specifically) | Tokens-emitted overhead | Citation |
|---|---|---|---|---|---|
| XGrammar | full CFG incl recursion; JSON-schema; XML; Python DSL | <40µs JSON, <200µs CFG; near-zero on warm schemas; CPU-bottleneck stalls on unseen complex schemas | works (vLLM/SGLang default backend, March 2026); needs Qwen tokenizer registered | none (token-level mask) | arxiv:2411.15100; jsonschemabench |
| Outlines (dottxt) | regex+JSON-schema (FSM-based); no native recursion (flattens) | compile-time 40s-10min on complex schemas; runtime fast once compiled | works via transformers/llama.cpp adapters | none | arxiv:2501.10868 |
| LM-Format-Enforcer | JSON-schema, regex, allows model to choose whitespace/field order | moderate; scans next-token candidates per step | works (llama-cpp-python, vLLM, TGI) | none, but freer-form output | github.com/noamgat/lm-format-enforcer |
| Llama.cpp GBNF | BNF grammar; partial JSON-schema via `json_schema_to_grammar.py` | low overhead in llama.cpp pipeline | works (Qwen2.5-Coder runs on llama.cpp); but ELAI sidecar uses transformers, not llama.cpp | none | github.com/ggml-org/llama.cpp |
| Microsoft Guidance | full CFG + procedural | wins overall on JSONSchemaBench efficiency+coverage | works | none | arxiv:2501.10868 |

For ELAI's stack (Qwen2.5-Coder-7B, transformers + bitsandbytes NF4 on RTX 3090, anthropic-protocol sidecar): XGrammar via transformers `LogitsProcessor` is the cleanest fit — same library used by vLLM/SGLang upstream, recursion-capable for nested tool-args, no Outlines compile-timeout risk. Pair with cold-schema warmup (pre-compile all 15 IPC schemas + the tool schema set at sidecar boot) to dodge the CPU-bottleneck stalls reported in 2026.

Outlines lowest compliance on JSONSchemaBench (arxiv:2501.10868) is a hard rule-out for production despite the 1.8% hallucination win with one-shot prompting — that 1.8% only counts cases that compiled.

## Edit-format winner for code edits

Verified from arxiv:2510.12487 (Diff-XYZ, Oct 2025) and https://aider.chat/docs/leaderboards/edit.html (Apr 2025):

| Format | Larger proprietary models | Open ≤14B (Qwen-class) | Failure mode |
|---|---|---|---|
| SEARCH/REPLACE (Aider diff) | best (Claude-3.5-Sonnet 84.2%/99.2% format) | brittle: DeepSeek-Coder-V2 72.9% > Qwen2.5-Coder-7B with diff (drops to whole-file) | delimiter collision when source contains `<<<<<<` or `=======` |
| Unified diff (udiff) | OK on GPT-4-Turbo (61% vs 20% search-replace, BUT measured laziness reduction not correctness); Diff-XYZ says udiff context lines disorient models | bad — small models lose track of context lines | misaligned context, off-by-one hunks |
| Whole-file rewrite | wasteful tokens | best for small open models (Qwen2.5-Coder-7B 57.9% / Llama-3.1-70B 58.6%, both 100% format) | token cost; risks unrelated overwrites |
| Hunk-rewrite | similar to search-replace | same as search-replace | similar |

Concrete win for ELAI: keep the 3-tier matcher (exact → whitespace-normalized → ellipsis-elision) but switch tier-1 from substring-match to SEARCH/REPLACE-block-match with explicit fence tokens that aren't valid Rust/Python/Svelte syntax (e.g. `<<< ELAI_SEARCH` rather than `<<<<<<<`). Add a delimiter-collision pre-check (arxiv:2604.27296 calls this out as the search-replace failure mode). For Qwen2.5-Coder-7B specifically, ablation suggests falling back to whole-file rewrite on files <200 lines yields higher correctness than diff-format (this matches the Aider leaderboard Qwen scores: Qwen2.5-Coder-7B `whole` 57.9% but unranked on `diff`).

GPT-4-Turbo udiff result is a laziness ablation not a correctness ablation — don't generalize across model families.

## Tool-call failure taxonomy

| # | Failure mode | Symptom in ELAI | Prevention | Citation |
|---|---|---|---|---|
| 1 | Silent format fallback (```json wrap instead of `<tool_call>`) | parser returns nothing, agent retries same prompt | XGrammar mask on `<tool_call>...</tool_call>` envelope at sidecar level; reject candidates without envelope | https://github.com/hanXen/vllm-qwen2.5-coder-tool-parser; qwen.readthedocs.io |
| 2 | JSON-in-JSON escape (`}}}` instead of `}}`) | malformed args, edit fails sequence-match | constrained decoding (CFG-based, XGrammar) — FSM-based can't catch | arxiv:2501.10868 |
| 3 | Hallucinated file path failing visibility filter | "visibility_filtered" error returned instead of NotFound | sidecar receives the agent's path-allowlist as part of GroundingRequest, pre-filters tool schema enum to only allow-listed paths before sending to model | The Elder Scripts self-correction review |
| 4 | Hallucinated package name (slopsquatting) | `cargo add nonexistent-crate` | per CLAUDE.md anti-slopsquatting + pre-execution registry check (npm/pypi/crates.io exists+date check) before Bash/Edit tool fires | arxiv:2406.10279; USENIX Sec 2025 |
| 5 | Edit fails 3-tier sequence match, retries with same prompt | infinite-retry loop; ELAI's documented current state | one-shot retry with error embedded in re-prompt; on second failure escalate to fixer agent (matches ELAI's existing anti-spiral pattern) | arxiv:2509.18847; arxiv:2603.11495 |
| 6 | Parallel-tool over-firing in multi-turn | duplicate ToolRequest for same op | PARL-MT progress-awareness checks: dedupe requests with same (tool, args-hash) within window | arxiv:2509.23206 |
| 7 | Multi-step plan divergence under long context | agent forgets earlier tool output | TPS-Bench-style scheduling: planner emits explicit DAG; executor logs each node-completion as ToolResult+ProgressMarker | arxiv:2511.01527 |
| 8 | Long-context tool-response truncation | answer-retrieval degrades 7–91% | budget tool response per IPC message; spill long results to disk + return reference, not inline | arxiv:2505.10570 |
| 9 | Schema-sanitization mismatch (internal params leaking) | model sees fields it shouldn't, fills hallucinated values | strict allowlist (not denylist) at the Tier-2 trait boundary | github.com/noamgat/lm-format-enforcer pattern |
| 10 | DeepSeek-R1 INSTRUCTOR mode retries unbounded | exhausts grounding budget | cap retries at 1, escalate; mirror Claude-Sonnet-4 50%/Qwen3-8B 20% recovery gap | arxiv:2509.18847 |

## Best techniques to adopt

Ranked by ROI for ELAI's stack (Qwen2.5-Coder-7B sidecar, 15-type IPC, 5-tier auditor, Rust+Python):

1. **XGrammar tier-2 backend** | arxiv:2411.15100, arxiv:2501.10868 | replace current GRAMMAR/INSTRUCTOR mode with XGrammar `LogitsProcessor` in sidecar python/elai_inference/server.py; pre-compile all 15 IPC schemas at boot. Validation tier-1 (determinism) becomes "schema-mask-emitted" not "JSON.parse-succeeded".
2. **SEARCH/REPLACE edit-format with custom fences + delimiter-collision pre-check** | arxiv:2510.12487, aider.chat/docs/unified-diffs.html | rewrite the Edit tool's 3-tier matcher; fence with `<<< ELAI_SEARCH` / `=== ELAI_REPLACE` to dodge git-conflict-token collisions; pre-scan source for fence tokens and fall back to whole-file rewrite if found. Hook: Edit tool, ToolRequest IPC type, validation tier-2 (quality/AST).
3. **Hermes `<tool_call>` envelope enforced via grammar** | qwen.readthedocs.io; github.com/hanXen | wrap Qwen2.5-Coder tool calls in `<tool_call>...</tool_call>` at sidecar grammar level; this is what vLLM upstream does. Hook: InferRequest/InferResponse IPC types.
4. **One-shot retry with error-embedded reprompt** | arxiv:2509.18847, arxiv:2603.11495 | when Tier-2 validation fails, re-prompt ONCE with the structured error as a `<tool_error>` message; after that, escalate to fixer agent (existing ELAI agent). Hook: orchestrator retry loop. Stops the documented "agent retries with same prompt" infinite loop.
5. **CodeAct execution channel as parallel tool path** | arxiv:2402.01030 | add a `code_action` tool that runs Python in a sandboxed bash session and composes other ELAI tools; +20% success rate; cuts orchestrator IPC round-trips. Hook: new ToolRequest variant, BashRequest sandbox, validation tier-3 (unit tests). Big lift — pilot on the indexing+grep+edit composite.
6. **Pre-execution registry verification for package-touching tools** | arxiv:2406.10279 | when Edit/Bash adds a dep (`Cargo.toml`, `pyproject.toml`, `package.json`), call npm/pypi/crates.io API; reject unknown or <30-day-old packages. Hook: Edit + Bash, validation tier-1 (determinism). Implements CLAUDE.md anti-slopsquatting.
7. **Cap'n Proto for IPC hot path, postcard for cold path** | github.com/djkoloski/rust_serialization_benchmark | swap JSON-over-stdio for Cap'n Proto on InferRequest/InferResponse/ThinkStep (high frequency, zero-copy wins big at 14GB/s); keep JSON for AgentRequest/CheckpointSync (debuggability matters more than throughput). Postcard is the middle option if you don't want Cap'n Proto's IDL/build-step.
8. **PARL-MT progress markers in multi-turn** | arxiv:2509.23206, arxiv:2511.01527 | emit a ProgressMarker IPC frame after each ToolResult; orchestrator dedupes (tool, args-hash) within a 5-frame window to kill duplicate fires.
9. **Erlang-OTP-style supervisor tree for agent pool** | zylos.ai/research, Bastion crate | wrap each agent in a "let-it-crash" supervisor; on AgentKill IPC, restart with the last CheckpointSync. ELAI's 15-type IPC already has the primitives — formalize the supervision strategy (one-for-one for orchestrator-of-agents, rest-for-one for fixer-after-auditor).
10. **Long-tool-response spill-to-disk** | arxiv:2505.10570 | budget ToolResult inline payload to N tokens; spill the rest to `workflow/sessions/{id}/spill/{uuid}.txt` and pass reference. Cuts the 7–91% degradation curve.

## Anti-patterns

- Blind retry with same prompt after Tier-2 failure — Qwen3-8B-class models recover ~20% on retry; the loop burns budget without progress (arxiv:2509.18847).
- Unified-diff format for Qwen-class open models — context lines disorient (arxiv:2510.12487, arxiv:2604.27296).
- Outlines on complex/recursive schemas — 40s-10min compile timeouts kill latency budget; lowest compliance rate on JSONSchemaBench (arxiv:2501.10868).
- Returning `visibility_filtered` distinct from NotFound — leaks information AND breaks the agent's mental model; the agent retries instead of giving up (matches the documented ELAI failure mode).
- "JSON mode" without a real grammar (e.g. OpenAI-style instruction-only JSON) — silent ```json fallback documented for Qwen2.5-Coder (github.com/hanXen).
- One serialization format for all 15 IPC types — different reliability/throughput needs; over-uniform encoding costs latency or debuggability.
- Trusting LLM-emitted package names without registry check — 19.7% baseline hallucination rate, 58% repeatable, attackable (arxiv:2406.10279).
- Letting multi-turn context grow unbounded past ~100K — agentic capability halves even on 1M-context models (arxiv:2512.02445 quoted in LongFuncEval discussion).

## Open questions

1. Quantify Qwen2.5-Coder-7B-NF4 BFCL-style multi-turn score on ELAI's actual 15-type IPC tool set — no public number exists. Target: ≥0.50 multi-turn with XGrammar+Hermes wrapping.
2. Measure edit-format pass rate ablation on Qwen2.5-Coder-7B-NF4: SEARCH/REPLACE vs whole-file vs hunk-rewrite, on ELAI's actual `.elai_cc/crates` edit set. Hypothesis from Diff-XYZ: whole-file wins below ~13B model size; need to confirm at 7B-NF4.
3. Token-overhead measurement for adding `<tool_call>` envelope + XGrammar mask on the Qwen2.5-Coder-7B-NF4 path: does the documented <40µs/token JSON figure hold under 4-bit quant + bitsandbytes, or does CPU mask-gen dominate on smaller-model pipelines (the 2026 erratic-behavior note)?
4. Does CodeAct's +20% generalize to 7B-class open models? The arxiv:2402.01030 study averaged across 17 LLMs — open-7B class was a minority. Worth a small ablation before the full integration lift.
5. Cap'n Proto vs postcard end-to-end IPC latency on ELAI's sidecar tunnel (192.0.2.10:6465 → 127.0.0.1:8100): does zero-copy win materialize over a TCP-tunneled WSL2 link, or does network dominate?
