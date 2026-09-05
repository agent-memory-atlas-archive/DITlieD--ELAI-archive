# ELAI K-plan exhaustive feature discovery

Source audited: `docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md` (600/600 lines read).

Orientation only: `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md` (592/592 lines read). No feature below is promoted to explicit K scope merely because it appears in the overview.

Generated for the corpus-wide feature compilation. This is a summary-layer extraction, not a substitute for inspecting the original K plan files and their grounding.

## Evidence and scope legend

- **E** — explicit in `K-SUMMARY_INDEX.md`; wording may be atomized but no capability is added.
- **I** — inference needed to describe a composition or consequence; never treated as committed behavior.
- **A** — ambiguity, stale index metadata, or a missing/deferred source requiring deeper inspection.
- **Status vocabulary** preserves the summary's own distinctions: WIRED/SHIPPED, BUILD/PLANNED, DESIGN, PARTIAL, FIXTURE-now, BENCH-PENDING, RunPod/network-gated, default-OFF/observe-first, OUT-OF-K, KILLED, and user-gated.
- Each atomic item inherits the line anchor printed under its K heading. Cross-entry claims cite the cross-cutting or preamble anchor explicitly.

## Quantitative coverage

- Summary coverage: **600/600 lines (100%)**.
- Normal detailed entry coverage: **24/24 headings** — K-1..K-9 and K-11..K-25.
- Additional partial record: **K-26**, described only in the summary preamble (lines 82-91), with no normal entry body.
- Deliberate identifier gap: **K-10 has no standalone file or summary entry**; its sequencing doctrine is said to live in `K-0_INDEX.md` (lines 93-99).
- Unindexed on-disk candidates: **K-18a** and **K-27** exist but are not described by this summary.
- Source-tree manifest coverage: **32/32 files** presently under `K-plan/` are accounted for below (one is this summary; 31 are deeper-inspection sources/candidates).
- Atomic feature/behavior count: **622 explicit atoms**, **1 entry-local ambiguity atom**, plus **8 ambiguity** and **2 inference** items in the separate ledger (**633 traceable items total**).

## K's role in the completed ELAI system

Source anchors: premise lines 101-112; entry-contract lines 93-99; cross-cutting lines 588-600.

- **K-X.01 [E, purpose]** K is primarily a sequencing, composition, proof, distribution, affordability, packaging, and discipline layer over the verification/evidence spine, not a conventional capability letter.
- **K-X.02 [E, exceptions]** K-23 multi-repo/workspace memory and K-24 billing/credits are the two explicitly acknowledged net-new capability exceptions.
- **K-X.03 [E, product thesis]** The product is the measured number: external positioning waits on falsification, measurement, portable evidence, and PR distribution rather than shipping unsupported claims.
- **K-X.04 [E, first gate]** K first invokes the already-shipped A-1 falsifier and may trigger a kill-switch before further build work.
- **K-X.05 [E, sequencing]** A-1..A-4 plus K-3 gate the rest of the corpus under the K-10 no-new-letters rule.
- **K-X.06 [E, proof]** A-2/A-3 with K-1/K-2 produce the missing headline measurements.
- **K-X.07 [E, distribution]** A-4 supplies the portable artifact and K-3/K-11 put its machine and human render where review happens: the PR/check-run and cockpit.
- **K-X.08 [E, affordability]** K-4 measures whole-system gate friction; K-5 must choose a deterministic risk tier that fits beneath K-4's locked ceiling.
- **K-X.09 [E, buyer packaging]** K-6 turns existing evidence into an offline compliance bundle and K-7 turns governed measurements and controls into buyer-facing AppSec/insurance prose.
- **K-X.10 [E, depreciation]** K-8 performs the initial absorbability audit; K-25 makes it a staleness-gated standing watchdog; K-9 separately refreshes local/model-lane choices quarterly.
- **K-X.11 [E, differentiators]** KD-1 is number-before-claim; KD-2 is meet-the-buyer-in-the-PR; KD-3 is a composed-friction ceiling; KD-4 is provider-absorption pre-gating; KD-5 is a time-boxed wedge/window clock.
- **K-X.12 [E, truth labels]** Every external figure KW-1..KW-6 must be tagged `[EXTERNAL]`; every shipped-number claim must be ELAI-measured.
- **K-X.13 [E, composition]** Entries must name their reused substrate and explicit do-not-reimplement boundaries under GD-1.
- **K-X.14 [E, record]** Measured runtime outputs are expected to be journal-recomputable under GD-2.
- **K-X.15 [E, done gates]** Runtime legs use a W-1 real run-trigger observable-effect gate; positioning/doctrine legs use E-23/JD-1 artifact-and-loading gates.
- **K-X.16 [E, negative proof]** Each entry defines an R2 negative or broken arm so success is causally attributable and fail-closed behavior is tested.
- **K-X.17 [E, UI discipline]** Runtime-facing additions either name a frontend pair or defer the choice to edit time under W-4; CLI/doc-only work may justify N/A under W-6.
- **K-X.18 [E, SSB discipline]** Most K units add no SSB family or prefer reuse with at most one event; K-23 and K-24 inherit the explicit sequential +1 event bump.
- **K-X.19 [E, promotion discipline]** The two net-new tracks inherit run-trigger PASS, R10 integration smoke, journal-as-record, default-OFF enablement, and an allowlist promotion row.
- **K-X.20 [E, scope boundary]** The eight Section-A items A-1..A-8 are pre-existing plan files that K sequences; they are not K sessions and are not summarized here.

## Exhaustive atomic inventory

### K-1 — problem-quantification dossier + claims policy

Anchor: lines 116-134. Track K-T1 PROVE; E-23-class; KD-1; STRONG grounding; doc+lint; effort S; wave KW-1.

- **K-1.01 [E, document]** Create a problem-quantification dossier that binds external problem figures to ELAI-measured results as those measurements land.
- **K-1.02 [E, external datum]** Preserve KW-1 USENIX hallucinated-package figures: 5.2%/21.7% and 58%/43% recurrence, explicitly external.
- **K-1.03 [E, external datum]** Preserve KW-2's 11x-vulnerabilities figure and citation `arXiv:2507.02976`, explicitly external.
- **K-1.04 [E, external datum]** Preserve KW-3 reward-hacking figures 73.8%/46.8%, explicitly external.
- **K-1.05 [E, external datum]** Preserve KW-4 CodeAct figures approximately 50%/-60%, explicitly external.
- **K-1.06 [E, external datum]** Preserve KW-5 SpecBench +28 percentage points, explicitly external.
- **K-1.07 [E, measured join]** Bind those figures to the A-1 kill-gate verdict, A-2 SeCodePLT result, and A-3 SAFEDELTA delta when available.
- **K-1.08 [E, policy]** Create a claims-policy document.
- **K-1.09 [E, invariant]** An external-facing claim may ship only with a journal-recomputable ELAI number or an explicit `[EXTERNAL]`/`[DESIGN]` tag.
- **K-1.10 [E, tooling]** Run a claims-policy lint over README and positioning copy.
- **K-1.11 [E, wording]** Treat the I-43 trace-authentication-versus-deterministic-replay wording table per lane as normative.
- **K-1.12 [E, dependency]** Invoke I-43's A-1 verdict; do not rebuild it.
- **K-1.13 [E, exclusion]** KW-6 DORA/METR/vendor material remains PENDING-VERIFY and excluded until dossier-build reverification.
- **K-1.14 [E, integration]** Place the dossier under `docs/strategy/` or `workflow/reasoning/`.
- **K-1.15 [E, integration]** Place the lint under `.claude/tools/` or an `elai`-side positioning/README check.
- **K-1.16 [E, state]** Add no SSB event family.
- **K-1.17 [E, done gate]** Every dossier row must contain either a measured ELAI number plus GD-2 recompute pointer, or an explicit design/external tag plus KW source.
- **K-1.18 [E, negative]** A seeded unmeasured superlative such as “the safest agent” must fail lint.
- **K-1.19 [E, negative]** A bare external number without an explicit tag must fail lint.

### K-2 — routing-economics bench

Anchor: lines 135-153. Track K-T1 PROVE; PROVABLE+EFFICIENT; STRONG gap grounding/external figures; BUILD; effort M; wave KW-2; live cost leg depends on J-28.

- **K-2.01 [E, CLI]** Add `elai bench routing-economics` through a `routing-economics` bench subcommand at the `args_bench.rs:75` seam.
- **K-2.02 [E, corpus]** Run against a fixed task corpus.
- **K-2.03 [E, ablation]** Compare cloud-only with the H-7 architect/editor hybrid.
- **K-2.04 [E, metric]** Report the percentage of steps offloaded.
- **K-2.05 [E, metric]** Report dollar delta.
- **K-2.06 [E, metric]** Report quality delta as verified-OUTCOME rate, never pass@k.
- **K-2.07 [E, determinism]** Run both arms under `ELAI_SEED`.
- **K-2.08 [E, provenance]** Make results GD-2 recomputable from journal evidence.
- **K-2.09 [E, reuse]** Mirror the SAFEDELTA/QDELTA runner shape rather than inventing a new harness.
- **K-2.10 [E, reuse]** Consume H-7 routing and I-25 router substrate.
- **K-2.11 [E, dependency]** Block the live dollar leg on J-28 normalized per-lane cost capture.
- **K-2.12 [E, display]** Feed H-9 cockpit display.
- **K-2.13 [E, reuse]** Reuse I-27's verified-outcome-versus-compute harness; do not fork it.
- **K-2.14 [E, constraint]** Leave E-39 cap supremacy unchanged.
- **K-2.15 [E, runner]** Add a runner mirroring `benchmarks/safedelta_runner.rs`.
- **K-2.16 [E, state]** Prefer an existing bench/`RouteDecisionEvent` row; otherwise add at most `RoutingEconomicsMeasured`.
- **K-2.17 [E, UI]** Decide at edit time whether a hybrid-versus-cloud table belongs on the runtime/cost panel.
- **K-2.18 [E, artifact]** Emit one artifact per arm with dollars per verified task.
- **K-2.19 [E, delivery]** Ship the deterministic FIXTURE tier immediately.
- **K-2.20 [E, fail-closed]** Refuse to emit a live dollar number until J-28 exists; never fabricate cost.
- **K-2.21 [E, negative]** A `--force-cloud-everything` arm must collapse the savings delta, demonstrating the hybrid caused the result.
- **K-2.22 [E, market status]** The “first production number” is a planned positioning claim; the cited HERA ~45% offload is research-grade/external, not an ELAI result.

### K-3 — PR/CI evidence emitter

Anchor: lines 156-177. Track K-T2 WEDGE+DISTRIBUTION; AUDITABLE; STRONG live-module grounding; BUILD renderer; effort M-L; wave KW-2; KD-2/KD-5.

- **K-3.01 [E, distribution]** Render ELAI evidence into GitHub check-runs and PR comments.
- **K-3.02 [E, render]** Include F-1/F-2/F-9 gate verdicts per hunk.
- **K-3.03 [E, render]** Include F-4 stdout-hash-bound test outcomes.
- **K-3.04 [E, render]** Include G-5 DSR and vouched/unvouched bands.
- **K-3.05 [E, render]** Place H-52 anchored findings using its resolver so fabricated line numbers are structurally impossible.
- **K-3.06 [E, render]** Include the G-8 attestation digest.
- **K-3.07 [E, render]** Include an offline-verification instruction using `elai verify-attestation`.
- **K-3.08 [E, render]** Include the J-16 blast-radius cone summary when available.
- **K-3.09 [E, fail-closed UI]** Render a hunk with no evidence as `UNVOUCHED`; never omit it or mark it clean.
- **K-3.10 [E, reuse]** Implement as a renderer over the existing C-40/E-41 `github_pr` adapter and `elai pr-review`; reimplement none of the posting machinery.
- **K-3.11 [E, reuse]** Reuse G-5 review-surface, H-52 anchors, F-4 evidence, G-8 attestation, and J-16 cone data.
- **K-3.12 [E, data boundary]** Read the RRA evidence manifest as a closed set.
- **K-3.13 [E, structural security]** Make out-of-evidence comments structurally impossible by rendering only from the closed manifest.
- **K-3.14 [E, integration]** Add the renderer beneath `crates/orchestrator/src/github_pr/` on the body-composer/comment path.
- **K-3.15 [E, state]** Prefer existing `github_pr`, review-surface, or attestation rows; add at most one event.
- **K-3.16 [E, UI]** Pair with the existing GitHub state surface (`github.svelte.ts`), decision finalized at edit time.
- **K-3.17 [E, done gate]** On a real fixture-repo PR, every check-run claim must join to a journal row.
- **K-3.18 [E, negative]** A `--post-unvouched-as-clean` broken arm must fail.

