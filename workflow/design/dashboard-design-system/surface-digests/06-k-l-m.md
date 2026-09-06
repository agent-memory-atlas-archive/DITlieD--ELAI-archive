# Surface digest: K + L + M (lines 58853-59951)

Source range: User-facing surfaces (K) 58853–59433, (L) 59434–59776, (M) 59777–59951. Statuses are plan-recorded, not runtime proof.

---

## K — Trust wedge, distribution, buyer evidence, workspace, billing

### Surface inventory

- **K-0 doctrine index** [type: file|CLI] [atoms: K0-E007..E067] [status: planned doctrine]
  Tracks K-T2 wedge (A-4 portable `.rra`/`elai wrap`, K-3 PR/CI evidence, A-5 review study), K-T4 buyer evidence (K-6 compliance export, K-7 AppSec brief), KD-1..KD-5. CLI kill-gate: `elai bench claim-scope [--suite][--seed][--json]` first wave command.
  Kill-switches: A-1/A-3 null safety delta → stop trust headline; K-4 tax untolerable → collapse; GitHub native crypto evidence → A-4 pivots compat, K-3 survives; A-5 no review win → cockpit = compliance artifact; no 2x3090 → K-9 still runs.

- **KGM ground map** [type: CLI|module] [atoms: KGM-E004..E028] [mixed]
  KG-1 claim-scoper SHIPPED (`ClaimScopeMeasured`, `elai bench claim-scope`). KG-2 `github_pr` FULL WIRED + Tauri `commands_c40.rs` + `github.svelte.ts`. KG-3 review surface present. KG-7 RRA CLI + `elai verify-attestation`. Bench dispatcher live `args_bench.rs`. A-4 PLANNED; K-3 does not wait on A-4.

- **K-1 claims dossier + lint** [type: file|CLI] [atoms: K1-E002..E030] [planned BUILD]
  Artifacts: `ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md`, `ELAI-CLAIMS-POLICY.md`, `.claude/tools/claims_policy_lint.py`. Trigger: `python .claude/tools/claims_policy_lint.py --files docs/strategy/positioning/*.md`. frontend_pair N/A (W-6). Fail-closed on untagged superlatives/numbers.

- **K-2 routing-economics bench** [type: CLI|panel?] [atoms: K2-E003..A003] [planned]
  `elai bench routing-economics [--seed] [--json] [--force-cloud-everything]`. Two arms: cloud-only vs H-7 hybrid; y-axis verified-OUTCOME rate not pass@k; ELAI_SEED. Live cost may be 'cost unavailable' (K-26). FORK-3: HF-vs-hybrid tile vs CLI-only decide-at-edit; RunnerPresent → W-6 CLI default.

- **K-3 PR/CI evidence emitter** [type: CLI|check-run|PR comment|panel] [atoms: K3-E002..A003] [planned BUILD; rides WIRED github_pr]
  Renderer over github_pr: one check-run (gate verdicts + attestation digest + offline-verify line) + per-hunk PR comments via H-52 anchors only (hallucinated lines impossible). No-evidence hunk → UNVOUCHED, never silent clean. `--post-unvouched-as-clean` MUST fail.
  Entry: `elai pr-review <pr>` / `elai pr-review <fixture-pr> --emit-evidence`. Dashboard deep view via `github.svelte.ts` status (W-4 decide fields); PR = buyer view (KD-2). Offline line uses `elai verify-attestation`. Prefer-reuse SSB or +PrEvidenceEmitted (CheckRunPosted/HunkVouched/HunkUnvouched).

- **K-4 gate-tax bench** [type: CLI] [atoms: K4-E003..C001] [planned]
  `elai bench gate-tax [--seed] [--json]`. Composed friction report vs LOCKED ceiling; breach = non-zero CLI exit (release BLOCK); top-3 friction contributors. GateTaxMeasured payload (coding vs audit tax split). Load-bearing surfaces = CLI JSON + row; optional friction tile not mandated (summary still offers edit-time tile). Superseded summary claims ON/OFF ablation + wire-dark inventory VETOED by refined source.

- **K-5 adaptive tier policy** [type: panel?] [atoms: K5-E003..A003] [planned]
  Always-on cheap legs E-30/F-1/F-9/G-1. frontend_pair: tier-decision rows on audit-timeline (W-4) OR W-6 absence.

- **K-6 compliance export** [type: CLI|file|tile?] [atoms: K6-E002..A003] [planned]
  `elai export compliance [--out <dir|archive>]` → redacted RRA + attestation + audit extract + retention + procurement mapping; offline verify; no raw secrets. Mapping doc `ELAI-COMPLIANCE-MAPPING.md`. frontend_pair N/A W-6 OR optional compliance-export tile (edit-time). Bundle format archive vs directory open.

- **K-7 AppSec/insurance brief** [type: file] [atoms: K7-E002..A001] [planned]
  Pure prose `ELAI-APPSEC-INSURANCE-BRIEF.md` problem→control→evidence. Must pass claims_policy_lint. frontend_pair N/A (W-6).

- **K-8 provider-absorption pregate** [type: file|rule] [atoms: K8-E004..I001] [planned]
  Rule `provider-absorption-pregate.md`; efficiency-class task re-WebSearch before EXECUTING; kill/de-scope proposal to user. No dashboard. Audit flags skip (process, not hard refuse).

- **K-9 model-refresh cadence** [type: CLI|tile?] [atoms: K9-E001..A001] [planned KW-2]
  `elai bench model-refresh [--lane]`. Decision row ADOPT/KEEP/RE-EVALUATE; regress auto-KEPT-incumbent; swap needs user ratification. No silent routing/allowlist mutation. Optional refresh-decision tile (W-4) or W-6 CLI-only.

