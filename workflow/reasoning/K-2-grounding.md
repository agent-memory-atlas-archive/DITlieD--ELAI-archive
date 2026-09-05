K-2 routing-economics — grounding log (K-26 MOVE 6a refine-before-execute, 2026-07-11)

Refined by: K-26 controller session (opus 4.8 1m). Plan-gate: was FAIL 8/9 (legacy, no wargame blocks) -> PASS 9/9 after rewrite. Independent adversarial-auditor mode=plan pass deferred to MOVE 6b pre-build (D-6: author is not the final red-teamer).

Claim | Verdict | Evidence
J-28 token ledger NOT shipped, live cost leg blocked (KC-3) | VERIFIED-STALE (now false) | UsageTurnEvent present in ssb/action_type.rs + payloads.rs + schema.rs at schema_version 272; committed fe069d0 (K-26 MOVE 1). The KC-3 blocker is discharged at the capture layer; live-arm gate is now W-10 (RunPod/remote-sidecar infra), not J-28.
bench dispatcher at args_bench.rs:75 | VERIFIED-CORRECTED | real chain at args_bench.rs:52-93 (`if sub == "X"`); gate-tax arm @ :64, case-reuse @ :61, safedelta @ :58, qdelta @ :79; error list @ :93. K-2's ":75" was stale.
SSB "live 236/545" | VERIFIED-CORRECTED | current flat-beacon max = 272 (RecurrenceHitRateMeasured 270, GateTaxMeasured 271, UsageTurnEvent 272). RoutingEconomicsMeasured -> 273 (RECON_NEEDED-1: re-grep at edit).
A-6 CloudModeRouter::route + RuntimeRoutingPolicy | VERIFIED | cloud_mode.rs:265 `struct CloudModeRouter`, :284 `pub fn route(...)`, uses providers::runtime::RuntimeRoutingPolicy::evaluate. The routed substrate exists; the two arms differ only in the policy/context input (RECON_NEEDED-3 confirms the exact field at edit).
safedelta_runner / bench_qdelta runner shape | VERIFIED | parse_bench_safedelta_args @ args_bench.rs:902 + benchmarks/safedelta_runner.rs; parse_bench_qdelta_args @ :1199. gate_tax_runner.rs is the freshest flat-beacon + RunnerStatus{Measured|RunnerPresent} precedent to mirror (read this session).
flat-beacon + emit_beacon_and_rra recompute (GD-2) | VERIFIED | GateTaxMeasured/RecurrenceHitRateMeasured are flat action_types (one sql_str, no Kind) emitted via emit_beacon_and_rra in bench_gate_tax/bench_case_reuse; K-2 mirrors exactly.

UNGROUNDED / upgraded to RECON_NEEDED at edit: RECON_NEEDED-1 (SSB max at edit), -2 (dispatcher line + error list), -3 (route signature force-cloud-vs-hybrid input), -4 (UsageTurnEvent field names for normalized per-lane cost). Each carries its exact settling grep/read in the plan.

Scope note (K-26 MOVE 6 ON_FAIL): this build ships the RUNNER + the deterministic FIXTURE arm + the SSB beacon; the LIVE $ arm returns 'cost unavailable' (RunnerPresent) because no live cloud+local lane is funded in the K-26 window (W-10). RunnerPresent is the honest verdict; a fabricated live cost is R1.