### K-4 — composed gate-tax budget

Anchor: lines 180-198. Track K-T3 ADAPTIVE GATE ECONOMY; PROVABLE; STRONG grounding; BUILD instrument; effort M; wave KW-3; KD-3; K-T3 keystone.

- **K-4.01 [E, CLI]** Add `elai bench gate-tax` through a `gate-tax` bench subcommand at `args_bench.rs:75`.
- **K-4.02 [E, cadence]** Produce a per-release friction report.
- **K-4.03 [E, metric]** Measure end-to-end added wall-clock per task class.
- **K-4.04 [E, metric]** Measure composed whole-system over-block rate over a fixed corpus.
- **K-4.05 [E, ablation]** Compare gates ON versus gates OFF using SAFEDELTA-style framing.
- **K-4.06 [E, interpretation]** Treat friction as QDELTA's denominator twin.
- **K-4.07 [E, configuration]** Lock a ceiling to a real user-selected value established from the first ON/OFF measurement; never invent it.
- **K-4.08 [E, release gate]** Block a release when the composed ceiling is breached.
- **K-4.09 [E, observability]** Name the top three friction contributors.
- **K-4.10 [E, reuse]** Live beside and share I-38 QDELTA's runner shape and corpus.
- **K-4.11 [E, reuse]** Reuse G-6 ablation framing, the 105-row `wire-dark-allowlist.toml` gate inventory, and E-49 metrics.
- **K-4.12 [E, state]** Add at most `GateTaxMeasured`, or extend I-38 rows.
- **K-4.13 [E, UI]** Consider a friction tile on the cost/audit surface at edit time.
- **K-4.14 [E, done gate]** Make the composed number journal-recomputable.
- **K-4.15 [E, negative]** A deliberately spammy fixture gate must trip the ceiling and fail the release gate.
- **K-4.16 [E, denominator]** A gates-OFF run establishes the ON/OFF denominator.
- **K-4.17 [E, fail-loud]** A zero ceiling with any default-ON gate must fail loudly, forcing a real configured value.

### K-5 — risk-tiered gating policy

Anchor: lines 199-216. Track K-T3; CANT-WRECK; STRONG substrate grounding; BUILD policy; effort M; wave KW-3; KD-3/R8; depends on K-4.

- **K-5.01 [E, policy]** Define a deterministic R8-frozen `GateIntensity` policy table.
- **K-5.02 [E, always-on]** Keep the E-30 argv allowlist always on.
- **K-5.03 [E, always-on]** Keep F-1/F-9 secret/dependency checks always on.
- **K-5.04 [E, always-on]** Keep G-1 taint checks always on.
- **K-5.05 [E, expensive gates]** Trigger I-20 mutation testing only through risk policy.
- **K-5.06 [E, expensive gates]** Trigger H-26 differential testing only through risk policy.
- **K-5.07 [E, expensive gates]** Trigger VLM tier 2 only through risk policy.
- **K-5.08 [E, expensive gates]** Trigger E-53 formal verification only through risk policy.
- **K-5.09 [E, risk input]** Use J-16 blast-radius cone size.
- **K-5.10 [E, risk input]** Use F-2 CWE-surface class.
- **K-5.11 [E, risk input]** Use B-22 autonomy level.
- **K-5.12 [E, risk input]** Use D-15 novelty.
- **K-5.13 [E, risk input]** Use file criticality.
- **K-5.14 [E, authority]** Never let the LLM select a tier; an LLM-proposed tier is ignored.
- **K-5.15 [E, economy]** Collapse safely for low-risk mechanical edits by using fewer turns and only cheap gates.
- **K-5.16 [E, safety]** Use the full expensive stack at high-risk decision points.
- **K-5.17 [E, integration]** Implement a compositional policy-table-plus-selector dispatcher.
- **K-5.18 [E, integrity]** Put the policy-table hash into the frozen gate manifest.
- **K-5.19 [E, self-protection]** Prevent a run from weakening or editing its own gating policy.
- **K-5.20 [E, state]** Add at most `GateIntensitySelected`, otherwise reuse F-1 rows.
- **K-5.21 [E, UI]** Consider tier-decision rows in the audit timeline.
- **K-5.22 [E, expected behavior]** A trivial rename emits only always-on gate rows and zero expensive-leg rows.
- **K-5.23 [E, expected behavior]** A high-cone authentication-touching edit fires the full stack.
- **K-5.24 [E, proof]** Demonstrate measured composed tax below K-4's ceiling on at least three real runs.
- **K-5.25 [E, negative]** Block an attempted in-run edit to the frozen policy table.

### K-6 — compliance export

Anchor: lines 219-236. Track K-T4 BUYER-SIDE EVIDENCE; AUDITABLE; STRONG shipped-substrate grounding; BUILD compose; effort S-M; wave KW-3; KD-5.

- **K-6.01 [E, CLI]** Add `elai export compliance`.
- **K-6.02 [E, bundle]** Emit one bundle, not separate uncoordinated artifacts.
- **K-6.03 [E, bundle]** Include a G-9-redacted G-2 RRA.
- **K-6.04 [E, bundle]** Include the G-8 attestation.
- **K-6.05 [E, bundle]** Include an audit-trail extract.
- **K-6.06 [E, bundle]** Include a retention statement that names the I-44 mode.
- **K-6.07 [E, mapping]** Include a mapping document that links bundle fields to procurement clauses.
- **K-6.08 [E, mapping]** Map requirements through code, build, and release traceability.
- **K-6.09 [E, mapping]** Map audit-log requirements.
- **K-6.10 [E, mapping]** Map OMB M-25-22-style independent-evaluation and reproducibility rights.
- **K-6.11 [E, wording]** Describe EU AI Act Article 50 only as “designed to support”; never claim Article-50 certification.
- **K-6.12 [E, mapping]** Address PLD exposure.
- **K-6.13 [E, mapping]** Provide SOC2-adjacent control narratives.
- **K-6.14 [E, reuse]** Compose G-2/G-8/G-9/I-44/I-18; add no new attestation machinery.
- **K-6.15 [E, integration]** Add a CLI arm composing existing RRA, attestation, and redaction paths plus a strategy mapping doc.
- **K-6.16 [E, state]** Add at most `ComplianceExported`, otherwise reuse G-8 rows.
- **K-6.17 [E, UI]** CLI-only N/A is justified; a compliance tile remains an edit-time option.
- **K-6.18 [E, portability]** Verify the exported bundle offline on a clean machine with `elai verify-attestation` and a conformance checker.
- **K-6.19 [E, privacy test]** A raw-secret grep over the export must be empty.
- **K-6.20 [E, traceability]** Every procurement clause in the mapping document must cite a bundle field.
- **K-6.21 [E, negative]** A bundle without its attestation digest fails offline verification.
- **K-6.22 [E, negative]** An unredacted RRA containing a seeded secret fails the secret-grep gate.

### K-7 — appsec/insurance positioning brief

Anchor: lines 237-249. Track K-T4; PROVABLE; STRONG KW grounding; positioning; effort S; wave KW-3; KD-1.

- **K-7.01 [E, document]** Create a pure-prose AppSec/insurance positioning brief under `docs/strategy/`.
- **K-7.02 [E, buyer]** Position the evidence pack within AppSec/SCA budgets.
- **K-7.03 [E, evidence]** Link the F-9 hallucinated-dependency gate to governed KW-1 USENIX figures.
- **K-7.04 [E, evidence]** Link F-2 CWE gates to the governed KW-2 11x-vulnerability figure.
- **K-7.05 [E, insurance]** Answer insurance questions about controls, logs, rollback, and independent evidence.
- **K-7.06 [E, dependency]** Source every number from K-1's dossier.
- **K-7.07 [E, reuse]** Cite G-8/G-9/I-44 controls; add no runtime behavior.
- **K-7.08 [E, state]** Add no SSB event.
- **K-7.09 [E, governance]** Tag every number as measured or external.
- **K-7.10 [E, human gate]** Require user review before external use.
- **K-7.11 [E, negative]** Fail K-1 lint for any number lacking a measured-or-external tag.

### K-8 — absorbable-unit audit + provider-check pre-gate

Anchor: lines 252-271. Track K-T5 DEPRECIATION+REFRESH; AUDITABLE; STRONG map/research grounding; audit+rule; effort S-M; wave KW-1; KD-4.

- **K-8.01 [E, audit]** Sweep remaining PLANNED/PARTIAL efficiency units in H-Track-1 H-1..H-9 residue.
- **K-8.02 [E, audit]** Sweep Track-14 I-10..I-16 residue.
- **K-8.03 [E, audit]** Sweep E-2/E-25/E-32 residual legs.
- **K-8.04 [E, audit]** Sweep J-24 tool-context residual legs.
- **K-8.05 [E, classification]** Tag provider-native units `ABSORBED` and kill or de-scope them to their instrument half.
- **K-8.06 [E, classification]** Tag units requiring a build-time provider recheck `AT-RISK`.
- **K-8.07 [E, classification]** Tag verification-coupled units worth keeping `DURABLE`.
- **K-8.08 [E, artifact]** Produce a kill/de-scope decision list.
- **K-8.09 [E, authority]** Require explicit user confirmation before any removal; never auto-apply the list.
- **K-8.10 [E, doctrine]** Add a KD-4 planning pre-gate rule.
- **K-8.11 [E, fresh research]** Before executing any efficiency-class unit, verify the provider's current native surface via WebSearch.
- **K-8.12 [E, integration]** Upsert absorbability tags into `ELAI-FEATURE-MAP.md`.
- **K-8.13 [E, integration]** Add a conditional-routing `.claude/rules/` file for efficiency-class tasks.
- **K-8.14 [E, integration]** Store the kill list as a `workflow/reasoning/` user-decision artifact.
- **K-8.15 [E, state]** Add no SSB event.
- **K-8.16 [E, done gate]** Make feature-map tags grep-provable.
- **K-8.17 [E, done gate]** Demonstrate the rule loading in a live efficiency-class task.
- **K-8.18 [E, negative]** Flag an efficiency-class task that skips the provider recheck.
- **K-8.19 [E, violation]** Treat an automatically applied removal as a house-rule violation.

### K-9 — quarterly model-refresh cadence

Anchor: lines 272-288. Track K-T5; REACH; STRONG J-9 grounding; process+bench wrapper; effort S-M; wave KW-2; depends on J-9.

- **K-9.01 [E, cadence]** Add a recurring quarterly evaluated checkpoint.
- **K-9.02 [E, selection]** Compare top J-9 catalog candidates per lane with the incumbent.
- **K-9.03 [E, metric]** Measure fixture pass rate.
- **K-9.04 [E, metric]** Measure VRAM use.
- **K-9.05 [E, metric]** Measure tokens per second.
- **K-9.06 [E, metric]** Measure dollars per verified task.
- **K-9.07 [E, lanes]** Evaluate through J-9 catalog, E-27 AWQ, and J-19 vLLM lanes.
- **K-9.08 [E, decision]** Emit an `ADOPT`, `KEEP`, or `RE-EVALUATE` decision row.
- **K-9.09 [E, human gate]** Require user ratification of that row.
- **K-9.10 [E, doctrine]** Keep the local lane scoped to mechanical/privacy/latency work.
- **K-9.11 [E, doctrine]** Treat 24GB-class hardware as the autonomy-candidate tier, pending the 2x3090 box.
- **K-9.12 [E, doctrine]** Keep cloud as the planning/architect lane.
- **K-9.13 [E, prior decisions]** Treat Gemma-4 and Qwen3-32B rejections as tier-conditional catalog data, not permanent global bans.
- **K-9.14 [E, routing integrity]** Do not change routing without user sign-off.
- **K-9.15 [E, implementation]** Add a cadence checklist and bench wrapper over J-9.
- **K-9.16 [E, state]** Prefer J-9 rows; otherwise add at most `ModelRefreshCadence`.
- **K-9.17 [E, UI]** Consider a refresh-decision tile at edit time.
- **K-9.18 [E, artifact]** Emit a dated refresh report on the first run.
- **K-9.19 [E, safe fallback]** Automatically keep the incumbent when a candidate regresses fixtures.
- **K-9.20 [E, negative]** Reject a claimed win without a fixture run.
- **K-9.21 [E, negative]** Block a silent model swap without ratification.