- **K-11 human Evidence section** [type: PR body|cockpit|CLI flag] [atoms: K11-E002..A002] [planned; extends K-3]
  Markdown `## Evidence`: RRA summary + per-test links + collapsible logs + screenshot refs (visual-ui-loop). Same EvidenceArtifactSet in G-5 reviewer cockpit (Tauri read, cross_lang_diff). `elai pr-review --emit-evidence` default-ON behind kill-switch `ELAI_PR_EVIDENCE_RENDER_DISABLE`. Risk-tiered by G-5 DSR; no-artifact = UNVOUCHED. PR = buyer; cockpit = deep (KD-2). Refs/links not blobs (GitHub size limits). Wedge: poll-CI + surface, human merges, no autonomous force-push.

- **K-12 losslessness gate** [type: runtime|panel reuse] [atoms: K12-E010..I001] [planned]
  Default-ON behind `ELAI_LOSSLESS_GATE_DISABLE`; observe-then-enforce GD-4 (≥3 runs). frontend_pair N/A — verdicts flow to existing context/trust panels via SSB.

- **K-13 OWASP mapping** [type: file|CLI lint] [atoms: K13-E002..E022] [planned]
  `ELAI-OWASP-CONTROL-MAPPING.md` LLM01-10 + ASI01-10 cells. Feeds K-7. frontend_pair N/A. Lint fails unmeasured 'covered' without [DESIGN].

- **K-14 time-determinism doctrine** [type: file] [atoms: K14-E004..E017] [planned]
  Rule `time-determinism.md`. Volatile time after last cache breakpoint. Differentiator via dossier/claims/K-7. frontend_pair N/A.

- **K-15 reasoning-budget bench** [type: CLI|panel] [atoms: K15-E002..A002] [planned; BENCH-PENDING J track]
  `elai bench reasoning-budget --suite default --seed N --json`. Curve receipts/verified-OUTCOME under SAFEDELTA GRC ON vs flat OFF. frontend_pair: curve+knee on I-27 ConvergencePanel + H-9 CloudPanel (W-4) or N/A ride.

- **K-16 reasoning-receipts doctrine** [type: file|lint] [atoms: K16-E006..E026] [planned]
  Doctrine + claims recogniser only. frontend_pair N/A.

- **K-17 context-regime claims** [type: file|panel?] [atoms: K17-E002..E024] [planned; policy WIRED]
  Context_policy Local 0.75 / Cloud 0.85 WIRED (`ELAI_CONTEXT_POLICY_DISABLE`); folklore fractions to be replaced by J-34 knee. frontend_pair: ContextStatePanel (DARK E-23) + H-9 or dossier/brief N/A.

- **K-18 case-reuse bench** [type: CLI|panel] [atoms: K18-E002..A002] [planned]
  `elai bench case-reuse --suite default --seed N --json`. Recurrence-hit-rate arm ships now; reuse-into-generation default-OFF allowlist. frontend_pair: hit-rate + $/iter on ConvergencePanel + CloudPanel; optional VCG case-altitude (W-4).

- **K-18a case-reuse fixture arm** [type: CLI] [atoms: K18a-E003..I001] [planned runnable subset]
  `elai bench case-reuse --suite fixture --seed 1234 --json`. Arm-1 retrieval only; proven 4-file CLI template for K-4.

- **K-19 self-improvement fitness doctrine** [type: file|lint] [atoms: K19-E005..E019] [planned]
  Observe-first reuse; NEVER inject into auditor (H-31). Claims lint fails untagged self-improving. frontend_pair N/A.

- **K-20 re-executable RRA packaging** [type: PR line|docs] [atoms: K20-E003..A002] [planned package; J-14-extension pending]
  Differentiator on K-3/K-11 PR (“verify offline / re-execute with `elai ...`”) + cockpit; else dossier/brief. CLI cited not authored: `elai run --emit-rra`, `rra show`, `replay --verify`, planned `rra export-fixture`, A-4 `elai wrap`. Exact PR wording illustrative.

- **K-21 journal-completeness doctrine** [type: file|lint] [atoms: K21-E006..I001] [planned]
  Rule journal-completeness.md; Linux-only observer honesty; Windows mediation-only. frontend_pair N/A — E1 reconcile + C8 canvas out-of-K.

- **K-22 doctrine coverage tool** [type: CLI|file] [atoms: K22-E003..I001] [planned]
  `.claude/tools/doctrine_coverage.py --json` + ledger md + doctrine-enforcement.md. frontend_pair N/A; coverage tile later optional (W-6).

- **K-23 multi-repo workspace** [type: CLI|canvas|file] [atoms: K23-E006..E050] [planned NET-NEW; default-OFF]
  Object: `workspace.toml` (member project_roots + relationship hints). Global journal `workspace.ssb.db`. Flag `ELAI_WORKSPACE_ENABLE` wire-dark allowlist (type=experiment, promotion_plan_id=K-23-promotion).
  CLI: `elai workspace create|add|map|search <id> "<q>"|impact <id> <repo>:<symbol>|orchestrate`. Map renders Sound vs Heuristic edges (heuristic = warning badge, never asserted as call edge). Search fans per-member pipeline, RRF k=60, per-repo provenance, no new index. Impact extends B-29/J-16 over Sound only. Orchestrate: workspace FSM above per-root; controller never edits member code; 4 tripwires; polls J-2/H-36 only.
  Canvas WORKSPACE altitude beside J-31/J-60/J-98: nodes=repos, edges colored Sound/Heuristic, in-flight delegation lights node, workspace-session+PRs temporal layer; re-derived byte-identical from workspace.ssb.db. Exact drawer/window decide-at-edit. Seat-scoped (J-59); cross-seat OUT. Cap small member set (2–4). Until J-18: Heuristic-only advisory, impact observe-only.

