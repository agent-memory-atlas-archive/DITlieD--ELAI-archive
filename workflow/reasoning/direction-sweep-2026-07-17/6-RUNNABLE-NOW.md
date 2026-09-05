RUNNABLE-NOW — THE ORDERED EXECUTION MAP FOR THE PROOF-SPINE NUMBERS
STATUS: COMPLETE (2026-07-18 revision: dependency-ordered WAVES added after the per-row table,
cross-referencing each bench to its provenance plan-cluster status; the per-row table, lane
analysis, the client-exists CORRECTION, and the PS-1 priority answer are preserved unchanged)

Question this file answers: for every row in .elai_cc/benchmarks/registry.json, (a) is it
RUNNABLE-NOW given the compute we actually have and the exact command + lane, and (b) given ITS
provenance plan-cluster's status, is the number READY-TO-MEASURE, PREMATURE (feature cluster not
done), or BLOCKED (runner PLANNED / needs the Tier-2 harness / needs journal / needs J-28)?
The ordered WAVES section at the end is the execution sequence; the per-row table is the evidence.

THE CADENCE PRINCIPLE (why this file is ordered, not just a checklist): a bench being WIRED (Lane 0
fixture-grade) is NOT the same as its feature being SHIPPED. Most benches are ABLATIONS (feature-ON
vs feature-OFF); a number is READY when ITS OWN provenance cluster is done, never when "all plans"
are done ("all plans done" never arrives, letters keep opening: B..Q, P open). Composed numbers
(PS-2 gate-tax, PS-4 $/task) want a stable gate SET and re-measure after any plan that adds/removes
a gate. PS-6 is longitudinal by design. Kill-gate logic: measure the cheap falsifier EARLY (WAVE 1)
so a flat number kills a direction before more plans stack on it (improvement-plan F0.1).
The full prerequisite build order is the sibling file 7-BENCHMARK-READINESS-PLAN.md.

THE HEADLINE FINDING (read this first)
No registry row can be pointed at the live :8181 sidecar or a free API bridge TODAY, because NO
benchmark runner contains an inference client. VERIFIED: `grep -rlE 'reqwest|hyper|ureq' crates/
orchestrator/src/benchmarks/` returns ZERO files. Every one of the 24 wired bench runners computes
DETERMINISTICALLY over an EMBEDDED FIXTURE / curated static corpus / JSONL — none generates code from a
model. The SAFEDELTA Tier-2 "live model-in-loop" arm is a `SafeDeltaTier2Config::pending()` stub whose
only reference to ELAI_E2E_POD_ENDPOINT lives in a `#[ignore]`-gated TEST, not a production path
(safedelta_runner.rs:78-100, 507-516). So "gate:pod" is NOT dischargeable by standing up an endpoint:
the bench-path WIRING (a patch-generation loop + a dynamic exploit-exec scorer) does NOT EXIST YET.
Standing up :8181 alone produces zero numbers.

CORRECTION (2026-07-18, source-verified): the INFERENCE CLIENT itself is NOT missing. ELAI ships a
production Anthropic-protocol client at crates/providers/src/runtime/sidecar.rs (SidecarRuntime: reqwest,
complete() -> /v1/messages, stream(), health_check(); RuntimeAdapter trait) plus the CLI agent loop at
crates/cli/src/runtime_client.rs. What is missing is the bench runners CALLING that layer: the bench
subdir has no client because the Tier-2 arm was stubbed as pending(), not because ELAI cannot drive a
model. Net-new work is the patch-gen LOOP + exploit scorer + hardened corpus that REUSE the existing
client, not a client from scratch. NOTE ALSO: SidecarRuntime's DEFAULT_SIDECAR_URL is still the dead
:8100 (sidecar.rs:44); the live lane is :8181, so even the existing client's default points at the
retired port (mid-migration per commit 8bc668b).

WHAT THIS MEANS: the fixture benches RUN NOW and produce fixture / RunnerPresent / static-gate numbers
(most already on disk); the LIVE model-in-loop HEADLINE numbers (PS-1 live, qdelta live, PS-4 live cost)
are BLOCKED on BUILDING a Tier-2 model-in-loop harness FIRST, then a credible model — not on GPU access.

COMPUTE LANES (verified this session, 2026-07-18)

