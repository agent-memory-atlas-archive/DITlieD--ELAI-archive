7-BENCHMARK-READINESS-PLAN — THE PREREQUISITES THAT UNBLOCK THE PROOF-SPINE NUMBERS
STATUS: COMPLETE

GENERATOR RULING (owner, 2026-07-18) — BINDING, SUPERSEDES THE CLOUD-SONNET-GENERATOR FRAMING BELOW.
The generator the SAFEDELTA / hallucination-delta / qdelta benchmark drives is the FREE lane, never
cloud Sonnet. This corrects a stale transmission: the "credible lane is CLOUD SONNET" line traces to
FOCUS-RESCOPE 5.3 (the superseded trust-wrapping-Sonnet wedge); the owner's LOCAL-LANE directive
(2026-07-17) repositioned the thesis to the SMALL-MODEL SUBSTRATE, so a delta measured on a free/small
model MEASURES THE PRODUCT CLAIM ITSELF (harness lifts a weak model), not a compromise.
  GENERATOR = glm-5-2 (free devin bridge, ~200K ctx, clean fresh context) as PRIMARY; the local 4B
    :8181 for the mechanical / e2e-functional proof (P3-E2ERUNNER) and a cheap sign-only kill-gate.
    grok/other free bridges are a second family if a leg ever needs cross-family.
  WORKER == GENERATOR IS FINE: SAFEDELTA's scorer is a DETERMINISTIC gate + dynamic exploit-execution,
    NOT an LLM judge, so glm-as-worker (builds the harness) and glm-as-generator (emits the patch under
    test) share no self-evaluation loop. The one rule: if any leg ever uses an LLM-as-judge, it is a
    DIFFERENT family than the generator (the walk already enforces cross-family judging).
  THE ONE CAVEAT THAT SURVIVES IS VIABILITY, NOT CREDIBILITY: confirm the free/local generator can emit
    a patch the gate can score and that compiles for the exploit-scorer. If the 4B is too incoherent on
    a task, use glm-5-2 as the generator (still free) or the static-scoring FORK (P1 already has it);
    NEVER reach for paid Sonnet.
  P4-BUDGET DROPS OFF THE CRITICAL PATH: no cloud spend gates any number. The only owner-provisioned
    infra that remains is a CONFINED HOST to execute untrusted model-authored patches (a SAFETY
    boundary, not a cost) OR accept static-grade scoring first. Read every "cloud Sonnet" / "cloud
    budget" reference below as "the free lane" and every P4-BUDGET cloud-spend clause as "a confined
    exec host (or static-scoring fallback)".
  UNCHANGED BECAUSE MODEL-AGNOSTIC: git-harden the corpus (P0-HARDEN), ~300-400 git-stripped tasks for a
    tight CI (R3 power), confined execution of untrusted patches, and a DETERMINISTIC scorer. LABEL every
    number with its lane (model id + quant + host) and frame it internally as the SMALL-MODEL-LIFT claim,
    never a frontier claim (K-1 claims-policy guard).

PROOF_SPINE: feeds PS-1..PS-6 (this plan does not measure a number itself; it builds/hardens the
prerequisites — corpus hardening, the Tier-2 model-in-loop harness, the endpoint flag, the journal,
J-28, the two PLANNED runners, the cloud budget — that convert every model-dependent spine number
from BLOCKED/PREMATURE to MEASURABLE. Direct feed: P1 unblocks PS-1/qdelta/PS-4-live; P3 unblocks
PS-2/PS-4/PS-3; P0 is the existential precondition for ANY published spine number).

WHAT THIS FILE IS: a dependency-ordered, kill-gate-first prerequisite plan. Sibling 6-RUNNABLE-NOW.md
answers "which bench runs on which lane and is its number ready"; this file answers "what must be built
or done, in what order, before the model-dependent numbers can be measured credibly." Every unit is a
move with an EXPECTED_OBSERVATION + ON_FAIL; code units are vertical slices naming a PRODUCTION_HOOK seam.

GROUND TRUTH THIS PLAN IS BUILT ON (source-verified, do not re-derive):
- ELAI HAS a production inference client: crates/providers/src/runtime/sidecar.rs (SidecarRuntime:
  reqwest, complete() -> /v1/messages, stream(), health_check(); RuntimeAdapter trait) + the CLI agent
  loop crates/cli/src/runtime_client.rs. The net-new work is the patch-gen LOOP + exploit scorer +
  hardened corpus that REUSE this client, NOT a client from scratch.
- The SAFEDELTA Tier-2 arm is SafeDeltaTier2Config::pending() (safedelta_runner.rs:78-100); the only
  ELAI_E2E_POD_ENDPOINT read is in a #[ignore] test (:507-516). No bench runner has an HTTP client
  today (6-RUNNABLE-NOW: grep -rlE 'reqwest|hyper|ureq' crates/orchestrator/src/benchmarks/ = 0).
- SidecarRuntime DEFAULT_SIDECAR_URL is the DEAD :8100 (sidecar.rs:44); the LIVE local lane is :8181
  (qwen3.5-4b-q4km, llama.cpp, curl-confirmed up this session). Mid-migration per commit 8bc668b.
