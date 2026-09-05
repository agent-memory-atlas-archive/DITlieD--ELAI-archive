# J-82: Deterministic token-efficient emission — TOON tool-output + dictionary-log lane + protected-span tokenizer

STATUS: PLANNED (WAVE KAT-2; M; NEW (measure-first); composes J-24/H-4/mcp_budget — deterministic, non-generative token reduction on NON-critical text only)

Objective: ELAI has NO systematic token-efficient OUTPUT format for its tools — tool returns are serde-JSON / ad-hoc compact strings (e.g. context_budget.format()'s hand-rolled CONSTRAINTS block), and mcp_budget.rs (crates/plugins/src/mcp_budget.rs, TokenBudget + enforce(), DEFAULT_BUDGET_TOKENS=5000) governs tool-LIST size, not tool-OUTPUT size. J-82 adds three DETERMINISTIC, NON-GENERATIVE, NON-CRITICAL token-reduction legs, each with an ELAI-MEASURED done-gate (NEVER the authors' 40%/3x claims — ID-5/HD-1): (a) a token-efficient OUTPUT format (TOON or an ELAI-measured alternative) for the heaviest tool returns (the elai CLI + IPC tool results + the .claude harness tools the model consumes); composes J-24 CONTEXT-FRUGAL TOOLS / H-4 TOOL SEARCH / mcp_budget. (b) a dictionary-encoding lane (Amazon 2026, the one black-box near-lossless token reducer, exact-match >0.99 on Claude 3.7) for REPETITIVE tool-output ONLY (build logs / test spew / telemetry), NEVER code — and only where dropped volatile values (timestamps/UUIDs/hex) are ALSO archived to the journal (else it violates losslessness). (c) a deterministic protected-span tokenizer (the code-context-engine grammar.py pattern: priority-ordered fence>inline>URL>datetime>version>path>identifier>prose; drops ONLY prose fillers, structured tokens pass BYTE-FOR-BYTE) to harden the I-5 pristine-filter / H-6 masking prose-stripping. HARD BOUNDARY: deterministic-reversible ONLY (IC-3 ciphers banned); NO LLM summarization (HD-2 — busts cache, hallucinates); NEVER on code facts / identifiers / must-preserve atoms (K-12); the done-gate is the ELAI-MEASURED token delta on real tool outputs, never a literature number; OBSERVE/measure before any default-ON (GD-4). Source: workflow/reasoning/steal-kunchen-agentic-tools-2026-06-24.md C2.3 + SYNTHESIS TIER-2(4) + the context-compression-losslessness-2026-06-25.md PART 5/8-R5 repo-half (grammar.py protected-span steal; dictionary-encoding narrow lane; honey-for-devs bench-verified safe-on-code).

Estimated effort: ~1 session (~M). In-process formatting; the only external touch is fetching the TOON grammar spec + npm-verifying any JS dep.
Pre-requisites (re-grep live per W-7): mcp_budget.rs (TokenBudget/enforce/DEFAULT_BUDGET_TOKENS=5000); J-24 CONTEXT-FRUGAL TOOLS (the steering/tool-context legs); H-4 TOOL SEARCH (77K→8.7K tool-def cut); context_budget.format() (the hand-rolled compact block to subsume); I-5 pristine_path_filter.rs + H-6 masking (the prose-strip seam leg (c) hardens); Tier-C IPC tool-result path (the heaviest returns).
Parallel-safe with: J-80, J-81, K-11, K-12 (disjoint files). Composes K-12 (the faithfulness gate guarantees leg-c never drops a must-preserve span).
Source: steal-kunchen C2.3 (axi TOON + the 10 AXI principles; the GitHub-MCP-3x-cost benchmark) + context-compression PART 5 (code-context-engine grammar.py; honey-for-devs precompress.js bench; LoPace = storage-not-token; LLMLingua lossy-on-identifiers). Doctrine: J-0_INDEX.md (W/GD/HD inherited; ADDS no delta family).

