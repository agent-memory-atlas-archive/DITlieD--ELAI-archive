# Verification Report: phase-1F.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-1F.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 1F is an evaluation phase (not a feature phase) whose RESOLVED note records a DROP
decision for speculative decoding dated 2026-04-07, with steps 1F.3-1F.5 explicitly
skipped. Steps 1F.1 (SpeculativeConfig skeleton) and 1F.2 (probe + documented decision)
are fully implemented and match their acceptance criteria. The only gaps are the metrics
TOML files required by steps 1F.3-1F.5 — these steps were intentionally skipped after
the NOT_AVAILABLE determination, but the spec still lists their TOML outputs as EXIT
CRITERIA. The decision document exists with all required content.

## Deliverables verified DONE

- SpeculativeConfig struct with speculative_decoding: bool, draft_model: Option<String>, num_draft_tokens: u8 — evidence: .elai_cc/crates/providers/src/providers/ollama.rs:254-259
- SpeculativeConfig serde (Serialize, Deserialize derives) — evidence: ollama.rs:254
- SpeculativeConfig default values (false, None, 5) — evidence: ollama.rs:261-268
- SpeculativeConfig::new() validates num_draft_tokens >= 1, returns Err for 0 — evidence: ollama.rs:271-285
- OllamaConfig has nested SpeculativeConfig field — evidence: ollama.rs:288-302
- OllamaConfigBuilder supports speculative() setter — evidence: ollama.rs:331-333
- Unit test: test_speculative_config_defaults — evidence: ollama.rs:715-720
- Unit test: test_speculative_config_serde_roundtrip — evidence: ollama.rs:722-738
- Unit test: test_speculative_config_rejects_zero_draft_tokens — evidence: ollama.rs:740-748
- ProbeResult enum (Supported | NotSupported) with Display "SUPPORTED:/NOT_SUPPORTED:" prefix — evidence: ollama.rs:17-53
- ProbeResult serde support (Serialize, Deserialize) — evidence: ollama.rs:17
- probe_speculative_support() async fn — checks version, baseline vs draft API calls, detects silent ignore — evidence: ollama.rs:102-237
- Probe anti-false-positive logic: compares response keys (draft-specific keys list) — evidence: ollama.rs:170-192
- Unit test: test_probe_result_not_supported_display — evidence: ollama.rs:753-762
- Unit test: test_probe_result_supported_display — evidence: ollama.rs:765-776
- Unit test: test_probe_result_serde_roundtrip_not_supported — evidence: ollama.rs:779-787
- Unit test: test_probe_result_serde_roundtrip_supported — evidence: ollama.rs:789-799
- Integration test (ignored): test_probe_local_ollama — evidence: ollama.rs:824-853
- Binary NOT_AVAILABLE determination with ollama v0.20.2 evidence — evidence: workflow/reasoning/phase-1F-decisions.md:8-23
- Fallback evaluation documented (llama.cpp not available, vLLM deferred to Phase 2) — evidence: workflow/reasoning/phase-1F-decisions.md:28-33
- DROP decision documented with date and rationale — evidence: workflow/reasoning/phase-1F-decisions.md:38-40
- VRAM budget note (headroom reclaimed for embedding models) — evidence: workflow/reasoning/phase-1F-decisions.md:40
- Decision document path correct: workflow/reasoning/phase-1F-decisions.md — evidence: file exists at that path

## Deliverables PARTIAL

- 1F.2 ACCEPTANCE: "API parameter names recorded for use in step 1F.3" if AVAILABLE
  - exists: phase-1F-decisions.md records ollama_version=0.20.2 and "no speculative decoding API parameters"
  - gap: N/A — spec says "If NOT_AVAILABLE: documented fallback decision" which IS present; AVAILABLE branch is moot given DROP. No true gap.

## Deliverables MISSING

- .elai_cc/metrics/phase_1F_correctness.toml — searched: metrics/ glob, phase_1F pattern; no evidence. Note: step 1F.3 was explicitly skipped per DROP decision at 1F.2.
- .elai_cc/metrics/phase_1F_throughput.toml — searched: metrics/ glob; no evidence. Step 1F.4 skipped.
- .elai_cc/metrics/phase_1F_vram.toml — searched: metrics/ glob; no evidence. Step 1F.5 skipped.
- .elai_cc/metrics/phase_1F_prompt_lookup.toml — searched: metrics/ glob; no evidence. Step 1F.3b: spec says document as "UNAVAILABLE" in decision doc if tool support absent; decision doc does not contain a "UNAVAILABLE" note for prompt lookup specifically. The decision doc terminates at step 1F.2 without addressing 1F.3b.

## Deliverables UNVERIFIABLE

- "Phase 0 + 1A-1C tests still pass (no regression)" — the report cannot run tests; this is a runtime assertion. The spec lists it as an EXIT CRITERION but cannot be confirmed from static analysis.
- ollama v0.20.2 live probe result (PR #8134 confirmed closed) — the decision doc records this as a live measurement; verifying the live probe output is outside static analysis scope.

## What must be done (action list)

- [ ] Add prompt lookup decoding "UNAVAILABLE" note to workflow/reasoning/phase-1F-decisions.md (step 1F.3b requires documenting availability even if skipped after NOT_AVAILABLE at 1F.2). Target: phase-1F-decisions.md, add "Prompt Lookup Decoding (1F.3b): UNAVAILABLE — not evaluated because phase terminated at 1F.2 with NOT_AVAILABLE. Re-evaluate in Phase 2 alongside vLLM."
- [ ] Confirm cargo test -p elai-providers passes (9 unit tests for 1F.1 + 1F.2) to satisfy the "Existing tests still pass" exit criterion. (Auditor action, not main model.)
- [ ] The four metrics TOMLs (phase_1F_correctness.toml, phase_1F_throughput.toml, phase_1F_vram.toml, phase_1F_prompt_lookup.toml) are absent — this is INTENTIONAL per the DROP decision. If the spec is considered final, add a stub .elai_cc/metrics/phase_1F_drop.toml recording the DROP outcome so the EXIT CRITERIA "All measurements in .elai_cc/metrics/phase_1F_*.toml" is satisfied with a canonical artifact.

## Notes
The RESOLVED block in phase-1F.txt (L41-50) explicitly records that steps 1F.3-1F.5 were
skipped at commit eeb9a63 dated 2026-04-07. The implementation in ollama.rs matches what
that commit describes: ProbeResult enum, probe_speculative_support(), 9 unit tests. The
only true gap is the 1F.3b prompt lookup branch and the missing metrics stub for the DROP
outcome. The decision document is substantively complete; adding a single sentence for
prompt lookup and optionally a stub TOML would close the phase per its own EXIT CRITERIA.