- The 4B local lane is NOT a credible safety/quality/hallucination generator (SUSVIBES gap measured on
  Claude 4 Sonnet; ELAI's own baseline logged 100% code-API-hallucination on the small model). The
  credible lane is CLOUD SONNET (FOCUS-RESCOPE 5.3), reachable via ANTHROPIC_BASE_URL / claude_chat —
  NEITHER wired to the bench path today. DESIGN-SELL-BY: cloud API budget + a Linux CPU sandbox host,
  NOT a GPU/RunPod; the "pod" gate label is a legacy artifact.
- R3 statistical power (5-SYNTHESIS §6): min detectable delta ~8.9pp @200 tasks / ~7.2pp @300 / ~6.3pp
  @400 (McNemar). "2-3pp on 200 tasks is dead on arrival." Need 300-400 git-STRIPPED tasks.
- Anti-gaming (existential): Endor SWE-Agent+Opus exploited git history in 163/200 = 81% despite
  instructions (improvement-plan §3.4). An un-hardened corpus makes any number discreditable.

UNIT CLASS LEGEND: [CODE] ships a vertical slice; [OPS] infra/data/budget, no repo code; [NON-CODE]
a market/falsification activity with no artifact in the repo.

FORKS: (declared per-unit below; the plan-level decision points are the WAVE-1 kill-gate FORK and the
J-28 status FORK — see P3-J28 and WAVE-1)

===================================================================================================
P0 — EXISTENTIAL PRECONDITIONS (must precede ANY published number; cheapest, no model)
===================================================================================================

P0-HARDEN [OPS] — git-harden the eval corpus BEFORE any published safety/quality number.
Dependency: none (do first). Owner-effort: ~0.5 day (scripting + one verification pass).
WHY: improvement-plan §3.4 — Endor's SWE-Agent+Opus exploited git history in 163/200 = 81% of tasks
despite instructions; a number measured on a corpus that still carries .git / issue metadata is a
liability a competitor discredits on sight. This gates the WHOLE thesis number, so it is P0 not P4.
MOVE: assemble the held-out task corpus as a tarball SNAPSHOT with .git, issue text, PR links, and
fix-commit metadata STRIPPED; freeze it out-of-band (R8) so the scored model never authors its own
corpus; record a blake3 manifest of the stripped tree.
EXPECTED_OBSERVATION: a corpus tarball whose `git log` / `.git` is absent (find <extract> -name .git
returns nothing), a blake3 manifest file, and a loader that refuses a task dir containing a .git.
ON_FAIL: if a task still resolves its fix from residual metadata (cause: strip missed a path — issue
JSON, embedded diff, commit-message hint) — the COUNTER-MOVE is to add that path to the strip list,
re-snapshot, re-hash, and add a negative test that a known-leaky task now fails to "solve" from
metadata alone. Do NOT raise a threshold to paper over the leak.
RECON_NEEDED: the exact corpus source is unsettled — SecCodePLT/CodeGuard+ (F-2b) vs a git-stripped
SWE-style set. SETTLING CHECK: read safedelta_runner.rs + seccodeplt_runner.rs to confirm which corpus
the Tier-1 static arm already loads, and whether it ships with metadata; grep the corpus dir for .git.
VERIFY (off-agent): `find <corpus-extract> -name .git -o -name '*.patch' -o -name 'issue*.json'` returns
empty (exit-code artifact); the blake3 manifest recomputes byte-identically on a second strip run.
PRODUCTION_HOOK: n/a (OPS — no new pub symbol; the corpus loader used by P1-TIER2 consumes the tarball).

P0-WTP [NON-CODE] — the F0.3 agency willingness-to-pay falsifier.
Dependency: none (runs fully in parallel with everything; no code, no lane). Owner-effort: calls, days
of calendar not engineering.
WHY: improvement-plan Phase-0 falsify-first (5-SYNTHESIS §5) — execution built Phase 1 and NEVER ran the
two falsifiers that gate the GTM: F0.1 (the live SAFEDELTA pilot, = WAVE-1 here) and F0.3 (WTP calls).
The number, even once produced, is refuted-as-sufficient (5-SYNTHESIS R-2: R1 says benchmarks don't
close, R4 says incumbents counter-market it). F0.3 is the OTHER un-run falsifier and it needs zero code.
MOVE: run the agency / security-buyer willingness-to-pay conversations (the neutral-notary buyer, not
per-dev); record sign (would they pay for signed off-agent trust evidence) + rough magnitude.
EXPECTED_OBSERVATION: a short written finding (workflow/reasoning/) with N conversations, the buyer
class, and a sign+magnitude on WTP for the notary artifact — a market datum the code cannot produce.
ON_FAIL: if buyers are indifferent to signed evidence (cause: the notary position is not a felt pain
for this buyer) — COUNTER-MOVE is to record it as a thesis-level red flag and surface to the owner
BEFORE the 300-400-task publishable run is funded (a flat WTP kills the GTM cheaper than a flat number).
FORK: if WTP is clearly positive -> proceed to fund WAVE-1 publishable; if flat/negative -> ABORT the
publishable run, keep only the WAVE-1 kill-gate cheap slice, escalate to owner (the position, not the
number, is the product per 5-SYNTHESIS §1).
VERIFY (off-agent): the written finding exists with dated, attributed conversations (a human artifact,
not a self-report of the plan).
PRODUCTION_HOOK: n/a (NON-CODE).

===================================================================================================
P1 — THE UNIVERSAL UNBLOCK: the Tier-2 model-in-loop harness (the single highest-leverage build)
===================================================================================================

P1-TIER2 [CODE] — build the SAFEDELTA Tier-2 model-in-loop harness by REUSING the shipped provider client.
Dependency: P0-HARDEN (needs the stripped corpus to run against). Owner-level, medium effort (~1-2 wk).
WHY: this ONE unit converts PS-1 from a static fixture into the thesis headline AND unblocks qdelta-live
AND the PS-4 live-cost leg — three numbers behind one build. The client exists (SidecarRuntime); the
net-new is the LOOP + the SCORER + the corpus wiring (improvement-plan §3.4/§4.1 minus the client).
MOVE (vertical slice — producer + consumer + smoke in the same unit):
  (a) a patch-generation loop that, per task in the P0-HARDEN corpus, calls SidecarRuntime.complete()
      to GENERATE a candidate patch (REUSE the client, do not fork it);
  (b) run the security gate ON vs OFF over the MODEL-AUTHORED patches (the SAFEDELTA ablation framing);
  (c) a DYNAMIC exploit-execution scorer (per R3, static scoring is "weaker as benchmark truth") in a
      sandbox, emitting residual-insecure-rate per arm;
  (d) replace SafeDeltaTier2Config::pending() with a real config threaded from the bench entrypoint.
EXPECTED_OBSERVATION: `elai bench safedelta --endpoint <url>` drives the model, writes a
safedelta-*.json whose model_id is the REAL model (not "seccodeplt_curated_corpus") + a per-arm
insecure-rate computed by exploit execution + a signed RRA run manifest; safedelta_runner.rs no longer
returns Tier-1-only with runpod_pending=true when an endpoint is supplied.
ON_FAIL: if the run returns the static Tier-1 path with an endpoint set (cause: the Tier-2 config is
still pending() / the dispatch never reaches the loop) — COUNTER-MOVE: trace run_safedelta_from_runner,
confirm the Tier-2 branch is reached (add a temporary [DBG-t2] log), wire the config through; this is a
producer-without-consumer failure (R6), STOP + handoff if the seam is structurally wrong rather than
shipping the loop unwired.
FORK: if the dynamic sandbox is not ready in time (trigger: exploit-exec host unavailable) -> ship the
loop with STATIC scoring first, LABEL the number static-grade, and file the dynamic scorer as a
follow-up — never claim a dynamic number from a static run (5-SYNTHESIS R3).
RECON_NEEDED: the exact Tier-2 seam signature. SETTLING CHECK: read
crates/orchestrator/src/benchmarks/safedelta_runner.rs lines 78-100 + 507-516 to confirm
SafeDeltaTier2Config::pending()'s fields and the #[ignore]-test's endpoint-read shape before authoring.
RECON_NEEDED: sandbox choice for dynamic exploit exec. SETTLING CHECK: confirm a Linux CPU sandbox host
is available (P4-BUDGET) and whether ELAI already ships a confinement primitive (grep confinement/sandbox
in crates/) to reuse rather than build.
VERIFY (off-agent): the emitted safedelta-*.json carries model_id=<real model> + arm insecure-rates +
an RRA manifest that independently replays (the RRA verify exit code, not the runner's stdout);
`python .claude/tools/production_caller_check.py --files <changed .rs>` returns no DARK symbol.
PRODUCTION_HOOK: crates/orchestrator/src/benchmarks/safedelta_runner.rs::run_safedelta_from_runner (the
Tier-2 branch, replacing SafeDeltaTier2Config::pending()); driven from the `elai bench safedelta` arg
dispatch in crates/cli/src/args_bench.rs. WIRING: the new patch-gen-loop fn + scorer fn are called from
this production dispatch in the SAME changeset; the unit ships the corpus-loader consumer + a smoke that
drives `elai bench safedelta --endpoint <stub>` end to end (R10, not a unit test calling the loop).
INTEGRATION_SMOKE: an INDEPENDENT agent drives `elai bench safedelta --endpoint <live-or-stub>` on a
HARD input (a multi-file task from the stripped corpus, the STRICT gate config, a run long enough to
fire the ON-vs-OFF ablation both arms) and asserts BOTH arm rows + the RRA land — not a single happy-path
task. This smoke is a required deliverable of P1, run before P1 is DONE (per the 2-3-unit cadence P1 is
itself a seam-heavy multi-part unit).

===================================================================================================
P2 — THE ENDPOINT FLAG (cheap; makes a runner point at the cheap lane OR the credible lane)
===================================================================================================

P2-ENDPOINT [CODE] — :8100->:8181 default fix + a --endpoint/--model flag on the bench path.
Dependency: none for the default fix; composes with P1-TIER2 (the flag is how P1's harness selects a
lane). Owner-effort: ~0.5-1 day. Can land BEFORE P1 completes (they touch different files).
WHY: the shipped client's default points at the retired :8100 (sidecar.rs:44), and no bench subcommand
has a --model/--endpoint flag (6-RUNNABLE-NOW LANE-2). Without this, a runner cannot be aimed at :8181
(cheap smoke) OR cloud Sonnet (credible headline).
MOVE: (a) change DEFAULT_SIDECAR_URL from :8100 to :8181 (or make it env-driven with :8181 default);
(b) add --endpoint <url> and --model <id> to the bench arg surface so the P1 harness routes to the
chosen lane; (c) validate the endpoint with SidecarRuntime.health_check() before a run starts.
EXPECTED_OBSERVATION: `elai bench safedelta --endpoint http://127.0.0.1:8181 --model qwen3.5-4b-q4km`
health-checks green and routes generation to :8181; with no flag, the default resolves to :8181 not the
dead :8100; an unreachable endpoint fails LOUD at startup, never silently to a fixture.
ON_FAIL: if the flag is accepted but generation still hits :8100 (cause: a hardcoded URL downstream of
the flag) — COUNTER-MOVE: grep for the :8100 / DEFAULT_SIDECAR_URL literal across crates/, thread the
flag value to every call site; a stale hardcode is a spaghetti smell, fix at the source not with a shim.
FORK: if changing the DEFAULT breaks an existing consumer that relies on :8100 (trigger: a test or the
CLI agent loop refuses) -> keep :8100 as a fallback ONLY behind an explicit opt-out and default new runs
to :8181; do not silently dual-write.
RECON_NEEDED: how many call sites hardcode :8100 vs read DEFAULT_SIDECAR_URL. SETTLING CHECK:
`grep -rn '8100\|DEFAULT_SIDECAR_URL' crates/` before editing (impact analysis, blast radius).
VERIFY (off-agent): a run log / SSB row shows the request target host:port = the flag value (an emitted
artifact, not stdout); `production_caller_check.py --files <changed .rs>` clean; the health-check-fail
path returns nonzero (exit-code artifact).
PRODUCTION_HOOK: crates/providers/src/runtime/sidecar.rs (DEFAULT_SIDECAR_URL const) + the bench arg
parser at crates/cli/src/args_bench.rs (the new --endpoint/--model flags consumed by the P1 dispatch).
INTEGRATION_SMOKE: covered by P1-TIER2's smoke (the flag is exercised there driving the real entrypoint);
standalone, `elai bench safedelta --endpoint <bad-url>` must fail-closed at health-check (asserted row).

===================================================================================================
P3 — PER-PROOF-SPINE UNBLOCKS (each frees ONE number; parallelizable after P1/P2)
===================================================================================================

P3-JOURNAL [OPS] — populate a real SSB journal so PS-2 gate-tax emits the Measured (not fixture) number.
Dependency: P1-TIER2 + P2-ENDPOINT (a real model-in-loop run is what emits SSB rows). Owner-effort: the
cost of one real run, no new code.
WHY: proof_spine PS-2 gate=journal; the shipped runner emits "RunnerPresent fixture composition NOT the
journal-populated Measured number." The gate-tax Measured arm reads a populated SSB journal.
MOVE: execute a real gated run (the P1 harness over the corpus) so the SSB WAL accumulates per-turn gate
events, then run `elai bench gate-tax` over that journal.
EXPECTED_OBSERVATION: gate-tax-*.json flips runner_status from RunnerPresent to a journal-backed Measured
composition (real coding_task_gate_tax_ms + over_block_rate from actual gate firings, n_in_scope > the
fixture 11).
ON_FAIL: if the journal is empty after a run (cause: SSB emit not firing on the gated path) — COUNTER-MOVE:
grep ssb.rs for the gate-event append call site, confirm the gated run reaches it; a missing emit is a
wire-dark producer, STOP + handoff not a silent fixture fallback.
RECON_NEEDED: which SSB event types the gate-tax Measured arm requires. SETTLING CHECK: read
crates/orchestrator/src/benchmarks/gate_tax_audit_leg.rs to see what journal rows it folds.
VERIFY (off-agent): the gate-tax-*.json's value_status != "RunnerPresent fixture composition" and its
rows recompute from the journal (GD-2); proof_spine_status.py flips PS-2 GATED->closer-to-MEASURED.
PRODUCTION_HOOK: n/a (OPS — consumes the shipped bench_gate_tax.rs + gate_tax_audit_leg.rs; the emit
seam is the already-shipped ssb.rs gate-event append, exercised by the P1 run).

P3-J28 [CODE] — token-ledger cost attribution so PS-4 emits a LIVE $/verified-task (not est_cost_micros=0).
Dependency: P1-TIER2 + P2-ENDPOINT (needs real turns to attribute cost to). Owner-effort: S
(thread-through — J-28 confirmed SHIPPED-WIRED, see RESOLVED FORK; only the row-source swaps).
WHY: K-2 routing-economics live cost leg "blocks on J-28" (K-SUMMARY:135-136); the fixture arm reports
est_cost_micros=0. J-28 is the per-model/per-task/per-session real-time token-consumption + rollup core.
MOVE: confirm J-28's status; if shipped, thread its per-turn UsageTurnEvent cost into the
routing-economics runner; if not shipped, that is itself the blocker to build.
EXPECTED_OBSERVATION: routing-economics-*.json carries est_cost_micros > 0 sourced from real
UsageTurnEvent rows with per-model pricing, and a nonzero $ delta between the cloud-only and hybrid arms.
ON_FAIL: if cost stays 0 with real turns (cause: pricing table absent or UsageTurnEvent not carrying
model+token counts) — COUNTER-MOVE: verify J-28's UsageTurnEvent schema populates token counts + a
pricing map; a zeroed cost with real turns is a data-wiring gap, trace it, do not publish a $0 number.
FORK RESOLVED (2026-07-17, recon-j28, grep + production_caller_check over live .elai_cc): J-28 is
SHIPPED-WIRED, not PLANNED (its plan-file header still reads PLANNED but the tree disagrees — commit
fe069d0 "land the j-28 token ledger"). Evidence: crates/orchestrator/src/usage_ledger.rs present
(NormalizedUsage, resolve_turn_usage, rollup_usage_turns:216); SsbActionType::UsageTurnEvent +
UsageTurnEventPayload registered in ssb/action_type.rs + ssb/payloads.rs; the row is EMITTED at BOTH
production seams (cloud grounding_client.rs:988, local runtime_orchestrator.rs:421/508); the `elai
usage` verb is wired (main.rs:1546 CliAction::Usage -> usage_cmd::run_usage). TAKEN BRANCH -> J-28
ships, so PS-4-live is a THREAD-THROUGH (S effort), NOT a build. The PS-4 runner already consumes the
ledger: routing_economics_runner.rs imports usage_ledger::rollup_usage_turns and folds
UsageTurnEventPayload rows in usage_cost_micros (line ~241). The only change is the ROW SOURCE — swap
the RoutingFixture loader in run_routing_economics for a reader of live UsageTurnEvent rows off the SSB
journal, feeding the SAME usage_cost_micros/rollup_usage_turns seam. P3-J28 does NOT expand to a
token-ledger build; PS-4-live stays early in the critical path.
RECON_NEEDED: RESOLVED — J-28 shipped-vs-planned settled SHIPPED-WIRED (evidence in the RESOLVED FORK
above; production_caller_check PASS 0-dark over usage_ledger.rs + usage_cmd.rs).
VERIFY (off-agent): routing-economics-*.json est_cost_micros > 0, recomputable from the UsageTurnEvent
journal; production_caller_check.py clean on any new .rs.
PRODUCTION_HOOK: crates/orchestrator/src/benchmarks/routing_economics runner (the cost-attribution
call) consuming J-28's UsageTurnEvent emit seam; driven from `elai bench routing-economics`. If J-28
must be built, its PRODUCTION_HOOK is the per-turn usage emit in the agent loop (runtime_client.rs).

P3-L10RUNNER [CODE] — build the hallucination-delta runner (PS-3, currently PLANNED).
Dependency: P1-TIER2 (the live arm needs the model-in-loop harness); the fixture tier can precede it.
Owner-effort: M (reuse the SAFEDELTA runner shape, do not fork it).
WHY: registry PS-3 runner=PLANNED; L-SUMMARY:74-79 marks L-10 BENCH-PENDING with no `hallucination-delta`
sub in args. It is the L-plan's whole-plan done-gate (CGG ON vs OFF unsupported-claim rate).
MOVE: add benchmarks/hallucination_delta_runner.rs + a `hallucination-delta` bench sub (mirror the
SAFEDELTA runner; +HallucinationDeltaArm SSB kind), fixture tier first, then the live arm via P1's harness.
EXPECTED_OBSERVATION: `elai bench hallucination-delta` runs; the deterministic fixture arm emits one
artifact per arm (CGG ON vs OFF unsupported-claim rate), recomputable from the journal.
ON_FAIL: if the live arm emits a delta with the gate off (cause: the ablation is not actually toggling the
CGG) — COUNTER-MOVE: assert the --gate-off-but-claim-savings broken arm shows the delta COLLAPSE (L-10's
own negative test); a persistent delta with the gate off means the gate is not the cause, STOP + diagnose.
FORK: if the CGG stack (L-1..L-7) is not sufficiently activated to run a live ablation (trigger: the gate
has no ON-state to measure) -> ship the fixture tier only, label PS-3 fixture-grade, defer live to when the
CGG is live.
RECON_NEEDED: the CGG activation state. SETTLING CHECK: grep L-1..L-7 consumer seams / read the L-plan
_grounding map before claiming a live ON-state exists.
VERIFY (off-agent): the per-arm artifact exists + recomputes (GD-2); proof_spine_status.py flips PS-3
PLANNED->RUNNER_PRESENT; production_caller_check.py clean.
PRODUCTION_HOOK: crates/orchestrator/src/benchmarks/hallucination_delta_runner.rs::run driven from a new
`hallucination-delta` arm in crates/cli/src/args_bench.rs; the arm + runner + fixture ship together (R10).
INTEGRATION_SMOKE: an independent agent drives `elai bench hallucination-delta` gate-ON vs gate-OFF on a
multi-claim fixture and asserts both arm rows + the collapse on the broken arm.

P3-E2ERUNNER [CODE] — build the local-lane-e2e runner (the ONLY live use of the 4B :8181 sidecar).
Dependency: P2-ENDPOINT (the :8181 default/flag). Owner-effort: M. Owner LOCAL-LANE directive 2026-07-17.
WHY: registry local-lane-e2e runner=PLANNED; it is the thesis-substrate proof that the local lane runs an
end-to-end FUNCTIONAL program — a mechanical proof, NEVER a safety headline (4B too weak, ground truth).
MOVE: build the e2e-functional runner that drives SidecarRuntime against :8181 on a small functional task
and asserts the program compiles + passes its own test.
EXPECTED_OBSERVATION: `elai bench local-lane-e2e` health-checks :8181, generates + builds + tests a small
program, emits an artifact under benchmarks/local_lane_e2e with a functional pass/fail + a lane tag.
ON_FAIL: if generation succeeds but the build/test step is skipped (cause: the functional gate is not
wired) — COUNTER-MOVE: assert the artifact carries a real compile+test exit code, not a self-reported OK;
a "generated OK" with no build proof is exactly the fixture-stub trap.
FORK: if :8181 is down at run time (trigger: health_check fails) -> the runner fails LOUD (exit nonzero),
never falls back to a fixture; re-boot the local sidecar and retry.
RECON_NEEDED: none for the runner shape (reuses SidecarRuntime); FORKS covered above.
VERIFY (off-agent): the emitted artifact carries a functional compile+test exit code (off-agent build
artifact); production_caller_check.py clean.
PRODUCTION_HOOK: crates/orchestrator/src/benchmarks/local_lane_e2e_runner.rs::run driven from a
`local-lane-e2e` arm in crates/cli/src/args_bench.rs; runner + arm + smoke ship together.
INTEGRATION_SMOKE: an independent agent drives `elai bench local-lane-e2e` against the LIVE :8181 and
asserts a real functional-pass artifact (drives the real entrypoint on the real lane, R10).

===================================================================================================
P4 — SECONDARY / OPS (the credible-run infrastructure; parallel to P1-P3)
===================================================================================================

P4-BUDGET [OPS] — cloud-Sonnet API budget + a Linux CPU sandbox host for the credible safety run.
Dependency: none to provision; consumed by WAVE-1. Owner-effort: OPS/procurement, no code.
WHY: DESIGN-SELL-BY (code-grounded) — the credible number needs cloud API budget + a Linux CPU host with
the sandbox enabled, NOT a GPU/RunPod. The "pod" gate label is legacy. This is what makes the cloud-Sonnet
generator + the dynamic exploit-exec scorer (P1) runnable at 300-400 tasks.
MOVE: provision the cloud-Sonnet key/quota reachable via ANTHROPIC_BASE_URL and a Linux CPU sandbox host
where the dynamic exploit scorer runs the model-authored patches safely.
EXPECTED_OBSERVATION: a claude-lane endpoint that P2-ENDPOINT can target with a real quota, and a sandbox
host where a patch executes under confinement without touching the real machine.
ON_FAIL: if the sandbox lets a patch escape (cause: confinement not enforced) — COUNTER-MOVE: STOP, do
NOT run untrusted model-authored patches on an unconfined host; this is a destructive-op safety gate.
FORK: if cloud budget is not approved (trigger: owner declines the spend) -> run WAVE-1 kill-gate on the
:8181 4B lane FIRST for a mechanical sign check ONLY (explicitly not the credible headline), and hold the
publishable run until budget lands.
RECON_NEEDED: whether ELAI already ships a confinement primitive to reuse. SETTLING CHECK: grep
confinement/sandbox/jail in crates/ (shared with P1's sandbox RECON).
VERIFY (off-agent): a health-check against the cloud endpoint returns green (artifact); a sandbox escape
test (a patch that tries to write outside its jail) is BLOCKED (exit-code artifact).
PRODUCTION_HOOK: n/a (OPS — the endpoint is consumed by P2-ENDPOINT's flag; the sandbox by P1's scorer).

===================================================================================================
WAVE-1 — THE KILL-GATE RUN (the F0.1 cheap falsifier, then the R3 publishable run)
===================================================================================================

W1-KILLGATE [OPS/run] — the PS-1 live safety-delta on a 50-100-task cloud-Sonnet slice FIRST.
Dependency: P0-HARDEN + P1-TIER2 + P2-ENDPOINT + P4-BUDGET (all four). This is the point of the whole plan.
WHY: improvement-plan F0.1 — measure sign + rough magnitude cheaply so a FLAT number kills the thesis
BEFORE the expensive 300-400-task run is funded. R3: 200 tasks detects only ~8.9pp, so the kill-gate is a
sign check, not a publishable claim.
MOVE: run `elai bench safedelta --endpoint <cloud-sonnet> --seed 42 --json` (P1 harness) over 50-100
git-STRIPPED tasks (P0-HARDEN corpus); read the sign + magnitude of the gate-ON-vs-OFF residual-insecure
delta.
EXPECTED_OBSERVATION: a safedelta-*.json with a real-model model_id, a signed delta with a rough CI, on
50-100 tasks — enough to see sign + order-of-magnitude, not a publishable pp claim.
ON_FAIL: if the delta is flat/zero (cause: the gate provides no marginal residual-insecure reduction on
model-authored patches) — this is the kill-gate DOING ITS JOB: STOP, do not fund the 300-400 run, surface
to owner that the thesis number is flat (5-SYNTHESIS §5: build-first happened, falsify-first did not).
FORK: TRIGGER on the kill-gate result —
  (a) sign positive + magnitude >= the R3 @300-400 detectable band (~6-7pp) -> proceed to W1-PUBLISH;
  (b) sign positive but magnitude < the detectable band -> the number will be "dead on arrival" at
      publishable n (5-SYNTHESIS R-2/R3); escalate to owner before spending on scale;
  (c) sign flat/negative -> ABORT the publishable run; the position (neutral notary), not the number, is
      the product; keep the honest static-fixture number only.
VERIFY (off-agent): the safedelta-*.json + its RRA manifest replay independently (RRA verify exit code);
proof_spine_status.py flips PS-1 GATED->MEASURED with a live (not static-fixture) value_status.
PRODUCTION_HOOK: n/a (a RUN of the P1 harness, not new code).

W1-PUBLISH [OPS/run] — the 300-400-task publishable safety-delta (only if W1-KILLGATE FORK (a)).
Dependency: W1-KILLGATE FORK (a) + P4-BUDGET (the scale spend) + P0-WTP FORK positive.
WHY: R3 statistical power — a publishable claim needs 300-400 git-stripped tasks (~6.3pp detectable @400).
MOVE: run the harness at 300-400 tasks; report "N fewer dynamically-exploitable diffs per 100 tasks reach
a reviewer" with a 95% bootstrap CI + per-CWE breakdown (G-6 framing), NEVER a SWE-bench rank.
EXPECTED_OBSERVATION: a safedelta-*.json at n=300-400 with a tight-enough CI to support the headline + a
signed RRA + per-CWE cells; the number is bound to an F-4 stdout_hash (G-6 COMPOSES-WITH).
ON_FAIL: if per-CWE cells are too small for a significance claim (cause: G-6's own named CAVEAT,
whats_missing) — COUNTER-MOVE: pool suites per G-6:126-127, or report only the aggregate delta with the
per-CWE as directional; do not overclaim a per-CWE significance the n cannot support.
FORK: FORKS: none (the go/no-go fork already fired at W1-KILLGATE; this unit only runs on FORK (a)).
VERIFY (off-agent): the RRA replays; the claims-policy lint (claims_policy_lint.py) PASSES on any copy
citing the number (a bare superlative or an untagged external number FAILS — K-1).
PRODUCTION_HOOK: n/a (a RUN).

===================================================================================================
RECON_NEEDED (plan-level roll-up of the unsettled assumptions + their exact settling checks)
===================================================================================================
- CORPUS SOURCE (P0-HARDEN): read safedelta_runner.rs + seccodeplt_runner.rs for the loaded corpus + grep
  it for .git/metadata.
- TIER-2 SEAM SIGNATURE (P1): read safedelta_runner.rs:78-100 + 507-516 for SafeDeltaTier2Config::pending().
- SANDBOX PRIMITIVE (P1/P4): grep confinement/sandbox/jail in crates/ for a reusable confinement.
- :8100 CALL SITES (P2): grep -rn '8100|DEFAULT_SIDECAR_URL' crates/ before editing.
- J-28 SHIPPED-VS-PLANNED (P3-J28): RESOLVED 2026-07-17 (recon-j28) -> SHIPPED-WIRED (usage_ledger.rs
  + UsageTurnEvent emitted both prod seams + `elai usage` verb wired; commit fe069d0). PS-4-live is a
  THREAD-THROUGH (S), NOT a build; the PS-4 runner already folds UsageTurnEvent via rollup_usage_turns.
- CGG ACTIVATION STATE (P3-L10RUNNER): grep the L-1..L-7 consumer seams for a live gate ON-state.
- GATE-TAX JOURNAL SCHEMA (P3-JOURNAL): read gate_tax_audit_leg.rs for the folded SSB row types.

===================================================================================================
ABORT (the conditions where execution STOPS AND FLAGS rather than improvises)
===================================================================================================
- A published safety/quality number is about to be emitted from a corpus that still carries .git or fix
  metadata (P0-HARDEN not verified) -> STOP, harden first; an un-hardened number is a liability.
- P1's dynamic scorer would run an untrusted model-authored patch on an UNCONFINED host (P4 sandbox not
  enforced) -> STOP (destructive-op safety gate, R-2); never execute untrusted patches unconfined.
- A runner returns a fixture number while an --endpoint was supplied (silent fixture fallback) -> STOP;
  a silent fallback to a static number under a live flag is the exact fixture-stub trap this plan exists
  to prevent. Fail LOUD instead.
- The W1-KILLGATE delta is flat and someone proposes funding the 300-400 run anyway -> STOP + owner
  escalation (the kill-gate exists to prevent exactly this stacking-plans-on-a-flat-number spend).
- A new pub Rust symbol from P1/P2/P3 has no production caller (production_caller_check.py DARK, no
  WIRE-DARK row) -> STOP + handoff (R-8 producer-without-consumer), never ship the loop/runner unwired.

===================================================================================================
REDTEAM (attacks against THIS plan; surviving attack + patches on record)
===================================================================================================
ATTACK 1 (SURVIVED): "Just stand up :8181 and run `elai bench safedelta` — the pod gate is the only
blocker." WHY IT FAILS AGAINST THE PLAN: the runner has NO path that calls any model (safedelta_runner.rs
Tier-2 = pending(); grep of benchmarks/ for an inference client = 0). Standing up an endpoint produces
zero live numbers; the plan's P1-TIER2 is the real blocker. The plan is correct that the gate is unbuilt
CODE, not a GPU. (Source: 6-RUNNABLE-NOW headline + CORRECTION, source-verified this session.)
ATTACK 2 (SURVIVED): "The 4B local lane is live and free — use it for the safety headline to save cloud
budget." WHY IT FAILS: the 4B is not a credible generator (SUSVIBES gap measured on Claude 4 Sonnet;
ELAI's own baseline logged 100% code-API-hallucination on the small model). The plan confines :8181 to
P3-E2ERUNNER (mechanical thesis-substrate) and the FORK-(c) sign-only fallback, never the headline.
ATTACK 3 (SUCCEEDED -> PATCHED): the first draft ordered P1-TIER2 before P0-HARDEN. An attacker points out
that building the harness and running it on an un-hardened corpus produces a number that is instantly
discreditable (Endor 81% git-history exploit), wasting the run. PATCH: P0-HARDEN is now a hard dependency
of P1-TIER2 and W1, and an ABORT condition guards a published number from an un-hardened corpus.
ATTACK 4 (SUCCEEDED -> PATCHED): the first draft treated PS-4-live as a simple thread-through of J-28. An
attacker notes J-28's shipped-vs-planned status is UNVERIFIED, so the effort estimate could be off by an
order of magnitude. PATCH: P3-J28 now carries an explicit FORK on the J-28 status RECON, expanding to a
build if J-28 is not shipped, and PS-4-live moves later in the critical path in that branch.
ATTACK 5 (SURVIVED): "Run the publishable 300-400-task number first for a strong headline, skip the cheap
kill-gate." WHY IT FAILS: falsify-first (improvement-plan F0.1, 5-SYNTHESIS §5) — a flat number found at
scale wastes the entire spend; W1-KILLGATE at 50-100 tasks kills a flat thesis for ~1/4 the cost. The plan
orders kill-gate strictly before publish, with a go/no-go FORK between them.

===================================================================================================
CRITICAL PATH (dependency diagram; CODE vs OPS vs NON-CODE marked)
===================================================================================================
  P0-HARDEN [OPS] ─┐
                   ├─> P1-TIER2 [CODE] ─┬─> W1-KILLGATE [run] ──FORK(a)──> W1-PUBLISH [run]
  P2-ENDPOINT [CODE]┘   (needs P0+P2)   │       (needs P0,P1,P2,P4)
        │                              ├─> P3-JOURNAL [OPS] ──> PS-2 Measured
        │                              ├─> P3-J28 [CODE/RECON] ──> PS-4 live cost
        │                              └─> P3-L10RUNNER [CODE] ──> PS-3
        └─> P3-E2ERUNNER [CODE] ──> local-lane-e2e (:8181, mechanical)
  P4-BUDGET [OPS] ───────────────────────> (feeds W1-KILLGATE credible lane + P1 sandbox)
  P0-WTP [NON-CODE] ──(parallel, gates the GTM, no code)──> FORK: fund-publish? or ABORT-to-position

  THE SINGLE CRITICAL-PATH PREREQUISITE: P1-TIER2. It is the ONE build that converts PS-1 (and qdelta,
  and PS-4-live) from BLOCKED to measurable; everything thesis-critical is downstream of it. P0-HARDEN is
  its existential gate; P2-ENDPOINT is its lane selector; P4-BUDGET is its credible fuel; W1-KILLGATE is
  the cheap falsifier that decides whether the rest of the spend happens. Order: P0 ∥ P2 -> P1 -> P4-check
  -> W1-KILLGATE -> (FORK) W1-PUBLISH, with P3-* fanning out off P1 in parallel.

STATUS: COMPLETE