### K-11 — human-viewable evidence render

Anchor: lines 291-310. KAT wave; AUDITABLE; STRONG shipped-substrate/demand grounding; BUILD renderer; effort M; wave KW-2; extends K-3/KD-2.

- **K-11.01 [E, render]** Extend K-3's per-hunk verdict renderer with human-viewable evidence.
- **K-11.02 [E, PR UI]** Add a `## Evidence` section to the PR comment/check-run.
- **K-11.03 [E, cockpit UI]** Show the same evidence in the reviewer cockpit.
- **K-11.04 [E, evidence]** Render the G-2 RRA digest.
- **K-11.05 [E, evidence]** Render F-4 stdout-hash test outcomes and their links.
- **K-11.06 [E, evidence]** Render G-5 DSR vouched/unvouched bands.
- **K-11.07 [E, evidence]** Render H-52 anchored findings.
- **K-11.08 [E, evidence]** Render references to screenshots already captured by `visual-ui-loop`.
- **K-11.09 [E, prioritization]** Risk-tier the view using G-5 DSR so reviewers inspect by tier.
- **K-11.10 [E, fail-closed UI]** Render a no-artifact hunk as `UNVOUCHED`, never clean or omitted.
- **K-11.11 [E, reuse]** Use K-3/E-41 `github_pr`, `ReviewCommentKind`, G-5, G-2, F-4, H-52, and existing screenshots; add no evidence machinery.
- **K-11.12 [E, implementation]** Add `crates/orchestrator/src/github_pr/evidence_render.rs`.
- **K-11.13 [E, CLI]** Add `elai pr-review --emit-evidence`.
- **K-11.14 [E, default]** Make evidence emission default-ON.
- **K-11.15 [E, kill switch]** Allow `ELAI_PR_EVIDENCE_RENDER_DISABLE` to disable the render.
- **K-11.16 [E, state]** Prefer existing SSB rows; add at most `EvidenceArtifactRendered`.
- **K-11.17 [E, operations]** Gate live check-run posting on network availability/permission under W-10.
- **K-11.18 [E, delivery boundary]** Resolve the PR-auto-fix doctrine tension by shipping poll-and-surface only in the wedge slice.
- **K-11.19 [E, human authority]** Keep merge human-controlled.
- **K-11.20 [E, git safety]** Do not autonomously force-push.
- **K-11.21 [E, grounding limit]** Treat the kunchen “engineering manager, not diff reviewer” statement as an AUTHOR/market-validation claim, not an ELAI measurement.

### K-12 — losslessness enforcement + handoff-faithfulness gate

Anchor: lines 311-331. KAT wave; CANT-LIE; STRONG code grounding; BUILD gate; effort S-M; wave KW-3; KD-1.

- **K-12.01 [E, compaction gate]** Change `check_lossless_compaction` from WARN to fail-closed when `unrecoverable_count > 0`.
- **K-12.02 [E, archival invariant]** Call a compacted record lossless only if archival ran first.
- **K-12.03 [E, missing evidence]** Treat missing R1 archival/wiring evidence as FAIL.
- **K-12.04 [E, schema]** Add `critical_atoms { kind, value, must_preserve }` to I-11 `BeliefState`.
- **K-12.05 [E, handoff gate]** After any model/lane handoff, assert every must-preserve atom's exact string/symbol is present in the SSB record.
- **K-12.06 [E, pristine rebuild]** Run the same assertion after I-5 pristine rebuild.
- **K-12.07 [E, trimming]** Run the same assertion after trimming/compaction.
- **K-12.08 [E, fail-closed]** Fail the handoff/trim when any must-preserve atom is missing.
- **K-12.09 [E, determinism]** Implement Critical-Atom-Recall as an exact deterministic check.
- **K-12.10 [E, prohibition]** Never use semantic similarity to satisfy the gate.
- **K-12.11 [E, prohibition]** Never use a generative model or ciphered comparison to adjudicate the gate.
- **K-12.12 [E, reuse]** Reuse H-49 RV-2, I-11, SessionDAG recover stubs, I-5, and G-2 verification; do not reimplement them.
- **K-12.13 [E, status correction]** Correct stale feature-map labels: I-5/I-11/H-50 are BUILT.
- **K-12.14 [E, integration]** Wire fail-close in `compact.rs`.
- **K-12.15 [E, integration]** Add the CriticalAtom schema in `belief_state.rs` and a `handoff_faithfulness.rs` gate.
- **K-12.16 [E, integration]** Wire the gate at `escalation_waterfall.rs:239` and the live compaction path.
- **K-12.17 [E, default/rollout]** Make the gate default-ON behind `ELAI_LOSSLESS_GATE_DISABLE`, while using observe-then-enforce rollout under GD-4.
- **K-12.18 [E, state]** Prefer `BeliefStateEvent` plus a Kind; add at most `HandoffFaithfulnessGated`.
- **K-12.19 [E, UI]** Add no separate frontend; verdicts flow to existing context/trust panels.
- **K-12.20 [E, test]** A compaction with any unrecoverable item must return `Err`, not warn.
- **K-12.21 [E, integration smoke]** A HARD multi-turn local-to-cloud handoff that drops an atom must fail R10.
- **K-12.22 [E, negative]** A paraphrase does not satisfy exact preservation.
- **K-12.23 [E, authority]** The LLM never sets `must_preserve` and never adjudicates the result.

### K-13 — OWASP control-mapping dossier + injection-defense posture proof

Anchor: lines 334-357. OGT wave; PROVABLE+AUDITABLE; STRONG web/code grounding; E-23 PACKAGE+PROVE; effort S; wave KW-3; KD-1.

- **K-13.01 [E, document]** Create `docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md`.
- **K-13.02 [E, coverage]** Include one row for each OWASP LLM01-LLM10 (2025).
- **K-13.03 [E, coverage]** Include one row for each OWASP ASI01-ASI10 (2025).
- **K-13.04 [E, cell schema]** Each cell names ELAI control(s).
- **K-13.05 [E, cell schema]** Each cell includes a code/plan citation.
- **K-13.06 [E, cell schema]** Each cell carries exactly an honest status such as `WIRED-MEASURED`, `WIRED-DESIGN`, `PARTIAL`, `PLANNED`, or `N-A`.
- **K-13.07 [E, claims]** Each cell has an ELAI-measured number or an explicit `[DESIGN]`/`[EXTERNAL]` tag.
- **K-13.08 [E, boundary]** Treat model inversion/extraction as N/A only for the current single-user-local boundary.
- **K-13.09 [E, lifecycle]** Reopen that N/A boundary for multi-seat under J-59.
- **K-13.10 [E, proof]** Bind prompt-injection controls LLM01/LLM05/ASI01 to the G-6 SAFEDELTA AgentDojo gate-ON-versus-OFF delta.
- **K-13.11 [E, honest status]** Mark that injection row `[BENCH-PENDING]` until A-2/A-3/AgentDojo runs land.
- **K-13.12 [E, reuse]** Extend K-1 dossier/lint; do not fork them.
- **K-13.13 [E, first-use fact]** This is the repository's first OWASP-string mapping according to the summary's authoring grep.
- **K-13.14 [E, buyer integration]** Cross-link the mapping into K-6 as a procurement clause without amending/rebuilding K-6.
- **K-13.15 [E, buyer integration]** Let K-7 answer the buyer's “OWASP coverage?” question from this mapping.
- **K-13.16 [E, reused controls]** Map existing controls G-1/F-9/F-2/F-1/F-3/G-9/I-1/I-50/E-30/Tier-C/F-4/H-53/G-5/E-24/G-4/C-42/E-43/B-22 rather than restating them.
- **K-13.17 [E, state/UI]** Add no SSB family and no frontend.
- **K-13.18 [E, done gate]** The K-1 lint must pass only when every mapping cell is tagged.
- **K-13.19 [E, negative]** A “fully covered” cell without a number/tag fails.
- **K-13.20 [E, negative]** An “OWASP certified” superlative fails.
- **K-13.21 [E, prohibition]** Fabricating the pending injection number is banned.
- **K-13.22 [E, OUT-OF-K]** WI-1 transformation-surviving taint, WI-2 cross-process taint, and WI-3 E-37 perimeter completion belong to an E-37/G-1 technical extension, not K.
- **K-13.23 [E, OUT-OF-K]** WI-6 config-ingress admission belongs to an E-43/E-24 technical extension, not K.

### K-14 — time-awareness determinism doctrine + replayable-async-tools differentiator

Anchor: lines 358-385. OGT wave; CANT-WRECK+PROVABLE; STRONG grounding; E-23 doctrine+claim; effort S; wave KW-3; KD-1/KD-2.

- **K-14.01 [E, doctrine]** Add `.claude/rules/time-determinism.md`.
- **K-14.02 [E, TZ-1]** Put volatile time strictly after the last cache breakpoint, never in system/rules prompts.
- **K-14.03 [E, TZ-1 rationale]** Preserve cache/replay behavior by obeying the R-1 mitigation.
- **K-14.04 [E, TZ-2]** Source time through `OsClock`.
- **K-14.05 [E, TZ-2]** Journal the exact time bytes supplied as model input.
- **K-14.06 [E, TZ-2 replay]** Replay those bytes rather than recomputing time.
- **K-14.07 [E, TZ-2 determinism]** Keep `SeededClock` gate/journal byte identity.
- **K-14.08 [E, TZ-3]** Never use wall time as a gate input.
- **K-14.09 [E, TZ-3]** Never use wall time as a transition or promotion input.
- **K-14.10 [E, TZ-4]** Keep wall-clock kill on the supervisor liveness plane outside the replay set.
- **K-14.11 [E, TZ-4]** Record the wall-clock kill but do not evaluate it as replay evidence.
- **K-14.12 [E, TZ-4]** Use step/no-progress as the primary liveness criterion.
- **K-14.13 [E, TZ-4]** Permit wall-clock kill only in AUTONOMOUS mode.
- **K-14.14 [E, routing]** Load the doctrine through conditional routing for time, timestamp, watchdog, and async-tool work.
- **K-14.15 [E, differentiator]** Add IMP-9 to the K-1 dossier: decoupled async tools whose lifecycle lands in a deterministic, replayable, taint-tagged journal.
- **K-14.16 [E, honest status]** Tag IMP-9 `[DESIGN]`, with G3-G6 wired but the model-facing loop J-future.
- **K-14.17 [E, measurement]** Attach no measured number to IMP-9 until one exists.
- **K-14.18 [E, reuse]** Extend K-1/K-7/K-8 doctrine and existing clock/RRA/cache/async primitives; do not fork.
- **K-14.19 [E, state/UI]** Add no SSB family and no frontend.
- **K-14.20 [E, done gate]** A live relevant task must load the rule; K-1 lint must pass the tagged claim.
- **K-14.21 [E, negative]** An untagged “fully deterministic async tools” claim fails.
- **K-14.22 [E, audit]** Catch a time block in a system prompt or wall-clock in a gate at doctrine audit.
- **K-14.23 [E, OUT-OF-K]** T-1 timestamp slot, T-2 agent-time, T-3 H-51 resume header, T-4 async-tool loop, and T-5 watchdog promotion belong to a future J-family time track bound by TZ-1..TZ-4.

### K-15 — reasoning-budget bench + receipt-grounded-reasoning differentiator

Anchor: lines 388-412. GRC wave; PROVABLE+PACKAGE; STRONG live-substrate/paper grounding; BUILD instrument+doc; effort M; wave KW-3; BENCH-PENDING on J-83/J-85.