LANE 1 — LOCAL SIDECAR :8181. CONFIRMED LIVE (curl http://127.0.0.1:8181/v1/models twice this session
-> {"status":"ok"}, model qwen3.5-4b-q4km, llama.cpp, capabilities:["completion"], n_ctx 32768). Legacy
:8100 DEAD (curl refused). USABILITY FOR BENCHES: NONE today — no bench runner has an HTTP client to
call it. The ONLY registry row that would exercise it live is `local-lane-e2e` (kind e2e-functional),
whose runner is PLANNED (unbuilt). CAPABILITY CAVEAT: 4B model — fine for mechanical/e2e-functional
smoke, too weak to be a CREDIBLE generator for a safety/quality/hallucination headline (SUSVIBES's gap
was measured on Claude 4 Sonnet; ELAI's own baseline had a 100% code-API-hallucination rate on the small
model; a 7B caps ~23% SWE vs 32B ~37%).

LANE 2 — FREE API BRIDGES (gpt-bridge MCP: gpt_chat / grok_chat / gemini_chat / claude_chat /
perplexity_chat). These exist as CHAT-LANE MCP tools (CLAUDE.md:39). USABILITY FOR BENCHES: NONE today —
no bench runner selects or calls a bridge; there is no --model/--endpoint flag on any bench subcommand
that routes to a bridge, and no env the bench path reads for a model endpoint (the only env the runners
read is ELAI_SEED). A bench becomes free-lane-runnable ONLY after a Tier-2 harness is built that speaks
to one of these. VERIFIED: grep for grounding_client/runtime_client/anthropic in benchmarks/ = ZERO.

LANE 3 — NEMOTRON ULTRA (1M context), owner-named. UNVERIFIED / NOT CONFIGURED. VERIFIED: grep -ni
'nemotron' across CLAUDE.md and /home/user/teikoku/CLAUDE.md = ZERO hits. No API
key, bridge, or provider config for it found anywhere in the repo. Excluded from RUNNABLE-NOW; do not
assume it exists. If it is wired somewhere outside these files, that wiring must be produced before it
counts.

LANE 0 — THE HOST ITSELF (no model). Every fixture/static bench runs here. A prebuilt debug binary EXISTS
(.elai_cc/target/debug/elai, dated 2026-07-17), so the fixture benches are runnable without a fresh
build. This is the only lane that discharges anything TODAY, and only fixture-grade numbers.

PER-ROW EXECUTION TABLE

Columns: id | proof_spine | runner exists+builds | gate | existing artifact | lane that satisfies | verdict
| exact command. "Runner exists" verified via grep of the Bench* variants in crates/cli/src/args.rs
(24 wired). "Existing artifact" from `find benchmarks -type f` this session (real report vs
fixture-only vs empty). Prebuilt binary at .elai_cc/target/debug/elai; run from .elai_cc/.

PROOF-SPINE ROWS
- PS-1 safedelta | PS-1 | YES (Tier-1 arm only) | pod | already-produced-real BUT fixture-grade
  (safedelta-*.json: model_id=seccodeplt_curated_corpus, static gate over curated corpus, safety_delta
  0.34375, n=96) | Lane 0 (static arm) / NO LANE (live arm) | RUNNABLE-NOW (static, fixture-grade) +
  BLOCKED (live headline: Tier-2 model-in-loop harness UNBUILT, not a GPU gate) | `elai bench safedelta
  --seed 42 --json`  (reproduces the static ~0.34; does NOT produce the credible live number)
- PS-2 gate-tax | PS-2 | YES | journal | already-produced (gate-tax-*.json runner_status=RunnerPresent,
  task_class_label=fixture, coding_task_gate_tax_ms 0.26) | Lane 0 | RUNNABLE-NOW (fixture composition;
  MEASURED arm needs a real SSB journal, journal gate) | `elai bench gate-tax`
- PS-3 hallucination-delta | PS-3 | NO (runner=PLANNED) | none | empty | none | BLOCKED (PLANNED runner,
  owner L-10) | n/a
- PS-4 routing-economics | PS-4 | YES | cloud-dollar | already-produced (routing-economics-*.json
  suite=routing_economics_fixture_v1, deterministic UsageTurnEvent rows, est_cost_micros=0) | Lane 0
  (fixture arm) / NO LANE (live cost) | RUNNABLE-NOW (fixture $/task) + BLOCKED (live cost leg on J-28) |
  `elai bench routing-economics --seed 0 --json`
- PS-5 case-reuse | PS-5 | YES | none | already-produced-real (MEASURED 2026-07-17, recurrence_hit@1
  0.95, bm25 static retrieval, no model) | Lane 0 | RUNNABLE-NOW (already measured; observe-arm only,
  the reuse-into-generation moat number is K-18 Task 2, still PENDING on the unbuilt VCG J-track) |
  `elai bench case-reuse fixture --seed 0 --json`
