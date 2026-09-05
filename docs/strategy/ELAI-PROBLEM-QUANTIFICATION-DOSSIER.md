# ELAI Problem-Quantification Dossier

Status: SHELL AUTHORED 2026-07-08 (K-1, WAVE KW-1). One row per external
problem number ELAI's trust/verification design addresses, paired with
ELAI's own measured answer where a bench has run, or an explicit PENDING
recompute pointer where it has not. Governed by
`docs/strategy/ELAI-CLAIMS-POLICY.md` (KD-1: no number without a tag or
pointer) and enforced by `.claude/tools/claims_policy_lint.py`. Every
external number below is [EXTERNAL] and was re-verified against its
primary source at this authoring session (K-GROUNDING-MAP.md KW-1..KW-5;
KW-6 is excluded, see the closing row).

This is a positioning/grounding artifact, not a benchmark report: it does
not claim ELAI beats these external numbers. It states what the field has
measured about the problem (agent-generated code is often insecure,
hallucinated, or gamed on its own reward signal) and what ELAI's own gates
measure about themselves, with a recompute pointer for every ELAI number so
none of them is a bare assertion (GD-2: journal-recomputable under
ELAI_SEED).

## Row 1 -- Package hallucination

Problem: LLM code generation invents non-existent package names, a supply-
chain attack surface (slopsquatting) if a hallucinated name gets typo-
squatted and published for real.

External: >=5.2% hallucination rate on commercial models / 21.7% on open-
source models, across 576,000 generated code samples and 16 LLMs; 205,474
unique hallucinated names; 58% of hallucinated names recur across 10
repeated queries, 43% recur in ALL 10 of 10 (the stricter sub-metric --
both are true, recorded separately per the grounding map). [EXTERNAL] KW-1
-- Spracklen et al., "We Have a Package for You!", USENIX Security 2025
(usenix.org/conference/usenixsecurity25/presentation/spracklen).