## Context
Why this matters: the "MCP is token-expensive" thesis is ALREADY ELAI's (mcp_budget cites the ~55K-token GitHub-MCP / ~80K-token 5-server overhead); the axi benchmark (AXI ~79K vs MCP ~185K tokens/task, AUTHOR-self-reported, ELAI re-measures) is direct external CONFIRMATION + citable grounding for the H-4/H-34/J-24/mcp_budget thread. The STRATEGIC FORK (recorded): axi's answer is "prefer CLI+TOON over MCP"; ELAI's is "make MCP cheap (defer-load H-34 + budget mcp_budget + tool-search H-4)" — ELAI's deep MCP investment (codegraph/gpt-bridge/local-rag/canvas) means it does NOT abandon MCP; the AXI principles instead apply to ELAI's OWN CLI surface + harness tools. leg (c) is the cleanest, safest steal (the deep-research repo-half's verdict): a deterministic protected-span tokenizer is strictly safer than LLMLingua for ELAI (LLMLingua is lossy-on-identifiers + needs a model pass), and it is a cleaner implementation of masking ELAI already does. Honest residue (HD-1/GD-5): honey-for-devs' own bench shows the real-world saving on already-terse internal prompts is MARGINAL (median 0.0% on 266 real prompts; the 16.5% headline was an artificially verbose corpus) — so J-82 scopes to the HEAVIEST tool returns first and measures before defaulting on; a marginal measured delta is a legitimate KEEP-OFF outcome.

## Existing state (B/C/D/E/F/G/H/I/J coverage — W-7 anchors, executor re-greps)
- crates/plugins/src/mcp_budget.rs [WIRED] — TokenBudget + enforce(), DEFAULT_BUDGET_TOKENS=5000; governs tool-LIST size, NOT tool-OUTPUT size (the gap J-82-a fills).
- J-24 NON-PROMPT STEERING + CONTEXT-FRUGAL TOOLS [PLANNED] — the steering + tool-context thread J-82 composes (the 10-principle checklist rider lands here).
- H-4 TOOL SEARCH [PARTIAL] / H-34 MCP DEFER-LOADING [PARTIAL] — the "make MCP cheap" legs (the strategic complement, not replaced).
- context_budget.format() [WIRED] — the hand-rolled compact CONSTRAINTS block (a candidate first consumer of the format).
- I-5 pristine_path_filter.rs [BUILT] + H-6 masking [PARTIAL] — the prose-strip seam leg (c) hardens.
- K-12 handoff-faithfulness gate [PLANNED, this batch] — guarantees leg (c) never drops a must-preserve span.

## Reads (mandatory before coding)
1. _grounding/J-HARNESS-GROUNDING-MAP.md; workflow/reasoning/steal-kunchen-agentic-tools-2026-06-24.md C2.3 + the axi interface map workflow/papers/axi-interface.md; context-compression-losslessness-2026-06-25.md PART 5 (code-context-engine grammar.py; honey-for-devs; LoPace; LLMLingua) + the repo interface maps under workflow/papers/.
2. mcp_budget.rs; context_budget.rs (format()); the Tier-C IPC tool-result path; I-5 pristine_path_filter.rs + H-6 masking.
3. F-0 W-1..W-10; G-0 GD-2/GD-4/GD-5; H-0 HD-1/HD-2; I-0 IC-3 (deterministic-reversible-only); CLAUDE.md (blake3-only, anti-slopsquatting, bun-not-node, no .unwrap on prod).

## Tasks (WAVE KAT-2 — deterministic non-critical token reduction, measure-first)
1. Measurement harness FIRST (~40 LOC): `elai bench tool-output` measures the token delta of each leg on REAL heavy tool returns (deterministic corpus, ELAI tokenizer); this is the done-gate, run BEFORE any default-on (GD-4). No leg ships default-on without a measured positive delta.
2. Leg (a) token-efficient output format (~70 LOC): adopt TOON (anti-slopsquat: toon-go pinned-real v0.0.0-20251202084852; axi-sdk-js ^0.1.7 UNVERIFIED — `npm view` before any JS path; fetch the grammar at toonformat.dev/reference/spec.html, do NOT invent it) OR an ELAI-measured alternative, applied to the heaviest tool returns first; deterministic, reversible to JSON.
3. Leg (b) dictionary-log lane (~50 LOC): collapse repetitive tool-output (build logs/test spew) ONLY, and ONLY where the dropped volatile values are archived to the journal (else it is lossy — REJECT for that stream). NEVER on code.
4. Leg (c) protected-span tokenizer (~60 LOC): priority-ordered fence>inline>URL>datetime>version>path>identifier>prose; structured tokens pass byte-for-byte; drop prose fillers only; harden I-5/H-6 prose-stripping; composes K-12 (a must-preserve span is never dropped).
5. workflow/reasoning/J-82-decisions.md (incl. the measured deltas + any KEEP-OFF verdict).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/token_emit/ (or crates/tools) — the output format (a) + dictionary-log lane (b) + protected-span tokenizer (c)
  - crates/cli — `elai bench tool-output` (the ELAI-measured done-gate)