- **K-24 credit billing** [type: CLI|panel|webhook] [atoms: K24-E001..A002] [planned NET-NEW; default-OFF]
  Model: subscription → monthly CREDITS → spend per metered usage; ELAI pays upstream; owns limits/rollover/routing (Nous/OpenRouter shape). Flag `ELAI_BILLING_ENABLE` allowlist K-24-promotion.
  CLI: `elai billing balance` folds ledger. L3 OpenRouter managed backend B-37 hand-rolled; Settled via J-28; cost ~1.055× face. L4 J-49 admission: balance≥hold AND fair-share AND tier rate/concurrency → fail-closed queue or refuse with clear out-of-credits surface. L5 Payment Stripe/Paddle OUTSIDE ELAI (PCI zero); webhooks Granted/Expired/AdjustmentManual + seat-suspension on chargeback. Rollover/tier limits = seats.toml DATA.
  UI: credit balance + reserve/settle + tier rate limits on H-9 CloudPanel OR dedicated billing tile (decide-at-edit W-4); Svelte 5 runes + tauri command + ts-rs; cross_lang_diff. Honest copy: monthly credits metered at provider cost; routes to cheapest verifiable model; NEVER “unlimited Sonnet/Opus/GPT.” L6 SAFEDELTA $/verified-task under ELAI_SEED. Events: CreditLedgerEvent hash-chained recomputable.

- **K-25 absorption watch** [type: CLI|file] [atoms: K25-E003..E024] [planned]
  `absorption_watch.py --json --max-age-days 90` over `provider-surface.json`. Exit 0/1/2/3. frontend_pair N/A. Exit-1 → kill/de-scope proposals.

- **K-26 gpt-5.5 dogfood campaign** [type: CLI|panel|tile] [atoms: K26-E003..E030] [planned campaign; deadline 2026-07-15 credits]
  Surfaces in seed queue: Dashboard RecurrenceHitRateMeasured panel; GateTaxMeasured tile; `elai usage`; `elai bench gate-tax`; `elai bench case-reuse --suite journal`; `elai bench routing-economics`; `elai run`. REDTEAM: `elai run` has NO model-selection seam (FORK-5).

- **K-27 gpt-5.6-sol containment** [type: process|CLI config] [atoms: K27-E001..E018] [campaign extension]
  Codex output TAINTED; never controller; AI-5 patch screen. In-product routing does NOT accept gpt-5.6-* until one-line widen. Not a product UI surface beyond containment chain.

### User flows

- FLOW KW-1 invoke [K0-E066,K0-E053] [planned]: `elai bench claim-scope --suite default --seed 1234 --json` → NULL/NEGATIVE = kill-switch 1 STOP → else continue KW-2; parallel K-8.
- FLOW Section-A order [K0-E023,KMB-E010] [planned]: A-1 invoke → go → A-2 → A-3; A-4 parallel after G-9 live; A-5 after K-3 or cockpit-only if K-3 slips.
- FLOW K-3 emit evidence [K3-E009,K3-E013,K3-E011] [planned]: `elai pr-review <pr> --emit-evidence` → github_pr adapter → check-run + H-52-anchored hunk comments → UNVOUCHED if no evidence → reject `--post-unvouched-as-clean` → optional github.svelte.ts status.
- FLOW K-11 evidence section [K11-E008,K11-E009,K11-E024] [planned]: same emit path → EvidenceArtifactSet assembler → `## Evidence` in PR + G-5 cockpit Tauri read → kill-switch ELAI_PR_EVIDENCE_RENDER_DISABLE disables render.
- FLOW K-4 release tax [K4-E038,K4-E039] [planned]: `elai bench gate-tax` → compute coding/audit tax → if coding tax OR over_block_rate > ceiling → non-zero exit BLOCK release; else JSON + GateTaxMeasured.
- FLOW K-6 buyer export [K6-E002,K6-E010] [planned]: `elai export compliance --out …` → redacted bundle → offline `elai verify-attestation` + conformance on clean machine.
- FLOW K-9 model swap [K9-E003,K9-E012] [planned]: `elai bench model-refresh` → fixture candidates → ADOPT/KEEP/RE-EVALUATE row → regress = auto KEEP → ADOPT needs user ratification → no silent routing write.
- FLOW K-23 workspace [K23-E027,K23-E011..E016,K23-E025] [planned wire-dark]: create/add members → map (Sound/Heuristic) → search federated RRF → impact Sound-only → orchestrate multi-root PRs → canvas WORKSPACE altitude re-derive; store under member root FAILS; cloud store FAILS.
- FLOW K-24 credits [K24-E007,K24-E011,K24-E012] [planned wire-dark]: payment webhook Granted → ledger fold → `elai billing balance` / CloudPanel tile → J-49 admission hold → OpenRouter settle Settled → out-of-credits refuse surface; chargeback → AdjustmentManual + suspend.
- FLOW K-2 routing bench [K2-E018,K2-E023] [planned]: force-cloud arm %offloaded==0 savings~0 → hybrid arm >0 on same fixture under ELAI_SEED.

### Onboarding-relevant surfaces
- K-1 dossier + claims policy (external-facing claim discipline before marketing ships).
- K-6 compliance bundle + mapping (procurement onboarding artifact).
- K-7 AppSec brief (buyer questionnaire shape).
- K-23 workspace create/add (multi-repo setup; default-OFF experiment).
- K-24 billing enable + balance (commerce onboarding; default-OFF).
- B-43 installer activation offer is M-track (composes with K-24 seats); not fully specified in K atoms beyond K-24 dependency list.