- PS-6 wire-dark rate | PS-6 | YES (python sweep, not a bench binary) | none | already-produced-real
  (MEASURED 2026-07-09, 7.69% fn-grain) | Lane 0 | RUNNABLE-NOW (static tree sweep, no model) | `python
  .claude/tools/production_caller_check.py` full-tree sweep + dated ledger

FEATURE-BENCH ROWS (all wired, all fixture/static, all Lane 0, all RUNNABLE-NOW fixture-grade unless noted)
- seccodeplt | PS-1 | YES | pod | produced (5 files) | Lane 0 static | RUNNABLE-NOW (static insecure-rate;
  the PS-1 static precursor) | `elai bench seccodeplt --json`
- qdelta | (none) | YES | pod | empty | Lane 0 static / NO LANE live | RUNNABLE-BUT gate:pod live arm
  needs the same unbuilt model-in-loop harness as PS-1 | `elai bench qdelta <config> --seed N --json`
- arc | (none) | YES | cloud-dollar | produced (2, fixture) | Lane 0 (fixture JSONL) | RUNNABLE-NOW over
  the bundled mini fixture; the live cloud arm is cloud-dollar gated | `elai bench arc`
- convergence | (none) | YES | none | empty | Lane 0 | RUNNABLE-NOW | `elai bench convergence`
- model-refresh | (none) | YES | none | empty (committed ead13d9) | Lane 0 | RUNNABLE-NOW | `elai bench
  model-refresh`
- sycophancy | (none) | YES | none | produced (report + 101 fixtures) | Lane 0 | RUNNABLE-NOW | `elai
  bench sycophancy`
- staleness | (none) | YES | none | produced (6) | Lane 0 | RUNNABLE-NOW | `elai bench staleness [--json]
  [--broken-retrieval]`
- mode-routing | (none) | YES | none | fixtures only (2) | Lane 0 | RUNNABLE-NOW | `elai bench mode-routing`
- conformal | (none) | YES | none | produced (1 + fixtures) | Lane 0 | RUNNABLE-NOW | `elai bench conformal`
- rag | (none) | YES | none | empty | Lane 0 (bench_rag.rs is the ONE bench that references an inference
  client — may drive a live retrieval/embed path; verify before trusting) | RUNNABLE-NOW (retrieval
  quality) | `elai bench rag`
- prmbench | (none) | YES | none | produced (2 + fixtures) | Lane 0 | RUNNABLE-NOW | `elai bench prmbench`
- canary-regression | (none) | YES | none | produced (3 reports + 101 fixtures) | Lane 0 | RUNNABLE-NOW |
  `elai bench canary-regression`
- tamper | (none) | YES | none | produced (2 reports + scenarios) | Lane 0 | RUNNABLE-NOW | `elai bench
  tamper`
- error-amplification | (none) | YES | none | empty | Lane 0 | RUNNABLE-NOW | `elai bench
  error-amplification [--seed N] [--json] [--no-repair]`
- mcp-schema-budget | (none) | YES | none | empty | Lane 0 | RUNNABLE-NOW | `elai bench mcp-schema-budget`
- crane | (none) | YES | none | empty | Lane 0 | RUNNABLE-NOW (constrained-decode fidelity over fixtures) |
  `elai bench crane`
- snowballing | (none) | YES | none | empty | Lane 0 | RUNNABLE-NOW | `elai bench snowballing --pool A
  --output <PATH>`
- claim-scope | (none) | YES | none | empty | Lane 0 | RUNNABLE-NOW | `elai bench claim-scope`
- governance-probe | (none) | YES | none | empty | Lane 0 | RUNNABLE-NOW | `elai bench governance-probe`
- sequence-probe | (none) | YES | none | empty | Lane 0 | RUNNABLE-NOW | `elai bench sequence-probe`
- ablation | (none) | YES | none | empty | Lane 0 | RUNNABLE-NOW | `elai bench ablation --tile <T>
  --fixtures <F> --budget <N>`

E2E-FUNCTIONAL ROW
- local-lane-e2e | (none) | NO (runner=PLANNED) | none | empty | Lane 1 (:8181) WOULD satisfy it | BLOCKED
  (PLANNED runner; this is the ONLY row whose lane is the live 4B sidecar, and it is unbuilt) | n/a

NOTE ON THE 24 vs 26: `elai bench swe-bench-pro` exists as a Metrics action (args.rs:8415) but is not a
registry row; hallucination-delta and local-lane-e2e are the only two registry runners that are PLANNED.

ADVISORY-READ RECONCILIATION (proof_spine_status.py + elai_status_query.py, this session)