- **K-15.01 [E, CLI]** Add `elai bench reasoning-budget` through the bench-subcommand seam.
- **K-15.02 [E, curve]** Measure verified-outcome rate versus reasoning compute per `(model, task-type)`.
- **K-15.03 [E, calibration]** Derive a measured peak and knee from the curve.
- **K-15.04 [E, sibling]** Use the J-34 empirical context-calibration shape, with reasoning budget rather than context fill on the x-axis.
- **K-15.05 [E, ablation]** Compare GRC-v3 router ON with flat-budget OFF.
- **K-15.06 [E, determinism]** Run both arms under `ELAI_SEED`.
- **K-15.07 [E, outcome]** Put receipt-backed verified outcomes on the y-axis; never use pass@k or self-grading.
- **K-15.08 [E, provenance]** Make the curve GD-2 recomputable.
- **K-15.09 [E, consumer]** Supply the measured knee to J-84/J-85's overthinking ceiling.
- **K-15.10 [E, granularity]** Store the knee per model and task type.
- **K-15.11 [E, prohibition]** Never import the paper's 6K/7K knee as an ELAI threshold.
- **K-15.12 [E, delivery]** Ship the deterministic FIXTURE tier immediately.
- **K-15.13 [E, deferred]** Keep the ON arm BENCH-PENDING until J-83/J-85 exist.
- **K-15.14 [E, operations]** Keep the live local-lane run RunPod-gated.
- **K-15.15 [E, differentiator]** Add “receipt-grounded reasoning” to the K-1 dossier and K-7 brief.
- **K-15.16 [E, meaning]** Define the differentiator as confidence from an external receipt at every step, never from the chain self-grading.
- **K-15.17 [E, truth label]** Mark the differentiator `[DESIGN]`/`[BENCH-PENDING]`.
- **K-15.18 [E, reuse]** Reuse I-27 convergence, J-34 calibration storage, G-6 SAFEDELTA, H-9, and K-1/K-7; do not fork their harnesses.
- **K-15.19 [E, implementation]** Add `benchmarks/reasoning_budget_runner.rs`.
- **K-15.20 [E, state]** Prefer `ConvergenceCurveMeasured` with a `ReasoningBudgetArm` kind; add at most one new family.
- **K-15.21 [E, UI]** Reuse I-27 `ConvergencePanel` and H-9 `CloudPanel`, or explicitly justify N/A at edit time.
- **K-15.22 [E, proof]** The FIXTURE arm must run and emit a recomputable curve.
- **K-15.23 [E, pending proof]** The ON arm must eventually show verified-outcome lift at/near the knee versus OFF; this remains BENCH-PENDING.
- **K-15.24 [E, negative]** Reject an untagged “reasoning self-verifies” claim.
- **K-15.25 [E, negative]** Reject a bare unmeasured “GRC-v3 +20%” claim.
- **K-15.26 [E, negative]** Reject a paper-derived 7K knee represented as ELAI's measured knee.

### K-16 — reasoning-receipts discipline doctrine

Anchor: lines 413-438. GRC wave; CANT-LIE+DISCIPLINE; STRONG spine/paper grounding; E-23 doctrine; effort S; wave KW-3; pre-gates J-83..J-86.

- **K-16.01 [E, doctrine]** Add `.claude/rules/reasoning-receipts.md`.
- **K-16.02 [E, RR-1]** Treat a reasoning trace as monitoring/UX only.
- **K-16.03 [E, RR-1]** Never admit a reasoning trace as the confidence or selection signal.
- **K-16.04 [E, RR-1]** Require an external receipt for admissible confidence.
- **K-16.05 [E, RR-2]** Prefer a deterministic receipt over all other selector evidence.
- **K-16.06 [E, RR-2 fallback]** If no receipt exists, allow a statistical vote only over independent samples.
- **K-16.07 [E, RR-2 boundary]** Never use the vote fallback for factual recall; route factual recall to retrieval.
- **K-16.08 [E, RR-2 prohibition]** Ban LLM self-evaluation as a selector.
- **K-16.09 [E, RR-3]** Measure a reasoning-budget/overthinking knee per model and task type with K-15.
- **K-16.10 [E, RR-3 prohibition]** Never import a paper constant as the knee.
- **K-16.11 [E, RR-3 prohibition]** Never use context-window size as the reasoning-control lever.
- **K-16.12 [E, RR-4]** Commit and freeze receipt-confirmed state.
- **K-16.13 [E, routing]** Load the rule for reasoning-engine, GRC, reasoning-budget, and candidate-selection work.
- **K-16.14 [E, enforcement scope]** Keep this rule auditor-applied; add no new runtime gate.
- **K-16.15 [E, runtime basis]** Rely on shipped F-4/D-13/I-4/H-39 runtime archetypes rather than restating them.
- **K-16.16 [E, claims]** Extend K-1 lint for receipt-grounded wording and K-15's claim.
- **K-16.17 [E, state/UI]** Add no SSB family and no frontend.
- **K-16.18 [E, done gate]** A task building J-83..J-86 must load the rule through conditional routing.
- **K-16.19 [E, audit]** Catch an LLM-self-evaluating candidate selector in reviewed design.
- **K-16.20 [E, negative]** Fail an untagged “the model checks its own reasoning” claim.
- **K-16.21 [E, negative]** Fail a paper knee represented as ELAI's own threshold.

### K-17 — context-management measured + regime-aware differentiator

Anchor: lines 441-462. CMG wave; PROVABLE+PACKAGE; STRONG code grounding with external/BENCH-PENDING knee; E-23 doc+lint; effort S; wave KW-3; KD-1/KD-4.

- **K-17.01 [E, differentiator]** Add a K-1 dossier row stating that ELAI measures a context-degradation knee per `(model, lane)` against gate verdicts.
- **K-17.02 [E, cloud policy]** Describe the cloud regime as cache-stable-prefix plus late compaction.
- **K-17.03 [E, local policy]** Describe the local regime as window-minimization plus unload-early.
- **K-17.04 [E, replacement]** Replace folklore 0.75-local/0.85-cloud window fractions with the measured J-34 knee.
- **K-17.05 [E, replacement]** Replace fixed slot-order claims with measured calibration.
- **K-17.06 [E, truth label]** Mark the row `[DESIGN]` for the wired 10-slot allocator/regime split.
- **K-17.07 [E, truth label]** Mark measured lift `[BENCH-PENDING]` on J-34.
- **K-17.08 [E, no duplication]** Add no parallel context bench; J-34 `elai bench context-fill` is the number source.
- **K-17.09 [E, status]** Treat `context_budget.rs` 10-slot allocation and `context_policy.rs` regime split as WIRED/code-grounded.
- **K-17.10 [E, depreciation framing]** Treat generic caching/compaction as provider-absorbed; claim the durable verification-coupled measurement, not the bare feature.
- **K-17.11 [E, doctrine reuse]** Cite CL-1..CL-4; do not re-author them.
- **K-17.12 [E, claims]** Extend K-1 lint and K-7 brief.
- **K-17.13 [E, state]** Add no SSB family; point to J-34 `ContextCalibrationEvent`.
- **K-17.14 [E, UI]** Reuse `ContextStatePanel`/H-9 or justify N/A.
- **K-17.15 [E, done gate]** The dossier row must exist with both honest tags and its code basis.
- **K-17.16 [E, negative]** Fail an untagged “ELAI never loses context” claim.
- **K-17.17 [E, negative]** Fail an unmeasured “0% degradation at full window” claim.
- **K-17.18 [E, negative]** Fail an external RULER/Chroma knee represented as ELAI's knee.
- **K-17.19 [E, OUT-OF-K]** IMP-1 build J-34; IMP-2 wire `adaptive_reduce`; IMP-3 wire I-11 escalation condense; IMP-4 wire I-5 pristine injection plus J-78 access spec; IMP-5 wire J-16 plus J-38 retrieval; IMP-6 wire J-54 pool plus H-7 routing; IMP-7 wire I-1 supervisor plane; and IMP-8 wire J-19 vLLM belong to a J context-activation track bound by CL-1..CL-4.

### K-18 — case-reuse economics bench + per-customer-compounding-state differentiator

Anchor: lines 465-493. VCG wave; PROVABLE+PACKAGE; STRONG live-substrate/research grounding; BUILD instrument+doc; effort M; wave KW-3; reuse arm BENCH-PENDING on VCG J-track.

- **K-18.01 [E, CLI]** Add `elai bench case-reuse` through the shared bench seam.
- **K-18.02 [E, granularity]** Measure per `(bug-class, model)`.
- **K-18.03 [E, metric]** Measure iteration delta.
- **K-18.04 [E, metric]** Measure dollar delta.
- **K-18.05 [E, metric]** Measure recurrence-hit rate.
- **K-18.06 [E, ablation]** Compare case-reuse ON against cold/OFF.
- **K-18.07 [E, ON behavior]** Inject an F-4-PROVEN case node as a demonstration in the ON arm.
- **K-18.08 [E, determinism]** Run under `ELAI_SEED`.
- **K-18.09 [E, provenance]** Make results GD-2 recomputable.
- **K-18.10 [E, outcome]** Use external receipts/verified outcomes rather than pass@k or self-grading.
- **K-18.11 [E, safe slice]** Ship recurrence-hit observation first with zero reuse into generation.
- **K-18.12 [E, pending]** Keep reuse-ON economics BENCH-PENDING on the VCG J-track.
- **K-18.13 [E, discipline dependency]** Require K-19 execution-grounded fitness before activating reuse.
- **K-18.14 [E, differentiator]** Add “gets faster/cheaper on YOUR recurring bugs over time” to K-1/K-7.
- **K-18.15 [E, truth label]** Mark that differentiator `[DESIGN]`/`[BENCH-PENDING]`.
- **K-18.16 [E, market status]** Treat “no production number exists” and moat #2 ranking as research/positioning context, not a measured result.
- **K-18.17 [E, reuse]** Reuse I-27/G-6/H-9/F-4/G-2/failure taxonomy/B-29 and case-memory ingredients; do not fork their runners.
- **K-18.18 [E, implementation]** Add a runner mirroring convergence/SAFEDELTA.
- **K-18.19 [E, state]** Prefer an existing bench row/`ConvergenceCurveMeasured` with `CaseReuseArm`/`RecurrenceHit`; add at most one family.
- **K-18.20 [E, UI]** Reuse I-27/H-9 panels or justify N/A.
- **K-18.21 [E, done gate]** The recurrence-hit and fixture arms run now and emit recomputable numbers.
- **K-18.22 [E, pending proof]** The reuse-ON arm must eventually reduce iteration/dollar cost versus cold.
- **K-18.23 [E, negative]** Fail untagged “learns from your code” or “improves with use” claims.
- **K-18.24 [E, negative]** Fail a bare unmeasured “50% fewer iterations” claim.
- **K-18.25 [E, negative]** Fail byte-hash, LLM-helpfulness, or retrieval-count reuse-fitness designs.
- **K-18.26 [E, OUT-OF-K]** The VCG J-track mints a typed case only on F-4 proof, self-wires deterministic edges, adds a MEMORY/CASE canvas altitude, gates retrieval-time reuse, and defines episodic-to-skill-to-rule promotion; it is technical companion scope, not K-18 implementation.

### K-19 — self-improvement/case-reuse fitness discipline doctrine

Anchor: lines 494-519. VCG wave; CANT-WRECK+DISCIPLINE; STRONG spine/paper/live-gap grounding; E-23 doctrine; effort S; wave KW-3.

- **K-19.01 [E, doctrine]** Add `.claude/rules/self-improvement-fitness.md`.
- **K-19.02 [E, applicability]** Bind self-improvement, case-graph, archive-promotion, playbook-curation, memory-reuse, and DGM work.
- **K-19.03 [E, SF-1]** Admit execution-grounded fitness only.
- **K-19.04 [E, SF-1 prohibition]** Never use a byte hash as fitness; replace live GAP-1 `derived_alpha=blake3(config_bytes)`.
- **K-19.05 [E, SF-1 prohibition]** Never use LLM-judged helpfulness.
- **K-19.06 [E, SF-1 prohibition]** Never treat “it was retrieved” as fitness.
- **K-19.07 [E, SF-1 positive]** Require a gate-PROVEN pass on a held-out recurrence.
- **K-19.08 [E, SF-2]** Require a stronger-than-generator filter under D-13.
- **K-19.09 [E, SF-2]** Make fitness abstention-aware under RD-1.
- **K-19.10 [E, SF-2]** Detect traps behaviorally with measured recall, replacing live GAP-2 keyword grep.
- **K-19.11 [E, SF-3]** Roll out observe-first under GD-4.
- **K-19.12 [E, SF-3]** Keep reuse default-OFF and behind the wire-dark allowlist.
- **K-19.13 [E, SF-3 firewall]** Never inject reused cases into the auditor.
- **K-19.14 [E, SF-4]** Re-derive a case against the current commit and AST.
- **K-19.15 [E, SF-4]** Hard-filter superseded cases; do not merely narrate them as outdated.
- **K-19.16 [E, SF-4]** Never delete historical cases under reversibility doctrine.
- **K-19.17 [E, SF-4]** Quarantine poisoning under E-5/J-59.
- **K-19.18 [E, routing]** Load the rule through conditional routing for relevant work.
- **K-19.19 [E, enforcement scope]** Keep this auditor-applied; add no new runtime gate.
- **K-19.20 [E, claims]** Extend K-1 lint for self-improving/learns-from-code/improves-with-use language.
- **K-19.21 [E, state/UI]** Add no SSB family and no frontend.
- **K-19.22 [E, done gate]** A VCG or K-18 reuse task must load the rule.
- **K-19.23 [E, audit]** Catch a byte-hash/LLM-helpful fitness design.
- **K-19.24 [E, audit]** Catch reuse into the auditor.
- **K-19.25 [E, research rationale]** Preserve the summary's central-risk claim that self-graded improvement collapses, with external Misevolve/F-10 figures treated as grounding, not ELAI measurements.