### Notifications / background surfaces
- K-3/K-11: GitHub check-runs + PR comments (external buyer notifications).
- K-8: audit flag on skipped provider re-check (operator, not user toast).
- K-9: decision row + dated refresh report (no silent swap).
- K-12/K-21: observe-mode journal rows before enforce (no dedicated toast named).
- K-24: webhook-driven Granted/Expired; out-of-credits refuse; seat-suspension flag.
- K-23: canvas in-flight delegation lights on member nodes.
- K-14/H-36: AgentNotificationEmitted cited as substrate (not new UI).

### Decide-at-edit / open UI decisions
- K0-A003: K-9 and K-6 frontend_pair deferred W-4/W-6.
- K2-A003 / FORK-3: HF-vs-hybrid panel tile vs CLI-only.
- K3-A003: exact check-run status fields on github.svelte.ts.
- K4: optional friction tile summary vs refined CLI-only load-bearing (I001).
- K5-A003: audit-timeline tier-decision presentation.
- K6-A001/A003: archive vs directory; compliance tile v1.
- K9-A001: refresh-decision tile vs CLI-only.
- K11-A002: collapsible-log markdown + GitHub size cutoffs.
- K12-A003: observe vs hard BLOCK schedule beyond ≥3 runs.
- K15-A002: dedicated tile vs ride ConvergencePanel/CloudPanel.
- K17: ContextStatePanel once mounted vs N/A.
- K18-A002: dedicated case-reuse tile vs existing panels.
- K20-A002: exact PR re-execute wording not frozen.
- K23-A002: exact drawer/window for WORKSPACE altitude.
- K24-A002: billing tile vs ride H-9 CloudPanel.
- KMB-E030: K-4 ceiling VALUE user call; K-3 GitHub-only vs GitLab; K-6 archive vs dir; canvas demotion confirmation.
- Superseded (summary vs refined): K4-C001 summary ON/OFF ablation + wire-dark 105 inventory + args_bench:75 line claim VETOED by refined source.

---

## L — Causal grounding, quality, familiarity surfaces, context, eval, explore

### Surface inventory

- **L-0 plan index** [type: doctrine] [atoms: L0-E001..E051, SUMMARY/UI-001..008] [planned tracks]
  L-T1 CGG L-1..L-10; L-T2 familiarity activation L-11..L-14 (dark doors, not invention); L-15 repair; L-T3 context L-16..L-19; L-T4 competitor L-20..L-21; L-T5 explore L-22..L-24. FS-2 every user surface same trust gates; FS-4 agentskills/CC/MCP compat; EX-3 survivor-map not single answer; EX-6 status computed never curated.
  Named CLI cluster (SUMMARY/UI-001): `elai bench hallucination-delta`; `/reason --profile research`; `elai skill add/improve`; `elai rule add/list/test/promote/retire`; `elai build "<idea>"`, `elai artifact run`, `elai rra verify`, J-2 MCP build; `elai eval mine-commits`, `elai bench competitor`; `/reason --profile explore`.
  Flags: `ELAI_HALLUCINATION_GATE_DISABLE` (observe/journal, no block); `ELAI_CONTEXT_PROJECTION_DISABLE` (legacy mutate rollback); `ELAI_VERIFICATION_INTENSITY_DISABLE` (byte-identical restore); `ELAI_SEED`.

- **L-CGG substrate** [type: runtime|drawer] [atoms: LCGG-E007..E041] [mixed DARK/WIRED]
  G-7 conformal DARK (`ELAI_CONFORMAL_REFUSAL_ENABLE`); D-16 UniCR DARK; D-13 adversary DARK (`ELAI_ADVERSARY_ENABLE`); I-4 search ladder DARK (`ELAI_SEARCH_LADDER_ENABLE`); B-39 Chat Reflect WIRED — `/reason` + ReasoningDrawer; D-11 checker DARK (`ELAI_CHECKER_ENABLE`).

- **L-OQF familiarity substrate** [type: CLI|drawer] [atoms: LOQF-E014..E044] [mixed]
  E-43 skill system DARK (`ELAI_SKILL_SYSTEM_ENABLE`); J-63 `elai artifact run` PLANNED; I-40 wrap/RRA WIRED (`elai wrap`, `elai rra verify`); J-2 MCP PLANNED; J-102 admission PLANNED. Named L-12/13/14 surfaces: skill add/drawer, rule add/hook-register, build, artifact run, wrap, rra verify, dashboard one-field.

- **L-1 claim extraction** [type: tile?] [atoms: L1-E019,A003] [planned]
  frontend_pair: claims tile on trust panel OR N/A (feeds L-2 risk tile). Decide-at-edit W-4.

- **L-2 ClaimRisk fuse KEYSTONE** [type: tile?] [atoms: L2-E002..A004] [planned]
  ClaimRisk tiers Low/Medium/High/Critical + PredictorSignal. frontend_pair: ClaimRisk tile on trust/context OR N/A to existing trust panels. Event ClaimRiskScored or generalize G-7 (decide-at-edit).

- **L-3 round-trip residual** [type: none] [atoms: L3-E011] [planned]
  frontend_pair N/A justified; feeds L-2 risk tile.

- **L-4 semantic entropy** [type: none] [atoms: L4-E022] [planned net-new signal]
  frontend_pair N/A justified; feeds L-2. Done-gate ELAI gold set never paper AUROC.

- **L-5 do-test-on-claims** [type: none] [atoms: L5-E015,I001] [planned KEYSTONE]
  frontend_pair N/A; causal_supported lives on L-2 ClaimRisk.