consumes_symbols:
  - mcp_budget.rs::TokenBudget/enforce (the budget seam — extend to OUTPUT, never re-implement); context_budget.format() (first format consumer)
  - J-24 tool-context legs; H-4/H-34 (the strategic complement); I-5 pristine_path_filter.rs + H-6 masking (leg c seam); K-12 faithfulness gate (must-preserve guard)
reachable_from:
  - the Tier-C IPC tool-result path + the elai CLI heavy-return path → the output format (PRODUCTION tool output)
  - `elai bench tool-output` (PRODUCTION CLI, the measurement gate); the log lane on the build/test-output emit path
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM summarization (HD-2) and NO ciphers/abbrev substitution (IC-3) — deterministic only
  - NEVER on code facts / identifiers / must-preserve atoms (leg c protects them; leg b is logs-only)
  - NO log collapse without journal archival of the dropped volatile values (else lossy)
  - NO literature number as a done-gate (ELAI-measured only, ID-5); NO unverified dep (anti-slopsquat npm view); NO .unwrap()/.expect() on prod
frontend_pair:
  N/A with justification — tool-output formatting is a runtime/harness concern with no dashboard surface; the measured deltas surface via `elai bench tool-output` + the usage panels (J-29). Justified.
observable_effect:
  trigger: "elai bench tool-output --corpus heavy-returns --seed 1234 --json  (the measurement gate); a build-log stream through leg (b); a prose+code blob through leg (c)"
  expect_ssb: "bench rows (reuse a bench family / I-27 ConvergenceCurve-style), net 0..+1 from LIVE ~267/577 (decide at edit, W-9); no new env gate (W-2)"
  expect_artifact: "a measured token delta per leg recorded in J-82-decisions.md; leg (c) output where EVERY fenced/path/identifier/URL span survives BYTE-FOR-BYTE (only prose dropped); NEGATIVE arm: a code span or a must-preserve atom MUST NOT be altered, and a log collapse with un-archived volatiles MUST be rejected"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-82_deterministic_token_efficient_emission.md --run-trigger