### K-20 — re-executable-RRA differentiator

Anchor: lines 522-528. LFG wave; PROVABLE+PACKAGE; STRONG G-2/J-14 grounding; E-23 doc+lint; effort S; wave KW-3; KD-1/KD-2/KD-5; BENCH-PENDING.

- **K-20.01 [E, differentiator]** Add “re-executable, not just replayable, evidence” as the wedge in K-1/K-7.
- **K-20.02 [E, third-party property]** A third party reruns against the recorded G-1 taint-ingested external bytes.
- **K-20.03 [E, cassette]** Store those bytes in a content-keyed VCR cassette.
- **K-20.04 [E, environment]** Pin execution with an OCI digest, lockfile, and CAS fixtures.
- **K-20.05 [E, verdict]** Require a matching I-45 objective-sealed verdict.
- **K-20.06 [E, distinction]** Describe the property as external faithfulness absent from replay-and-trust or screenshot evidence.
- **K-20.07 [E, truth label]** Mark the dossier row `[DESIGN]`/`[BENCH-PENDING]`.
- **K-20.08 [E, honest tier]** Distinguish action-faithful cloud replay from token-faithful replay.
- **K-20.09 [E, token boundary]** Claim token faithfulness only on lanes where J-13 capability checks pass.
- **K-20.10 [E, drift]** Treat world drift as a fidelity downgrade; a cassette proves the old world, not the live world.
- **K-20.11 [E, no duplication]** Add no new bench; J-14's extension owns re-execution proof.
- **K-20.12 [E, reuse]** Compose G-2, J-14, G-1, I-45, I-18, G-9, A-4, K-3/K-11, and other K differentiator rows.
- **K-20.13 [E, orthogonality]** Keep re-executability distinct from K-21 completeness.
- **K-20.14 [E, integration]** Add the tagged row to `docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md`, extend K-1 lint, and add a K-7 line.
- **K-20.15 [E, state/UI]** Add no SSB family; point to future `ReproPackEvent`; ride K-3/K-11 or N/A.
- **K-20.16 [E, negative]** Fail an untagged “fully reproducible” claim.
- **K-20.17 [E, negative]** Fail “bit-for-bit re-execute the live world” because of world drift.
- **K-20.18 [E, negative]** Fail a token-faithful claim on an uncapped lane.
- **K-20.19 [E, OUT-OF-K]** Technical re-execution is a J-14 extension and is not authored in K.

### K-21 — journal-completeness discipline doctrine

Anchor: lines 529-535. LFG wave; CANT-WRECK+DISCIPLINE; STRONG spine/report grounding; E-23 doctrine; effort S; wave KW-3; pre-gates E1 J-track.

- **K-21.01 [E, doctrine]** Add `.claude/rules/journal-completeness.md`.
- **K-21.02 [E, applicability]** Bind completeness, side-effect mediation, kernel observation, reconciliation, and any trust-the-journal/RRA/canvas/derived-status work.
- **K-21.03 [E, JC-1]** Distinguish integrity from completeness.
- **K-21.04 [E, JC-1]** A hash chain proves a row was not altered; it does not prove every side effect wrote a row.
- **K-21.05 [E, JC-1]** An append-only log cannot determine its own omissions.
- **K-21.06 [E, JC-2]** Declare the high-value side-effect set that must be mediated or independently observed.
- **K-21.07 [E, JC-2 mediation]** Permit `SECCOMP_RET_USER_NOTIF` atomic commit extending G-3/Tier-C.
- **K-21.08 [E, JC-2 observation]** Alternatively use an independent eBPF/LSM observer.
- **K-21.09 [E, JC-2 normalization]** Require a normalization layer for observer events.
- **K-21.10 [E, JC-2 prohibition]** Never assume an effect was journaled.
- **K-21.11 [E, JC-3]** Treat reconciliation missingness as FAIL.
- **K-21.12 [E, JC-3 monitor]** Monitor the monitor with dropped-event counters.
- **K-21.13 [E, JC-3 fail-close]** Fail closed on dropped observer events.
- **K-21.14 [E, JC-3 state]** Treat observer impairment as first-class state/evidence.
- **K-21.15 [E, JC-3 determinism]** Make reconciliation deterministic; the LLM does not adjudicate.
- **K-21.16 [E, JC-3 rollout]** Use observe-first rollout.
- **K-21.17 [E, JC-4 scope]** State the honestly covered scope.
- **K-21.18 [E, JC-4 platform]** Describe the observer as Linux-only and Windows as mediation-only.
- **K-21.19 [E, JC-4 trust ceiling]** State that host trust requires a Rekor/TPM-IMA rider on I-18/G-8.
- **K-21.20 [E, routing]** Load the doctrine conditionally on relevant tasks.
- **K-21.21 [E, enforcement scope]** Keep this rule auditor-applied; add no runtime gate or SSB family.
- **K-21.22 [E, claims]** Extend K-1 lint for scoped completeness language.
- **K-21.23 [E, done gate]** A live observer/completeness task must load the rule.
- **K-21.24 [E, negative]** Fail “complete/tamper-proof journal” when untagged or when it conflates integrity with completeness.
- **K-21.25 [E, audit]** Catch SKIP-on-missing-observer reconciliation.
- **K-21.26 [E, audit]** Catch a Windows claim that advertises the Linux observer.
- **K-21.27 [E, OUT-OF-K]** The E1 technical track owns mediation extension, eBPF/LSM observation, reconciliation, and monitor-the-monitor runtime machinery.

### K-22 — doctrine-enforcement-coverage ledger

Anchor: lines 536-542. LFG wave; PROVABLE+DISCIPLINE; STRONG harness/report grounding; audit+tool+rule; effort S-M; wave KW-3; KD-1/KD-3.

- **K-22.01 [E, tool]** Add deterministic `.claude/tools/doctrine_coverage.py`.
- **K-22.02 [E, input]** Extract every doctrine clause from `CLAUDE.md`.
- **K-22.03 [E, input]** Extract every doctrine clause from `.claude/rules/*.md`.
- **K-22.04 [E, join]** Join clauses to enforcers through `.claude/gate-manifest.txt` and the `.claude/tools/` checker inventory.
- **K-22.05 [E, classification]** Tag genuinely frozen enforcement `FROZEN-CHECKER`.
- **K-22.06 [E, classification]** Tag human/auditor-only enforcement `AUDITOR-ADVISORY`.
- **K-22.07 [E, classification]** Tag missing/unmappable enforcement `UNENFORCED`.
- **K-22.08 [E, metric]** Emit the percentage of clauses enforced by frozen checkers.
- **K-22.09 [E, provenance]** Make the coverage percentage recomputable from the rulebook and manifest.
- **K-22.10 [E, backlog]** Emit a compilation backlog of unenforced clauses.
- **K-22.11 [E, authority]** Treat backlog items as user-confirmed proposals; never auto-build them.
- **K-22.12 [E, doctrine]** Add `.claude/rules/doctrine-enforcement.md`.
- **K-22.13 [E, edit contract]** Require every future rulebook edit to declare each clause's enforcement tag.
- **K-22.14 [E, determinism]** Never let the LLM grade enforcement.
- **K-22.15 [E, fail-closed classification]** An unmappable clause is `UNENFORCED`, never silently enforced.
- **K-22.16 [E, threat]** Measure the one-level-up residue where workers are gated but the rulebook controlling gates is itself ungated.
- **K-22.17 [E, reuse]** Consume `gate_freeze_check.py`, the gate manifest, checker inventory, and J-70 traceability; do not fork them.
- **K-22.18 [E, consumer]** Let K-4 consult this ledger to distinguish real frozen gates from advisory doctrine.
- **K-22.19 [E, integration]** Emit a `docs/strategy/` ledger and add a conditional-routing trigger plus K-1 lint extension.
- **K-22.20 [E, state/UI]** Add no SSB family and no frontend.
- **K-22.21 [E, done gate]** Run the tool and emit the per-clause ledger, coverage percentage, and backlog.
- **K-22.22 [E, done gate]** Load doctrine-enforcement rule on a live rulebook edit.
- **K-22.23 [E, negative]** A clause claiming a checker absent from the manifest is `UNENFORCED`.
- **K-22.24 [E, negative]** Fail an untagged “fully enforced doctrine/every rule is a frozen gate” claim.

### K-23 — multi-repo/workspace memory

Anchor: lines 545-551, with preamble lines 61-73 and cross-cutting lines 588-600. MPW wave; REACH+PROVABLE; STRONG code/comparator grounding; NET-NEW BUILD track; effort L; U1..U6 (~6-7 sessions); depends on J-18/J-80/J-2/H-36 for later slices.

- **K-23.01 [E, capability]** Add a thin cross-repository workspace layer above deep per-root ELAI substrate.
- **K-23.02 [E, gap]** Preserve existing per-root stores unchanged; current stores remain functions of one `project_root` with `BM25_PROJECT_ID="default"`.
- **K-23.03 [E, boundary]** Do not mistake Cosmos cross-project artifacts or canvas multi-project views for cross-repo code intelligence.
- **K-23.04 [E, U1 object]** Add a workspace object and manifest.
- **K-23.05 [E, U1 storage]** Store workspaces globally at `~/.elai/workspaces/<id>/`, beside Cosmos and never inside a member repo.
- **K-23.06 [E, U1 journal]** Add a global `workspace.ssb.db` workspace journal.
- **K-23.07 [E, U1 CLI]** Add `workspace create`.
- **K-23.08 [E, U1 CLI]** Add `workspace add`.
- **K-23.09 [E, U1 CLI]** Add `workspace map`.
- **K-23.10 [E, U2 projection]** Derive a `cross_repo_edges` projection.
- **K-23.11 [E, U2 cache]** Key the derived projection by a BLAKE3 hash of member DKG manifests and cascade invalidation on change.
- **K-23.12 [E, U2 edge class]** Classify LSP/SCIP-derived J-18 edges as `Sound`.
- **K-23.13 [E, U2 edge class]** Classify name/import-derived edges as `Heuristic`.
- **K-23.14 [E, correctness]** Permit Sound edges to drive gates.
- **K-23.15 [E, correctness]** Permit Heuristic edges to advise only; they cannot drive impact gates.
- **K-23.16 [E, U3 retrieval]** Fan a workspace query to every member's existing retrieval pipeline.
- **K-23.17 [E, U3 merge]** Merge member results with reciprocal-rank fusion at `k=60`.
- **K-23.18 [E, U3 provenance]** Preserve per-repository provenance on every result.
- **K-23.19 [E, U3 reuse]** Build no centralized/new index.
- **K-23.20 [E, U4 impact]** Extend the B-29/J-16 blast-radius cone across repositories.
- **K-23.21 [E, U4 precision]** Traverse Sound edges only for cross-repo gating.
- **K-23.22 [E, U5 CLI]** Add `workspace orchestrate`.
- **K-23.23 [E, U5 FSM]** Add a workspace FSM above each per-root FSM.
- **K-23.24 [E, U5 ordering]** Kahn-order member work from Sound dependencies.
- **K-23.25 [E, U5 delegation]** Dispatch Tier-C subagents into J-80 worktrees.
- **K-23.26 [E, U5 isolation]** Each delegated member runs its own existing FSM.
- **K-23.27 [E, U5 lifecycle]** Add a `WORKSPACE_MERGING` phase/state.
- **K-23.28 [E, U5 verification]** Run a cross-repository contract integration smoke under R10.
- **K-23.29 [E, U6 resume]** Resume from the workspace journal plus each member's H-51 digest.
- **K-23.30 [E, U6 provenance]** Attach `source_repo` provenance.
- **K-23.31 [E, U6 trust]** Extend J-21 poisoning/trust evaluation per repository.
- **K-23.32 [E, U6 UI]** Add a canvas `WORKSPACE` altitude beside J-31/J-60/J-98.
- **K-23.33 [E, mode]** In `CONNECTED` mode, expose real Sound edges.
- **K-23.34 [E, mode]** In `DISCONNECTED` mode, provide federation only with an empty edge set.
- **K-23.35 [E, honesty]** Never fabricate a relationship between disconnected repositories.
- **K-23.36 [E, reuse]** Reuse `paths.rs` unchanged and compose Cosmos, RRF, B-29/J-16, J-18, orchestrator, Tier-C, J-80, J-2/H-36, Kahn, context budget, H-51/J-21/C-48/J-59, SSB/FSM, and canvas substrate.
- **K-23.37 [E, implementation]** Add `crates/orchestrator/src/workspace/`.
- **K-23.38 [E, implementation]** Add workspace verbs under the CLI.
- **K-23.39 [E, implementation]** Extend `crates/orchestrator/src/blast_radius.rs`.
- **K-23.40 [E, state]** Add one Pattern-2 `WorkspaceEvent` family, subject to W-9 live-count confirmation.
- **K-23.41 [E, configuration]** Gate the track with `ELAI_WORKSPACE_ENABLE` and an allowlist promotion row.
- **K-23.42 [E, fixture]** On a two-repo fixture, `workspace map acme` reports exactly one Sound `types <- api` edge and zero false edges.
- **K-23.43 [E, impact proof]** `workspace impact` reports the real consumer in repo B with zero false consumers.
- **K-23.44 [E, orchestration proof]** `workspace orchestrate` lands two PRs and a green cross-repo contract test.
- **K-23.45 [E, record proof]** WorkspaceEvent rows in `workspace.ssb.db` are GD-2 recomputable.
- **K-23.46 [E, UI proof]** The canvas altitude re-derives byte-identically.
- **K-23.47 [E, negative]** A fabricated edge between disconnected members fails.
- **K-23.48 [E, negative]** A Heuristic edge driving the impact gate fails.
- **K-23.49 [E, negative]** A workspace store written under a member root fails.
- **K-23.50 [E, local-first]** A centralized cloud workspace store fails.
- **K-23.51 [E, deferred]** Cross-seat/org-shared team memory is out until a separate per-seat opt-in decision under J-49/J-59; it is not a prerequisite.