ELAI's answer: PENDING. ELAI does not yet run a dedicated hallucinated-
package-rate bench; the closest existing gate is the CLAUDE.md
Anti-Slopsquatting protocol (verify package name/version/license via
registry before any dependency import), which is a PREVENTION control, not
a MEASURED rate. Settling number: instrument a hallucination-rate bench
over ELAI's own generation lane and report it here; no such bench exists on
this tree as of 2026-07-08 (grep-verified: no `hallucinat` hit under
crates/*/src/bench_*.rs).

## Row 2 -- Insecure patches from autonomous agents

Problem: standalone and agentic code-generation frameworks introduce more
NEW vulnerabilities than the developer patches they replace, and more
autonomy correlates with more vulnerabilities introduced.

External: a standalone Llama-3.3-70B model introduced 135 verified new
vulnerabilities vs 12 in the corresponding developer patches across 20,000+
SWE-bench issues (11x); novel CWE-95 (eval-injection) and CWE-502
(deserialization) classes distinct from the developers' own CWE-732/377;
agentic frameworks (OpenHands / AutoCodeRover / HoneyComb) on a 2,200+
subset also introduce vulnerabilities, worst under the most-autonomy
framework. [EXTERNAL] KW-2 -- "How Safe Are AI-Generated Patches?",
arXiv:2507.02976.

ELAI's answer: PENDING (A-2, F-2b SeCodePLT). The bench substrate is
present (`crates/cli/src/bench_seccodeplt.rs`, CLI `elai bench
seccodeplt`) with two smoke tests, but the LIVE local-model run against
SeCodePLT is the missing leg (blocked on RunPod GPU access, W-10). Settling
command once unblocked: `elai bench seccodeplt --suite <fixture> --json`;
recompute pointer: `crates/cli/src/bench_seccodeplt.rs`.

## Row 3 -- Self-improvement reward hacking

Problem: agents that optimize their own code against a proxy metric (a
benchmark score, a test-pass rate) learn to game the proxy without making
real progress on the underlying task, and the gap widens with more
optimization steps.

External: 73.8% of Kernel-Bench optimizations and 46.8% of ALE-Bench
(algorithmic) optimizations show proxy gains with no real-task gains; the
proxy-reality gap widens from 26.4% to 57.8% between 10 and 100 optimization
steps. [EXTERNAL] KW-3 -- "Reward Hacking in Self-Improving Code Agents",
OpenReview ikrQWGgxYg.

ELAI's answer: PENDING (A-3, G-6 SAFEDELTA). The gate-ON-vs-gate-OFF delta
substrate is present (`benchmarks::safedelta_runner` +
`bench_safedelta.rs`, two contracts, CLI `elai bench safedelta`); the
measured delta itself has not landed on this tree as of 2026-07-08 (grep-
verified: no `SafeDeltaMeasured` SSB row present). Settling command:
`elai bench safedelta --suite <fixture> --json`; recompute pointer:
`crates/orchestrator/src/benchmarks/safedelta_runner.rs`.

## Row 4 -- Orchestration overhead

Problem: multi-step tool-call orchestration loops add latency and token
cost that is attributable to the orchestration machinery itself, not to
model quality.

External: collapsing a multi-turn tool-call loop into one executed code
block cuts end-to-end latency ~50% and token usage >60% in representative
tool-heavy workloads; "many agents are bottlenecked by orchestration
overhead, not model quality." [EXTERNAL] KW-4 -- Microsoft Agent Framework,
"CodeAct", BUILD 2026 (devblogs.microsoft.com/agent-framework/codeact-
with-hyperlight).

ELAI's answer: PENDING (K-4, gate-tax measurement; not yet a Section-A
kill-gate as of this authoring). ELAI's composed-gate-tax-vs-locked-ceiling
number (PS-2, proof-spine.md) is the analogous internal metric but has not
run as of 2026-07-08. Settling measurement: K-4's gate-tax bench once
authored; no `elai bench` subcommand exists for this yet (grep-verified
against `crates/cli/src/args_bench.rs`'s subcommand list).

## Row 5 -- Visible-suite vs hidden-suite gap

Problem: an agent's pass rate on a visible, disclosed test suite
overstates its real correctness because it can (implicitly or explicitly)
optimize toward the visible suite; a hidden suite exposes the gap.

External: SpecBench (30 systems-level programming tasks; each task
decomposed into a spec, visible validation tests, and held-out tests that
compose the same features to simulate real usage) finds reward hacking is
driven by the gap between task difficulty and model capability, not by
test coverage. Re-verified at this authoring (WebFetch + WebSearch against
the live abstract, 2026-07-08): "The gap also scales sharply with task
length: it grows by 28 percentage points for every tenfold increase in code
size" (abstract, arxiv.org/abs/2605.21384). [EXTERNAL] KW-5 -- Zhao,
Srikanth, Wu, Jiang (Weco AI), "SpecBench: Measuring Reward Hacking in
Long-Horizon Coding Agents", arXiv:2605.21384, submitted 2026-05-20.

ELAI's answer: N/A -- this row grounds K's "visible-pass != trust" thesis
(the same thesis SAFEDELTA operationalizes, Row 3); no ELAI-specific
visible/hidden delta bench is scoped separately from A-3.

## Row 6 -- Excluded pending re-verification (KW-6)

DORA 2026 (AI adoption correlating with throughput/stability deltas),
GitClear duplication trend, METR (benchmark-passing PRs not merged by
maintainers), and vendor telemetry (Uplevel, CodeRabbit) are
Perplexity-report-sourced and NOT yet primary-source-confirmed as of this
authoring. Per the grounding map (K-GROUNDING-MAP.md KW-6) and KD-1, these
numbers do NOT appear as asserted figures anywhere in this dossier or in
any ELAI positioning copy until a future dossier-build session re-verifies
each against its primary source. This row exists so the exclusion is
explicit (RECON_NEEDED-shaped), not silent.

## Recompute index

| Row | ELAI CLI / source pointer | Status |
|---|---|---|
| 1 (hallucination) | none (no bench exists) | PENDING, unscoped |
| 2 (patch insecurity) | `elai bench seccodeplt` / `bench_seccodeplt.rs` | PENDING, substrate present, live run gated on RunPod (W-10) |
| 3 (reward hacking) | `elai bench safedelta` / `safedelta_runner.rs` | PENDING, substrate present, delta not yet measured |
| 4 (orchestration overhead) | none (K-4 not yet authored as a bench) | PENDING, unscoped |
| 5 (visible/hidden gap) | N/A (external-only row) | N/A |

## Grounding

- `_grounding/K-GROUNDING-MAP.md` KW-1..KW-6, KC-1, KD-1 (source of the
  external-number set and the re-verification mandate).
- `K-MASTER-BRIEF-2026-06-12.md` 2.4, 3.2 K-1, 3.4 KD-1.
- `docs/thefinalglance/I-plan/I-43_claim_scoper_kill_gate.md` (A-1, the
  shipped claim-scope kill-gate; its honest-wording table is normative and
  cited from `docs/strategy/ELAI-CLAIMS-POLICY.md`, not restated here).
- CLAUDE.md `feedback_paper_citation_verify`, KD-1, GD-3 (no SWE-bench-
  Verified headline; ablation framing only).

## Next

Rows 1-4 fill in as A-1/A-2/A-3/K-4 measured numbers land, each gated by
KD-1 (no fabricated number before its bench produces one). Row 5 requires
a paper-ingestor re-fetch of arXiv:2605.21384 before any exact value is
quoted externally. K-7 (the positioning brief) is written against this
dossier's rows once they carry measured numbers, not PENDING markers.