- **L-6 curiosity router** [type: panel?] [atoms: L6-E002..A001] [planned]
  High/Critical ClaimRisk auto-fires targeted evidence buy (not blanket). frontend_pair: retrieval/escalation surface OR existing panels (W-4).

- **L-7 hallucination gate** [type: panel] [atoms: L7-E001..A002] [planned WAVE 3 observe-first]
  Observe-only first GD-4; block behind `ELAI_HALLUCINATION_GATE_DISABLE` (kill-switch; when set, observe-and-record only). frontend_pair: refusal/mode panels (D-27) OR N/A existing. Critical → AutonomousGate EvidenceCheck fail path.

- **L-8 claim quarantine hygiene** [type: panel reuse] [atoms: L8-E016] [planned]
  frontend_pair N/A — quarantine via existing staleness/context panels. Kill-switch reuses F-5 (not new flag).

- **L-9 research profile** [type: CLI|drawer badge] [atoms: L9-E001..A001] [planned WAVE 4 EXTENSION]
  `/reason --profile research` (not new engine): Cosmos papers + Mode A checkers + CGG over reasoners; composes B-39 + ReasoningDrawer. frontend_pair: research-profile badge + per-claim checker verdict (W-4). Trigger: wrong arithmetic caught by sympy/z3 not model; unverifiable abstained. Reuses `ELAI_CHECKER_ENABLE`.

- **L-10 hallucination-delta bench** [type: CLI|tile?] [atoms: L10-E002..A003] [planned]
  `elai bench hallucination-delta [--scope fixture|live] [--seed] [--json]`. ELAI_SEED journal-recomputable; receipts y-axis; no LLM-judge. frontend_pair: delta tile on audit/cost OR existing bench panels. L-15 adds `--repair` arm.

- **L-11 SOLID quality checker** [type: CLI|tile|gate] [atoms: L11-E004..A002] [planned]
  `.claude/tools/solid_proxy_check.py --files/--base/--json`; exit 1 BLOCK / 2 config error. Composes I-38 `--quality`. frontend_pair: quality-verdict tile on vibesec/GatesPanel OR N/A audit surface.

- **L-12 skill front door ACTIVATION** [type: CLI|panel] [atoms: L12-E002..I001] [planned; E-43 DARK flip]
  `elai skill add <dir>`: agentskills_validator → skillspector → J-102 blake3 fail-closed → VibeGate F-1 → register + skill-imports.log. `elai skill improve <name>`: I-12 curator; surface proposed edit + eval delta for user confirm (FS-1 DATA). Kill-switch only `ELAI_SKILL_SYSTEM_ENABLE`. frontend_pair: SkillsPanel.svelte add/improve/trust-tier + eval delta OR N/A.

- **L-13 user rules authoring** [type: CLI|drawer] [atoms: L13-E003..A002] [planned]
  CLI: `elai rule add|list|test|promote|retire`. Drawers RulesDrawer/ToolsHooksDrawer: add-rule form + SHADOW fire-rate + promote/retire (drawers WIRED; authoring extension). Flow: add → J-102 → SHADOW row; hook SHADOW observe / ACTIVE block. Immune-weaken REJECTED. `elai rule test` pre-promotion dry-run. Exact form fields decide-at-edit.

- **L-14 one-shot verified builder KEYSTONE** [type: CLI|page|MCP] [atoms: L14-E002..A003] [planned ACTIVATION+COMPOSE]
  CLI front door: `elai build "<idea>"` for lazy/one-shot user. Dashboard: one-field “build from an idea” on ProjectSelector/Intro — tree forming + receipt + quality verdict; reuse pipeline/plan windows.
  Path: CLI/dashboard → pipeline factory → E-12 → L-11 quality → I-40 wrap RRA always (FS-3 non-negotiable); build report includes receipt path + `elai rra verify` instructions. `elai artifact run` via J-63 loopback Tier-C (if unshipped: emit app+receipt + residue soft-dep). Expose build as J-2 MCP verb for CC/Codex (FS-4).
  Receipt UX: PRESENT not noisy — one-line `verified: <receipt> (elai rra verify)` not audit wall; UX decide-at-edit. Kill-switch on build verb; reuse `ELAI_WRAP_RRA_DISABLE` for receipt path [unclear dual wording L14-A003]. Contract: fixture app+RRA verifies; L-11 fail reports findings not DONE; no value-claim without measured number; J-2 MCP reachable. W-1: hard multi-route idea e2e smoke.
  Status substrate: pipeline L1/L2 WIRED; E-12 WIRED; J-63 PLANNED; I-40 WIRED; J-2 PLANNED; L-11 PLANNED.

- **L-15 RepairController** [type: runtime FSM|bench arm] [atoms: L15-E002..E021] [planned]
  S0 EXTRACT → S1 SCORE ClaimRisk → S2 DECIDE High/Critical ENTER REPAIR → S3 DIAGNOSE/REMEDY → S4 RE-VERIFY → S5 GATE J-83 → S6 COMMIT-AND-FREEZE. Kill-switch `ELAI_CGG_REPAIR_ENABLE` default-OFF. Trigger: L-10 `--repair` arm; no dedicated UI named. External-receipt-only commit (self-say-so = P0).

- **L-16 context projection** [type: gauge badge] [atoms: L16-E012..A002] [planned]
  Default-ON behind `ELAI_CONTEXT_PROJECTION_DISABLE` (one-release mutate rollback). Context gauge (H-50 reuse): badge `view: hot-reuse | recomputed` OR N/A.

- **L-17 epoch cause** [type: gauge] [atoms: L17-E013] [planned]
  Context gauge shows last epoch cause OR N/A.