proof_spine_status.py --json (2026-07-18): 2/6 MEASURED (PS-5, PS-6). PS-1 GATED(pod), PS-2
RUNNER_PRESENT(journal), PS-3 PLANNED, PS-4 RUNNER_PRESENT(cloud-dollar). Its own PS-1 lane_tag confirms
my finding verbatim: "static-gate / seccodeplt-curated-corpus (no live model inference; live
model-in-loop headline needs ELAI_E2E_POD_ENDPOINT)". The advisory read AGREES with the per-row table:
the two measured numbers are the two that need no model; every model-dependent number is gated, and the
gate is the absence of a live model-in-loop harness, not a design fault.

elai_status_query.py --json (snapshot 51.7m, git 494ef36, not stale): OPEN 4. The two escalated punch
items are relevant to why the live numbers are stuck: I-33 (governed_prompt_decision has zero callers,
"blocked on owner seam-design: task_runner.rs hook + ELAI_E25_PROMPT_HANDLER_ENABLE") and I-23 (calibration
fork blocked on the unshipped I-4 PUCT seam + an ID-5 threshold baseline run). H Track-3 capstone gates
"built + CLI-reachable but NOT intercepting the loops they guard." None of these is the bench-endpoint
gap directly, but they are the same class: shipped machinery whose LIVE consumer seam is unbuilt.

RECONCILIATION VERDICT: no disagreement between the advisories and the source-level findings. Both say
the model-dependent numbers are un-produced; the source read adds WHY — the runners have no inference
client, so no endpoint (local or free) discharges them until a Tier-2 model-in-loop harness is authored.

PS-1 SAFETY DELTA — THE PRIORITY ANSWER

Q: Can `elai bench safedelta` be pointed at the live :8181 lane or a free/Nemotron API RIGHT NOW, and if
so what is the exact command?

A: NO. Not because the 4B is too weak (though it is), but because the runner has no path that calls ANY
model. `elai bench safedelta --seed 42 --json` runs TODAY and reproduces the STATIC number
(safety_delta 0.34375 over the seccodeplt_curated_corpus, model_id literally "seccodeplt_curated_corpus")
— but that is a static gate over a curated corpus, NOT a model generating code with the gate ON vs OFF.
There is no --endpoint/--model flag that routes safedelta to :8181 or a bridge; the ONLY env the runner
reads for a live arm is ELAI_E2E_POD_ENDPOINT, and that read exists solely inside a `#[ignore]` test
(safedelta_runner.rs:507-516). The production `run_safedelta_from_runner` always returns the Tier-1
static path and marks runpod_pending=true; the Tier-2 config is `pending()` with no HTTP client behind it.