### K-24 — credit-based seats on an OpenRouter rail

Anchor: lines 554-560, with preamble lines 61-73 and cross-cutting lines 588-600. BCC wave; PROVABLE+EFFICIENT; STRONG code/provider grounding; NET-NEW BUILD track; effort L; L1..L6 (~6-7 sessions); depends on J-28/J-49/J-54/J-55; L6 RunPod-gated.

- **K-24.01 [E, business model]** Convert a subscription into monthly credits denominated in real provider dollars.
- **K-24.02 [E, metering]** Spend credits per metered provider usage.
- **K-24.03 [E, upstream]** ELAI pays upstream provider cost.
- **K-24.04 [E, controls]** ELAI owns limits, rollover, and routing policy.
- **K-24.05 [E, safety]** Cap every request; never market or implement an unlimited agent loop.
- **K-24.06 [E, value thesis]** Position the advantage as more resolved tasks per dollar through local routing, verification, and caching—not a provider deal.
- **K-24.07 [E, margin]** Treat the J-54 local/free pool, made correct by J-55, as the margin engine.
- **K-24.08 [E, L1 event]** Add `CreditLedgerEvent` as an SSB sub-journal event family.
- **K-24.09 [E, L1 fold]** Compute balance by folding `Granted - Settled - open Reserved - Expired`.
- **K-24.10 [E, L1 prohibition]** Never store balance as a mutable counter.
- **K-24.11 [E, L1 integrity]** Hash-chain the ledger using I-18/J-69.
- **K-24.12 [E, L1 concurrency]** Use SSB's single writer as double-spend defense.
- **K-24.13 [E, L1 CLI]** Add `billing balance`.
- **K-24.14 [E, L2 reservation]** Estimate request cost before dispatch.
- **K-24.15 [E, L2 reservation]** Emit a `Reserved` hold before the provider call.
- **K-24.16 [E, L2 metering]** Execute with J-29 live metering.
- **K-24.17 [E, L2 runaway safety]** Enforce the J-81/E-39 runaway cap during the call.
- **K-24.18 [E, L2 settlement]** Settle from returned actual cost.
- **K-24.19 [E, L2 release]** Release the unused reservation in the same tick.
- **K-24.20 [E, L3 backend]** Add a managed OpenRouter backend as B-37 backend #12.
- **K-24.21 [E, L3 dependency]** Implement the backend directly/hand-rolled per J-54; add no new dependency.
- **K-24.22 [E, L3 booking]** Capture actual cost through J-28.
- **K-24.23 [E, L4 admission]** Gate J-49 seat admission on credit balance.
- **K-24.24 [E, L4 limits]** Apply tier rate limits.
- **K-24.25 [E, L4 failure]** Refuse out-of-credit work with a clear user-facing surface.
- **K-24.26 [E, L5 refill]** Refill credits on subscription renewal.
- **K-24.27 [E, L5 rollover]** Apply rollover rules defined as data in `seats.toml`.
- **K-24.28 [E, L5 webhook]** Translate payment webhooks into `Granted` and `Expired` events.
- **K-24.29 [E, L5 boundary]** Keep Stripe/Paddle outside ELAI and maintain zero PCI scope.
- **K-24.30 [E, L6 measurement]** Run SAFEDELTA for dollars per resolved task.
- **K-24.31 [E, L6 claims gate]** Gate any “$20=$200” style claim on the ELAI-measured result.
- **K-24.32 [E, L6 business gate]** Gate launch on measured local-lane resolve rate because frontier-margin cost remains material.
- **K-24.33 [E, registry boundary]** Use OpenRouter as transport+meter, never as the dynamic model registry; preserve I-51's registry rejection.
- **K-24.34 [E, fallback]** Preserve B-37 direct-provider fallback.
- **K-24.35 [E, implementation]** Add `crates/orchestrator/src/billing/`, `openrouter_backend.rs`, and `elai billing` CLI.
- **K-24.36 [E, integration]** Wire J-49 admission, J-28 booking, and an H-9 CloudPanel billing tile.
- **K-24.37 [E, event kinds]** Support `Granted`, `Reserved`, `Settled`, `Refunded`, `Expired`, and `AdjustmentManual` kinds.
- **K-24.38 [E, configuration]** Gate the track with `ELAI_BILLING_ENABLE` and an allowlist promotion row.
- **K-24.39 [E, proof]** A live OpenRouter call books `Settled == returned actual cost`.
- **K-24.40 [E, proof]** `billing balance` folds correctly.
- **K-24.41 [E, proof]** A renewal webhook emits `Granted(face+bonus)` plus `Expired(over-cap)`.
- **K-24.42 [E, negative]** A mutable-counter balance fails.
- **K-24.43 [E, split-brain]** Using OpenRouter/LiteLLM spend storage as ELAI's ledger fails.
- **K-24.44 [E, negative]** A settlement different from metered actual cost fails.
- **K-24.45 [E, negative]** An unsupported “$20=$200” claim fails K-1 lint.
- **K-24.46 [E, copy constraint]** Pricing copy never says “unlimited.”
- **K-24.47 [E, positioning]** Sell as “proven more value per dollar, verified,” never merely “cheap.”
- **K-24.48 [E, GTM hypothesis]** Treat a $39-49/team-or-agency seat as a plausible first revenue path from GTM memory, not a verified launch fact.

### K-25 — continuous absorption watchdog

Anchor: lines 563-587, with preamble lines 74-81. CAW wave; CANT-WASTE; STRONG spec-eval/K-8 grounding; audit tool+rule; effort S-M; KW-1-adjacent; depends on K-8.

- **K-25.01 [E, snapshot]** Add `workflow/status/provider-surface.json`.
- **K-25.02 [E, schema]** Each snapshot row contains `provider`.
- **K-25.03 [E, schema]** Each snapshot row contains `capability_key`.
- **K-25.04 [E, schema]** Each snapshot row has status `NATIVE`, `PARTIAL`, or `ABSENT`.
- **K-25.05 [E, schema]** Each snapshot row contains `source_url`.
- **K-25.06 [E, schema]** Each snapshot row contains `verified_date`.
- **K-25.07 [E, parse contract]** A row without a source or date fails parsing.
- **K-25.08 [E, tool]** Add stdlib `.claude/tools/absorption_watch.py`.
- **K-25.09 [E, CLI]** Support `--json` output.
- **K-25.10 [E, CLI]** Support `--max-age-days`, default 90.
- **K-25.11 [E, join]** Join the provider snapshot to K-8 `ABSORBED`/`AT-RISK`/`DURABLE` feature-map tags.
- **K-25.12 [E, exit code]** Exit 0 for clean/no overlap.
- **K-25.13 [E, exit code]** Exit 1 for `NATIVE_OVERLAP`, naming the overlapping rows.
- **K-25.14 [E, exit code]** Exit 2 for parse errors and fail closed.
- **K-25.15 [E, exit code]** Exit 3 for a stale snapshot.
- **K-25.16 [E, doctrine]** Add `.claude/rules/absorption-watch.md` on K-8's conditional-routing lane.
- **K-25.17 [E, stale behavior]** On exit 3, refresh via WebSearch and re-date sources before executing the efficiency task.
- **K-25.18 [E, overlap behavior]** Surface exit-1 rows as kill/de-scope proposals.
- **K-25.19 [E, authority]** Never auto-apply those proposals.
- **K-25.20 [E, epistemic boundary]** Declare the keyword join unsound and advisory/LLM-adjudicated.
- **K-25.21 [E, done-gate boundary]** Never let the watchdog verdict discharge a runtime done gate or audit PASS.
- **K-25.22 [E, precedent]** Mirror the `elai_status_query.py` Layer-0 advisory-cache boundary.
- **K-25.23 [E, sibling]** Let K-9's cadence serve as a natural provider-snapshot refresh trigger without merging their axes.
- **K-25.24 [E, state/UI]** Add no SSB family and no frontend.
- **K-25.25 [E, proof-spine status]** Mark the unit spine-exempt; it contributes no PROOF_SPINE number.
- **K-25.26 [E, fixture]** One NATIVE snapshot row overlapping one planned unit returns exit 1 and names the pair.
- **K-25.27 [E, fixture]** A stale snapshot returns exit 3 and never silently reports clean.
- **K-25.28 [E, fixture]** A malformed row returns exit 2.
- **K-25.29 [E, integration test]** A live efficiency task loads the rule and follows the exit code.
- **K-25.30 [E, violation]** Citing the advisory checker as a done-gate/audit PASS is a violation.
- **K-25.31 [E, violation]** Auto-applying de-scope remains a house-rule violation.

### K-26 — codex-burn proof-spine campaign (preamble-only)

Anchor: lines 82-91 only. No normal summary entry exists; inspect the original before treating this as a feature plan.

- **K-26.01 [E, campaign]** Define a hard-deadlined five-day operations/build campaign to spend two expiring `bmapi`/`codex-proxy` credit pools.
- **K-26.02 [E, historical deadline]** Record the stated credit expiry as 2026-07-15; this is historical/stale operational context, not a continuing product behavior.
- **K-26.03 [E, proof target]** Feed proof-spine objectives PS-2, PS-4, and PS-5.
- **K-26.04 [E, work item]** Finish and commit J-28.
- **K-26.05 [E, work item]** Run the SP-3B GPT-5.5 live smoke.
- **K-26.06 [E, dogfood]** Put real `.elai_cc` fixes through `elai run` for model-agnostic accrual.
- **K-26.07 [E, seam gap]** Record FORK-5: `elai run` lacks a GPT-5.5 seam.
- **K-26.08 [E, route A]** Option A widens B-27's Anthropic-only validation as dogfood task zero.
- **K-26.09 [E, route B]** Option B uses an existing lane plus per-call GPT-5.5 Agent-tool subagents.
- **K-26.10 [E, work item]** Lock the K-4 gate-tax ceiling for PS-2.
- **K-26.11 [E, work item]** Run the PS-5 journal probe.
- **K-26.12 [E, conditional build]** Build `--suite journal` conditionally from the probe result.
- **K-26.13 [E, work item]** Refine-before-execute and build K-2 for PS-4.
- **K-26.14 [E, operations]** Rotate from account A to account B, then burn surplus credits.
- **K-26.15 [E, plan validation]** Record `plan_gate` PASS 10/10.
- **K-26.16 [E, red team]** Record a non-author red team with PLAN-BLOCK v1.
- **K-26.17 [E, repaired defect]** Patch the `elai run` model-seam gap found by the red team.
- **K-26.18 [E, repaired defect]** Patch the wrong VERIFY-query table name found by the red team.
- **K-26.19 [E, scope exclusion]** Leave PS-1 RunPod-gated and out of the campaign.
- **K-26.20 [E, scope exclusion]** Leave PS-3 in L-track and out of the campaign.
- **K-26.21 [E, scope exclusion]** Leave unrelated harness fixes out.
- **K-26.22 [A, summary drift]** The summary title still says “all 24 sessions” and the body ends at K-25 even though this K-26 preamble was appended; the original is mandatory for completion/status.