- **L-18 projection integrity** [type: none new] [atoms: L18-I001] [planned]
  No new operator UI; effects via undo/branch + auditor gate failures.

- **L-19 recall TTL** [type: gauge] [atoms: L19-E011,A002] [planned]
  Gauge string `recalled N (TTL k)` OR N/A decide-at-edit.

- **L-20 competitor eval supply** [type: CLI|tile?] [atoms: L20-E002..A001] [planned]
  LEG A: `elai eval mine-commits` → J-56 rows (prompt, base_sha, reference_diff blake3, category, provenance) via `elai eval ingest`. LEG B: `elai bench competitor` arms claude_code/codex_cli/opencode/gemini_cli via cli_common spawn, Tier-C, same envelope; ELAI keeps gates, competitors vendor-permissive inside disposable workspace only (honest framing). frontend_pair: paired-arm tile on bench/audit OR N/A (E-49 panel PARTIAL).

- **L-21 verification intensity** [type: badge] [atoms: L21-E008..A002] [planned observe-first]
  Journal intensity first; behavioral flip behind `ELAI_VERIFICATION_INTENSITY_DISABLE` (default-ON once promoted). frontend_pair: intensity badge on RuntimePanel family OR N/A. Unprofiled model → HIGHEST intensity; DISABLE=1 byte-identical to fixed policy.

- **L-22 falsifier surface** [type: chip] [atoms: L22-E022..A002] [planned]
  frontend_pair: falsifier chip on L-2 trust/claim tile (refuter + pass/forbids-nothing) OR N/A rides ClaimRisk. Causal/Behavior forbidding nothing → High ClaimRisk floor.

- **L-23 explore profile KEYSTONE** [type: CLI|canvas] [atoms: L23-E001..A003] [planned WAVE 2 NEW ~90% composition]
  `/reason --profile explore` ≡ `elai explore` ≡ `elai reason --profile explore "<hard open question>"`. Deliverable: survivor map {surviving explanations, refuted + falsifier, deciding-evidence gaps}; SSB ExploreRoundCompleted (SurvivorRetained/RefutedByFalsifier/OrbitDetected/ProblemTurnedOver). frontend_pair: survivor map on reasoning/canvas via K-11/N-7 (rivals + refutation + gaps); Tauri read conditional. Does NOT re-implement /reason; REJECTED Dung adjudicator. Quality ELAI-measured (L-10 explore arm) never DeepReason figures.

- **L-24 conjecture lifecycle store** [type: CLI|canvas|debug flag] [atoms: L24-E010..I001] [planned]
  conjecture_store + ExploreLaneToken retrieval; status fold byte-recomputable. Production: `elai reason --profile explore` → L-23 pack → store. Suspended/quarantined only via `--include-quarantined` debug read, never default pack. frontend_pair: conjecture map on explore/canvas (status color + missing-tests tooltip + quarantine count) rides L-23/K-11/N-7. Bench: hallucination-delta explore arm novelty column. K-11 renders “what was tried + what would change the answer” for non-author.

### User flows

- FLOW L-2 score path [L2-E025,L6-E002,L7] [planned]: L-1 ClaimSet → ClaimRisk fuse → High/Critical → L-6 curiosity auto-effort OR L-7 ForceEffort/gate → Critical fails EvidenceCheck; observe-only until promote (ELAI_HALLUCINATION_GATE_DISABLE).
- FLOW L-9 research [L9-E013,L9-E015] [planned]: `/reason --profile research` → B-39 N=4 + Mode A checkers + CGG → wrong step caught by checker → badge/verdicts on ReasoningDrawer.
- FLOW L-12 skill add [L12-E003] [planned]: `elai skill add <dir>` → validator → skillspector → J-102 → VibeGate → register OR fail-closed quarantine; improve → curator proposal → user confirm.
- FLOW L-13 rule lifecycle [L13-E010,L13-E012] [planned]: `elai rule add` → J-102 → SHADOW → live hook would-WARN without block → `elai rule test` FP measure → promote when FP bound → retire; immune-weaken REJECTED.
- FLOW L-14 one-shot build [L14-E002..E023] [planned] FULL:
  1. Entry: CLI `elai build "<idea>"` OR ProjectSelector/Intro one-field OR J-2 MCP build verb.
  2. Pipeline factory + E-12 templates/checklist runs.
  3. L-11 quality check; on fail → report findings, not DONE.
  4. Always attach RRA via elai wrap / RunArtifactEmitted (ELAI_WRAP_RRA_DISABLE kills receipt path).
  5. Output: app dir + one-line verified receipt + rra verify instructions; optional tree forming UI.
  6. Optional `elai artifact run` (J-63 Tier-C) or residue if J-63 missing.
  7. Error/empty: no value-claim without measured number; receipt must recompute/verify.
- FLOW L-15 repair [L15-E003,L15-E011] [planned wire-dark]: L-10 `--repair` → S0..S6 → commit only with external receipt; else abstain terminations.
- FLOW L-20 competitor [L20-E017] [planned]: `elai eval mine-commits --repo <path> --max 20` → `elai bench competitor --arms elai,claude_code --rows <subset> --seeds 5` in disposable workspaces.
- FLOW L-23/L-24 explore [L23-E003,L24-E016] [planned]: `elai reason --profile explore "<q>"` → round loop + falsifiers → survivor/conjecture map → quarantine debug via `--include-quarantined`; fixture byte-identical under ELAI_SEED + RRA.