WHAT PS-1 ACTUALLY NEEDS (in order):
1. BUILD the Tier-2 model-in-loop harness by REUSING the existing provider client
   (crates/providers/src/runtime/sidecar.rs, already speaks /v1/messages): a patch-generation loop that
   calls it to GENERATE a patch per task, runs the gate ON vs OFF over the model-authored patches, and
   scores residual-insecurity by DYNAMIC exploit execution (per R3, static-scoring is "weaker as benchmark
   truth"). The net-new code is the LOOP + the scorer + the corpus, NOT the HTTP client (that exists);
   roughly the scope the improvement-plan §3.4 + §4.1 describe, minus the client. Point it at :8181 (fix
   the :8100 default) for the cheap lane, cloud Sonnet for the credible one.
2. GIT-HARDEN the corpus (tarball snapshot, no .git) BEFORE any number — Endor's 81% git-history
   exploitation makes an un-hardened number a liability a competitor can discredit (improvement-plan §3.4,
   existential).
3. A CREDIBLE GENERATOR MODEL on 300-400 git-stripped tasks (R3 power: 200 tasks detects only ~9pp).

WHICH LANE WOULD BE CREDIBLE, AND DO WE HAVE IT:
- The 4B local :8181 lane is NOT credible for the headline: SUSVIBES's works-vs-secure gap was measured
  on Claude 4 Sonnet; a 4B trails frontier badly and ELAI's own baseline logged a 100%
  code-API-hallucination rate on the small model. Use :8181 only for the e2e-functional / mechanical
  smoke, never the safety headline.
- The CREDIBLE lane is CLOUD SONNET — the wedge lane per the strategy (FOCUS-RESCOPE 5.3: "run the wedge
  + SAFEDELTA with gates wrapping CLOUD SONNET"). It is reachable via ANTHROPIC_BASE_URL / the claude_chat
  bridge, but NEITHER is wired to the bench path today. DESIGN-SELL-BY (2026-06-05, code-grounded) says
  the number needs cloud API budget + a Linux CPU host with the sandbox enabled — NOT a GPU/RunPod — and
  the "pod" gate label is a legacy artifact of the abandoned RunPod plan.
- A strong FREE BRIDGE (claude_chat / grok_chat) could serve as the generator once the Tier-2 harness has
  a client, but the bridges are rate-limit-fragile browser-pool tools, not a benchmark-grade endpoint.

BOTTOM LINE ON PS-1: the number the whole GTM stakes on is blocked on BUILDING a ~medium model-in-loop
harness + git-hardening + a cloud-budget run, not on a pod. The static 0.34 already exists and is not the
credible headline. The 4B lane cannot serve it; cloud Sonnet can, and is affordable per DESIGN-SELL-BY,
but the harness that would drive it is unbuilt.

RUN THESE FIRST (prioritized by: feeds a proof-spine number, runnable today, cheapest lane)

TIER A — DISCHARGEABLE TODAY, ZERO NEW CODE, ZERO MODEL (Lane 0, prebuilt binary):
1. `python .claude/tools/production_caller_check.py` full-tree + refresh the PS-6 ledger — PS-6, already
   measured, cheapest, re-confirms the wire-dark rate. (feeds a spine number, zero cost)
2. `elai bench case-reuse fixture --seed 0 --json` — PS-5, already measured; re-runnable for the receipt.
3. `elai bench gate-tax` — PS-2, RunnerPresent fixture arm; produces the composed-gate-tax fixture number
   (the MEASURED arm still needs a populated SSB journal, journal gate).
4. `elai bench routing-economics --seed 0 --json` — PS-4, fixture $/task arm (live cost still blocks on
   J-28). Feeds the "$/verified-task" pitch at fixture grade.
5. `elai bench safedelta --seed 42 --json` + `elai bench seccodeplt --json` — PS-1 STATIC precursor;
   reproduces the 0.34 static-gate number for the receipt. NOT the credible headline; label it
   static-fixture.
(These write artifacts under .elai_cc/benchmarks/ — a repo mutation, so out of THIS agent's scope; they
are the run plan, not runs I executed.)

TIER B — THE REAL BLOCKER (net-new code before any credible headline number):
6. BUILD the SAFEDELTA Tier-2 model-in-loop harness (inference client + patch-generation loop +
   dynamic-exploit scorer + .git-stripped tarball corpus). This is the single highest-leverage build in
   the whole direction: it converts PS-1 from a fixture into the thesis headline. Owner-level, medium
   effort; improvement-plan §3.4/§4.1 is the spec.
7. Point that harness at CLOUD SONNET (ANTHROPIC_BASE_URL / claude_chat) on 50-100 git-stripped tasks
   FIRST (improvement-plan F0.1 kill-gate: sign + rough magnitude; kill the thesis cheaply if flat), then
   scale to 300-400 tasks for the publishable CI.
8. IN PARALLEL, run F0.3 (the agency willingness-to-pay calls) — no code, no lane, but it is the OTHER
   un-run falsifier that gates the whole GTM and it needs no engineering at all.

TIER C — SECONDARY, once the harness exists:
9. Reuse the Tier-2 harness for qdelta (quality delta) and the PS-4 live-cost leg. J-28 RESOLVED
   SHIPPED-WIRED (2026-07-17), so PS-4-live-cost is now a THREAD-THROUGH (S: swap the runner's fixture
   row-source for live SSB UsageTurnEvent rows), blocked only on the live turns the Tier-2 harness supplies.
10. `local-lane-e2e` on :8181 — the ONLY live use of the 4B sidecar; build its PLANNED runner to prove
    the local lane runs an end-to-end functional program (owner LOCAL-LANE directive 2026-07-17). It is a
    mechanical/thesis-substrate proof, NOT a safety headline.

THE ONE-LINE ANSWER: 24 of the 26 registry runners are wired (2 PLANNED: hallucination-delta, local-lane-e2e);
after the provenance-cluster cross-reference, ~14 produce a LEGITIMATE fixture receipt TODAY (WAVE 0, feature
shipped + no model needed), ~7 are PREMATURE (runner wired but the feature cluster is PLANNED/PARTIAL or the
number needs a live model — qdelta/arc/sycophancy/conformal/mode-routing/snowballing/claim-scope), and 5 are
BLOCKED (2 PLANNED runners + PS-1-live/PS-4-live-cost/PS-2-journal). The critical path is: WAVE 0 receipts now
> build the ONE Tier-2 model-in-loop harness (reuses the shipped crates/providers SidecarRuntime, NOT a new
client) > WAVE 1 the PS-1 kill-gate on a 50-100-task cloud-Sonnet slice > WAVE 2 per-feature ablations each
gated on ITS cluster > WAVE 3 composed numbers (journal, J-28) > WAVE 4 the two PLANNED runners. The "pod"
gate is a legacy label; the real gate on the thesis number is unbuilt consuming code + a cloud-budget run +
market falsification, never a GPU.

===================================================================================================
ORDERED EXECUTION SEQUENCE — THE WAVES (added 2026-07-18; the cadence, not just the catalog)
===================================================================================================

Cross-reference method: each bench's `provenance` field (registry.json) resolved to its plan-cluster
status by grep of the letter's SUMMARY_INDEX (cited inline). READINESS verdict = f(runner wired?,
feature cluster shipped?, live model needed?). SHIPPED/PARTIAL/PLANNED are the cluster's status, NOT
the runner's (a runner can be wired over a fixture while its feature is PLANNED — that number is
PREMATURE, a fixture-stub, not a real ablation).