## How the K features work together

### 1. Falsify, measure, govern the claim, then distribute it

1. **A-1** runs first and can kill the wedge before K builds around it.
2. **A-2/A-3**, **K-2**, **K-4**, **K-15**, **K-18**, and the J-owned proof legs produce measured results; fixture and pending/live tiers remain visibly distinct.
3. **K-1** is the central truth registry and lint: every external number, design statement, and measured claim receives an honest status and recompute/source pointer.
4. **K-3** turns closed-set machine evidence into PR/check-run verdicts; **K-11** adds the human evidence view and DSR prioritization.
5. **K-6**, **K-7**, **K-13**, **K-14**, **K-15**, **K-17**, **K-18**, and **K-20** reuse K-1 rather than inventing separate claim-governance systems.

### 2. Make verification affordable without letting the model weaken it

1. **K-4** measures composed wall-clock and over-block cost across the whole gate stack and locks the acceptable ceiling from real data.
2. **K-22** tells K-4 which doctrine clauses have real frozen enforcement versus advisory-only enforcement.
3. **K-5** chooses deterministic intensity using risk facts, keeps cheap protections always on, and escalates expensive gates only on fixed triggers.
4. The chosen K-5 policy hash is frozen; the model cannot select or modify its own tier.
5. At least three real runs must demonstrate K-5 remains under K-4's ceiling.

### 3. Package one evidence spine for code review and procurement

1. Gate/test/taint/attestation/cone evidence is born in F/G/H/J substrate and recorded in the journal/RRA.
2. **K-3/K-11** render the same evidence set in the PR and cockpit; missing evidence becomes `UNVOUCHED`.
3. **K-6** composes the redacted RRA, attestation, audit extract, retention mode, and clause mapping into one offline-verifiable bundle.
4. **K-13** cross-links OWASP controls into that procurement view without claiming certification.
5. **K-7** uses only K-1-governed numbers and is human-reviewed before external use.

### 4. Keep provider/model choices fresh without automated destructive decisions

1. **K-8** classifies efficiency work as absorbed, at-risk, or durable and produces only a proposal for removal.
2. **K-25** persists a dated provider-native snapshot, fails stale/malformed state with distinct exit codes, and requires refresh before execution.
3. **K-25** remains advisory: its keyword join cannot satisfy a done gate and cannot remove work.
4. **K-9** evaluates the separate model-lane axis quarterly; regressions keep the incumbent and adoption still needs user ratification.

### 5. Convert research claims into measured, bounded differentiators

- **K-14** constrains future time/async work so volatile context remains replay-compatible while supervisor liveness stays outside replay authority.
- **K-15/K-16** pair a measured reasoning-budget knee with a doctrine that admits receipts, independent votes only as fallback, and never self-evaluation.
- **K-17** packages J-34's per-model/lane context knee and opposite cloud/local regimes; it adds no duplicate bench.
- **K-18/K-19** pair case-reuse economics with execution-grounded, stronger-than-generator, abstention-aware fitness and an auditor firewall.
- **K-20/K-21** keep three properties separate: journal integrity, journal completeness, and re-executable external faithfulness.
- **K-22** measures how much of ELAI's own doctrine is actually compiled into frozen enforcement.

### 6. Add the two net-new K capability tracks without bypassing the spine

- **K-23 workspace:** each root retains its own deep index/FSM; the workspace layer derives Sound cross-repo edges, federates retrieval, expands impact only over Sound edges, orders delegated per-root work, journals workspace state globally, and renders a deterministic canvas altitude.
- **K-24 billing:** seat admission reserves a hash-chained credit balance, meters the real provider call under runaway caps, settles actual cost, and folds balance from events; business claims remain blocked until SAFEDELTA measures dollars per resolved task.
- Both tracks are default-off/allowlisted, add exactly one intended event family, retain local-first record ownership, and require real integration smokes.

## Cross-cutting constraints and invariants

- **Claim truth:** measured ELAI number plus recompute pointer, or explicit design/external/pending status; never a bare superlative or imported research number presented as ELAI's result.
- **Closed-set evidence:** render only evidence present in the RRA/manifest; no artifact means `UNVOUCHED`, not clean.
- **Fail-closed missingness:** missing cost capture, archival, critical atom, observer event, enforcer mapping, attestation digest, or snapshot schema cannot silently pass.
- **Deterministic authority:** the LLM does not choose gate intensity, set must-preserve atoms, adjudicate critical-atom recall, select from self-evaluation, grade doctrine enforcement, or reconcile observer completeness.
- **Frozen judgment:** a run cannot mutate its own gate-intensity table; rule/checker claims are grounded in the frozen gate manifest.
- **External receipts:** reasoning traces and self-reports are UX only; execution/test/retrieval receipts decide confidence and selection.
- **Integrity is not completeness:** I-18/J-69 hash chaining protects recorded rows, while K-21 separately demands mediation/observation and reconciliation for omissions.
- **Exact handoff fidelity:** must-preserve atoms are exact-string/symbol obligations; paraphrase is failure.
- **Time determinism:** volatile time comes after cache breakpoints, exact time input is journaled, wall time never decides gates/transitions/promotions, and supervisor wall-clock kill is outside replay.
- **Human governance:** removal, model adoption/routing change, unenforced-clause compilation, and cross-seat sharing need explicit user decisions.
- **Observe-first/default-off:** risky case reuse, observer enforcement, new workspace, and billing promotion follow observe-first or allowlisted activation as specified.
- **No duplicate substrate:** K renderers, benches, doctrine, exports, and differentiator rows extend named F/G/H/I/J/K components rather than fork them.
- **Local-first storage:** workspace state cannot move under an arbitrary member root or into a centralized cloud store; credit truth cannot be delegated to a provider spend store.
- **Privacy/security:** compliance export must be redacted; payment processors remain outside ELAI/PCI scope; reused cases never enter the auditor; repository provenance is retained.
- **Honest platform/mode boundaries:** Linux observation is not advertised on Windows; disconnected repositories get no invented edges; cloud/action-faithful replay is not mislabeled token-faithful; cassettes prove an old captured world.
- **Cost safety:** billing requests are reserved and capped, settlements equal actual metering, and “unlimited” is forbidden.
- **Advisory-cache boundary:** provider overlap detection can prompt a refresh/proposal but can never discharge objective verification.

## Dependency, status, and deferred-work ledger

| Unit | Explicit status/nature in summary | Blocking or composing dependencies | Deferred, gated, or excluded leg |
|---|---|---|---|
| K-1 | Planned doc+lint; STRONG grounding | I-43/A-1, A-2/A-3, GD-2/3/5 | KW-6 pending verification |
| K-2 | BUILD; fixture tier now | H-7, I-25, I-27, H-9 | Live dollars blocked on J-28 |
| K-3 | BUILD renderer over WIRED adapter | C-40/E-41, F-1/2/4/9, G-5/8, H-52, J-16 | J-16 summary only when available; real PR required |
| K-4 | BUILD instrument; K-T3 keystone | I-38, G-6, gate inventory, E-49 | Ceiling cannot be chosen before first measurement |
| K-5 | BUILD frozen policy | K-4, B-22, C-30, J-16, F/I/H/E gates | Promotion requires >=3 real runs under ceiling |
| K-6 | BUILD composition | G-2/G-8/G-9/I-44/I-18 | No new attestation; optional UI undecided |
| K-7 | Pure prose | K-1, F-2/F-9, G-8/G-9/I-44 | External use waits on user review |
| K-8 | Audit+rule | Feature map, W-7, absorption record | Removal always user-confirmed |
| K-9 | Process+bench wrapper | J-9, E-27, J-19, E-31, I-16 | Swap waits on fixture result and user ratification |
| K-11 | BUILD renderer; default-ON design | K-3/E-41/G-5/G-2/F-4/H-52 | Live posting network-gated; auto-fix/merge excluded |
| K-12 | BUILD gate over built substrate | H-49, I-11, I-5, SessionDAG, G-2 | Observe-then-enforce rollout; exact match only |
| K-13 | E-23 PACKAGE+PROVE | K-1/K-6/K-7/G-6 plus existing controls | Injection row BENCH-PENDING; WI-1/2/3/6 OUT-OF-K |
| K-14 | E-23 doctrine+claim | K-1/K-7/K-8, clocks/RRA/G3-G6 | Model-facing async loop and T-1..T-5 OUT-OF-K |
| K-15 | BUILD fixture instrument+doc | I-27/J-34/G-6/H-9, J-83/J-85 | ON arm BENCH-PENDING; live local run RunPod-gated |
| K-16 | Auditor-applied doctrine | K-15, F-4/D-13/I-4/H-39, J-83..86 | No new runtime gate |
| K-17 | E-23 doc+lint; allocator/policy cited WIRED | J-34, context files, CL-1..4 | Lift BENCH-PENDING; IMP-1..8 OUT-OF-K |
| K-18 | BUILD fixture/observe arm+doc | I-27/G-6/F-4/G-2/B-29, K-19, VCG J-track | Reuse-ON economics BENCH-PENDING; VCG engine OUT-OF-K |
| K-19 | Auditor-applied doctrine | D/H/J trust substrate, K-18, VCG | Runtime gate not added; reuse remains default-OFF |
| K-20 | E-23 differentiator row | G-2/J-14/G-1/I-45/I-18/G-9/K-1/K-7 | Re-execution proof is J-14 extension, BENCH-PENDING |
| K-21 | Auditor-applied doctrine | G-3/Tier-C/SSB/I-18/J-69/G-2 | E1 runtime observer/mediation/reconcile machinery OUT-OF-K |
| K-22 | Planned audit tool+rule | gate manifest/freezer/checker inventory/J-70/K-1/K-4 | Backlog is proposal only; no auto compilation |
| K-23 | NET-NEW L track-in-one-file | J-18, J-80, J-2/H-36 and reused per-root stack | Cross-seat sharing OUT; enable/allowlist required |
| K-24 | NET-NEW L track-in-one-file | J-28, J-49, J-54/J-55, J-29/J-81/B-37 | L6 RunPod-gated; payment processor external |
| K-25 | Planned audit tool+rule; no SSB/spine number | K-8; K-9 is refresh sibling | Advisory only; stale data must be refreshed |
| K-26 | Preamble-only historical ops/build campaign | PS-2/4/5, J-28, K-2/K-4 | PS-1, PS-3, harness fixes OUT; original needed for actual status |

### Explicit technical companions that are not K deliverables

- **OWASP:** WI-1 transformation-surviving taint binding, WI-2 cross-process taint, WI-3 E-37 perimeter completion, WI-6 config-ingress admission.
- **Time:** T-1 timestamp slot, T-2 agent time, T-3 resume header, T-4 async-tool model loop, T-5 watchdog promotion.
- **GRC-v3:** J-83..J-86 technical reasoning-engine track.
- **Context activation:** IMP-1..IMP-8 as enumerated under K-17.
- **Verified Case Graph:** proof-minted typed nodes, deterministic edges, MEMORY/CASE canvas altitude, gated reuse, episodic-to-skill-to-rule ladder.
- **Lacuna E1:** G-3/Tier-C mediation extension, eBPF/LSM observer, reconciliation, and monitor-the-monitor.
- **Other lacuna fills:** E2 runtime-observed DKG edges; E3 machine-derived-first audit lane; L1 objective-correctness spec bundle plus cross-family jury; L8 plan-DAG producer/consumer type completeness; L3 J-14 re-execution; L4 usage-weighted reachability; L5 cross-language/dynamic blast cone; L6 orchestrator-FSM dispatch ordering; L7 cant-lie X mutation floor; and unspecified Tier-2/Tier-3/THIN fills.