### Onboarding-relevant surfaces
- L-12 skill add (import existing CC/agentskills skills).
- L-13 rule add (user rule/hook registration SHADOW first).
- L-14 `elai build` + ProjectSelector/Intro (primary non-expert front door).
- L-9/L-23 reason profiles (power-user depth after base agent works).
- FS-4 format compatibility preserves existing CC/Codex workflows (MCP consume+serve).

### Notifications / background surfaces
- L-6 auto-fires curiosity effort (background routing, not toast named).
- L-7 observe-mode would-be GateAction journaled before block.
- L-13 SHADOW would-WARN without block until promote.
- L-15 repair FSM every transition SSB row.
- L-21 intensity journaling before behavior flip.
- No dedicated push/toast atoms beyond gate/refusal panels.

### Decide-at-edit / open UI decisions
- Nearly every frontend_pair is “decide at edit OR justified N/A” (L-002/AC-010) — candidates not committed UI.
- L1-A003 claims tile; L2-A003 ClaimRisk tile + L2-A004 event shape; L6-A001 retrieval UI; L7-A002 refusal/mode panels; L9-A001 badge visual; L10-A003 delta tile; L11-A002 quality tile; L12 SkillsPanel pair; L13-A002 drawer field set; L14-A002 ProjectSelector/Intro layout + tree/receipt/verdict composition; L14-A003 build kill-switch env name vs ELAI_WRAP_RRA_DISABLE dual wording; L16-A002 gauge badge; L19-A002 recalled N TTL k; L20-A001 paired-arm tile; L21-A002 RuntimePanel intensity badge; L22-A002 falsifier chip; L23-A003 Tauri survivor map conditional.
- SUMMARY/UI-002 lists full edit-time candidate set: claims tile, ClaimRisk tile, retrieval/escalation, refusal/mode, research badge, hallucination-delta tile, GatesPanel/vibesec quality, SkillsPanel, RulesDrawer/ToolsHooksDrawer, ProjectSelector/Intro builder, context epoch/recall gauge, paired-arm tile, RuntimePanel intensity, falsifier chip, survivor-map status.

---

## M — Commercial onboarding spine (license, org policy, fleet, update, packaging, EEF)

### Surface inventory

- **M-0 commercial index** [type: doctrine] [atoms: M0-E042,E048] [planned waves]
  WAVE M1: M-1 + M-5a elai-verify OSS + M-5d tiers.toml. M-3 fleet report WAVE M3 depends M-1, MA-3, MA-6.

- **M-1 seat entitlement KEYSTONE** [type: CLI|banner|installer|Settings?] [atoms: M1-E010..C001] [planned]
  CLI: `elai license activate <file|key>`, `elai license status`, `elai license deactivate`. Tiers DATA via M-5d tiers.toml. Free tier = everything through L not explicitly Team in tiers.toml. Expiry → 14-day grace (ExpiredGrace rows + status banner) → DegradedToFree. Banner user-visible; placement/copy/dismiss unspecified (M1-A002).
  Composes G-8 attestation, K-24 credit ledger + seats.toml, ClockProvider, K-1 claims lint, B-43 installer (activation offer one line at install-end). Entry: license CLI real; tier() on K-24 grant path. Trigger: `elai license activate fixtures/team-entitlement.signed && elai license status`.
  Summary SettingsWindow license-status line is W-4 deferred; numbered M-1 does not name SettingsWindow (M1-C001). Superseded as target UI: SettingsWindow license-status line retained under SUP-001 only (`SUMMARY/SURF-002` / M1-E043).

- **M-2 org policy pack** [type: CLI|boot warning] [atoms: M2-E015..A003] [planned]
  CLI admin: `elai org policy init|sign|verify|show`. Drift DETECTED not auto-corrected: boot compares applied-pack hash vs org-pinned → OrgPolicyEvent(Drift) + boot warning + M-3 rollup. Boot warning presentation CLI-only vs UI badge unspecified (M2-A003).

- **M-3 fleet evidence rollup** [type: CLI|file render] [atoms: M3-E019..A002] [planned WAVE M3]
  Seat: `elai export compliance --org`. Admin: `elai org report <dir-of-seat-exports>` → one org bundle; `elai org report --verify` re-checks every chain. Dual render: human markdown sheet + machine msgpack/toml (one aggregation path). Behavior without `--verify` unspecified (M3-A002). Trigger: 3 fixture exports (clean / Drift / broken chain) → report --verify.

- **M-4 signed update + support** [type: CLI|file] [atoms: M4-E016..A003] [planned]
  CLI: `elai update check|apply|rollback|channel`. Rollback restores prior binary and verifies hash before exec. Boot-time non-blocking update check. Support: `elai support bundle` → `workflow/support/bundle-{date}.tar.zst` G-9-redacted; prints content summary BEFORE user shares. Expect SSB: UpdateAvailable, RefusedHash on bad artifact, Applied, RolledBack. Windows restart-activation wording required not drafted (M4-A003).

- **M-5 commercial packaging + elai-verify + tiers** [type: binary|docs|config] [atoms: M5-E009..A004] [planned WAVE M1]
  Standalone `elai-verify` crate/binary (zero engine deps): wraps G-8 attestation verify + G-2 `rra verify` + K-6 bundle conformance + M-3 org-bundle verify. Trigger clean machine: `elai-verify attestation fixtures/run.attestation && elai-verify rra fixtures/run.rra`. Subcommand grammar for org-bundle beyond attestation/rra implied not fully typed (M5-A004).
  tiers.toml Free/Pro/Team → feature keys for M-1 tier() + K-24 credit grants; `elai license status` renders names/features from tiers.toml. Pricing truth: claims-lint pricing recognizer greps copy vs tiers.toml.
  Docs: support-expectation page (honest solo posture, no fake SLA — published page not runtime panel); EULA/DPA drafts; trust-continuity escrow-on-lapse OR conversion-to-OSS-on-abandonment under `docs/strategy/commercial/`. Exact feature-key matrix + numeric prices data-only not fixed in plan (M5-A003).