Row shape: bench | proof_spine | provenance cluster + STATUS (cited) | readiness verdict | command |
unblocked-by (prerequisite ID from 7-BENCHMARK-READINESS-PLAN.md).

--- WAVE 0 — MODEL-FREE, CLUSTER-DONE, MEASURABLE TODAY (legitimate receipts, Lane 0, prebuilt binary) ---
These are runner-wired AND their feature is genuinely shipped AND the number needs no model, so the
fixture number MEANS something today. Run them for receipts; they cost nothing and unblock nobody but
themselves.
- PS-6 wire-dark | PS-6 | dated-ledger + production_caller_check, MEASURED 2026-07-09 (registry ledger row) |
  READY-TODAY | `python .claude/tools/production_caller_check.py` full-tree + refresh ledger | none
- PS-5 case-reuse | PS-5 | K-18 SHIPPED+MEASURED 2026-07-17 (K-SUMMARY:51-53 "PROVES+PACKAGES", ledger present) |
  READY-TODAY (observe arm; the reuse-into-generation moat number is K-18 Task 2, BENCH-PENDING on the VCG
  J-track) | `elai bench case-reuse fixture --seed 0 --json` | none (moat number blocked by P3-VCG)
- PS-2 gate-tax (fixture) | PS-2 | K-4 SHIPPED, BUILD-instrument (K-SUMMARY:180-192; bench_gate_tax.rs +
  artifacts on disk) | READY-TODAY (fixture composition; the journal-Measured arm is WAVE 3) |
  `elai bench gate-tax` | journal-arm unblocked by P3-JOURNAL
- PS-4 routing-economics (fixture) | PS-4 | K-2 SHIPPED, BUILD (K-SUMMARY:135-142) | READY-TODAY (fixture
  $/task; live cost is WAVE 3) | `elai bench routing-economics --seed 0 --json` | live-cost unblocked by
  P3-J28
- PS-1 safedelta (STATIC) + seccodeplt | PS-1 | G-6 PARTIAL (G-SUMMARY:111-127 Tier-1 static SHIPPED,
  Tier-2 live arm pending()) + F-2b bench SHIPPED-but-never-run-live (F-SUMMARY:198-213) | READY-TODAY
  (static-fixture precursor, label it static; NOT the credible headline) | `elai bench safedelta --seed 42
  --json` + `elai bench seccodeplt --json` | live headline unblocked by P1-TIER2
- convergence | (none) | I-27 SHIPPED, BUILD instrument (I-SUMMARY:499-510) | READY-TODAY (reuse ON/OFF over
  fixture, deterministic) | `elai bench convergence` | none
- model-refresh | (none) | K-9 SHIPPED, committed ead13d9 (deterministic cadence) | READY-TODAY |
  `elai bench model-refresh` | none
- canary-regression | (none) | I-24 SHIPPED, EXTENSION (I-SUMMARY:442-456; frozen canary set, R8) |
  READY-TODAY | `elai bench canary-regression` | none
- sequence-probe | (none) | I-35 SHIPPED (I-SUMMARY:663; temporal enforcement, deterministic) | READY-TODAY |
  `elai bench sequence-probe` | none
- crane | (none) | I-34 SHIPPED, BUILD (I-SUMMARY:641-662; grammar-pin fidelity over fixtures, deterministic) |
  READY-TODAY (constrained-decode fidelity; the reasoning-tax delta needs a model = WAVE 2) |
  `elai bench crane` | tax-delta unblocked by P1-TIER2
