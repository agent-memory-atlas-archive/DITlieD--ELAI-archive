# P-family aggregate — ELAI deep feature discovery

## Document contract

- **Family:** P (PERSONAL-SOFTWARE INCARNATION + COMMAND-CENTER + COMPUTER-USE-STEAL + OPENCLAW-STEAL + CREATIVITY-FALSIFIER; P-0..P-25).
- **Inputs:** `DEEP-P-001.discovery.md`, `DEEP-P-002.discovery.md`, `DEEP-P-003.discovery.md`, `16-P-SUMMARY_INDEX.discovery.md`.
- **Policy:** Every distinct atomic record is preserved with its ID and source anchor (ledger file + line). No prose compression. No silent drop of minor atoms.
- **Namespacing:** Deep atoms keep native IDs under ledger prefix `P-001/`, `P-002/`, `P-003/`. Summary atoms use `SUMMARY/`. Cross-file IX/AC/SD stay namespaced by ledger (e.g. `P-001/IX-003`, `P-002/SD-P1-001`).
- **NEWER-PLAN PRECEDENCE (within family):** An explicit SUPERSEDED/replacement/CORRECT marker in a source beats everything; otherwise a higher plan number is newer and wins the **target state** description. Losing descriptions are retained as superseded notes with anchors. **Deep ledgers supersede the summary** where they conflict (many Tier-1 vs Tier-2 corrections).
- **Status language preserved:** PLANNED / DEFERRED / PROOF_SPINE none / RECON_NEEDED / WEAK-until-RECON are not collapsed. Plan language is not treated as implementation evidence.

### Ledgers merged

| Ledger | Path | Feature atoms (E/I/A/C) | IX | AC | SD | Subtotal |
|---|---|---:|---:|---:|---:|---:|
| P-001 | `docs/thefinalglance/feature-discovery/deep/DEEP-P-001.discovery.md` | **267** (E=234 I=5 A=23 C=5) | 20 | 12 | 27 | **326** |
| P-002 | `docs/thefinalglance/feature-discovery/deep/DEEP-P-002.discovery.md` | **269** (E=220 I=0 A=42 C=7) | 22 | 16 | 38 | **345** |
| P-003 | `docs/thefinalglance/feature-discovery/deep/DEEP-P-003.discovery.md` | **225** (E=191 I=7 A=22 C=5) | 19 | 12 | 38 | **294** |
| SUMMARY | `docs/thefinalglance/feature-discovery/16-P-SUMMARY_INDEX.discovery.md` | inventory **698** bold IDs | — | — | — | **698** |
| **Aggregate distinct records** | — | — | — | — | — | **1663** |

Declared deep footers vs extract (exact match):
- P-001: **267** unique source atoms (234 E + 5 I + 23 A + 5 C) + 20 IX + 12 AC + 27 SD = **326** records.
- P-002: **269** unique source atoms (220 E + 0 I + 42 A + 7 C) + 22 IX + 16 AC + 38 SD = **345** records.
- P-003: **225** unique source atoms (191 E + 7 I + 22 A + 5 C) + 19 IX + 12 AC + 38 SD = **294** records.
- SUMMARY: declared 629 explicit + 6 inference + 63 ambiguity = **698** plan inventory labels; bold-ID extract **698/698**.

- P-001 extracted: atoms=267 E=234 I=5 A=23 C=5; IX=20; AC=12; SD=27.
- P-002 extracted: atoms=269 E=220 I=0 A=42 C=7; IX=22; AC=16; SD=38.
- P-003 extracted: atoms=225 E=191 I=7 A=22 C=5; IX=19; AC=12; SD=38.

## Notation

- `[E]` explicit · `[I]` inference · `[A]` ambiguity · `[C]` conflict/correction · `[Δ]` summary delta · `[IX]` interaction · `[AC]` ambiguity/conflict register · `[SURFACE]` user-facing · `[superseded]` losing target-state retained.
- Class/domain from deep tables is preserved as `E/CAP`, `E/SURFACE`, etc.
- Anchors: `P-N:Lx-Ly` = P-plan original detail; `16-P:Lx-Ly` / `P-G` = summary orientation.

## Per-unit atomic records (deep ledgers)

Feature and unit-local A/I/C atoms. Cross-file IX, AC registers, and SD records follow in dedicated sections. Summary inventory follows after deep ledgers.

## Ledger P-001

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-P-001.discovery.md`

### Unit `P-0` — P-plan index (doctrine, catalog, sequencing) — from P-001 (64 records)

- **P-001/P0-E001** [E] class=`E/STATUS` anchor=`P-0:L1-L2` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L36): P-plan is the PERSONAL-SOFTWARE INCARNATION side, wiring-enforced, dated 2026-07-07 from the spec-eval design digest, with third/fourth waves 2026-07-11, owner-acked `PROOF_SPINE: none`.
- **P-001/P0-E002** [E] class=`E/EVIDENCE` anchor=`P-0:L4-L8` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L37): Primary synthesis source is `workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md` (T1..T6, GAP-1..GAP-8, A-1..A-7). Digest gaps map 1:1 to P-1..P-8.
- **P-001/P0-E003** [E] class=`E/EVIDENCE` anchor=`P-0:L9-L12` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L38): Cross-source anchors include r/ClaudeAI bespoke-tools thread archive and Ink & Switch malleable-software essay + geoffreylitt.com.
- **P-001/P0-E004** [E] class=`E/STATUS` anchor=`P-0:L14-L19` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L39): SECOND WAVE command-center units P-9..P-11 source `elai-command-center-eval-2026-07-08.md`; ranked changes 1/2/3 = P-9/P-10/P-11; all `PROOF_SPINE: none`, PSR-2-acked 2026-07-09, plan_gate_check exit 0, execute after proof spine.
- **P-001/P0-E005** [E] class=`E/CAP` anchor=`P-0:L20-L23` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L40): P-9 is connector-pack admission: signed versioned declarative connector/stage-pack with ONE generic `http_call` kind parameterized by a DATA spec, plus a code lane (B-37), mounted through L-12/skillspector/J-102/N-6; KEYSTONE of second wave.
- **P-001/P0-E006** [E] class=`E/CAP` anchor=`P-0:L24-L26` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L41): P-10 is IngressAdapter family normalizing pull/push/streaming/other to taint-tagged journaled SsbEvent (O-2 trigger enum must not grow) plus first-class CredentialVault (secrets never journaled/exported).
- **P-001/P0-E007** [E] [SURFACE] class=`E/CAP` anchor=`P-0:L27-L28` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L42): P-11 is `elai workflow from-goal`: intent to CLOSED-enum-constrained proposed TaskSpec graph on O-4 canvas, human-confirmed before run, with L-14 RRA receipt.
- **P-001/P0-E008** [E] class=`E/FLOW` anchor=`P-0:L29-L31` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L43): Second-wave sequencing is P-9 then P-10 then P-11. GAP-4 (P-8 merge spike) and GAP-5 (distribution sequencing) are not new files.
- **P-001/P0-E009** [E] class=`E/STATUS` anchor=`P-0:L33-L41` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L44): THIRD WAVE COMPUTER-USE-STEAL P-12..P-16 sources `canvas-computer-use-eval-2026-07-11.md`; central finding is no general desktop/GUI computer-use lane and no agentic-run-mints-deterministic-replayable-artifact instance.
- **P-001/P0-E010** [E] [SURFACE] class=`E/CAP` anchor=`P-0:L42-L44` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L45): P-12 desktop/GUI operator lane: cua sandbox substrate + OmniParser Tier-3 pixel fallback + openclaw frameId/display fail-closed binding + per-action bounded screen-recording + crosshair/zoom crops, all under ELAI gates; cua in-loop safety is a stubbed TODO.
- **P-001/P0-E011** [E] class=`E/CAP` anchor=`P-0:L45-L48` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L46): P-13 perception provenance + Set-of-Marks: VLM picks integer id off tagged list never coordinates; source tags `axtree_exact \| ocr_text \| vlm_caption \| pixel_yolo`; Tier-3 pixel only when AXTree empty/provably incomplete; counts never from VLM.
- **P-001/P0-E012** [E] class=`E/CAP` anchor=`P-0:L49-L52` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L47): P-14 explore-then-crystallize: exploration emits deterministic Playwright/pyautogui script as O-2 TaskSpec stage with G-2 RRA binding; feeds PS-5.
- **P-001/P0-E013** [E] class=`E/CAP` anchor=`P-0:L53-L56` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L48): P-15 verified case-graph: agent-s S2 retrieve/distill/fuse gated by J-96 mint-on-proof; bBoN narrative judge over journal rows; feeds PS-5.
- **P-001/P0-E014** [E] class=`E/CAP` anchor=`P-0:L57-L61` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L49): P-16 gated install catalog: `{tool, source, mirror, alias, license_tag}` + installPolicy behind L-12/skillspector/N-6; headless P-9 code-pack first, P-12 GUI last resort.
- **P-001/P0-E015** [E] class=`E/CAP` anchor=`P-0:L62-L72` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L50): P-23 journaled co-browse: canvas browse over E-14; Tier-1 read-only extraction; Tier-2 CDP screencast dumb glass + ONE InputProxy journaling actor-tagged input before dispatch; vault-custody profiles via P-10; P-6 gated outbound text; G-1-tainted browse content; native Tauri multiwebview excluded as mechanism.
- **P-001/P0-E016** [E] class=`E/CAP` anchor=`P-0:L73-L82` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L51): P-24 mobile design/test: three rungs (DeviceProfile + device_frame/E-14 matrix; Expo-web design mode; native emulator as P-12 backend + mobile_axtree + Maestro crystallize actuation-only); provisioned via P-16.
- **P-001/P0-E017** [E] class=`E/STATUS` anchor=`P-0:L84-L91` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L52): FOURTH WAVE OPENCLAW-STEAL P-17..P-22 sources `openclaw-vs-elai-comparison-2026-07-11.md`; steal is mechanics, not trust spine.
- **P-001/P0-E018** [E] class=`E/CAP` anchor=`P-0:L92-L95` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L53): P-17 provider failover + honest routing economics; feeds PS-4.
- **P-001/P0-E019** [E] class=`E/CAP` anchor=`P-0:L96-L100` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L54): P-18 durable TaskSpec exec + approval drift-binding: revision-counter concurrency, sticky-cancel, retry ladders, watchdog, anti-TOCTOU hashed plan + operand fingerprints, commitment due-time clamp; lobster resume-token is RECON_NEEDED/UNVERIFIED-external.
- **P-001/P0-E020** [E] [SURFACE] class=`E/CAP` anchor=`P-0:L101-L104` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L55): P-19 config/ops lifecycle + `elai doctor` + verified backups: fail-closed defaults, LKG/rejected-write/shrink-guard/atomic writes, doctor detect/repair/lint, VACUUM-INTO backups, restart continuation, compaction split-rule.
- **P-001/P0-E021** [E] class=`E/CAP` anchor=`P-0:L105-L108` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L56): P-20 context/memory lifecycle: pre-compaction memory-flush, versioned skill/rules index, Dreaming promotion gates prior art, per-provider cache-control mapping.
- **P-001/P0-E022** [E] class=`E/CAP` anchor=`P-0:L109-L114` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L57): P-21 gate-credibility: verify-the-verifier canaries hard-block whole gate exitCode=1; missing-artifact=FAIL with in-manifest escape; skipBackfill sibling-pass dep; STRUCTURAL redaction; posture labels; feeds PS-6.
- **P-001/P0-E023** [E] class=`E/CAP` anchor=`P-0:L115-L119` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L58): P-22 control-plane/channel hardening: fenced child output, depth-scoped tool-strip, 9-step channel-binding, pairing codes, dmScope, challenge-signed pairing, unknown-event fail-closed, mandatory idempotency keys, trusted-policy hook tier.
- **P-001/P0-E024** [E] class=`E/STATUS` anchor=`P-0:L124-L129` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L59): Proof-spine feeds: P-14/P-15→PS-5, P-17→PS-4, P-21→PS-6; P-12/P-13/P-16/P-18/P-19/P-20/P-22 and P-23/P-24 are product/ops/distribution with no PS number; all execute after proof spine (A-7); deep red-team deferred to execution pre-flight.
- **P-001/P0-E025** [E] class=`E/STATUS` anchor=`P-0:L131-L138` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L60): P is product-surface successor to O-plan (personal research machine → personal software-fleet machine); composes B..O, restates none (GD-1).
- **P-001/P0-E026** [E] class=`E/DEP` anchor=`P-0:L139-L146` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L61): Mandatory predecessor reading before any P-N execution: incarnation digest, ELAI-FEATURE-MAP, O-0, F-0 (W-1..W-10), G-0 (GD-1..GD-5), this index, P-SUMMARY_INDEX, then target detail.
- **P-001/P0-E027** [E] class=`E/STATUS` anchor=`P-0:L149-L156` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L62): Central personal-incarnation finding: trust spine exists; missing last layer lifts grain changeset→app, task→fleet, seat→world, org→household/team.
- **P-001/P0-E028** [E] class=`E/CAP` anchor=`P-0:L157-L158, L220-L221` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L63): P-1 KEYSTONE app-fleet: AppManifest + FleetRegistry + O-2 health-tick TaskSpecs + G-10 per-app canaries + gated auto-repair (answers T2).
- **P-001/P0-E029** [E] class=`E/CAP` anchor=`P-0:L159-L160, L222-L223` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L64): P-2 inter-tool ports + world-grain bus on O-4 (answers T3).
- **P-001/P0-E030** [E] [SURFACE] class=`E/CAP` anchor=`P-0:L161-L162, L224-L225` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L65): P-3 persona/mechanism split + `elai app export/import` re-verified on recipient seat (answers T5).
- **P-001/P0-E031** [E] class=`E/CAP` anchor=`P-0:L163-L164, L226-L227` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L66): P-4 living external brains; GATED behind Obsidian-API RECON.
- **P-001/P0-E032** [E] [SURFACE] class=`E/CAP` anchor=`P-0:L165-L166, L228-L229` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L67): P-5 graduated-ceremony toy/personal profile + non-coder operator surface (answers T1 non-coder).
- **P-001/P0-E033** [E] class=`E/CAP` anchor=`P-0:L167-L168, L230-L231` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L68): P-6 life-data privacy lattice as dimension on N-6 (answers T6).
- **P-001/P0-E034** [E] class=`E/CAP` anchor=`P-0:L169-L170, L232-L233` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L69): P-7 per-user OperatingProfile, asymmetric loosen only on measured own-run FP evidence; three archetypes as defaults on one engine.
- **P-001/P0-E035** [E] class=`E/CAP` anchor=`P-0:L171-L172, L234-L235` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L70): P-8 2-10 human shared world; hard kernel is lossless deterministic two-seat SSB-stream merge (spike first).
- **P-001/P0-E036** [E] class=`E/STATUS` anchor=`P-0:L188-L191` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L71): Status legend: PLANNED / IN PROGRESS / DONE / DEFERRED. DONE requires full wiring done-gate (wiring_contract_check --run-trigger, production_caller/python_wire clean, R10 smoke); cargo-green is necessary but not sufficient.
- **P-001/P0-E037** [E] class=`E/LIMIT` anchor=`P-0:L196-L204` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L72): Inherits W-1..W-10 and GD-1..GD-5 verbatim; adds no new delta family. A-1..A-7 enforced as per-unit stub_ban + REDTEAM. A-1 binds N-2/N-4/N-6/N-11 single lattice/claim/fold.
- **P-001/P0-E038** [E] class=`E/STATUS` anchor=`P-0:L206-L209` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L73): All of P-1..P-8 are `PROOF_SPINE: none` (product-surface); authored now, executed after N-1 + PS-1..PS-6 (A-7).
- **P-001/P0-E039** [E] [SURFACE] class=`E/TEST` anchor=`P-0:L211-L214` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L74): DONE definition: cargo check GREEN; auditor PASS on affected crates only; wiring_contract_check --run-trigger PASS; wire checker clean or WIRE-DARK[P-N]; cross_lang_diff for P-2/P-5 frontend pairs; corpus_composition_check (N-2) PASS; R10 smoke green; memory-writer ran.
- **P-001/P0-E040** [E] class=`E/DATA` anchor=`P-0:L293-L306` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L75): SCHEMA_VERSION bump sites per unit (never hard-code baseline; re-read ssb.rs + action_type_lock.rs at edit time W-9): P-1 fleet_health_tick; P-2 tool_port_edge; P-3 app_export/app_import; P-4 vault_ingest/claim_violation; P-5 ceremony_profile_set; P-6 privacy_grant/privacy_read_refusal; P-7 operating_profile_change; P-8 seat_journal_merge + seat_id/agent_id columns; P-12 operator_action/operator_frame_bind; P-13 prefer-reuse J-1/J-8 (net 0..+1); P-14 crystallized_stage_minted; P-15 reuse-hit field on J-96 (0..+1); P-16 install_catalog_admit; P-17 provider_failover_event (+ typed billed/estimate on J-28); P-18 taskspec_run_revision/approval_drift_reject; P-19 config_write_rejected/doctor_finding; P-20 memory_flush_event; P-21 prefer-reuse Q-6 posture/canary (0..+1); P-22 subagent_return_fenced/idempotency_replayed; P-23 cobrowse_session/cobrowse_input; P-24 emulator_session (+ device_profile/render_source prefer-reuse).
- **P-001/P0-E041** [E] class=`E/FLOW` anchor=`P-0:L311-L317` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L76): Advisory sequencing WAVE 1: P-5 + P-1 parallel (KEYSTONE P-1); WAVE 2: P-2 + P-6 + P-4 (after Obsidian RECON); WAVE 3: P-3 + P-7; WAVE 4: P-8 spike then team grain.
- **P-001/P0-E042** [E] class=`E/OBS` anchor=`P-0:L318-L319` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L77): Every wave carries A-2 falsifier from day one: fleet-health green %, wired-edge count, exported-tool re-verification success rate, toy-tier measured gate tax, profile-promotion FP rate, merge determinism.
- **P-001/P0-E043** [E] class=`E/LIMIT` anchor=`P-0:L324-L327` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L78): Hard rules: blake3 not blake2/sha2; no .unwrap()/.expect() on prod; scoped tests auditor-only; anti-slopsquatting; closed enums deny_unknown; kill-switch not enable-flag (W-2); external claims WebSearch-verified at authoring.
- **P-001/P0-E044** [E] class=`E/LIMIT` anchor=`P-0:L332-L332` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L79): Carried correction P-1: gated auto-repair is a CHANGESET through same wiring/claim gates, never silent patch.
- **P-001/P0-E045** [E] class=`E/LIMIT` anchor=`P-0:L333-L333` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L80): Carried correction P-2/P-6: ride N-6 ONE lattice; second provenance/privacy scale is authoring FAIL.
- **P-001/P0-E046** [E] class=`E/LIMIT` anchor=`P-0:L334-L334` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L81): Carried correction P-4: no code against training-asserted store API; Obsidian/Notion RECON BLOCKING.
- **P-001/P0-E047** [E] class=`E/LIMIT` anchor=`P-0:L335-L335` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L82): Carried correction P-5: observe-only still RUNS + JOURNALS; toy reduces ceremony never skips evidence.
- **P-001/P0-E048** [E] class=`E/LIMIT` anchor=`P-0:L336-L336` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L83): Carried correction P-7: loosens ONLY on measured own-run FP; user affect never loosens a gate.
- **P-001/P0-E049** [E] class=`E/LIMIT` anchor=`P-0:L337-L337` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L84): Carried correction P-8: merge lossless + deterministic + local-first; no LWW; no org SaaS control plane.
- **P-001/P0-E050** [E] class=`E/LIMIT` anchor=`P-0:L338-L341` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L85): Carried correction P-12: cua in-loop safety is stubbed TODO; operator-action is G-1 taint + operator lane never load-bearing; OmniParser icon_detect AGPL / Florence MIT productization license anchor.
- **P-001/P0-E051** [E] class=`E/LIMIT` anchor=`P-0:L342-L343` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L86): Carried correction P-13: counts never from VLM; axtree_exact only authoritative; per-crop caption never element identity (S8 anti-pattern).
- **P-001/P0-E052** [E] [SURFACE] class=`E/LIMIT` anchor=`P-0:L344-L346` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L87): Carried correction P-14: openclaw jsonl-replay is INPUT-through-mock not real record→re-score; cite cua replay_env; script without minting-run receipt refused.
- **P-001/P0-E053** [E] class=`E/LIMIT` anchor=`P-0:L347-L349` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L88): Carried correction P-15: do not copy agent-s ungated save; bBoN over journal rows never screenshots.
- **P-001/P0-E054** [E] class=`E/LIMIT` anchor=`P-0:L350-L351` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L89): Carried correction P-16: sanctioned download form is P-9 headless code-pack; P-12 GUI last resort.
- **P-001/P0-E055** [E] class=`E/LIMIT` anchor=`P-0:L352-L356` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L90): Carried correction P-18: lobster resume-token UNVERIFIED-external C1; author real subprocess-checkpoint; do not claim openclaw shipped resume-token.
- **P-001/P0-E056** [E] class=`E/LIMIT` anchor=`P-0:L357-L358` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L91): Carried correction P-19: do opposite of openclaw full process.env copy (env allowlist); adopt cwd path-jail case-fold-before-compare on win32.
- **P-001/P0-E057** [E] class=`E/LIMIT` anchor=`P-0:L359-L362` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L92): Carried correction P-21: copy unknown=FAIL not classified-failure-pass; keep metadata audit and content debug bundle SEPARATE BY TYPE.
- **P-001/P0-E058** [E] class=`E/LIMIT` anchor=`P-0:L363-L365` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L93): Carried correction P-22: do not port ClawHub no-default-gate or advisory regex scanner; P-16 install is fail-closed via L-12.
- **P-001/P0-E059** [E] class=`E/STATUS` anchor=`P-0:L367-L386` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L94): FIFTH WAVE P-25 exogenous-injection + evidence-coupled-revision creativity falsifier (B-EXO-0): six arms, two preregistered families, K=30 fixtures; harness-only; PROOF_SPINE none; NOT spine-gated; SUPPORTED family authors adoption unit; BOTH NULL/REFUTED closes creativity line.
- **P-001/P0-E060** [E] class=`E/STATUS` anchor=`P-0:L388-L398` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L95): Session log empty at generation; execution gated behind proof spine; next step WAVE 1 P-1 + P-5 in parallel after RECON greps for P-1.
- **P-001/P0-E061** [E] [SURFACE] class=`E/SURFACE` anchor=`P-0:L133-L137` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L96): P ships new user-facing features: app fleet, inter-tool bus, hand-to-friend export, living brains, non-coder operator surface, life-data privacy lattice, per-user operating profile, small-team world.
- **P-001/P0-I001** [I] class=`I/DEP` anchor=`P-0:L20-L23, L57-L61, L255-L257` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L97): Because P-9 is second-wave KEYSTONE and P-16 depends on P-9/P-10, install-catalog execution is blocked until connector-pack + vault land even though P-0 lists CU-3 after CU-1/CU-2.
- **P-001/P0-A001** [A] class=`A/STATUS` anchor=`P-0:L236-L237` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L98): P-0 allocates P-9..P-11 only in prose (no table rows); detail files exist but are outside this deep chunk.
- **P-001/P0-A002** [A] class=`A/FLOW` anchor=`P-0:L243-L288, L311-L317` unit=`P-0` (ledger-line: DEEP-P-001.discovery.md:L99): Computer-use wave labels CU-1..CU-5 and OpenClaw OC-1..OC-3 interleave with personal waves without a single total order beyond A-7 spine-first.

### Unit `P-4` — living external brains — from P-001 (26 records)

- **P-001/P4-E001** [E] class=`E/STATUS` anchor=`P-4:L3-L5` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L109): P-4 is PLANNED WAVE 2, product-surface PROOF_SPINE none, owner-acked 2026-07-07, BUILD-on-shipped, GATED behind Obsidian/store-API RECON before any code; WEAK-until-RECON; effort M-L; depends O-3 + P-6.
- **P-001/P4-E002** [E] class=`E/CAP` anchor=`P-4:L7-L7` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L110): Premise: lift O-3 from deterministic markdown vault to living stores (Obsidian concurrent plugins, Notion, personal SQLite/spreadsheets) with bidirectional-safe per-class contract.
- **P-001/P4-E003** [E] class=`E/DATA` anchor=`P-4:L10-L10` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L111): `LivingStoreContract = { class: Obsidian\|Notion\|SQLite\|Spreadsheet, watch (change feed), ingest (F-5 staleness + N-6 tier per byte), write_scope (O-3 namespace-claims — writes REFUSED outside claimed namespaces), ttl }`.
- **P-001/P4-E004** [E] class=`E/FLOW` anchor=`P-4:L10-L10, L15-L15` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L112): B-35 context-artifacts pipeline is lifted from static paths to living stores with TTL + change feeds.
- **P-001/P4-E005** [E] class=`E/PRIV` anchor=`P-4:L10-L10, L17-L17, L32-L32, L41-L43` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L113): D-5/O-3 life-layer denylist becomes HARD ROUTING WALL: life-data never enters project/cloud prompt without explicit journaled grant (P-6 boundary consumed).
- **P-001/P4-E006** [E] class=`E/LIMIT` anchor=`P-4:L10-L10` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L114): Contract is per store CLASS not per user (A-5 two-masters); vault path is data.
- **P-001/P4-E007** [E] class=`E/LIMIT` anchor=`P-4:L11-L11, L63-L66` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L115): Excluded: convert live store into memory_facts; write outside claimed namespace; ingest without N-6 tier + F-5 staleness; life-data to cloud without P-6 grant; any code against unverified store-API surface.
- **P-001/P4-E008** [E] class=`E/DEP` anchor=`P-4:L13-L17, L56-L57, L91-L92` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L116): Consumes O-3 vault + namespace-claims, B-35, F-5, N-6, D-5 denylist, P-6 + E-20 routing.
- **P-001/P4-E009** [E] class=`E/DEFER` anchor=`P-4:L19-L23` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L117): RECON_NEEDED (BLOCKING): Obsidian plugin/URI/file surface for watch+safe write; Notion API write/permission/rate limits; O-3 namespace-claims reuse vs per-class adapter; B-35 change-feed vs static-path entrypoint.
- **P-001/P4-E010** [E] class=`E/CAP` anchor=`P-4:L26-L26, L52-L55` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L118): Tier-1 AFTER Obsidian RECON: LivingStoreContract type + Obsidian filesystem-watch adapter (watch + F-5/N-6 ingest + WRITE-into-claimed-namespace only) + journaled `vault_ingest` / `claim_violation` SSB rows.
- **P-001/P4-E011** [E] class=`E/DEFER` anchor=`P-4:L27-L27` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L119): Tier-2: Notion adapter; SQLite/spreadsheet adapters; bidirectional write-back beyond append (concurrent-edit merge spike kin to P-8).
- **P-001/P4-E012** [E] class=`E/INT` anchor=`P-4:L30-L30` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L120): Integration in crates/knowledge: LivingStoreContract + Obsidian watch adapter + F-5/N-6 tagging + O-3 claims-gated writer + SSB emit.
- **P-001/P4-E013** [E] [SURFACE] class=`E/SURFACE` anchor=`P-4:L31-L31, L55-L55, L58-L59, L72-L72` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L121): CLI production caller: `elai brain mount\|sync <store>`.
- **P-001/P4-E014** [E] class=`E/OBS` anchor=`P-4:L35-L37` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L122): M1 expect: edit mounted Obsidian note fires change-feed and lands `vault_ingest {note_path, n6_tier, staleness_ttl}`; concurrent plugin edit ingested at settled state not mid-write.
- **P-001/P4-E015** [E] class=`E/FAIL` anchor=`P-4:L38-L40, L68-L70` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L123): M2: write into ELAI-claimed namespace succeeds; write outside lands `claim_violation` and hard refuses.
- **P-001/P4-E016** [E] class=`E/PRIV` anchor=`P-4:L41-L43` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L124): M3: life-domain note toward cloud without grant blocked+journaled; with grant routes with grant on record.
- **P-001/P4-E017** [E] class=`E/DEFER` anchor=`P-4:L46-L46` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L125): FORK: if Obsidian RECON finds no safe write surface, ship Tier-1 READ-ONLY mount (watch+ingest+tier, no write-back) and defer write-back.
- **P-001/P4-E018** [E] class=`E/FAIL` anchor=`P-4:L49-L49` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L126): ABORT: stop if any code would target training-asserted ungrounded store-API surface.
- **P-001/P4-E019** [E] [SURFACE] class=`E/INT` anchor=`P-4:L58-L61` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L127): Reachability: live CLI mount/sync + live filesystem-watch on vault edit; `#[cfg(test)]` never satisfies R7.
- **P-001/P4-E020** [E] class=`E/TEST` anchor=`P-4:L75-L78` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L128): VERIFY off-agent: wiring_contract_check + production_caller_check; pass = vault_ingest with N-6+F-5 TTL + claim_violation on unclaimed write.
- **P-001/P4-E021** [E] class=`E/TEST` anchor=`P-4:L80-L80` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L129): INTEGRATION_SMOKE: fixture vault claimed+user namespaces, concurrent edits, life-domain cloud route must block without grant; fires concurrency+claim+routing (R10).
- **P-001/P4-E022** [E] class=`E/SEC` anchor=`P-4:L83-L83` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L130): REDTEAM: private health note exfil fails via P-6 wall; concurrent plugin corruption patched by settled-state ingest + claims gate + read-only FORK default.
- **P-001/P4-E023** [E] class=`E/EVENT` anchor=`P-4:L26-L26, L54-L54, L69-L69` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L131): SSB rows: `vault_ingest`, `claim_violation` (baseline bump via P-0 map).
- **P-001/P4-I001** [I] class=`I/FLOW` anchor=`P-4:L10-L10, L27-L27` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L132): Spreadsheet class is listed in the contract enum but has no dedicated Tier-1/Tier-2 adapter paragraph beyond “SQLite/spreadsheet adapters” Tier-2.
- **P-001/P4-A001** [A] class=`A/CFG` anchor=`P-4:L10-L10, L35-L37` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L133): No concrete TTL values, watch debounce, settled-state detection, or change-feed protocol are specified.
- **P-001/P4-A002** [A] class=`A/DATA` anchor=`P-4:L36-L36, L69-L69` unit=`P-4` (ledger-line: DEEP-P-001.discovery.md:L134): Exact `vault_ingest` / `claim_violation` field schemas beyond the named fields are not fully specified.

### Unit `P-5` — operator surface + graduated ceremony — from P-001 (27 records)

- **P-001/P5-E001** [E] class=`E/STATUS` anchor=`P-5:L3-L5` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L144): P-5 PLANNED WAVE 1, PROOF_SPINE none, ~90% composition, effort M, MODERATE grounding.
- **P-001/P5-E002** [E] [SURFACE] class=`E/SURFACE` anchor=`P-5:L7-L7` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L145): Two missing pieces: fleet-grain plain-language operation (“mywardrobe tool broke” → P-1 diagnosis as sentence + gated repair button) and graduated ceremony so toy tools do not pay trading-strategy gate tax.
- **P-001/P5-E003** [E] class=`E/DATA` anchor=`P-5:L10-L10` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L146): `CeremonyProfile { tier: toy\|personal\|project\|critical, gate_mode: observe_only\|enforce, receipt_verbosity: one_line\|full, autonomy }` — declared and journaled.
- **P-001/P5-E004** [E] class=`E/CFG` anchor=`P-5:L10-L10, L21-L21` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L147): At toy: gates observe-only (still RUN and journal; do not block); receipts collapse to one line.
- **P-001/P5-E005** [E] class=`E/LIMIT` anchor=`P-5:L11-L11, L63-L66` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L148): Excluded: silent gate skip; fabricated/collapsed-away evidence; second gate engine; loosening without declared journaled profile choice.
- **P-001/P5-E006** [E] class=`E/DEP` anchor=`P-5:L14-L18, L56-L57` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L149): Rides K-5 risk-tiered gating + K-4 gate-tax ceiling; does not re-author gate engine. Consumes N-7, P-1 fleet red tick, GD-4, W-2.
- **P-001/P5-E007** [E] [SURFACE] class=`E/CAP` anchor=`P-5:L21-L21, L52-L55` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L150): Tier-1: CeremonyProfile type + toy/personal wiring into K-5 observe-only + K-4 tax measurement + one-line receipt + N-7 fleet-grain plain-language render of P-1 red tick.
- **P-001/P5-E008** [E] [SURFACE] class=`E/DEFER` anchor=`P-5:L22-L22` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L151): Tier-2: one-click gated repair BUTTON (with P-1 Tier-2) + full non-coder GUI operator panel.
- **P-001/P5-E009** [E] [SURFACE] class=`E/SURFACE` anchor=`P-5:L26-L26, L53-L53, L59-L59` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L152): CLI: `elai profile set <tier>` + `elai profile tax`; fleet render reads through `elai fleet list` (P-1).
- **P-001/P5-E010** [E] [SURFACE] class=`E/SURFACE` anchor=`P-5:L27-L27` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L153): Tier-2 frontend_pair: fleet operator panel in dashboard-v2 with N-7 plain-language cards + repair button.
- **P-001/P5-E011** [E] class=`E/EVENT` anchor=`P-5:L31-L31, L55-L55, L69-L69` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L154): Setting tier journals `ceremony_profile_set`.
- **P-001/P5-E012** [E] class=`E/FLOW` anchor=`P-5:L30-L32, L68-L69` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L155): M1: toy tier subsequent gated action runs gate, journals verdict, does NOT block on WARN; critical tier blocks.
- **P-001/P5-E013** [E] [SURFACE] class=`E/OBS` anchor=`P-5:L33-L35, L64-L64, L70-L70` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L156): M2: one-line receipt POINTS at full journal entry (never replaces); full evidence retrievable.
- **P-001/P5-E014** [E] [SURFACE] class=`E/OBS` anchor=`P-5:L36-L38` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L157): M3: `elai profile tax --tier toy` reports measured composed gate tax (near-zero for A-3); red tick as one N-7 sentence example: “your wardrobe tool broke: its weather dependency is stale”.
- **P-001/P5-E015** [E] class=`E/FAIL` anchor=`P-5:L32-L32` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L158): ON_FAIL if toy SKIPS gate: make observe-only run+journal+warn; assert journal row exists.
- **P-001/P5-E016** [E] class=`E/FAIL` anchor=`P-5:L49-L49` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L159): ABORT if toy near-zero-tax only achievable by skipping a gate (W-2/R1 forbids).
- **P-001/P5-E017** [E] class=`E/DEFER` anchor=`P-5:L41-L41` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L160): FORK: if K-5 gate-mode hard-coded per-gate, add tier parameter minimally or route observe-only to P-1 fleet ticks; do not fork gate engine.
- **P-001/P5-E018** [E] class=`E/DEFER` anchor=`P-5:L44-L46` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L161): RECON_NEEDED: K-5 gate-mode API; K-4 tier-scoped tax entrypoint; N-7 render-target extensibility.
- **P-001/P5-E019** [E] [SURFACE] class=`E/INT` anchor=`P-5:L58-L61, L72-L72` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L162): Production reachability: `elai profile set|tax` CLI + live gated action at toy tier; test callers never R7.
- **P-001/P5-E020** [E] class=`E/TEST` anchor=`P-5:L75-L78` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L163): VERIFY pass: observe-only at toy (verdict journaled, no block) + enforce at critical + one-line resolves to journal + measured near-zero tax.
- **P-001/P5-E021** [E] class=`E/TEST` anchor=`P-5:L80-L80` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L164): INTEGRATION_SMOKE multi-tier: set toy, drive P-1 red-tick, escalate to critical, assert both journal rows + tax delta (R10).
- **P-001/P5-E022** [E] class=`E/SEC` anchor=`P-5:L83-L83` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L165): REDTEAM: hide failure via green one-liner fails because observe-only journals and one-liner indexes; toy-tier gate-dodge on critical tool patched by P-7 asymmetry + tool risk property.
- **P-001/P5-E023** [E] [SURFACE] class=`E/SURFACE` anchor=`P-5:L7-L7, L21-L27, L33-L38` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L166): User-facing: ceremony profile selection CLI; tax report CLI; one-line receipts; plain-language fleet diagnosis sentence; (Tier-2) repair button + dashboard fleet operator panel.
- **P-001/P5-I001** [I] class=`I/CFG` anchor=`P-5:L10-L10` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L167): `autonomy` is a CeremonyProfile field but no tier→autonomy mapping is defined.
- **P-001/P5-A001** [A] class=`A/CFG` anchor=`P-5:L37-L38, L70-L70` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L168): “Near-zero” tax has no numeric threshold, workload, or CI.
- **P-001/P5-A002** [A] class=`A/CFG` anchor=`P-5:L10-L10, L21-L21` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L169): Blocking/verbosity behavior for `personal` and `project` tiers is not enumerated beyond enum presence.
- **P-001/P5-A003** [A] [SURFACE] class=`A/SURFACE` anchor=`P-5:L22-L22, L27-L27` unit=`P-5` (ledger-line: DEEP-P-001.discovery.md:L170): Exact dashboard drawer/panel identity for Tier-2 operator surface is deferred without wireframe.

### Unit `P-8` — small-team collaboration grain — from P-001 (23 records)

- **P-001/P8-E001** [E] class=`E/STATUS` anchor=`P-8:L3-L5` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L180): P-8 PLANNED WAVE 4, product-surface, BUILD-on-shipped + genuine design SPIKE for shared-journal merge; effort L; depends O-3 + K-23; MODERATE grounding.
- **P-001/P8-E002** [E] class=`E/CAP` anchor=`P-8:L7-L7` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L181): Missing grain is 2-10 humans + their agents in ONE world; hard kernel is defined JOIN of two seats’ SSB streams (none today).
- **P-001/P8-E003** [E] class=`E/CAP` anchor=`P-8:L10-L10` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L182): Steal/build: shared vault namespaces from O-3 claims; cross-seat file/lock discipline lifted from harness; per-seat provenance `seat_id`+`agent_id` on every SSB row; M-3 rollup at shared project; HARD KERNEL is merge semantics (design spike FIRST).
- **P-001/P8-E004** [E] class=`E/LIMIT` anchor=`P-8:L11-L11, L64-L67` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L183): Excluded: org SaaS/hosted control plane; merge that silently drops/reorders events; per-seat code fork; shipping composition before merge kernel defined.
- **P-001/P8-E005** [E] class=`E/DEP` anchor=`P-8:L13-L18, L57-L58` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L184): Consumes O-3 namespace-claims, K-23 multi-repo memory, I-2 isolation, M-2/M-3 rollup, harness multi-session locks, Phase 3H branch/merge precedent.
- **P-001/P8-E006** [E] class=`E/CAP` anchor=`P-8:L21-L21, L53-L56` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L185): Tier-1 SPIKE + minimal join: define lossless deterministic causally-ordered two-seat merge (NOT LWW) + seat_id/agent_id on every row + two-seat shared-namespace R/W with cross-seat locks + journaled `seat_journal_merge` row.
- **P-001/P8-E007** [E] class=`E/DEFER` anchor=`P-8:L22-L22` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L186): Tier-2: full 3-10 seat scale; M-3 rollup at shared project; conflict-resolution UX; agent-attribution query surface.
- **P-001/P8-E008** [E] [SURFACE] class=`E/SURFACE` anchor=`P-8:L26-L26, L56-L56, L60-L60` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L187): CLI: `elai world join\|merge\|whodid <query>` is Tier-1 production caller.
- **P-001/P8-E009** [E] class=`E/DET` anchor=`P-8:L30-L32, L70-L71` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L188): M1: each event keyed by `{seat_id, agent_id, lamport_or_hybrid_clock}`; merge commutative-up-to-total-order; A→B order_hash == B→A; no event dropped.
- **P-001/P8-E010** [E] class=`E/DATA` anchor=`P-8:L33-L35, L54-L54` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L189): M2: add seat_id + agent_id to SSB row schema with SCHEMA_VERSION bump at LIVE baseline (W-9 re-read).
- **P-001/P8-E011** [E] [SURFACE] class=`E/SURFACE` anchor=`P-8:L34-L34, L71-L71` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L190): `elai world whodid <namespace>` returns seat+agent for each change.
- **P-001/P8-E012** [E] class=`E/CONC` anchor=`P-8:L36-L38, L70-L70` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L191): M3: two seats writing same shared namespace serialize through cross-seat lock (no lost update); merge emits `seat_journal_merge {seats, event_count, order_hash}`.
- **P-001/P8-E013** [E] class=`E/DEFER` anchor=`P-8:L41-L41` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L192): FORK: if existing SSB has no causal clock and adding is invasive, stamp hybrid logical clock at emit (additive) rather than retrofit historical rows; never ship LWW placeholder.
- **P-001/P8-E014** [E] class=`E/DEFER` anchor=`P-8:L44-L47` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L193): RECON_NEEDED: SSB row schema/clock; Phase 3H merge precedent; harness cross-session lock API; O-3 two-claimant model.
- **P-001/P8-E015** [E] class=`E/FAIL` anchor=`P-8:L50-L50` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L194): ABORT if lossless deterministic merge requires org-hosted control plane/central server (violates M doctrine + local-first T6).
- **P-001/P8-E016** [E] [SURFACE] class=`E/INT` anchor=`P-8:L59-L62, L73-L73` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L195): Reachability: live `elai world` CLI + shared-namespace write path enforces lock; test never R7.
- **P-001/P8-E017** [E] class=`E/TEST` anchor=`P-8:L76-L79` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L196): VERIFY: deterministic lossless merge identical order_hash both directions; every row has seat_id/agent_id; concurrent write serializes.
- **P-001/P8-E018** [E] [SURFACE] class=`E/TEST` anchor=`P-8:L81-L81` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L197): INTEGRATION_SMOKE: independent agent simulates TWO seats with agents, interleaved+concurrent events, A→B and B→A merge, whodid attribution, serialized concurrent write (R10); single-seat happy path insufficient.
- **P-001/P8-E019** [E] class=`E/SEC` anchor=`P-8:L84-L84` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L198): REDTEAM: history rewrite via merge-order fails (deterministic order_hash); concurrent lost-write patched by product lock lift.
- **P-001/P8-E020** [E] class=`E/DEP` anchor=`P-8:L91-L92` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L199): COMPOSES-WITH O-3 (EXTEND to N seats), K-23, I-2, M-2/M-3, harness locks, 3H; P-7 team default targets this unit; seat provenance serves P-1 fleet attribution.
- **P-001/P8-I001** [I] [SURFACE] class=`I/SURFACE` anchor=`P-8:L22-L22, L26-L26, L34-L34` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L200): Agent-attribution query surface is Tier-2 even though `whodid` is Tier-1 CLI; Tier-1 whodid is the minimal attribution query, richer surface deferred.
- **P-001/P8-A001** [A] class=`A/DET` anchor=`P-8:L21-L21, L30-L32, L89-L89` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L201): Exact merge/CRDT algorithm, clock type choice, and tiebreak details are open design (spike deliverable).
- **P-001/P8-A002** [A] class=`A/SEC` anchor=`P-8:L22-L22` unit=`P-8` (ledger-line: DEEP-P-001.discovery.md:L202): Identity enrollment, revocation, transport, offline reconciliation, and conflict UX unspecified (Tier-2/open).

### Unit `P-12` — desktop/GUI operator lane — from P-001 (26 records)

- **P-001/P12-E001** [E] class=`E/STATUS` anchor=`P-12:L3-L5` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L212): P-12 PLANNED WAVE CU-2, NEW capability, PROOF_SPINE none (product), owner-acked 2026-07-11, execute after proof spine; effort L; depends P-13.
- **P-001/P12-E002** [E] class=`E/CAP` anchor=`P-12:L6-L6` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L213): Adds general desktop/GUI computer-use stage for “drive this GUI app / test this game build”; comparators are vendored substrate + operator lanes, never rivals (I-40 wrap inversion).
- **P-001/P12-E003** [E] class=`E/DEP` anchor=`P-12:L11-L16, L33-L36` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L214): Consumes E-14 browser doctrine (mirror), J-1/J-8 perception (via P-13), G-1 taint, G-2 RRA ActionStream, 4F isolation, 4I evaluate-loop doctrine.
- **P-001/P12-E004** [E] class=`E/CAP` anchor=`P-12:L19-L19` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L215): Tier-1 OperatorLane: (a) vendored cua sandbox adapter (disposable VM/container, MIT); (b) openclaw-style frame-binding: coordinate action authorized only while exact sha256-hashed image model saw is still in context; epoch-invalidated on compaction drop; fail-closed on no-frame/node-switch/geometry-change; NEVER retargets display 0; (c) two-phase arm-commit (journal grant BEFORE exposing capability); (d) dual lease expiry (periodic sweep AND inline at dispatch); (e) per-action bounded screen-recording hashed to off-agent SSB row.
- **P-001/P12-E005** [E] class=`E/DEFER` anchor=`P-12:L20-L20` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L216): Tier-2: OmniParser Tier-3 pixel adapter (lands with P-13); full game-harness capture loop over 4I; multi-monitor.
- **P-001/P12-E006** [E] [SURFACE] class=`E/LIMIT` anchor=`P-12:L21-L21` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L217): Excluded: cua’s own in-loop gating (stubbed TODO — ELAI supplies gate); ungated self-extension.
- **P-001/P12-E007** [E] class=`E/CAP` anchor=`P-12:L30-L32` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L218): Produces `crates/orchestrator/src/operator/mod.rs` (OperatorLane) and `frame_bind.rs` (FrameBinding: sha256(image-bytes) → authorized-action guard).
- **P-001/P12-E008** [E] [SURFACE] class=`E/INT` anchor=`P-12:L37-L40, L75-L76` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L219): Production reachability: task_runner O-2 tick dispatching `operator_action` TaskSpec stage + CLI `elai operator run`; test/contracts never R7.
- **P-001/P12-E009** [E] class=`E/LIMIT` anchor=`P-12:L41-L45` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L220): Stub ban: no actuation on stale/compacted frame_hash; no capability before arm-grant row; no .unwrap/.expect on sandbox path; blake3 for frame hash not blake2/sha2; operator action never trusted/load-bearing.
- **P-001/P12-E010** [E] [SURFACE] class=`E/SURFACE` anchor=`P-12:L46-L47` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L221): frontend_pair: operator-action rows + screen-recording thumbnail in an existing dashboard drawer (exact drawer decided at edit, W-4).
- **P-001/P12-E011** [E] class=`E/EVENT` anchor=`P-12:L50-L51, L54-L54` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L222): expect_ssb `operator_action`; also `operator_frame_bind` row in wargame.
- **P-001/P12-E012** [E] class=`E/OBS` anchor=`P-12:L51-L51` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L223): expect_artifact: bounded screen-recording file hash equals operator_action row frame/record hash.
- **P-001/P12-E013** [E] class=`E/FAIL` anchor=`P-12:L51-L51, L54-L55` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L224): NEGATIVE: action on stale frame_hash, or arm exposing capability before grant row, FAILS.
- **P-001/P12-E014** [E] class=`E/DATA` anchor=`P-12:L54-L54` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L225): operator_action row fields include `{frame_hash, action, lease_id}`.
- **P-001/P12-E015** [E] class=`E/DEFER` anchor=`P-12:L56-L56` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L226): FORK: if cua sandbox cannot pin deterministic image, route Docker/Windows-Sandbox backend and record substrate in RRA hardware pin; NEVER host desktop.
- **P-001/P12-E016** [E] class=`E/DEFER` anchor=`P-12:L57-L57` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L227): RECON_NEEDED: cua sandbox API + MIT license; OmniParser icon_detect AGPL; exact orchestrator compaction hook for frame epoch.
- **P-001/P12-E017** [E] class=`E/FAIL` anchor=`P-12:L58-L58` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L228): ABORT if coordinate action would actuate without live matching frame binding, or sandbox not disposable/isolated (P0).
- **P-001/P12-E018** [E] [SURFACE] class=`E/TEST` anchor=`P-12:L59-L64` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L229): VERIFY via plan_gate_check, wiring_contract_check, production_caller_check; proof is off-agent operator_action + recording-hash match via real CLI.
- **P-001/P12-E019** [E] class=`E/SEC` anchor=`P-12:L65-L65` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L230): REDTEAM: cached frame-hash replay defeated by epoch invalidation; host-desktop fallback patched to ABORT/other sandbox.
- **P-001/P12-E020** [E] class=`E/TEST` anchor=`P-12:L68-L68` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L231): INTEGRATION_SMOKE after P-12+P-13: multi-step click+type long enough for mid-run compaction; assert live bindings, re-bind after invalidation, hash match; toy single-click insufficient R10.
- **P-001/P12-E021** [E] [SURFACE] class=`E/SURFACE` anchor=`P-12:L38-L39, L46-L47, L49-L49` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L232): User-facing surfaces: `elai operator run` CLI; operator_action TaskSpec stage; O-4/operator node implied via task stage; dashboard drawer rows + recording thumbnail.
- **P-001/P12-E022** [E] class=`E/DEP` anchor=`P-12:L34-L34` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L233): PerceptionScaffold from P-13 is consumed, not reimplemented.
- **P-001/P12-C001** [C] class=`C/DEFER` anchor=`P-12:L20-L20; cross-ref P-0:L42-L44` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L234): P-0 index lists OmniParser Tier-3 as part of P-12 package, but this detail defers Tier-3 pixel adapter to Tier-2 with P-13; Tier-1 is sandbox+frame-bind+arm/lease+recording.
- **P-001/P12-A001** [A] class=`A/CFG` anchor=`P-12:L19-L19` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L235): Bounded recording duration/size and lease TTL values unspecified.
- **P-001/P12-A002** [A] [SURFACE] class=`A/SURFACE` anchor=`P-12:L46-L47` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L236): Exact existing dashboard drawer identity left to W-4 at edit.
- **P-001/P12-A003** [A] class=`A/DET` anchor=`P-12:L19-L19, L32-L32, L44-L44, L73-L73` unit=`P-12` (ledger-line: DEEP-P-001.discovery.md:L237): Frame-binding text says sha256 of image bytes; hard rules elsewhere require blake3 — hash algorithm for frame_bind needs reconcile at edit.

### Unit `P-13` — perception provenance + set-of-marks — from P-001 (26 records)

- **P-001/P13-E001** [E] class=`E/STATUS` anchor=`P-13:L3-L5` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L247): P-13 PLANNED WAVE CU-1, EXTENSION of E-14+J-1/J-8/J-77, PROOF_SPINE none, STRONG grounding (OmniParser 70.5→93.8% verified in eval), effort M.
- **P-001/P13-E002** [E] class=`E/CAP` anchor=`P-13:L6-L6, L19-L19` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L248): Set-of-Marks id-indirection: VLM returns integer id off tagged element list, never coordinates; harness owns geometry.
- **P-001/P13-E003** [E] [SURFACE] class=`E/EVIDENCE` anchor=`P-13:L9-L9` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L249): OmniParser captions+IDs lift grounding 70.5→93.8%; ScreenSpot 73.0 vs raw GPT-4V 16.2 (comparator evidence, not ELAI threshold).
- **P-001/P13-E004** [E] class=`E/DEP` anchor=`P-13:L11-L16` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L250): Extends J-1 PerceptionScaffold + /see (counts-from-AXTree-never-VLM); rides J-8 schema; composes J-77 faithfulness; confirms E-14 AXTree doctrine; source tag slots G-1.
- **P-001/P13-E005** [E] class=`E/CAP` anchor=`P-13:L19-L19` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L251): Tier-1: (S1) id-indirection on AXTree node list; (S4) per-element source tag first-class; (S2) interactability flag filtering VLM candidate set.
- **P-001/P13-E006** [E] class=`E/DEFER` anchor=`P-13:L20-L20` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L252): Tier-2: (S3) advisory captions for unnamed canvas/game icons; (S5) asymmetric 80%-containment OCR-into-element fusion; Tier-3 pixel/YOLO adapter only on empty/incomplete AXTree (lands with P-12 game surfaces).
- **P-001/P13-E007** [E] class=`E/LIMIT` anchor=`P-13:L21-L21, L39-L41` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L253): Excluded S8: per-crop caption as element identity; counts from VLM.
- **P-001/P13-E008** [E] class=`E/DATA` anchor=`P-13:L30-L31` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L254): Produces ElementSource enum `{AxtreeExact,OcrText,VlmCaption,PixelYolo}` + box_id field; set_of_marks.rs numbered-list builder + integer resolver.
- **P-001/P13-E009** [E] class=`E/INT` anchor=`P-13:L36-L37, L71-L72` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L255): Reachability: perception gateway service consumed by E-14 + P-12; no new entrypoint claimed in PRODUCTION_HOOK (existing gateway callers); test never R7.
- **P-001/P13-E010** [E] class=`E/LIMIT` anchor=`P-13:L38-L42` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L256): Stub ban: no VLM counts; no element without source tag; only axtree_exact authoritative; no model-returned coordinates; no .unwrap; no blake2/sha2.
- **P-001/P13-E011** [E] [SURFACE] class=`E/SURFACE` anchor=`P-13:L43-L44` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L257): frontend_pair: rides J-1 existing perception tile — source-tagged element breakdown (no new surface).
- **P-001/P13-E012** [E] [SURFACE] class=`E/SURFACE` anchor=`P-13:L46-L46` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L258): Observable CLI trigger in contract: `elai perception scaffold --fixture ... --json`.
- **P-001/P13-E013** [E] class=`E/EVENT` anchor=`P-13:L47-L47` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L259): expect_ssb `perception_scaffold_emitted`.
- **P-001/P13-E014** [E] class=`E/OBS` anchor=`P-13:L48-L48, L51-L51` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L260): Element list carries integer ids + source tag per element; trace shows ZERO VLM count calls.
- **P-001/P13-E015** [E] class=`E/FAIL` anchor=`P-13:L48-L48, L52-L52` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L261): Missing source tag or VLM-sourced count FAILS.
- **P-001/P13-E016** [E] class=`E/FLOW` anchor=`P-13:L51-L51, L61-L61` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L262): “click element 7” resolves by id (harness geometry), never model coordinates; coordinate payload rejected.
- **P-001/P13-E017** [E] [SURFACE] class=`E/FAIL` anchor=`P-13:L55-L55` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L263): ABORT if source tag requires second perception scaffold or second provenance scale (A-1/N-2).
- **P-001/P13-E018** [E] [SURFACE] class=`E/TEST` anchor=`P-13:L56-L60` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L264): VERIFY via plan_gate + wiring_contract; proof off-agent row + zero VLM count trace via real `elai perception scaffold`.
- **P-001/P13-E019** [E] class=`E/TEST` anchor=`P-13:L63-L64` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L265): INTEGRATION_SMOKE joint with P-12 on live canvas/game empty AXTree; standalone scaffold fixture also.
- **P-001/P13-E020** [E] class=`E/SEC` anchor=`P-13:L61-L61` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L266): REDTEAM: vlm_caption as authoritative defeated by tag hierarchy; direct coordinates defeated by integer-only resolver.
- **P-001/P13-E021** [E] class=`E/EVENT` anchor=`P-13:L33-L33` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L267): Prefer-reuse J-1/J-8 rows net 0..+1 (P-0 map).
- **P-001/P13-C001** [C] [SURFACE] class=`C/SURFACE` anchor=`P-13:L46-L46, L71-L72` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L268): Detail documents CLI trigger `elai perception scaffold` as acceptance entrypoint while PRODUCTION_HOOK says no new entrypoint (extend gateway in place). Surface may be existing CLI wrapping gateway, but “add no CLI” summary language conflicts with this trigger.
- **P-001/P13-C002** [C] class=`C/DEFER` anchor=`P-13:L20-L20, L51-L51` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L269): Summary acceptance expects Tier-3 pixel leg fire once on empty-AXTree fixture; detail defers Tier-3 pixel/YOLO to Tier-2 with P-12.
- **P-001/P13-A001** [A] class=`A/DATA` anchor=`P-13:L19-L20` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L270): Source-tag precedence when multiple sources contribute to one element is undefined.
- **P-001/P13-A002** [A] class=`A/DATA` anchor=`P-13:L20-L20` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L271): “Asymmetric 80%-containment” OCR fusion rule details are Tier-2 and not specified.
- **P-001/P13-A003** [A] class=`A/CFG` anchor=`P-13:L20-L20` unit=`P-13` (ledger-line: DEEP-P-001.discovery.md:L272): “Provably incomplete” AXTree definition not operationalized.

### Unit `P-16` — gated install catalog — from P-001 (23 records)

- **P-001/P16-E001** [E] class=`E/STATUS` anchor=`P-16:L3-L5` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L282): P-16 PLANNED WAVE CU-3, EXTENSION of P-9+P-10+L-12, PROOF_SPINE none (distribution), owner-acked 2026-07-11; effort M; depends P-9,P-10.
- **P-001/P16-E002** [E] class=`E/CAP` anchor=`P-16:L6-L6` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L283): Adds install catalog so capability-on-demand feels instant, behind existing admission family; owner “install blender/ps on demand” done fail-closed.
- **P-001/P16-E003** [E] [SURFACE] class=`E/LIMIT` anchor=`P-16:L9-L9, L21-L21, L40-L40` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L284): Sanctioned form is P-9 HEADLESS code-pack wrapping app CLI/API (e.g. `blender --background --python`); GUI computer-use is last resort not mechanism.
- **P-001/P16-E004** [E] class=`E/DEP` anchor=`P-16:L11-L16, L32-L34` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L285): Consumes P-9 ConnectorPack, P-10 CredentialVault, L-12 SkillspectorGate, N-6 license_tag dimension, P-12 as GUI last resort.
- **P-001/P16-E005** [E] class=`E/DATA` anchor=`P-16:L19-L19, L30-L31` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L286): Tier-1: install catalog table rows `{tool, source, mirror, alias, license_tag}` + installPolicy admission on L-12 path.
- **P-001/P16-E006** [E] class=`E/DATA` anchor=`P-16:L31-L31` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L287): Catalog data file: `data/connector/install-catalog.toml`.
- **P-001/P16-E007** [E] [SURFACE] class=`E/SURFACE` anchor=`P-16:L19-L19, L36-L36, L46-L46, L71-L72` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L288): CLI: `elai connector install <tool>` resolves catalog row to P-9 headless code-pack.
- **P-001/P16-E008** [E] class=`E/DEFER` anchor=`P-16:L20-L20` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L289): Tier-2: hidden-until-verified registry release gate (openclaw S17) composed with M distribution; per-OS mirror selection.
- **P-001/P16-E009** [E] [SURFACE] class=`E/LIMIT` anchor=`P-16:L21-L21, L41-L41` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L290): Excluded: agent-zero ungated apt-get; openclaw advisory regex scanner; GUI-driving install when headless CLI exists.
- **P-001/P16-E010** [E] class=`E/CAP` anchor=`P-16:L30-L30` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L291): Module `crates/orchestrator/src/connector/install_catalog.rs` InstallCatalog + installPolicy.
- **P-001/P16-E011** [E] class=`E/LIMIT` anchor=`P-16:L38-L42` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L292): Stub ban: no install bypassing L-12; no second scanner; no secrets in catalog/journal (P-10 vault); no .unwrap; no blake2/sha2.
- **P-001/P16-E012** [E] [SURFACE] class=`E/SURFACE` anchor=`P-16:L43-L44` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L293): frontend_pair Tier-2: catalog + admission verdict in connector surface (W-4).
- **P-001/P16-E013** [E] class=`E/EVENT` anchor=`P-16:L47-L48` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L294): expect_ssb `install_catalog_admit`; row `{tool, source, license_tag, verdict}` per attempt.
- **P-001/P16-E014** [E] class=`E/FLOW` anchor=`P-16:L48-L48, L51-L51` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L295): Clean-license tool mounts P-9 headless pack with verdict=admit; installPolicy-flagged tool verdict=refuse and does NOT install.
- **P-001/P16-E015** [E] [SURFACE] class=`E/FAIL` anchor=`P-16:L48-L48` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L296): NEGATIVE: bypass L-12 or GUI install despite headless CLI FAILS.
- **P-001/P16-E016** [E] [SURFACE] class=`E/DEFER` anchor=`P-16:L53-L53` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L297): FORK: if tool has no headless CLI/API, route P-12 operator-lane stage (taint-tagged, never load-bearing), not refuse-or-drive-as-infra.
- **P-001/P16-E017** [E] class=`E/FAIL` anchor=`P-16:L55-L55` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L298): ABORT if catalog install could execute unadmitted code (ClawHub-no-gate failure mode).
- **P-001/P16-E018** [E] class=`E/SEC` anchor=`P-16:L61-L61` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L299): REDTEAM: slopsquat/typo defeated by license_tag + skillspector; arbitrary shell install patched because row resolves ONLY to P-9 pack never raw shell string.
- **P-001/P16-E019** [E] class=`E/TEST` anchor=`P-16:L64-L64` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L300): INTEGRATION_SMOKE: real blender install e2e + subsequent workflow drives blender --background; flagged/typo refused; stub catalog insufficient R10.
- **P-001/P16-E020** [E] [SURFACE] class=`E/TEST` anchor=`P-16:L56-L60` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L301): VERIFY: off-agent install_catalog_admit + flagged refusal via real CLI; plan_gate + wiring_contract.
- **P-001/P16-I001** [I] class=`I/DEP` anchor=`P-16:L5-L5, L12-L13` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L302): Because P-9/P-10 are out of this deep chunk but named dependencies, Tier-1 cannot execute until those packs/vault exist.
- **P-001/P16-A001** [A] class=`A/DATA` anchor=`P-16:L19-L19` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L303): Source/mirror integrity checks, catalog signing/update, alias collision rules, and license-policy vocabulary not specified.
- **P-001/P16-A002** [A] class=`A/CFG` anchor=`P-16:L19-L19, L51-L52` unit=`P-16` (ledger-line: DEEP-P-001.discovery.md:L304): installPolicy flag vocabulary and SkillspectorGate composition parameters not enumerated.

### Unit `P-18` — durable TaskSpec exec + approval drift-binding — from P-001 (27 records)

- **P-001/P18-E001** [E] class=`E/STATUS` anchor=`P-18:L3-L5` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L314): P-18 PLANNED WAVE OC-2, EXTENSION O-2+O-4+P-11+3A/4G, PROOF_SPINE none, owner-acked 2026-07-11; effort M-L; depends P-11; STRONG verified half, WEAK-until-RECON resume-token.
- **P-001/P18-E002** [E] class=`E/CAP` anchor=`P-18:L6-L6` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L315): Premise: add verified durable-execution mechanics + approval drift-binding anti-TOCTOU; carry lobster resume-token as RECON re-derived subprocess-checkpoint.
- **P-001/P18-E003** [E] class=`E/EVIDENCE` anchor=`P-18:L9-L9` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L316): openclaw Task Flow revision-counter VERIFIED-IN-SOURCE; exec-approval binds canonical systemRunPlan; lobster resume-token UNVERIFIED-external C1.
- **P-001/P18-E004** [E] class=`E/DEP` anchor=`P-18:L11-L16, L32-L34` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L317): Consumes O-2 TaskSpec run store, O-4 canvas visualization, P-11 confirm gate, 3A/4G ApprovalGate, Q-4 swap-restart, G blast-radius.
- **P-001/P18-E005** [E] class=`E/CAP` anchor=`P-18:L19-L19` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L318): Tier-1: (a) revision-counter optimistic concurrency on run store (stale expectedRevision → conflict, never clobber) + sticky persisted cancel surviving restart; (b) approval drift-binding — every human/autonomous approval binds canonical hashed run-plan (cwd/argv/env/pinned-executable) + operand fingerprints; re-validate at exec; ANY drift = deny.
- **P-001/P18-E006** [E] class=`E/DEFER` anchor=`P-18:L20-L20` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L319): Tier-2: retry-ladders-per-failure-class + watchdog/preflight; commitment due-time clamp (S19); subprocess-checkpoint resume model (C1 re-derivation).
- **P-001/P18-E007** [E] class=`E/LIMIT` anchor=`P-18:L21-L21` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L320): Excluded: claiming lobster “approve+resume never re-runs prior steps” as shipped openclaw fact.
- **P-001/P18-E008** [E] class=`E/CAP` anchor=`P-18:L30-L31` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L321): Produces RunStore (revision-counter + sticky-cancel) and ApprovalPlan (hashed plan + fingerprints; re-validate → deny).
- **P-001/P18-E009** [E] [SURFACE] class=`E/SURFACE` anchor=`P-18:L36-L36, L72-L73` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L322): CLI: `elai workflow resume\|cancel`.
- **P-001/P18-E010** [E] [SURFACE] class=`E/INT` anchor=`P-18:L36-L38` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L323): Production: CLI resume/cancel + FSM gates re-validating bound plan; test never R7.
- **P-001/P18-E011** [E] class=`E/LIMIT` anchor=`P-18:L39-L43, L70-L70` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L324): Stub ban: no LWW concurrent run write; no drifted plan execution; no commitment same-tick fire; no cooperative-only timeout as hard kill; no .unwrap; no blake2/sha2 (blake3 for plan hash).
- **P-001/P18-E012** [E] [SURFACE] class=`E/SURFACE` anchor=`P-18:L44-L45` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L325): frontend_pair: run-revision + drift-reject rows in O-4 workflow view (W-4).
- **P-001/P18-E013** [E] class=`E/EVENT` anchor=`P-18:L48-L49, L52-L52` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L326): expect_ssb includes `approval_drift_reject`; wargame also `taskspec_run_revision` conflict.
- **P-001/P18-E014** [E] class=`E/CONC` anchor=`P-18:L52-L52` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L327): Two concurrent writes one stale revision → one conflict, no clobber.
- **P-001/P18-E015** [E] class=`E/FAIL` anchor=`P-18:L52-L52` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L328): Operand swap after grant → approval_drift_reject; cancel survives simulated restart.
- **P-001/P18-E016** [E] class=`E/FAIL` anchor=`P-18:L49-L49` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L329): NEGATIVE: LWW clobber, drifted plan executes, or commitment fires in inference tick FAILS.
- **P-001/P18-E017** [E] class=`E/DEFER` anchor=`P-18:L54-L54` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L330): FORK C1: if real checkpoint cannot be built, ship Tier-1 revision+drift alone; do not ship cooperative-only resume imitation.
- **P-001/P18-E018** [E] class=`E/DEFER` anchor=`P-18:L55-L55` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L331): RECON_NEEDED: lobster-runner.ts + task-flow-registry.ts; prototype subprocess step-checkpoint before resume leg.
- **P-001/P18-E019** [E] class=`E/FAIL` anchor=`P-18:L56-L56` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L332): ABORT if approval could bind plan LOOSELY (partial argv/env/operand hash = TOCTOU hole).
- **P-001/P18-E020** [E] class=`E/TEST` anchor=`P-18:L57-L61` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L333): VERIFY off-agent approval_drift_reject + taskspec_run_revision conflict via real workflow/FSM entrypoints.
- **P-001/P18-E021** [E] class=`E/TEST` anchor=`P-18:L65-L65` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L334): INTEGRATION_SMOKE: multi-step through P-11 confirm; concurrent writers; mid-run restart sticky-cancel; post-approval operand swap; under 4G autonomous; toy single-write insufficient R10.
- **P-001/P18-E022** [E] class=`E/SEC` anchor=`P-18:L62-L62` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L335): REDTEAM: TOCTOU operand swap defeated by fingerprint revalidation; argv edit after approval defeated by full plan hash.
- **P-001/P18-E023** [E] class=`E/LIMIT` anchor=`P-18:L20-L20, L42-L42` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L336): Due-time clamp appears in stub_ban (≥1 tick after creation) even though Tier-2 deferral lists S19 clamp — load-bearing ban is stated now.
- **P-001/P18-C001** [C] class=`C/DEFER` anchor=`P-18:L20-L20, L42-L42; cross-ref P-0:L96-L100` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L337): Index and summary list retry ladders, watchdog, and due-time clamp as package features; detail places them in Tier-2 while still forbidding same-tick commitment in Tier-1 stub_ban.
- **P-001/P18-A001** [A] class=`A/CFG` anchor=`P-18:L20-L20` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L338): Retry classes, backoff, watchdog deadlines, checkpoint granularity undefined (Tier-2).
- **P-001/P18-A002** [A] class=`A/DATA` anchor=`P-18:L19-L19, L70-L70` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L339): Canonical serialization and operand-fingerprint algorithm details unspecified beyond blake3 plan hash.
- **P-001/P18-A003** [A] class=`A/INT` anchor=`P-18:L5-L5, L14-L14` unit=`P-18` (ledger-line: DEEP-P-001.discovery.md:L340): Exact P-11 “MOVE 3” confirm seam is outside this chunk; detail depends on it.

### Unit `P-21` — gate credibility hardening — from P-001 (25 records)

- **P-001/P21-E001** [E] class=`E/STATUS` anchor=`P-21:L3-L5` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L350): P-21 PLANNED WAVE OC-1, EXTENSION Q-6 + wiring R10 + G-9 + M, PROOF_SPINE feeds PS-6, STRONG grounding V1/V2, effort M.
- **P-001/P21-E002** [E] class=`E/CAP` anchor=`P-21:L6-L6` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L351): Steal verify-the-verifier canaries + structural redaction as reference implementations hardening gate corpus / PS-6.
- **P-001/P21-E003** [E] class=`E/EVIDENCE` anchor=`P-21:L9-L9` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L352): V1: known-BAD canaries through SAME production detectors; missed canary fails WHOLE gate exitCode=1. V2: durable audit type has NO content field (structural redaction). N1: missing-artifact=FAIL with in-manifest escape. N2: skipBackfill sibling-pass. C3: copy unknown=FAIL not classified-failure-pass. V3: keep metadata export SEPARATE BY TYPE from content debug bundle.
- **P-001/P21-E004** [E] class=`E/DEP` anchor=`P-21:L11-L15, L31-L33` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L353): Extends Q-6 EvidenceCorruptionCorpus; composes R10; hardens G-9; M is metadata-only projection consumer.
- **P-001/P21-E005** [E] class=`E/CAP` anchor=`P-21:L18-L18` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L354): Tier-1: (a) seeded negative-control canaries into Q-6 (missed canary hard-BLOCKS whole gate exit 1); (b) missing-artifact=FAIL with in-manifest explicit escape (env-blocked \| optional-gap only) — N1 ≈ WIRE-DARK.
- **P-001/P21-E006** [E] class=`E/DEFER` anchor=`P-21:L19-L19` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L355): Tier-2: (c) skipBackfill cross-lane into R10; (d) STRUCTURAL redaction closed metadata-only audit-export into G-9 separate BY TYPE from debug bundle; (e) posture labels (user-path \| direct-gateway \| native-approval) on R10 smoke evidence.
- **P-001/P21-E007** [E] class=`E/LIMIT` anchor=`P-21:L20-L20` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L356): Excluded: openclaw green-report default letting classified failure pass; conflating metadata audit with content trajectory bundle.
- **P-001/P21-E008** [E] class=`E/CAP` anchor=`P-21:L29-L30` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L357): Produces verifier_canary.rs and metadata_export.rs (closed metadata-only struct, hash-don’t-store ids).
- **P-001/P21-E009** [E] class=`E/INT` anchor=`P-21:L34-L36, L70-L71` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L358): Production: gate_corpus runner invoking canary lane; test never R7.
- **P-001/P21-E010** [E] class=`E/LIMIT` anchor=`P-21:L37-L41` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L359): Stub ban: no missed canary still pass; no classified failure green by default; no content field on metadata export; no missing artifact as pass without manifest escape; no .unwrap; no blake2/sha2.
- **P-001/P21-E011** [E] [SURFACE] class=`E/SURFACE` anchor=`P-21:L42-L43` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L360): frontend_pair: rides existing gate/evidence surface (corpus kill-rate + posture label; W-4).
- **P-001/P21-E012** [E] [SURFACE] class=`E/SURFACE` anchor=`P-21:L45-L45` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L361): CLI trigger: `elai bench gate-corpus --with-canaries --break-detector redaction`.
- **P-001/P21-E013** [E] class=`E/EVENT` anchor=`P-21:L46-L46` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L362): expect_ssb `gate_credibility_measured` (prefer-reuse Q-6 posture/canary field net 0..+1 per P-0).
- **P-001/P21-E014** [E] class=`E/TEST` anchor=`P-21:L47-L47, L50-L50` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L363): Broken canary detector → self-test detects miss → WHOLE gate fail exit 1; smoke without posture label FAILS; metadata-export has no content field.
- **P-001/P21-E015** [E] class=`E/FAIL` anchor=`P-21:L47-L47` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L364): NEGATIVE: missed canary still passes; classified failure green by default; content in metadata-only export FAILS.
- **P-001/P21-E016** [E] class=`E/DEFER` anchor=`P-21:L52-L52` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L365): FORK: env-absent required artifact uses manifest env-blocked escape only; undeclared absence still FAIL; declared env-block → WARN.
- **P-001/P21-E017** [E] class=`E/FAIL` anchor=`P-21:L54-L54` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L366): ABORT if metadata export and content debug bundle share a type or serializer (V3 trap).
- **P-001/P21-E018** [E] class=`E/TEST` anchor=`P-21:L55-L59` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L367): VERIFY off-agent gate_credibility_measured + exit 1 on broken canary via real bench entrypoint.
- **P-001/P21-E019** [E] class=`E/TEST` anchor=`P-21:L63-L63` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L368): INTEGRATION_SMOKE: whole frozen gate manifest; each checker seeded canary + broken-detector arm; every smoke posture-labeled; no export path carries content; single-gate canary insufficient R10.
- **P-001/P21-E020** [E] class=`E/SEC` anchor=`P-21:L60-L60` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L369): REDTEAM: silent detector rot caught by canary; audit-export exfil patched by closed metadata-only type.
- **P-001/P21-E021** [E] class=`E/INT` anchor=`P-21:L70-L71` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L370): PRODUCTION_HOOK also wires R10 smoke posture label and G-9 closed metadata-export in “this changeset” despite Tier-2 deferral of (d)(e) — see conflict.
- **P-001/P21-C001** [C] class=`C/DEFER` anchor=`P-21:L18-L19, L29-L30, L37-L41, L47-L47, L70-L71` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L371): Revised scope places skipBackfill, structural redaction, and posture labels in Tier-2, but WIRING_CONTRACT produces metadata_export.rs, stub_ban forbids content field and missing posture, observable_effect fails smoke without posture label, and PRODUCTION_HOOK claims all wired in this changeset. Tier boundary is internally inconsistent.
- **P-001/P21-A001** [A] class=`A/CFG` anchor=`P-21:L18-L18` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L372): Canary corpus contents, seeding cadence, detector coverage, and required kill-rate not enumerated.
- **P-001/P21-A002** [A] class=`A/CFG` anchor=`P-21:L18-L18, L52-L52` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L373): optional-gap and env-blocked admission criteria named not defined.
- **P-001/P21-A003** [A] class=`A/SEC` anchor=`P-21:L19-L19` unit=`P-21` (ledger-line: DEEP-P-001.discovery.md:L374): Debug-bundle custody/access policy outside this unit.

## Ledger P-002

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-P-002.discovery.md`

### Unit `P-1` — app-fleet layer (health-tick + gated auto-repair) [KEYSTONE] — from P-002 (35 records)

- **P-002/P1-E001** [E] class=`E/STATUS` anchor=`P-1:L1-L5` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L37): P-1 is PLANNED WAVE 1 KEYSTONE, product-surface/distribution class with no PROOF_SPINE, owner-acked 2026-07-07, BUILD-on-shipped, Track PERSONAL-INCARNATION, lever `fleet-grain-over-taskspec`, GAP-1 / T2 keep-it-alive, MODERATE index-derived PLAN-asserted grounding, ~80% composition, effort M-L.
- **P-002/P1-E002** [E] class=`E/CAP` anchor=`P-1:L6-L7` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L38): The app is the unit of care one grain above changeset/task: each of the user's N bespoke apps becomes a first-class object that is journaled, scheduled, and receipted.
- **P-002/P1-E003** [E] class=`E/DATA` anchor=`P-1:L9-L10, L31-L32` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L39): `AppManifest` closed field set is `{ app_id, one_sentence_purpose, data_namespaces: [ns], ports (P-2 typed input/output schemas), smoke_trigger, rra_lineage, dep_manifest }`.
- **P-002/P1-E004** [E] class=`E/DATA` anchor=`P-1:L10-L10, L22-L22, L31-L32` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L40): `FleetRegistry` is a SQLite rows-as-data store (like rules engine / O-2 TaskSpec) enumerating every ELAI-built app.
- **P-002/P1-E005** [E] class=`E/FLOW` anchor=`P-1:L10-L11, L14-L14, L63-L64` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L41): A health tick is an O-2 TaskSpec with `trigger=Cron`, `pipeline=[Smoke, DriftCanary, DepRotCheck]`, `sink=DashboardInbox` — not a second scheduler.
- **P-002/P1-E006** [E] class=`E/DEP` anchor=`P-1:L10-L11, L15-L15` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L42): Per-app drift canaries GENERALIZE G-10 from ELAI's own canary set to each app's declared smoke set (the model-update-breaks-use-case gap).
- **P-002/P1-E007** [E] class=`E/DEP` anchor=`P-1:L11-L11, L16-L16` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L43): Dependency-rot watching points F-9's existence/age gate at each app's `dep_manifest` on a TTL; F-5 is staleness quarantine also pointed at a manifest.
- **P-002/P1-E008** [E] [SURFACE] class=`E/SEC` anchor=`P-1:L11-L11, L18-L18, L49-L50, L64-L65` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L44): When a tick goes red, ELAI proposes a fix that MUST pass the same wiring/claim gates as any changeset (never a silent patch), rendered human-first via N-7, one-click apply — gated AUTO-REPAIR.
- **P-002/P1-E009** [E] class=`E/LIMIT` anchor=`P-1:L11-L11, L63-L67` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L45): EXCLUDED: a second scheduler; auto-repair that bypasses wiring/claim gates (A-repair-must-gate); a per-app code fork of the health machinery (manifest is DATA, A-5 two-masters); silent rollback (every repair/rollback is journaled + N-7-rendered).
- **P-002/P1-E010** [E] [SURFACE] class=`E/DEP` anchor=`P-1:L17-L17` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L46): L-14 `elai build "<idea>"` emits an RRA receipt; the manifest's `rra_lineage` is that receipt's id; the fleet registers what L-14 builds.
- **P-002/P1-E011** [E] [SURFACE] class=`E/DEP` anchor=`P-1:L19-L19` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L47): M-4's signed-updater manifest/rollback pattern is the shape; M-4 updates ELAI itself while P-1 applies the pattern per-user-app (GAP-1's distinction).
- **P-002/P1-E012** [E] [SURFACE] class=`E/CFG` anchor=`P-1:L21-L22` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L48): SHIP NOW (Tier-1): AppManifest schema + FleetRegistry SQLite + `elai fleet register <app>` CLI + health-tick TaskSpec template with Smoke + DepRotCheck stages (reusing O-2 + F-9) + red-tick DashboardInbox card via N-7.
- **P-002/P1-E013** [E] [SURFACE] class=`E/DEFER` anchor=`P-1:L22-L23, L41-L42` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L49): DEFER (Tier-2): per-app drift canaries (needs G-10 generalized — asserted-not-designed per A-6) + gated auto-repair apply-button (changeset/claim-gate seam).
- **P-002/P1-E014** [E] class=`E/INT` anchor=`P-1:L25-L26` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L50): Integration: crates/orchestrator (or rules-engine) owns AppManifest table + FleetRegistry loader + health-tick TaskSpec factory + red-tick SSB emit.
- **P-002/P1-E015** [E] [SURFACE] class=`E/SURFACE` anchor=`P-1:L27-L27, L54-L55, L59-L60, L73-L73` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L51): CLI surface: `elai fleet register\|list\|tick <app>` subcommand (cli/main.rs) is the Tier-1 production caller.
- **P-002/P1-E016** [E] [SURFACE] class=`E/SURFACE` anchor=`P-1:L28-L28` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L52): Frontend: DashboardInbox (O-2 intervention-card sink) renders the red-tick card; a dedicated FleetPanel is deferred to P-5's operator surface.
- **P-002/P1-E017** [E] class=`E/SEC` anchor=`P-1:L31-L33, L58-L58` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L53): M1: strict deserializer validates `data_namespaces` against O-3 vault namespace-claims; a manifest naming an unclaimed namespace is a hard refusal.
- **P-002/P1-E018** [E] class=`E/EVENT` anchor=`P-1:L34-L36, L67-L67, L69-L71` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L54): M2: each tick journals a `fleet_health_tick` SSB row carrying `{app_id, verdict, red_reasons}`; broken app lands `verdict=red` with populated `red_reasons`.
- **P-002/P1-E019** [E] class=`E/IDEM` anchor=`P-1:L34-L36, L69-L70` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L55): At the due tick each app's smoke fires exactly once; one `fleet_health_tick` row per due tick.
- **P-002/P1-E020** [E] [SURFACE] class=`E/SURFACE` anchor=`P-1:L37-L39, L68-L71` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L56): M3: red tick renders one N-7 DashboardInbox card (plain-language diagnosis: which app, what broke, one sentence); green tick auto-archives and leaves only the SSB row, no card.
- **P-002/P1-E021** [E] class=`E/FAIL` anchor=`P-1:L33-L33, L36-L36, L39-L39` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L57): ON_FAIL arms: unclaimed-namespace register succeeds (privacy bypass); tick without SSB row (R1 missing evidence); green ticks spam inbox (missing auto-archive-on-green keyed off `verdict`).
- **P-002/P1-E022** [E] class=`E/CFG` anchor=`P-1:L41-L42` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L58): FORK: if G-10 canary abstraction is NOT extractable per-app without restructuring, ship Tier-1 Smoke+DepRotCheck only, route drift canary to Tier-2, record G-10-generalization seam; do NOT re-implement a second canary engine.
- **P-002/P1-E023** [E] [SURFACE] class=`E/LIMIT` anchor=`P-1:L49-L50` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L59): ABORT: if gated auto-repair would apply without wiring_contract_check / claim gates, stop and flag; ship diagnosis + propose-fix render only; apply-button lands only once it routes through the real gate seam.
- **P-002/P1-E024** [E] [SURFACE] class=`E/INT` anchor=`P-1:L59-L62, L73-L73` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L60): Production reachability: `elai fleet register` on a live run AND task_runner interval fires a due health-tick TaskSpec; a `#[cfg(test)]` caller NEVER satisfies R7.
- **P-002/P1-E025** [E] [SURFACE] class=`E/TEST` anchor=`P-1:L75-L79` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L61): VERIFY: wiring_contract_check + production_caller_check; pass = `fleet_health_tick` row landed off-agent from real task_runner tick (or `elai fleet tick`), fired exactly once, red-tick inbox card for broken-app, auto-archive for green.
- **P-002/P1-E026** [E] class=`E/TEST` anchor=`P-1:L81-L81` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L62): INTEGRATION_SMOKE R10: register TWO apps (healthy + broken after simulated dep-rot), advance several ticks, assert single-fire journaling per app + red N-7 card + green auto-archive + no cross-app leakage of red_reasons.
- **P-002/P1-E027** [E] class=`E/SEC` anchor=`P-1:L83-L84` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L63): REDTEAM failed attack: silent red-tick fix is blocked by stub_ban + ABORT (repair is a changeset). Succeeded attack: register 200 apps storming task_runner — patched by ONE batched TaskSpec iterating the registry (one row per app, one dispatch per tick).
- **P-002/P1-E028** [E] class=`E/EVIDENCE` anchor=`P-1:L86-L90` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L64): Grounding: design digest GAP-1/T2/A-2 (falsifier: fleet-health green %, median time-to-repair) + A-6 G-10 recon; reddit bespoke-tools thread; Ink & Switch malleable-software essay.
- **P-002/P1-E029** [E] class=`E/DEP` anchor=`P-1:L92-L93` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L65): COMPOSES-WITH: O-2 TaskSpec + DashboardInbox (EXTEND), G-10 (generalize), F-9/F-5, L-14 RRA, N-7, O-3 namespace-claims. Re-implements NONE (GD-1). P-2 supplies typed ports; P-5 renders fleet operator grain; P-6 validates namespaces against privacy lattice.
- **P-002/P1-A001** [A] [SURFACE] class=`A/DEP` anchor=`P-1:L44-L45` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L66): Whether G-10's canary set is parameterizable by an app's smoke set or hard-bound to ELAI fixtures is RECON_NEEDED before Tier-1 drift-canary scope.
- **P-002/P1-A002** [A] class=`A/INT` anchor=`P-1:L46-L46` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L67): O-2 TaskSpec factory API (programmatic create vs row-authored) is RECON_NEEDED.
- **P-002/P1-A003** [A] class=`A/INT` anchor=`P-1:L47-L47` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L68): F-9 dep existence/age checker entrypoint (manifest path vs Cargo.lock) is RECON_NEEDED.
- **P-002/P1-A004** [A] class=`A/CFG` anchor=`P-1:L10-L11, L21-L22` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L69): Exact TTL for F-9 dep_manifest watching and Cron schedule defaults for the health-tick template are not specified.
- **P-002/P1-C001** [C] class=`C/DEFER` anchor=`P-1:L10-L11, L21-L23, L41-L42` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L70): Opening steal/build names DriftCanary in the pipeline, but REVISED Tier-1 ships only Smoke + DepRotCheck; drift canary is Tier-2 behind G-10 generalization.
- **P-002/P1-C002** [C] [SURFACE] class=`C/DEFER` anchor=`P-1:L11-L11, L21-L23, L49-L50` unit=`P-1` (ledger-line: DEEP-P-002.discovery.md:L71): Opening prose includes one-click gated auto-repair, but Tier-1 ships diagnosis + propose-fix only; apply-button is Tier-2 + ABORT-gated.

### Unit `P-2` — inter-tool ports + world-grain event bus — from P-002 (29 records)

- **P-002/P2-E001** [E] class=`E/STATUS` anchor=`P-2:L1-L5` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L81): P-2 is PLANNED WAVE 2, product-surface, no PROOF_SPINE, owner-acked 2026-07-07, BUILD-on-shipped mostly composition; port-schema registry is the genuine build; Track PERSONAL-INCARNATION; lever `typed-ports-over-canvas`; GAP-2 / T3 glue; MODERATE grounding; effort M; depends P-1 + O-4.
- **P-002/P2-E002** [E] class=`E/CAP` anchor=`P-2:L6-L10` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L82): Every ELAI-built tool ships typed PORTS in its P-1 manifest; canvas wires tool->tool edges carrying N-6 provenance tier + G-1 taint; the bus is typed-edge projection + journaled cross-tool invocation SSB rows, not a runtime.
- **P-002/P2-E003** [E] class=`E/DATA` anchor=`P-2:L10-L10, L30-L31` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L83): Extend P-1 AppManifest with `ports: { inputs: [PortSchema], outputs: [PortSchema] }` where `PortSchema = { name, json_schema, domain_label (life\|work\|shared, P-6) }`.
- **P-002/P2-E004** [E] class=`E/DATA` anchor=`P-2:L10-L10, L30-L31, L52-L53` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L84): `PortRegistry` indexes every app's ports for wiring.
- **P-002/P2-E005** [E] class=`E/FLOW` anchor=`P-2:L10-L10, L33-L35` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L85): On O-4 canvas, a tool->tool EDGE binds an output port to a compatible input port; edge carries N-6 provenance tier of the source + G-1 taint of flowing bytes.
- **P-002/P2-E006** [E] [SURFACE] class=`E/SURFACE` anchor=`P-2:L10-L10, L36-L38` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L86): An edge whose source port has NO consumer renders VISIBLY DISCONNECTED (O-4 wire-dark-made-visual at world grain).
- **P-002/P2-E007** [E] class=`E/SEC` anchor=`P-2:L10-L10, L21-L22, L82-L83` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L87): elai-verify (M-5) conformance to PortSchema makes a third-party tool a mountable node (Tier-2, with L-12 admission).
- **P-002/P2-E008** [E] class=`E/LIMIT` anchor=`P-2:L11-L11, L48-L49, L62-L66` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L88): EXCLUDED: live message-broker/runtime bus (A-3; dispatch is the FSM); second provenance scale (ride N-6 — A-1); untyped/free-text port (closed json_schema, `deny_unknown`); edge carrying bytes WITHOUT source taint tier.
- **P-002/P2-E009** [E] [SURFACE] class=`E/CFG` anchor=`P-2:L20-L21` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L89): SHIP NOW (Tier-1): PortSchema extension + PortRegistry + static port-compatibility check (`elai fleet wire <appA.out> <appB.in>`) refusing incompatible schemas + journaled `tool_port_edge` SSB row + G-1 taint tier on the edge record.
- **P-002/P2-E010** [E] class=`E/DEFER` anchor=`P-2:L21-L22, L27-L27` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L90): DEFER (Tier-2): O-4 canvas WORKFLOW-mode rendering of tool->tool edges + third-party mount via M-5 elai-verify (L-12 admission seam shared with P-3).
- **P-002/P2-E011** [E] [SURFACE] class=`E/SURFACE` anchor=`P-2:L26-L26, L36-L38, L53-L54, L68-L70` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L91): CLI: `elai fleet wire\|ports <app>` is the Tier-1 production caller; Tier-1 exposes zero-consumer outputs via `elai fleet ports --unwired`.
- **P-002/P2-E012** [E] [SURFACE] class=`E/SURFACE` anchor=`P-2:L27-L27, L59-L60` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L92): frontend_pair Tier-2: dashboard-v2 O-4 WORKFLOW view renders tool->tool edges with N-6 tier chrome + disconnected-source affordance.
- **P-002/P2-E013** [E] class=`E/FAIL` anchor=`P-2:L30-L32` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L93): Malformed port schema is hard parse-refusal; untyped port acceptance is ON_FAIL (interchange-format invariant).
- **P-002/P2-E014** [E] class=`E/EVENT` anchor=`P-2:L33-L35, L67-L70` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L94): Successful bind emits `tool_port_edge` SSB row `{src_app.out, dst_app.in, n6_tier, taint_tier}`; schema-incompatible wire lands NO edge row and returns a refusal.
- **P-002/P2-E015** [E] class=`E/SEC` anchor=`P-2:L33-L35, L65-L66, L82-L83` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L95): Taint-tier propagation: source port's G-1 taint tier is forced onto every edge record; dropped tier is a smoke failure.
- **P-002/P2-E016** [E] [SURFACE] class=`E/CFG` anchor=`P-2:L40-L41` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L96): FORK: if O-4 edge model is hard-bound to research-pipeline stage enum and cannot carry tool->tool edges without restructuring, ship Tier-1 CLI fully and route ALL canvas rendering to Tier-2; trust value is realized at CLI grain without canvas.
- **P-002/P2-E017** [E] class=`E/LIMIT` anchor=`P-2:L48-L49` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L97): ABORT: if wiring the world-bus would require a LIVE message-broker executing edges outside the FSM, stop; bus is projection + journaled rows, never a runtime.
- **P-002/P2-E018** [E] [SURFACE] class=`E/INT` anchor=`P-2:L58-L61, L72-L72` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L98): Production reachability: `elai fleet wire` binds on a live run; Tier-2 O-4 WORKFLOW canvas renders/creates the edge; test callers never satisfy R7.
- **P-002/P2-E019** [E] [SURFACE] class=`E/TEST` anchor=`P-2:L74-L78` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L99): VERIFY: wiring_contract_check + production_caller_check; pass = `tool_port_edge` row from real `elai fleet wire` carrying N-6 + G-1 taint, incompatible wire refused with no edge row.
- **P-002/P2-E020** [E] class=`E/TEST` anchor=`P-2:L80-L80` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L100): INTEGRATION_SMOKE R10: three apps (weather source, wardrobe consumer, budget unrelated); wire weather->wardrobe; refuse weather->budget; assert taint tier + `--unwired` unrelated output + G-2 replay of wardrobe reproduces weather bytes.
- **P-002/P2-E021** [E] class=`E/SEC` anchor=`P-2:L82-L83` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L101): REDTEAM failed: launder untrusted weather bytes by dropping taint tier — stub_ban forces propagation. Succeeded attack: malicious third-party tool declaring conformant schema — patched by Tier-2 M-5 elai-verify + L-12 admission (schema-declaration alone never admits).
- **P-002/P2-E022** [E] class=`E/EVIDENCE` anchor=`P-2:L85-L88` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L102): Grounding: design digest GAP-2/T3/A-1/A-2 (falsifier: wired-edge count, disconnected-output rate); reddit glue quotes; feature-map O-4, N-6, G-1/G-2, M-5/M-6.
- **P-002/P2-E023** [E] class=`E/DEP` anchor=`P-2:L90-L91` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L103): COMPOSES-WITH: P-1 AppManifest (EXTEND ports), O-4 WORKFLOW (EXTEND edges), N-6, G-1/G-2, M-5/M-6. P-6 supplies `domain_label`; P-3 export bundles ports.
- **P-002/P2-A001** [A] class=`A/INT` anchor=`P-2:L43-L44` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L104): O-4 edge data model (typed stage->stage vs generic node pair) is RECON_NEEDED.
- **P-002/P2-A002** [A] class=`A/INT` anchor=`P-2:L45-L45` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L105): G-1 taint API for tagging/reading a byte-stream tier is RECON_NEEDED.
- **P-002/P2-A003** [A] class=`A/INT` anchor=`P-2:L46-L46` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L106): Whether N-6 provenance tier attaches to arbitrary records or only claim objects is RECON_NEEDED.
- **P-002/P2-A004** [A] class=`A/DATA` anchor=`P-2:L20-L21, L33-L35` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L107): Compatibility relation details and schema-version behavior for PortSchema are not fully specified beyond refuse-on-mismatch.
- **P-002/P2-A005** [A] class=`A/EVENT` anchor=`P-2:L67-L70` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L108): Source does not require a separate refusal/audit SSB row for incompatible wire; only "NO edge row + refusal return".
- **P-002/P2-C001** [C] [SURFACE] class=`C/DEFER` anchor=`P-2:L10-L10, L20-L22` unit=`P-2` (ledger-line: DEEP-P-002.discovery.md:L109): Steal/build names O-4 canvas edge rendering and third-party mount as core, but REVISED scope makes both Tier-2; Tier-1 is CLI registry/wire/SSB only.

### Unit `P-6` — life-data privacy lattice — from P-002 (25 records)

- **P-002/P6-E001** [E] class=`E/STATUS` anchor=`P-6:L1-L5` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L119): P-6 is PLANNED WAVE 2, product-surface, no PROOF_SPINE, owner-acked 2026-07-07, BUILD-on-shipped extending N-6 with a privacy dimension (no second lattice); Track PERSONAL-INCARNATION; lever `out-flow-privacy-dimension`; GAP-6 / T6; MODERATE grounding; composition; effort M; depends N-6 + O-3.
- **P-002/P6-E002** [E] class=`E/PRIV` anchor=`P-6:L6-L10` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L120): Third privacy axis: WHICH TOOL may read WHICH NAMESPACE; domain label life/work/shared + per-tool read grants; G-1 taint's MIRROR IMAGE (owned bytes flowing OUT toward models/tools vs untrusted bytes flowing IN).
- **P-002/P6-E003** [E] class=`E/DATA` anchor=`P-6:L9-L10, L30-L31` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L121): Extend N-6 provenance-lattice mapping table with `domain: life\|work\|shared` per namespace and `read_grants: [{tool_id, namespace, granted_at, journal_ref}]`.
- **P-002/P6-E004** [E] class=`E/SEC` anchor=`P-6:L10-L10, L20-L22` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L122): Enforcement at TWO boundaries: (a) O-3 vault read path — ungranted tool refused; (b) E-20 routing — owned byte's domain gates model/tool prompt entry.
- **P-002/P6-E005** [E] class=`E/PRIV` anchor=`P-6:L10-L11, L62-L65` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L123): Default is deny (no grant = no read); a grant is an explicit, journaled, revertible act (H-49).
- **P-002/P6-E006** [E] class=`E/LIMIT` anchor=`P-6:L11-L11, L48-L49, L62-L66` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L124): EXCLUDED: SECOND privacy lattice (A-1/N-2 authoring FAIL); implicit/default-allow read; grant that isn't journaled+revertible; mixing OUT-flow privacy dimension with IN-flow taint into one confused field.
- **P-002/P6-E007** [E] class=`E/CFG` anchor=`P-6:L20-L21` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L125): SHIP NOW (Tier-1): domain dimension on N-6 mapping + read_grants table + O-3 vault-read enforcement + journaled `privacy_grant` / `privacy_read_refusal` SSB rows + N-7 render of each grant.
- **P-002/P6-E008** [E] [SURFACE] class=`E/DEFER` anchor=`P-6:L21-L22, L27-L27` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L126): DEFER (Tier-2): E-20 routing-boundary enforcement (lands with P-4 living-brain routing wall) + grant-management operator surface (frontend with P-5).
- **P-002/P6-E009** [E] class=`E/INT` anchor=`P-6:L24-L25` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L127): Integration: crates/knowledge (or N-6 home) owns domain + read_grants + vault-read enforcement hook + SSB emit.
- **P-002/P6-E010** [E] [SURFACE] class=`E/SURFACE` anchor=`P-6:L26-L26, L54-L55, L58-L59` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L128): CLI: `elai privacy grant\|revoke\|show <tool> <namespace>` is the Tier-1 production caller.
- **P-002/P6-E011** [E] class=`E/EVENT` anchor=`P-6:L33-L38, L67-L70` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L129): Granting emits `privacy_grant` row + N-7 human-rendered line; ungranted vault read lands `privacy_read_refusal` row.
- **P-002/P6-E012** [E] [SURFACE] class=`E/RECOVERY` anchor=`P-6:L36-L38` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L130): `elai privacy revoke` is the inverse journaled op; tool loses read access after revoke.
- **P-002/P6-E013** [E] class=`E/CFG` anchor=`P-6:L40-L41` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L131): FORK: if N-6 mapping cannot take a new dimension without N-4 claim-object schema migration, ship Tier-1 as a JOIN table keyed on N-6 namespace id (still one lattice / N-2-clean), record the seam.
- **P-002/P6-E014** [E] class=`E/LIMIT` anchor=`P-6:L48-L49` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L132): ABORT: if adding privacy requires minting a SECOND provenance/privacy lattice, stop and flag; corpus has exactly one lattice by doctrine.
- **P-002/P6-E015** [E] [SURFACE] class=`E/INT` anchor=`P-6:L58-L61, L72-L72` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L133): Production reachability: `elai privacy grant\|revoke` on live run AND O-3 vault-read path enforces grants on live read; test callers never satisfy R7.
- **P-002/P6-E016** [E] class=`E/TEST` anchor=`P-6:L74-L79` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L134): VERIFY: wiring_contract_check + production_caller_check + corpus_composition_check (N-2 must pass — no second lattice); pass = ungranted vault read refused with `privacy_read_refusal`, granted read succeeded, grant journaled+revertible, N-2 confirms single lattice.
- **P-002/P6-E017** [E] class=`E/TEST` anchor=`P-6:L81-L81` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L135): INTEGRATION_SMOKE R10: label three namespaces (budget=life, project=work, notes=shared), grant one tool on one namespace, cross-namespace multi-tool reads + revoke mid-run, assert every ungranted read refuses + grant/revoke journal + no owned byte to ungranted tool.
- **P-002/P6-E018** [E] class=`E/SEC` anchor=`P-6:L83-L84` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L136): REDTEAM failed: relabel health namespace shared to read from budget tool — relabel is journaled N-6 change and still needs explicit grant. Succeeded attack: second privacy scale as "helper table" — patched by N-2 corpus_composition_check at authoring.
- **P-002/P6-E019** [E] class=`E/EVIDENCE` anchor=`P-6:L86-L89` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L137): Grounding: design digest GAP-6/T6/A-1; reddit Azko87/Upbeat-Armadillo quotes; feature-map N-6/N-2/G-1/G-9/E-20/O-3/N-7/H-49.
- **P-002/P6-E020** [E] class=`E/DEP` anchor=`P-6:L91-L92` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L138): COMPOSES-WITH: N-6 (EXTEND privacy dimension), N-2, G-1 (mirror), G-9, E-20, O-3, N-7, H-49. O-3 owns vault boundary; P-4 is same E-20 enforcement point (Tier-2 shared).
- **P-002/P6-A001** [A] class=`A/INT` anchor=`P-6:L43-L44` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L139): N-6 mapping-table schema and whether a new dimension touches N-4 claim objects is RECON_NEEDED.
- **P-002/P6-A002** [A] class=`A/INT` anchor=`P-6:L45-L45` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L140): O-3 vault-read entrypoint for grant-check insertion is RECON_NEEDED.
- **P-002/P6-A003** [A] class=`A/INT` anchor=`P-6:L46-L46` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L141): E-20 routing boundary for domain-gated prompt entry (Tier-2) is RECON_NEEDED.
- **P-002/P6-A004** [A] class=`A/DATA` anchor=`P-6:L9-L10, L33-L35` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L142): Grant scope beyond the example (expiry, nested namespaces, cross-domain precedence) is not specified.
- **P-002/P6-C001** [C] class=`C/DEFER` anchor=`P-6:L10-L10, L20-L22` unit=`P-6` (ledger-line: DEEP-P-002.discovery.md:L143): Steal/build names dual enforcement at O-3 AND E-20, but REVISED Tier-1 ships only O-3 vault-read; E-20 is Tier-2.

### Unit `P-7` — per-user operating profile — from P-002 (25 records)

- **P-002/P7-E001** [E] class=`E/STATUS` anchor=`P-7:L1-L5` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L153): P-7 is PLANNED WAVE 3, product-surface, no PROOF_SPINE, owner-acked 2026-07-07, BUILD-on-shipped riding anti-selection-bias promotion; Track PERSONAL-INCARNATION; lever `measured-asymmetric-process-adaptation`; GAP-7; MODERATE grounding; composition on N-3/H-17/H-18; effort M-L; depends N-3 landed.
- **P-002/P7-E002** [E] class=`E/CAP` anchor=`P-7:L6-L7, L11-L11, L17-L17, L63-L64` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L154): Process adaptation (autonomy, gate strictness, explanation verbosity per DOMAIN per user) is disjoint from E-5/E-4 CONTENT adaptation (voice/terseness).
- **P-002/P7-E003** [E] class=`E/DATA` anchor=`P-7:L9-L10, L20-L21` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L155): `OperatingProfile { per_domain: { autonomy, gate_strictness, explanation_verbosity } }` learned from journaled outcomes and promoted through H-17 trial ledger + H-18 deflated significance (what N-3 generalizes).
- **P-002/P7-E004** [E] class=`E/SEC` anchor=`P-7:L10-L10, L15-L15, L29-L32, L49-L49, L60-L61` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L156): ASYMMETRY invariant (M-2 at process grain): profile may TIGHTEN gates freely; may LOOSEN ONLY against measured false-positive evidence from THAT user's own runs — defense against adaptation-as-sycophancy (A-4).
- **P-002/P7-E005** [E] class=`E/LIMIT` anchor=`P-7:L11-L11, L60-L64` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L157): EXCLUDED: loosening driven by anything but measured own-run FP evidence; unmeasured/vibe promotion; profile change that isn't observe-first + journaled + revertible; adapting CONTENT (E-5's job).
- **P-002/P7-E006** [E] class=`E/CFG` anchor=`P-7:L20-L21` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L158): SHIP NOW (Tier-1): OperatingProfile type + three default profiles (toy/connected/team) + ASYMMETRY enforcement + journaled `operating_profile_change` SSB row + observe-first + revert.
- **P-002/P7-E007** [E] class=`E/DEFER` anchor=`P-7:L21-L22, L40-L41` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L159): DEFER (Tier-2): automatic per-domain learning loop from journaled outcomes (full N-3 pipeline once N-3 live and FP-rate falsifier measured) + per-domain granularity beyond the three defaults.
- **P-002/P7-E008** [E] class=`E/CFG` anchor=`P-7:L6-L7, L36-L38` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L160): Three defaults on ONE engine: toy = P-5 ceremony floor; connected = P-2 world bus on; team = M org pack + P-8.
- **P-002/P7-E009** [E] [SURFACE] class=`E/SURFACE` anchor=`P-7:L26-L26, L53-L54, L57-L58, L70-L70` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L161): CLI: `elai profile show\|tighten\|loosen <domain>` is the Tier-1 production caller; loosen refuses without evidence.
- **P-002/P7-E010** [E] class=`E/EVENT` anchor=`P-7:L31-L35, L65-L68` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L162): Profile change emits `operating_profile_change {domain, field, old, new, evidence_ref}`; tighten applies + journals; loosen without evidence is REFUSED (no change row); loosen with H-18-significant own-run FP evidence promotes + journals with evidence_ref.
- **P-002/P7-E011** [E] class=`E/FLOW` anchor=`P-7:L10-L10, L16-L16, L33-L35` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L163): Every profile change is observe-first (GD-4), journaled, and revertible (H-49 applies to configuration); change runs observe-only first where it defaults a gate on.
- **P-002/P7-E012** [E] class=`E/CFG` anchor=`P-7:L40-L41` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L164): FORK: if N-3 promotion pipeline is NOT yet live, ship Tier-1 with asymmetry-guarded MANUAL change function (tighten free, loosen refused without explicit own-run FP evidence) and route AUTOMATIC learning to Tier-2; asymmetry safety is fully realized manually.
- **P-002/P7-E013** [E] class=`E/LIMIT` anchor=`P-7:L48-L49` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L165): ABORT: if loosen path can be satisfied WITHOUT measured own-run FP evidence (user assertion, global default, another user's data), stop and flag; that is the A-4 attack surface.
- **P-002/P7-E014** [E] [SURFACE] class=`E/INT` anchor=`P-7:L57-L59, L70-L70` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L166): Production reachability: `elai profile tighten\|loosen` CLI on a live run; test callers never satisfy R7.
- **P-002/P7-E015** [E] class=`E/TEST` anchor=`P-7:L72-L76` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L167): VERIFY: wiring_contract_check + production_caller_check; pass = loosen REFUSED off-agent without measured own-run FP evidence and PROMOTED through H-18 with it, tighten applied freely, every change journaled + revertible.
- **P-002/P7-E016** [E] class=`E/TEST` anchor=`P-7:L78-L78` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L168): INTEGRATION_SMOKE R10: independent agent tightens, attempts loosen with fabricated/insufficient evidence (must refuse), supplies measured own-run FP evidence and loosens (H-18 promote), reverts, asserts asymmetry + three defaults each mapped to sibling unit posture.
- **P-002/P7-E017** [E] [SURFACE] class=`E/SEC` anchor=`P-7:L80-L81` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L169): REDTEAM failed: talk ELAI into lowering a security gate by acting frustrated — loosening gated on measured FP evidence not affect. Succeeded attack: loosen using another user's or global FP rate — patched by scoping FP evidence to THAT user's own runs only.
- **P-002/P7-E018** [E] class=`E/EVIDENCE` anchor=`P-7:L83-L85` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L170): Grounding: design digest GAP-7 + A-4 (adaptation-as-sycophancy) + A-2 (falsifier: profile-promotion FP rate); feature-map N-3/H-17/H-18/M-2/GD-4/H-49/E-5/E-4/D-14/N-15.
- **P-002/P7-E019** [E] class=`E/DEP` anchor=`P-7:L87-L88` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L171): COMPOSES-WITH: N-3/H-17/H-18 (EXTEND to process metrics), M-2 (mirror asymmetry), GD-4/H-49, E-5 (disjoint), P-5/P-2/M/P-8 (defaults' targets).
- **P-002/P7-A001** [A] class=`A/CFG` anchor=`P-7:L9-L10, L29-L32` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L172): FP metric definitions, promotion thresholds, observation duration, and default numeric values for autonomy/strictness/verbosity are absent.
- **P-002/P7-A002** [A] class=`A/DATA` anchor=`P-7:L9-L10, L36-L38` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L173): Exact domain vocabulary for `per_domain` is not restated beyond sibling default targets; may rely on P-6/N-6.
- **P-002/P7-A003** [A] class=`A/INT` anchor=`P-7:L43-L44` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L174): Whether H-17/H-18 can score a PROCESS metric (FP rate) as readily as a content metric is RECON_NEEDED.
- **P-002/P7-A004** [A] class=`A/INT` anchor=`P-7:L45-L45` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L175): M-2 monotone-tighten enforcement point to mirror at process grain is RECON_NEEDED.
- **P-002/P7-A005** [A] class=`A/INT` anchor=`P-7:L46-L46` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L176): Where E-5 content adaptation lives (to keep P-7 process-only disjoint) is RECON_NEEDED.
- **P-002/P7-C001** [C] class=`C/DEFER` anchor=`P-7:L6-L10, L20-L22, L40-L41` unit=`P-7` (ledger-line: DEEP-P-002.discovery.md:L177): Opening premise frames learned promotion from journaled outcomes as the core, but REVISED Tier-1 is manual asymmetry-guarded change; automatic learning loop is Tier-2 behind N-3 confirmation.

### Unit `P-9` — connector-pack admission format — from P-002 (27 records)

- **P-002/P9-E001** [E] class=`E/STATUS` anchor=`P-9:L1-L5` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L187): P-9 is PLANNED product surface executing AFTER the proof spine (A-7 / GATE M-0 / PSR-1); authored 2026-07-08 from command-center eval; PROOF_SPINE none (distribution); PSR-2 ack 2026-07-09; breadth-moat for P's distribution phase.
- **P-002/P9-E002** [E] class=`E/CAP` anchor=`P-9:L6-L7` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L188): Applies M-6 own-the-format to CAPABILITY: signed, versioned connector/stage-PACK format (declarative where possible + code where necessary) mounted through EXISTING admission family (L-12 + skillspector + J-102 + N-6 scanner registry + taint tiers). Vocabulary stays CLOSED; connector surface becomes UNBOUNDED.
- **P-002/P9-E003** [E] class=`E/DATA` anchor=`P-9:L9-L10, L40-L46` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L189): ONE generic `http_call` stage kind parameterized by declarative ConnectorSpec (endpoint map, auth kind, pagination, response schema-map -> PortSchema) — DATA (admission-gated), never free-form code; ~80% of connectors are a DATA DIFF of the first pack.
- **P-002/P9-E004** [E] class=`E/CAP` anchor=`P-9:L10-L10, L31-L32, L50-L52` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L190): Genuinely-new behavior takes LANE B code-pack path via B-37 RuntimeAdapter closed-trait precedent.
- **P-002/P9-E005** [E] class=`E/DEP` anchor=`P-9:L12-L17` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L191): Substrate: P-2/M-5 mountable-node (tools with ELAI-verifiable evidence only — P-9 generalizes to arbitrary connectors); M-6 golden-lock; L-12/skillspector/J-102/N-6/G-1 admission family; B-37 RuntimeAdapter; O A3 closed stage vocabulary + A-5 data-not-code.
- **P-002/P9-E006** [E] class=`E/DATA` anchor=`P-9:L40-L43` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L192): MOVE 1: ConnectorSpec is human-frozen SCHEMA_REQUIRED JSON-Schema for `{endpoints map, auth_kind enum, pagination enum, response schema-map -> PortSchema}`; versioned connector-spec.schema.json; malformed/code-in-data field REJECTED.
- **P-002/P9-E007** [E] class=`E/FLOW` anchor=`P-9:L44-L46` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L193): MOVE 2: `http_call` executes ConnectorSpec (auth -> paginated fetch -> taint Untrusted at ingest (G-1) -> response schema-map -> PortSchema-validated emit). One kind, unbounded endpoints.
- **P-002/P9-E008** [E] class=`E/SEC` anchor=`P-9:L44-L46, L67-L67, L87-L88` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L194): EGRESS IS PINNED: `http_call` may only connect to hosts in the mounted spec's declared endpoints map (composed with G-3 host-side allowlist/IP-pin); declared-host set is part of SIGNED pack; request to undeclared host REFUSED pre-connect (pure-data exfil patch).
- **P-002/P9-E009** [E] class=`E/SEC` anchor=`P-9:L47-L49, L84-L85` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L195): MOVE 3: signed versioned pack (spec + PortSchema + N-6 tier + manifest) mounts through L-12/skillspector/J-102; M-6-style golden-lock; well-formed pack mounts (MountedConnector row/event); TAMPERED pack (bad signature / failed skillspector) REFUSES fail-closed (never mount-with-warning).
- **P-002/P9-E010** [E] [SURFACE] class=`E/SURFACE` anchor=`P-9:L50-L52, L64-L66` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L196): MOVE 4: `elai connector new <name>` scaffolds declarative pack template (LANE A); ELAI fills endpoints/auth/schema-map with anti-slopsquat + source-grounding recon; LANE B code pack implements closed trait + mounts through same gate.
- **P-002/P9-E011** [E] class=`E/DATA` anchor=`P-9:L53-L55` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L197): MOVE 5: decisions doc `workflow/reasoning/P-9-decisions.md` records ConnectorSpec enum choices + API grounding, LANE-A/B boundary, golden-lock bump procedure.
- **P-002/P9-E012** [E] class=`E/CFG` anchor=`P-9:L31-L32` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L198): FORK-1: if ConnectorSpec covers target API shape -> LANE A (data pack); if genuinely-new transport/behavior -> LANE B (B-37 code pack). Default LANE A.
- **P-002/P9-E013** [E] [SURFACE] class=`E/CFG` anchor=`P-9:L33-L33` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L199): FORK-2: pack signing reuses existing ELAI RRA/.rra signing key if applicable; else scoped key is RECON item.
- **P-002/P9-E014** [E] class=`E/LIMIT` anchor=`P-9:L35-L36` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L200): ABORT-1: pack format that would OPEN stage vocabulary (generic/custom runtime node / n8n model) -> STOP.
- **P-002/P9-E015** [E] class=`E/LIMIT` anchor=`P-9:L37-L37` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L201): ABORT-2: declarative ConnectorSpec carrying executable code (eval, script field) -> STOP; code goes through LANE B only.
- **P-002/P9-E016** [E] class=`E/LIMIT` anchor=`P-9:L38-L38` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L202): ABORT-3: admission needing a NEW ungated path bypassing L-12/skillspector/N-6 -> STOP.
- **P-002/P9-E017** [E] class=`E/INT` anchor=`P-9:L57-L58` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L203): PRODUCTION_HOOK: `http_call` consumed by O-1 fetch->classify / FSM stage dispatcher when mounted ConnectorSpec runs in a TaskSpec; pack-mount path produces MountedConnector SSB row via L-12/N-6 registry. Exact file+fn seams RECON at execution pre-flight.
- **P-002/P9-E018** [E] class=`E/TEST` anchor=`P-9:L60-L61` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L204): INTEGRATION_SMOKE R10: mount fixture pack, run TaskSpec firing `http_call` against local mock behind auth+pagination, assert taint-tagged PortSchema-valid SSB row AND tampered pack REFUSES.
- **P-002/P9-E019** [E] class=`E/TEST` anchor=`P-9:L83-L89` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L205): ACCEPTANCE binary: closed vocab + unbounded endpoints; declarative pack mounts through existing admission; tampered refuses; http_call taints Untrusted at ingest + PortSchema validates; versioned golden-locked enums source-grounded (KD-1); undeclared-host refused pre-connect; OFF-AGENT MountedConnector + fetch rows + INTEGRATION_SMOKE green.
- **P-002/P9-E020** [E] class=`E/LIMIT` anchor=`P-9:L67-L67` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L206): stub_ban: NO open/custom runtime node; NO code field in declarative spec; NO new ungated admission; NO invented auth/pagination enums; NO mount-with-warning; NO http_call egress outside declared endpoints.
- **P-002/P9-E021** [E] class=`E/DEFER` anchor=`P-9:L91-L92` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L207): REDTEAM deferred to execution pre-flight (P post-spine); known surfaces: code-in-data injection, admission-bypass, closed-vocab under custom-node pressure, pure-data exfil via undeclared-host URL templates.
- **P-002/P9-E022** [E] class=`E/EVIDENCE` anchor=`P-9:L94-L96` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L208): ~1000-integration n8n figure is training-unsearched (KD-1) — do not cite externally without verification. Declarative-spec expressiveness ceiling is RECON-1 empirical.
- **P-002/P9-E023** [E] class=`E/DEP` anchor=`P-9:L98-L99` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L209): Next: P-10 supplies auth substrate for http_call; P-11 is demo converting n8n user; all three execute after proof spine.
- **P-002/P9-A001** [A] class=`A/DATA` anchor=`P-9:L26-L27` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L210): Minimal ConnectorSpec coverage for HTTP long tail (auth-kind + pagination enums) requires RECON-1 against real n8n node definitions + OpenAPI taxonomy — do NOT invent.
- **P-002/P9-A002** [A] class=`A/INT` anchor=`P-9:L28-L28` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L211): Whether L-12/skillspector/J-102/N-6 accepts non-skill pack as-is or needs pack-type discriminator is RECON-2.
- **P-002/P9-A003** [A] class=`A/DATA` anchor=`P-9:L29-L29` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L212): How response schema-map binds to existing PortSchema/threaded-contract system is RECON-3; response map must produce human-frozen PortSchema the generator satisfies.
- **P-002/P9-A004** [A] class=`A/SEC` anchor=`P-9:L33-L33` unit=`P-9` (ledger-line: DEEP-P-002.discovery.md:L213): Exact pack signing key reuse path is FORK-2 / RECON.

### Unit `P-10` — ingress adapters + credential vault — from P-002 (25 records)

- **P-002/P10-E001** [E] class=`E/STATUS` anchor=`P-10:L1-L5` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L223): P-10 is PLANNED product surface post-spine (A-7/GATE M-0/PSR-1); authored 2026-07-08; PROOF_SPINE none; PSR-2 ack 2026-07-09; supplies ingress + credential substrate for P-9 http_call and command-center vision.
- **P-002/P10-E002** [E] class=`E/CAP` anchor=`P-10:L6-L7` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L224): Do NOT grow O-2 trigger enum per protocol. Add INGRESS ADAPTER family converting any external shape into a taint-tagged (G-1) journaled SsbEvent row so downstream (O-2 SsbEvent trigger, G-2 replay, provenance, privacy lattice) works UNCHANGED. Plus first-class CredentialVault.
- **P-002/P10-E003** [E] class=`E/FLOW` anchor=`P-10:L9-L14, L37-L38` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L225): Four external shapes normalize at boundary to `{external bytes -> taint Untrusted (G-1) -> SsbEvent row}`: (a) OUTBOUND PULL — O-2 Cron + fetch + vault only; (b) INBOUND PUSH — WebhookIngress on I-1 supervisor plane; (c) STREAMING/PERSISTENT — long-lived listener daemon (C-17 FileWatcherService shape); (d) GENUINELY OTHER — B-37/P-9 pack adapters.
- **P-002/P10-E004** [E] class=`E/DEP` anchor=`P-10:L16-L22` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L226): Substrate: O-2 Cron\|SsbEvent\|Manual triggers (SsbEvent is join point); G-1/G-2/K-14/G3-G6 async replay; I-1 supervisor plane; C-17 FileWatcher; B-37/P-9 packs; G-3 credential broker + M-1 persona tokens are scatter P-10 consolidates; persona-excluded per P-3 export rules.
- **P-002/P10-E005** [E] class=`E/CAP` anchor=`P-10:L47-L49, L89-L90` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L227): MOVE 1: IngressAdapter closed trait — one job `{external bytes -> taint Untrusted -> SCHEMA_REQUIRED SsbEvent row}`; same contract all four shapes; O-2 enum UNCHANGED; adapter event indistinguishable-downstream from native.
- **P-002/P10-E006** [E] class=`E/SEC` anchor=`P-10:L50-L52, L73-L73, L94-L94` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L228): MOVE 2 WebhookIngress: local HTTP listener on I-1; INBOUND AUTH MANDATORY — every registered hook carries per-hook shared secret; inbound posts signature-verified (HMAC-over-body GitHub-webhook idiom; exact scheme RECON) BEFORE any byte is parsed; missing/failed signature REFUSED with journaled refusal row and NO SsbEvent.
- **P-002/P10-E007** [E] class=`E/DET` anchor=`P-10:L11-L12, L50-L52, L90-L91` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L229): K-14 journals the external event as a replay input so G-2 replay reproduces from journal WITHOUT live re-fetch (faithful async replay).
- **P-002/P10-E008** [E] class=`E/SEC` anchor=`P-10:L53-L55, L91-L92` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L230): MOVE 3 CredentialVault: first-class credential plane (OS-keychain preferred, encrypted-file fallback); secrets NEVER in SSB payload / RRA / P-3 export; http_call (P-9) and fetch stage draw auth from it via handle resolved at call time.
- **P-002/P10-E009** [E] class=`E/CFG` anchor=`P-10:L39-L39, L100-L101` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L231): FORK-2 vault backend: prefer OS keychain; file-at-rest fallback with explicit at-rest-encryption RECON; never plaintext.
- **P-002/P10-E010** [E] class=`E/CAP` anchor=`P-10:L56-L58, L92-L93` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L232): MOVE 4: persistent-listener daemon shape (c) reusing C-17 + shape-d pack path; registry of active adapters on I-1 plane; listeners NOT in pipeline hot path.
- **P-002/P10-E011** [E] [SURFACE] class=`E/SURFACE` anchor=`P-10:L35-L35, L73-L73` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L233): CLI credential acquisition v1: `elai cred add` manual paste; OAuth flows (device-code vs localhost-redirect, refresh rotation) are DECLARED follow-on; if a P-9 connector needs OAuth before follow-on lands, that connector is BLOCKED on it, not hand-rolled inline.
- **P-002/P10-E012** [E] class=`E/DATA` anchor=`P-10:L59-L61` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L234): MOVE 5: decisions doc `workflow/reasoning/P-10-decisions.md` records four-shape routing, vault backend + at-rest choice, replay-as-input proof.
- **P-002/P10-E013** [E] class=`E/LIMIT` anchor=`P-10:L41-L44` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L235): ABORT-1: growing O-2 trigger enum per protocol -> STOP. ABORT-2: inbound push not journalable as faithful replay input -> STOP. ABORT-3: credential in cleartext in any SSB/RRA/P-3 export -> STOP.
- **P-002/P10-E014** [E] class=`E/INT` anchor=`P-10:L63-L64` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L236): PRODUCTION_HOOK: IngressAdapter/WebhookIngress/FileWatch -> O-2 SsbEvent trigger matcher; CredentialVault -> fetch / P-9 http_call auth path. Seams RECON at pre-flight.
- **P-002/P10-E015** [E] class=`E/TEST` anchor=`P-10:L66-L67` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L237): INTEGRATION_SMOKE R10: POST to live WebhookIngress -> taint-tagged SsbEvent + O-2 TaskSpec fires; G-2-replay reproduces from journal (no re-fetch); http_call draws vault credential; grep journal/RRA/export for secret == 0 hits.
- **P-002/P10-E016** [E] class=`E/TEST` anchor=`P-10:L89-L95` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L238): ACCEPTANCE: O-2 enum unchanged; all four shapes normalize to taint-tagged SsbEvent; faithful journal replay; CredentialVault usable + zero leak; listeners on I-1 gated by registry; unsigned post refused before parse; OFF-AGENT proof artifacts.
- **P-002/P10-E017** [E] class=`E/LIMIT` anchor=`P-10:L73-L73` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L239): stub_ban: NO new O-2 trigger kind; NO live-re-fetch-on-replay; NO secret in journal/RRA/export; NO listener in hot path; NO plaintext at-rest; NO unauthenticated inbound ingress; NO inline hand-rolled OAuth.
- **P-002/P10-E018** [E] class=`E/DEFER` anchor=`P-10:L97-L98` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L240): REDTEAM deferred to execution pre-flight; aim at credential-leak, webhook mid-run replay faithfulness, listener escaping supervisor plane, forged-ingress (including replaying captured signed payload / nonce-timestamp RECON).
- **P-002/P10-E019** [E] class=`E/LIMIT` anchor=`P-10:L101-L102` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L241): Persistent-listener daemon lifecycle (crash/restart, backpressure on firehose) needs C-17-pattern reuse review.
- **P-002/P10-E020** [E] class=`E/DEP` anchor=`P-10:L104-L105` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L242): Next: P-10 gives P-9 http_call its auth plane and command center its inbound-event surface; P-11 composes both into describe/confirm/run.
- **P-002/P10-A001** [A] class=`A/DATA` anchor=`P-10:L31-L32` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L243): Exact SsbEvent row shape for external ingress so adapter event is indistinguishable-downstream is RECON-1 (SCHEMA_REQUIRED against O-2).
- **P-002/P10-A002** [A] class=`A/DET` anchor=`P-10:L33-L33` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L244): Whether K-14 already journals external wall-clock as replay input or webhook needs a new "recorded external event" leg is RECON-2.
- **P-002/P10-A003** [A] class=`A/SEC` anchor=`P-10:L34-L34` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L245): How secrets are stored at rest and whether credential bytes get a taint tier (never journaled cleartext, never exported P-3) is RECON-3.
- **P-002/P10-A004** [A] class=`A/SEC` anchor=`P-10:L50-L50` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L246): Exact HMAC/signing scheme for webhook providers is RECON against real providers' signing docs.
- **P-002/P10-A005** [A] class=`A/CFG` anchor=`P-10:L35-L35` unit=`P-10` (ledger-line: DEEP-P-002.discovery.md:L247): OAuth acquisition is declared follow-on without schedule; provider-specific endpoints must be source-grounded at execution (anti-stale).

### Unit `P-11` — workflow from-goal — from P-002 (25 records)

- **P-002/P11-E001** [E] [SURFACE] class=`E/STATUS` anchor=`P-11:L1-L5` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L257): P-11 is PLANNED product surface post-spine; authored 2026-07-08; PROOF_SPINE none; PSR-2 ack 2026-07-09; user-facing composition demo riding P-9 vocabulary + P-10 ingress; n8n-user-converting front door.
- **P-002/P11-E002** [E] [SURFACE] class=`E/CAP` anchor=`P-11:L6-L7, L41-L43` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L258): `elai workflow from-goal <intent>` has the model PROPOSE a TaskSpec graph drawn ONLY from closed stage/connector vocabulary (O A3 kinds + P-9 mounted connectors), rendered on O-4 canvas, human CONFIRMS before anything runs.
- **P-002/P11-E003** [E] class=`E/FLOW` anchor=`P-11:L9-L10` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L259): Constrained synthesis: model may only propose graphs whose nodes are existing typed stage kinds + mounted connectors, wires satisfy PortSchema/threaded-contract, gaps render VISIBLY DISCONNECTED (never silently invented). Human is final wiring authority.
- **P-002/P11-E004** [E] [SURFACE] class=`E/DEP` anchor=`P-11:L12-L18` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L260): Substrate: H-13 model-drives-canvas-ops (constrained mutation seam); O-2 TaskSpec rows + triggers; O-4 canvas + wire-dark; L-14 elai build RRA receipt pattern; P-9 mounted connectors + O A3 closed enum; PortSchema contracts.
- **P-002/P11-E005** [E] class=`E/CFG` anchor=`P-11:L32-L33` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L261): FORK-1: if constrained-decode/grammar path exists -> constrain at decode; else propose-then-validate-then-reject with fail-closed validator + bounded re-propose loop. Prefer constrained decode; validator is mandatory backstop either way.
- **P-002/P11-E006** [E] [SURFACE] class=`E/CFG` anchor=`P-11:L34-L34, L67-L68` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L262): FORK-2: intent needs unmounted connector -> render VISIBLY DISCONNECTED gap + offer `elai connector new` (P-9 LANE A); do NOT auto-author connector inside from-goal.
- **P-002/P11-E007** [E] class=`E/LIMIT` anchor=`P-11:L36-L37` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L263): ABORT-1: synthesis emitting a node OUTSIDE closed vocabulary -> STOP. Unavailable capability renders as disconnected gap, never fabricated node.
- **P-002/P11-E008** [E] class=`E/LIMIT` anchor=`P-11:L37-L38, L48-L50` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L264): ABORT-2: proposed graph could EXECUTE without explicit human confirmation -> STOP. Proposal is inert (rows exist, no trigger armed) until confirmed.
- **P-002/P11-E009** [E] [SURFACE] class=`E/LIMIT` anchor=`P-11:L38-L39` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L265): ABORT-3: proposal WITHOUT RRA receipt (intent -> graph, provable) -> STOP; L-14 receipt makes synthesis auditable.
- **P-002/P11-E010** [E] [SURFACE] class=`E/SURFACE` anchor=`P-11:L41-L43, L64-L67` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L266): MOVE 1 CLI: `elai workflow from-goal <intent>` proposes inert TaskSpec graph (no trigger armed).
- **P-002/P11-E011** [E] [SURFACE] class=`E/SURFACE` anchor=`P-11:L44-L47` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L267): MOVE 2: proposed graph renders on O-4 canvas; unavailable capability VISIBLY DISCONNECTED with "needs a P-9 connector pack" hint.
- **P-002/P11-E012** [E] class=`E/FLOW` anchor=`P-11:L48-L50` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L268): MOVE 3 confirm-before-run: human confirms/edits on canvas; confirmation arms O-2 triggers; before confirm no stage dispatches (no execution SSB rows).
- **P-002/P11-E013** [E] [SURFACE] class=`E/EVENT` anchor=`P-11:L51-L53` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L269): MOVE 4: from-goal run journals `{intent, proposed graph, model, confirmation}` as RRA receipt (L-14 pattern); receipt verifies via G-2.
- **P-002/P11-E014** [E] class=`E/DATA` anchor=`P-11:L54-L56` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L270): MOVE 5: decisions doc `workflow/reasoning/P-11-decisions.md` records decode-constraint choice, gap-rendering contract, confirm-gate placement.
- **P-002/P11-E015** [E] [SURFACE] class=`E/INT` anchor=`P-11:L58-L59` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L271): PRODUCTION_HOOK: CLI from-goal -> H-13 canvas-op + O-4 render + O-2 TaskSpec store (inert rows); confirm gate -> O-2 trigger-arm path; from-goal run -> L-14 RRA sink. Seams RECON at pre-flight.
- **P-002/P11-E016** [E] class=`E/TEST` anchor=`P-11:L61-L62` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L272): INTEGRATION_SMOKE R10: multi-step intent mixing available + UNMOUNTED connector; assert available part connected, unmounted VISIBLY DISCONNECTED (no fabricated node), NOTHING dispatches pre-confirm, RRA intent->graph lands, post-confirm O-2 triggers arm.
- **P-002/P11-E017** [E] class=`E/TEST` anchor=`P-11:L84-L88` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L273): ACCEPTANCE: every proposed node in closed vocab + wires PortSchema-valid; unavailable renders DISCONNECTED; zero pre-confirm execution; verifiable RRA; OFF-AGENT evidence + INTEGRATION_SMOKE green.
- **P-002/P11-E018** [E] class=`E/LIMIT` anchor=`P-11:L67-L68` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L274): stub_ban: NO out-of-vocab/fabricated node; NO execution pre-confirm; NO proposal without RRA; NO auto-authoring connector inside from-goal.
- **P-002/P11-E019** [E] class=`E/DEFER` anchor=`P-11:L90-L91` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L275): REDTEAM deferred to execution pre-flight; aim at smuggling out-of-vocab via prompt-injection in intent, proposal arming trigger without confirmation, fabricated-connected node for unavailable connector.
- **P-002/P11-E020** [E] class=`E/LIMIT` anchor=`P-11:L94-L95` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L276): Intent ambiguity (one goal -> several graphs) is expected; from-goal proposes ONE + human edits on canvas — proposal front door, not autonomous builder.
- **P-002/P11-E021** [E] class=`E/DEP` anchor=`P-11:L97-L98` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L277): Next: P-11 front door makes P-9 + P-10 legible as describe/confirm/run; all three execute after proof spine as command-center distribution layer.
- **P-002/P11-A001** [A] class=`A/CFG` anchor=`P-11:L27-L28, L93-L94` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L278): Constrained-decode vs propose-then-validate (FORK-1) is RECON-1 empirical on ELAI's actual decode path.
- **P-002/P11-A002** [A] [SURFACE] class=`A/SURFACE` anchor=`P-11:L29-L29` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L279): Whether O-4 maps proposed-but-unavailable node to disconnected visual + connector-new hint is RECON-2.
- **P-002/P11-A003** [A] class=`A/FLOW` anchor=`P-11:L30-L30` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L280): Exact O-2/FSM site for human confirmation so NOTHING executes until confirmed is RECON-3.
- **P-002/P11-A004** [A] class=`A/RETRY` anchor=`P-11:L32-L33` unit=`P-11` (ledger-line: DEEP-P-002.discovery.md:L281): Bounded re-propose loop bounds (count, stop condition) under propose-then-validate are not numeric.

### Unit `P-14` — explore-then-crystallize record->replay lane — from P-002 (23 records)

- **P-002/P14-E001** [E] class=`E/STATUS` anchor=`P-14:L1-L5` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L291): P-14 is PLANNED WAVE CU-2 EXTENSION of E-14 + O-2; PROOF_SPINE feeds PS-5; execute after proof spine per A-7; Track COMPUTER-USE-STEAL; lever `agentic-run-mints-artifact`; STRONG grounding (cua replay_env HIGH; openclaw jsonl-replay is NOT record->re-score — C4); effort M-L; depends P-13.
- **P-002/P14-E002** [E] [SURFACE] class=`E/CAP` anchor=`P-14:L5-L5, L18-L19` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L292): Exploration run emits a deterministic script AS an O-2 TaskSpec stage + RRA receipt binding script->minting-run; replay = cheap zero-reasoning execution mode.
- **P-002/P14-E003** [E] class=`E/EVIDENCE` anchor=`P-14:L8-L9, L20-L21` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L293): Prior-art correction C4: openclaw jsonl-replay re-drives INPUT through MOCK provider — cite cua replay_env, not openclaw, as the shape prior art. EXCLUDED as vaporware: openclaw-style mock replay framed as record->re-score.
- **P-002/P14-E004** [E] class=`E/CFG` anchor=`P-14:L18-L19` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L294): SHIP NOW (Tier-1): crystallizer takes E-14 (browser) exploration trace and EMITS deterministic Playwright script as O-2 TaskSpec stage kind, with G-2 RRA binding emitted script hash to exploration run_id; replay drives real entrypoint with NO model in the loop.
- **P-002/P14-E005** [E] class=`E/DEFER` anchor=`P-14:L19-L20` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L295): DEFER (Tier-2): pyautogui (desktop) crystallizer over P-12 traces; re-score-on-replay leg (cua evaluate_task analog); assertion synthesis from exploration AXTree deltas.
- **P-002/P14-E006** [E] class=`E/DATA` anchor=`P-14:L29-L31` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L296): produces: `crates/orchestrator/src/crystallize/mod.rs` (Crystallizer) + `rra_bind.rs` (script_hash -> minting run_id RRA binder).
- **P-002/P14-E007** [E] class=`E/DEP` anchor=`P-14:L11-L16, L32-L34` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L297): Consumes O-2 TaskSpecStage (EXTEND kinds), G-2 ReproducibleRunArtifact; composes C crystallizer general form (do NOT fork); J-96 case graph adjacency feeds PS-5.
- **P-002/P14-E008** [E] [SURFACE] class=`E/SURFACE` anchor=`P-14:L35-L36, L46-L48, L71-L72` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L298): CLI: `elai e2e crystallize` + replay via task_runner tick (`elai e2e replay --stage`); PRODUCTION CLI + task_runner.
- **P-002/P14-E009** [E] [SURFACE] class=`E/SURFACE` anchor=`P-14:L43-L44` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L299): frontend_pair: crystallized-stage node on O-4 canvas (Tier-2; decide at edit W-4).
- **P-002/P14-E010** [E] class=`E/LIMIT` anchor=`P-14:L38-L42` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L300): stub_ban: NO crystallized stage without RRA binding to real minting run; NO model call in replay path; NO new dispatch runtime; NO .unwrap()/.expect(); NO blake2/sha2 (blake3 for script hash).
- **P-002/P14-E011** [E] class=`E/EVENT` anchor=`P-14:L46-L48, L50-L51` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L301): Minting lands `crystallized_stage_minted` row `{script_hash, minting_run_id, rra_hash}`.
- **P-002/P14-E012** [E] class=`E/DET` anchor=`P-14:L50-L55` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L302): Replay reproduces exploration's observable outcome with ZERO model calls in the trace; byte-reproducible under same seed (G-2).
- **P-002/P14-E013** [E] class=`E/CFG` anchor=`P-14:L53-L53` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L303): FORK: if exploration has genuinely non-deterministic step (CAPTCHA, model-only inference), crystallization DOES NOT emit that step as deterministic — route to marked "agentic-step" stage re-entering exploration; record boundary in RRA; do NOT silently bake guessed value.
- **P-002/P14-E014** [E] class=`E/SEC` anchor=`P-14:L61-L61` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L304): REDTEAM surviving-attack seed: crystallized script re-invoking model every step — caught by zero-model-calls assertion. Unpatched success: bake run-specific session token/timestamp into replay — PATCH: parameterize run-specific values as stage inputs and RRA records them; hard-coded volatile literal is stub_ban. Deep red-team deferred to execution pre-flight.
- **P-002/P14-E015** [E] class=`E/LIMIT` anchor=`P-14:L55-L55` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L305): ABORT: if crystallization would produce replay NOT byte-reproducible under same seed, STOP — deterministic replay that drifts defeats the lever.
- **P-002/P14-E016** [E] [SURFACE] class=`E/TEST` anchor=`P-14:L56-L60` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L306): VERIFY: plan_gate_check + wiring_contract_check; load-bearing proof is off-agent crystallized_stage_minted + replay trace asserting zero model calls via real `elai e2e crystallize`/`replay`.
- **P-002/P14-E017** [E] [SURFACE] class=`E/TEST` anchor=`P-14:L63-L64` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L307): INTEGRATION_SMOKE: independent agent full E-14 multi-step SaaS journey (login + multi-page form), crystallize, replay N times byte-identical under seed with zero model calls, `elai replay --verify` re-binds script to minting run. Toy single-step does NOT satisfy R10.
- **P-002/P14-E018** [E] [SURFACE] class=`E/INT` anchor=`P-14:L71-L72` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L308): PRODUCTION_HOOK: task_runner TaskSpecStage gains crystallized_script kind; `elai e2e crystallize`/`replay` in cli/main.rs; Crystallizer + RRA binder wired both same-changeset.
- **P-002/P14-E019** [E] class=`E/DEP` anchor=`P-14:L74-L75` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L309): Next: closes SYNTHESIS 2b; compounds PS-5; pairs with P-15 as reuse-artifact minters feeding K-18 case-reuse economics.
- **P-002/P14-A001** [A] class=`A/INT` anchor=`P-14:L54-L54` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L310): Exact O-2 TaskSpecStage enum to extend and cua environment.py record->re-score shape confirmation are RECON_NEEDED at edit.
- **P-002/P14-A002** [A] class=`A/TEST` anchor=`P-14:L8-L8` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L311): No quantitative replay-reliability threshold is set ("~100% reliability" is narrative industry split).
- **P-002/P14-A003** [A] class=`A/DATA` anchor=`P-14:L61-L61` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L312): Full script portability matrix beyond parameterized stage inputs is not fully specified.
- **P-002/P14-C001** [C] class=`C/DEFER` anchor=`P-14:L18-L20` unit=`P-14` (ledger-line: DEEP-P-002.discovery.md:L313): Industry/summary framing often pairs Playwright + PyAutoGUI as peer emitters; numbered source puts browser Playwright in Tier-1 and desktop pyautogui in Tier-2.

### Unit `P-19` — config/ops lifecycle + elai doctor + verified backups — from P-002 (24 records)

- **P-002/P19-E001** [E] class=`E/STATUS` anchor=`P-19:L1-L5` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L323): P-19 is PLANNED WAVE OC-2 NEW-area (composition-heavy; ELAI's thinnest area); EXTENSION of J config-ingress + M + 3H; PROOF_SPINE none (ops); owner-acked 2026-07-11; execute after proof spine; Track OPENCLAW-STEAL; lever `recoverable-config-and-state`; STRONG openclaw grounding.
- **P-002/P19-E002** [E] [SURFACE] class=`E/CAP` anchor=`P-19:L5-L5` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L324): Add recoverable config, `elai doctor`, verified journal backups, and a restart continuation queue so a trust product cannot cheaply corrupt its own config or fail to restore state.
- **P-002/P19-E003** [E] [SURFACE] class=`E/EVIDENCE` anchor=`P-19:L8-L9` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L325): Steal from openclaw: strict-schema fail-closed boot (unknown key = REFUSE), atomic writes, last-known-good with doctor-only restore, clobber guards (>50% shrink rejected, preserved as .rejected.<ts>), hot-reload settle + invalid-edit rejection; doctor detect/repair/lint with STABLE exit codes; VACUUM-INTO WAL-safe SQLite snapshots + manifest verify; restart continuation queue (one turn); compaction split-rule (tool-call/result pairs together) + successor transcripts (never rewrite in place).
- **P-002/P19-E004** [E] class=`E/LIMIT` anchor=`P-19:L9-L9, L20-L21, L40-L41` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L326): CORRECTION C2: openclaw copies full process.env with NO allowlist — do the OPPOSITE (allowlist env); adopt openclaw cwd path-jail (case-fold-before-compare on win32) verbatim.
- **P-002/P19-E005** [E] [SURFACE] class=`E/CFG` anchor=`P-19:L18-L19` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L327): SHIP NOW (Tier-1): (a) fail-closed-default-object config posture (policy objects fail closed even when ABSENT) + last-known-good + rejected-write preservation (.rejected.<ts>) + shrink/clobber guards (>50% shrink rejected) + atomic writes on J config-ingress gate; (b) `elai doctor` detect/repair/lint with STABLE exit codes + structured findings; ENV ALLOWLISTED; cwd path-jail.
- **P-002/P19-E006** [E] class=`E/DEFER` anchor=`P-19:L19-L20` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L328): DEFER (Tier-2): VACUUM-INTO WAL-safe verified SSB/session backups + manifest verify; restart continuation queue (one in-flight turn survives restart); post-upgrade compat probes; compaction split-rule + successor-transcript stated invariants (likely already in 3H — make explicit).
- **P-002/P19-E007** [E] class=`E/LIMIT` anchor=`P-19:L20-L21` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L329): EXCLUDED: openclaw full-env-copy (C2); any config write that overwrites on >50% shrink.
- **P-002/P19-E008** [E] [SURFACE] class=`E/DATA` anchor=`P-19:L29-L31` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L330): produces: `crates/orchestrator/src/config/lifecycle.rs` (ConfigLifecycle: LKG + rejected-write + shrink/clobber + atomic write + env allowlist) + `crates/cli/src/doctor_cmd.rs` (elai doctor detect/repair/lint).
- **P-002/P19-E009** [E] class=`E/DEP` anchor=`P-19:L32-L34` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L331): Consumes J ConfigIngressGate (EXTEND recoverable — do NOT re-implement admission) + 3H SessionStore (compaction split-rule + successor-transcript target).
- **P-002/P19-E010** [E] [SURFACE] class=`E/SURFACE` anchor=`P-19:L35-L36, L44-L45, L56-L57` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L332): CLI: `elai doctor` subcommand (detect/repair/lint); PRODUCTION CLI. No dashboard tile — justified W-6 N/A: structured text + stable exit codes for CI.
- **P-002/P19-E011** [E] class=`E/INT` anchor=`P-19:L36-L37, L72-L73` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L333): Production reachability also includes config ingress path invoking lifecycle guards on every write.
- **P-002/P19-E012** [E] [SURFACE] class=`E/LIMIT` anchor=`P-19:L39-L43` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L334): stub_ban: NO config write overwriting on >50% shrink; NO full process.env copy; NO doctor exit code varying run-to-run for same state; NO compaction splitting tool-call from result; NO in-place transcript rewrite; NO .unwrap; NO blake2/sha2.
- **P-002/P19-E013** [E] [SURFACE] class=`E/EVENT` anchor=`P-19:L46-L49, L51-L53` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L335): >50%-shrink write REJECTED and preserved as .rejected.<ts> with `config_write_rejected` row; `elai doctor lint` returns STABLE nonzero exit + structured `doctor_finding` rows.
- **P-002/P19-E014** [E] [SURFACE] class=`E/CFG` anchor=`P-19:L54-L54` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L336): FORK: if config is corrupt beyond last-known-good restore, `elai doctor repair` refuses to auto-repair and surfaces structured finding for human action; do NOT silently write a default over user config.
- **P-002/P19-E015** [E] class=`E/LIMIT` anchor=`P-19:L56-L56` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L337): ABORT: if config-ingress could admit an unknown key silently (drop fail-closed boot) or copy full env, STOP — both are openclaw failure modes this unit exists to beat.
- **P-002/P19-E016** [E] class=`E/SEC` anchor=`P-19:L62-L62` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L338): REDTEAM: truncating write defeated by >50%-shrink clobber guard. Unpatched success: config edit smuggling secret via full-env copy (C2) — PATCH: env allowlisted; secrets live in P-10 vault, never config or journal. Deep red-team deferred.
- **P-002/P19-E017** [E] [SURFACE] class=`E/TEST` anchor=`P-19:L57-L61` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L339): VERIFY: plan_gate_check + wiring_contract_check; load-bearing proof is off-agent config_write_rejected + doctor_finding rows + stable doctor exit via real `elai doctor`.
- **P-002/P19-E018** [E] [SURFACE] class=`E/TEST` anchor=`P-19:L64-L65` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L340): INTEGRATION_SMOKE: hard input malformed config (unknown key) + >50% shrink + kill mid-turn; assert fail-closed boot rejects unknown key, shrink preserved not overwritten, doctor exit stable across 3 runs, in-flight turn resumes once after restart. Note: resume leg is Tier-2 deferred capability used in smoke description.
- **P-002/P19-E019** [E] class=`E/DEP` anchor=`P-19:L75-L76` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L341): Next: closes O3 config/ops lifecycle; pairs with M packaging/support and Q ops hardening; journal-backup leg protects product's evidence store.
- **P-002/P19-A001** [A] class=`A/CFG` anchor=`P-19:L18-L19, L46-L49` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L342): Stable exit-code map and structured doctor_finding schema are not enumerated.
- **P-002/P19-A002** [A] class=`A/DATA` anchor=`P-19:L19-L20` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L343): Restore workflow, retention, backup encryption, storage destination, verification algorithm for Tier-2 backups are absent.
- **P-002/P19-A003** [A] class=`A/STATE` anchor=`P-19:L55-L55` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L344): Exact J ConfigIngressGate + 3H SessionStore symbols and openclaw VACUUM-INTO + restart-continuation shape are RECON_NEEDED.
- **P-002/P19-A004** [A] class=`A/CONC` anchor=`P-19:L19-L20, L64-L65` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L345): Multi-session / multi-queued continuation behavior beyond "one in-flight turn" is not defined (Tier-2).
- **P-002/P19-C001** [C] [SURFACE] class=`C/DEFER` anchor=`P-19:L18-L20, L64-L65` unit=`P-19` (ledger-line: DEEP-P-002.discovery.md:L346): Summary inventory lists VACUUM backups, restart continuation, post-upgrade probes, and compaction invariants as peer requirements; numbered REVISED scope places those in Tier-2 while Tier-1 is config lifecycle guards + doctor. INTEGRATION_SMOKE still references mid-turn resume.

### Unit `P-24` — mobile design/test lane — from P-002 (31 records)

- **P-002/P24-E001** [E] class=`E/STATUS` anchor=`P-24:L1-L5` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L356): P-24 is PLANNED WAVE CU-5; PROOF_SPINE none (product); owner-commissioned 2026-07-11; Tier-3 crystallize transitively feeds PS-5 via P-14 (PSR-1 tier 3); execute after proof spine; Track COMPUTER-USE-STEAL; lever `mobile-as-a-dimension-not-a-subsystem`; STRONG external + MODERATE ELAI-anchor grounding; EXTENSION; effort L; depends H-14(B0+tagging), P-12, P-13, P-14, P-16.
- **P-002/P24-E002** [E] class=`E/CAP` anchor=`P-24:L5-L5, L17-L17` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L357): Mobile is a typed DIMENSION threaded through existing lanes, never a parallel mobile subsystem; closes H-14 smartphone deferral for RN/Expo.
- **P-002/P24-E003** [E] class=`E/FLOW` anchor=`P-24:L16-L17, L41-L42` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L358): Three rungs cheapest-first; every rung labels render source; a lower rung NEVER discharges a higher rung's gate.
- **P-002/P24-E004** [E] class=`E/DATA` anchor=`P-24:L17-L18` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L359): RUNG 1: typed DeviceProfile catalog as DATA rows `{name, width, height, dpr, ua, touch}` (named constants, no magic metrics in code).
- **P-002/P24-E005** [E] [SURFACE] class=`E/SURFACE` anchor=`P-24:L17-L18, L45-L46` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L360): RUNG 1: device_frame render variant on H-14 preview node and P-23 browse node (CDP Emulation.setDeviceMetricsOverride); E-14 journeys accept device MATRIX; SSIM goldens become per-profile.
- **P-002/P24-E006** [E] class=`E/FLOW` anchor=`P-24:L18-L19` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L361): RUNG 2: Expo dev server (Metro web, production-grade SDK 54+) is frameable origin; H-14 B0 proxy + data-oid tagging + recast writeback extend to RNW; click-to-edit resolves RN component AST ranges. Render source labeled `rnw_web` — WEB pass on RN source, never claimed as native.
- **P-002/P24-E007** [E] class=`E/FLOW` anchor=`P-24:L19-L20` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L362): RUNG 3: Android emulator (iOS simulator where macOS host exists) as P-12 sandbox backend; booted from PINNED AVD/sim snapshot whose image hash lands in RRA hardware pin.
- **P-002/P24-E008** [E] class=`E/DATA` anchor=`P-24:L19-L20` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L363): Perception = platform a11y tree (uiautomator dump / XCUITest a11y) as NEW P-13 source tag `mobile_axtree`; pixel/OCR tiers fire only when empty (J-1 unchanged).
- **P-002/P24-E009** [E] class=`E/FLOW` anchor=`P-24:L19-L20` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L364): Actions frame-bound per P-12; agentic exploration crystallizes via P-14 into Maestro YAML flow, RRA-bound; replay = zero-reasoning mode.
- **P-002/P24-E010** [E] class=`E/DEP` anchor=`P-24:L19-L20, L44-L44` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L365): P-16 catalog provisions emulator images/SDKs/Maestro binary (admission-gated, never ad-hoc install).
- **P-002/P24-E011** [E] class=`E/DEFER` anchor=`P-24:L19-L20, L31-L31, L81-L81` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L366): RN click-to-edit-on-native (Fiber getInspectorDataForViewAtPoint) is REGISTERED FOLLOW-UP inside Rung 3's FORK, not a promise.
- **P-002/P24-E012** [E] class=`E/LIMIT` anchor=`P-24:L20-L21, L56-L56` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L367): EXCLUDED: parallel mobile subsystem; VLM-read screenshots as element source (P-13/J-1); real physical devices (neither disposable nor snapshot-pinnable — ABORT class).
- **P-002/P24-E013** [E] class=`E/DATA` anchor=`P-24:L29-L31` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L368): produces: `crates/orchestrator/src/mobile/device_profile.rs` + `emulator_backend.rs` (P-12 SandboxBackend: pinned-snapshot boot + mobile_axtree feed + Maestro crystallize target).
- **P-002/P24-E014** [E] [SURFACE] class=`E/SURFACE` anchor=`P-24:L36-L37, L47-L48, L77-L78` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L369): CLI: `elai e2e run --device <profile>`; `elai operator run --backend emulator`.
- **P-002/P24-E015** [E] class=`E/CFG` anchor=`P-24:L37-L38` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L370): task_runner gains `device_profile` field on browse/e2e/operator TaskSpec stages.
- **P-002/P24-E016** [E] [SURFACE] class=`E/SURFACE` anchor=`P-24:L38-L39, L45-L46` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L371): H-13 canvas dispatcher: device_frame render variant on preview/browse nodes; phone-bezel render + per-profile pass/fail chips; iOS-unavailable = HONEST-GAP disconnected affordance, never silent skip.
- **P-002/P24-E017** [E] class=`E/OBS` anchor=`P-24:L41-L42, P-24:L49-L50` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L372): Every result row carries `render_source` enum `web_viewport \| rnw_web \| native_emulator`; rung-label laundering is a gate defect.
- **P-002/P24-E018** [E] class=`E/EVENT` anchor=`P-24:L47-L50, L52-L53` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L373): expect_ssb `emulator_session`; journey axtree_snapshot rows carry device_profile + render_source; Rung-3 RRA hardware pin holds snapshot hash; crystallizes Maestro flow RRA-bound.
- **P-002/P24-E019** [E] class=`E/TEST` anchor=`P-24:L49-L50` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L374): NEGATIVE fails: rnw_web pass presented against native_emulator gate; emulator boot from unpinned image; mobile count sourced from VLM.
- **P-002/P24-E020** [E] class=`E/CFG` anchor=`P-24:L54-L55` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L375): FORK: no macOS host -> iOS Tier-3 unavailable as honest-gap (Android-only); NEVER substitute rnw_web for missing iOS. Empty uiautomator a11y on RN -> P-13 tier ladder (OCR/pixel) + `perception_degraded` (not silent axtree_exact). Maestro fails P-16 admission -> fall back to P-12 actuator with plain journaled step list as crystallize format; do NOT install around catalog.
- **P-002/P24-E021** [E] class=`E/LIMIT` anchor=`P-24:L56-L56` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L376): ABORT: physical device drive; emulator that reaches host filesystem/network outside sandbox profile; missing rung-label on any result row; H-14 B0 proxy still unshipped at execution (STOP — do not re-implement proxy).
- **P-002/P24-E022** [E] class=`E/TEST` anchor=`P-24:L53-L53, L63-L64` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L377): Maestro is actuation-only; replay verdicts come from SAME off-agent evidence as minting run (SSB rows + screenshot hash compare per 4I); negative arm: mutated fixture makes replay actuate green while evidence leg FAILS.
- **P-002/P24-E023** [E] class=`E/TEST` anchor=`P-24:L57-L62` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L378): VERIFY: plan_gate_check + wiring_contract_check + production_caller_check on device_profile.rs + emulator_backend.rs.
- **P-002/P24-E024** [E] class=`E/TEST` anchor=`P-24:L66-L67` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L379): INTEGRATION_SMOKE after P-24 lands with P-12/P-13/P-14: SAME journey across all three rungs on HARD Expo fixture (broken phone-only layout + control lacking a11y props); assert Rung1 layout catch, Rung2 AST click, Rung3 pinned boot + honest perception_degraded + Maestro crystallize + evidence-leg replay + correct render_source every row.
- **P-002/P24-E025** [E] class=`E/SEC` anchor=`P-24:L63-L64` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L380): REDTEAM failed: claim mobile coverage from cheap rung — render_source typed field blocks. Failed: flaky emulator reboots different image until green — snapshot hash in RRA ABORTs mismatch. Succeeded unpatched: Maestro assertVisible as verdict authority — PATCH Maestro actuation-only + evidence-leg. Deep red-team deferred.
- **P-002/P24-E026** [E] class=`E/EVIDENCE` anchor=`P-24:L69-L75` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L381): Grounding: H-14:1,7,184; Expo web SDK 54+ searched 2026-07-11; Maestro mobile-dev-inc searched 2026-07-11; CDP deviceScaleFactor in-repo capture harness; 4I/4D/4E/B-42.
- **P-002/P24-E027** [E] [SURFACE] class=`E/INT` anchor=`P-24:L77-L78` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L382): PRODUCTION_HOOK: cli device/emulator flags + task_runner device_profile + H-13 device_frame — DeviceProfile resolver + emulator backend wired into ALL THREE same-changeset (vertical slice; Rung 3 crystallize arm registers in P-14 target enum same-changeset).
- **P-002/P24-A001** [A] class=`A/DEP` anchor=`P-24:L55-L55` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L383): Maestro license + latest release + binary checksum channel (P-16 license_tag) is RECON at edit (anti-slopsquat).
- **P-002/P24-A002** [A] class=`A/INT` anchor=`P-24:L55-L55` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L384): Expo SDK data-oid tagging through Metro, uiautomator dump fidelity on RN fixture, and CDP device-metrics call sites need settling checks.
- **P-002/P24-A003** [A] class=`A/CFG` anchor=`P-24:L17-L18` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L385): Device-profile field defaults beyond schema, SSIM thresholds, and full matrix coverage enumeration are not fixed.
- **P-002/P24-A004** [A] class=`A/DEFER` anchor=`P-24:L19-L20, L81-L81` unit=`P-24` (ledger-line: DEEP-P-002.discovery.md:L386): Native Fiber-inspector click-to-edit remains registered Rung-3 follow-up without acceptance criteria here.

## Ledger P-003

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-P-003.discovery.md`

### Unit `P-3` — persona/mechanism split + hand-this-box-to-a-friend export — from P-003 (31 records)

- **P-003/P3-E001** [E] class=`E/STATUS` anchor=`P-3:L1-L5` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L34): P-3 is PLANNED Wave 3, PERSONAL-INCARNATION, lever mechanism-vs-persona-factoring, MODERATE grounding, effort M; nature is bundle=build and factoring=doctrine; depends on P-1 manifest and P-2 ports; PROOF_SPINE none (product-surface/distribution class), owner-acked 2026-07-07.
- **P-003/P3-E002** [E] class=`E/CAP` anchor=`P-3:L7-L7` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L35): Premise: untransferability is a factoring failure, not a law of nature; every built tool is factored MECHANISM (code, tests, gates) vs PERSONA (rules rows, memory namespaces, thresholds, aesthetic tokens — all DATA).
- **P-003/P3-E003** [E] class=`E/DATA` anchor=`P-3:L10-L10, L20-L22, L54-L54` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L36): AppBundle shape is `{ mechanism: (code + tests + gates), manifest (P-1 + P-2 ports), rra_receipt (L-14/G-2 lineage), provenance: G-9-redacted }`.
- **P-003/P3-E004** [E] class=`E/DATA` anchor=`P-3:L7-L11, L54-L54, L63-L63` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L37): PERSONA layer (rules rows, memory namespaces, thresholds, aesthetic tokens) is EXCLUDED from the bundle by construction and regenerated on the recipient seat.
- **P-003/P3-E005** [E] [SURFACE] class=`E/SURFACE` anchor=`P-3:L10-L10, L22-L22, L27-L27, L32-L32, L55-L55, L60-L60, L72-L72` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L38): CLI verb `elai app export <app>` writes the bundle plus a receipt.
- **P-003/P3-E006** [E] [SURFACE] class=`E/SURFACE` anchor=`P-3:L10-L10, L22-L22, L27-L27, L34-L35, L55-L55, L60-L60, L72-L72` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L39): CLI verb `elai app import <bundle>` is the Tier-1 import production caller.
- **P-003/P3-E007** [E] [SURFACE] class=`E/FLOW` anchor=`P-3:L10-L11, L15-L15, L22-L22, L34-L36, L64-L64` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L40): Import runs M-5 elai-verify against the bundle evidence, refuses on a failed receipt, then mounts the mechanism.
- **P-003/P3-E008** [E] class=`E/SEC` anchor=`P-3:L10-L11, L17-L17, L22-L22, L31-L33, L65-L65` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L41): G-9 write-time secret/PII redaction is mandatory on the export provenance leg; unredacted provenance is banned.
- **P-003/P3-E009** [E] [SURFACE] class=`E/SEC` anchor=`P-3:L10-L10, L16-L16, L23-L23` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L42): Sharing surface is admission-gated: every artifact carries a receipt and passes the same L-12/skillspector admission gates used for skills; friend-to-friend and marketplace share one gate.
- **P-003/P3-E010** [E] class=`E/LIMIT` anchor=`P-3:L11-L11, L63-L66` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L43): EXCLUDED: exporting the persona layer; an unreceipted/unredacted bundle; a bundle that mounts WITHOUT M-5 re-verification (A-import-must-verify); a per-recipient code fork (mechanism shared verbatim; only persona differs — A-5).
- **P-003/P3-E011** [E] class=`E/DEP` anchor=`P-3:L13-L19, L57-L58, L90-L91` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L44): PLAN-asserted ground: FS-1/L-12/L-13 data extension; M-5 verifier + M-6 evidence format; L-12 + skillspector admission; G-8/G-9 redaction; L-14 RRA + G-2 lineage; P-1 manifest; P-2 ports.
- **P-003/P3-E012** [E] class=`E/DEFER` anchor=`P-3:L21-L23, L28-L28, L37-L39` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L45): Tier-2 deferred: persona regeneration against importer E-5 memory + P-7 profile seam; friend-to-friend/marketplace sharing surface (lands with sharing wiring pass shared with P-2 Tier-2).
- **P-003/P3-E013** [E] class=`E/INT` anchor=`P-3:L25-L28, L54-L56` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L46): Integration targets orchestrator (or knowledge) AppBundle serializer/deserializer + export/import flow + M-5 verify call + G-9 redaction pass.
- **P-003/P3-E014** [E] class=`E/OBS` anchor=`P-3:L35-L35, L56-L56, L68-L70` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L47): Export emits journaled `app_export` SSB row; import emits `app_import` (refusal without mount, or mount on clean).
- **P-003/P3-E015** [E] class=`E/TEST` anchor=`P-3:L31-L33` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L48): M1: export writes persona-absent, G-9-redacted provenance; un-redactable secret in provenance is hard refusal.
- **P-003/P3-E016** [E] [SURFACE] class=`E/TEST` anchor=`P-3:L34-L36` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L49): M2: valid bundle re-verifies GREEN and mounts (`app_import` SSB); tampered mechanism or stale receipt is REFUSED with no mount.
- **P-003/P3-E017** [E] class=`E/TEST` anchor=`P-3:L37-L39` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L50): M3 Tier-2: after import, thresholds/rules/tokens reflect IMPORTER memory, not sender; regeneration is journaled and revertible.
- **P-003/P3-E018** [E] [SURFACE] class=`E/RECOVERY` anchor=`P-3:L41-L42` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L51): FORK: if M-5 cannot yet verify a whole-app bundle, ship Tier-1 export + import-with-receipt-presence-check (refuse absent/malformed receipt), route full evidence re-verification to Tier-2; never ship a mount that skips verification entirely.
- **P-003/P3-E019** [E] class=`E/LIMIT` anchor=`P-3:L49-L50, L66-L66` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L52): ABORT: if factoring requires per-recipient CODE changes rather than data-only persona swap, stop and flag as substrate gap (A-5 two-masters); never ship a per-recipient fork.
- **P-003/P3-E020** [E] [SURFACE] class=`E/INT` anchor=`P-3:L59-L61, L72-L72` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L53): Production reachability is live `elai app export` / `elai app import` CLI; `#[cfg(test)]` never satisfies R7. PRODUCTION_HOOK is `cli/main.rs` subcommand dispatch.
- **P-003/P3-E021** [E] class=`E/TEST` anchor=`P-3:L74-L78` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L54): VERIFY: auditor runs wiring_contract_check + production_caller_check off-agent; pass requires `app_export` on real export with persona-free G-9-redacted bundle, tampered import refused, clean import mounted.
- **P-003/P3-E022** [E] class=`E/TEST` anchor=`P-3:L80-L80` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L55): INTEGRATION_SMOKE R10: independent agent exports real app with planted secret + persona, imports on second simulated seat, tampers between export/import (must refuse), re-imports clean (must mount+verify), asserts no sender persona.
- **P-003/P3-E023** [E] class=`E/SEC` anchor=`P-3:L82-L83` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L56): REDTEAM failed attack: post-signing mechanism edit is refused by M-5 re-verify. Succeeded-then-patched attack: API-key leak via provenance, patched by mandatory G-9 export-path pass.
- **P-003/P3-E024** [E] class=`E/DEP` anchor=`P-3:L90-L91` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L57): Composes with P-1/P-2, M-5/M-6, L-12, G-8/G-9, FS-1/L-13, E-5 + P-7 (Tier-2); re-implements NONE (GD-1); P-7 owns profile engine; P-6 domain labels ride in ports.
- **P-003/P3-E025** [E] class=`E/EVIDENCE` anchor=`P-3:L85-L88` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L58): Grounding cites design digest GAP-3/T5/A-5/A-2, reddit thread archive (heynoswearing, Kind_Silver_1921, skerit), and feature-map M-5/M-6/L-12/G-8/G-9/FS-1/L-14.
- **P-003/P3-I001** [I] class=`I/STATE` anchor=`P-3:L21-L23, L34-L39` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L59): Tier-1 import mounts mechanism without persona regen; recipient runs mechanism under default/empty persona until Tier-2 lands.
- **P-003/P3-A001** [A] class=`A/DATA` anchor=`P-3:L10-L10, L54-L54` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L60): Exact AppBundle serialization version, field schema, signature envelope, and on-disk packaging are not specified beyond the structural outline.
- **P-003/P3-A002** [A] class=`A/CFG` anchor=`P-3:L47-L47, L7-L7` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L61): Which concrete files/namespaces constitute the persona exclusion set remains RECON_NEEDED.
- **P-003/P3-A003** [A] class=`A/DEP` anchor=`P-3:L41-L45` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L62): M-5 whole-bundle verification grain (changeset vs packaged artifact) is RECON_NEEDED and may force the presence-check FORK.
- **P-003/P3-A004** [A] [SURFACE] class=`A/SURFACE` anchor=`P-3:L23-L23, L27-L27` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L63): Friend-to-friend/marketplace hosting UX is deferred; no Tier-1 UI beyond CLI is named.
- **P-003/P3-C001** [C] class=`C/DEFER` anchor=`P-3:L7-L10, L21-L23` unit=`P-3` (ledger-line: DEEP-P-003.discovery.md:L64): Opening premise describes full import+regenerate flow, but REVISED scope ships Tier-1 without persona regen; regen is Tier-2.

### Unit `P-15` — verified case-graph hardening (gated agent-s memory loop + journal-row bBoN) — from P-003 (23 records)

- **P-003/P15-E001** [E] class=`E/STATUS` anchor=`P-15:L1-L5` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L74): P-15 is PLANNED Wave CU-3, COMPUTER-USE-STEAL, lever gate-the-save-not-every-run, STRONG grounding, EXTENSION of closed J-96, effort M; PROOF_SPINE feeds PS-5; execute after proof spine per A-7.
- **P-003/P15-E002** [E] class=`E/CAP` anchor=`P-15:L6-L9, L18-L21` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L75): Port agent-s S2 KnowledgeBase retrieve/distill/fuse loop onto J-96, but GATE the save behind mint-on-proof; agent-s saves every run ungated (the gap this closes).
- **P-003/P15-E003** [E] class=`E/CAP` anchor=`P-15:L18-L20, L30-L34, L41-L41` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L76): Tier-1 SHIP NOW: retrieve/distill/fuse loop with save gated by J-96 mint-on-proof; reuse-hit field on J-96 case row (net 0..+1); loop default-off behind J-96 strict reuse filter.
- **P-003/P15-E004** [E] class=`E/DEFER` anchor=`P-15:L20-L20, L31-L31` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L77): Tier-2 DEFER: bBoN behavior-narrative comparative judge over journal rows (tie-breaker); episodic/narrative two-tier split; embedding-retrieval tuning.
- **P-003/P15-E005** [E] [SURFACE] class=`E/LIMIT` anchor=`P-15:L21-L21, L39-L40` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L78): EXCLUDED: agent-s ungated KnowledgeBase commit; bBoN judge sourced from screenshots (ELAI has ground-truth journal deltas).
- **P-003/P15-E006** [E] class=`E/DEP` anchor=`P-15:L11-L16, L32-L34, L68-L68` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L79): Extends J-96 CaseNode/MintOnProof; composes K-18 recurrence-hit-rate, K-19 gate-the-save discipline, I evidence-ranked search (receipts first, narrative breaks ties), F-4 PROVEN case node.
- **P-003/P15-E007** [E] class=`E/CAP` anchor=`P-15:L29-L31` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L80): Produces `crates/orchestrator/src/case_graph/reuse_loop.rs` and `bbon_judge.rs` (bBoN module named even though Tier-2).
- **P-003/P15-E008** [E] class=`E/INT` anchor=`P-15:L35-L37, L71-L72` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L81): Production reachability: orchestrator reuse path behind J-96 execution-grounded filter; test/contracts callers never satisfy R7.
- **P-003/P15-E009** [E] [SURFACE] class=`E/SURFACE` anchor=`P-15:L43-L44` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L82): frontend_pair rides J-96 memory CanvasMode node chrome; reuse-hit surfaced on the case node (W-4).
- **P-003/P15-E010** [E] [SURFACE] class=`E/SURFACE` anchor=`P-15:L46-L46, L56-L56` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L83): CLI trigger surface: `elai case reuse --replay tests/fixtures/case/recurrence_bugclass_a.json`.
- **P-003/P15-E011** [E] [SURFACE] class=`E/EVENT` anchor=`P-15:L47-L48, L71-L72` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L84): Expected SSB: `recurrence_hit_rate_measured`; K-18 bench `elai bench case-reuse` consumes reuse-hit.
- **P-003/P15-E012** [E] class=`E/OBS` anchor=`P-15:L47-L48, L51-L52` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L85): Distill/fuse retrieves case; unproven run summary is NOT saved; reuse-hit lands on case row and K-18 rate reflects it.
- **P-003/P15-E013** [E] class=`E/FAIL` anchor=`P-15:L53-L53` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L86): FORK: if retrieved case no longer reproduces (fix rotted), route to re-exploration + re-mint; record staleness on the case row.
- **P-003/P15-E014** [E] class=`E/LIMIT` anchor=`P-15:L41-L41, L55-L55` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L87): ABORT if reuse-into-prompt fires without J-96 execution-grounded filter (default-off); un-gameable stronger-than-generator filter is DGM spine theorem.
- **P-003/P15-E015** [E] [SURFACE] class=`E/TEST` anchor=`P-15:L56-L60` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L88): VERIFY proof is off-agent recurrence_hit_rate_measured + absence of case node for unproven fixture via real `elai case reuse`; plan_gate_check + wiring_contract_check.
- **P-003/P15-E016** [E] [SURFACE] class=`E/SEC` anchor=`P-15:L61-L61` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L89): REDTEAM: poisoned case reuse defeated by mint-on-proof + strict reuse filter; bBoN narrative sway patched by scoring journal ROWS not model prose. Deep red-team deferred to execution pre-flight.
- **P-003/P15-E017** [E] class=`E/TEST` anchor=`P-15:L63-L64` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L90): INTEGRATION_SMOKE R10: hard multi-turn recurrence asserts proven fix mints, unproven mints nothing, recurrence reuses behind filter and moves K-18 rate; shares CU-track R10 cadence with P-14.
- **P-003/P15-E018** [E] class=`E/LIMIT` anchor=`P-15:L38-L42` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L91): Production bans: NO ungated save; NO screenshot bBoN; NO reuse-into-prompt without filter; NO `.unwrap()`/`.expect()`; NO blake2/sha2.
- **P-003/P15-E019** [E] class=`E/EVIDENCE` anchor=`P-15:L8-L9, L66-L67` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L92): Source: canvas-computer-use-eval ENTRY 11 agent-s arXiv:2410.08164/2504.00906/2510.02250 verified.
- **P-003/P15-I001** [I] class=`I/STATE` anchor=`P-15:L20-L20, L30-L31` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L93): `bbon_judge.rs` is listed in produces_symbols for the unit even though bBoN is Tier-2 deferred; Tier-1 may land an empty/stub path only if wiring forbids dark symbols, else the module is deferred with the feature.
- **P-003/P15-A001** [A] class=`A/CFG` anchor=`P-15:L54-L54, L19-L20` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L94): Exact retrieve/distill/fuse API shape, embedding model, tier schemas, fusion policy, and promotion/on-switch criteria for default-off remain open (RECON_NEEDED + summary A02).
- **P-003/P15-A002** [A] class=`A/DATA` anchor=`P-15:L19-L19, L53-L53` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L95): reuse-hit field type/schema beyond net 0..+1 and staleness recording fields are not fully specified.
- **P-003/P15-C001** [C] class=`C/DEFER` anchor=`P-15:L20-L20, L51-L51` unit=`P-15` (ledger-line: DEEP-P-003.discovery.md:L96): Wargame EXPECTED_OBSERVATION asserts bBoN ranks candidates, but REVISED scope defers bBoN to Tier-2; Tier-1 acceptance must not require bBoN rows.

### Unit `P-17` — provider failover semantics + honest routing economics — from P-003 (27 records)

- **P-003/P17-E001** [E] class=`E/STATUS` anchor=`P-17:L1-L6` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L106): P-17 is PLANNED Wave OC-1, OPENCLAW-STEAL, lever failover-semantics-package, STRONG grounding, EXTENSION of J-28/I-50/Q-4/B-37, effort M; PROOF_SPINE feeds PS-4.
- **P-003/P17-E002** [E] class=`E/CAP` anchor=`P-17:L6-L9, L19-L22` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L107): Port openclaw failure-semantics package into J free-API rotation pool + I-50 auth broker; never-merge billed-truth-with-estimate; honest failover preconditions PS-4 $/verified-task.
- **P-003/P17-E003** [E] class=`E/CFG` anchor=`P-17:L9-L9, L20-L20` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L108): Rate-limit cooldown ladder: 30s -> 1m -> 5m (own lane).
- **P-003/P17-E004** [E] class=`E/CFG` anchor=`P-17:L9-L9, L20-L20` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L109): Billing failure cooldown: disabled lane 5h -> 24h (own lane).
- **P-003/P17-E005** [E] class=`E/CFG` anchor=`P-17:L9-L9, L20-L20` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L110): Auth-permanent cooldown: 10-60m range (own lane).
- **P-003/P17-E006** [E] class=`E/CFG` anchor=`P-17:L9-L9, L20-L20, L40-L40` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L111): Cooldowns are MODEL-scoped so sibling models still try; also distinguish profile-scoped cooldowns.
- **P-003/P17-E007** [E] class=`E/STATE` anchor=`P-17:L9-L9, L20-L20` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L112): Session-stickiness for cache warmth: profile pinned per session; rotation only on reset/compaction/cooldown; rotation is a journaled cache-invalidation event (composes H cache-stable layout).
- **P-003/P17-E008** [E] class=`E/FLOW` anchor=`P-17:L9-L9, L20-L20` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L113): persist-override-BEFORE-retry closes the live-switch race.
- **P-003/P17-E009** [E] class=`E/SEC` anchor=`P-17:L9-L9, L20-L22, L32-L32, L41-L41, L54-L54` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L114): strict-explicit-selection: user explicit model choice fails loudly, never silently substituted; substitution is Q-4 recorded event if user opted in.
- **P-003/P17-E010** [E] class=`E/DATA` anchor=`P-17:L12-L12, L20-L22, L34-L34, L42-L42` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L115): J-28 UsageTurnEvent extended with typed billed + estimate fields end-to-end; never merged on any surface.
- **P-003/P17-E011** [E] class=`E/CAP` anchor=`P-17:L30-L32` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L116): Produces `crates/providers/src/failover.rs::FailoverPolicy` and `strict_selection.rs` explicit-selection guard.
- **P-003/P17-E012** [E] class=`E/DEP` anchor=`P-17:L11-L17, L33-L35, L67-L69` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L117): Consumes UsageTurnEvent (J-28) and AuthBroker (I-50); B-37 fallback chains consume; K-2 routing-economics bench is the PS-4 consumer.
- **P-003/P17-E013** [E] class=`E/INT` anchor=`P-17:L36-L38, L72-L73` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L118): Production path: `crates/providers/src/dispatch.rs` invokes FailoverPolicy on every provider call; test callers never satisfy R7.
- **P-003/P17-E014** [E] [SURFACE] class=`E/SURFACE` anchor=`P-17:L44-L45, L49-L49` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L119): frontend_pair: failover state + billed-vs-estimate label on usage surface (RuntimePanel / CloudPanel; W-4).
- **P-003/P17-E015** [E] [SURFACE] class=`E/SURFACE` anchor=`P-17:L47-L47` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L120): CLI/test trigger: `elai provider failover-test --fixture tests/fixtures/providers/billing_then_ratelimit.json`.
- **P-003/P17-E016** [E] class=`E/EVENT` anchor=`P-17:L48-L49, L52-L52` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L121): Expected SSB: `provider_failover_event` per event with correct lane + cooldown.
- **P-003/P17-E017** [E] class=`E/FAIL` anchor=`P-17:L40-L42, L49-L49, L52-L53` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L122): NEGATIVE fails: billing failure cools whole provider; silent substitution of explicit model; merged billed/estimate number.
- **P-003/P17-E018** [E] class=`E/FLOW` anchor=`P-17:L54-L54` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L123): FORK: if user-pinned model enters cooldown, DO NOT silently substitute — surface strict failure or Q-4-recorded swap-restart if opted in; retry only within pinned model ladder.
- **P-003/P17-E019** [E] class=`E/DEFER` anchor=`P-17:L21-L21` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L124): Tier-2: full ~140-plugin auth-profile store; auto-override reprobe-origin-every-5min loop.
- **P-003/P17-E020** [E] class=`E/LIMIT` anchor=`P-17:L22-L22` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L125): EXCLUDED vaporware: silent substitution of explicit model; merged billed/estimate number on any surface.
- **P-003/P17-E021** [E] class=`E/TEST` anchor=`P-17:L57-L61` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L126): VERIFY: off-agent provider_failover_event rows with correct lanes + separate typed billed/estimate via real dispatch; plan_gate_check + wiring_contract_check.
- **P-003/P17-E022** [E] class=`E/TEST` anchor=`P-17:L64-L65` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L127): INTEGRATION_SMOKE R10: hard multi-turn across 422 leaked-field, billing, and rate-limit; asserts per-lane cooldowns, sibling survival, no silent substitution, separate fields into K-2.
- **P-003/P17-E023** [E] class=`E/SEC` anchor=`P-17:L62-L62` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L128): REDTEAM: billing error on every model contained by lane-scoped cooldown; silent cheap-model substitution for pinned expensive model patched by strict-explicit-selection (PS-4 attribution).
- **P-003/P17-E024** [E] class=`E/LIMIT` anchor=`P-17:L43-L43, L70-L70` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L129): Production bans `.unwrap()`/`.expect()` on dispatch/failover; NO blake2/sha2; tokio::sync::Mutex in async cited as hard rule.
- **P-003/P17-A001** [A] class=`A/CFG` anchor=`P-17:L9-L9, L55-L55` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L130): Auth cooldown is a range 10-60m, not a deterministic ladder; exact status/error-code to lane mapping is RECON_NEEDED.
- **P-003/P17-A002** [A] class=`A/DATA` anchor=`P-17:L53-L53, L40-L40` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L131): Precise identity of the "billed lane" scope object (provider×profile×model keys) is not fully named beyond {provider, lane, model} counter-move text.
- **P-003/P17-A003** [A] [SURFACE] class=`A/SURFACE` anchor=`P-17:L44-L45` unit=`P-17` (ledger-line: DEEP-P-003.discovery.md:L132): Exact RuntimePanel/CloudPanel field names and layout for billed-vs-estimate are not specified.

### Unit `P-20` — context/memory lifecycle hardening (pre-compaction flush + on-demand index) — from P-003 (25 records)

- **P-003/P20-E001** [E] class=`E/STATUS` anchor=`P-20:L1-L5` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L142): P-20 is PLANNED Wave OC-1, OPENCLAW-STEAL, lever no-context-lost-uncached, STRONG grounding, EXTENSION of H-50/H-51 + H user-memory + L familiarity + H MCP defer-load, effort M; PROOF_SPINE none.
- **P-003/P20-E002** [E] class=`E/CAP` anchor=`P-20:L6-L9, L18-L19` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L143): Tier-1 SHIP: (a) pre-compaction memory-flush as ORDERED journaled step (durable notes BEFORE compaction, once per cycle); invariant "no compaction destroys context never offered to durable memory"; (b) versioned on-demand skill/rules index.
- **P-003/P20-E003** [E] class=`E/FLOW` anchor=`P-20:L9-L9` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L144): openclaw prior art: silent NO_REPLY turn writes durable notes BEFORE compaction, once per cycle, soft threshold 4000 tokens.
- **P-003/P20-E004** [E] class=`E/DATA` anchor=`P-20:L9-L9, L19-L19, L31-L31` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L145): On-demand index compact entry: name/description/path + sha256 VERSION marker (~24 tokens/skill); model reads file when needed; re-reads on version change; budget compact-degrade.
- **P-003/P20-E005** [E] class=`E/DEFER` anchor=`P-20:L20-L20` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L146): Tier-2: Dreaming promotion gates (score/recall/diversity) as prior art for H memory governance; per-provider cache-control mapping table into H caching.
- **P-003/P20-E006** [E] class=`E/CFG` anchor=`P-20:L9-L9` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L147): Deferred provider mapping prior art: Anthropic cache_control 5m/1h; OpenAI prompt_cache_key; Gemini cachedContents.
- **P-003/P20-E007** [E] class=`E/LIMIT` anchor=`P-20:L21-L21, L39-L41` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L148): EXCLUDED: second memory store (ride H two-tier; A-1/N-2); flush that fires AFTER compaction.
- **P-003/P20-E008** [E] class=`E/DEP` anchor=`P-20:L11-L16, L33-L34, L75-L75` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L149): Flush is IN-session, distinct from H-51 cross-session handoff; flush TARGET is H two-tier DurableMemory; index consumer is L familiarity; one lazy-load mechanism with H MCP defer-load.
- **P-003/P20-E009** [E] class=`E/CAP` anchor=`P-20:L29-L31` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L150): Produces `crates/orchestrator/src/context/precompact_flush.rs` and `ondemand_index.rs`.
- **P-003/P20-E010** [E] class=`E/INT` anchor=`P-20:L33-L36, L71-L72` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L151): Extends `lifecycle.rs::CompactionPass` with flush ordering — do NOT re-implement; production path is compaction invoking flush before compacting.
- **P-003/P20-E011** [E] [SURFACE] class=`E/SURFACE` anchor=`P-20:L43-L44` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L152): frontend_pair N/A: internal context machinery; no user surface beyond existing cost/memory panels (W-6 N/A justified).
- **P-003/P20-E012** [E] [SURFACE] class=`E/SURFACE` anchor=`P-20:L46-L46` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L153): Trigger command: `elai session drive-to-compaction --fixture tests/fixtures/context/long_session.json`.
- **P-003/P20-E013** [E] class=`E/EVENT` anchor=`P-20:L47-L48, L51-L51` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L154): Expected SSB: `memory_flush_event` lands BEFORE the compaction row (ordered), carrying durable notes.
- **P-003/P20-E014** [E] class=`E/OBS` anchor=`P-20:L48-L48, L51-L52` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L155): Skill index shows only compact entries until skill needed; file read once (trace); NEGATIVE: compaction before flush or full skill body preloaded FAILS.
- **P-003/P20-E015** [E] class=`E/LIMIT` anchor=`P-20:L38-L42` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L156): stub_ban: NO compaction before flush; NO full skill body in prompt before invoke; NO second memory store; NO `.unwrap()`/`.expect()`; NO blake2/sha2 (blake3/sha256 for version marker per source contract).
- **P-003/P20-E016** [E] class=`E/TEST` anchor=`P-20:L56-L60` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L157): VERIFY: off-agent memory_flush_event ordered before compaction on real session compaction path; plan_gate_check + wiring_contract_check.
- **P-003/P20-E017** [E] class=`E/TEST` anchor=`P-20:L63-L64` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L158): INTEGRATION_SMOKE R10: long multi-turn past threshold TWICE on hard tool-heavy history; each compaction preceded by flush; no tool-call/result pair split; on-demand index loads body only on first invocation; composes with P-19 compaction split-rule.
- **P-003/P20-E018** [E] class=`E/SEC` anchor=`P-20:L61-L61` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L159): REDTEAM: silent late-context loss defeated by ordered flush; huge skill corpus prompt bloat patched by ~24 tokens/skill until invocation.
- **P-003/P20-E019** [E] class=`E/ABORT` anchor=`P-20:L55-L55` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L160): ABORT if flush would require a SECOND durable memory store.
- **P-003/P20-E020** [E] class=`E/EVIDENCE` anchor=`P-20:L9-L9, L66-L69` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L161): Grounding: openclaw-vs-elai-comparison 2.6/2.7/2.8 steals S3/S7/S16/S18; H-50/H-51, H user-memory, L familiarity, H MCP defer-load PLAN-asserted.
- **P-003/P20-I001** [I] [SURFACE] class=`I/CFG` anchor=`P-20:L9-L9, L18-L19` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L162): Soft threshold 4000 tokens is openclaw prior art cited in context; whether ELAI Tier-1 adopts that numeric threshold or reuses H-51 thresholds is not locked in REVISED scope.
- **P-003/P20-A001** [A] class=`A/FLOW` anchor=`P-20:L18-L19, L9-L9` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L163): Memory note selection policy, what counts as "offered", and flush failure/retry behavior are unspecified.
- **P-003/P20-A002** [A] class=`A/CFG` anchor=`P-20:L19-L19` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L164): Compact-degrade levels and token-measurement method are absent.
- **P-003/P20-A003** [A] class=`A/DEFER` anchor=`P-20:L20-L20, L9-L9` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L165): Dreaming score/recall/diversity thresholds and provider cache TTLs remain prior art only until Tier-2.
- **P-003/P20-C001** [C] class=`C/DEFER` anchor=`P-20:L6-L6, L18-L20` unit=`P-20` (ledger-line: DEEP-P-003.discovery.md:L166): Premise lists Dreaming and per-provider cache-control as unit goals, but REVISED scope defers both to Tier-2; Tier-1 is flush + on-demand index only.

### Unit `P-22` — control-plane / multi-agent / channel hardening — from P-003 (27 records)

- **P-003/P22-E001** [E] class=`E/STATUS` anchor=`P-22:L1-L6` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L176): P-22 is PLANNED Wave OC-3, OPENCLAW-STEAL, lever fenced-returns-signed-ingress, STRONG grounding, EXTENSION of 3C+G-1+J+P-10+M+4C, effort M-L; PROOF_SPINE none.
- **P-003/P22-E002** [E] class=`E/CAP` anchor=`P-22:L19-L20, L31-L32` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L177): Tier-1 SHIP: (a) child-output-as-fenced-evidence (subagent return is DATA, structurally unable to carry instructions; composes G-1 taint source) + depth-scoped tool-strip defaults for 3C.
- **P-003/P22-E003** [E] class=`E/CFG` anchor=`P-22:L9-L9, L20-L20, L32-L32, L41-L41` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L178): Depth-scoped tool policy: subagents lose gateway/cron/messaging; leaves lose spawn.
- **P-003/P22-E004** [E] class=`E/SEC` anchor=`P-22:L9-L9, L20-L20, L31-L31, L40-L40, L52-L53` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L179): Subagent return is fenced taint-source; instructions in it are inert; cannot steer parent.
- **P-003/P22-E005** [E] class=`E/DEFER` anchor=`P-22:L20-L21, L9-L9` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L180): Tier-2 (b): 9-step deterministic channel-binding precedence + pairing-code UX + dmScope isolation for P-10 + front-door.
- **P-003/P22-E006** [E] class=`E/DEFER` anchor=`P-22:L21-L21, L9-L9` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L181): Tier-2 (c): challenge-signed device pairing + fail-closed unknown-event broadcast gating + mandatory idempotency keys for J control plane + M seats.
- **P-003/P22-E007** [E] class=`E/DEFER` anchor=`P-22:L21-L21, L9-L9` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L182): Tier-2 (d): trusted-policy hook tier ordered before third-party + manifest-validated-without-execution for 4C.
- **P-003/P22-E008** [E] class=`E/CFG` anchor=`P-22:L9-L9` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L183): openclaw pairing prior art (Tier-2): 8-char codes, 1h expiry, <=3 pending; dmScope isolation; DEVICE_AUTH_* error codes.
- **P-003/P22-E009** [E] class=`E/LIMIT` anchor=`P-22:L22-L22, L43-L43` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L184): EXCLUDED: second provenance scale (N-2/A-1); shipping ELAI's own ~30 channels (P owns FORMAT, not ecosystem).
- **P-003/P22-E010** [E] class=`E/CAP` anchor=`P-22:L30-L32` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L185): Produces `crates/orchestrator/src/agents/fenced_return.rs` and `tool_strip.rs`.
- **P-003/P22-E011** [E] class=`E/DEP` anchor=`P-22:L11-L17, L33-L35, L67-L69` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L186): Consumes AgentSpawn (3C) and TaintSource (G-1); targets also J control plane, P-10 ingress, M seats, 4C hooks.
- **P-003/P22-E012** [E] class=`E/INT` anchor=`P-22:L36-L38, L72-L73` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L187): Production path: `agents/spawn.rs` applies tool_strip and wraps every return via fenced_return before parent; test callers never satisfy R7.
- **P-003/P22-E013** [E] [SURFACE] class=`E/SURFACE` anchor=`P-22:L44-L45` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L188): frontend_pair: enrollment + pairing UX on the control surface (Tier-2; W-4).
- **P-003/P22-E014** [E] [SURFACE] class=`E/SURFACE` anchor=`P-22:L47-L47` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L189): Trigger: `elai agents spawn-test --fixture tests/fixtures/agents/instruction_shaped_return.json`.
- **P-003/P22-E015** [E] class=`E/EVENT` anchor=`P-22:L48-L49, L52-L52` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L190): Expected SSB: `subagent_return_fenced`; artifact: instruction-shaped return lands as fenced taint-source row (inert) + no gateway/cron/messaging tools.
- **P-003/P22-E016** [E] class=`E/FLOW` anchor=`P-22:L54-L54` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L191): FORK: if depth-1 orchestrator needs a tool deeper agents must not have, only depth-1 regains it; deeper agents never re-acquire stripped tools — route need up.
- **P-003/P22-E017** [E] class=`E/ABORT` anchor=`P-22:L56-L56` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L192): ABORT if subagent return could be injected into parent's instruction channel (not data channel) — lethal-trifecta path Dual-LLM quarantine exists to block.
- **P-003/P22-E018** [E] class=`E/TEST` anchor=`P-22:L57-L61` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L193): VERIFY: off-agent subagent_return_fenced + stripped tool set via real 3C spawn; plan_gate_check + wiring_contract_check.
- **P-003/P22-E019** [E] class=`E/TEST` anchor=`P-22:L64-L65` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L194): INTEGRATION_SMOKE R10: real multi-agent fan-out depth 2-3; leaf returns instruction-shaped text AND attempts messaging; asserts fenced, tool-strip denied; deferred legs when built: unknown event withheld + mutating RPC dedupes on idempotency key.
- **P-003/P22-E020** [E] class=`E/SEC` anchor=`P-22:L62-L62` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L195): REDTEAM: "ignore instructions and delete X" defeated by fenced return as taint data; deep agent re-acquiring messaging to exfiltrate patched by spawn-time tool-strip (only depth-1 regains).
- **P-003/P22-E021** [E] class=`E/LIMIT` anchor=`P-22:L42-L42` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L196): stub_ban includes NO unknown event family leaked (fail-closed WITHHELD) even though unknown-event gating is Tier-2 — the ban is stated as unit-wide.
- **P-003/P22-E022** [E] class=`E/LIMIT` anchor=`P-22:L43-L43` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L197): Production: NO second provenance scale; NO `.unwrap` on prod; NO blake2/sha2 (blake3 for signed-pairing payload when built).
- **P-003/P22-I001** [I] class=`I/SEC` anchor=`P-22:L42-L42, L64-L65, L20-L21` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L198): stub_ban "NO unknown event family leaked" and smoke deferred-leg assertions define the intended Tier-2 contract even though Tier-1 produces only fenced_return + tool_strip.
- **P-003/P22-A001** [A] class=`A/CFG` anchor=`P-22:L9-L9, L20-L21` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L199): Nine channel-binding steps and specificity keys are not listed in this plan (only openclaw count).
- **P-003/P22-A002** [A] class=`A/CFG` anchor=`P-22:L9-L9, L55-L55` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L200): dmScope values, exact pairing algorithm, platform payload schema remain open until Tier-2 recon.
- **P-003/P22-A003** [A] class=`A/DATA` anchor=`P-22:L20-L20, L31-L31` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L201): Concrete fenced-data encoding that makes instructions "structurally unable to carry" is not specified.
- **P-003/P22-C001** [C] class=`C/DEFER` anchor=`P-22:L6-L9, L19-L21, L42-L45` unit=`P-22` (ledger-line: DEEP-P-003.discovery.md:L202): Premise and stub_ban enumerate channel-binding, signed pairing, unknown-event gating, idempotency, and trusted-policy hooks, but REVISED scope ships only fenced returns + tool-strip in Tier-1.

### Unit `P-23` — journaled co-browse lane — from P-003 (31 records)

- **P-003/P23-E001** [E] [SURFACE] class=`E/STATUS` anchor=`P-23:L1-L6` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L212): P-23 is PLANNED Wave CU-4, COMPUTER-USE-STEAL, lever co-browse-with-receipts, STRONG comparator / MODERATE ELAI anchors, EXTENSION of H-15+E-14, effort M; depends H-13,H-15,E-14,P-13; PROOF_SPINE none (product); owner-commissioned 2026-07-11.
- **P-003/P23-E002** [E] class=`E/CAP` anchor=`P-23:L6-L9` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L213): Closes ENTRY 14 design hole: general-purpose co-browse where unjournaled input path is structurally impossible; comparator ships pane without receipts.
- **P-003/P23-E003** [E] [SURFACE] class=`E/SURFACE` anchor=`P-23:L19-L19, L48-L49` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L214): Tier-1 READ lane: canvas browse node rendering page EXTRACTION — E-14 substrate nav + AXTree + screenshot, read-only, content taint-tagged web at ingest, deixis via H-15; minimal ceremony (P-5).
- **P-003/P23-E004** [E] [SURFACE] class=`E/SURFACE` anchor=`P-23:L20-L20, L48-L49` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L215): Tier-2 CO-BROWSE lane: orchestrator-owned Playwright/CDP browser; CDP screencast frames stream into browse node — THE PANE IS DUMB GLASS.
- **P-003/P23-E005** [E] class=`E/CAP` anchor=`P-23:L20-L20, L34-L34, L45-L45` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L216): One InputProxy is the only holder of the engine input session; every nav/click/keystroke — user (forwarded from pane) or model (P-13 id-pointing; coordinate fallback under P-12 frame-binding) — lands actor-tagged cobrowse_input SSB row BEFORE dispatch.
- **P-003/P23-E006** [E] class=`E/DET` anchor=`P-23:L20-L20, L34-L34, L45-L45, L57-L57` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L217): Journal completeness is structural, not policy: no second input path; CDP input session handle private to proxy (type-level).
- **P-003/P23-E007** [E] class=`E/CFG` anchor=`P-23:L21-L21` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L218): Session persistence typed: default = ephemeral profile destroyed on close; `--persist <name>` = named profile in P-10 credential vault; choice is a field on cobrowse_session row.
- **P-003/P23-E008** [E] class=`E/PRIV` anchor=`P-23:L22-L22, L67-L67` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L219): Egress gating: user-typed text bound for web form passes P-6 privacy lattice + domain-first-seen warning row before dispatch.
- **P-003/P23-E009** [E] class=`E/SEC` anchor=`P-23:L23-L23, L67-L67` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L220): Authority: everything read in pane is G-1 taint source; browse content can never authorize a mutating lane.
- **P-003/P23-E010** [E] class=`E/LIMIT` anchor=`P-23:L24-L24, L58-L58` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L221): EXCLUDED as mechanism: native embedded webview as interactive surface (Tauri multiwebview unstable cargo flag, open bugs #10131/#10420, remote DOM in trusted shell, unjournaled native input). Survives only as FORK option with InputProxy still interposed.
- **P-003/P23-E011** [E] class=`E/CAP` anchor=`P-23:L32-L34` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L222): Produces `crates/orchestrator/src/cobrowse/mod.rs` (CoBrowseLane: session mgr + screencast pump + profile custody hook) and `input_proxy.rs` (InputProxy journal-then-dispatch).
- **P-003/P23-E012** [E] class=`E/DEP` anchor=`P-23:L12-L16, L35-L38, L75-L75` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L223): Consumes E-14 browser_e2e engine (if unexecuted, this unit vendors substrate and E-14 consumes it — threaded, not second engine); P-13 PerceptionScaffold; P-12 FrameBinding; G-1 TaintSource; G-2 ActionStream.
- **P-003/P23-E013** [E] [SURFACE] class=`E/SURFACE` anchor=`P-23:L40-L40, L51-L51, L78-L79` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L224): CLI: `elai browse <url>` production CLI.
- **P-003/P23-E014** [E] [SURFACE] class=`E/SURFACE` anchor=`P-23:L41-L41, L78-L79` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L225): Front-door router lane table: "pull up / open <url>" resolves to browse lane.
- **P-003/P23-E015** [E] [SURFACE] class=`E/SURFACE` anchor=`P-23:L42-L42, L78-L79` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L226): H-13 canvas dispatcher: browse node view_type + forwarded pane input.
- **P-003/P23-E016** [E] [SURFACE] class=`E/SURFACE` anchor=`P-23:L48-L49` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L227): frontend_pair: canvas browse node screencast (Tier-2) / extraction (Tier-1) + actor-colored input trail so user SEES the journal as it lands.
- **P-003/P23-E017** [E] class=`E/EVENT` anchor=`P-23:L52-L53, L56-L56` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L228): Expected SSB: `cobrowse_session` {url, profile: ephemeral\|vault:<name>}; every input has PRIOR actor-tagged `cobrowse_input` {actor, target: element-id\|coord+frame_hash}.
- **P-003/P23-E018** [E] class=`E/TEST` anchor=`P-23:L53-L53, L56-L56` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L229): Journal-then-dispatch ordering checked off-agent against engine CDP log; NEGATIVE: input without prior row, model coordinate on stale frame hash, ephemeral session leaving profile bytes FAILS.
- **P-003/P23-E019** [E] class=`E/CFG` anchor=`P-23:L58-L58` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L230): FORK latency tripline: if co-browse input-to-paint p95 exceeds 250ms on local loop (measured), evaluate native Tauri multiwebview ONLY once multiwebview left unstable flag AND InputProxy still interposed; if engine lacks CDP screencast, fall back to timed captureScreenshot loop (degraded FPS, identical receipts).
- **P-003/P23-E020** [E] class=`E/ABORT` anchor=`P-23:L60-L60, L45-L46` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L231): ABORT: any input path bypassing InputProxy (P0); persistent profile outside vault custody; profile isolation failure (session seeing personal browser state).
- **P-003/P23-E021** [E] [SURFACE] class=`E/TEST` anchor=`P-23:L61-L66` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L232): VERIFY via real `elai browse`: plan_gate_check + wiring_contract_check + production_caller_check on cobrowse modules.
- **P-003/P23-E022** [E] [SURFACE] class=`E/SEC` anchor=`P-23:L67-L67` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L233): REDTEAM failed: hostile fake ELAI approval dialog inside pane (UI redress) — harness chrome never in screencast; pane click only becomes cobrowse_input, never harness command. Failed: page text instructing destructive click — G-1 taint blocks untrusted->destructive sink. Succeeded unpatched then patched: user types secret into phishing form — egress passes P-6 + domain-first-seen warning BEFORE dispatch.
- **P-003/P23-E023** [E] class=`E/TEST` anchor=`P-23:L69-L70` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L234): INTEGRATION_SMOKE after P-23+P-13: hard docs site + hostile fixture, multi-step interleaved user+model long enough for mid-run compaction; asserts prior rows, injection never reaches mutating lane, zero ephemeral bytes, P-14 crystallize mints RRA-bound replay.
- **P-003/P23-E024** [E] class=`E/DEP` anchor=`P-23:L81-L82, L75-L75` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L235): Pairs with P-13 pointing, P-14 crystallize, P-16 catalog; reuses P-6, P-10, O-2 per files.
- **P-003/P23-E025** [E] [SURFACE] class=`E/INT` anchor=`P-23:L78-L79` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L236): Vertical slice: CoBrowseLane/InputProxy wired into ALL THREE of CLI, front-door, H-13 canvas in this changeset.
- **P-003/P23-E026** [E] [SURFACE] class=`E/LIMIT` anchor=`P-23:L47-L47, L76-L76` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L237): Production: NO remote DOM inside trusted dashboard webview; NO `.unwrap()`/`.expect()`; blake3 for frame hashes; anti-slopsquat on browser-driver crate; bun-not-node if capture sidecar is JS.
- **P-003/P23-E027** [E] class=`E/CAP` anchor=`P-23:L15-L15, L70-L70` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L238): Co-browse exploration can mint deterministic replay script RRA-bound via P-14.
- **P-003/P23-I001** [I] class=`I/STATE` anchor=`P-23:L13-L13, L19-L20, L48-L49` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L239): Browse node is an H-15 view_type; Tier-1 and Tier-2 share the node with different render modes (extraction vs screencast).
- **P-003/P23-A001** [A] class=`A/CFG` anchor=`P-23:L19-L24` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L240): Cookie/download/upload/pop-up/permission dialogs and multi-tab behavior are not specified.
- **P-003/P23-A002** [A] class=`A/DEP` anchor=`P-23:L21-L21, L16-L16` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L241): P-10 vault custody details for named profiles depend on P-10 which is absent from P-summary sections.
- **P-003/P23-A003** [A] class=`A/INT` anchor=`P-23:L59-L59` unit=`P-23` (ledger-line: DEEP-P-003.discovery.md:L242): Rust-native driver vs bun/Playwright capture sidecar choice is RECON_NEEDED at edit.

### Unit `P-25` — exogenous-injection + evidence-coupled-revision creativity falsifier (B-EXO-0) — from P-003 (61 records)

- **P-003/P25-E001** [E] class=`E/STATUS` anchor=`P-25:L1-L9` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L252): P-25 is PLANNING CANDIDATE (V4 candidate-to-core binding repair); no M1/source/test/seal/smoke/benchmark authorization until fresh independent V4 PASS; track RESEARCH-CONDITIONED-CREATIVITY; lever deterministic-exogenous-injection; effort M; PROOF_SPINE none.
- **P-003/P25-E002** [E] class=`E/STATUS` anchor=`P-25:L6-L6, L30-L30` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L253): Authority: `P25-EVAL-TRUST-V4` + `P25-CANDIDATE-CORE-BINDING-V1` require one fresh independent PASS as a set; V2/V3 PASSes are historical only; RECON-2 import-map must be re-migrated and re-audited before controlling M3.
- **P-003/P25-E003** [E] class=`E/STATUS` anchor=`P-25:L11-L11, L15-L21` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L254): Three creativity controllers dead: B-0 ProbeUtility, B-3 FDL, Q-10 B-CREATIVITY-0 (~8.8x cost); surviving measured lever is exogenous grounded info (pilot P-A 3.5x real anchors).
- **P-003/P25-E004** [E] class=`E/CAP` anchor=`P-25:L11-L11, L32-L36` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L255): B-EXO-0 tests TWO preregistered families on one frozen open-world corpus: FAMILY A injection (same single DIRECT call + deterministic verified artifacts, zero extra model calls, hard cost cap); FAMILY B evidence-coupled revision (one revision call fed public triage kill-reasons vs self-revision control).
- **P-003/P25-E005** [E] [SURFACE] class=`E/DATA` anchor=`P-25:L32-L36, L26-L26` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L256): K=30 paired real ELAI improvement fixtures; no synthetic tier; B-CREATIVITY-1 corpus design reused under own preregistration (prereg 2026-07-12 §6.1).
- **P-003/P25-E006** [E] class=`E/CFG` anchor=`P-25:L36-L36` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L257): Six arms, same model/candidate/token/parser/evaluator table; arm order seeded per task; generator never sees arm names; every completion becomes exactly 6 `CandidateSlotV1` records.
- **P-003/P25-E007** [E] class=`E/CAP` anchor=`P-25:L38-L38` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L258): Arm 1 DIRECT — target-only single self-coordinating call (measured champion baseline).
- **P-003/P25-E008** [E] class=`E/CAP` anchor=`P-25:L39-L39` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L259): Arm 2 EXO_RELEVANT (FAMILY A treatment) — DIRECT + k (frozen, default 3) verified target-relevant artifacts with provenance headers: code slice (elai.map), measured row (bench/journal/feature-map), VerifiedCitation (else second code slice); deterministic seeded model-free retrieval sealed in manifest.
- **P-003/P25-E009** [E] class=`E/CAP` anchor=`P-25:L40-L40` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L260): Arm 3 EXO_SHUFFLED — same count/envelope, artifacts from DIFFERENT fixture via frozen no-fixed-point permutation STRATIFIED across subsystems; RetrievalClassifierV1 rederives labels; same-subsystem-collision count must be exactly zero.
- **P-003/P25-E010** [E] class=`E/CAP` anchor=`P-25:L41-L41` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L261): Arm 4 PAD_MATCHED — DIRECT + neutral envelope_pad to same frozen model-free envelope; order (EXO_RELEVANT,EXO_SHUFFLED,PAD_MATCHED); count `(len(exact_utf8_prompt_bytes)+2)//4`; exact spread <= 1/50.
- **P-003/P25-E011** [E] class=`E/CAP` anchor=`P-25:L42-L42` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L262): Arm 5 TRIAGE_REVISE (FAMILY B treatment) — DIRECT then public triage over VALID slots via pinned PublicTriageContextV1 (demarcation+anchor+composition; ledger EXCLUDED); TriageReceiptV1; RevisionEvidenceV1 ALIGNED; one revision call emits final 6-card set; SCORED_REVISED points back to RevisionCallInputV1.
- **P-003/P25-E012** [E] class=`E/CAP` anchor=`P-25:L43-L43` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L263): Arm 6 SHUFFLED_REVISE (FAMILY B control) — identical two-call structure/template but RevisionEvidenceV1.mode=SHUFFLED with M1-seeded no-fixed-point C1..C6 permutation.
- **P-003/P25-E013** [E] class=`E/DATA` anchor=`P-25:L42-L42` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L264): exo-card V2 grammar must emit in order: id, mechanism_family, target_area, target_units, expected_effect, falsifier, and canonical one-line strict-object payload_json; public triage receives no payload or source span.
- **P-003/P25-E014** [E] class=`E/OBS` anchor=`P-25:L45-L45` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L265): Co-primary P1 `execution_validated_mechanism_yield` = count of six scored-role slots with valid card/payload, public triage survive, candidate-bound trusted-core PASS, deduped by MechanismIdentityV1={candidate_payload_raw_sha256}.
- **P-003/P25-E015** [E] class=`E/OBS` anchor=`P-25:L45-L45` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L266): Co-primary P2 `best_target_value_delta` = max candidate value among six scored slots minus max DIRECT value for same task; INVALID=0/1.
- **P-003/P25-E016** [E] class=`E/DET` anchor=`P-25:L45-L45` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L267): Every P1/P2/G1/G2/G3 paired result uses exactly 5,000 bootstrap samples and M1-frozen seed `20260712`; mixed/reseeded rows invalid. Revision-source sets never enter P1/P2.
- **P-003/P25-E017** [E] class=`E/TEST` anchor=`P-25:L47-L47, L49-L49` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L268): G1: real-anchor-rate non-inferiority vs DIRECT (>= -0.05 one-sided, per family). G2: grounding/ScientificReceiptV1 validity non-inferiority.
- **P-003/P25-E018** [E] class=`E/TEST` anchor=`P-25:L47-L47, L49-L49` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L269): G3-A FAMILY A hard cost cap: EXO_RELEVANT sealed call/token accounting <= 1.3x DIRECT (ratios 13/10); zero preprocessing model calls.
- **P-003/P25-E019** [E] class=`E/TEST` anchor=`P-25:L47-L47, L49-L49` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L270): G3-B FAMILY B: TRIAGE_REVISE <= 2.5x DIRECT (ratios 5/2) AND cost-normalized P1 (yield per token) non-inferior to DIRECT (>= -0.05).
- **P-003/P25-E020** [E] class=`E/TEST` anchor=`P-25:L47-L47, L49-L49` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L271): G4: zero hidden_answer/leakage/manifest_binding/token_envelope violations; revision inputs scanned like prompts.
- **P-003/P25-E021** [E] class=`E/TEST` anchor=`P-25:L51-L52` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L272): FAMILY A SUPPORTED requires P1 and P2 CI95 lower bounds > 0 for EXO_RELEVANT minus EACH of DIRECT, EXO_SHUFFLED, PAD_MATCHED; all G1/G2/G3-A/G4 pass.
- **P-003/P25-E022** [E] class=`E/TEST` anchor=`P-25:L53-L54` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L273): FAMILY B SUPPORTED requires P1 and P2 CI95 lower bounds > 0 for TRIAGE_REVISE minus EACH of DIRECT and SHUFFLED_REVISE; all G1/G2/G3-B/G4 pass.
- **P-003/P25-E023** [E] class=`E/LIMIT` anchor=`P-25:L45-L45, L55-L55` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L274): No combined "creativity supported" claim; multiplicity declared under both-families-null. Diversity/judge preference can never promote.
- **P-003/P25-E024** [E] class=`E/DEFER` anchor=`P-25:L57-L57` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L275): SUPPORTED family -> separate Tier-2 adoption unit only for THAT mechanism (A: exo-inject in improvement-conjecture stage-1 + paper-to-plan seam; B: triage-feedback revision leg). BOTH NULL/REFUTED -> creativity line CLOSES entirely. No B-EXO-1/-2 pre-authorized.
- **P-003/P25-E025** [E] class=`E/CFG` anchor=`P-25:L59-L59` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L276): Cost: arms 1-4 = 120 calls; arms 5-6 = 120 calls; total 240 M5 generation + 8 M4 smoke = 248; retrieval/triage/pad/preflight/verify add zero model calls.
- **P-003/P25-E026** [E] class=`E/LIMIT` anchor=`P-25:L61-L65, P-25:L134-L136` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L277): Integration: .claude/tools only; .elai_cc NOTHING (no Rust/skill/runtime); frontend_pair N/A offline harness; conditional skills only post-SUPPORTED separate unit.
- **P-003/P25-E027** [E] class=`E/FLOW` anchor=`P-25:L69-L69` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L278): Normative order: M0 trust-substrate build/audit -> M1 -> M3 -> M2 draft data -> semantic qualification -> deterministic hidden generation -> sealing/freeze -> M4 -> M5. Numeric stage names do not authorize M2 before M3.
- **P-003/P25-E028** [E] class=`E/CAP` anchor=`P-25:L71-L71` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L279): M0 implements exactly exo_records.py, exo_trust_records.py, exo_canonical.py, exo_trust.py, exo_curator.py, tests/test_exo_trust_v4.py under fixture blindness; every source < 1000 lines; independent audit PASS before M1; N1/N2/N3/N14/N15 immutable after M1 root.
- **P-003/P25-E029** [E] class=`E/CAP` anchor=`P-25:L73-L73` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L280): M1 freezes all six arms, ArmOrderSetV1, TaskExecutionLogV1 eight-row expansion, endpoint http://127.0.0.1:8810/v1/messages, model kimi-k2-7, max_tokens=1400, timeout 240000 ms, one-POST/zero-tool/no-cache/no-retry, K=30, all schemas/hash domains/gates/bans.
- **P-003/P25-E030** [E] class=`E/DATA` anchor=`P-25:L75-L75` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L281): M1 freezes P25-REQUEST-JSON-V1 sorted compact three-key JSON + LF; P25_RESPONSE_MAX_BYTES=1048576; P25_RESPONSE_READ_CHUNK_BYTES=65536; NFC waiver only at messages[0].content.
- **P-003/P25-E031** [E] class=`E/FLOW` anchor=`P-25:L75-L75` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L282): CompletionArchiveSink: pre-POST single-use ExecutionPermit; stage prompt/request/response; canonicalize CompletionRecordV1; independently rehash; atomic promote; failure discards; TaskExecutionLogV1 only after exact eight rows.
- **P-003/P25-E032** [E] class=`E/CAP` anchor=`P-25:L79-L86` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L283): M3-A..F six cumulative batches (analysis, cores/retrieval/seal/qualification, public triage/prompts, completion transport, pool/runner, independent verifier); each <=5 files, <=3 deliverables, independent gate; no M0 byte edit.
- **P-003/P25-E033** [E] class=`E/CAP` anchor=`P-25:L82-L82, P-25:L61-L61` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L284): Four trusted cores: grep_presence_core, parser_fixture_core, bounded_simulation_core, benchmark_delta_core; dispatch_core indexes frozen table by fixture kind only; zero model/filesystem/process capability inside cores.
- **P-003/P25-E034** [E] class=`E/TEST` anchor=`P-25:L88-L88, P-25:L510-L512` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L285): Final fixture-blind M3 freeze after all six batch PASS: full Tier-0 + M0 trust + six M3 modules + A1-A8 + schema/wiring + python_wire_check 0 DARK + maintainability 0 BLOCK; pins PublicTriageContextV1 map rows; forms M3FreezePayloadV1/RecordV1/anchor/locator.
- **P-003/P25-E035** [E] class=`E/FLOW` anchor=`P-25:L94-L96` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L286): M2 prepare-corpus after M3 anchor: five-stratum seeded selection; FixtureDraftV1; M2AttemptLedgerV1; 30 terminal ELIGIBLE; >=20 executable micro-experiments; SealedCorpusV1 + RetrievalAssignmentV1 + ArmOrderSetV1; P25RunConfigV1 with external expected digest.
- **P-003/P25-E036** [E] class=`E/FAIL` anchor=`P-25:L96-L96, P-25:L108-L108` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L287): If frozen pool cannot yield 30 ELIGIBLE or no bounded_simulation member: POSTPONED_INSUFFICIENT_EXECUTABLE_POOL; never lower K or substitute judge.
- **P-003/P25-E037** [E] [SURFACE] class=`E/SURFACE` anchor=`P-25:L98-L100, P-25:L356-L357` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L288): M4 CLI: `python3 .claude/tools/exo_injection_ab.py run-smoke --repo-root --run-config --expected-run-config-sha256 --out`; selects lowest selection_slot terminal-ELIGIBLE bounded_simulation; eight-call sequence; publishes P25SmokeReportV1 only (no K=30 decision).
- **P-003/P25-E038** [E] [SURFACE] class=`E/SURFACE` anchor=`P-25:L102-L104, P-25:L357-L357` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L289): M5 CLI: `python3 .claude/tools/exo_injection_ab.py run-scored` same trust inputs over full K=30; writes family verdict and closes out.
- **P-003/P25-E039** [E] [SURFACE] class=`E/SURFACE` anchor=`P-25:L358-L359, P-25:L514-L514` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L290): Independent checker: `python3 .claude/tools/exo_creativity_artifact_check.py --verify-p25-smoke` and `--verify-p25-all` with same three trust inputs; PASS = exit 0 and 0 violations.
- **P-003/P25-E040** [E] [SURFACE] class=`E/SURFACE` anchor=`P-25:L341-L348` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L291): Curator CLIs under exo_injection_ab.py: curate-m0-file-set, curate-m1-artifact, curate-m1-anchor, lock-m1, curate-m3-payload, curate-m3-freeze, curate-m3-anchor, lock-m3 (each with exact flags).
- **P-003/P25-E041** [E] class=`E/CAP` anchor=`P-25:L138-L302` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L292): Produces large closed symbol set including FileRowV1/TrustRootV2/M1*/M3* anchors, TaskExecutionLogV1, CandidateCardV2, four CandidateTaskView/Payload variants, RetrievalClassifierV1/CatalogV1, ExactCompletionClient, decide_family_a/b, verify_p25_* suite, prepare_pool/prepare_corpus, etc. (full produces_symbols list).
- **P-003/P25-E042** [E] class=`E/DEP` anchor=`P-25:L303-L322, P-25:L61-L61` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L293): Consumes P25-EVAL-TRUST-V4, P25-CANDIDATE-CORE-BINDING-V1, RECON-2 N1-N16, research_creativity_common paired bootstrap, conjecture_triage_core six functions, elai.map, ELAI-FEATURE-MAP, b0v3 report, VerifiedCitation, Q-10 runner/checker/tests as UNCHANGED references (P-25 imports none of Q-10 runner).
- **P-003/P25-E043** [E] class=`E/LIMIT` anchor=`P-25:L134-L136` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L294): ABORT list includes: model call in retrieval/triage/packetizer; analogy/relational structure; LLM judge in primary gate; growing conjecture_research_coupling_ab.py past 1000 lines; any .elai_cc/skill/runtime edit before SUPPORTED; lowering K or 1.3x/2.5x caps after fixture inspection; continuing after both families NULL/REFUTED; fusing novelty/grounding/value/cost; hidden-side into revision (G4 INVALID); more than ONE revision round; fixture/candidate naming evaluator behavior; M2 before M3; wall-clock in scientific scores; generic BLAKE3/local receipts/mutable triage/historical hashes as trust authority.
- **P-003/P25-E044** [E] class=`E/LIMIT` anchor=`P-25:L361-L499` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L295): Extensive stub_ban rejects caller injection of paths/seeds/clients/orders/counts/layouts/kinds, packetizer, analogy fields, second revision, wall-clock scoring, hidden into revision, .elai_cc paths, etc.
- **P-003/P25-E045** [E] class=`E/OBS` anchor=`P-25:L501-L501` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L296): Artifact layout: content-addressed records/<schema_id>/<sha256>.json and raw/<origin_kind>/<sha256>.bin; singleton report branches; out_dir published by no-overwrite atomic rename from reserved `<out_dir.name>.p25-stage`.
- **P-003/P25-E046** [E] class=`E/DET` anchor=`P-25:L28-L28` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L297): Bridge lane kimi-k2-7 via devin-proxy 127.0.0.1:8810 FREE; usage.input_tokens is chars/4 estimate — P-25 freezes nearest-quarter-byte proxy over exact archived UTF-8 prompt bytes, no preprocessing meter call.
- **P-003/P25-E047** [E] class=`E/FAIL` anchor=`P-25:L110-L110` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L298): FORK: treatment beats DIRECT but not EXO_SHUFFLED -> NULL by joint gate; must name "real text helps, relevance does not" pattern.
- **P-003/P25-E048** [E] class=`E/FAIL` anchor=`P-25:L112-L112` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L299): FORK: bridge unavailable (3-step health fails) -> documented alternate FREE lane with prereg version bump; never silent model swap.
- **P-003/P25-E049** [E] class=`E/FAIL` anchor=`P-25:L114-L114` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L300): FORK: core cannot close in-process without fs/process/cargo -> M3 TCB gate BLOCK before M2.
- **P-003/P25-E050** [E] class=`E/TEST` anchor=`P-25:L506-L506` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L301): INTEGRATION_SMOKE is M4 non-decision smoke on live lane; auto second smoke if prereg/root/manifest/config bumps before M5.
- **P-003/P25-E051** [E] class=`E/SEC` anchor=`P-25:L518-L548` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L302): REDTEAM author+independent+round-2 patches: retrieval/hidden separation, cost cap lock, K=30 no-shrink, artifact-sourcing failure branch, four core implementations, lexical-leak limitation labeled, stratified shuffle, card grammar for FAMILY B, kill-leg names, SHUFFLED_REVISE control, multiplicity declaration.
- **P-003/P25-E052** [E] class=`E/LIMIT` anchor=`P-25:L47-L47` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L303): Wall-clock observations diagnose infrastructure only and never enter scientific score, guard, verdict, or hash.
- **P-003/P25-E053** [E] class=`E/CFG` anchor=`P-25:L42-L42, P-25:L543-L543` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L304): Public triage legs: demarcation + anchor + composition only; ledger leg EXCLUDED (would leak prior-run info); no "provenance" kill leg.
- **P-003/P25-E054** [E] class=`E/DATA` anchor=`P-25:L49-L49` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L305): GuardInputSetV1 binds bootstrap_input_set_sha256, rows, run_config_sha256, schema_version, sealed_corpus_sha256, violation_counts — decision API consumes this, not unbound cost report.
- **P-003/P25-I001** [I] class=`I/STATUS` anchor=`P-25:L1-L6, L71-L88` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L306): Because STATUS is PLANNING CANDIDATE pending V4 PASS, all M0-M5 symbols are planned deliverables, not shipped inventory claims.
- **P-003/P25-I002** [I] [SURFACE] class=`I/SURFACE` anchor=`P-25:L64-L65, L341-L359` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L307): Curator/run/checker CLIs are the only operator-facing surfaces; no dashboard panel, drawer, or notification is in scope.
- **P-003/P25-A001** [A] class=`A/STATUS` anchor=`P-25:L1-L6` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L308): V4 PASS is required but not established by this plan file itself; execution is unauthorized until that external audit.
- **P-003/P25-A002** [A] class=`A/DATA` anchor=`P-25:L73-L75` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L309): Exact preregistration document body (beyond freezes listed) and complete hash-domain registry contents live in M1 artifacts not embedded here.
- **P-003/P25-A003** [A] class=`A/DEP` anchor=`P-25:L6-L6, P-25:L120-L122` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L310): RECON-2 import-map migration to candidate-bound API is required and incomplete for controlling M3.
- **P-003/P25-A004** [A] class=`A/CFG` anchor=`P-25:L39-L39` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L311): Exact k default is 3 but frozen k and per-fixture substitution table are M1/manifest values not fully tabulated in prose.
- **P-003/P25-C001** [C] class=`C/STATUS` anchor=`P-25:L3-L6` unit=`P-25` (ledger-line: DEEP-P-003.discovery.md:L312): Header STATUS-HISTORY notes V3 scientific defect (candidates had no input edge to trusted core) repaired by V4 binding; any reader using V3 PASS as authority conflicts with stated precedence.

## Summary inventory atoms — from 16-P-SUMMARY_INDEX

Source: `docs/thefinalglance/feature-discovery/16-P-SUMMARY_INDEX.discovery.md`

Orientation-only inventory from the summary discovery pass. Where a deep SD-*/C-* record CORRECTs or QUALIFYs the same claim, **deep wins target state**; the summary atom is retained and the correction is listed under Superseded notes / SD.

### Summary unit `PLAN-WIDE` (24 records)

- **SUMMARY/P-G-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L60): P lifts the existing trust spine one grain: changeset to app, task to fleet, seat to world, and org to household.
- **SUMMARY/P-G-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L61): The product goal is one personal world of nodes and connected systems whose contents are protected, organized, and committed.
- **SUMMARY/P-G-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L62): The intended scale ranges from one solo toy, through connected small projects, to a large multi-person project.
- **SUMMARY/P-G-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L63): The eight initial personal-incarnation slices are described as approximately 80% composition of already-specified subsystems.
- **SUMMARY/P-G-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L64): P’s differentiator at every grain is the combined use of receipts, taint, replay, cant-lie enforcement, and wiring gates.
- **SUMMARY/P-G-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L65): P is classified as a product-surface letter with no direct PROOF_SPINE number; it is owner-acknowledged.
- **SUMMARY/P-G-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L66): P is authored now but is to execute only after the proof spine under A-7 scope honesty.
- **SUMMARY/P-G-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L67): The anti-fragmentation rule forbids a second lattice or scheduler; authoring must pass N-2 and reuse N-4/N-6/N-11.
- **SUMMARY/P-G-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L68): The trust ceremony must be measured at the toy tier and be near-zero-cost or GAP-5 fails.
- **SUMMARY/P-G-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L69): Process adaptation may loosen only from measured false-positive evidence belonging to that same user, never from user affect.
- **SUMMARY/P-G-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L70): Substrates ship as code, while personas, packs, and profiles remain data.
- **SUMMARY/P-G-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L71): Per-user, per-recipient, and per-seat code forks are forbidden.
- **SUMMARY/P-G-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L72): Every P unit must expose its measured number from day one; default-off features without a number are treated as the whole-feature falsifier.
- **SUMMARY/P-G-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L73): Named personal-track measures are fleet-health green percentage, wired-edge count, exported-tool re-verification success rate, toy-tier gate tax, profile-promotion false-positive rate, and merge determinism.
- **SUMMARY/P-G-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L74): “No number, no claim” is the KD-1 rule for P.
- **SUMMARY/P-G-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L75): User habits live in journaled data; G-10 monitors the model and B-37 swaps it.
- **SUMMARY/P-G-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L76): Per-app generalization of the G-10 drift canary is only asserted and is explicitly left as P-1 recon work.
- **SUMMARY/P-G-E18** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L77): Computer-use comparators are treated as complementary operator backends, not rivals; a raw-capability winner can be vendored beneath `elai wrap` and ELAI’s taint, journal, and gates.
- **SUMMARY/P-G-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L78): Computer-use exposed two declared gaps: no general desktop/GUI operator lane and no concrete agentic-run-to-replayable-artifact lane.
- **SUMMARY/P-G-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L79): OpenClaw mechanics may be stolen as designs only, never as code.
- **SUMMARY/P-G-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L80): P retains ELAI’s seeded determinism, replay verification, off-agent runtime gating, claim grounding, and prompt-injection boundary rather than adopting comparator trust semantics.
- **SUMMARY/P-G-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L81): P does not need to beat comparator breadth; it may contain capability winners as vendored, gated backends.
- **SUMMARY/P-G-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L82): P-14/P-15 feed PS-5, P-17 feeds PS-4, and P-21 feeds PS-6; all other computer-use/OpenClaw units declare no proof-spine contribution.
- **SUMMARY/P-G-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L83): Deep red-team work for the steal tracks is deferred to each unit’s execution preflight; each detail file is expected to retain a surviving-attack seed.

### Summary unit `P-1` (25 records)

- **SUMMARY/P1-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L93): Treat each of the user’s bespoke apps as a first-class object rather than only a collection of tasks.
- **SUMMARY/P1-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L94): Add an `AppManifest`.
- **SUMMARY/P1-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L95): `AppManifest` records the app purpose.
- **SUMMARY/P1-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L96): `AppManifest` records data namespaces.
- **SUMMARY/P1-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L97): `AppManifest` records ports.
- **SUMMARY/P1-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L98): `AppManifest` records the smoke check.
- **SUMMARY/P1-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L99): `AppManifest` records RRA lineage.
- **SUMMARY/P1-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L100): `AppManifest` records a dependency manifest.
- **SUMMARY/P1-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L101): Add a `FleetRegistry` for the app objects.
- **SUMMARY/P1-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L102): Implement a fleet health tick as an O-2 `TaskSpec`; do not create a separate scheduling mechanism.
- **SUMMARY/P1-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L103): Generalize G-10 drift canaries to operate per app.
- **SUMMARY/P1-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L104): Point F-9 dependency-rot checking at each app manifest.
- **SUMMARY/P1-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L105): A red health tick proposes a repair.
- **SUMMARY/P1-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L106): A proposed repair must pass the same wiring and claim gates used elsewhere.
- **SUMMARY/P1-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L107): Auto-repair is gated; a red tick must never become a silent patch.
- **SUMMARY/P1-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L108): Render the red state human-first through N-7.
- **SUMMARY/P1-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L109): Add the orchestrator `AppManifest` table, `FleetRegistry`, and health-tick `TaskSpec` factory.
- **SUMMARY/P1-E18** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L110): Expose the fleet surface through the `elai fleet` CLI.
- **SUMMARY/P1-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L111): Registering an app with a broken smoke and advancing the task-runner tick must land exactly one `fleet_health_tick` row off-agent.
- **SUMMARY/P1-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L112): The health row contains `{app_id, verdict=red, red_reasons}`.
- **SUMMARY/P1-E21** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L113): A red tick creates an N-7 inbox card.
- **SUMMARY/P1-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L114): A green tick that still creates a card fails acceptance.
- **SUMMARY/P1-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L115): A red tick with empty `red_reasons` fails acceptance.
- **SUMMARY/P1-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L121): The per-app G-10 canary generalization is asserted but not designed; P cross-cutting explicitly leaves it for recon.
- **SUMMARY/P1-A02** [A] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L122): The index names `elai fleet` but does not enumerate its subcommands or config schema.

### Summary unit `P-2` (19 records)

- **SUMMARY/P2-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L130): Extend `AppManifest` with a `PortSchema`.
- **SUMMARY/P2-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L131): Add a `PortRegistry`.
- **SUMMARY/P2-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L132): Let the O-4 canvas bind typed tool-output ports to typed tool-input ports.
- **SUMMARY/P2-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L133): Every tool-to-tool edge carries an N-6 provenance tier.
- **SUMMARY/P2-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L134): Every tool-to-tool edge carries a G-1 taint tier.
- **SUMMARY/P2-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L135): Render an output with zero consumers as visibly disconnected.
- **SUMMARY/P2-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L136): Use M-5 `elai-verify` conformance to admit a third-party tool as a mountable node.
- **SUMMARY/P2-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L137): Do not introduce a live broker or separate runtime.
- **SUMMARY/P2-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L138): Keep invocation dispatch in the FSM under O-4 doctrine.
- **SUMMARY/P2-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L139): Define the “event bus” as a typed-edge projection plus journaled invocation rows.
- **SUMMARY/P2-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L140): Add a compatibility checker alongside `PortSchema` and `PortRegistry` in the orchestrator.
- **SUMMARY/P2-E12** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L141): Expose `elai fleet wire` and `elai fleet ports`.
- **SUMMARY/P2-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L142): Render the port/edge view through O-4 at Tier 2.
- **SUMMARY/P2-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L143): A successful bind lands one `tool_port_edge` row.
- **SUMMARY/P2-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L144): The edge row contains `{src.out, dst.in, n6_tier, taint_tier}`.
- **SUMMARY/P2-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L145): An incompatible wire is refused and lands no edge row.
- **SUMMARY/P2-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L146): An edge missing its taint tier fails acceptance.
- **SUMMARY/P2-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L152): “Incompatible wire lands NO row” is explicit for the edge row, but the summary does not say whether a separate refusal/audit row must exist.
- **SUMMARY/P2-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L153): The compatibility relation and schema-version behavior are not defined in the summary.

### Summary unit `P-3` (24 records)

- **SUMMARY/P3-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L161): Factor transferable app mechanism separately from recipient-specific persona data.
- **SUMMARY/P3-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L162): Add an `AppBundle`.
- **SUMMARY/P3-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L163): The bundle includes the mechanism.
- **SUMMARY/P3-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L164): The bundle includes the manifest.
- **SUMMARY/P3-E05** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L165): The bundle includes an RRA receipt.
- **SUMMARY/P3-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L166): The bundle includes G-9-redacted provenance.
- **SUMMARY/P3-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L167): Persona rules rows are data and are excluded from the bundle.
- **SUMMARY/P3-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L168): Persona memory namespaces are data and are excluded from the bundle.
- **SUMMARY/P3-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L169): Persona thresholds are data and are excluded from the bundle.
- **SUMMARY/P3-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L170): Persona tokens are data and are excluded from the bundle.
- **SUMMARY/P3-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L171): Regenerate the persona layer on the recipient’s seat.
- **SUMMARY/P3-E12** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L172): `elai app import` re-verifies a bundle through M-5.
- **SUMMARY/P3-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L173): Refuse to mount a tampered bundle.
- **SUMMARY/P3-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L174): Apply the L-12/Skillspector admission family to app bundles.
- **SUMMARY/P3-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L175): Friend-to-friend sharing and marketplace sharing use the same admission gate.
- **SUMMARY/P3-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L176): Add orchestrator `AppBundle`, export/import, M-5 verification, and G-9 redaction wiring.
- **SUMMARY/P3-E17** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L177): Expose `elai app export` and `elai app import`.
- **SUMMARY/P3-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L178): Export lands an `app_export` row.
- **SUMMARY/P3-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L179): A tampered import lands an `app_import` refusal and does not mount.
- **SUMMARY/P3-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L180): A clean import lands a mount row.
- **SUMMARY/P3-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L181): Any tampered bundle that mounts fails acceptance.
- **SUMMARY/P3-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L182): Any exported bundle that leaks a secret fails acceptance.
- **SUMMARY/P3-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L188): The exact bundle format/version/signature fields are absent from the summary.
- **SUMMARY/P3-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L189): Persona regeneration inputs and whether every excluded category is regenerated or defaulted are not specified.

### Summary unit `P-4` (21 records)

- **SUMMARY/P4-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L197): Extend static O-3 vault mounts into live stores.
- **SUMMARY/P4-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L198): Cover Obsidian under concurrent plugin edits as a live-store class.
- **SUMMARY/P4-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L199): Cover Notion as a live-store class, subject to source recon.
- **SUMMARY/P4-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L200): Cover SQLite as a live-store class.
- **SUMMARY/P4-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L201): Add one `LivingStoreContract` per store class.
- **SUMMARY/P4-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L202): A living-store contract defines watching.
- **SUMMARY/P4-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L203): A living-store contract defines a change feed.
- **SUMMARY/P4-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L204): Ingest through the contract is tagged through F-5/N-6.
- **SUMMARY/P4-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L205): Writes are allowed only inside namespaces claimed through O-3.
- **SUMMARY/P4-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L206): Lift B-35’s context pipeline to living stores.
- **SUMMARY/P4-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L207): Make the D-5/O-3 life denylist a hard routing wall through P-6.
- **SUMMARY/P4-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L208): Block implementation until the Obsidian/Notion API is source-grounded.
- **SUMMARY/P4-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L209): Add `LivingStoreContract`, an Obsidian watch adapter, and an O-3-claim-gated writer in `crates/knowledge`.
- **SUMMARY/P4-E14** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L210): Expose `elai brain mount` and `elai brain sync`.
- **SUMMARY/P4-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L211): Editing a claimed-namespace note lands `vault_ingest {note_path, n6_tier, staleness_ttl}`.
- **SUMMARY/P4-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L212): Attempting an unclaimed write lands a `claim_violation` row.
- **SUMMARY/P4-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L213): Mutating a user-owned note outside the claimed namespace fails acceptance.
- **SUMMARY/P4-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L214): Ingest missing either provenance tier or staleness TTL fails acceptance.
- **SUMMARY/P4-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L220): The digest says Obsidian/Notion surfaces are training-only and ungrounded, yet says the Notion MCP is a live grounding source “this session”; the hard `WEAK-until-RECON`/`GATED` label still controls.
- **SUMMARY/P4-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L221): Only an Obsidian adapter is named in integration despite three store classes being named.
- **SUMMARY/P4-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L222): Watch ordering, conflict handling, deletions, and writeback semantics are not summarized.

### Summary unit `P-5` (21 records)

- **SUMMARY/P5-E01** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L230): Add a non-coder operator surface at the P-1 fleet grain.
- **SUMMARY/P5-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L231): Add a `CeremonyProfile` over K-5/K-4.
- **SUMMARY/P5-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L232): `CeremonyProfile` has `toy`, `personal`, `project`, and `critical` tiers.
- **SUMMARY/P5-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L233): At `toy`, gates execute in observe-only mode.
- **SUMMARY/P5-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L234): Toy gates still run.
- **SUMMARY/P5-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L235): Toy gate verdicts are still journaled.
- **SUMMARY/P5-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L236): Toy gates do not block on warnings.
- **SUMMARY/P5-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L237): No tier may silently skip a gate.
- **SUMMARY/P5-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L238): Collapse toy-facing receipts to one line.
- **SUMMARY/P5-E10** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L239): The one-line receipt must index/resolve to the full journal.
- **SUMMARY/P5-E11** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L240): Render a red fleet tick as one sentence plus one gated repair button through N-7.
- **SUMMARY/P5-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L241): Measure the toy-tier gate tax and require it to be near zero.
- **SUMMARY/P5-E13** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L242): Add orchestrator `CeremonyProfile`, K-5 gate-mode wiring, and one-line receipt rendering.
- **SUMMARY/P5-E14** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L243): Expose `elai profile set` and `elai profile tax`.
- **SUMMARY/P5-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L244): Setting a tier lands a `ceremony_profile_set` row.
- **SUMMARY/P5-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L245): The underlying verdict is journaled at both toy and critical tiers.
- **SUMMARY/P5-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L246): Critical blocks on WARN where toy does not.
- **SUMMARY/P5-E18** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L247): `elai profile tax --tier toy` is the named tax measurement entry point.
- **SUMMARY/P5-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L248): Dropping the journal row at toy fails acceptance.
- **SUMMARY/P5-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L254): “Near-zero” has no numeric ceiling, workload, or confidence interval in the summary.
- **SUMMARY/P5-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L255): Blocking behavior for `personal` and `project` is not enumerated.

### Summary unit `P-6` (20 records)

- **SUMMARY/P6-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L263): Extend the one N-6 lattice with a privacy domain dimension; do not create a second lattice.
- **SUMMARY/P6-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L264): The domain enum is `life | work | shared`.
- **SUMMARY/P6-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L265): Add per-tool `read_grants`.
- **SUMMARY/P6-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L266): Read grants are default-deny.
- **SUMMARY/P6-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L267): Enforce read grants at the O-3 vault-read boundary.
- **SUMMARY/P6-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L268): Enforce the same grants at E-20 routing.
- **SUMMARY/P6-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L269): Treat life-data egress as the mirror of G-1 taint: owned bytes flowing out versus untrusted bytes flowing in.
- **SUMMARY/P6-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L270): Render every grant through N-7.
- **SUMMARY/P6-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L271): Journal every grant.
- **SUMMARY/P6-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L272): Make every grant revertible through H-49.
- **SUMMARY/P6-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L273): Make a second privacy-lattice table an authoring failure under A-1/N-2.
- **SUMMARY/P6-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L274): Add the domain dimension, grants, and O-3 read enforcement in `crates/knowledge`.
- **SUMMARY/P6-E13** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L275): Expose `elai privacy grant`, `revoke`, and `show`.
- **SUMMARY/P6-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L276): Granting a tool access lands a `privacy_grant` row.
- **SUMMARY/P6-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L277): A read inside the granted namespace succeeds.
- **SUMMARY/P6-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L278): A read in another ungranted namespace lands `privacy_read_refusal`.
- **SUMMARY/P6-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L279): N-2 must confirm that exactly one lattice exists.
- **SUMMARY/P6-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L280): An ungranted read that succeeds fails acceptance.
- **SUMMARY/P6-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L281): A second lattice table fails acceptance.
- **SUMMARY/P6-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L287): Grant scope beyond the example namespace, such as expiry, nested namespaces, and cross-domain precedence, is not specified.

### Summary unit `P-7` (24 records)

- **SUMMARY/P7-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L295): Adapt process separately from E-5 content memory.
- **SUMMARY/P7-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L296): Add an `OperatingProfile`.
- **SUMMARY/P7-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L297): Store autonomy per domain.
- **SUMMARY/P7-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L298): Store strictness per domain.
- **SUMMARY/P7-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L299): Store verbosity per domain.
- **SUMMARY/P7-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L300): Learn and promote profile changes through N-3/H-17/H-18 measurements rather than subjective affect.
- **SUMMARY/P7-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L301): Tightening a profile is free to apply.
- **SUMMARY/P7-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L302): Loosening requires measured false-positive evidence.
- **SUMMARY/P7-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L303): Loosening evidence must come from the same user’s own runs.
- **SUMMARY/P7-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L304): Raw counts alone cannot authorize loosening.
- **SUMMARY/P7-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L305): Another user’s data cannot authorize loosening.
- **SUMMARY/P7-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L306): Observe profile effects before promotion.
- **SUMMARY/P7-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L307): Journal every profile change/refusal.
- **SUMMARY/P7-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L308): Make profile changes revertible through H-49.
- **SUMMARY/P7-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L309): Define `solo toy`, `connected`, and `team` as defaults over one engine, not separate products.
- **SUMMARY/P7-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L310): Add the profile and an asymmetry-guarded change function to the orchestrator.
- **SUMMARY/P7-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L311): Route loosen promotion through H-17/H-18.
- **SUMMARY/P7-E18** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L312): Expose `elai profile show`, `tighten`, and `loosen`.
- **SUMMARY/P7-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L313): A loosen request without evidence is refused.
- **SUMMARY/P7-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L314): A loosen request with measured own-run FP evidence is promoted through H-18.
- **SUMMARY/P7-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L315): Tightening applies directly.
- **SUMMARY/P7-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L316): Refusal, promotion, and revert rows carry an `evidence_ref` where applicable.
- **SUMMARY/P7-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L322): False-positive metric definitions, promotion thresholds, observation duration, and default values are absent.
- **SUMMARY/P7-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L323): The exact domain vocabulary is not restated here and may rely on P-6/N-6.

### Summary unit `P-8` (32 records)

- **SUMMARY/P8-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L331): Support 2-10 humans and their agents in one shared world.
- **SUMMARY/P8-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L332): Extend O-3 namespace claims into shared vault namespaces with N claimants.
- **SUMMARY/P8-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L333): Lift cross-session harness locks to product-level cross-seat locks.
- **SUMMARY/P8-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L334): Add `seat_id` provenance to every SSB row.
- **SUMMARY/P8-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L335): Add `agent_id` provenance to every SSB row.
- **SUMMARY/P8-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L336): Make attribution answerable as a query.
- **SUMMARY/P8-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L337): Apply the M-3 rollup at shared-project grain.
- **SUMMARY/P8-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L338): Build the merge kernel as a spike first.
- **SUMMARY/P8-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L339): Merge two seat journal streams losslessly.
- **SUMMARY/P8-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L340): Make merge deterministic.
- **SUMMARY/P8-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L341): Preserve causal ordering across the merged streams.
- **SUMMARY/P8-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L342): Forbid last-writer-wins merge semantics.
- **SUMMARY/P8-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L343): Do not introduce an org control plane.
- **SUMMARY/P8-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L344): Keep collaboration local-first.
- **SUMMARY/P8-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L345): Add SSB-stream merge, provenance columns, and cross-seat locks across orchestrator/knowledge.
- **SUMMARY/P8-E16** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L346): Expose `elai world join`, `merge`, and `whodid`.
- **SUMMARY/P8-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L347): A merge lands `seat_journal_merge {seats, event_count, order_hash}`.
- **SUMMARY/P8-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L348): Merging the same two streams in either direction produces an identical `order_hash`.
- **SUMMARY/P8-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L349): A dropped event fails acceptance.
- **SUMMARY/P8-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L350): A direction-dependent `order_hash` fails acceptance.
- **SUMMARY/P8-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L351): A lost concurrent write fails acceptance.
- **SUMMARY/P8-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L357): The deterministic causal merge algorithm is explicitly an open hard kernel.
- **SUMMARY/P8-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L358): Identity enrollment, revocation, transport, offline reconciliation, and conflict presentation are not defined in this summary.
- **SUMMARY/P9-I01** [I] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L364): Later sections identify P-9 as a connector/stage-pack format and a gated pack lane.
- **SUMMARY/P9-I02** [I] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L365): P-16 says an install-catalog target is a P-9 pack and distinguishes a headless code-pack connector from a GUI fallback.
- **SUMMARY/P9-A01** [A] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L366): No P-9 fields, acceptance rows, CLI, status, or complete behavior are present in the summary.
- **SUMMARY/P10-I01** [I] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L367): Later sections identify P-10 as a credential vault/auth plane used by connectors.
- **SUMMARY/P10-I02** [I] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L368): P-23 assigns named browser-profile custody to P-10.
- **SUMMARY/P10-I03** [I] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L369): P-22 associates P-10 ingress adapters with channel-binding precedence and `dmScope` session isolation.
- **SUMMARY/P10-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L370): No P-10 vault schema, adapter taxonomy, credential lifecycle, acceptance gate, or direct entrypoint is summarized.
- **SUMMARY/P11-I01** [I] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L371): Later sections identify P-11 as workflow-from-goal with a human confirm gate; P-18 hardens its “MOVE 3” through drift binding.
- **SUMMARY/P11-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L372): No P-11 graph-synthesis rules, UI/API, events, negative gates, or grounding/status block is present.

### Summary unit `P-12` (37 records)

- **SUMMARY/P12-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L380): Add a general desktop/GUI `OperatorLane` for workflows such as driving a GUI app or testing a game build.
- **SUMMARY/P12-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L381): Execute the lane inside a disposable sandbox.
- **SUMMARY/P12-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L382): Vendor the isolation substrate from CUA’s Windows Sandbox, Lume, or Docker mechanisms rather than reimplementing it.
- **SUMMARY/P12-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L383): Treat the vendored CUA substrate as MIT-licensed at summary time.
- **SUMMARY/P12-E05** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L384): Supply ELAI fail-closed gates because the comparator’s in-loop safety is only a commented-out TODO.
- **SUMMARY/P12-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L385): Consume the P-13 perception feed.
- **SUMMARY/P12-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L386): Fire the OmniParser Tier-3 pixel fallback only when the accessibility tree is empty or incomplete.
- **SUMMARY/P12-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L387): Bind every coordinate action to the exact SHA-256 hash of the image delivered to the model.
- **SUMMARY/P12-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L388): Authorize that coordinate action only while the exact image remains in context.
- **SUMMARY/P12-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L389): Invalidate the frame epoch when compaction removes the image from context.
- **SUMMARY/P12-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L390): Never retarget an action to display 0 if its original display/frame is unavailable.
- **SUMMARY/P12-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L391): Use a two-phase arm/commit sequence.
- **SUMMARY/P12-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L392): Journal the capability grant before exposing the actuation capability.
- **SUMMARY/P12-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L393): Apply lease TTL expiry both during periodic sweep and inline at use.
- **SUMMARY/P12-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L394): Capture a bounded screen recording for every operator action.
- **SUMMARY/P12-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L395): Hash that screen recording into an off-agent evidence row.
- **SUMMARY/P12-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L396): Treat every operator action as a G-1 taint source.
- **SUMMARY/P12-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L397): Mark the action as an operator-lane action.
- **SUMMARY/P12-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L398): Never make GUI actuation load-bearing infrastructure on which workflow correctness depends.
- **SUMMARY/P12-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L399): Reuse 4F isolation, P-13 perception, and G-1 taint rather than reimplementing them.
- **SUMMARY/P12-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L400): Compare CUA’s pure-function `OperatorNormalizer` repair against H-13 `sanitizeAction` rather than assuming equivalence.
- **SUMMARY/P12-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L401): Add `crates/orchestrator/src/operator/` with lane, sandbox adapter, frame binder, and arm/lease components.
- **SUMMARY/P12-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L402): Add `operator_action` as an O-2 `TaskSpec` stage kind.
- **SUMMARY/P12-E24** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L403): Expose `elai operator run`.
- **SUMMARY/P12-E25** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L404): Render an operator node on the O-4 canvas.
- **SUMMARY/P12-E26** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L405): Pair operator-action rows with a screen-recording thumbnail in an existing drawer.
- **SUMMARY/P12-E27** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L406): Re-verify at edit time that OmniParser `icon_detect` weights are AGPL and Florence captions are MIT.
- **SUMMARY/P12-E28** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L407): A successful sandbox click lands `operator_action {frame_hash, action, lease_id}`.
- **SUMMARY/P12-E29** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L408): The same action lands one `operator_frame_bind` row.
- **SUMMARY/P12-E30** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L409): The recording artifact hash must match the journal row.
- **SUMMARY/P12-E31** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L410): Reject a coordinate action against a stale or compacted frame with no actuation.
- **SUMMARY/P12-E32** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L411): Actuating on a stale `frame_hash` fails acceptance.
- **SUMMARY/P12-E33** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L412): Exposing capability before the grant row lands fails acceptance.
- **SUMMARY/P12-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L418): “Incomplete” accessibility tree detection is not defined here; P-13 sharpens it to “provably incomplete.”
- **SUMMARY/P12-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L419): The exact bounded-recording duration/size and lease duration are unspecified.
- **SUMMARY/P12-A03** [A] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L420): The existing frontend drawer is deliberately left to an edit-time W-4 decision.
- **SUMMARY/P12-A04** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L421): The license assertion is explicitly provisional and must be rechecked.

### Summary unit `P-13` (29 records)

- **SUMMARY/P13-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L429): Extend J-1 perception beyond conditioned bytes and AXTree/DOM text so the model can point safely.
- **SUMMARY/P13-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L430): Apply set-of-marks ID indirection to the accessibility-tree node list.
- **SUMMARY/P13-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L431): Have the model/VLM return an integer element ID rather than raw geometry.
- **SUMMARY/P13-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L432): Keep element geometry owned by the deterministic harness.
- **SUMMARY/P13-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L433): Attach a source tag to every element.
- **SUMMARY/P13-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L434): The source-tag enum is `axtree_exact | ocr_text | vlm_caption | pixel_yolo`.
- **SUMMARY/P13-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L435): Treat the source tag as the element’s trust boundary for G-1 taint/grounding.
- **SUMMARY/P13-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L436): Add an interactability flag.
- **SUMMARY/P13-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L437): Filter the VLM candidate set through the interactability flag.
- **SUMMARY/P13-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L438): Use captions only as advisory data for unnamed canvas/game icons.
- **SUMMARY/P13-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L439): Fuse OCR text into element labels asymmetrically.
- **SUMMARY/P13-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L440): Fire Tier-3 pixel/YOLO perception only when the accessibility tree is empty or provably incomplete.
- **SUMMARY/P13-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L441): Cover games, WebGL, and native dead-UIA surfaces through that guarded fallback.
- **SUMMARY/P13-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L442): Never obtain counts from a VLM; preserve the J-1 counting law.
- **SUMMARY/P13-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L443): Never let a per-crop caption stand in for element identity.
- **SUMMARY/P13-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L444): Treat a mis-sourced element as a J-77 ghost citation.
- **SUMMARY/P13-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L445): Extend J-1/J-8 scaffold rows with source and box-ID fields, preferring reuse and adding at most one net row.
- **SUMMARY/P13-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L446): Put the guarded Tier-3 adapter in the existing `crates/orchestrator/src/perception/` seam.
- **SUMMARY/P13-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L447): Render the source-tagged element breakdown in the existing perception tile.
- **SUMMARY/P13-E20** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L448): Add no CLI; expose perception as a service seam for E-14 and P-12.
- **SUMMARY/P13-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L449): An empty-AXTree canvas fixture produces integer IDs and a source tag for every element.
- **SUMMARY/P13-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L450): The Tier-3 pixel leg fires exactly once for that fixture.
- **SUMMARY/P13-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L451): The trace contains zero VLM count calls.
- **SUMMARY/P13-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L452): “Click element 7” resolves through the ID index rather than model-supplied coordinates.
- **SUMMARY/P13-E25** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L453): A VLM-sourced count fails acceptance.
- **SUMMARY/P13-E26** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L454): An element missing its source tag fails acceptance.
- **SUMMARY/P13-A01** [A] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L460): The cited 70.5→93.8% is comparator evidence, not an ELAI acceptance threshold.
- **SUMMARY/P13-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L461): The summary does not define source-tag precedence when several sources contribute to one element.
- **SUMMARY/P13-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L462): “Asymmetric” OCR fusion is named but its overwrite/merge rule is not specified.

### Summary unit `P-14` (24 records)

- **SUMMARY/P14-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L470): Let an E-14 browser or P-12 desktop exploration run mint a deterministic replay artifact.
- **SUMMARY/P14-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L471): Emit Playwright for browser exploration.
- **SUMMARY/P14-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L472): Emit PyAutoGUI for desktop exploration.
- **SUMMARY/P14-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L473): Represent the emitted script as an O-2 `TaskSpec` stage.
- **SUMMARY/P14-E05** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L474): Bind the script to the exact exploration run that minted it through a G-2 RRA receipt.
- **SUMMARY/P14-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L475): Use script replay as the cheap, zero-reasoning execution mode.
- **SUMMARY/P14-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L476): Invoke no model during replay.
- **SUMMARY/P14-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L477): Keep the crystallized flow inside O-2 dispatch rather than adding a second dispatcher.
- **SUMMARY/P14-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L478): Extend the existing compiled-pipeline/template-crystallizer general form rather than fork it.
- **SUMMARY/P14-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L479): Use P-13 perception during exploration.
- **SUMMARY/P14-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L480): Add the O-2 stage kind `crystallized_script`.
- **SUMMARY/P14-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L481): Add a script emitter over the E-14/P-12 trace.
- **SUMMARY/P14-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L482): Add an RRA binder for minted scripts.
- **SUMMARY/P14-E14** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L483): Expose `elai e2e crystallize`.
- **SUMMARY/P14-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L484): Replay through the normal task-runner tick.
- **SUMMARY/P14-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L485): Render a crystallized-stage node on the O-4 canvas at Tier 2.
- **SUMMARY/P14-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L486): Minting lands `crystallized_stage_minted {script_hash, minting_run_id, rra_hash}`.
- **SUMMARY/P14-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L487): A replay drives the real entry point with zero model calls in its trace.
- **SUMMARY/P14-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L488): Replay must reproduce the exploration’s observable outcome.
- **SUMMARY/P14-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L489): A script whose RRA does not bind to a real minting run fails acceptance.
- **SUMMARY/P14-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L490): A replay that silently invokes a model fails acceptance.
- **SUMMARY/P14-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L491): Cite CUA `replay_env` and Agent Zero’s skills/scripts convention as prior art, not copied code.
- **SUMMARY/P14-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L497): “~100% reliable” is narrative motivation; no quantitative replay-reliability threshold is set.
- **SUMMARY/P14-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L498): Script portability, nondeterministic UI handling, and secret/credential parameterization are not defined here.

### Summary unit `P-15` (23 records)

- **SUMMARY/P15-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L506): Preserve J-96’s rule that a typed case node is minted only after a proven fix.
- **SUMMARY/P15-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L507): Add Agent-S-style retrieve behavior to the verified case graph.
- **SUMMARY/P15-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L508): Add distillation behavior to the reuse loop.
- **SUMMARY/P15-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L509): Add fusion of retrieved case information into the next plan.
- **SUMMARY/P15-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L510): Support narrative and episodic case tiers in that loop.
- **SUMMARY/P15-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L511): Use embedding retrieval as part of the loop.
- **SUMMARY/P15-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L512): Gate every save behind J-96’s execution-grounded mint-on-proof filter.
- **SUMMARY/P15-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L513): Do not copy Agent-S’s save-every-run/ungated-commit behavior.
- **SUMMARY/P15-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L514): Add a bBoN behavior-narrative comparative judge.
- **SUMMARY/P15-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L515): Run the bBoN judge over journal rows, not screenshots.
- **SUMMARY/P15-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L516): Use ground-truth action/effect deltas available in journal rows.
- **SUMMARY/P15-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L517): Insert bBoN only as an I evidence-ranked-search tie-breaker.
- **SUMMARY/P15-E13** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L518): Receipts choose first; narrative only breaks receipt-level ties.
- **SUMMARY/P15-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L519): Keep reuse behind J-96’s strict execution-grounded filter and default-off.
- **SUMMARY/P15-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L520): Reuse J-96 memory canvas node chrome.
- **SUMMARY/P15-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L521): A proven fix followed by recurrence retrieves and fuses the case.
- **SUMMARY/P15-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L522): An unproven run’s summary is not saved.
- **SUMMARY/P15-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L523): Record a `reuse-hit` field on the J-96 case row.
- **SUMMARY/P15-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L524): Feed that hit into K-18 recurrence-hit-rate economics.
- **SUMMARY/P15-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L525): A case saved from an unproven run fails acceptance.
- **SUMMARY/P15-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L526): A bBoN judgment sourced from screenshots fails acceptance.
- **SUMMARY/P15-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L532): Retrieval embedding model, tier schemas, tie conditions, and fusion policy are not summarized.
- **SUMMARY/P15-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L533): “Default-off” coexists with P’s default-off graveyard doctrine; promotion/on-switch criteria must come from the detail plan or K-18/K-19.

### Summary unit `P-16` (23 records)

- **SUMMARY/P16-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L541): Add a named install catalog so capability-on-demand feels immediate.
- **SUMMARY/P16-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L542): Each catalog row carries `tool`.
- **SUMMARY/P16-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L543): Each catalog row carries `source`.
- **SUMMARY/P16-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L544): Each catalog row carries `mirror`.
- **SUMMARY/P16-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L545): Each catalog row carries `alias`.
- **SUMMARY/P16-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L546): Each catalog row carries `license_tag`.
- **SUMMARY/P16-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L547): Add an `installPolicy` admission decision behind the existing L-12/Skillspector/N-6/hidden-until-verified family.
- **SUMMARY/P16-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L548): Resolve an install request to a P-9 headless code-pack connector when one exists.
- **SUMMARY/P16-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L549): Prefer deterministic, journalable, replayable headless interfaces such as `blender --background --python`.
- **SUMMARY/P16-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L550): Use P-12 GUI driving only as the last resort.
- **SUMMARY/P16-E11** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L551): Never use GUI driving as the install mechanism when a headless CLI exists.
- **SUMMARY/P16-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L552): Reuse L-12 admission; do not add a second scanner.
- **SUMMARY/P16-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L553): Do not copy OpenClaw’s advisory regex scanner.
- **SUMMARY/P16-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L554): Add the install-catalog table and the policy check on the existing L-12 path.
- **SUMMARY/P16-E15** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L555): Expose `elai connector install <tool>`.
- **SUMMARY/P16-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L556): Render catalog data and admission verdict in the connector surface at Tier 2.
- **SUMMARY/P16-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L557): Every install attempt lands `install_catalog_admit {tool, source, license_tag, verdict}`.
- **SUMMARY/P16-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L558): A clean catalog item mounts a P-9 headless pack.
- **SUMMARY/P16-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L559): A flagged item is refused and no install occurs.
- **SUMMARY/P16-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L560): Bypassing the L-12 admission family fails acceptance.
- **SUMMARY/P16-E21** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L561): GUI-driving an install despite a headless CLI fails acceptance.
- **SUMMARY/P16-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L567): Its two direct prerequisites, P-9/P-10, have no sections in this summary.
- **SUMMARY/P16-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L568): Source/mirror integrity, catalog signing/update, alias collisions, and license-policy vocabulary are not summarized.

### Summary unit `P-17` (28 records)

- **SUMMARY/P17-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L686): Extend J’s free-API rotation pool and I-50 auth broker with a complete failure-semantics package.
- **SUMMARY/P17-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L687): Classify rate-limit failures into their own cooldown lane.
- **SUMMARY/P17-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L688): Escalate rate-limit cooldowns through 30 seconds, 1 minute, then 5 minutes.
- **SUMMARY/P17-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L689): Classify billing failures into their own cooldown lane.
- **SUMMARY/P17-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L690): Escalate billing cooldowns from 5 hours to 24 hours.
- **SUMMARY/P17-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L691): Classify authentication failures into their own cooldown lane.
- **SUMMARY/P17-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L692): Apply an authentication cooldown in the stated 10-to-60-minute range.
- **SUMMARY/P17-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L693): Distinguish model-scoped cooldowns from profile-scoped cooldowns.
- **SUMMARY/P17-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L694): Allow a sibling model on the same provider to try when only one model/lane is cooled.
- **SUMMARY/P17-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L695): Preserve session stickiness to keep provider caches warm.
- **SUMMARY/P17-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L696): Treat rotation as a cache-invalidation event.
- **SUMMARY/P17-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L697): Journal that cache-invalidation event.
- **SUMMARY/P17-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L698): Persist an override before retrying, closing the live-switch race.
- **SUMMARY/P17-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L699): Honor a user’s explicit model choice strictly.
- **SUMMARY/P17-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L700): Never silently substitute an explicitly selected model.
- **SUMMARY/P17-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L701): Record every permitted substitution as an event, consistent with Q-4 swap/restart.
- **SUMMARY/P17-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L702): Keep billed truth and estimated usage as different typed fields end to end.
- **SUMMARY/P17-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L703): Require every usage surface to label which of the two it displays.
- **SUMMARY/P17-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L704): Extend J-28 `UsageTurnEvent` with the typed billed/estimate distinction.
- **SUMMARY/P17-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L705): Implement the cooldown taxonomy, scope, and stickiness in the provider failover state machine.
- **SUMMARY/P17-E21** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L706): Render failover and a billed-versus-estimate label in the RuntimePanel/CloudPanel usage surface.
- **SUMMARY/P17-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L707): Every forced billing/rate-limit transition lands a `provider_failover_event` row with its correct lane and cooldown.
- **SUMMARY/P17-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L708): Cooling the whole provider for a billing-lane failure fails acceptance.
- **SUMMARY/P17-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L709): Silent substitution of an explicit model fails acceptance.
- **SUMMARY/P17-E25** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L710): Merging billed and estimated values into one number fails acceptance.
- **SUMMARY/P17-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L716): The summary gives an auth cooldown range rather than a deterministic ladder.
- **SUMMARY/P17-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L717): Exact mapping from failure status/error code to model/profile/billing lane and persistence lifetime is not enumerated.
- **SUMMARY/P17-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L718): “Billing failure cools the billed lane” is clear as a negative invariant, but the precise scope object behind that lane is not named.

### Summary unit `P-18` (38 records)

- **SUMMARY/P18-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L726): Extend the O-2/P-11 `TaskSpec` run store with durable-execution mechanics.
- **SUMMARY/P18-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L727): Add a revision counter for optimistic concurrency.
- **SUMMARY/P18-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L728): Require writers to provide an expected revision.
- **SUMMARY/P18-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L729): Turn a stale expected revision into a conflict.
- **SUMMARY/P18-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L730): Refuse a stale write without clobbering current state.
- **SUMMARY/P18-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L731): Persist cancel intent.
- **SUMMARY/P18-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L732): Make cancel intent sticky across restart.
- **SUMMARY/P18-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L733): Add retry ladders per failure class.
- **SUMMARY/P18-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L734): Add watchdog phases.
- **SUMMARY/P18-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L735): Add preflight phases.
- **SUMMARY/P18-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L736): Clamp commitment due time so an inferred follow-up cannot fire in the same tick in which it was inferred.
- **SUMMARY/P18-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L737): Bind every human approval to a canonical hashed run plan.
- **SUMMARY/P18-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L738): Bind every autonomous approval to the same canonical plan form.
- **SUMMARY/P18-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L739): Include `cwd` in the approval-bound plan.
- **SUMMARY/P18-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L740): Include `argv` in the approval-bound plan.
- **SUMMARY/P18-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L741): Include the allowlisted `env` in the approval-bound plan.
- **SUMMARY/P18-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L742): Include the pinned executable in the approval-bound plan.
- **SUMMARY/P18-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L743): Bind concrete operand fingerprints alongside the plan hash.
- **SUMMARY/P18-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L744): Revalidate the plan and operands immediately before execution.
- **SUMMARY/P18-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L745): Deny execution on any drift between grant and use.
- **SUMMARY/P18-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L746): Carry “approve+resume skips prior completed steps” only as `RECON_NEEDED`, not a verified mechanism.
- **SUMMARY/P18-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L747): Re-derive resume as a real subprocess-checkpoint model.
- **SUMMARY/P18-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L748): Test that model with SIGKILL.
- **SUMMARY/P18-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L749): Test pipe backpressure.
- **SUMMARY/P18-E25** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L750): Add a step-checkpoint store rather than rely on cooperative wrapper enforcement.
- **SUMMARY/P18-E26** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L751): Add the run-store mechanics and approval plan hasher/validator to the 3A/4G/P-11 approval seams.
- **SUMMARY/P18-E27** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L752): Expose `elai workflow resume` and `elai workflow cancel`.
- **SUMMARY/P18-E28** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L753): Render run revision and drift-reject rows in the workflow view.
- **SUMMARY/P18-E29** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L754): Two concurrent writes with one stale revision land one `taskspec_run_revision` conflict and no clobber.
- **SUMMARY/P18-E30** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L755): Swapping an operand after grant lands `approval_drift_reject`.
- **SUMMARY/P18-E31** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L756): A simulated restart preserves cancellation.
- **SUMMARY/P18-E32** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L757): Last-writer-wins behavior fails acceptance.
- **SUMMARY/P18-E33** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L758): Executing a drifted plan fails acceptance.
- **SUMMARY/P18-E34** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L759): Firing a commitment in its own inference tick fails acceptance.
- **SUMMARY/P18-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L765): Resume-token step skipping is explicitly weak/unverified and must not be implemented from the external claim.
- **SUMMARY/P18-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L766): Retry classes, backoff values, watchdog deadlines, and checkpoint granularity are absent.
- **SUMMARY/P18-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L767): Canonical serialization and operand-fingerprint algorithms are not defined in the summary.
- **SUMMARY/P18-A04** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L768): P-11 is absent, so the exact confirmation seam called “MOVE 3” needs the detail source.

### Summary unit `P-19` (36 records)

- **SUMMARY/P19-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L776): Make absent policy objects instantiate fail-closed defaults.
- **SUMMARY/P19-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L777): Preserve a last-known-good configuration.
- **SUMMARY/P19-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L778): Preserve rejected configuration writes as `.rejected.<ts>`.
- **SUMMARY/P19-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L779): Reject a write that shrinks the configuration by more than 50%.
- **SUMMARY/P19-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L780): Add clobber guards beyond the shrink check.
- **SUMMARY/P19-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L781): Write configuration atomically.
- **SUMMARY/P19-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L782): Route these mechanics through J’s existing config-ingress admission gate.
- **SUMMARY/P19-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L783): Journal recoverable rejected writes.
- **SUMMARY/P19-E09** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L784): Add `elai doctor detect`.
- **SUMMARY/P19-E10** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L785): Add `elai doctor repair`.
- **SUMMARY/P19-E11** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L786): Add `elai doctor lint`.
- **SUMMARY/P19-E12** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L787): Give doctor modes stable exit codes suitable for CI.
- **SUMMARY/P19-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L788): Run post-upgrade compatibility probes.
- **SUMMARY/P19-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L789): Back up SSB/session SQLite state through WAL-safe `VACUUM INTO`.
- **SUMMARY/P19-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L790): Produce a backup manifest.
- **SUMMARY/P19-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L791): Verify the backup against its manifest.
- **SUMMARY/P19-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L792): Add a restart continuation queue.
- **SUMMARY/P19-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L793): Resume one in-flight turn once after restart.
- **SUMMARY/P19-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L794): Never split a tool call from its result during compaction.
- **SUMMARY/P19-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L795): Create a successor transcript during compaction.
- **SUMMARY/P19-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L796): Never rewrite the prior transcript in place.
- **SUMMARY/P19-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L797): Allowlist environment variables; never copy the full environment.
- **SUMMARY/P19-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L798): Adopt a working-directory path jail.
- **SUMMARY/P19-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L799): Case-fold paths before comparing them on Windows.
- **SUMMARY/P19-E25** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L800): Implement config LKG/rejected writes/guards plus the doctor command and backup/continuation machinery.
- **SUMMARY/P19-E26** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L801): Keep doctor as a structured-text CLI/ops surface with no new frontend.
- **SUMMARY/P19-E27** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L802): A >50%-shrink write is rejected, preserved, and lands `config_write_rejected`.
- **SUMMARY/P19-E28** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L803): `elai doctor lint` returns a stable nonzero exit for that bad state.
- **SUMMARY/P19-E29** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L804): Doctor emits structured `doctor_finding` rows.
- **SUMMARY/P19-E30** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L805): Killing mid-turn causes the in-flight turn to resume exactly once after restart.
- **SUMMARY/P19-E31** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L806): A shrink overwrite fails acceptance.
- **SUMMARY/P19-E32** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L807): A run-to-run-varying doctor exit code fails acceptance.
- **SUMMARY/P19-E33** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L808): Splitting a tool call from its result fails acceptance.
- **SUMMARY/P19-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L814): The stable exit-code map and structured `doctor_finding` schema are not enumerated.
- **SUMMARY/P19-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L815): Restore workflow, retention, backup encryption, storage destination, and verification algorithm are absent.
- **SUMMARY/P19-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L816): The summary says “one in-flight turn” but does not define behavior with multiple sessions/queued continuations.

### Summary unit `P-20` (34 records)

- **SUMMARY/P20-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L824): Insert a pre-compaction memory flush as an ordered, journaled step.
- **SUMMARY/P20-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L825): Use a silent `NO_REPLY`-equivalent turn to offer notes to durable memory.
- **SUMMARY/P20-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L826): Write durable notes before compaction.
- **SUMMARY/P20-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L827): Flush once per compaction cycle.
- **SUMMARY/P20-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L828): Forbid compaction from destroying context that was never offered to durable memory.
- **SUMMARY/P20-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L829): Keep this in-session flush distinct from H-51 cross-session sleep-time handoff.
- **SUMMARY/P20-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L830): Add a versioned, on-demand index shared by skills and rules.
- **SUMMARY/P20-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L831): Store skill/rule name in each compact index entry.
- **SUMMARY/P20-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L832): Store description in each compact index entry.
- **SUMMARY/P20-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L833): Store path in each compact index entry.
- **SUMMARY/P20-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L834): Store a SHA-256 version marker in each entry.
- **SUMMARY/P20-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L835): Target roughly 24 prompt tokens per skill entry.
- **SUMMARY/P20-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L836): Read the full skill/rule file only when needed.
- **SUMMARY/P20-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L837): Read that file once per unchanged version.
- **SUMMARY/P20-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L838): Re-read the file when its version changes.
- **SUMMARY/P20-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L839): Compact/degrade index detail when the context budget demands it.
- **SUMMARY/P20-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L840): Use one lazy-load mechanism for skills, rules, and MCP defer-loading.
- **SUMMARY/P20-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L841): Use Dreaming’s score, recall, and diversity promotion gates as prior art for H memory governance.
- **SUMMARY/P20-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L842): Add a per-provider cache-control mapping table.
- **SUMMARY/P20-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L843): Map Anthropic to `cache_control` with 5-minute/1-hour behavior.
- **SUMMARY/P20-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L844): Map OpenAI to `prompt_cache_key`.
- **SUMMARY/P20-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L845): Map Gemini to `cachedContents`.
- **SUMMARY/P20-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L846): Do not add a second memory store.
- **SUMMARY/P20-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L847): Add the flush hook, versioned loader, and provider mapping table to the context lifecycle.
- **SUMMARY/P20-E25** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L848): Add no new frontend beyond existing cost/memory panels.
- **SUMMARY/P20-E26** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L849): At threshold, `memory_flush_event` lands before the compaction row and carries durable notes.
- **SUMMARY/P20-E27** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L850): The initial prompt contains only compact index entries.
- **SUMMARY/P20-E28** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L851): Invoking a skill reads its file once in the trace.
- **SUMMARY/P20-E29** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L852): Compaction before flush fails acceptance.
- **SUMMARY/P20-E30** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L853): Loading a full skill body before invocation fails acceptance.
- **SUMMARY/P20-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L859): Memory note selection, failure/retry behavior, and what counts as “offered” are unspecified.
- **SUMMARY/P20-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L860): Compact-degrade levels and the actual token-measurement method are absent.
- **SUMMARY/P20-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L861): Dreaming’s score/recall/diversity thresholds are only cited as prior art, not defined.
- **SUMMARY/P20-A04** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L862): Provider cache behavior is temporally unstable and must be reverified at implementation time.

### Summary unit `P-21` (32 records)

- **SUMMARY/P21-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L870): Add verify-the-verifier seeded negative-control canaries to Q-6.
- **SUMMARY/P21-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L871): Construct known-bad artifacts for those canaries.
- **SUMMARY/P21-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L872): Run canaries through the same production detectors used on real work.
- **SUMMARY/P21-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L873): Hard-block the entire gate if a detector misses its canary.
- **SUMMARY/P21-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L874): Return exit code 1 for that whole-gate failure.
- **SUMMARY/P21-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L875): Treat a missing artifact as FAIL by default.
- **SUMMARY/P21-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L876): Permit a missing-artifact escape only when explicitly represented in the manifest.
- **SUMMARY/P21-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L877): Restrict `missingVerdict` escape reasons to `environment-blocked` or `optional-gap`.
- **SUMMARY/P21-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L878): Apply `skipBackfill` as a cross-lane dependency.
- **SUMMARY/P21-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L879): Clear a lane with skipped rows only if a named sibling lane passed.
- **SUMMARY/P21-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L880): Make audit/export output a closed metadata struct.
- **SUMMARY/P21-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L881): Exclude content-bearing fields structurally so secrets have nowhere to appear.
- **SUMMARY/P21-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L882): Hash identifiers instead of storing their content.
- **SUMMARY/P21-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L883): Restrict fields to a whitelist or the value `unknown`.
- **SUMMARY/P21-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L884): Keep the shareable metadata projection and content-carrying debug bundle separate by type.
- **SUMMARY/P21-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L885): Add smoke-evidence posture labels.
- **SUMMARY/P21-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L886): Support posture `user-path`.
- **SUMMARY/P21-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L887): Support posture `direct-gateway`.
- **SUMMARY/P21-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L888): Support posture `native-approval`.
- **SUMMARY/P21-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L889): Fail smoke evidence that lacks a posture label.
- **SUMMARY/P21-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L890): Apply `unknown=FAIL`.
- **SUMMARY/P21-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L891): Do not allow a classified product bug to produce a green report by default.
- **SUMMARY/P21-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L892): Add the canary lane/missing escape to Q-6, posture to R10 evidence, and closed export type to G-9.
- **SUMMARY/P21-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L893): Reuse the existing gate/evidence UI surface.
- **SUMMARY/P21-E25** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L894): A deliberately broken canary detector makes the self-test and whole gate fail with exit 1.
- **SUMMARY/P21-E26** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L895): The audit-export type has no content field.
- **SUMMARY/P21-E27** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L896): A missed canary that still permits PASS fails acceptance.
- **SUMMARY/P21-E28** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L897): A classified failure that defaults to green fails acceptance.
- **SUMMARY/P21-E29** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L898): Content in the metadata-only export fails acceptance.
- **SUMMARY/P21-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L904): Canary corpus contents, seeding cadence, detector coverage, and required kill-rate are not enumerated here.
- **SUMMARY/P21-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L905): `optional-gap` and `environment-blocked` admission criteria are named but not defined.
- **SUMMARY/P21-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L906): Debug-bundle custody/access policy is outside this summary.

### Summary unit `P-22` (35 records)

- **SUMMARY/P22-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L914): Treat child/subagent output as fenced evidence data.
- **SUMMARY/P22-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L915): Make a subagent return structurally unable to carry executable instructions.
- **SUMMARY/P22-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L916): Treat every subagent return as a G-1 taint source.
- **SUMMARY/P22-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L917): Apply depth-scoped default tool stripping.
- **SUMMARY/P22-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L918): Remove gateway authority from subagents by default.
- **SUMMARY/P22-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L919): Remove cron authority from subagents by default.
- **SUMMARY/P22-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L920): Remove messaging authority from subagents by default.
- **SUMMARY/P22-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L921): Remove spawn authority from leaf agents.
- **SUMMARY/P22-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L922): Add a deterministic nine-step channel-binding precedence.
- **SUMMARY/P22-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L923): Resolve channel binding by most-specific-wins.
- **SUMMARY/P22-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L924): Never let the model choose the channel.
- **SUMMARY/P22-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L925): Add pairing codes that are short-lived.
- **SUMMARY/P22-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L926): Cap pending pairing requests.
- **SUMMARY/P22-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L927): Add `dmScope` session-isolation options for P-10 ingress/front-door/M enrollment.
- **SUMMARY/P22-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L928): Pair devices through a challenge-signed flow.
- **SUMMARY/P22-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L929): Bind the signature payload to the platform/device context.
- **SUMMARY/P22-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L930): Return stable authentication error codes.
- **SUMMARY/P22-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L931): Withhold an unknown event family from broadcast rather than leak it.
- **SUMMARY/P22-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L932): Require idempotency keys on every side-effecting RPC.
- **SUMMARY/P22-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L933): Deduplicate a replayed mutating RPC by that idempotency key.
- **SUMMARY/P22-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L934): Order a trusted-policy hook tier before all third-party hooks.
- **SUMMARY/P22-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L935): Prevent plugins from preempting ELAI’s frozen gates.
- **SUMMARY/P22-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L936): Validate plugin manifests without executing them.
- **SUMMARY/P22-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L937): Reuse the existing N-6 provenance scale; do not fork another.
- **SUMMARY/P22-E25** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L938): Wire fenced returns/tool stripping, signed pairing/unknown gating/idempotency, channel precedence, and hook ordering into their existing seams.
- **SUMMARY/P22-E26** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L939): Render enrollment and pairing UX on the control surface at Tier 2.
- **SUMMARY/P22-E27** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L940): Instruction-shaped subagent text lands as a fenced taint-source row and remains inert.
- **SUMMARY/P22-E28** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L941): An unknown broadcast event is withheld.
- **SUMMARY/P22-E29** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L942): A replayed mutating RPC is applied at most once.
- **SUMMARY/P22-E30** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L943): A subagent return that steers its parent fails acceptance.
- **SUMMARY/P22-E31** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L944): Leaking an unknown event family fails acceptance.
- **SUMMARY/P22-E32** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L945): Double-applying a mutating RPC fails acceptance.
- **SUMMARY/P22-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L951): The nine binding steps and their exact specificity keys are not listed.
- **SUMMARY/P22-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L952): `dmScope` option values, code lifetimes, pending caps, signature algorithm, and platform payload schema are absent.
- **SUMMARY/P22-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L953): “Instructions structurally unable to carry” requires a concrete fenced-data encoding not shown here.

### Summary unit `P-23` (38 records)

- **SUMMARY/P23-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L576): Add a general co-browse surface as a canvas browse node over the existing E-14 engine.
- **SUMMARY/P23-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L577): Tier 1 performs read-only extraction for requests such as pulling up documentation.
- **SUMMARY/P23-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L578): Tag Tier-1 page extraction as tainted ingress.
- **SUMMARY/P23-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L579): Route read-only pointing/deixis through H-15.
- **SUMMARY/P23-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L580): Tier 2 performs co-browse through a CDP screencast.
- **SUMMARY/P23-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L581): Keep the visible pane as dumb glass; engine-owned browser state remains authoritative.
- **SUMMARY/P23-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L582): Interpose exactly one `InputProxy` with a private engine input handle.
- **SUMMARY/P23-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L583): Make bypassing `InputProxy` structurally uncompileable.
- **SUMMARY/P23-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L584): Journal navigation before dispatch.
- **SUMMARY/P23-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L585): Journal clicks before dispatch.
- **SUMMARY/P23-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L586): Journal keystrokes before dispatch.
- **SUMMARY/P23-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L587): Tag every input with the actor, covering both user and model.
- **SUMMARY/P23-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L588): Make session persistence a typed choice.
- **SUMMARY/P23-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L589): Default sessions to ephemeral.
- **SUMMARY/P23-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L590): Hold named persistent profiles in P-10 credential-vault custody.
- **SUMMARY/P23-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L591): Gate outbound typed text through P-6 privacy.
- **SUMMARY/P23-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L592): Show a domain-first-seen warning before outbound text reaches a newly seen domain.
- **SUMMARY/P23-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L593): Have the model point through P-13 element IDs.
- **SUMMARY/P23-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L594): Permit coordinate fallback only under P-12 frame binding.
- **SUMMARY/P23-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L595): Exclude native Tauri multiwebview as the primary mechanism because its cargo flag is unstable.
- **SUMMARY/P23-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L596): Permit native multiwebview only as a latency-triplined fork.
- **SUMMARY/P23-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L597): Keep `InputProxy` interposed even in that fork.
- **SUMMARY/P23-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L598): Never create a second browser substrate; thread the E-14 contract if E-14 is not executed at edit time.
- **SUMMARY/P23-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L599): Allow a co-browse exploration to crystallize through P-14 into an RRA-bound replay script.
- **SUMMARY/P23-E25** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L600): Add `crates/orchestrator/src/cobrowse/{mod.rs,input_proxy.rs}`.
- **SUMMARY/P23-E26** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L601): Expose `elai browse <url>` and a front-door browse lane.
- **SUMMARY/P23-E27** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L602): Wire the CLI, front door, and H-13 canvas dispatcher in the same changeset.
- **SUMMARY/P23-E28** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L603): Render screencast/extraction plus an actor-colored input trail.
- **SUMMARY/P23-E29** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L604): The fixture run lands one `cobrowse_session` row.
- **SUMMARY/P23-E30** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L605): Every dispatched input has one prior actor-tagged `cobrowse_input` row.
- **SUMMARY/P23-E31** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L606): Check journal-before-dispatch ordering off-agent against the engine CDP log.
- **SUMMARY/P23-E32** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L607): An ephemeral session leaves zero profile bytes.
- **SUMMARY/P23-E33** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L608): Any page input without a prior row fails acceptance.
- **SUMMARY/P23-E34** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L609): A model coordinate action against a stale frame hash fails acceptance.
- **SUMMARY/P23-E35** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L610): A persistent profile outside vault custody fails acceptance.
- **SUMMARY/P23-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L616): The latency tripline and conditions for the native-multiwebview fork are not quantified.
- **SUMMARY/P23-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L617): Cookie/download/upload/pop-up/permission behavior is not summarized.
- **SUMMARY/P23-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L618): P-10 profile-custody behavior cannot be fully traced because P-10 is absent from the summary.

### Summary unit `P-24` (48 records)

- **SUMMARY/P24-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L626): Model mobile as a typed dimension of existing design/test/operator lanes, not a parallel subsystem.
- **SUMMARY/P24-E02** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L627): Cover the React Native/Expo gap explicitly deferred by H-14.
- **SUMMARY/P24-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L628): Make the existing `ConsumerMobileApp` product type previewable.
- **SUMMARY/P24-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L629): Make that product type click-editable at the supported rung.
- **SUMMARY/P24-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L630): Make that product type testable.
- **SUMMARY/P24-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L631): Implement three rungs in cheapest-first order.
- **SUMMARY/P24-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L632): Rung 1 adds a typed `DeviceProfile` catalog as data rows.
- **SUMMARY/P24-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L633): Rung 1 adds a `device_frame` render variant to H-14 preview nodes.
- **SUMMARY/P24-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L634): Rung 1 adds the same device-frame dimension to P-23 browse nodes.
- **SUMMARY/P24-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L635): Rung 1 runs E-14 journeys across a device matrix.
- **SUMMARY/P24-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L636): Rung 1 maintains per-profile SSIM goldens using CDP metrics emulation and in-repo capture precedent.
- **SUMMARY/P24-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L637): Rung 2 treats an Expo development server as a frameable web origin.
- **SUMMARY/P24-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L638): Rung 2 reuses H-14 proxying.
- **SUMMARY/P24-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L639): Rung 2 reuses `data-oid` tagging.
- **SUMMARY/P24-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L640): Rung 2 reuses Recast writeback over React-Native-Web output.
- **SUMMARY/P24-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L641): A click in Rung 2 resolves to React Native component AST ranges.
- **SUMMARY/P24-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L642): Rung 3 treats a native emulator as another P-12 sandbox backend.
- **SUMMARY/P24-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L643): Boot the emulator from a pinned snapshot.
- **SUMMARY/P24-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L644): Include the snapshot hash in the RRA hardware/environment pin.
- **SUMMARY/P24-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L645): Add `mobile_axtree` as a P-13 source tag.
- **SUMMARY/P24-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L646): Obtain mobile accessibility from UIAutomator/XCUITest accessibility data.
- **SUMMARY/P24-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L647): Use pixel perception only when the mobile accessibility tree is empty, preserving J-1.
- **SUMMARY/P24-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L648): Crystallize a Maestro YAML flow through P-14.
- **SUMMARY/P24-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L649): Treat the Maestro flow as actuation-only.
- **SUMMARY/P24-E25** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L650): Obtain replay verdicts from the same off-agent SSB/screenshot evidence leg as the minting run.
- **SUMMARY/P24-E26** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L651): Provision emulator, SDK, and Maestro through P-16.
- **SUMMARY/P24-E27** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L652): Attach `render_source` to every mobile result row.
- **SUMMARY/P24-E28** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L653): The render-source enum is `web_viewport | rnw_web | native_emulator`.
- **SUMMARY/P24-E29** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L654): Never let a lower-rung pass discharge a higher-rung gate.
- **SUMMARY/P24-E30** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L655): Represent “mobile tested” as a typed claim.
- **SUMMARY/P24-E31** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L656): Class physical devices as ABORT because they are neither disposable nor snapshot-pinnable.
- **SUMMARY/P24-E32** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L657): Keep native React Native click-to-edit through a Fiber inspector as a registered Rung-3 follow-up/fork.
- **SUMMARY/P24-E33** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L658): Do not front-run H-14; extend its proxy, tagging, and writeback only after it exists.
- **SUMMARY/P24-E34** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L659): Add `crates/orchestrator/src/mobile/{device_profile.rs,emulator_backend.rs}`.
- **SUMMARY/P24-E35** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L660): Expose `elai e2e run --device` and `elai operator run --backend emulator`.
- **SUMMARY/P24-E36** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L661): Add `device_profile` to task-runner stages.
- **SUMMARY/P24-E37** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L662): Add the H-13 `device_frame` render variant.
- **SUMMARY/P24-E38** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L663): Render a phone-bezel node plus per-profile pass/fail chips.
- **SUMMARY/P24-E39** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L664): On systems without macOS, render iOS as an honest-gap affordance rather than silently skip it.
- **SUMMARY/P24-E40** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L665): The web fixture emits per-profile `axtree_snapshot` rows tagged with `device_profile` and `render_source`.
- **SUMMARY/P24-E41** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L666): A Rung-3 run lands `emulator_session {snapshot_hash, backend}`.
- **SUMMARY/P24-E42** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L667): That run crystallizes an RRA-bound Maestro flow.
- **SUMMARY/P24-E43** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L668): Presenting an `rnw_web` pass against a native gate fails acceptance.
- **SUMMARY/P24-E44** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L669): An unpinned emulator image fails acceptance.
- **SUMMARY/P24-E45** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L670): A VLM-sourced mobile count fails acceptance.
- **SUMMARY/P24-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L676): Native Fiber-inspector click-to-edit is explicitly deferred inside a Rung-3 fork.
- **SUMMARY/P24-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L677): Physical-device support is explicitly outside the executable lane unless its non-disposable/non-pinnable problem is redesigned.
- **SUMMARY/P24-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L678): Device-profile fields, SSIM thresholds, matrix coverage, and Android/iOS backend support levels are not enumerated.

### Summary unit `P-25` (63 records)

- **SUMMARY/P25-E01** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L961): Test both surviving creativity-lever families on one frozen corpus.
- **SUMMARY/P25-E02** [E] [SURFACE] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L962): Define the corpus as K=30 open-world real ELAI improvement fixtures.
- **SUMMARY/P25-E03** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L963): Give fixtures hidden deterministic micro-executors.
- **SUMMARY/P25-E04** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L964): Treat B-CREATIVITY-1 corpus design as specified but never built at summary time.
- **SUMMARY/P25-E05** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L965): Run six arms total.
- **SUMMARY/P25-E06** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L966): Use two independently preregistered joint gates, one per family.
- **SUMMARY/P25-E07** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L967): Forbid cherry-picking between the two gates.
- **SUMMARY/P25-E08** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L968): Declare multiplicity explicitly.
- **SUMMARY/P25-E09** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L969): Family A calls one direct champion conditioned on exactly k=3 retrieved artifacts.
- **SUMMARY/P25-E10** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L970): Retrieve Family-A artifacts deterministically.
- **SUMMARY/P25-E11** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L971): Make Family-A retrieval model-free.
- **SUMMARY/P25-E12** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L972): Require retrieved artifacts to be target-relevant and verified.
- **SUMMARY/P25-E13** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L973): Draw the three artifact classes from an `elai.map` anchor slice, a measured bench/journal row, and an E-48 `VerifiedCitation`.
- **SUMMARY/P25-E14** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L974): Compare injection against a `DIRECT` arm.
- **SUMMARY/P25-E15** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L975): Compare against cross-subsystem-stratified `EXO_SHUFFLED`.
- **SUMMARY/P25-E16** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L976): Compare against `PAD_MATCHED`.
- **SUMMARY/P25-E17** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L977): Apply a hard 1.3× Family-A cost cap.
- **SUMMARY/P25-E18** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L978): Permit zero preprocessing model calls; that absence is part of the mechanism.
- **SUMMARY/P25-E19** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L979): Family B `TRIAGE_REVISE` first generates candidates.
- **SUMMARY/P25-E20** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L980): Run real deterministic public triage legs over the candidates.
- **SUMMARY/P25-E21** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L981): Include demarcation, anchor, and composition triage legs.
- **SUMMARY/P25-E22** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L982): Exclude the ledger triage leg.
- **SUMMARY/P25-E23** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L983): Make the exogenous-card grammar emit the fields being judged.
- **SUMMARY/P25-E24** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L984): Reconcile `KEY_ALIASES` before judging.
- **SUMMARY/P25-E25** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L985): Feed each candidate’s actual kill reasons into one revision call.
- **SUMMARY/P25-E26** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L986): Compare against `SHUFFLED_REVISE` with the identical template.
- **SUMMARY/P25-E27** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L987): Permute kill reasons away from their true candidates in the shuffled control.
- **SUMMARY/P25-E28** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L988): Apply a hard 2.5× Family-B cost cap.
- **SUMMARY/P25-E29** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L989): Require cost-normalized P1 non-inferiority.
- **SUMMARY/P25-E30** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L990): Fail a Family-B win bought by approximately double spend.
- **SUMMARY/P25-E31** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L991): Use execution-validated mechanism yield as a co-primary for each family.
- **SUMMARY/P25-E32** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L992): Use best target-value delta as the other co-primary.
- **SUMMARY/P25-E33** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L993): Evaluate co-primaries with paired bootstrap 95% confidence intervals.
- **SUMMARY/P25-E34** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L994): Do not add packetization, analogy, archive conditioning, operator rotation, or probes to this bench.
- **SUMMARY/P25-E35** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L995): Distinguish sequential evidence-coupled revision from previously tested archive cycles and parallel context-blind calls.
- **SUMMARY/P25-E36** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L996): Record rejection-ledger clearance.
- **SUMMARY/P25-E37** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L997): Preserve two red-team passes—authoring and delta—and patch all BLOCK findings before execution.
- **SUMMARY/P25-E38** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L998): Import Q-10 Tier-0 instrument modules rather than enlarge the runner beyond 1,000 lines.
- **SUMMARY/P25-E39** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L999): Use `conjecture_triage.py` kill legs as Family-B evidence.
- **SUMMARY/P25-E40** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1000): Use the research-creativity analysis/verifier.
- **SUMMARY/P25-E41** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1001): Use `elai.map` and E-48 `VerifiedCitation` rows as read-only substrate.
- **SUMMARY/P25-E42** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1002): Use the grounding-as-exit doctrine as the Family-B design source.
- **SUMMARY/P25-E43** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1003): Integrate planned `.claude/tools/exo_retrieval.py` and `.claude/tools/exo_injection_ab.py`.
- **SUMMARY/P25-E44** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1004): Implement a six-arm, two-call revision lifecycle.
- **SUMMARY/P25-E45** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1005): Freeze the triage-render template and fixture corpus.
- **SUMMARY/P25-E46** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1006): Put nothing in `.elai_cc` for this harness-only bench.
- **SUMMARY/P25-E47** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1007): Produce a sealed evidence directory.
- **SUMMARY/P25-E48** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1008): Require an independent checker to exit 0.
- **SUMMARY/P25-E49** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1009): Include a byte-level check of revision input.
- **SUMMARY/P25-E50** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1010): If Family A is supported, create a separate adoption unit for an improvement-conjecture stage-1 exogenous leg plus paper-to-plan seam.
- **SUMMARY/P25-E51** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1011): If Family B is supported, create a separate adoption unit for a triage-feedback revision leg.
- **SUMMARY/P25-E52** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1012): If both families are null/refuted, close the creativity line entirely.
- **SUMMARY/P25-E53** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1013): A retrieval-time model call fails the bench.
- **SUMMARY/P25-E54** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1014): A triage-time model call fails the bench.
- **SUMMARY/P25-E55** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1015): Hidden-side data in revision input fails the bench.
- **SUMMARY/P25-E56** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1016): A second revision round fails the bench.
- **SUMMARY/P25-E57** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1017): A shuffled arm matching treatment relevance fails the bench.
- **SUMMARY/P25-E58** [E] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1018): A cost row beyond its family cap without G3 firing fails the bench.
- **SUMMARY/P25-A01** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1024): The pilot’s 3.5× anchor gain supports injection mechanics only; both creative-yield effects remain hypotheses.
- **SUMMARY/P25-A02** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1025): The K=30 B-CREATIVITY-1 corpus is explicitly described as never built in this summary.
- **SUMMARY/P25-A03** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1026): The named integration files `exo_retrieval.py` and `exo_injection_ab.py` are not present at those exact paths in the current tree; newer P-25/EXO candidate tools exist and require detail-source reconciliation.
- **SUMMARY/P25-A04** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1027): Exact gate formulas, P1 definition, target-value metric, bootstrap resampling plan, and G3 behavior are not reproduced here.
- **SUMMARY/P25-A05** [A] (summary-line: 16-P-SUMMARY_INDEX.discovery.md:L1028): “Six-arm, two-call revision lifecycle” can be reconciled as four Family-A arms plus two Family-B arms, but the phrase “two-call” needs the detail plan to disambiguate per-candidate versus per-bench calls.

## Cross-file interaction records (IX-*)

Namespaced by deep ledger. Records reference feature atoms and are not double-counted as features in the deep footers; they are distinct aggregate records.

### Interactions — P-001 (20)

- **P-001/IX-001** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L382): P-0 is the doctrine/catalog parent: every detail unit inherits W-1..W-10, GD-1..GD-5, A-7 spine-after-authoring, SCHEMA_VERSION discipline, and carried corrections. _(evidence: P0-E026–P0-E043, P0-E044–P0-E058)_
- **P-001/IX-002** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L383): Personal WAVE 1 pairs P-5 ceremony with P-1 KEYSTONE fleet; P-5 renders P-1 red ticks and defers repair button to P-1 Tier-2. _(evidence: P0-E028, P0-E041; P5-E002, P5-E007–P5-E008)_
- **P-001/IX-003** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L384): P-4 consumes P-6 life/work/shared routing wall + E-20 at vault boundary; P-0 places both in WAVE 2 with P-4 after Obsidian RECON. _(evidence: P0-E031–P0-E033, P0-E041; P4-E005, P4-E008, P4-E016)_
- **P-001/IX-004** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L385): P-4 concurrent write-back merge is deferred as kin to P-8 shared-journal kernel; both are local-first merge design spikes. _(evidence: P4-E011; P8-E006, P8-E015)_
- **P-001/IX-005** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L386): P-8 seat_id/agent_id provenance is said to serve P-1 fleet attribution and is targeted by P-7 team default. _(evidence: P8-E020; P0-E034–P0-E035)_
- **P-001/IX-006** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L387): Computer-use dependency chain: P-13 CU-1 feeds P-12 perception; P-12 depends P-13; joint R10 smoke after both land. _(evidence: P12-E001, P12-E022, P12-E020; P13-E009, P13-E019)_
- **P-001/IX-007** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L388): P-16 resolves installs to P-9 headless packs and uses P-12 only as GUI last-resort FORK; secrets via P-10 vault. _(evidence: P16-E003–P16-E004, P16-E016; P0-E014, P0-E054)_
- **P-001/IX-008** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L389): P-18 hardens P-11 confirm gate and O-2 run store; smoke requires P-11 multi-step confirm under 4G. _(evidence: P18-E001, P18-E004, P18-E021)_
- **P-001/IX-009** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L390): P-21 extends Q-6 and feeds PS-6; P-0 lists P-21 among proof-spine feeders with P-14/P-15/P-17. _(evidence: P21-E001, P21-E004; P0-E022, P0-E024)_
- **P-001/IX-010** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L391): Frame-binding (P-12) + id-indirection (P-13) compose: model never owns coordinates; harness geometry + live frame hash authorize actuation. _(evidence: P12-E004, P12-E013; P13-E002, P13-E016)_
- **P-001/IX-011** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L392): Operator actions (P-12) and install GUI fallback (P-16) are G-1 taint operator lanes, never load-bearing correctness evidence. _(evidence: P12-E009; P16-E016; P0-E050)_
- **P-001/IX-012** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L393): All units restate off-agent evidence / missing-evidence=FAIL / test-caller≠R7 / auditor wiring_contract_check pattern from P-0 DONE definition. _(evidence: P0-E036, P0-E039; P4-E019–P4-E020; P5-E019–P5-E020; P8-E016–P8-E017; P12-E008,P12-E018; P13-E018; P16-E020; P18-E020; P21-E018)_
- **P-001/IX-013** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L394): SSB action_type map in P-0 assigns row families used by P-4/P-5/P-8/P-12/P-13/P-16/P-18/P-21 events. _(evidence: P0-E040; P4-E023; P5-E011; P8-E012; P12-E011; P13-E013; P16-E013; P18-E013; P21-E013)_
- **P-001/IX-014** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L395): P-5 observe-only + P-21 canary hard-block are opposite ceremony vs credibility poles on the same trust spine: ceremony may warn-not-block at toy; gate corpus still must bite. _(evidence: P5-E004, P5-E012; P21-E005, P21-E014)_
- **P-001/IX-015** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L396): P-8 forbids LWW merge; P-18 forbids LWW run-store clobber — shared local-first concurrency doctrine. _(evidence: P8-E004, P8-E009; P18-E005, P18-E011)_
- **P-001/IX-016** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L397): Deep red-team deferred to execution pre-flight for steal-track units (P-12/P-13/P-16/P-18/P-21) while personal units carry inline REDTEAM seeds. _(evidence: P0-E024; P12-E019; P13-E020; P16-E018; P18-E022; P21-E020)_
- **P-001/IX-017** [IX] class=`I` (ledger-line: DEEP-P-001.discovery.md:L398): P-0 second-wave P-9 KEYSTONE + P-16 dependency implies CU-3 install work cannot land before command-center packs even if CU waves are sequenced separately. _(evidence: P0-E005, P0-I001; P16-I001)_
- **P-001/IX-018** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L399): Hashing: P-0 hard-rule blake3; P-12 FrameBinding text uses sha256; P-18 plan hash blake3; conflict must resolve at edit without inventing dual algorithms. _(evidence: P0-E043; P12-A003; P18-E011)_
- **P-001/IX-019** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L400): User-facing CLI verb set in this chunk: `elai brain mount\|sync`, `elai profile set\|tax`, `elai world join\|merge\|whodid`, `elai operator run`, `elai perception scaffold`, `elai connector install`, `elai workflow resume\|cancel`, `elai bench gate-corpus`. _(evidence: P4-E013; P5-E009; P8-E008; P12-E008; P13-E012; P16-E007; P18-E009; P21-E012)_
- **P-001/IX-020** [IX] class=`E` (ledger-line: DEEP-P-001.discovery.md:L401): Dashboard/frontend pairs in this chunk: P-5 fleet operator panel (Tier-2); P-12 drawer thumbnail; P-13 perception tile reuse; P-16 connector surface (Tier-2); P-18 O-4 revision/drift rows; P-21 existing gate/evidence surface. _(evidence: P5-E010; P12-E010; P13-E011; P16-E012; P18-E012; P21-E011)_

### Interactions — P-002 (22)

- **P-002/IX-001** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L394): P-1 is the KEYSTONE: AppManifest is the bind point for P-2 ports, P-6 namespace validation, L-14 RRA lineage, and P-5 operator fleet render. _(evidence: P1-E002–P1-E004, P1-E029; P2-E003; P6-E017)_
- **P-002/IX-002** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L395): P-2 extends P-1 manifests with PortSchema and defers O-4 canvas edge chrome to Tier-2; dispatch remains FSM (O-4 doctrine), never a bus runtime. _(evidence: P2-E002–P2-E010, P2-E017)_
- **P-002/IX-003** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L396): P-6 domain_label on ports and AppManifest data_namespaces share the life\|work\|shared vocabulary; P-1 refuses unclaimed namespaces at register via O-3 claims. _(evidence: P1-E017; P2-E003; P6-E003–P6-E005)_
- **P-002/IX-004** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L397): P-6 OUT-flow privacy is the mirror of G-1 IN-flow taint; both ride N-6's ONE lattice (N-2 forbids a second scale). _(evidence: P6-E002–P6-E006, P6-E014)_
- **P-002/IX-005** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L398): P-7 connected default turns P-2 world bus on; toy default targets P-5 ceremony; team targets M org pack + P-8 — one engine, three data defaults. _(evidence: P7-E008; P2-E001; P1-E016)_
- **P-002/IX-006** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L399): P-7 process adaptation is deliberately disjoint from E-5/E-4 content adaptation and extends D-14/N-15 anti-sycophancy to process grain via M-2 asymmetry. _(evidence: P7-E002–P7-E005)_
- **P-002/IX-007** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L400): Command-center triad: P-9 owns connector-pack format + http_call; P-10 owns ingress adapters + CredentialVault auth plane; P-11 is the from-goal front door composing both for the user. All three execute after the proof spine. _(evidence: P9-E001–P9-E003, P9-E023; P10-E001–P10-E002, P10-E020; P11-E001–P11-E002, P11-E021)_
- **P-002/IX-008** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L401): P-9 http_call taints Untrusted at ingest (G-1), validates to PortSchema (P-2), pins egress to signed declared hosts (G-3 compose), and mounts only through existing L-12/skillspector/J-102/N-6 admission. _(evidence: P9-E007–P9-E009, P9-E020)_
- **P-002/IX-009** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L402): P-10 normalizes all external shapes to O-2 SsbEvent without growing the trigger enum; WebhookIngress lives on I-1 supervisor plane, not the pipeline hot path. _(evidence: P10-E002–P10-E006, P10-E010)_
- **P-002/IX-010** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L403): P-10 CredentialVault must never leak into SSB/RRA/P-3 export and is the secret home P-19's env-allowlist patch points at (secrets not in config/journal). _(evidence: P10-E008, P10-E013; P19-E016)_
- **P-002/IX-011** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L404): P-11 proposals are constrained to O A3 closed kinds + P-9 mounted connectors; unmounted needs render as O-4 wire-dark gaps offering P-9 `elai connector new`, never fabricated nodes or auto-authored packs. _(evidence: P11-E002–P11-E007, P11-E011; P9-E010)_
- **P-002/IX-012** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L405): P-11 confirm-before-run keeps proposed TaskSpec rows inert until human arms O-2 triggers; P-18 later hardens this confirmation seam (referenced by summary). _(evidence: P11-E008, P11-E012)_
- **P-002/IX-013** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L406): P-14 Tier-1 crystallizes E-14 browser traces into O-2 crystallized_script stages with G-2 RRA binding; P-24 Rung 3 reuses the same crystallizer arm for Maestro YAML. _(evidence: P14-E004–P14-E011; P24-E009, P24-E027)_
- **P-002/IX-014** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L407): P-14 and P-15 both mint reuse artifacts feeding PS-5; openclaw mock-replay is explicitly excluded (C4). _(evidence: P14-E001–P14-E003, P14-E019)_
- **P-002/IX-015** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L408): P-19 Tier-1 extends J config-ingress with recoverable guards and `elai doctor`; Tier-2 backups/continuation protect the same evidence store other units journal into. _(evidence: P19-E005–P19-E006, P19-E019)_
- **P-002/IX-016** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L409): P-24 threads DeviceProfile/device_frame/mobile_axtree/emulator backend through H-14, E-14, P-12, P-13, P-14, P-16, P-23 without forking a mobile subsystem. _(evidence: P24-E002–P24-E010, P24-E026)_
- **P-002/IX-017** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L410): P-24 render_source honesty (`web_viewport\|rnw_web\|native_emulator`) is the structural anti-laundering control analogous to P-2/P-11 wire-dark honest gaps. _(evidence: P24-E017–P24-E019; P2-E006; P11-E011)_
- **P-002/IX-018** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L411): Shared production-caller doctrine: every unit forbids `#[cfg(test)]`/contracts callers from satisfying R7 and names real CLI and/or scheduler/FSM production hooks. _(evidence: P1-E024; P2-E018; P6-E015; P7-E014; P9-E017; P10-E014; P11-E015; P14-E008; P19-E010–P19-E011; P24-E027)_
- **P-002/IX-019** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L412): Shared off-agent verification pattern: wiring_contract_check (+ often production_caller_check) and R10 multi-path INTEGRATION_SMOKE; P-6 additionally requires N-2 corpus_composition_check. _(evidence: P1-E025–P1-E026; P6-E016–P6-E017; P9-E018–P9-E019)_
- **P-002/IX-020** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L413): Shared anti-second-substrate doctrine: no second scheduler (P-1), no second lattice (P-6), no second provenance scale (P-2), no second dispatch runtime (P-14), no parallel mobile subsystem (P-24), no open stage vocabulary (P-9/P-11). _(evidence: P1-E009; P2-E008; P6-E006; P9-E014; P11-E007; P14-E010; P24-E012)_
- **P-002/IX-021** [IX] class=`I` (ledger-line: DEEP-P-002.discovery.md:L414): Dashboard DESIGN.md surfaces implied by this chunk: DashboardInbox red-tick cards (P-1), deferred FleetPanel (P-5), O-4 port edges + disconnected chrome (P-2 Tier-2), deferred privacy grant panel (P-5/P-6), O-4 proposed-graph + confirm UI (P-11), crystallized-stage node (P-14 Tier-2), device_frame + pass/fail chips + iOS honest-gap (P-24), intentionally no doctor dashboard tile (P-19). _(evidence: P1-E016, P1-E020; P2-E012; P6-E008; P11-E011–P11-E012; P14-E009; P19-E010; P24-E016)_
- **P-002/IX-022** [IX] class=`E` (ledger-line: DEEP-P-002.discovery.md:L415): Post-spine execution gate is uniform for product-surface units in this chunk (especially P-9/P-10/P-11 and steal-track units); personal-wave units are owner-acked WAVE 1–3 but still product-surface class. _(evidence: P1-E001; P9-E001; P10-E001; P11-E001; P14-E001; P19-E001; P24-E001)_

### Interactions — P-003 (19)

- **P-003/IX-001** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L320): P-3 export/import composes P-1 AppManifest + P-2 ports as the mechanism payload; persona regen (Tier-2) needs E-5 memory + P-7 profile. _(evidence: P3-E003, P3-E011, P3-E012, P3-E024)_
- **P-003/IX-002** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L321): P-3 import re-verification rides M-5/M-6; admission rides L-12/skillspector; provenance redaction rides G-9; RRA/lineage rides L-14/G-2. _(evidence: P3-E007–P3-E009, P3-E011)_
- **P-003/IX-003** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L322): P-15 extends closed J-96 CaseNode/MintOnProof; reuse-hit feeds K-18/PS-5; bBoN (Tier-2) slots behind I evidence-ranked search after receipts. _(evidence: P15-E003–P15-E006, P15-E011)_
- **P-003/IX-004** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L323): P-15 and P-14 are co-minters of reuse artifacts feeding K-18; share CU-track R10 cadence. _(evidence: P15-E017)_
- **P-003/IX-005** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L324): P-17 hardens J free-API rotation + I-50 auth + J-28 ledger; feeds K-2/PS-4; substitutions align with Q-4; B-37 consumes fallback chains. _(evidence: P17-E001–P17-E002, P17-E010–P17-E012)_
- **P-003/IX-006** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L325): P-17 session stickiness/rotation-as-cache-invalidation composes H cache-stable layout thesis. _(evidence: P17-E007)_
- **P-003/IX-007** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L326): P-20 flush is in-session ordered pre-step for H-51 CompactionPass; target is H two-tier DurableMemory; pairs with P-19 compaction split-rule; H-51 remains cross-session sibling. _(evidence: P20-E002, P20-E008, P20-E017)_
- **P-003/IX-008** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L327): P-20 on-demand index unifies L familiarity skills/rules with H MCP defer-load as one lazy-load mechanism. _(evidence: P20-E004, P20-E008)_
- **P-003/IX-009** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L328): P-22 Tier-1 fenced return is G-1 taint discipline at the 3C agent boundary; tool-strip binds AgentSpawn. _(evidence: P22-E002–P22-E004, P22-E011–P22-E012)_
- **P-003/IX-010** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L329): P-22 Tier-2 channel-binding/pairing/idempotency/hooks compose P-10 ingress, J front-door/control plane, M seats, 4C plugins. _(evidence: P22-E005–P22-E007, P22-E011)_
- **P-003/IX-011** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L330): P-23 reuses E-14 engine (or vendors then E-14 consumes); H-15 view_type + H-13 dispatcher; P-13 ids + P-12 frame bind; P-14 crystallize; P-6 egress; P-10 vault; G-1/G-2. _(evidence: P23-E003–P23-E009, P23-E012, P23-E024)_
- **P-003/IX-012** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L331): P-23 front-door "pull up X" and `elai browse` are co-equal production entrypoints with canvas browse node. _(evidence: P23-E013–P23-E015, P23-E025)_
- **P-003/IX-013** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L332): P-25 is harness-only and must not edit .elai_cc; adoption units (improvement-conjecture / paper-to-plan) are separate and conditional on SUPPORTED family. _(evidence: P25-E024, P25-E026, P25-E043)_
- **P-003/IX-014** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L333): P-25 FAMILY B instantiates grounding-as-exit at creativity grain; FAMILY A continues pilot P-A exogenous-injection lever; both supersede refuted Q-10 mechanism without reusing its runner. _(evidence: P25-E003–P25-E004, P25-E042)_
- **P-003/IX-015** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L334): P-25 retrieval substrate: elai.map anchors, sealed bench rows, E-48 VerifiedCitation, feature-map ledger — read-only. _(evidence: P25-E008, P25-E042)_
- **P-003/IX-016** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L335): Across P-15/P-17/P-20/P-22/P-23 product units, production callers must be non-test live paths (R7); P-25 production callers are the real CLI entrypoints, not unit tests. _(evidence: P15-E008; P17-E013; P20-E010; P22-E012; P23-E025; P25-E037–P25-E039)_
- **P-003/IX-017** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L336): Shared hash ban: product units ban blake2/sha2; P-20 allows blake3/sha256 for version markers; P-23 uses blake3 for frame hashes; P-25 trust authority forbids generic BLAKE3 as sole authority in favor of native Git/OID domains. _(evidence: P15-E018; P17-E024; P20-E015; P22-E022; P23-E026; P25-E043)_
- **P-003/IX-018** [IX] class=`I` (ledger-line: DEEP-P-003.discovery.md:L337): P-3 marketplace/friend sharing (Tier-2) and P-22 control-surface pairing UX (Tier-2) are distinct distribution vs device-enrollment surfaces that may later share L-12 admission patterns but are not the same product. _(evidence: P3-E009, P3-E012; P22-E013)_
- **P-003/IX-019** [IX] class=`E` (ledger-line: DEEP-P-003.discovery.md:L338): PS-number coupling in this chunk: P-15 feeds PS-5; P-17 feeds PS-4; P-3/P-20/P-22/P-23/P-25 declare no PS contribution. _(evidence: P15-E001; P17-E001; P3-E001; P20-E001; P22-E001; P23-E001; P25-E001)_

## Ambiguity and conflict registers (AC-*)

### AC register — P-001 (12)

- **P-001/AC-001** [AC] kind=`ambiguity` (ledger-line: DEEP-P-001.discovery.md:L407): P-0 allocates P-9..P-11 only in prose; full bodies outside chunk. _(refs: P0-A001)_
- **P-001/AC-002** [AC] kind=`ambiguity` (ledger-line: DEEP-P-001.discovery.md:L408): Interleaving of personal waves, CU-*, OC-* lacks single total order beyond A-7. _(refs: P0-A002)_
- **P-001/AC-003** [AC] kind=`ambiguity` (ledger-line: DEEP-P-001.discovery.md:L409): P-4 TTL/watch/settled-state/change-feed protocol unspecified. _(refs: P4-A001–P4-A002)_
- **P-001/AC-004** [AC] kind=`ambiguity` (ledger-line: DEEP-P-001.discovery.md:L410): P-5 near-zero tax threshold and personal/project tier semantics open. _(refs: P5-A001–P5-A003)_
- **P-001/AC-005** [AC] kind=`open design` (ledger-line: DEEP-P-001.discovery.md:L411): P-8 merge algorithm/clock/identity/transport open spike. _(refs: P8-A001–P8-A002)_
- **P-001/AC-006** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-001.discovery.md:L412): P-12 Tier-1 defers OmniParser while P-0 packages it with P-12. _(refs: P12-C001)_
- **P-001/AC-007** [AC] kind=`ambiguity` (ledger-line: DEEP-P-001.discovery.md:L413): P-12 recording/lease values and drawer identity open; frame hash sha256 vs blake3 tension. _(refs: P12-A001–P12-A003)_
- **P-001/AC-008** [AC] kind=`conflict/surface` (ledger-line: DEEP-P-001.discovery.md:L414): P-13 CLI trigger vs “no new entrypoint” PRODUCTION_HOOK; summary “add no CLI”. _(refs: P13-C001)_
- **P-001/AC-009** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-001.discovery.md:L415): P-13 Tier-3 pixel deferred vs summary fixture expecting pixel leg. _(refs: P13-C002, P13-A001–P13-A003)_
- **P-001/AC-010** [AC] kind=`ambiguity` (ledger-line: DEEP-P-001.discovery.md:L416): P-16 catalog integrity/signing/license vocabulary open. _(refs: P16-A001–P16-A002)_
- **P-001/AC-011** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-001.discovery.md:L417): P-18 Tier-2 defers retry/watchdog/clamp while stub_ban already forbids same-tick commitment. _(refs: P18-C001, P18-A001–P18-A003)_
- **P-001/AC-012** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-001.discovery.md:L418): P-21 Tier-2 vs wiring/production_hook/stub_ban requiring metadata export + posture labels now. _(refs: P21-C001, P21-A001–P21-A003)_

### AC register — P-002 (16)

- **P-002/AC-001** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-002.discovery.md:L421): P-1 opening pipeline names DriftCanary and gated auto-repair apply; REVISED Tier-1 is Smoke+DepRotCheck diagnosis only. _(refs: P1-C001, P1-C002)_
- **P-002/AC-002** [AC] kind=`ambiguity/dep` (ledger-line: DEEP-P-002.discovery.md:L422): P-1 G-10 per-app canary extractability, TaskSpec factory API, and F-9 entrypoint remain RECON. _(refs: P1-A001–P1-A003)_
- **P-002/AC-003** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-002.discovery.md:L423): P-2 steal/build centers O-4 canvas edges and third-party mount; REVISED Tier-1 is CLI-only. _(refs: P2-C001)_
- **P-002/AC-004** [AC] kind=`ambiguity` (ledger-line: DEEP-P-002.discovery.md:L424): P-2 compatibility/version rules and separate refusal-row requirement are open. _(refs: P2-A004–P2-A005)_
- **P-002/AC-005** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-002.discovery.md:L425): P-6 dual O-3+E-20 enforcement is stated in steal/build but E-20 is Tier-2 only. _(refs: P6-C001)_
- **P-002/AC-006** [AC] kind=`ambiguity/data` (ledger-line: DEEP-P-002.discovery.md:L426): P-6 grant expiry/nesting/precedence unspecified. _(refs: P6-A004)_
- **P-002/AC-007** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-002.discovery.md:L427): P-7 premise is measured auto-promotion; Tier-1 is manual asymmetry-guarded change. _(refs: P7-C001)_
- **P-002/AC-008** [AC] kind=`ambiguity/config` (ledger-line: DEEP-P-002.discovery.md:L428): P-7 FP metrics/thresholds/observation duration/defaults and domain vocabulary unspecified. _(refs: P7-A001–P7-A002)_
- **P-002/AC-009** [AC] kind=`ambiguity` (ledger-line: DEEP-P-002.discovery.md:L429): P-9 ConnectorSpec enums, pack-type admission discriminator, PortSchema binding, and signing key are RECON. _(refs: P9-A001–P9-A004)_
- **P-002/AC-010** [AC] kind=`ambiguity` (ledger-line: DEEP-P-002.discovery.md:L430): P-10 SsbEvent ingress shape, K-14 replay leg, vault at-rest, webhook signing scheme, OAuth follow-on schedule are RECON. _(refs: P10-A001–P10-A005)_
- **P-002/AC-011** [AC] kind=`ambiguity` (ledger-line: DEEP-P-002.discovery.md:L431): P-11 constrained-decode path, gap visual mapping, confirm-gate site, and re-propose bounds are RECON/open. _(refs: P11-A001–P11-A004)_
- **P-002/AC-012** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-002.discovery.md:L432): P-14 Playwright is Tier-1; PyAutoGUI is Tier-2 despite peer framing in coarser inventories. _(refs: P14-C001)_
- **P-002/AC-013** [AC] kind=`ambiguity` (ledger-line: DEEP-P-002.discovery.md:L433): P-14 Stage enum extension site and quantitative reliability threshold open. _(refs: P14-A001–P14-A002)_
- **P-002/AC-014** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-002.discovery.md:L434): P-19 summary treats backups/continuation/compaction as peer features; numbered Tier-1 is config guards + doctor only. _(refs: P19-C001)_
- **P-002/AC-015** [AC] kind=`ambiguity` (ledger-line: DEEP-P-002.discovery.md:L435): P-19 exit-code map, doctor_finding schema, backup restore details, multi-session continuation open. _(refs: P19-A001–P19-A004)_
- **P-002/AC-016** [AC] kind=`ambiguity` (ledger-line: DEEP-P-002.discovery.md:L436): P-24 Maestro/Expo/uiautomator RECON items, SSIM thresholds, native Fiber follow-up criteria open. _(refs: P24-A001–P24-A004)_

### AC register — P-003 (12)

- **P-003/AC-001** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-003.discovery.md:L344): P-3 premise describes full import+regen; REVISED Tier-1 omits persona regen. _(refs: P3-C001)_
- **P-003/AC-002** [AC] kind=`ambiguity/data` (ledger-line: DEEP-P-003.discovery.md:L345): P-3 AppBundle serialization, persona exclusion set, and M-5 whole-bundle grain remain open. _(refs: P3-A001–P3-A003)_
- **P-003/AC-003** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-003.discovery.md:L346): P-15 wargame asserts bBoN ranks, but REVISED scope defers bBoN/tiers/embeddings to Tier-2. _(refs: P15-C001, P15-E004)_
- **P-003/AC-004** [AC] kind=`ambiguity/config` (ledger-line: DEEP-P-003.discovery.md:L347): P-15 retrieval/fusion/promotion criteria and reuse-hit schema details unspecified. _(refs: P15-A001–P15-A002)_
- **P-003/AC-005** [AC] kind=`ambiguity/config` (ledger-line: DEEP-P-003.discovery.md:L348): P-17 auth cooldown range and status→lane mapping incomplete. _(refs: P17-A001–P17-A002)_
- **P-003/AC-006** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-003.discovery.md:L349): P-20 premise lists Dreaming + provider cache table; REVISED Tier-1 is flush + index only. _(refs: P20-C001)_
- **P-003/AC-007** [AC] kind=`ambiguity/flow` (ledger-line: DEEP-P-003.discovery.md:L350): P-20 note selection, "offered" definition, compact-degrade, and 4000-token threshold adoption unresolved. _(refs: P20-I001, P20-A001–P20-A003)_
- **P-003/AC-008** [AC] kind=`conflict/scope` (ledger-line: DEEP-P-003.discovery.md:L351): P-22 premise/stub_ban include channel-binding/pairing/idempotency/hooks; Tier-1 ships only fenced return + tool-strip. _(refs: P22-C001, P22-I001)_
- **P-003/AC-009** [AC] kind=`ambiguity/security` (ledger-line: DEEP-P-003.discovery.md:L352): P-22 fenced encoding, nine binding steps, pairing parameters unspecified. _(refs: P22-A001–P22-A003)_
- **P-003/AC-010** [AC] kind=`ambiguity/product` (ledger-line: DEEP-P-003.discovery.md:L353): P-23 cookie/permission/multi-tab behavior and driver choice open; P-10 custody external. _(refs: P23-A001–P23-A003)_
- **P-003/AC-011** [AC] kind=`status/gate` (ledger-line: DEEP-P-003.discovery.md:L354): P-25 blocked on fresh V4 PASS + RECON-2 re-audit; V3 PASS must not authorize execution. _(refs: P25-C001, P25-A001, P25-A003)_
- **P-003/AC-012** [AC] kind=`ambiguity/schema` (ledger-line: DEEP-P-003.discovery.md:L355): P-25 full hash-domain registry and frozen k/substitution tables live in M1 artifacts not fully inlined. _(refs: P25-A002, P25-A004)_

## Summary-delta records (SD-*)

Deep-vs-summary deltas. CORRECT/QUALIFY rows establish newer-plan (deep) precedence for target state.

### Summary deltas — P-001 (27)

- **P-001/SD-P0-001** [Δ] summary-anchor=`16-P:L1-L7, L1111` source-atoms=`P0-E001–P0-E060` (ledger-line: DEEP-P-001.discovery.md:L428): **ADD** full doctrine body the summary deliberately deferred to P-0: predecessor reading list, W/GD inheritance, DONE definition, SCHEMA_VERSION map, sequencing waves, carried corrections, P-25 preregistered arms, session-log empty state.
- **P-001/SD-P0-002** [Δ] summary-anchor=`16-P:L19-L20, L360-L371` source-atoms=`P0-E004–P0-E008, P0-E015–P0-E016` (ledger-line: DEEP-P-001.discovery.md:L429): **ADD** command-center prose allocation for P-9/P-10/P-11 and P-23/P-24 addenda as indexed in P-0 (still not detail extraction).
- **P-001/SD-P0-003** [Δ] summary-anchor=`16-P plan-wide` source-atoms=`P0-E040` (ledger-line: DEEP-P-001.discovery.md:L430): **ADD** exact SSB action_type inventory and prefer-reuse net-0..+1 units.
- **P-001/SD-P4-001** [Δ] summary-anchor=`16-P:L191-L222` source-atoms=`P4-E003–P4-E004, P4-E014` (ledger-line: DEEP-P-001.discovery.md:L436): **ADD** full LivingStoreContract field set including `ttl` and Spreadsheet class; B-35 lift; settled-state concurrent ingest rule.
- **P-001/SD-P4-002** [Δ] summary-anchor=`16-P:L191-L222` source-atoms=`P4-E017–P4-E021` (ledger-line: DEEP-P-001.discovery.md:L437): **ADD** FORK to read-only mount, ABORT on ungrounded API, exact CLI production hooks, INTEGRATION_SMOKE multi-namespace R10.
- **P-001/SD-P4-003** [Δ] summary-anchor=`16-P:L218-L220` source-atoms=`P4-E022; P4:L89` (ledger-line: DEEP-P-001.discovery.md:L438): **ADD** REDTEAM pairs and Notion MCP live-grounding note in grounding section.
- **P-001/SD-P5-001** [Δ] summary-anchor=`16-P:L224-L255` source-atoms=`P5-E003–P5-E004, P5-E012–P5-E014` (ledger-line: DEEP-P-001.discovery.md:L444): **ADD** full CeremonyProfile fields `gate_mode`, `receipt_verbosity`, `autonomy`; critical-vs-toy block differential; example wardrobe sentence.
- **P-001/SD-P5-002** [Δ] summary-anchor=`16-P:L224-L255` source-atoms=`P5-E008, P5-E010, P5-E016–P5-E018` (ledger-line: DEEP-P-001.discovery.md:L445): **ADD** Tier-2 repair button + dashboard fleet operator panel deferral; ABORT if skip-only path to near-zero tax; K-5/K-4/N-7 RECON greps.
- **P-001/SD-P5-003** [Δ] summary-anchor=`16-P:L250-L255` source-atoms=`P5-E021–P5-E022` (ledger-line: DEEP-P-001.discovery.md:L446): **ADD** multi-tier INTEGRATION_SMOKE and P-7 asymmetry redteam patch for critical-tool relabel.
- **P-001/SD-P8-001** [Δ] summary-anchor=`16-P:L325-L358` source-atoms=`P8-E006, P8-E009–P8-E012` (ledger-line: DEEP-P-001.discovery.md:L452): **ADD** event key `{seat_id, agent_id, lamport_or_hybrid_clock}`, seat_journal_merge fields, SCHEMA_VERSION bump discipline, M1/M2/M3 move order (spike first).
- **P-001/SD-P8-002** [Δ] summary-anchor=`16-P:L325-L358` source-atoms=`P8-E013–P8-E019` (ledger-line: DEEP-P-001.discovery.md:L453): **ADD** FORK hybrid clock at emit, ABORT if central server required, bidirectional merge smoke R10, lost-update lock lift.
- **P-001/SD-P8-003** [Δ/CORRECT] summary-anchor=`16-P:L331-L331` source-atoms=`P8-E006–P8-E007` (ledger-line: DEEP-P-001.discovery.md:L454): **QUALIFY** 2-10 scale: Tier-1 is two-seat kernel; 3-10 + rollup UX deferred.
- **P-001/SD-P12-001** [Δ/CORRECT] summary-anchor=`16-P:L374-L421` source-atoms=`P12-E001, P12-E005, P12-C001` (ledger-line: DEEP-P-001.discovery.md:L460): **CORRECT** summary treats OmniParser Tier-3 as P-12 ship scope; detail defers it Tier-2 with P-13; depends P-13 is explicit.
- **P-001/SD-P12-002** [Δ] summary-anchor=`16-P:L374-L421` source-atoms=`P12-E004, P12-E007, P12-E015–P12-E020` (ledger-line: DEEP-P-001.discovery.md:L461): **ADD** two-phase arm-commit, dual lease expiry, exact module paths, FORK multi-provider sandbox, ABORT host-desktop, compaction mid-run smoke.
- **P-001/SD-P12-003** [Δ] summary-anchor=`16-P:L387-L388` source-atoms=`P12-A003` (ledger-line: DEEP-P-001.discovery.md:L462): **FLAG** sha256 frame-bind wording vs blake3 hard rule.
- **P-001/SD-P13-001** [Δ/CORRECT] summary-anchor=`16-P:L448-L448` source-atoms=`P13-E012, P13-C001` (ledger-line: DEEP-P-001.discovery.md:L468): **CORRECT** summary “Add no CLI”: detail acceptance uses `elai perception scaffold` trigger while PRODUCTION_HOOK extends gateway in place.
- **P-001/SD-P13-002** [Δ/CORRECT] summary-anchor=`16-P:L450-L451` source-atoms=`P13-E006, P13-C002` (ledger-line: DEEP-P-001.discovery.md:L469): **CORRECT** summary empty-AXTree acceptance expects Tier-3 pixel fire once; detail defers Tier-3 pixel/YOLO to Tier-2.
- **P-001/SD-P13-003** [Δ] summary-anchor=`16-P:L423-L462` source-atoms=`P13-E005, P13-E008, P13-E019` (ledger-line: DEEP-P-001.discovery.md:L470): **ADD** ElementSource enum Rust names, interactability filter, set_of_marks module, joint P-12 smoke rule.
- **P-001/SD-P16-001** [Δ] summary-anchor=`16-P:L535-L568` source-atoms=`P16-E006, P16-E010, P16-E016–P16-E019` (ledger-line: DEEP-P-001.discovery.md:L476): **ADD** exact data path `data/connector/install-catalog.toml`, InstallCatalog module path, FORK to P-12 for GUI-only tools, ABORT unadmitted code, blender e2e smoke.
- **P-001/SD-P16-002** [Δ] summary-anchor=`16-P:L535-L568` source-atoms=`P16-E018` (ledger-line: DEEP-P-001.discovery.md:L477): **ADD** catalog row resolves ONLY to P-9 pack never raw shell (redteam patch).
- **P-001/SD-P16-003** [Δ/CORRECT] summary-anchor=`16-P:L547` source-atoms=`P16-E008` (ledger-line: DEEP-P-001.discovery.md:L478): **QUALIFY** “hidden-until-verified” appears in summary admission family phrasing; detail defers that registry release gate to Tier-2.
- **P-001/SD-P18-001** [Δ/CORRECT] summary-anchor=`16-P:L720-L768` source-atoms=`P18-E005–P18-E007, P18-C001` (ledger-line: DEEP-P-001.discovery.md:L484): **CORRECT** summary lists retry ladders, watchdog, preflight, due-time clamp, and resume checkpoint tests as peer requirements; detail Tier-1 is revision-counter + sticky-cancel + drift-binding only; others Tier-2/RECON.
- **P-001/SD-P18-002** [Δ] summary-anchor=`16-P:L720-L768` source-atoms=`P18-E008, P18-E017–P18-E021` (ledger-line: DEEP-P-001.discovery.md:L485): **ADD** FORK not to ship cooperative-only resume; ABORT partial plan bind; exact RunStore/ApprovalPlan modules; 4G autonomous multi-step smoke.
- **P-001/SD-P18-003** [Δ/CORRECT] summary-anchor=`16-P:L736-L736, L759` source-atoms=`P18-E023` (ledger-line: DEEP-P-001.discovery.md:L486): **QUALIFY** same-tick commitment ban is active in stub_ban while S19 clamp remains Tier-2 feature work.
- **P-001/SD-P21-001** [Δ/CORRECT] summary-anchor=`16-P:L864-L906` source-atoms=`P21-E005–P21-E006, P21-C001` (ledger-line: DEEP-P-001.discovery.md:L492): **CORRECT/QUALIFY** summary lists skipBackfill, structural redaction, and posture labels as peer requirements; detail revised scope puts them Tier-2 while still wiring/stubbing some now — record internal conflict, do not flatten.
- **P-001/SD-P21-002** [Δ] summary-anchor=`16-P:L864-L906` source-atoms=`P21-E012–P21-E019` (ledger-line: DEEP-P-001.discovery.md:L493): **ADD** exact CLI `elai bench gate-corpus --with-canaries --break-detector redaction`, `gate_credibility_measured` row, env-blocked WARN FORK, whole-manifest R10 smoke.
- **P-001/SD-P21-003** [Δ] summary-anchor=`16-P:L864-L906` source-atoms=`P21-E003` (ledger-line: DEEP-P-001.discovery.md:L494): **ADD** V1/V2/V3/N1/N2/C3 correction anchors from openclaw deep-read as explicit design constraints.

### Summary deltas — P-002 (38)

- **P-002/SD-P1-001** [Δ/CORRECT] summary-anchor=`16-P:L87-L123` source-atoms=`P1-E012–P1-E013, P1-C001–P1-C002` (ledger-line: DEEP-P-002.discovery.md:L446): **CORRECT** drift canaries and one-click auto-repair are not Tier-1: Tier-1 is Smoke+DepRotCheck + N-7 diagnosis card; apply-button and G-10 generalization are Tier-2/ABORT-gated.
- **P-002/SD-P1-002** [Δ] summary-anchor=`16-P:L93-L118` source-atoms=`P1-E003, P1-E015, P1-E017` (ledger-line: DEEP-P-002.discovery.md:L447): **ADD** exact CLI verbs `register\|list\|tick`, closed AppManifest field set including `app_id`/`one_sentence_purpose`/`smoke_trigger`, and O-3 namespace-claims hard refusal at register.
- **P-002/SD-P1-003** [Δ] summary-anchor=`16-P:L109-L123` source-atoms=`P1-E016, P1-E019–P1-E020, P1-E027` (ledger-line: DEEP-P-002.discovery.md:L448): **ADD** auto-archive-on-green, exactly-once per due tick, batched multi-app TaskSpec storm patch, and dedicated FleetPanel deferral to P-5.
- **P-002/SD-P1-004** [Δ] summary-anchor=`16-P:L117-L118` source-atoms=`P1-E024–P1-E026, P1-E029` (ledger-line: DEEP-P-002.discovery.md:L449): **ADD** production dual hooks (CLI + task_runner), R10 two-app multi-tick smoke, wiring/production_caller checks, and full consume list including O-3.
- **P-002/SD-P1-005** [Δ] summary-anchor=`16-P:L119-L122` source-atoms=`P1-A001–P1-A004` (ledger-line: DEEP-P-002.discovery.md:L450): **EXPAND** P1-A01 recon with TaskSpec factory and F-9 entrypoint unknowns; add Cron/TTL default gap.
- **P-002/SD-P2-001** [Δ/CORRECT] summary-anchor=`16-P:L124-L153` source-atoms=`P2-E009–P2-E012, P2-C001` (ledger-line: DEEP-P-002.discovery.md:L456): **CORRECT/QUALIFY** O-4 canvas edge rendering and third-party M-5 mount are Tier-2; Tier-1 trust value is fully CLI (`wire`/`ports`/`--unwired`) + SSB.
- **P-002/SD-P2-002** [Δ] summary-anchor=`16-P:L130-L131` source-atoms=`P2-E003, P2-E013` (ledger-line: DEEP-P-002.discovery.md:L457): **ADD** PortSchema fields `{name, json_schema, domain_label}`, `deny_unknown`, and P-6 domain coupling.
- **P-002/SD-P2-003** [Δ] summary-anchor=`16-P:L137-L153` source-atoms=`P2-E016–P2-E017, P2-E021` (ledger-line: DEEP-P-002.discovery.md:L458): **ADD** FORK when O-4 edge model cannot carry tool->tool edges; ABORT on live broker; L-12 admission for malicious mount patch.
- **P-002/SD-P2-004** [Δ] summary-anchor=`16-P:L150-L153` source-atoms=`P2-A001–P2-A005` (ledger-line: DEEP-P-002.discovery.md:L459): **EXPAND** compatibility/version and refusal-row ambiguities beyond summary P2-A01/A02.
- **P-002/SD-P6-001** [Δ/CORRECT] summary-anchor=`16-P:L257-L288` source-atoms=`P6-E007–P6-E008, P6-C001` (ledger-line: DEEP-P-002.discovery.md:L465): **CORRECT** E-20 routing enforcement is Tier-2 (with P-4), not Tier-1; Tier-1 is O-3 vault-read only. Summary lists E-20 as peer requirement.
- **P-002/SD-P6-002** [Δ] summary-anchor=`16-P:L263-L287` source-atoms=`P6-E008, P6-E013, P6-E016` (ledger-line: DEEP-P-002.discovery.md:L466): **ADD** grant-management UI deferral to P-5, JOIN-table FORK if N-6 column migration hits N-4, and N-2 corpus_composition_check as hard VERIFY gate.
- **P-002/SD-P6-003** [Δ] summary-anchor=`16-P:L283-L287` source-atoms=`P6-E017–P6-E018` (ledger-line: DEEP-P-002.discovery.md:L467): **ADD** full INTEGRATION_SMOKE multi-namespace multi-tool + mid-run revoke scenario and redteam second-scale patch.
- **P-002/SD-P6-004** [Δ] summary-anchor=`16-P:L285-L287` source-atoms=`P6-A004` (ledger-line: DEEP-P-002.discovery.md:L468): **EXPAND** grant-scope ambiguity (expiry/nesting/precedence) already noted as P6-A01.
- **P-002/SD-P7-001** [Δ/CORRECT] summary-anchor=`16-P:L289-L323` source-atoms=`P7-E006–P7-E007, P7-E012, P7-C001` (ledger-line: DEEP-P-002.discovery.md:L474): **CORRECT/QUALIFY** automatic N-3 learning loop is Tier-2; Tier-1 is manual asymmetry-guarded change with FORK if N-3 not live. Summary reads as if measured promotion is immediate.
- **P-002/SD-P7-002** [Δ] summary-anchor=`16-P:L309-L316` source-atoms=`P7-E008, P7-E010` (ledger-line: DEEP-P-002.discovery.md:L475): **ADD** exact default wiring toy->P-5, connected->P-2 on, team->M+P-8 and `operating_profile_change` field shape.
- **P-002/SD-P7-003** [Δ] summary-anchor=`16-P:L306-L323` source-atoms=`P7-E011, P7-E013, P7-E016–P7-E017` (ledger-line: DEEP-P-002.discovery.md:L476): **ADD** ABORT on non-own-run FP evidence, observe-first GD-4 detail, and R10 multi-op refuse+promote+revert smoke.
- **P-002/SD-P7-004** [Δ] summary-anchor=`16-P:L320-L323` source-atoms=`P7-A001–P7-A005` (ledger-line: DEEP-P-002.discovery.md:L477): **EXPAND** RECON for H-17/H-18 process-metric scoring and M-2 enforcement point beyond summary P7-A01/A02.
- **P-002/SD-P9-001** [Δ] summary-anchor=`16-P:L360-L366` source-atoms=`P9-E001–P9-E023` (ledger-line: DEEP-P-002.discovery.md:L483): **ADD entire unit** — summary has only reference-derived P9-I01/I02/A01 with no fields, CLI, events, or acceptance. Numbered source supplies full ConnectorSpec/http_call/pack/admission design.
- **P-002/SD-P9-002** [Δ] summary-anchor=`16-P:L360-L366` source-atoms=`P9-E008–P9-E010` (ledger-line: DEEP-P-002.discovery.md:L484): **ADD** user-facing `elai connector new`, MountedConnector journal surface, LANE A/B fork, egress pin, fail-closed tampered-pack refusal.
- **P-002/SD-P9-003** [Δ] summary-anchor=`16-P:L19-L24, L360-L366` source-atoms=`P9-E001, P9-E021–P9-E022` (ledger-line: DEEP-P-002.discovery.md:L485): **ADD** post-spine execution timing, KD-1 n8n-count caution, and deferred redteam surface list.
- **P-002/SD-P10-001** [Δ] summary-anchor=`16-P:L367-L370` source-atoms=`P10-E001–P10-E020` (ledger-line: DEEP-P-002.discovery.md:L491): **ADD entire unit** — summary has only P10-I01..I03/A01 references from later units; no vault schema, adapter taxonomy, or entrypoints.
- **P-002/SD-P10-002** [Δ] summary-anchor=`16-P:L367-L370` source-atoms=`P10-E003–P10-E011` (ledger-line: DEEP-P-002.discovery.md:L492): **ADD** four-shape normalization, WebhookIngress HMAC-before-parse, `elai cred add` v1, CredentialVault non-leak, I-1 listener isolation, OAuth follow-on block.
- **P-002/SD-P10-003** [Δ] summary-anchor=`16-P:L367-L370` source-atoms=`P10-E007–P10-E008, P10-E015–P10-E016` (ledger-line: DEEP-P-002.discovery.md:L493): **ADD** faithful async replay requirement (ABORT-2) and zero-leak negative arm as load-bearing acceptance.
- **P-002/SD-P11-001** [Δ] summary-anchor=`16-P:L371-L372` source-atoms=`P11-E001–P11-E021` (ledger-line: DEEP-P-002.discovery.md:L499): **ADD entire unit** — summary has only P11-I01/A01; no graph-synthesis rules, UI/API, events, or gates.
- **P-002/SD-P11-002** [Δ] summary-anchor=`16-P:L371-L372` source-atoms=`P11-E010–P11-E013` (ledger-line: DEEP-P-002.discovery.md:L500): **ADD** `elai workflow from-goal`, O-4 proposed-graph + confirm UI, inert-until-confirm, RRA intent->graph, honest disconnected gap + connector-new hint.
- **P-002/SD-P11-003** [Δ] summary-anchor=`16-P:L371-L372` source-atoms=`P11-E005–P11-E009, P11-E019` (ledger-line: DEEP-P-002.discovery.md:L501): **ADD** FORK constrained-decode vs validate, ABORT closed-vocab/pre-confirm/RRA, and deferred redteam injection surfaces.
- **P-002/SD-P14-001** [Δ/CORRECT] summary-anchor=`16-P:L464-L498` source-atoms=`P14-E004–P14-E005, P14-C001` (ledger-line: DEEP-P-002.discovery.md:L507): **CORRECT** PyAutoGUI desktop crystallizer is Tier-2 deferred; Tier-1 is E-14 browser Playwright only. Summary lists both as peer emitters.
- **P-002/SD-P14-002** [Δ] summary-anchor=`16-P:L470-L498` source-atoms=`P14-E003, P14-E010, P14-E013–P14-E014` (ledger-line: DEEP-P-002.discovery.md:L508): **ADD** openclaw C4 correction (mock input replay is not prior art), agentic-step FORK for non-deterministic UI, blake3 script hash, and volatile-literal parameterization patch.
- **P-002/SD-P14-003** [Δ] summary-anchor=`16-P:L480-L493` source-atoms=`P14-E006, P14-E008, P14-E017` (ledger-line: DEEP-P-002.discovery.md:L509): **ADD** exact module paths crystallize/mod.rs + rra_bind.rs, `elai e2e replay --stage`, and R10 multi-step SaaS smoke.
- **P-002/SD-P14-004** [Δ/CORRECT] summary-anchor=`16-P:L491-L498` source-atoms=`P14-E003, P14-A002` (ledger-line: DEEP-P-002.discovery.md:L510): **QUALIFY** “~100% reliability” and Agent Zero citation: source emphasizes cua replay_env and excludes openclaw framing; no quantitative threshold.
- **P-002/SD-P19-001** [Δ/CORRECT] summary-anchor=`16-P:L770-L816` source-atoms=`P19-E005–P19-E006, P19-C001` (ledger-line: DEEP-P-002.discovery.md:L516): **CORRECT** VACUUM backups, restart continuation, post-upgrade probes, and compaction split/successor invariants are Tier-2; Tier-1 is fail-closed config lifecycle + doctor detect/repair/lint. Summary flattens all as peer E-rows.
- **P-002/SD-P19-002** [Δ] summary-anchor=`16-P:L787-L816` source-atoms=`P19-E014–P19-E016` (ledger-line: DEEP-P-002.discovery.md:L517): **ADD** FORK refuse-to-auto-repair beyond LKG, ABORT on silent unknown-key admit / full-env copy, and P-10 vault as secret home vs config.
- **P-002/SD-P19-003** [Δ] summary-anchor=`16-P:L800-L810` source-atoms=`P19-E004, P19-E008–P19-E010` (ledger-line: DEEP-P-002.discovery.md:L518): **ADD** exact symbols lifecycle.rs + doctor_cmd.rs, CI-facing no-frontend justification, and path-jail case-fold win32 detail.
- **P-002/SD-P19-004** [Δ] summary-anchor=`16-P:L805-L816` source-atoms=`P19-E018, P19-C001` (ledger-line: DEEP-P-002.discovery.md:L519): **FLAG** INTEGRATION_SMOKE references mid-turn resume while resume is Tier-2 deferred — execution must either promote resume or narrow the smoke.
- **P-002/SD-P24-001** [Δ] summary-anchor=`16-P:L620-L678` source-atoms=`P24-E020` (ledger-line: DEEP-P-002.discovery.md:L525): **ADD** Maestro admission-fail fallback to P-12 journaled step list, perception_degraded on empty RN a11y, and never-substitute-rnw_web-for-iOS FORK rules.
- **P-002/SD-P24-002** [Δ] summary-anchor=`16-P:L656-L672` source-atoms=`P24-E021, P24-E023, P24-E027` (ledger-line: DEEP-P-002.discovery.md:L526): **ADD** ABORT set (physical device, host escape, missing label, unshipped H-14 B0), production_caller_check on mobile modules, and vertical-slice same-changeset hook across CLI/task_runner/canvas.
- **P-002/SD-P24-003** [Δ] summary-anchor=`16-P:L648-L670` source-atoms=`P24-E022, P24-E025` (ledger-line: DEEP-P-002.discovery.md:L527): **ADD** evidence-leg negative arm (Maestro green / evidence FAIL) and redteam Maestro-as-verdict patch already in source ON_FAIL.
- **P-002/SD-P24-004** [Δ] summary-anchor=`16-P:L674-L678` source-atoms=`P24-A001–P24-A003` (ledger-line: DEEP-P-002.discovery.md:L528): **EXPAND** RECON list (Maestro license/checksum, Expo data-oid, uiautomator fidelity, CDP precedents) beyond summary A-rows.

### Summary deltas — P-003 (38)

- **P-003/SD-P3-001** [Δ] summary-anchor=`16-P:L155-L189` source-atoms=`P3-E012, P3-C001` (ledger-line: DEEP-P-003.discovery.md:L365): **ADD** REVISED Tier-1 vs Tier-2 split: Tier-1 is export + verify-gated import without persona regen; regen and marketplace/friend surface are Tier-2.
- **P-003/SD-P3-002** [Δ] summary-anchor=`16-P:L161-L182` source-atoms=`P3-E018–P3-E019` (ledger-line: DEEP-P-003.discovery.md:L366): **ADD** FORK for M-5 whole-bundle capability (presence-check floor) and ABORT if factoring needs code forks.
- **P-003/SD-P3-003** [Δ] summary-anchor=`16-P:L177-L182` source-atoms=`P3-E014–P3-E016, P3-E021–P3-E022` (ledger-line: DEEP-P-003.discovery.md:L367): **ADD** exact SSB names `app_export`/`app_import`, planted-secret smoke, production_caller_check, and R10 second-seat smoke.
- **P-003/SD-P3-004** [Δ] summary-anchor=`16-P:L184-L184` source-atoms=`P3-E023–P3-E024` (ledger-line: DEEP-P-003.discovery.md:L368): **ADD** redteam post-signing edit fail and provenance-leak patch; P-6 domain labels ride ports.
- **P-003/SD-P3-005** [Δ] summary-anchor=`16-P:L187-L189` source-atoms=`P3-A001–P3-A003` (ledger-line: DEEP-P-003.discovery.md:L369): **EXPAND** summary A01/A02 with recon items for G-9 entrypoint and persona physical layout.
- **P-003/SD-P15-001** [Δ/CORRECT] summary-anchor=`16-P:L507-L521` source-atoms=`P15-E003–P15-E005, P15-C001` (ledger-line: DEEP-P-003.discovery.md:L375): **CORRECT** summary inventory that lists narrative/episodic tiers, embedding retrieval, and bBoN as if Tier-1: source defers all three to Tier-2; Tier-1 is gated retrieve/distill/fuse + reuse-hit + default-off.
- **P-003/SD-P15-002** [Δ] summary-anchor=`16-P:L500-L533` source-atoms=`P15-E013` (ledger-line: DEEP-P-003.discovery.md:L376): **ADD** FORK for stale non-reproducing cases (re-explore + re-mint + staleness field).
- **P-003/SD-P15-003** [Δ] summary-anchor=`16-P:L514-L519` source-atoms=`P15-E010–P15-E011, P15-E014` (ledger-line: DEEP-P-003.discovery.md:L377): **ADD** ABORT if reuse-into-prompt lacks execution-grounded filter; DGM spine theorem cite; `elai case reuse` and `elai bench case-reuse` consumers.
- **P-003/SD-P15-004** [Δ] summary-anchor=`16-P:L515-L515` source-atoms=`P15-E007, P15-E009, P15-E017` (ledger-line: DEEP-P-003.discovery.md:L378): **ADD** produces_symbols paths reuse_loop.rs/bbon_judge.rs, memory CanvasMode chrome, and shared P-14 R10 cadence.
- **P-003/SD-P15-005** [Δ] summary-anchor=`16-P:L509-L513` source-atoms=`P15-C001` (ledger-line: DEEP-P-003.discovery.md:L379): **FLAG** bBoN appears in wargame expected observation while deferred — do not require bBoN for Tier-1 done.
- **P-003/SD-P17-001** [Δ] summary-anchor=`16-P:L705-L705` source-atoms=`P17-E011, P17-E013` (ledger-line: DEEP-P-003.discovery.md:L385): **ADD** exact module paths failover.rs/strict_selection.rs and dispatch.rs production hook.
- **P-003/SD-P17-002** [Δ] summary-anchor=`16-P:L699-L700` source-atoms=`P17-E018` (ledger-line: DEEP-P-003.discovery.md:L386): **ADD** FORK behavior for pinned-model cooldown (strict failure or Q-4 opt-in swap-restart only).
- **P-003/SD-P17-003** [Δ] summary-anchor=`16-P:L680-L718` source-atoms=`P17-E019–P17-E020` (ledger-line: DEEP-P-003.discovery.md:L387): **ADD** Tier-2 deferrals (~140-plugin store, 5-min reprobe) and EXCLUDED silent sub / merged number.
- **P-003/SD-P17-004** [Δ] summary-anchor=`16-P:L707-L710` source-atoms=`P17-E015, P17-E022–P17-E023` (ledger-line: DEEP-P-003.discovery.md:L388): **ADD** CLI failover-test fixture path, INTEGRATION_SMOKE multi-failure hard session, and redteam economics attack.
- **P-003/SD-P17-005** [Δ] summary-anchor=`16-P:L714-L718` source-atoms=`P17-A001–P17-A002` (ledger-line: DEEP-P-003.discovery.md:L389): **EXPAND** A01-A03 with counter-move scope keys {provider, lane, model}.
- **P-003/SD-P20-001** [Δ/CORRECT] summary-anchor=`16-P:L841-L848` source-atoms=`P20-E002, P20-E005, P20-C001` (ledger-line: DEEP-P-003.discovery.md:L395): **CORRECT** summary lists Dreaming promotion gates and per-provider cache mapping as atomic inventory items (P20-E18..E22): source DEFERs both to Tier-2; Tier-1 is flush + on-demand index only.
- **P-003/SD-P20-002** [Δ] summary-anchor=`16-P:L848-L848` source-atoms=`P20-E009–P20-E010` (ledger-line: DEEP-P-003.discovery.md:L396): **ADD** exact module paths precompact_flush.rs/ondemand_index.rs and CompactionPass extension rule.
- **P-003/SD-P20-003** [Δ] summary-anchor=`16-P:L849-L853` source-atoms=`P20-E012, P20-E017` (ledger-line: DEEP-P-003.discovery.md:L397): **ADD** trigger `elai session drive-to-compaction`, R10 double-threshold tool-heavy smoke, and P-19 split-rule composition.
- **P-003/SD-P20-004** [Δ] summary-anchor=`16-P:L824-L839` source-atoms=`P20-E003, P20-I001, P20-E015` (ledger-line: DEEP-P-003.discovery.md:L398): **ADD** openclaw soft threshold 4000 tokens as cited prior art (not locked Tier-1 numeric) and sha256 version marker allowance.
- **P-003/SD-P20-005** [Δ/CORRECT] summary-anchor=`16-P:L848-L848` source-atoms=`P20-E011` (ledger-line: DEEP-P-003.discovery.md:L399): **QUALIFY** frontend N/A beyond existing cost/memory panels — no new UI.
- **P-003/SD-P22-001** [Δ/CORRECT] summary-anchor=`16-P:L914-L939` source-atoms=`P22-E002, P22-E005–P22-E007, P22-C001` (ledger-line: DEEP-P-003.discovery.md:L405): **CORRECT** summary inventory treats nine-step channel-binding, pairing codes, dmScope, signed device pairing, unknown-event withhold, idempotency keys, and trusted-policy hooks as peer Tier-1 atoms: source DEFERs all of those to Tier-2; Tier-1 is only fenced returns + depth-scoped tool-strip.
- **P-003/SD-P22-002** [Δ] summary-anchor=`16-P:L951-L952` source-atoms=`P22-E008` (ledger-line: DEEP-P-003.discovery.md:L406): **ADD** exact openclaw pairing parameters (8-char, 1h, <=3 pending) as Tier-2 prior art and DEVICE_AUTH_* codes.
- **P-003/SD-P22-003** [Δ] summary-anchor=`16-P:L940-L945` source-atoms=`P22-E016–P22-E017, P22-E020` (ledger-line: DEEP-P-003.discovery.md:L407): **ADD** FORK depth-1-only tool regain rule; ABORT if return hits instruction channel; Dual-LLM quarantine composition.
- **P-003/SD-P22-004** [Δ] summary-anchor=`16-P:L938-L941` source-atoms=`P22-E010, P22-E014, P22-E019` (ledger-line: DEEP-P-003.discovery.md:L408): **ADD** modules fenced_return.rs/tool_strip.rs, spawn-test CLI fixture, and depth 2-3 R10 smoke.
- **P-003/SD-P22-005** [Δ] summary-anchor=`16-P:L931-L932` source-atoms=`P22-E021, P22-I001` (ledger-line: DEEP-P-003.discovery.md:L409): **FLAG** stub_ban unknown-event withhold is unit-stated while feature is Tier-2 — contract intent vs ship scope.
- **P-003/SD-P23-001** [Δ] summary-anchor=`16-P:L616-L616` source-atoms=`P23-E019` (ledger-line: DEEP-P-003.discovery.md:L415): **ADD** quantified latency FORK tripline p95 > 250ms and captureScreenshot degraded fallback.
- **P-003/SD-P23-002** [Δ] summary-anchor=`16-P:L591-L592` source-atoms=`P23-E008, P23-E022` (ledger-line: DEEP-P-003.discovery.md:L416): **ADD** domain-first-seen warning as privacy egress row; UI-redress and phishing-form redteam outcomes.
- **P-003/SD-P23-003** [Δ] summary-anchor=`16-P:L601-L602` source-atoms=`P23-E025, P23-E021` (ledger-line: DEEP-P-003.discovery.md:L417): **ADD** vertical-slice same-changeset wiring of CLI + front-door + H-13; production_caller_check on cobrowse modules.
- **P-003/SD-P23-004** [Δ] summary-anchor=`16-P:L598-L599` source-atoms=`P23-E012, P23-E023, P23-E027` (ledger-line: DEEP-P-003.discovery.md:L418): **ADD** threaded E-14 contract (vendor if unexecuted) and P-14 crystallize RRA-bound replay in smoke.
- **P-003/SD-P23-005** [Δ] summary-anchor=`16-P:L616-L616` source-atoms=`P23-E019` (ledger-line: DEEP-P-003.discovery.md:L419): **EXPAND** A01: summary said latency unquantified; source quantifies 250ms p95.
- **P-003/SD-P25-001** [Δ] summary-anchor=`16-P:L955-L958` source-atoms=`P25-E001–P25-E002, P25-C001` (ledger-line: DEEP-P-003.discovery.md:L425): **ADD** full V4 authority precedence, PLANNING CANDIDATE status, and ban on using V2/V3 PASS as execution authority.
- **P-003/SD-P25-002** [Δ] summary-anchor=`16-P:L969-L987` source-atoms=`P25-E007–P25-E013` (ledger-line: DEEP-P-003.discovery.md:L426): **ADD** complete six-arm definitions including PAD_MATCHED envelope math, stratified EXO_SHUFFLED, SHUFFLED_REVISE permutation, and exo-card V2 field order.
- **P-003/SD-P25-003** [Δ] summary-anchor=`16-P:L1027-L1027` source-atoms=`P25-E016–P25-E022` (ledger-line: DEEP-P-003.discovery.md:L427): **ADD** exact G1-G4 formulas (1.3x/2.5x ratios, -0.05 non-inferiority, 5000-sample seed 20260712) that summary A04 said were unreproduced.
- **P-003/SD-P25-004** [Δ] summary-anchor=`16-P:L1003-L1008` source-atoms=`P25-E027–P25-E040, P25-E045` (ledger-line: DEEP-P-003.discovery.md:L428): **ADD** M0-M5 ordered pipeline, six M3 batches, curator CLI surface, content-addressed artifact layout, and atomic out_dir rename.
- **P-003/SD-P25-005** [Δ] summary-anchor=`16-P:L1052-L1058` source-atoms=`P25-E036, P25-E043–P25-E044, P25-E047–P25-E049` (ledger-line: DEEP-P-003.discovery.md:L429): **ADD** full ABORT list, stub_ban injection rejections, FORKs (pool postpone, relevance-null pattern, bridge swap, core TCB block).
- **P-003/SD-P25-006** [Δ/CORRECT] summary-anchor=`16-P:L983-L999` source-atoms=`P25-E013, P25-E042, P25-E043` (ledger-line: DEEP-P-003.discovery.md:L430): **CORRECT/QUALIFY** summary P25-E24 KEY_ALIASES and P25-E38 "import Q-10 modules": source requires exo-card V2 grammar as M3 deliverable and forbids growing/importing Q-10 runner; only stdlib paired-bootstrap extraction is shared.
- **P-003/SD-P25-007** [Δ] summary-anchor=`16-P:L996-L997` source-atoms=`P25-E051, P25-E053` (ledger-line: DEEP-P-003.discovery.md:L431): **ADD** redteam multi-pass patches (stratified shuffle, ledger-leg exclusion, SHUFFLED_REVISE, multiplicity, card grammar differentiation).
- **P-003/SD-P25-008** [Δ/CORRECT] summary-anchor=`16-P:L1026-L1026` source-atoms=`P25-E041, P25-I001` (ledger-line: DEEP-P-003.discovery.md:L432): **QUALIFY** summary A03 path reconciliation: plan still names exo_retrieval.py/exo_injection_ab.py as produces_symbols; tree drift is orientation-only until execution recon, not a source atom of absence.

## Superseded notes (newer-plan precedence)

Losing descriptions retained with anchors. Deep Tier-1/Tier-2 REVISED scope and SD CORRECT/QUALIFY beat summary peer-requirement lists.

- **SUP-001** [superseded] id=`P-001/SD-P8-003` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L331-L331` (ledger=P-001, line=L454): **QUALIFY** 2-10 scale: Tier-1 is two-seat kernel; 3-10 + rollup UX deferred.
- **SUP-002** [superseded] id=`P-001/SD-P12-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L374-L421` (ledger=P-001, line=L460): **CORRECT** summary treats OmniParser Tier-3 as P-12 ship scope; detail defers it Tier-2 with P-13; depends P-13 is explicit.
- **SUP-003** [superseded] id=`P-001/SD-P13-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L448-L448` (ledger=P-001, line=L468): **CORRECT** summary “Add no CLI”: detail acceptance uses `elai perception scaffold` trigger while PRODUCTION_HOOK extends gateway in place.
- **SUP-004** [superseded] id=`P-001/SD-P13-002` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L450-L451` (ledger=P-001, line=L469): **CORRECT** summary empty-AXTree acceptance expects Tier-3 pixel fire once; detail defers Tier-3 pixel/YOLO to Tier-2.
- **SUP-005** [superseded] id=`P-001/SD-P16-003` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L547` (ledger=P-001, line=L478): **QUALIFY** “hidden-until-verified” appears in summary admission family phrasing; detail defers that registry release gate to Tier-2.
- **SUP-006** [superseded] id=`P-001/SD-P18-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L720-L768` (ledger=P-001, line=L484): **CORRECT** summary lists retry ladders, watchdog, preflight, due-time clamp, and resume checkpoint tests as peer requirements; detail Tier-1 is revision-counter + sticky-cancel + drift-binding only; others Tier-2/RECON.
- **SUP-007** [superseded] id=`P-001/SD-P18-003` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L736-L736, L759` (ledger=P-001, line=L486): **QUALIFY** same-tick commitment ban is active in stub_ban while S19 clamp remains Tier-2 feature work.
- **SUP-008** [superseded] id=`P-001/SD-P21-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L864-L906` (ledger=P-001, line=L492): **CORRECT/QUALIFY** summary lists skipBackfill, structural redaction, and posture labels as peer requirements; detail revised scope puts them Tier-2 while still wiring/stubbing some now — record internal conflict, do not flatten.
- **SUP-009** [superseded] id=`P-001/P12-C001` reason=`deep_conflict_target_state` anchor=`P-12:L20-L20; cross-ref P-0:L42-L44` (ledger=P-001, line=L234): P-0 index lists OmniParser Tier-3 as part of P-12 package, but this detail defers Tier-3 pixel adapter to Tier-2 with P-13; Tier-1 is sandbox+frame-bind+arm/lease+recording.
- **SUP-010** [superseded] id=`P-001/P13-C001` reason=`deep_conflict_target_state` anchor=`P-13:L46-L46, L71-L72` (ledger=P-001, line=L268): Detail documents CLI trigger `elai perception scaffold` as acceptance entrypoint while PRODUCTION_HOOK says no new entrypoint (extend gateway in place). Surface may be existing CLI wrapping gateway, but “add no CLI” summary language conflicts with this trigger.
- **SUP-011** [superseded] id=`P-001/P13-C002` reason=`deep_conflict_target_state` anchor=`P-13:L20-L20, L51-L51` (ledger=P-001, line=L269): Summary acceptance expects Tier-3 pixel leg fire once on empty-AXTree fixture; detail defers Tier-3 pixel/YOLO to Tier-2 with P-12.
- **SUP-012** [superseded] id=`P-001/P18-C001` reason=`deep_conflict_target_state` anchor=`P-18:L20-L20, L42-L42; cross-ref P-0:L96-L100` (ledger=P-001, line=L337): Index and summary list retry ladders, watchdog, and due-time clamp as package features; detail places them in Tier-2 while still forbidding same-tick commitment in Tier-1 stub_ban.
- **SUP-013** [superseded] id=`P-001/P21-C001` reason=`deep_conflict_target_state` anchor=`P-21:L18-L19, L29-L30, L37-L41, L47-L47, L70-L71` (ledger=P-001, line=L371): Revised scope places skipBackfill, structural redaction, and posture labels in Tier-2, but WIRING_CONTRACT produces metadata_export.rs, stub_ban forbids content field and missing posture, observable_effect fails smoke without posture label, and PRODUCTION_HOOK claims all wired in this changeset. Tier boundary is internally inconsistent.
- **SUP-014** [superseded] id=`P-002/SD-P1-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L87-L123` (ledger=P-002, line=L446): **CORRECT** drift canaries and one-click auto-repair are not Tier-1: Tier-1 is Smoke+DepRotCheck + N-7 diagnosis card; apply-button and G-10 generalization are Tier-2/ABORT-gated.
- **SUP-015** [superseded] id=`P-002/SD-P2-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L124-L153` (ledger=P-002, line=L456): **CORRECT/QUALIFY** O-4 canvas edge rendering and third-party M-5 mount are Tier-2; Tier-1 trust value is fully CLI (`wire`/`ports`/`--unwired`) + SSB.
- **SUP-016** [superseded] id=`P-002/SD-P6-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L257-L288` (ledger=P-002, line=L465): **CORRECT** E-20 routing enforcement is Tier-2 (with P-4), not Tier-1; Tier-1 is O-3 vault-read only. Summary lists E-20 as peer requirement.
- **SUP-017** [superseded] id=`P-002/SD-P7-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L289-L323` (ledger=P-002, line=L474): **CORRECT/QUALIFY** automatic N-3 learning loop is Tier-2; Tier-1 is manual asymmetry-guarded change with FORK if N-3 not live. Summary reads as if measured promotion is immediate.
- **SUP-018** [superseded] id=`P-002/SD-P14-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L464-L498` (ledger=P-002, line=L507): **CORRECT** PyAutoGUI desktop crystallizer is Tier-2 deferred; Tier-1 is E-14 browser Playwright only. Summary lists both as peer emitters.
- **SUP-019** [superseded] id=`P-002/SD-P14-004` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L491-L498` (ledger=P-002, line=L510): **QUALIFY** “~100% reliability” and Agent Zero citation: source emphasizes cua replay_env and excludes openclaw framing; no quantitative threshold.
- **SUP-020** [superseded] id=`P-002/SD-P19-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L770-L816` (ledger=P-002, line=L516): **CORRECT** VACUUM backups, restart continuation, post-upgrade probes, and compaction split/successor invariants are Tier-2; Tier-1 is fail-closed config lifecycle + doctor detect/repair/lint. Summary flattens all as peer E-rows.
- **SUP-021** [superseded] id=`P-002/P1-C001` reason=`deep_conflict_target_state` anchor=`P-1:L10-L11, L21-L23, L41-L42` (ledger=P-002, line=L70): Opening steal/build names DriftCanary in the pipeline, but REVISED Tier-1 ships only Smoke + DepRotCheck; drift canary is Tier-2 behind G-10 generalization.
- **SUP-022** [superseded] id=`P-002/P1-C002` reason=`deep_conflict_target_state` anchor=`P-1:L11-L11, L21-L23, L49-L50` (ledger=P-002, line=L71): Opening prose includes one-click gated auto-repair, but Tier-1 ships diagnosis + propose-fix only; apply-button is Tier-2 + ABORT-gated.
- **SUP-023** [superseded] id=`P-002/P2-C001` reason=`deep_conflict_target_state` anchor=`P-2:L10-L10, L20-L22` (ledger=P-002, line=L109): Steal/build names O-4 canvas edge rendering and third-party mount as core, but REVISED scope makes both Tier-2; Tier-1 is CLI registry/wire/SSB only.
- **SUP-024** [superseded] id=`P-002/P6-C001` reason=`deep_conflict_target_state` anchor=`P-6:L10-L10, L20-L22` (ledger=P-002, line=L143): Steal/build names dual enforcement at O-3 AND E-20, but REVISED Tier-1 ships only O-3 vault-read; E-20 is Tier-2.
- **SUP-025** [superseded] id=`P-002/P7-C001` reason=`deep_conflict_target_state` anchor=`P-7:L6-L10, L20-L22, L40-L41` (ledger=P-002, line=L177): Opening premise frames learned promotion from journaled outcomes as the core, but REVISED Tier-1 is manual asymmetry-guarded change; automatic learning loop is Tier-2 behind N-3 confirmation.
- **SUP-026** [superseded] id=`P-002/P14-C001` reason=`deep_conflict_target_state` anchor=`P-14:L18-L20` (ledger=P-002, line=L313): Industry/summary framing often pairs Playwright + PyAutoGUI as peer emitters; numbered source puts browser Playwright in Tier-1 and desktop pyautogui in Tier-2.
- **SUP-027** [superseded] id=`P-002/P19-C001` reason=`deep_conflict_target_state` anchor=`P-19:L18-L20, L64-L65` (ledger=P-002, line=L346): Summary inventory lists VACUUM backups, restart continuation, post-upgrade probes, and compaction invariants as peer requirements; numbered REVISED scope places those in Tier-2 while Tier-1 is config lifecycle guards + doctor. INTEGRATION_SMOKE still references mid-turn resume.
- **SUP-028** [superseded] id=`P-003/SD-P15-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L507-L521` (ledger=P-003, line=L375): **CORRECT** summary inventory that lists narrative/episodic tiers, embedding retrieval, and bBoN as if Tier-1: source defers all three to Tier-2; Tier-1 is gated retrieve/distill/fuse + reuse-hit + default-off.
- **SUP-029** [superseded] id=`P-003/SD-P20-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L841-L848` (ledger=P-003, line=L395): **CORRECT** summary lists Dreaming promotion gates and per-provider cache mapping as atomic inventory items (P20-E18..E22): source DEFERs both to Tier-2; Tier-1 is flush + on-demand index only.
- **SUP-030** [superseded] id=`P-003/SD-P20-005` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L848-L848` (ledger=P-003, line=L399): **QUALIFY** frontend N/A beyond existing cost/memory panels — no new UI.
- **SUP-031** [superseded] id=`P-003/SD-P22-001` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L914-L939` (ledger=P-003, line=L405): **CORRECT** summary inventory treats nine-step channel-binding, pairing codes, dmScope, signed device pairing, unknown-event withhold, idempotency keys, and trusted-policy hooks as peer Tier-1 atoms: source DEFERs all of those to Tier-2; Tier-1 is only fenced returns + depth-scoped tool-strip.
- **SUP-032** [superseded] id=`P-003/SD-P25-006` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L983-L999` (ledger=P-003, line=L430): **CORRECT/QUALIFY** summary P25-E24 KEY_ALIASES and P25-E38 "import Q-10 modules": source requires exo-card V2 grammar as M3 deliverable and forbids growing/importing Q-10 runner; only stdlib paired-bootstrap extraction is shared.
- **SUP-033** [superseded] id=`P-003/SD-P25-008` reason=`summary_delta_corrects_or_qualifies` anchor=`16-P:L1026-L1026` (ledger=P-003, line=L432): **QUALIFY** summary A03 path reconciliation: plan still names exo_retrieval.py/exo_injection_ab.py as produces_symbols; tree drift is orientation-only until execution recon, not a source atom of absence.
- **SUP-034** [superseded] id=`P-003/P3-C001` reason=`deep_conflict_target_state` anchor=`P-3:L7-L10, L21-L23` (ledger=P-003, line=L64): Opening premise describes full import+regenerate flow, but REVISED scope ships Tier-1 without persona regen; regen is Tier-2.
- **SUP-035** [superseded] id=`P-003/P15-C001` reason=`deep_conflict_target_state` anchor=`P-15:L20-L20, L51-L51` (ledger=P-003, line=L96): Wargame EXPECTED_OBSERVATION asserts bBoN ranks candidates, but REVISED scope defers bBoN to Tier-2; Tier-1 acceptance must not require bBoN rows.
- **SUP-036** [superseded] id=`P-003/P20-C001` reason=`deep_conflict_target_state` anchor=`P-20:L6-L6, L18-L20` (ledger=P-003, line=L166): Premise lists Dreaming and per-provider cache-control as unit goals, but REVISED scope defers both to Tier-2; Tier-1 is flush + on-demand index only.
- **SUP-037** [superseded] id=`P-003/P22-C001` reason=`deep_conflict_target_state` anchor=`P-22:L6-L9, L19-L21, L42-L45` (ledger=P-003, line=L202): Premise and stub_ban enumerate channel-binding, signed pairing, unknown-event gating, idempotency, and trusted-policy hooks, but REVISED scope ships only fenced returns + tool-strip in Tier-1.

Superseded notes count: **37**

## User-facing surfaces

Every atom that names a window, panel, drawer, command, CLI verb, flag, status display, or notification. Full text retained; feeds dashboard DESIGN.md. Reference by namespaced ID.

Deep+IX surface-tagged records: **160**. Summary surface-tagged records: **48**.

### Surfaces — P-001/P-0 (8)

- **P-001/P0-E007** [SURFACE] class=`E/CAP` anchor=`P-0:L27-L28` (L42): P-11 is `elai workflow from-goal`: intent to CLOSED-enum-constrained proposed TaskSpec graph on O-4 canvas, human-confirmed before run, with L-14 RRA receipt.
- **P-001/P0-E010** [SURFACE] class=`E/CAP` anchor=`P-0:L42-L44` (L45): P-12 desktop/GUI operator lane: cua sandbox substrate + OmniParser Tier-3 pixel fallback + openclaw frameId/display fail-closed binding + per-action bounded screen-recording + crosshair/zoom crops, all under ELAI gates; cua in-loop safety is a stubbed TODO.
- **P-001/P0-E020** [SURFACE] class=`E/CAP` anchor=`P-0:L101-L104` (L55): P-19 config/ops lifecycle + `elai doctor` + verified backups: fail-closed defaults, LKG/rejected-write/shrink-guard/atomic writes, doctor detect/repair/lint, VACUUM-INTO backups, restart continuation, compaction split-rule.
- **P-001/P0-E030** [SURFACE] class=`E/CAP` anchor=`P-0:L161-L162, L224-L225` (L65): P-3 persona/mechanism split + `elai app export/import` re-verified on recipient seat (answers T5).
- **P-001/P0-E032** [SURFACE] class=`E/CAP` anchor=`P-0:L165-L166, L228-L229` (L67): P-5 graduated-ceremony toy/personal profile + non-coder operator surface (answers T1 non-coder).
- **P-001/P0-E039** [SURFACE] class=`E/TEST` anchor=`P-0:L211-L214` (L74): DONE definition: cargo check GREEN; auditor PASS on affected crates only; wiring_contract_check --run-trigger PASS; wire checker clean or WIRE-DARK[P-N]; cross_lang_diff for P-2/P-5 frontend pairs; corpus_composition_check (N-2) PASS; R10 smoke green; memory-writer ran.
- **P-001/P0-E052** [SURFACE] class=`E/LIMIT` anchor=`P-0:L344-L346` (L87): Carried correction P-14: openclaw jsonl-replay is INPUT-through-mock not real record→re-score; cite cua replay_env; script without minting-run receipt refused.
- **P-001/P0-E061** [SURFACE] class=`E/SURFACE` anchor=`P-0:L133-L137` (L96): P ships new user-facing features: app fleet, inter-tool bus, hand-to-friend export, living brains, non-coder operator surface, life-data privacy lattice, per-user operating profile, small-team world.

### Surfaces — P-001/P-4 (2)

- **P-001/P4-E013** [SURFACE] class=`E/SURFACE` anchor=`P-4:L31-L31, L55-L55, L58-L59, L72-L72` (L121): CLI production caller: `elai brain mount\|sync <store>`.
- **P-001/P4-E019** [SURFACE] class=`E/INT` anchor=`P-4:L58-L61` (L127): Reachability: live CLI mount/sync + live filesystem-watch on vault edit; `#[cfg(test)]` never satisfies R7.

### Surfaces — P-001/P-5 (10)

- **P-001/P5-E002** [SURFACE] class=`E/SURFACE` anchor=`P-5:L7-L7` (L145): Two missing pieces: fleet-grain plain-language operation (“mywardrobe tool broke” → P-1 diagnosis as sentence + gated repair button) and graduated ceremony so toy tools do not pay trading-strategy gate tax.
- **P-001/P5-E007** [SURFACE] class=`E/CAP` anchor=`P-5:L21-L21, L52-L55` (L150): Tier-1: CeremonyProfile type + toy/personal wiring into K-5 observe-only + K-4 tax measurement + one-line receipt + N-7 fleet-grain plain-language render of P-1 red tick.
- **P-001/P5-E008** [SURFACE] class=`E/DEFER` anchor=`P-5:L22-L22` (L151): Tier-2: one-click gated repair BUTTON (with P-1 Tier-2) + full non-coder GUI operator panel.
- **P-001/P5-E009** [SURFACE] class=`E/SURFACE` anchor=`P-5:L26-L26, L53-L53, L59-L59` (L152): CLI: `elai profile set <tier>` + `elai profile tax`; fleet render reads through `elai fleet list` (P-1).
- **P-001/P5-E010** [SURFACE] class=`E/SURFACE` anchor=`P-5:L27-L27` (L153): Tier-2 frontend_pair: fleet operator panel in dashboard-v2 with N-7 plain-language cards + repair button.
- **P-001/P5-E013** [SURFACE] class=`E/OBS` anchor=`P-5:L33-L35, L64-L64, L70-L70` (L156): M2: one-line receipt POINTS at full journal entry (never replaces); full evidence retrievable.
- **P-001/P5-E014** [SURFACE] class=`E/OBS` anchor=`P-5:L36-L38` (L157): M3: `elai profile tax --tier toy` reports measured composed gate tax (near-zero for A-3); red tick as one N-7 sentence example: “your wardrobe tool broke: its weather dependency is stale”.
- **P-001/P5-E019** [SURFACE] class=`E/INT` anchor=`P-5:L58-L61, L72-L72` (L162): Production reachability: `elai profile set|tax` CLI + live gated action at toy tier; test callers never R7.
- **P-001/P5-E023** [SURFACE] class=`E/SURFACE` anchor=`P-5:L7-L7, L21-L27, L33-L38` (L166): User-facing: ceremony profile selection CLI; tax report CLI; one-line receipts; plain-language fleet diagnosis sentence; (Tier-2) repair button + dashboard fleet operator panel.
- **P-001/P5-A003** [SURFACE] class=`A/SURFACE` anchor=`P-5:L22-L22, L27-L27` (L170): Exact dashboard drawer/panel identity for Tier-2 operator surface is deferred without wireframe.

### Surfaces — P-001/P-8 (5)

- **P-001/P8-E008** [SURFACE] class=`E/SURFACE` anchor=`P-8:L26-L26, L56-L56, L60-L60` (L187): CLI: `elai world join\|merge\|whodid <query>` is Tier-1 production caller.
- **P-001/P8-E011** [SURFACE] class=`E/SURFACE` anchor=`P-8:L34-L34, L71-L71` (L190): `elai world whodid <namespace>` returns seat+agent for each change.
- **P-001/P8-E016** [SURFACE] class=`E/INT` anchor=`P-8:L59-L62, L73-L73` (L195): Reachability: live `elai world` CLI + shared-namespace write path enforces lock; test never R7.
- **P-001/P8-E018** [SURFACE] class=`E/TEST` anchor=`P-8:L81-L81` (L197): INTEGRATION_SMOKE: independent agent simulates TWO seats with agents, interleaved+concurrent events, A→B and B→A merge, whodid attribution, serialized concurrent write (R10); single-seat happy path insufficient.
- **P-001/P8-I001** [SURFACE] class=`I/SURFACE` anchor=`P-8:L22-L22, L26-L26, L34-L34` (L200): Agent-attribution query surface is Tier-2 even though `whodid` is Tier-1 CLI; Tier-1 whodid is the minimal attribution query, richer surface deferred.

### Surfaces — P-001/P-12 (6)

- **P-001/P12-E006** [SURFACE] class=`E/LIMIT` anchor=`P-12:L21-L21` (L217): Excluded: cua’s own in-loop gating (stubbed TODO — ELAI supplies gate); ungated self-extension.
- **P-001/P12-E008** [SURFACE] class=`E/INT` anchor=`P-12:L37-L40, L75-L76` (L219): Production reachability: task_runner O-2 tick dispatching `operator_action` TaskSpec stage + CLI `elai operator run`; test/contracts never R7.
- **P-001/P12-E010** [SURFACE] class=`E/SURFACE` anchor=`P-12:L46-L47` (L221): frontend_pair: operator-action rows + screen-recording thumbnail in an existing dashboard drawer (exact drawer decided at edit, W-4).
- **P-001/P12-E018** [SURFACE] class=`E/TEST` anchor=`P-12:L59-L64` (L229): VERIFY via plan_gate_check, wiring_contract_check, production_caller_check; proof is off-agent operator_action + recording-hash match via real CLI.
- **P-001/P12-E021** [SURFACE] class=`E/SURFACE` anchor=`P-12:L38-L39, L46-L47, L49-L49` (L232): User-facing surfaces: `elai operator run` CLI; operator_action TaskSpec stage; O-4/operator node implied via task stage; dashboard drawer rows + recording thumbnail.
- **P-001/P12-A002** [SURFACE] class=`A/SURFACE` anchor=`P-12:L46-L47` (L236): Exact existing dashboard drawer identity left to W-4 at edit.

### Surfaces — P-001/P-13 (6)

- **P-001/P13-E003** [SURFACE] class=`E/EVIDENCE` anchor=`P-13:L9-L9` (L249): OmniParser captions+IDs lift grounding 70.5→93.8%; ScreenSpot 73.0 vs raw GPT-4V 16.2 (comparator evidence, not ELAI threshold).
- **P-001/P13-E011** [SURFACE] class=`E/SURFACE` anchor=`P-13:L43-L44` (L257): frontend_pair: rides J-1 existing perception tile — source-tagged element breakdown (no new surface).
- **P-001/P13-E012** [SURFACE] class=`E/SURFACE` anchor=`P-13:L46-L46` (L258): Observable CLI trigger in contract: `elai perception scaffold --fixture ... --json`.
- **P-001/P13-E017** [SURFACE] class=`E/FAIL` anchor=`P-13:L55-L55` (L263): ABORT if source tag requires second perception scaffold or second provenance scale (A-1/N-2).
- **P-001/P13-E018** [SURFACE] class=`E/TEST` anchor=`P-13:L56-L60` (L264): VERIFY via plan_gate + wiring_contract; proof off-agent row + zero VLM count trace via real `elai perception scaffold`.
- **P-001/P13-C001** [SURFACE] class=`C/SURFACE` anchor=`P-13:L46-L46, L71-L72` (L268): Detail documents CLI trigger `elai perception scaffold` as acceptance entrypoint while PRODUCTION_HOOK says no new entrypoint (extend gateway in place). Surface may be existing CLI wrapping gateway, but “add no CLI” summary language conflicts with this trigger.

### Surfaces — P-001/P-16 (7)

- **P-001/P16-E003** [SURFACE] class=`E/LIMIT` anchor=`P-16:L9-L9, L21-L21, L40-L40` (L284): Sanctioned form is P-9 HEADLESS code-pack wrapping app CLI/API (e.g. `blender --background --python`); GUI computer-use is last resort not mechanism.
- **P-001/P16-E007** [SURFACE] class=`E/SURFACE` anchor=`P-16:L19-L19, L36-L36, L46-L46, L71-L72` (L288): CLI: `elai connector install <tool>` resolves catalog row to P-9 headless code-pack.
- **P-001/P16-E009** [SURFACE] class=`E/LIMIT` anchor=`P-16:L21-L21, L41-L41` (L290): Excluded: agent-zero ungated apt-get; openclaw advisory regex scanner; GUI-driving install when headless CLI exists.
- **P-001/P16-E012** [SURFACE] class=`E/SURFACE` anchor=`P-16:L43-L44` (L293): frontend_pair Tier-2: catalog + admission verdict in connector surface (W-4).
- **P-001/P16-E015** [SURFACE] class=`E/FAIL` anchor=`P-16:L48-L48` (L296): NEGATIVE: bypass L-12 or GUI install despite headless CLI FAILS.
- **P-001/P16-E016** [SURFACE] class=`E/DEFER` anchor=`P-16:L53-L53` (L297): FORK: if tool has no headless CLI/API, route P-12 operator-lane stage (taint-tagged, never load-bearing), not refuse-or-drive-as-infra.
- **P-001/P16-E020** [SURFACE] class=`E/TEST` anchor=`P-16:L56-L60` (L301): VERIFY: off-agent install_catalog_admit + flagged refusal via real CLI; plan_gate + wiring_contract.

### Surfaces — P-001/P-18 (3)

- **P-001/P18-E009** [SURFACE] class=`E/SURFACE` anchor=`P-18:L36-L36, L72-L73` (L322): CLI: `elai workflow resume\|cancel`.
- **P-001/P18-E010** [SURFACE] class=`E/INT` anchor=`P-18:L36-L38` (L323): Production: CLI resume/cancel + FSM gates re-validating bound plan; test never R7.
- **P-001/P18-E012** [SURFACE] class=`E/SURFACE` anchor=`P-18:L44-L45` (L325): frontend_pair: run-revision + drift-reject rows in O-4 workflow view (W-4).

### Surfaces — P-001/P-21 (2)

- **P-001/P21-E011** [SURFACE] class=`E/SURFACE` anchor=`P-21:L42-L43` (L360): frontend_pair: rides existing gate/evidence surface (corpus kill-rate + posture label; W-4).
- **P-001/P21-E012** [SURFACE] class=`E/SURFACE` anchor=`P-21:L45-L45` (L361): CLI trigger: `elai bench gate-corpus --with-canaries --break-detector redaction`.

### Surfaces — P-001/CROSS (3)

- **P-001/IX-002** [SURFACE] class=`IX/SURFACE` anchor=`P0-E028, P0-E041; P5-E002, P5-E007–P5-E008` (L383): Personal WAVE 1 pairs P-5 ceremony with P-1 KEYSTONE fleet; P-5 renders P-1 red ticks and defers repair button to P-1 Tier-2.
- **P-001/IX-019** [SURFACE] class=`IX/SURFACE` anchor=`P4-E013; P5-E009; P8-E008; P12-E008; P13-E012; P16-E007; P18-E009; P21-E012` (L400): User-facing CLI verb set in this chunk: `elai brain mount\|sync`, `elai profile set\|tax`, `elai world join\|merge\|whodid`, `elai operator run`, `elai perception scaffold`, `elai connector install`, `elai workflow resume\|cancel`, `elai bench gate-corpus`.
- **P-001/IX-020** [SURFACE] class=`IX/SURFACE` anchor=`P5-E010; P12-E010; P13-E011; P16-E012; P18-E012; P21-E011` (L401): Dashboard/frontend pairs in this chunk: P-5 fleet operator panel (Tier-2); P-12 drawer thumbnail; P-13 perception tile reuse; P-16 connector surface (Tier-2); P-18 O-4 revision/drift rows; P-21 existing gate/evidence surface.

### Surfaces — P-002/P-1 (13)

- **P-002/P1-E008** [SURFACE] class=`E/SEC` anchor=`P-1:L11-L11, L18-L18, L49-L50, L64-L65` (L44): When a tick goes red, ELAI proposes a fix that MUST pass the same wiring/claim gates as any changeset (never a silent patch), rendered human-first via N-7, one-click apply — gated AUTO-REPAIR.
- **P-002/P1-E010** [SURFACE] class=`E/DEP` anchor=`P-1:L17-L17` (L46): L-14 `elai build "<idea>"` emits an RRA receipt; the manifest's `rra_lineage` is that receipt's id; the fleet registers what L-14 builds.
- **P-002/P1-E011** [SURFACE] class=`E/DEP` anchor=`P-1:L19-L19` (L47): M-4's signed-updater manifest/rollback pattern is the shape; M-4 updates ELAI itself while P-1 applies the pattern per-user-app (GAP-1's distinction).
- **P-002/P1-E012** [SURFACE] class=`E/CFG` anchor=`P-1:L21-L22` (L48): SHIP NOW (Tier-1): AppManifest schema + FleetRegistry SQLite + `elai fleet register <app>` CLI + health-tick TaskSpec template with Smoke + DepRotCheck stages (reusing O-2 + F-9) + red-tick DashboardInbox card via N-7.
- **P-002/P1-E013** [SURFACE] class=`E/DEFER` anchor=`P-1:L22-L23, L41-L42` (L49): DEFER (Tier-2): per-app drift canaries (needs G-10 generalized — asserted-not-designed per A-6) + gated auto-repair apply-button (changeset/claim-gate seam).
- **P-002/P1-E015** [SURFACE] class=`E/SURFACE` anchor=`P-1:L27-L27, L54-L55, L59-L60, L73-L73` (L51): CLI surface: `elai fleet register\|list\|tick <app>` subcommand (cli/main.rs) is the Tier-1 production caller.
- **P-002/P1-E016** [SURFACE] class=`E/SURFACE` anchor=`P-1:L28-L28` (L52): Frontend: DashboardInbox (O-2 intervention-card sink) renders the red-tick card; a dedicated FleetPanel is deferred to P-5's operator surface.
- **P-002/P1-E020** [SURFACE] class=`E/SURFACE` anchor=`P-1:L37-L39, L68-L71` (L56): M3: red tick renders one N-7 DashboardInbox card (plain-language diagnosis: which app, what broke, one sentence); green tick auto-archives and leaves only the SSB row, no card.
- **P-002/P1-E023** [SURFACE] class=`E/LIMIT` anchor=`P-1:L49-L50` (L59): ABORT: if gated auto-repair would apply without wiring_contract_check / claim gates, stop and flag; ship diagnosis + propose-fix render only; apply-button lands only once it routes through the real gate seam.
- **P-002/P1-E024** [SURFACE] class=`E/INT` anchor=`P-1:L59-L62, L73-L73` (L60): Production reachability: `elai fleet register` on a live run AND task_runner interval fires a due health-tick TaskSpec; a `#[cfg(test)]` caller NEVER satisfies R7.
- **P-002/P1-E025** [SURFACE] class=`E/TEST` anchor=`P-1:L75-L79` (L61): VERIFY: wiring_contract_check + production_caller_check; pass = `fleet_health_tick` row landed off-agent from real task_runner tick (or `elai fleet tick`), fired exactly once, red-tick inbox card for broken-app, auto-archive for green.
- **P-002/P1-A001** [SURFACE] class=`A/DEP` anchor=`P-1:L44-L45` (L66): Whether G-10's canary set is parameterizable by an app's smoke set or hard-bound to ELAI fixtures is RECON_NEEDED before Tier-1 drift-canary scope.
- **P-002/P1-C002** [SURFACE] class=`C/DEFER` anchor=`P-1:L11-L11, L21-L23, L49-L50` (L71): Opening prose includes one-click gated auto-repair, but Tier-1 ships diagnosis + propose-fix only; apply-button is Tier-2 + ABORT-gated.

### Surfaces — P-002/P-2 (8)

- **P-002/P2-E006** [SURFACE] class=`E/SURFACE` anchor=`P-2:L10-L10, L36-L38` (L86): An edge whose source port has NO consumer renders VISIBLY DISCONNECTED (O-4 wire-dark-made-visual at world grain).
- **P-002/P2-E009** [SURFACE] class=`E/CFG` anchor=`P-2:L20-L21` (L89): SHIP NOW (Tier-1): PortSchema extension + PortRegistry + static port-compatibility check (`elai fleet wire <appA.out> <appB.in>`) refusing incompatible schemas + journaled `tool_port_edge` SSB row + G-1 taint tier on the edge record.
- **P-002/P2-E011** [SURFACE] class=`E/SURFACE` anchor=`P-2:L26-L26, L36-L38, L53-L54, L68-L70` (L91): CLI: `elai fleet wire\|ports <app>` is the Tier-1 production caller; Tier-1 exposes zero-consumer outputs via `elai fleet ports --unwired`.
- **P-002/P2-E012** [SURFACE] class=`E/SURFACE` anchor=`P-2:L27-L27, L59-L60` (L92): frontend_pair Tier-2: dashboard-v2 O-4 WORKFLOW view renders tool->tool edges with N-6 tier chrome + disconnected-source affordance.
- **P-002/P2-E016** [SURFACE] class=`E/CFG` anchor=`P-2:L40-L41` (L96): FORK: if O-4 edge model is hard-bound to research-pipeline stage enum and cannot carry tool->tool edges without restructuring, ship Tier-1 CLI fully and route ALL canvas rendering to Tier-2; trust value is realized at CLI grain without canvas.
- **P-002/P2-E018** [SURFACE] class=`E/INT` anchor=`P-2:L58-L61, L72-L72` (L98): Production reachability: `elai fleet wire` binds on a live run; Tier-2 O-4 WORKFLOW canvas renders/creates the edge; test callers never satisfy R7.
- **P-002/P2-E019** [SURFACE] class=`E/TEST` anchor=`P-2:L74-L78` (L99): VERIFY: wiring_contract_check + production_caller_check; pass = `tool_port_edge` row from real `elai fleet wire` carrying N-6 + G-1 taint, incompatible wire refused with no edge row.
- **P-002/P2-C001** [SURFACE] class=`C/DEFER` anchor=`P-2:L10-L10, L20-L22` (L109): Steal/build names O-4 canvas edge rendering and third-party mount as core, but REVISED scope makes both Tier-2; Tier-1 is CLI registry/wire/SSB only.

### Surfaces — P-002/P-6 (4)

- **P-002/P6-E008** [SURFACE] class=`E/DEFER` anchor=`P-6:L21-L22, L27-L27` (L126): DEFER (Tier-2): E-20 routing-boundary enforcement (lands with P-4 living-brain routing wall) + grant-management operator surface (frontend with P-5).
- **P-002/P6-E010** [SURFACE] class=`E/SURFACE` anchor=`P-6:L26-L26, L54-L55, L58-L59` (L128): CLI: `elai privacy grant\|revoke\|show <tool> <namespace>` is the Tier-1 production caller.
- **P-002/P6-E012** [SURFACE] class=`E/RECOVERY` anchor=`P-6:L36-L38` (L130): `elai privacy revoke` is the inverse journaled op; tool loses read access after revoke.
- **P-002/P6-E015** [SURFACE] class=`E/INT` anchor=`P-6:L58-L61, L72-L72` (L133): Production reachability: `elai privacy grant\|revoke` on live run AND O-3 vault-read path enforces grants on live read; test callers never satisfy R7.

### Surfaces — P-002/P-7 (3)

- **P-002/P7-E009** [SURFACE] class=`E/SURFACE` anchor=`P-7:L26-L26, L53-L54, L57-L58, L70-L70` (L161): CLI: `elai profile show\|tighten\|loosen <domain>` is the Tier-1 production caller; loosen refuses without evidence.
- **P-002/P7-E014** [SURFACE] class=`E/INT` anchor=`P-7:L57-L59, L70-L70` (L166): Production reachability: `elai profile tighten\|loosen` CLI on a live run; test callers never satisfy R7.
- **P-002/P7-E017** [SURFACE] class=`E/SEC` anchor=`P-7:L80-L81` (L169): REDTEAM failed: talk ELAI into lowering a security gate by acting frustrated — loosening gated on measured FP evidence not affect. Succeeded attack: loosen using another user's or global FP rate — patched by scoping FP evidence to THAT user's own runs only.

### Surfaces — P-002/P-9 (2)

- **P-002/P9-E010** [SURFACE] class=`E/SURFACE` anchor=`P-9:L50-L52, L64-L66` (L196): MOVE 4: `elai connector new <name>` scaffolds declarative pack template (LANE A); ELAI fills endpoints/auth/schema-map with anti-slopsquat + source-grounding recon; LANE B code pack implements closed trait + mounts through same gate.
- **P-002/P9-E013** [SURFACE] class=`E/CFG` anchor=`P-9:L33-L33` (L199): FORK-2: pack signing reuses existing ELAI RRA/.rra signing key if applicable; else scoped key is RECON item.

### Surfaces — P-002/P-10 (1)

- **P-002/P10-E011** [SURFACE] class=`E/SURFACE` anchor=`P-10:L35-L35, L73-L73` (L233): CLI credential acquisition v1: `elai cred add` manual paste; OAuth flows (device-code vs localhost-redirect, refresh rotation) are DECLARED follow-on; if a P-9 connector needs OAuth before follow-on lands, that connector is BLOCKED on it, not hand-rolled inline.

### Surfaces — P-002/P-11 (10)

- **P-002/P11-E001** [SURFACE] class=`E/STATUS` anchor=`P-11:L1-L5` (L257): P-11 is PLANNED product surface post-spine; authored 2026-07-08; PROOF_SPINE none; PSR-2 ack 2026-07-09; user-facing composition demo riding P-9 vocabulary + P-10 ingress; n8n-user-converting front door.
- **P-002/P11-E002** [SURFACE] class=`E/CAP` anchor=`P-11:L6-L7, L41-L43` (L258): `elai workflow from-goal <intent>` has the model PROPOSE a TaskSpec graph drawn ONLY from closed stage/connector vocabulary (O A3 kinds + P-9 mounted connectors), rendered on O-4 canvas, human CONFIRMS before anything runs.
- **P-002/P11-E004** [SURFACE] class=`E/DEP` anchor=`P-11:L12-L18` (L260): Substrate: H-13 model-drives-canvas-ops (constrained mutation seam); O-2 TaskSpec rows + triggers; O-4 canvas + wire-dark; L-14 elai build RRA receipt pattern; P-9 mounted connectors + O A3 closed enum; PortSchema contracts.
- **P-002/P11-E006** [SURFACE] class=`E/CFG` anchor=`P-11:L34-L34, L67-L68` (L262): FORK-2: intent needs unmounted connector -> render VISIBLY DISCONNECTED gap + offer `elai connector new` (P-9 LANE A); do NOT auto-author connector inside from-goal.
- **P-002/P11-E009** [SURFACE] class=`E/LIMIT` anchor=`P-11:L38-L39` (L265): ABORT-3: proposal WITHOUT RRA receipt (intent -> graph, provable) -> STOP; L-14 receipt makes synthesis auditable.
- **P-002/P11-E010** [SURFACE] class=`E/SURFACE` anchor=`P-11:L41-L43, L64-L67` (L266): MOVE 1 CLI: `elai workflow from-goal <intent>` proposes inert TaskSpec graph (no trigger armed).
- **P-002/P11-E011** [SURFACE] class=`E/SURFACE` anchor=`P-11:L44-L47` (L267): MOVE 2: proposed graph renders on O-4 canvas; unavailable capability VISIBLY DISCONNECTED with "needs a P-9 connector pack" hint.
- **P-002/P11-E013** [SURFACE] class=`E/EVENT` anchor=`P-11:L51-L53` (L269): MOVE 4: from-goal run journals `{intent, proposed graph, model, confirmation}` as RRA receipt (L-14 pattern); receipt verifies via G-2.
- **P-002/P11-E015** [SURFACE] class=`E/INT` anchor=`P-11:L58-L59` (L271): PRODUCTION_HOOK: CLI from-goal -> H-13 canvas-op + O-4 render + O-2 TaskSpec store (inert rows); confirm gate -> O-2 trigger-arm path; from-goal run -> L-14 RRA sink. Seams RECON at pre-flight.
- **P-002/P11-A002** [SURFACE] class=`A/SURFACE` anchor=`P-11:L29-L29` (L279): Whether O-4 maps proposed-but-unavailable node to disconnected visual + connector-new hint is RECON-2.

### Surfaces — P-002/P-14 (6)

- **P-002/P14-E002** [SURFACE] class=`E/CAP` anchor=`P-14:L5-L5, L18-L19` (L292): Exploration run emits a deterministic script AS an O-2 TaskSpec stage + RRA receipt binding script->minting-run; replay = cheap zero-reasoning execution mode.
- **P-002/P14-E008** [SURFACE] class=`E/SURFACE` anchor=`P-14:L35-L36, L46-L48, L71-L72` (L298): CLI: `elai e2e crystallize` + replay via task_runner tick (`elai e2e replay --stage`); PRODUCTION CLI + task_runner.
- **P-002/P14-E009** [SURFACE] class=`E/SURFACE` anchor=`P-14:L43-L44` (L299): frontend_pair: crystallized-stage node on O-4 canvas (Tier-2; decide at edit W-4).
- **P-002/P14-E016** [SURFACE] class=`E/TEST` anchor=`P-14:L56-L60` (L306): VERIFY: plan_gate_check + wiring_contract_check; load-bearing proof is off-agent crystallized_stage_minted + replay trace asserting zero model calls via real `elai e2e crystallize`/`replay`.
- **P-002/P14-E017** [SURFACE] class=`E/TEST` anchor=`P-14:L63-L64` (L307): INTEGRATION_SMOKE: independent agent full E-14 multi-step SaaS journey (login + multi-page form), crystallize, replay N times byte-identical under seed with zero model calls, `elai replay --verify` re-binds script to minting run. Toy single-step does NOT satisfy R10.
- **P-002/P14-E018** [SURFACE] class=`E/INT` anchor=`P-14:L71-L72` (L308): PRODUCTION_HOOK: task_runner TaskSpecStage gains crystallized_script kind; `elai e2e crystallize`/`replay` in cli/main.rs; Crystallizer + RRA binder wired both same-changeset.

### Surfaces — P-002/P-19 (11)

- **P-002/P19-E002** [SURFACE] class=`E/CAP` anchor=`P-19:L5-L5` (L324): Add recoverable config, `elai doctor`, verified journal backups, and a restart continuation queue so a trust product cannot cheaply corrupt its own config or fail to restore state.
- **P-002/P19-E003** [SURFACE] class=`E/EVIDENCE` anchor=`P-19:L8-L9` (L325): Steal from openclaw: strict-schema fail-closed boot (unknown key = REFUSE), atomic writes, last-known-good with doctor-only restore, clobber guards (>50% shrink rejected, preserved as .rejected.<ts>), hot-reload settle + invalid-edit rejection; doctor detect/repair/lint with STABLE exit codes; VACUUM-INTO WAL-safe SQLite snapshots + manifest verify; restart continuation queue (one turn); compaction split-rule (tool-call/result pairs together) + successor transcripts (never rewrite in place).
- **P-002/P19-E005** [SURFACE] class=`E/CFG` anchor=`P-19:L18-L19` (L327): SHIP NOW (Tier-1): (a) fail-closed-default-object config posture (policy objects fail closed even when ABSENT) + last-known-good + rejected-write preservation (.rejected.<ts>) + shrink/clobber guards (>50% shrink rejected) + atomic writes on J config-ingress gate; (b) `elai doctor` detect/repair/lint with STABLE exit codes + structured findings; ENV ALLOWLISTED; cwd path-jail.
- **P-002/P19-E008** [SURFACE] class=`E/DATA` anchor=`P-19:L29-L31` (L330): produces: `crates/orchestrator/src/config/lifecycle.rs` (ConfigLifecycle: LKG + rejected-write + shrink/clobber + atomic write + env allowlist) + `crates/cli/src/doctor_cmd.rs` (elai doctor detect/repair/lint).
- **P-002/P19-E010** [SURFACE] class=`E/SURFACE` anchor=`P-19:L35-L36, L44-L45, L56-L57` (L332): CLI: `elai doctor` subcommand (detect/repair/lint); PRODUCTION CLI. No dashboard tile — justified W-6 N/A: structured text + stable exit codes for CI.
- **P-002/P19-E012** [SURFACE] class=`E/LIMIT` anchor=`P-19:L39-L43` (L334): stub_ban: NO config write overwriting on >50% shrink; NO full process.env copy; NO doctor exit code varying run-to-run for same state; NO compaction splitting tool-call from result; NO in-place transcript rewrite; NO .unwrap; NO blake2/sha2.
- **P-002/P19-E013** [SURFACE] class=`E/EVENT` anchor=`P-19:L46-L49, L51-L53` (L335): >50%-shrink write REJECTED and preserved as .rejected.<ts> with `config_write_rejected` row; `elai doctor lint` returns STABLE nonzero exit + structured `doctor_finding` rows.
- **P-002/P19-E014** [SURFACE] class=`E/CFG` anchor=`P-19:L54-L54` (L336): FORK: if config is corrupt beyond last-known-good restore, `elai doctor repair` refuses to auto-repair and surfaces structured finding for human action; do NOT silently write a default over user config.
- **P-002/P19-E017** [SURFACE] class=`E/TEST` anchor=`P-19:L57-L61` (L339): VERIFY: plan_gate_check + wiring_contract_check; load-bearing proof is off-agent config_write_rejected + doctor_finding rows + stable doctor exit via real `elai doctor`.
- **P-002/P19-E018** [SURFACE] class=`E/TEST` anchor=`P-19:L64-L65` (L340): INTEGRATION_SMOKE: hard input malformed config (unknown key) + >50% shrink + kill mid-turn; assert fail-closed boot rejects unknown key, shrink preserved not overwritten, doctor exit stable across 3 runs, in-flight turn resumes once after restart. Note: resume leg is Tier-2 deferred capability used in smoke description.
- **P-002/P19-C001** [SURFACE] class=`C/DEFER` anchor=`P-19:L18-L20, L64-L65` (L346): Summary inventory lists VACUUM backups, restart continuation, post-upgrade probes, and compaction invariants as peer requirements; numbered REVISED scope places those in Tier-2 while Tier-1 is config lifecycle guards + doctor. INTEGRATION_SMOKE still references mid-turn resume.

### Surfaces — P-002/P-24 (4)

- **P-002/P24-E005** [SURFACE] class=`E/SURFACE` anchor=`P-24:L17-L18, L45-L46` (L360): RUNG 1: device_frame render variant on H-14 preview node and P-23 browse node (CDP Emulation.setDeviceMetricsOverride); E-14 journeys accept device MATRIX; SSIM goldens become per-profile.
- **P-002/P24-E014** [SURFACE] class=`E/SURFACE` anchor=`P-24:L36-L37, L47-L48, L77-L78` (L369): CLI: `elai e2e run --device <profile>`; `elai operator run --backend emulator`.
- **P-002/P24-E016** [SURFACE] class=`E/SURFACE` anchor=`P-24:L38-L39, L45-L46` (L371): H-13 canvas dispatcher: device_frame render variant on preview/browse nodes; phone-bezel render + per-profile pass/fail chips; iOS-unavailable = HONEST-GAP disconnected affordance, never silent skip.
- **P-002/P24-E027** [SURFACE] class=`E/INT` anchor=`P-24:L77-L78` (L382): PRODUCTION_HOOK: cli device/emulator flags + task_runner device_profile + H-13 device_frame — DeviceProfile resolver + emulator backend wired into ALL THREE same-changeset (vertical slice; Rung 3 crystallize arm registers in P-14 target enum same-changeset).

### Surfaces — P-002/CROSS (4)

- **P-002/IX-011** [SURFACE] class=`IX/SURFACE` anchor=`P11-E002–P11-E007, P11-E011; P9-E010` (L404): P-11 proposals are constrained to O A3 closed kinds + P-9 mounted connectors; unmounted needs render as O-4 wire-dark gaps offering P-9 `elai connector new`, never fabricated nodes or auto-authored packs.
- **P-002/IX-015** [SURFACE] class=`IX/SURFACE` anchor=`P19-E005–P19-E006, P19-E019` (L408): P-19 Tier-1 extends J config-ingress with recoverable guards and `elai doctor`; Tier-2 backups/continuation protect the same evidence store other units journal into.
- **P-002/IX-018** [SURFACE] class=`IX/SURFACE` anchor=`P1-E024; P2-E018; P6-E015; P7-E014; P9-E017; P10-E014; P11-E015; P14-E008; P19-E010–P19-E011; P24-E027` (L411): Shared production-caller doctrine: every unit forbids `#[cfg(test)]`/contracts callers from satisfying R7 and names real CLI and/or scheduler/FSM production hooks.
- **P-002/IX-021** [SURFACE] class=`IX/SURFACE` anchor=`P1-E016, P1-E020; P2-E012; P6-E008; P11-E011–P11-E012; P14-E009; P19-E010; P24-E016` (L414): Dashboard DESIGN.md surfaces implied by this chunk: DashboardInbox red-tick cards (P-1), deferred FleetPanel (P-5), O-4 port edges + disconnected chrome (P-2 Tier-2), deferred privacy grant panel (P-5/P-6), O-4 proposed-graph + confirm UI (P-11), crystallized-stage node (P-14 Tier-2), device_frame + pass/fail chips + iOS honest-gap (P-24), intentionally no doctor dashboard tile (P-19).

### Surfaces — P-003/P-3 (8)

- **P-003/P3-E005** [SURFACE] class=`E/SURFACE` anchor=`P-3:L10-L10, L22-L22, L27-L27, L32-L32, L55-L55, L60-L60, L72-L72` (L38): CLI verb `elai app export <app>` writes the bundle plus a receipt.
- **P-003/P3-E006** [SURFACE] class=`E/SURFACE` anchor=`P-3:L10-L10, L22-L22, L27-L27, L34-L35, L55-L55, L60-L60, L72-L72` (L39): CLI verb `elai app import <bundle>` is the Tier-1 import production caller.
- **P-003/P3-E007** [SURFACE] class=`E/FLOW` anchor=`P-3:L10-L11, L15-L15, L22-L22, L34-L36, L64-L64` (L40): Import runs M-5 elai-verify against the bundle evidence, refuses on a failed receipt, then mounts the mechanism.
- **P-003/P3-E009** [SURFACE] class=`E/SEC` anchor=`P-3:L10-L10, L16-L16, L23-L23` (L42): Sharing surface is admission-gated: every artifact carries a receipt and passes the same L-12/skillspector admission gates used for skills; friend-to-friend and marketplace share one gate.
- **P-003/P3-E016** [SURFACE] class=`E/TEST` anchor=`P-3:L34-L36` (L49): M2: valid bundle re-verifies GREEN and mounts (`app_import` SSB); tampered mechanism or stale receipt is REFUSED with no mount.
- **P-003/P3-E018** [SURFACE] class=`E/RECOVERY` anchor=`P-3:L41-L42` (L51): FORK: if M-5 cannot yet verify a whole-app bundle, ship Tier-1 export + import-with-receipt-presence-check (refuse absent/malformed receipt), route full evidence re-verification to Tier-2; never ship a mount that skips verification entirely.
- **P-003/P3-E020** [SURFACE] class=`E/INT` anchor=`P-3:L59-L61, L72-L72` (L53): Production reachability is live `elai app export` / `elai app import` CLI; `#[cfg(test)]` never satisfies R7. PRODUCTION_HOOK is `cli/main.rs` subcommand dispatch.
- **P-003/P3-A004** [SURFACE] class=`A/SURFACE` anchor=`P-3:L23-L23, L27-L27` (L63): Friend-to-friend/marketplace hosting UX is deferred; no Tier-1 UI beyond CLI is named.

### Surfaces — P-003/P-15 (6)

- **P-003/P15-E005** [SURFACE] class=`E/LIMIT` anchor=`P-15:L21-L21, L39-L40` (L78): EXCLUDED: agent-s ungated KnowledgeBase commit; bBoN judge sourced from screenshots (ELAI has ground-truth journal deltas).
- **P-003/P15-E009** [SURFACE] class=`E/SURFACE` anchor=`P-15:L43-L44` (L82): frontend_pair rides J-96 memory CanvasMode node chrome; reuse-hit surfaced on the case node (W-4).
- **P-003/P15-E010** [SURFACE] class=`E/SURFACE` anchor=`P-15:L46-L46, L56-L56` (L83): CLI trigger surface: `elai case reuse --replay tests/fixtures/case/recurrence_bugclass_a.json`.
- **P-003/P15-E011** [SURFACE] class=`E/EVENT` anchor=`P-15:L47-L48, L71-L72` (L84): Expected SSB: `recurrence_hit_rate_measured`; K-18 bench `elai bench case-reuse` consumes reuse-hit.
- **P-003/P15-E015** [SURFACE] class=`E/TEST` anchor=`P-15:L56-L60` (L88): VERIFY proof is off-agent recurrence_hit_rate_measured + absence of case node for unproven fixture via real `elai case reuse`; plan_gate_check + wiring_contract_check.
- **P-003/P15-E016** [SURFACE] class=`E/SEC` anchor=`P-15:L61-L61` (L89): REDTEAM: poisoned case reuse defeated by mint-on-proof + strict reuse filter; bBoN narrative sway patched by scoring journal ROWS not model prose. Deep red-team deferred to execution pre-flight.

### Surfaces — P-003/P-17 (3)

- **P-003/P17-E014** [SURFACE] class=`E/SURFACE` anchor=`P-17:L44-L45, L49-L49` (L119): frontend_pair: failover state + billed-vs-estimate label on usage surface (RuntimePanel / CloudPanel; W-4).
- **P-003/P17-E015** [SURFACE] class=`E/SURFACE` anchor=`P-17:L47-L47` (L120): CLI/test trigger: `elai provider failover-test --fixture tests/fixtures/providers/billing_then_ratelimit.json`.
- **P-003/P17-A003** [SURFACE] class=`A/SURFACE` anchor=`P-17:L44-L45` (L132): Exact RuntimePanel/CloudPanel field names and layout for billed-vs-estimate are not specified.

### Surfaces — P-003/P-20 (3)

- **P-003/P20-E011** [SURFACE] class=`E/SURFACE` anchor=`P-20:L43-L44` (L152): frontend_pair N/A: internal context machinery; no user surface beyond existing cost/memory panels (W-6 N/A justified).
- **P-003/P20-E012** [SURFACE] class=`E/SURFACE` anchor=`P-20:L46-L46` (L153): Trigger command: `elai session drive-to-compaction --fixture tests/fixtures/context/long_session.json`.
- **P-003/P20-I001** [SURFACE] class=`I/CFG` anchor=`P-20:L9-L9, L18-L19` (L162): Soft threshold 4000 tokens is openclaw prior art cited in context; whether ELAI Tier-1 adopts that numeric threshold or reuses H-51 thresholds is not locked in REVISED scope.

### Surfaces — P-003/P-22 (2)

- **P-003/P22-E013** [SURFACE] class=`E/SURFACE` anchor=`P-22:L44-L45` (L188): frontend_pair: enrollment + pairing UX on the control surface (Tier-2; W-4).
- **P-003/P22-E014** [SURFACE] class=`E/SURFACE` anchor=`P-22:L47-L47` (L189): Trigger: `elai agents spawn-test --fixture tests/fixtures/agents/instruction_shaped_return.json`.

### Surfaces — P-003/P-23 (11)

- **P-003/P23-E001** [SURFACE] class=`E/STATUS` anchor=`P-23:L1-L6` (L212): P-23 is PLANNED Wave CU-4, COMPUTER-USE-STEAL, lever co-browse-with-receipts, STRONG comparator / MODERATE ELAI anchors, EXTENSION of H-15+E-14, effort M; depends H-13,H-15,E-14,P-13; PROOF_SPINE none (product); owner-commissioned 2026-07-11.
- **P-003/P23-E003** [SURFACE] class=`E/SURFACE` anchor=`P-23:L19-L19, L48-L49` (L214): Tier-1 READ lane: canvas browse node rendering page EXTRACTION — E-14 substrate nav + AXTree + screenshot, read-only, content taint-tagged web at ingest, deixis via H-15; minimal ceremony (P-5).
- **P-003/P23-E004** [SURFACE] class=`E/SURFACE` anchor=`P-23:L20-L20, L48-L49` (L215): Tier-2 CO-BROWSE lane: orchestrator-owned Playwright/CDP browser; CDP screencast frames stream into browse node — THE PANE IS DUMB GLASS.
- **P-003/P23-E013** [SURFACE] class=`E/SURFACE` anchor=`P-23:L40-L40, L51-L51, L78-L79` (L224): CLI: `elai browse <url>` production CLI.
- **P-003/P23-E014** [SURFACE] class=`E/SURFACE` anchor=`P-23:L41-L41, L78-L79` (L225): Front-door router lane table: "pull up / open <url>" resolves to browse lane.
- **P-003/P23-E015** [SURFACE] class=`E/SURFACE` anchor=`P-23:L42-L42, L78-L79` (L226): H-13 canvas dispatcher: browse node view_type + forwarded pane input.
- **P-003/P23-E016** [SURFACE] class=`E/SURFACE` anchor=`P-23:L48-L49` (L227): frontend_pair: canvas browse node screencast (Tier-2) / extraction (Tier-1) + actor-colored input trail so user SEES the journal as it lands.
- **P-003/P23-E021** [SURFACE] class=`E/TEST` anchor=`P-23:L61-L66` (L232): VERIFY via real `elai browse`: plan_gate_check + wiring_contract_check + production_caller_check on cobrowse modules.
- **P-003/P23-E022** [SURFACE] class=`E/SEC` anchor=`P-23:L67-L67` (L233): REDTEAM failed: hostile fake ELAI approval dialog inside pane (UI redress) — harness chrome never in screencast; pane click only becomes cobrowse_input, never harness command. Failed: page text instructing destructive click — G-1 taint blocks untrusted->destructive sink. Succeeded unpatched then patched: user types secret into phishing form — egress passes P-6 + domain-first-seen warning BEFORE dispatch.
- **P-003/P23-E025** [SURFACE] class=`E/INT` anchor=`P-23:L78-L79` (L236): Vertical slice: CoBrowseLane/InputProxy wired into ALL THREE of CLI, front-door, H-13 canvas in this changeset.
- **P-003/P23-E026** [SURFACE] class=`E/LIMIT` anchor=`P-23:L47-L47, L76-L76` (L237): Production: NO remote DOM inside trusted dashboard webview; NO `.unwrap()`/`.expect()`; blake3 for frame hashes; anti-slopsquat on browser-driver crate; bun-not-node if capture sidecar is JS.

### Surfaces — P-003/P-25 (6)

- **P-003/P25-E005** [SURFACE] class=`E/DATA` anchor=`P-25:L32-L36, L26-L26` (L256): K=30 paired real ELAI improvement fixtures; no synthetic tier; B-CREATIVITY-1 corpus design reused under own preregistration (prereg 2026-07-12 §6.1).
- **P-003/P25-E037** [SURFACE] class=`E/SURFACE` anchor=`P-25:L98-L100, P-25:L356-L357` (L288): M4 CLI: `python3 .claude/tools/exo_injection_ab.py run-smoke --repo-root --run-config --expected-run-config-sha256 --out`; selects lowest selection_slot terminal-ELIGIBLE bounded_simulation; eight-call sequence; publishes P25SmokeReportV1 only (no K=30 decision).
- **P-003/P25-E038** [SURFACE] class=`E/SURFACE` anchor=`P-25:L102-L104, P-25:L357-L357` (L289): M5 CLI: `python3 .claude/tools/exo_injection_ab.py run-scored` same trust inputs over full K=30; writes family verdict and closes out.
- **P-003/P25-E039** [SURFACE] class=`E/SURFACE` anchor=`P-25:L358-L359, P-25:L514-L514` (L290): Independent checker: `python3 .claude/tools/exo_creativity_artifact_check.py --verify-p25-smoke` and `--verify-p25-all` with same three trust inputs; PASS = exit 0 and 0 violations.
- **P-003/P25-E040** [SURFACE] class=`E/SURFACE` anchor=`P-25:L341-L348` (L291): Curator CLIs under exo_injection_ab.py: curate-m0-file-set, curate-m1-artifact, curate-m1-anchor, lock-m1, curate-m3-payload, curate-m3-freeze, curate-m3-anchor, lock-m3 (each with exact flags).
- **P-003/P25-I002** [SURFACE] class=`I/SURFACE` anchor=`P-25:L64-L65, L341-L359` (L307): Curator/run/checker CLIs are the only operator-facing surfaces; no dashboard panel, drawer, or notification is in scope.

### Surfaces — P-003/CROSS (3)

- **P-003/IX-012** [SURFACE] class=`IX/SURFACE` anchor=`P23-E013–P23-E015, P23-E025` (L331): P-23 front-door "pull up X" and `elai browse` are co-equal production entrypoints with canvas browse node.
- **P-003/IX-016** [SURFACE] class=`IX/SURFACE` anchor=`P15-E008; P17-E013; P20-E010; P22-E012; P23-E025; P25-E037–P25-E039` (L335): Across P-15/P-17/P-20/P-22/P-23 product units, production callers must be non-test live paths (R7); P-25 production callers are the real CLI entrypoints, not unit tests.
- **P-003/IX-018** [SURFACE] class=`IX/SURFACE` anchor=`P3-E009, P3-E012; P22-E013` (L337): P-3 marketplace/friend sharing (Tier-2) and P-22 control-surface pairing UX (Tier-2) are distinct distribution vs device-enrollment surfaces that may later share L-12 admission patterns but are not the same product.

### Surfaces — SUMMARY (48)

- **SUMMARY/P-G-E18** [SURFACE] (L77): Computer-use comparators are treated as complementary operator backends, not rivals; a raw-capability winner can be vendored beneath `elai wrap` and ELAI’s taint, journal, and gates.
- **SUMMARY/P1-E18** [SURFACE] (L110): Expose the fleet surface through the `elai fleet` CLI.
- **SUMMARY/P1-E21** [SURFACE] (L113): A red tick creates an N-7 inbox card.
- **SUMMARY/P1-A02** [SURFACE] (L122): The index names `elai fleet` but does not enumerate its subcommands or config schema.
- **SUMMARY/P2-E12** [SURFACE] (L141): Expose `elai fleet wire` and `elai fleet ports`.
- **SUMMARY/P3-E05** [SURFACE] (L165): The bundle includes an RRA receipt.
- **SUMMARY/P3-E12** [SURFACE] (L172): `elai app import` re-verifies a bundle through M-5.
- **SUMMARY/P3-E17** [SURFACE] (L177): Expose `elai app export` and `elai app import`.
- **SUMMARY/P4-E14** [SURFACE] (L210): Expose `elai brain mount` and `elai brain sync`.
- **SUMMARY/P5-E01** [SURFACE] (L230): Add a non-coder operator surface at the P-1 fleet grain.
- **SUMMARY/P5-E10** [SURFACE] (L239): The one-line receipt must index/resolve to the full journal.
- **SUMMARY/P5-E11** [SURFACE] (L240): Render a red fleet tick as one sentence plus one gated repair button through N-7.
- **SUMMARY/P5-E13** [SURFACE] (L242): Add orchestrator `CeremonyProfile`, K-5 gate-mode wiring, and one-line receipt rendering.
- **SUMMARY/P5-E14** [SURFACE] (L243): Expose `elai profile set` and `elai profile tax`.
- **SUMMARY/P5-E18** [SURFACE] (L247): `elai profile tax --tier toy` is the named tax measurement entry point.
- **SUMMARY/P6-E13** [SURFACE] (L275): Expose `elai privacy grant`, `revoke`, and `show`.
- **SUMMARY/P7-E18** [SURFACE] (L312): Expose `elai profile show`, `tighten`, and `loosen`.
- **SUMMARY/P8-E16** [SURFACE] (L346): Expose `elai world join`, `merge`, and `whodid`.
- **SUMMARY/P9-A01** [SURFACE] (L366): No P-9 fields, acceptance rows, CLI, status, or complete behavior are present in the summary.
- **SUMMARY/P12-E05** [SURFACE] (L384): Supply ELAI fail-closed gates because the comparator’s in-loop safety is only a commented-out TODO.
- **SUMMARY/P12-E24** [SURFACE] (L403): Expose `elai operator run`.
- **SUMMARY/P12-E26** [SURFACE] (L405): Pair operator-action rows with a screen-recording thumbnail in an existing drawer.
- **SUMMARY/P12-A03** [SURFACE] (L420): The existing frontend drawer is deliberately left to an edit-time W-4 decision.
- **SUMMARY/P13-E20** [SURFACE] (L448): Add no CLI; expose perception as a service seam for E-14 and P-12.
- **SUMMARY/P13-A01** [SURFACE] (L460): The cited 70.5→93.8% is comparator evidence, not an ELAI acceptance threshold.
- **SUMMARY/P14-E05** [SURFACE] (L474): Bind the script to the exact exploration run that minted it through a G-2 RRA receipt.
- **SUMMARY/P14-E14** [SURFACE] (L483): Expose `elai e2e crystallize`.
- **SUMMARY/P15-E13** [SURFACE] (L518): Receipts choose first; narrative only breaks receipt-level ties.
- **SUMMARY/P16-E11** [SURFACE] (L551): Never use GUI driving as the install mechanism when a headless CLI exists.
- **SUMMARY/P16-E15** [SURFACE] (L555): Expose `elai connector install <tool>`.
- **SUMMARY/P16-E21** [SURFACE] (L561): GUI-driving an install despite a headless CLI fails acceptance.
- **SUMMARY/P23-E26** [SURFACE] (L601): Expose `elai browse <url>` and a front-door browse lane.
- **SUMMARY/P23-E27** [SURFACE] (L602): Wire the CLI, front door, and H-13 canvas dispatcher in the same changeset.
- **SUMMARY/P24-E35** [SURFACE] (L660): Expose `elai e2e run --device` and `elai operator run --backend emulator`.
- **SUMMARY/P17-E21** [SURFACE] (L706): Render failover and a billed-versus-estimate label in the RuntimePanel/CloudPanel usage surface.
- **SUMMARY/P18-E27** [SURFACE] (L752): Expose `elai workflow resume` and `elai workflow cancel`.
- **SUMMARY/P19-E09** [SURFACE] (L784): Add `elai doctor detect`.
- **SUMMARY/P19-E10** [SURFACE] (L785): Add `elai doctor repair`.
- **SUMMARY/P19-E11** [SURFACE] (L786): Add `elai doctor lint`.
- **SUMMARY/P19-E12** [SURFACE] (L787): Give doctor modes stable exit codes suitable for CI.
- **SUMMARY/P19-E25** [SURFACE] (L800): Implement config LKG/rejected writes/guards plus the doctor command and backup/continuation machinery.
- **SUMMARY/P19-E26** [SURFACE] (L801): Keep doctor as a structured-text CLI/ops surface with no new frontend.
- **SUMMARY/P19-E28** [SURFACE] (L803): `elai doctor lint` returns a stable nonzero exit for that bad state.
- **SUMMARY/P19-E29** [SURFACE] (L804): Doctor emits structured `doctor_finding` rows.
- **SUMMARY/P19-E32** [SURFACE] (L807): A run-to-run-varying doctor exit code fails acceptance.
- **SUMMARY/P20-E25** [SURFACE] (L848): Add no new frontend beyond existing cost/memory panels.
- **SUMMARY/P22-E26** [SURFACE] (L939): Render enrollment and pairing UX on the control surface at Tier 2.
- **SUMMARY/P25-E02** [SURFACE] (L962): Define the corpus as K=30 open-world real ELAI improvement fixtures.

### Curated surface checklist (family P, by product surface)

Reference atoms only; do not invent surfaces absent from the inventories.

- **CLI: elai fleet register|list|tick** — refs: P-002/P1-E015, SUMMARY/P1-E18
- **CLI: elai fleet wire|ports [--unwired]** — refs: P-002/P2-E011, SUMMARY/P2-E12
- **CLI: elai app export|import** — refs: P-003/P3-E005, P-003/P3-E006, SUMMARY/P3-E17
- **CLI: elai brain mount|sync** — refs: P-001/P4-E013
- **CLI: elai profile set|tax** — refs: P-001/P5-E009
- **CLI: elai profile show|tighten|loosen** — refs: P-002/P7-E009
- **CLI: elai privacy grant|revoke|show** — refs: P-002/P6-E010
- **CLI: elai world join|merge|whodid** — refs: P-001/P8-E008, P-001/P8-E011
- **CLI: elai connector new / install** — refs: P-002/P9-E010, P-001/P16-E007
- **CLI: elai workflow from-goal / resume|cancel** — refs: P-002/P11, P-001/P18-E009
- **CLI: elai operator run** — refs: P-001/P12-E008
- **CLI: elai perception scaffold** — refs: P-001/P13-E012
- **CLI: elai case reuse / elai bench case-reuse** — refs: P-003/P15-E010, P-003/P15-E011
- **CLI: elai provider failover-test** — refs: P-003/P17-E015
- **CLI: elai doctor / config ops (P-19)** — refs: P-002/P19
- **CLI: elai session drive-to-compaction** — refs: P-003/P20-E012
- **CLI: elai bench gate-corpus** — refs: P-001/P21-E012
- **CLI: elai agents spawn-test** — refs: P-003/P22-E014
- **UI: N-7 DashboardInbox red-tick cards** — refs: P-002/P1-E016, P-002/P1-E020
- **UI: Fleet operator panel + repair button (Tier-2, P-5)** — refs: P-001/P5-E008, P-001/P5-E010
- **UI: O-4 WORKFLOW tool-port edges (Tier-2, P-2)** — refs: P-002/P2-E012
- **UI: Operator drawer + recording thumbnail (P-12)** — refs: P-001/P12-E010
- **UI: Perception tile / Set-of-Marks chrome (P-13)** — refs: P-001/P13-E011
- **UI: Case reuse-hit on J-96 CanvasMode node (P-15)** — refs: P-003/P15-E009
- **UI: Failover + billed-vs-estimate on RuntimePanel/CloudPanel (P-17)** — refs: P-003/P17-E014
- **UI: O-4 revision/drift rows (P-18)** — refs: P-001/P18-E012
- **UI: Enrollment/pairing control surface (Tier-2, P-22)** — refs: P-003/P22-E013
- **UI: Co-browse dumb glass + InputProxy (P-23)** — refs: P-003/P23
- **Status: fleet_health_tick red/green + red_reasons** — refs: P-002/P1-E018
- **Status: one-line ceremony receipts indexing full journal** — refs: P-001/P5-E013
- **Status: provider_failover_event + cooldown lanes** — refs: P-003/P17-E016
- **Notification: privacy_grant N-7 human line; privacy_read_refusal** — refs: P-002/P6-E011

## Closing audit block

- **Ledgers merged:** 3/3 deep (`DEEP-P-001`, `DEEP-P-002`, `DEEP-P-003`) + summary (`16-P-SUMMARY_INDEX.discovery.md`).
- **Per-ledger atom counts (all distinct record kinds):**
  - P-001: **326** = feature 267 (E=234, I=5, A=23, C=5) + IX 20 + AC 12 + SD 27
  - P-002: **345** = feature 269 (E=220, I=0, A=42, C=7) + IX 22 + AC 16 + SD 38
  - P-003: **294** = feature 225 (E=191, I=7, A=22, C=5) + IX 19 + AC 12 + SD 38
  - SUMMARY: **698** bold inventory IDs (E/I/A mixed under `**ID** —` pattern)
- **Aggregate distinct record count:** **1663**
- **Superseded notes count:** **37**
- **Surface-tagged atoms (deep+IX):** **160**; summary surface-tagged: **48**
- **Zero-omission statement:** Every atomic table row with a distinct ID in the three deep ledgers (feature E/I/A/C), every IX-*, AC-*, and SD-* register row, and every bold inventory ID in the summary discovery file is present above under a namespaced ID with its source anchor (file + line). Registry/footer rows that only restate counts were not double-counted as new feature atoms. No atomic record was compressed into prose or dropped as minor.
- **Duplicate-ID check (P-001 feature atoms):** unique=267/267
- **Duplicate-ID check (P-002 feature atoms):** unique=269/269
- **Duplicate-ID check (P-003 feature atoms):** unique=225/225
- **Duplicate-ID check (SUMMARY):** unique=698/698
- **Output path:** `docs/thefinalglance/feature-discovery/aggregates/P-AGGREGATE.md`

---

*Aggregator: P-family deep feature discovery. Inputs read-only. Single write: this file.*