- **M-6 EEF v1 open evidence format** [type: CLI|docs|fixtures] [atoms: M6-E002..I001] [planned WAVE M3 behind GATE M-0]
  Prose spec + schemas + conformance fixtures at docs/strategy/eef/ (or sibling repo). `elai-verify conform <artifact>` pins spec version; valid PASS, one-byte-tamper/missing-field FAIL. Third-party emitter worked example. Fail-closed R1 normative (missing evidence = FAIL, UNVOUCHED never clean). Published beside elai-verify. Golden-lock schema file joins gate-manifest R8. v1 scope = only artifacts elai-verify already verifies.

- **SUMMARY commercial surface registry** [type: index] [atoms: SURF-001..016, IX-021]
  License CLI; SettingsWindow (superseded target); Org-policy CLI; Fleet CLI; Update CLI; Support CLI; elai-verify; elai-verify conform; Entitlement record; OrgPolicyPack; Organization bundle; Release manifest; Support bundle contents; Legal/commercial docs; tiers.toml; Published EEF.

### User flows

- FLOW M-1 activate [M1-E022,M1-E036,M1-E040] [planned]: install (B-43 offers activate one line) OR later `elai license activate` → signed entitlement → `elai license status` (tiers.toml features) → near expiry ExpiredGrace banner 14 days → DegradedToFree; deactivate CLI available.
- FLOW M-2 policy distribute [M2-E015,M2-E018] [planned]: admin `org policy init|sign` → seats apply pack → boot verify hash → match ok OR Drift event + boot warning + visible in org report.
- FLOW M-3 fleet report [M3-E019..E032] [planned]: seats `export compliance --org` → admin `org report <dir> --verify` → human md + machine file; broken chain fails verify.
- FLOW M-4 update [M4-E022,M4-E034,M4-E035] [planned]: boot check non-blocking OR `elai update check` → apply with blake3 per artifact → RefusedHash if mismatch → fix → Applied; `rollback` hash-verifies prior; `channel` selects stable/beta.
- FLOW M-4 support [M4-E028,M4-E029,M4-E039] [planned]: `elai support bundle` → print summary → user shares only after seeing contents; secret-grep empty (G-9).
- FLOW M-5 verify offline [M5-E034,M5-E040..E043] [planned]: clean engine-free machine → elai-verify attestation/rra/org-bundle fixtures PASS; one-byte tamper FAIL.
- FLOW M-6 conform [M6-E038,M6-E040] [planned]: `elai-verify conform fixtures/run.rra` PASS; tampered FAIL; third-party emitter example accepted by conform.

### Onboarding-relevant surfaces
- B-43 install-end activation offer (one line) + M-1 license CLI (commercial spine start).
- tiers.toml Free/Pro/Team (entitlement feature matrix source of truth).
- Free tier = full L-shipped surface set except Team-keyed features.
- support-expectation page (honest support posture).
- EULA/DPA/continuity docs under docs/strategy/commercial/.
- elai-verify as trust continuity artifact (verify without engine).
- M-4 channel selection (stable/beta) for release onboarding.

### Notifications / background surfaces
- ExpiredGrace status banner (user-visible; placement open).
- M-2 boot warning on OrgPolicy Drift.
- M-4 boot-time non-blocking update check; UpdateAvailable / Applied / RolledBack journal rows.
- M-3 Drift visibility in org rollup.
- No in-app toast system named beyond banner/boot warning.

### Decide-at-edit / open UI decisions
- M1-A002: ExpiredGrace banner panel/window placement, copy, dismiss.
- M1-A003 / M5-A003: Free vs Pro vs Team feature-key inventory + prices = tiers.toml content not plan-fixed.
- M1-C001 / AC-003: SettingsWindow license-status line is summary-only, not numbered M-1; treat as W-4 deferred / superseded target UI.
- M2-A003: boot warning CLI-only vs UI badge.
- M3-A002: `elai org report` without `--verify` behavior.
- M4-A003: Windows restart-activation UX wording undrafted.
- M5-A004: elai-verify org-bundle verb list incomplete.
- M6: multi-version negotiation protocol not defined (spec_version + pin implies coexistence).

---

## Cross-family wiring notes (from assigned range only)

- K-3/K-11 PR evidence is the meet-the-buyer surface; dashboard/cockpit is deep view (KD-2). L-23/L-24 and K-20 ride K-11/N-7 for human render.
- K-24 billing composes J-28/J-49/H-9; M-1 tier() and M-5d tiers.toml feed grants; Free tier defined relative to L.
- L-14 receipt path reuses I-40 RRA; M-5/M-6 elai-verify verifies those artifacts offline.
- L-12/L-13 activation of dark skill/rule doors; FS-2 same gates as engine (skillspector/J-102/VibeGate).
- Default-OFF experiments: `ELAI_WORKSPACE_ENABLE` (K-23), `ELAI_BILLING_ENABLE` (K-24), `ELAI_CGG_REPAIR_ENABLE` (L-15); kill-switches named throughout K/L.
- Claims discipline (K-1 lint) gates marketing copy for tiers, pricing, OWASP, re-exec, self-improving, context, and commercial docs.

---

## Count summary (extractor)

| Family | Surfaces (inventory bullets) | Named flows |
|--------|------------------------------|-------------|
| K | ~35 | 10 |
| L | ~30 | 8 |
| M | ~12 (+16 SUMMARY SURF registry) | 7 |