## Ambiguity and inference ledger

- **A-01 [A, stale title]** The title says “all 24 sessions” and “6 tracks K-T1..K-T6,” but the detailed body has 24 normal entries, headings only for K-T1..K-T5, plus a later K-26 preamble. K-T6's mapping is not recoverable from this summary.
- **A-02 [A, K-10]** K-10 is described only as a sequencing/no-new-letters doctrine in `K-0_INDEX.md`; there is no `K-10_*.md` file in the current K directory.
- **A-03 [A, K-26]** K-26's campaign has detailed preamble bullets but no normal track/lever/integration/observable-effect section, and its July 2026 credit deadline is historical.
- **A-04 [A, K-27]** `K-27_gpt56_sol_isolated_executor_lane.md` exists on disk but is not mentioned in the 600-line summary. Its filename alone is insufficient to inventory behavior.
- **A-05 [A, K-18a]** `K-18a_ps5_observe_arm_testrun.md` exists on disk but is not mentioned in the summary. It may relate to PS-5/K-18, but that relationship is not asserted here.
- **A-06 [A, rollout]** K-12 says default-ON behind a disable flag and also “observe-then-enforce”; the original must clarify the rollout transition and initial default.
- **A-07 [A, event counts]** “LIVE 268/577” and several net `0..+1` estimates are authoring snapshots with W-9 re-grep instructions, not stable present-day schema counts.
- **A-08 [A, market claims]** “Industry has no number,” moat ranking, Polygraph comparisons, OpenRouter/LiteLLM facts, and paper collapse figures are grounding claims, not ELAI feature behavior; originals/grounding must preserve citations.
- **A-09 [I, completed-system composition]** Once K-23 and K-24 are both built, workspace-orchestrated calls would presumably pass through the ordinary seat/meter controls because each member runs its own FSM, but the K summary never explicitly specifies workspace-billing interaction. Do not encode this presumed coupling without original/cross-plan confirmation.
- **A-10 [I, event unification]** Later N may canonicalize K event/measurement families, but K itself only states preferred reuse/net-at-most-one. This discovery preserves K's intent and does not preempt N's final schema.

## Exact deeper-source manifest

No file in this section was opened for feature extraction during this summary-layer pass. Line counts and existence were checked only to make handoff complete.

### A. K-plan originals for the 24 normal summary entries

Base path: `docs/thefinalglance/K-plan/`

| Summary ID | Exact file | Lines |
|---|---|---:|
| K-1 | `K-1_problem_quantification_dossier_claims_policy.md` | 162 |
| K-2 | `K-2_routing_economics_bench.md` | 175 |
| K-3 | `K-3_pr_ci_evidence_emitter.md` | 164 |
| K-4 | `K-4_composed_gate_tax_budget.md` | 379 |
| K-5 | `K-5_risk_tiered_gating_policy.md` | 154 |
| K-6 | `K-6_compliance_export.md` | 156 |
| K-7 | `K-7_appsec_insurance_positioning_brief.md` | 119 |
| K-8 | `K-8_absorbable_unit_audit_provider_pregate.md` | 143 |
| K-9 | `K-9_quarterly_model_refresh_cadence.md` | 152 |
| K-11 | `K-11_human_viewable_evidence_render.md` | 108 |
| K-12 | `K-12_losslessness_enforcement_handoff_faithfulness_gate.md` | 113 |
| K-13 | `K-13_owasp_control_mapping_injection_posture.md` | 220 |
| K-14 | `K-14_time_awareness_determinism_doctrine.md` | 213 |
| K-15 | `K-15_reasoning_budget_bench_receipt_grounded_differentiator.md` | 121 |
| K-16 | `K-16_reasoning_receipts_doctrine.md` | 121 |
| K-17 | `K-17_context_management_measured_regime_aware_differentiator.md` | 110 |
| K-18 | `K-18_case_reuse_economics_compounding_state_differentiator.md` | 119 |
| K-19 | `K-19_self_improvement_fitness_doctrine.md` | 123 |
| K-20 | `K-20_reexecutable_rra_differentiator.md` | 113 |
| K-21 | `K-21_journal_completeness_doctrine.md` | 128 |
| K-22 | `K-22_doctrine_enforcement_coverage_ledger.md` | 124 |
| K-23 | `K-23_multi_repo_workspace_memory.md` | 145 |
| K-24 | `K-24_credit_billing_monetization.md` | 147 |
| K-25 | `K-25_continuous_absorption_watchdog.md` | 163 |

### B. K index, master, grounding, and fixture sources

| Role | Exact file under the same base | Lines | Why inspect |
|---|---|---:|---|
| Sequencing/doctrine/index | `K-0_INDEX.md` | 589 | K-10 doctrine, KD-1..KD-5, KC/KW/KS/KG/GD/W/R rules, Section-A pointers, waves, kill-switches, gating order |
| Non-numbered master brief | `K-MASTER-BRIEF-2026-06-12.md` | 483 | Original wave framing, research/market thesis, likely K-T6 and source vocabulary resolution |
| Grounding map | `_grounding/K-GROUNDING-MAP.md` | 147 | W-7 corrections, live anchors, KW/KS/KG source resolution and status corrections |
| Negative fixture | `_fixtures/claims_negative.md` | 14 | Exact claims-lint negative cases and expected failures |
| Read source | `K-SUMMARY_INDEX.md` | 600 | This discovery's direct source; do not reassign as an original entry |

### C. On-disk later/special candidates absent or incomplete in the summary

| Exact file under the same base | Lines | Summary relationship |
|---|---:|---|
| `K-18a_ps5_observe_arm_testrun.md` | 178 | Not mentioned; deep-inspection candidate |
| `K-26_codex_burn_proof_spine_run.md` | 157 | Preamble-only at lines 82-91; needs full extraction |
| `K-27_gpt56_sol_isolated_executor_lane.md` | 32 | Entirely unindexed; needs full extraction and summary-drift reconciliation |

There is no standalone `K-10_*.md` file in the 32-file K-plan tree at audit time.

### D. Directly named reasoning, grounding, map, and master sources outside K-plan

- `workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md` — K-15/K-16 grounding; J-83..J-86 technical companion.
- `docs/thefinalglance/J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md` — shared GRC code-grounding map.
- `workflow/reasoning/CONTEXT-MANAGEMENT-DEEP-DIVE-2026-06-25.md` — K-17 grounding and IMP-1..IMP-8.
- `workflow/reasoning/self-improving-workflow-canvas-verified-case-graph-2026-06-25.md` — K-18/K-19 VCG grounding.
- `workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md` — K-13 WI-1..WI-6 grounding.
- `workflow/reasoning/time-awareness-design-2026-06-25.md` — K-14 T/IMP/TZ grounding.
- `workflow/reasoning/LACUNA-FILL-PROGRAM-2026-06-26.md` — K-20/K-21/K-22 and out-of-K lacuna track synthesis.
- `workflow/reasoning/multi-project-memory-analysis-2026-06-26.md` — K-23 grounding.
- `workflow/reasoning/elai-credit-billing-monetization-2026-06-26.md` — K-24 grounding.
- `workflow/reasoning/elai-spec-eval-ambition-2026-07-05.md` — K-25 ADD-2 source.
- `workflow/reasoning/proof-spine-2026-07-05.md` — corpus-wide proof-spine ruling relevant to K-25/K-26 sequencing.
- `docs/thefinalglance/ELAI-FEATURE-MAP.md` — K-8 tags, K-12 status correction, K-23 group 1.15, K-24 group 1.16; cited sections 1.4/1.5/1.6/1.7/1.10/1.14.
- `docs/specs/guide/steps/phase-index-MASTER.txt` — cited as `MASTER 412` by K-14 for autonomous wall-clock liveness.
- `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md` — orientation map only; already read, not an original K detail source.

### E. Unresolved named/source families that originals or grounding must identify exactly

- The KAT kunchen video/source and context-compression steal-run document(s); only the demand-side AUTHOR claim is described in the summary.
- The “two reasoning docs” cited for K-18/K-19; only the VCG canvas document is named directly in this summary.
- `LACUNA-SWEEP` and deep-research reports 7-10 synthesized by the lacuna program; filenames are not provided here.
- KW-1 through KW-6 primary sources: USENIX hallucinated packages; arXiv:2507.02976; reward-hacking study; CodeAct; SpecBench; and pending DORA/METR/vendor material.
- KS-2 section 1.9 HERA and KS-3 reports 16/17/18 on competitor evidence, absorption, and distribution gaps.
- K-15's three self-correction papers and K-16's four-paper grounding; titles are not given here.
- K-19's three-paper collapse grounding, Misevolve 99.4-to-54.4 and F-10 +37.6 sources.
- K-21 “report 8” and K-22 “reports 8/10 ContextCov” sources.
- Polygraph, OpenRouter, LiteLLM, and the 2026 project-market-positioning/GTM memory sources; URLs/files are not named in the summary.

### F. Runtime, harness, planned-artifact, and configuration anchors named by the summary

These are path tokens/anchors, not all fully resolved paths. The relevant original must resolve each basename to a repository path and re-grep its live line number before implementation.

- **K-1/K-6/K-7/K-13/K-17/K-20:** `docs/strategy/`, `workflow/reasoning/`, `docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md`, `docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md`, `rra_cmd.rs`.
- **K-2/K-4/K-15/K-18:** `args_bench.rs:75`, `benchmarks/safedelta_runner.rs`, `benchmarks/reasoning_budget_runner.rs`, `failure_taxonomy.py`, plus existing convergence/context-fill runners.
- **K-2 routing anchors:** `cloud_mode.rs`, `escalation_policy.rs`.
- **K-3/K-11 GitHub anchors:** `pr_review_cmd.rs`, `crates/orchestrator/src/github_pr/{review.rs,body_composer.rs,comment_watcher.rs,adapter_trait.rs,evidence_render.rs}`, `github.svelte.ts`, `.claude` `visual-ui-loop`.
- **K-4/K-5/K-19 promotion anchors:** `wire-dark-allowlist.toml`, the frozen gate manifest, `ELAI_LOSSLESS_GATE_DISABLE`, `ELAI_PR_EVIDENCE_RENDER_DISABLE`.
- **K-12 context anchors:** `compact.rs:545`, `belief_state.rs`, `escalation_waterfall.rs:239`, `session_dag.rs:1539`, `pristine_path_filter.rs`, `handoff_faithfulness.rs`.
- **K-14 time anchors:** `.claude/rules/time-determinism.md`, `clock.rs`, cache breakpoints, background-task/notification primitives.
- **K-16/K-19/K-21 doctrine anchors:** `.claude/rules/reasoning-receipts.md`, `.claude/rules/self-improvement-fitness.md`, `.claude/rules/journal-completeness.md`, and conditional-routing triggers.
- **K-17 context anchors:** `context_budget.rs`, `context_policy.rs`, `ContextCalibrationEvent`, `ContextStatePanel`.
- **K-20 RRA anchors:** `rra.rs`, future `ReproPackEvent`, J-14 cassette/`rra export-fixture` paths.
- **K-22 doctrine-coverage anchors:** `CLAUDE.md`, `.claude/rules/*.md`, `.claude/tools/doctrine_coverage.py`, `.claude/rules/doctrine-enforcement.md`, `.claude/gate-manifest.txt`, `gate_freeze_check.py`.
- **K-23 workspace anchors:** `paths.rs:173`, `crates/orchestrator/src/workspace/`, CLI workspace verbs, `crates/orchestrator/src/blast_radius.rs`, `workspace.ssb.db`, `ELAI_WORKSPACE_ENABLE`, canvas WORKSPACE altitude.
- **K-24 billing anchors:** `crates/orchestrator/src/billing/`, `openrouter_backend.rs`, `seats.toml`, `ssb.rs:1508`, `action_type_lock.rs:775`, `ELAI_BILLING_ENABLE`, H-9 `CloudPanel`.
- **K-25 watchdog anchors:** `workflow/status/provider-surface.json`, `.claude/tools/absorption_watch.py`, `.claude/rules/absorption-watch.md`, `elai_status_query.py`, `.claude/rules/proof-spine.md`, `proof_spine_status.py`.

## Summary-layer close condition

The direct source ledger is complete: every normal K heading, every K-26 preamble behavior, all cross-cutting invariants, explicit out-of-K companions, all 32 K-plan files, named external reasoning/map sources, and unresolved citation families are represented. The next pass should inspect originals in the manifest rather than widening this summary-derived inventory by inference.