- mcp-schema-budget | (none) | J-2 mechanical token-cost (J-SUMMARY:129-131; schema token count is
  deterministic, independent of J-2's WAVE-4 server build) | READY-TODAY | `elai bench mcp-schema-budget` | none
- governance-probe | (none) | governance surface, deterministic policy probe | READY-TODAY |
  `elai bench governance-probe` | none
- staleness (Tier-1) | (none) | F-5 PARTIAL (F-SUMMARY:93-95, 253-255; Tier-1 age-quarantine logic ships,
  Tier-2 NLI ONNX model ops-gated/DARK) | READY-TODAY (broken-retrieval + age quarantine over fixtures) |
  `elai bench staleness [--json] [--broken-retrieval]` | Tier-2 NLI unblocked by an ops model-stage (P4)
- ablation | (none) | generic cross-bench arm-on/arm-off harness | READY-TODAY (mechanics; a real arm delta
  inherits its arm's own readiness) | `elai bench ablation --tile <T> --fixtures <F> --budget <N>` | none
- rag | (none) | B/C RAG stack; bench_rag.rs is the one bench referencing an inference/embed client — verify
  whether it drives a live embed path before trusting a "no-model" claim | READY-TODAY (retrieval quality;
  RECON its embed dependency) | `elai bench rag` | RECON (may need embed sidecar)
- prmbench | (none) | PRM stack fixtures; the fixture threshold check is deterministic, but the PRM scorer
  itself (D-2 PPM sidecar) is PLANNED, so a LIVE PRM number is PREMATURE | READY-TODAY (fixture threshold) /
  PREMATURE (live PRM) | `elai bench prmbench` | live PRM unblocked by D-2 (PLANNED)
- tamper (scenario) | (none) | H-28 PARTIAL: built + CLI-reachable but "NOT intercepting the loops they guard"
  (elai_status_query punch: H Track-3; darwin_admit_loop hook not live) | READY-TODAY (scenario detection over
  fixtures is deterministic; the number is honest as a detection-rate, NOT as a live-loop interception proof) |
  `elai bench tamper` | live interception unblocked by the H-28 darwin_admit_loop wiring (H Track-3 punch)

--- WAVE 1 — THE KILL-GATE (the cheapest credible falsifier; improvement-plan F0.1) ---
Runs ONLY after P1-TIER2 (the model-in-loop harness) exists. This is the single number the whole thesis
stakes on, measured cheaply first to kill the direction if flat BEFORE more plans stack on it.
- PS-1 safedelta (LIVE) | PS-1 | G-6 PARTIAL, Tier-2 arm UNBUILT (safedelta_runner.rs:78-100 pending();
  live env read only in a #[ignore] test :507-516) | BLOCKED-until-P1 then KILL-GATE | `elai bench safedelta
  --endpoint <cloud-sonnet> --seed 42 --json` over 50-100 git-STRIPPED tasks (sign + magnitude; kill if flat) |
  unblocked by P0-HARDEN (corpus) + P1-TIER2 (harness) + P2-ENDPOINT (flag) + P4-BUDGET (cloud $ + sandbox host)

--- WAVE 2 — PER-FEATURE ABLATIONS, each gated on ITS cluster being done ---
Runner wired but the number is PREMATURE today: either the feature cluster is PLANNED/PARTIAL, or the
ablation needs a live model. Each names the gating plan unit(s). Run after WAVE 1 proves the harness path.
- qdelta (LIVE) | (none) | I-38 QDELTA SHIPPED static, live arm needs the SAME Tier-2 model-in-loop harness as
  PS-1 | PREMATURE (live) | `elai bench qdelta <config> --endpoint <model> --seed N --json` | P1-TIER2 + P2-ENDPOINT
- mode-routing (real) | (none) | D-10 select_grounding_mode predicate PLANNED (D-SUMMARY:80-82 "— PLANNED") |
  PREMATURE (the routed feature is PLANNED; today's macro-F1 is a fixture-stub over labels, not a live routing
  ablation) | `elai bench mode-routing` | D-10 (PLANNED) + D-21 coverage bench
- sycophancy | (none) | D-14 probe PLANNED (D-SUMMARY:104-108); D-23 bench is "D-14 gating evidence" and also
  PLANNED (D-SUMMARY:162-164) | PREMATURE (needs the probe + a model inverted-framing re-run) | `elai bench
  sycophancy` | D-14 (PLANNED) + P1-TIER2/P2-ENDPOINT (model for inverted re-run)
- conformal | (none) | D-16 UniCR PLANNED (D-SUMMARY:116-120) + D-22 validation PLANNED (D-SUMMARY:156) |
  PREMATURE (needs UniCR calibration + refusal events) | `elai bench conformal` | D-16 + D-22 (both PLANNED)
- arc | (none) | D-20 ARC-AGI-2 probe PLANNED (D-SUMMARY:144-148), explicitly "runs ELAI under cloud Sonnet
  (no local 7B for ARC)" + depends D-10 (mode routing, PLANNED) | PREMATURE/BLOCKED (needs cloud model + D-10) |
  `elai bench arc` (fixture mini-JSONL runs today; the credible arm is cloud-dollar + D-10) | D-10 + D-20 +
  P2-ENDPOINT + P4-BUDGET
- snowballing | (none) | L-family CGG; multi-turn compounding needs a live model + the CGG gate under
  measurement (L-1..L-7, partially activated) | PREMATURE | `elai bench snowballing --pool A --output <PATH>` |
  P1-TIER2 + the CGG stack
- claim-scope | (none) | K-1 claims-policy lint SHIPPED (claims_policy_lint.py); but over/under-claim RATE vs
  measured evidence needs live model outputs to score | PREMATURE (live rate) / the lint itself is shipped |
  `elai bench claim-scope` | P1-TIER2 (model outputs to score)
- error-amplification | (none) | multi-turn error stack; the compounding-error rate over turns needs a model
  actually taking turns; the fixture arm is mechanical | READY-TODAY (fixture) / PREMATURE (live rate) |
  `elai bench error-amplification [--seed N] [--json] [--no-repair]` | live arm P1-TIER2

--- WAVE 3 — COMPOSED NUMBERS (stable gate SET + their specific unblocks) ---
Re-measure after any plan that adds/removes a gate. Blocked on infra/data, not on the model harness.
- PS-2 gate-tax (journal-Measured) | PS-2 | K-4 SHIPPED but the Measured arm needs a POPULATED SSB journal
  (proof_spine PS-2 gate=journal, "RunnerPresent fixture composition NOT the journal-populated Measured number") |
  BLOCKED (journal) | `elai bench gate-tax` after journal populated | P3-JOURNAL (a real run that emits SSB rows)
- PS-4 routing-economics (live cost) | PS-4 | K-2 SHIPPED; J-28 RESOLVED 2026-07-17 SHIPPED-WIRED
  (usage_ledger.rs + UsageTurnEvent both prod seams + `elai usage` verb; commit fe069d0), so the live-cost
  leg is a THREAD-THROUGH not a build | BLOCKED only on live turns (P1-TIER2/P2-ENDPOINT), NOT on J-28 |
  `elai bench routing-economics --endpoint <model> --seed 0 --json` | P3-J28 (S effort: swap the runner's
  RoutingFixture row-source for live SSB UsageTurnEvent rows into the existing rollup_usage_turns seam)

--- WAVE 4 — THE PLANNED-RUNNER ROWS (build the runner first) ---
- PS-3 hallucination-delta | PS-3 | L-10 runner=PLANNED (registry; L-SUMMARY:74-79 "BENCH-PENDING live arm",
  no `hallucination-delta` sub in args). NOTE the L-10 text claims a fixture tier "ships now" but the registry
  and args disagree — treat as PLANNED until the bench sub is wired (RECON) | BLOCKED (runner PLANNED) |
  n/a | P4-L10RUNNER (build benchmarks/hallucination_delta_runner.rs, reuse SAFEDELTA shape) + P1-TIER2 for live
- local-lane-e2e | (none) | runner=PLANNED (registry); the ONLY row whose lane is the live 4B :8181 sidecar |
  BLOCKED (runner PLANNED) | n/a | P4-E2ERUNNER (build the PLANNED e2e-functional runner; owner LOCAL-LANE
  directive 2026-07-17) — mechanical/thesis-substrate proof, never a safety headline (4B too weak)

READINESS TALLY (post-cluster-cross-reference): 24/26 runners wired; ~14 WAVE-0 legitimate-receipt-today;
~8 PREMATURE (feature PLANNED/PARTIAL or needs live model: qdelta, mode-routing, sycophancy, conformal, arc,
snowballing, claim-scope, error-amplification-live; prmbench-live); 5 BLOCKED (PS-1-live, PS-2-journal,
PS-4-cost, PS-3 PLANNED runner, local-lane-e2e PLANNED runner). The two MEASURED spine numbers (PS-5, PS-6)
remain the two that need no model and sell nothing; every model-dependent number sits behind P1-TIER2.

STATUS: COMPLETE