rg "protected.span|fence|byte.for.byte|priority.order" .elai_cc/crates/orchestrator/src/token_emit/  # leg (c) present
rg -ci "llm|summari[sz]e|model.*compress|cosine|semantic" .elai_cc/crates/orchestrator/src/token_emit/  # MUST be 0 (deterministic only)
rg "bench tool-output" .elai_cc/crates/cli/                                                  # the ELAI-measured gate exists
test -f workflow/reasoning/J-82-decisions.md
```

## ACCEPTANCE
- [ ] measure-first: no leg ships default-on without a measured positive ELAI token delta (GD-4); a marginal delta is a legitimate KEEP-OFF.
- [ ] IC-3/HD-2: deterministic-reversible only; no LLM, no ciphers.
- [ ] leg (c): every structured span (fence/inline/URL/datetime/version/path/identifier) passes byte-for-byte; only prose dropped.
- [ ] leg (b): log collapse runs ONLY where dropped volatiles are journal-archived; NEVER on code.
- [ ] anti-slopsquat: toon-go pin verified; axi-sdk-js npm-verified before any JS path; TOON grammar fetched, not invented.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator (+plugins). The ONE new dep (if TOON adopted) anti-slopsquat-verified + user-confirmed.

## WIRING DONE-GATE (clause by clause)
- W-1: done = `elai bench tool-output` run-trigger produces measured deltas (not cargo-green).
- W-2: no new ELAI_*_ENABLE; legs default-OFF until measured, then default-ON behind a *_DISABLE kill-switch.
- W-3: real tool-output formatting, not a summary-only port of axi/TOON.
- W-4: frontend_pair N/A (surfaces via bench + J-29 usage); justified.
- W-5/W-10: in-process; only the npm-verify + grammar-fetch touch the network (build-time, not the hot path).
- W-6: N/A; justified.
- W-7: every anchor re-grepped at edit.
- W-9: bench SSB prefer-reuse; any bump from the LIVE baseline.
- HD-1/GD-5: every token number is ELAI-measured per config; the axi/TOON figures are tagged [EXTERNAL]/[DESIGN], never multiplied into a headline; honest KEEP-OFF residue named if the delta is marginal.

## Tests to add
- crates/contracts/tests/j82_token_emit_contract.rs (~LOC): (a) leg (c) preserves every fenced/path/identifier span byte-for-byte across a prose+code blob; (b) the format round-trips to JSON losslessly; (c) leg (b) refuses a log stream whose volatiles are not journal-archived; (d) leg (b) NEVER runs on a code stream; (e) `elai bench tool-output` emits a deterministic delta under ELAI_SEED; (f) a K-12 must-preserve atom is never dropped by any leg.

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/token_emit/{mod.rs, format.rs, log_dict.rs, protected_span.rs} (~180 LOC)
- MODIFY: .elai_cc/crates/plugins/src/mcp_budget.rs (extend the budget seam to OUTPUT) (~20 LOC)
- MODIFY: .elai_cc/crates/cli (`elai bench tool-output`) (~40 LOC)
- CREATE: crates/contracts/tests/j82_token_emit_contract.rs (~80 LOC)
- CREATE: workflow/reasoning/J-82-decisions.md

## Citations / Grounding
- steal-kunchen-agentic-tools-2026-06-24.md C2.3 (the 10 AXI principles; TOON ~40% self-reported; axi GitHub-MCP-3x / 185K-vs-79K benchmark = EXTERNAL CONFIRMATION for H-4/H-34/J-24/mcp_budget, ELAI re-measures ID-5; toon-go pinned-real, axi-sdk-js UNVERIFIED) + SYNTHESIS TIER-2(4).
- context-compression-losslessness-2026-06-25.md PART 5 (code-context-engine grammar.py protected-span = the clean deterministic steal; honey-for-devs precompress.js bench-verified 100%→100% pass, median 0.0% on real prompts = marginal; LoPace = storage-not-token, REJECT as a token lever; LLMLingua = lossy-on-identifiers + needs a model, evaluate-only at most) + PART 8 R5 (dictionary-encoding narrow lane).
- ELAI ground truth: mcp_budget.rs (DEFAULT_BUDGET_TOKENS=5000); context_budget.format(); J-24/H-4/H-34; I-5/H-6 (re-grepped at edit, W-7).
- CLAUDE.md: IC-3 deterministic-reversible-only, HD-2 no-resummarization, anti-slopsquatting (npm view), bun-not-node, blake3-only.

## Open questions / risks
- TOON-adopt vs ELAI-roll: TOON is a real format (toonformat.dev) but axi-sdk-js is unverified and the grammar must be fetched not invented; if the npm/dep path is unclean, roll an ELAI-measured compact format instead. Decide at authoring from the measured delta + the anti-slopsquat result.
- Marginal-delta risk: honey-for-devs shows internal-prompt savings can be ~0%; the heaviest tool returns (codegraph dumps, log spew, large IPC results) are where the win is — scope there first; a KEEP-OFF verdict on light returns is correct, not a failure.
- leg (b) losslessness: the log collapser is lossy on volatile values UNLESS archived; gate it on the journal-archive precondition (composes K-12's record-side losslessness), else REJECT for that stream.

## Next
J-82 gives ELAI a deterministic, measured, non-generative token-reduction surface for its own tool outputs (complementing the "make MCP cheap" H-4/H-34/J-24 thread, not abandoning MCP). Pairs with K-12 (the must-preserve guard) and J-24 (the 10-principle CLI-ergonomics rider). After J-82, the heaviest tool returns cost measurably fewer tokens with zero generative/lossy risk on code.

End of J-82.
