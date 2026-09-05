# Family M aggregate — ELAI deep feature discovery

## Document contract

- **Family:** M (commercial shipping / multi-seat; M-0 index + M-1..M-6 sessions; MD-1..MD-5 doctrine; MA-1..MA-7 Section-A pointers; enterprise rider M-7+).
- **Inputs:** `DEEP-M-001` (1 deep ledger) and `13-M-SUMMARY_INDEX.discovery.md`.
- **Policy:** Every distinct atomic record is preserved with its ID and source anchor (ledger file + line). No prose compression. No silent drop of minor atoms.
- **Namespacing:** Deep atoms keep native IDs under ledger prefix `M-001/`. Summary atoms use `SUMMARY/`. Cross-file IX/SD/AC stay namespaced by ledger (e.g. `M-001/IX-003`, `M-001/SD-M1-007`).
- **NEWER-PLAN PRECEDENCE (within family):** An explicit SUPERSEDED/replacement/CORRECT marker in a source beats everything; otherwise a higher plan number is newer and wins the **target state** description. Losing descriptions are retained as superseded notes with anchors. Deep ledgers supersede the summary orientation pass where they conflict.
- **Status language preserved:** BUILD / PLANNED / SHIPPED / DEFERRED / demand-gated are not collapsed.
- **Generated:** 2026-08-12 (M-family aggregator pass)

### Ledgers merged

| Ledger | Path | Source atoms | IX | SD | AC | Subtotal |
|---|---|---:|---:|---:|---:|---:|
| M-001 | `docs/thefinalglance/feature-discovery/deep/DEEP-M-001.discovery.md` | 360 | 24 | 39 | 12 | **435** |
| SUMMARY | `docs/thefinalglance/feature-discovery/13-M-SUMMARY_INDEX.discovery.md` | inv=304 | extra=179 | — | — | **483** |
| **Aggregate distinct records** | — | — | — | — | — | **918** |

### Per-ledger declared vs extracted

- **M-001:** source 360/360; ix 24/24; sd 39/39; ac 12/12 — **exact**
- **SUMMARY:** inventory atoms extracted **304** (summary declares 304 atomic inventory records) — **exact**; plus workflow/invariant/surface/dep/defer/fail/obs/accept/inf/amb/deeper relational rows = **179**

### Family map (session inventory)

| Unit | Deep owner | Wave | Role |
|---|---|---|---|
| M-0 INDEX | M-001 | gate | Commercial shipping doctrine MD-1..MD-5, MA-1..MA-7, GATE M-0, waves, enterprise rider |
| M-1 seat entitlement + purchase rail | M-001 | M1 KEYSTONE | Signed offline entitlement, mint endpoint, tier(), license CLI |
| M-2 org policy pack | M-001 | M2 | File-distributed OrgPolicyPack, drift, org policy CLI |
| M-3 fleet evidence rollup | M-001 | M3 | `elai org report`, K-6 extension, roster honesty |
| M-4 signed release + updater + support | M-001 | M2 | update CLI, support bundle, channel pin |
| M-5 commercial packaging + verifier OSS | M-001 | M1(a,d)/M3(b,c) | elai-verify, legal, tiers.toml, continuity |
| M-6 evidence format EEF v1 | M-001 | M3 | Schema extract, conform, golden lock, publication |
| Summary orientation | SUMMARY | — | Summary-bounded inventory (originals not read in that pass) |

## Notation

- `[E]` / `E/*` explicit · `[I]` inference · `[A]` ambiguity · `[C]` conflict/correction · `[Δ]` summary delta · `[SURFACE]` user-facing · `[superseded]` losing target-state retained · `[SUMMARY]` orientation inventory.
- Domain tags from deep ledgers (STATUS, CAP, SURFACE, FLOW, …) are preserved in the class field.
- Anchors: `M-N:Lx-Ly` = M-plan originals; `13-M:Ln` / `M:Ln` = summary discovery / M-SUMMARY_INDEX lines; ledger-line points at discovery file.

---

## Superseded notes (NEWER-PLAN PRECEDENCE)

Losing target-state descriptions retained. Count: **4**.

- **SUP-001** [superseded]
  - Winner (target state): M-001/M1-C001 + M-001/SD-M1-007 (deep)
  - Loser (retained): SUMMARY/M1-E043 + SUMMARY/M1-E044 (SettingsWindow license-status line)
  - Reason: Deep conflict register and summary delta CORRECT/REMOVE: numbered M-1 does not name SettingsWindow. Target state surfaces are `elai license status` + ExpiredGrace status banner + B-43 install-end activation. Summary SettingsWindow pairing is orientation-only and W-4 deferred speculation.
  - Winner anchor: DEEP-M-001.discovery.md:L165, L522; M-1:L37-L37, L50-L50, L56-L56
  - Loser anchor: 13-M-SUMMARY_INDEX.discovery.md:L133-L134, L441, L508, L589
- **SUP-002** [superseded]
  - Winner (target state): M-001/M0-E004 + M-001/M0-E051 + M-001/SD-M0-005 (M-7+ enterprise rider)
  - Loser (retained): SUMMARY residual text calling enterprise rider M-6+ (13-M:L141)
  - Reason: M-6 was reassigned to evidence-format open standard 2026-07-05; informal M-6+ renumbered to M-7+. Summary header already notes the correction; body L141 still says M-6+. Target state: M-7+.
  - Winner anchor: DEEP-M-001.discovery.md:L38, L84, L509; M-0:L4-L5, L68-L70
  - Loser anchor: 13-M-SUMMARY_INDEX.discovery.md:L141 (vs L2-L4 header correction)
- **SUP-003** [superseded]
  - Winner (target state): M-001/M4-E023 + M-001/SD-M4-002 (ReleaseChannelEvent arms named)
  - Loser (retained): SUMMARY/OBS ReleaseChannelEvent 'No variants specified' (13-M:L549)
  - Reason: Deep M-4 names Pattern-2 arms UpdateAvailable/Applied/RefusedHash/RolledBack (create-vs-reuse still W-9). Target state: arms known; prefer-reuse still open (AC-002).
  - Winner anchor: DEEP-M-001.discovery.md:L306, L549; M-4:L33-L34, L56-L57
  - Loser anchor: 13-M-SUMMARY_INDEX.discovery.md:L273-L274, L549
- **SUP-004** [superseded]
  - Winner (target state): Deep M-plan originals (DEEP-M-001 extraction) for all implement-level detail
  - Loser (retained): SUMMARY as sole authority for field schemas, CLI argument forms, fixture paths, module paths
  - Reason: Summary original-plan policy: originals inventoried but not read (13-M:L10). Deep ledger read all 7 originals completely. Where summary compresses and deep expands, deep description is target state; summary remains orientation inventory under SUMMARY/*.
  - Winner anchor: DEEP-M-001.discovery.md:L1-L11, L576-L585
  - Loser anchor: 13-M-SUMMARY_INDEX.discovery.md:L10-L11, L606-L620

---

## User-facing surfaces

Every window, panel, command, CLI verb, flag, status display, notification, and published commercial/docs surface of family M. Full text lives in ledger sections; this subsection indexes by namespaced ID for dashboard DESIGN.md feed.

### Deep SURFACE-class and command surfaces (reference by ID)

Deep surface-tagged / command-surface atoms: **73**.

- **M-001/M0-E042** [SURFACE-RELATED] class=`E/STATUS` anchor=`M-0:L54-L54` (DEEP-M-001.discovery.md:L75): M-3 is fleet evidence rollup (`elai org report`), WAVE M3, depends M-1, MA-3, MA-6.
- **M-001/M0-E048** [SURFACE-RELATED] class=`E/STATUS` anchor=`M-0:L63-L63` (DEEP-M-001.discovery.md:L81): WAVE M1: M-1; M-5a (verifier OSS extraction) + M-5d (tiers.toml).
- **M-001/M1-E010** [SURFACE-RELATED] class=`E/CFG` anchor=`M-1:L18-L18` (DEEP-M-001.discovery.md:L115): Tiers are DATA (M-5d tiers.toml).
- **M-001/M1-E022** [SURFACE] class=`E/SURFACE` anchor=`M-1:L37-L37` (DEEP-M-001.discovery.md:L127): CLI verbs: `elai license activate <file|key>`, `elai license status`, `elai license deactivate`.
- **M-001/M1-E029** [SURFACE-RELATED] class=`E/CFG` anchor=`M-1:L42-L43` (DEEP-M-001.discovery.md:L134): Free tier = everything shipped through L that is not explicitly Team-tier in tiers.toml (M-5d).
- **M-001/M1-E036** [SURFACE-RELATED] class=`E/STATE` anchor=`M-1:L49-L50` (DEEP-M-001.discovery.md:L141): Expiry -> 14-day grace (ExpiredGrace rows + status banner) -> DegradedToFree.
- **M-001/M1-E037** [SURFACE] class=`E/SURFACE` anchor=`M-1:L50-L50` (DEEP-M-001.discovery.md:L142): ExpiredGrace emits a status banner (user-visible).
- **M-001/M1-E039** [SURFACE-RELATED] class=`E/DEP` anchor=`M-1:L53-L56` (DEEP-M-001.discovery.md:L144): COMPOSES-WITH: G-8 attestation signing/verify core; K-24 credit ledger + seats.toml; ClockProvider (all expiry math — no SystemTime); K-1 claims lint (any UI copy about tiers); B-43 installer (activation offered at install-end, one line).
- **M-001/M1-E040** [SURFACE] class=`E/SURFACE` anchor=`M-1:L56-L56` (DEEP-M-001.discovery.md:L145): B-43 installer offers activation at install-end (one line).
- **M-001/M1-E042** [SURFACE-RELATED] class=`E/INT` anchor=`M-1:L61-L61` (DEEP-M-001.discovery.md:L147): WIRING consumes: G-8 signing core, K-24 grant path, tiers.toml (M-5d), ClockProvider.
- **M-001/M1-E043** [SURFACE-RELATED] class=`E/INT` anchor=`M-1:L62-L62` (DEEP-M-001.discovery.md:L148): reachable_from: `elai license *` CLI (real entrypoint); tier() read on the K-24 grant path.
- **M-001/M1-E044** [SURFACE-RELATED] class=`E/TEST` anchor=`M-1:L63-L63` (DEEP-M-001.discovery.md:L149): trigger: `elai license activate fixtures/team-entitlement.signed && elai license status`.
- **M-001/M1-A002** [SURFACE] class=`A/SURFACE` anchor=`M-1:L50-L50` (DEEP-M-001.discovery.md:L160): "status banner" for ExpiredGrace is named without panel/window placement, copy, or dismiss rules.
- **M-001/M1-A003** [SURFACE-RELATED] class=`A/CFG` anchor=`M-1:L18-L18, L42-L43` (DEEP-M-001.discovery.md:L161): Exact Free vs Pro vs Team feature-key inventory is deferred to M-5d tiers.toml content.
- **M-001/M1-C001** [SURFACE] class=`C/SURFACE` anchor=`M-1:L37-L37, L50-L50, L56-L56` (DEEP-M-001.discovery.md:L164): Summary discovery asserts a SettingsWindow license-status line (W-4 deferred). Numbered M-1 names `elai license status` + ExpiredGrace status banner and B-43 install-end activation; it does not name SettingsWindow.
- **M-001/M2-E015** [SURFACE-RELATED] class=`E/OBS` anchor=`M-2:L29-L30` (DEEP-M-001.discovery.md:L188): LOCK-4 drift is DETECTED + surfaced, not auto-corrected: boot compares applied-pack hash vs the org-pinned hash -> OrgPolicyEvent(Drift) + boot warning + M-3 rollup visibility.
- **M-001/M2-E016** [SURFACE] class=`E/SURFACE` anchor=`M-2:L29-L30` (DEEP-M-001.discovery.md:L189): Drift surfaces as a boot warning.
- **M-001/M2-E018** [SURFACE-RELATED] class=`E/CAP` anchor=`M-2:L34-L34` (DEEP-M-001.discovery.md:L191): Task 1: pack format + signer/verifier + `elai org policy init|sign|verify|show` (admin authoring side).
- **M-001/M2-E027** [SURFACE-RELATED] class=`E/DATA` anchor=`M-2:L50-L52` (DEEP-M-001.discovery.md:L200): WIRING produces: commerce/org_policy.rs (pack type, verify/apply, weaken gate), `elai org policy *` CLI, OrgPolicyEvent SSB (+1).
- **M-001/M2-E029** [SURFACE-RELATED] class=`E/INT` anchor=`M-2:L54-L54` (DEEP-M-001.discovery.md:L202): reachable_from: boot path (pack discovery) + `elai org policy *` CLI.
- **M-001/M2-A003** [SURFACE] class=`A/SURFACE` anchor=`M-2:L29-L30` (DEEP-M-001.discovery.md:L216): Boot warning presentation (CLI only vs UI badge) is not specified.
- **M-001/M3-E019** [SURFACE] class=`E/SURFACE` anchor=`M-3:L37-L37` (DEEP-M-001.discovery.md:L245): CLI arm: `elai export compliance --org`.
- **M-001/M3-E020** [SURFACE] class=`E/SURFACE` anchor=`M-3:L38-L38` (DEEP-M-001.discovery.md:L246): Task 2 aggregator: `elai org report <dir-of-seat-exports>` -> one org bundle.
- **M-001/M3-E022** [SURFACE-RELATED] class=`E/SEC` anchor=`M-3:L39-L40` (DEEP-M-001.discovery.md:L248): Offline-verifiable end-to-end (`elai org report --verify` re-checks every constituent chain).
- **M-001/M3-E023** [SURFACE] class=`E/SURFACE` anchor=`M-3:L39-L40` (DEEP-M-001.discovery.md:L249): Verify flag form: `elai org report --verify`.
- **M-001/M3-E026** [SURFACE] class=`E/SURFACE` anchor=`M-3:L43-L44` (DEEP-M-001.discovery.md:L252): Task 3 render: one human sheet (markdown) + one machine file (msgpack/toml) — the K-7 brief's org answer, generated not hand-written.
- **M-001/M3-E029** [SURFACE-RELATED] class=`E/DATA` anchor=`M-3:L53-L55` (DEEP-M-001.discovery.md:L255): WIRING produces: org_report.rs (aggregate + verify + render), `elai org report` CLI, seat-export field extension; +0..1 SSB.
- **M-001/M3-E031** [SURFACE-RELATED] class=`E/INT` anchor=`M-3:L57-L57` (DEEP-M-001.discovery.md:L257): reachable_from: `elai org report` (real entrypoint); `elai export compliance --org` on seats.
- **M-001/M3-E032** [SURFACE-RELATED] class=`E/TEST` anchor=`M-3:L58-L59` (DEEP-M-001.discovery.md:L258): trigger: 3 fixture seat exports (one clean, one with Drift, one with a broken chain) -> `elai org report fixtures/exports --verify`.
- **M-001/M3-I001** [SURFACE] class=`I/SURFACE` anchor=`M-3:L38-L44` (DEEP-M-001.discovery.md:L268): Human markdown sheet and machine msgpack/toml are dual renderings of one org bundle rather than two independent aggregation paths.
- **M-001/M3-A002** [SURFACE-RELATED] class=`A/FLOW` anchor=`M-3:L38-L40, L58-L59` (DEEP-M-001.discovery.md:L270): Behavior of `elai org report` without `--verify` is not specified.
- **M-001/M4-E016** [SURFACE] class=`E/SURFACE` anchor=`M-4:L26-L26` (DEEP-M-001.discovery.md:L298): `elai update rollback` restores it and verifies the hash before exec.
- **M-001/M4-E022** [SURFACE] class=`E/SURFACE` anchor=`M-4:L33-L33` (DEEP-M-001.discovery.md:L304): CLI verbs: `elai update check|apply|rollback|channel`.
- **M-001/M4-E028** [SURFACE] class=`E/SURFACE` anchor=`M-4:L39-L40` (DEEP-M-001.discovery.md:L310): Task 4 support bundle: `elai support bundle` -> `workflow/support/bundle-{date}.tar.zst`, G-9-redacted.
- **M-001/M4-E029** [SURFACE] class=`E/SURFACE` anchor=`M-4:L40-L40` (DEEP-M-001.discovery.md:L311): Support bundle prints a content summary so the user sees what it holds BEFORE sharing.
- **M-001/M4-E031** [SURFACE-RELATED] class=`E/DATA` anchor=`M-4:L48-L50` (DEEP-M-001.discovery.md:L313): WIRING produces: update.rs (check/apply/rollback/channel), release-manifest format + ops signing runbook, `elai support bundle`; +0..1 SSB.
- **M-001/M4-E033** [SURFACE-RELATED] class=`E/INT` anchor=`M-4:L52-L53` (DEEP-M-001.discovery.md:L315): reachable_from: `elai update *` + `elai support bundle` CLI (real entrypoints); boot-time non-blocking check.
- **M-001/M4-E034** [SURFACE-RELATED] class=`E/TEST` anchor=`M-4:L54-L55` (DEEP-M-001.discovery.md:L316): trigger: fixture release server dir with v(N+1) manifest (one artifact's blake3 deliberately wrong) -> `elai update check && elai update apply`.
- **M-001/M4-E035** [SURFACE-RELATED] class=`E/TEST` anchor=`M-4:L56-L57` (DEEP-M-001.discovery.md:L317): expect_ssb: UpdateAvailable row; RefusedHash row on the bad artifact; after fixing the fixture, Applied row; `elai update rollback` -> RolledBack + prior binary hash-verified.
- **M-001/M4-E039** [SURFACE-RELATED] class=`E/TEST` anchor=`M-4:L60-L62` (DEEP-M-001.discovery.md:L321): `elai support bundle` on a fixture session with a seeded AWS key + email produces a bundle whose secret-grep is EMPTY (G-9 arm) and prints its content summary.
- **M-001/M4-A003** [SURFACE] class=`A/SURFACE` anchor=`M-4:L64-L66` (DEEP-M-001.discovery.md:L328): UX wording for Windows restart-activation is required but not drafted.
- **M-001/M5-E009** [SURFACE-RELATED] class=`E/CAP` anchor=`M-5:L14-L15` (DEEP-M-001.discovery.md:L347): Slice (a) VERIFIER OSS EXTRACTION [WAVE M1 — the one build]: extract standalone `elai-verify` crate/binary.
- **M-001/M5-E010** [SURFACE-RELATED] class=`E/INT` anchor=`M-5:L15-L16` (DEEP-M-001.discovery.md:L348): elai-verify wraps: verify-attestation (G-8) + `rra verify` action-stream re-check (G-2) + the K-6 bundle conformance check + M-3 org-bundle verify.
- **M-001/M5-E020** [SURFACE] class=`E/SURFACE` anchor=`M-5:L28-L29` (DEEP-M-001.discovery.md:L358): Support-expectation page (what a seat gets; honest solo-operation posture, GD-5 — no fake SLA).
- **M-001/M5-E021** [SURFACE-RELATED] class=`E/CFG` anchor=`M-5:L30-L31` (DEEP-M-001.discovery.md:L359): Slice (d) TIER MATRIX AS DATA [WAVE M1]: tiers.toml Free / Pro / Team rows -> feature keys M-1's tier() gates + K-24's credit grants read.
- **M-001/M5-E029** [SURFACE-RELATED] class=`E/DEP` anchor=`M-5:L45-L48` (DEEP-M-001.discovery.md:L367): COMPOSES-WITH: G-8 attestation verify, G-2 rra verify, K-6 conformance, M-3 org verify (the four legs the OSS binary wraps); K-1 dossier + claims lint; K-7 brief (gains the verifier-OSS line); M-1 (tiers.toml consumer); GTM memory decisions (executed, not re-litigated).
- **M-001/M5-E030** [SURFACE-RELATED] class=`E/DATA` anchor=`M-5:L50-L52` (DEEP-M-001.discovery.md:L368): WIRING produces: elai-verify crate/binary (published), EULA/DPA/continuity drafts under docs/strategy/commercial/, tiers.toml, claims-lint pricing recognizer extension.
- **M-001/M5-E032** [SURFACE-RELATED] class=`E/INT` anchor=`M-5:L54-L55` (DEEP-M-001.discovery.md:L370): reachable_from: `elai-verify` standalone binary (real entrypoint, engine-free); tiers.toml read by `elai license status`.
- **M-001/M5-E033** [SURFACE] class=`E/SURFACE` anchor=`M-5:L54-L55, L62-L63` (DEEP-M-001.discovery.md:L371): `elai license status` renders tier names/features from tiers.toml.
- **M-001/M5-E034** [SURFACE-RELATED] class=`E/TEST` anchor=`M-5:L56-L57` (DEEP-M-001.discovery.md:L372): trigger: on a CLEAN machine (no engine): `elai-verify attestation fixtures/run.attestation && elai-verify rra fixtures/run.rra`.
- **M-001/M5-E039** [SURFACE-RELATED] class=`E/TEST` anchor=`M-5:L62-L63` (DEEP-M-001.discovery.md:L377): `elai license status` renders tier names/features from tiers.toml (fixture, M-1 join).
- **M-001/M5-E042** [SURFACE-RELATED] class=`E/CLAIM` anchor=`M-5:L71-L72` (DEEP-M-001.discovery.md:L380): Risk R3 tiers.toml drift vs marketing copy: the lint's pricing recognizer greps rendered copy against tiers.toml values — one source of pricing truth, mechanically enforced.
- **M-001/M5-I001** [SURFACE] class=`I/SURFACE` anchor=`M-5:L28-L29, L50-L52` (DEEP-M-001.discovery.md:L381): Support-expectation page is a published commercial documentation surface rather than a runtime panel; no CLI verb is named for it.
- **M-001/M5-A003** [SURFACE-RELATED] class=`A/CFG` anchor=`M-5:L30-L33` (DEEP-M-001.discovery.md:L384): Exact Free/Pro/Team feature-key matrix and numeric prices are data in tiers.toml and not fixed in this plan.
- **M-001/M5-A004** [SURFACE] class=`A/SURFACE` anchor=`M-5:L15-L16, L56-L60` (DEEP-M-001.discovery.md:L385): Subcommand grammar for org-bundle verify under elai-verify is implied by the four-leg wrap but not typed as a verb list beyond attestation and rra triggers.
- **M-001/M6-E002** [SURFACE-RELATED] class=`E/DEP` anchor=`M-6:L5-L6` (DEEP-M-001.discovery.md:L396): Depends on M-5a (elai-verify extracted), G-2/G-8/K-6 emit paths stable.
- **M-001/M6-E007** [SURFACE-RELATED] class=`E/CAP` anchor=`M-6:L14-L15` (DEEP-M-001.discovery.md:L401): Third-party tools can EMIT ELAI-verifiable evidence and elai-verify becomes the reference verifier of a format rather than the private tool of one engine.
- **M-001/M6-E017** [SURFACE-RELATED] class=`E/COMP` anchor=`M-6:L29-L29` (DEEP-M-001.discovery.md:L411): Slice (b) PROSE SPEC + CONFORMANCE FIXTURES at docs/strategy/eef/ (or a sibling repo at publication).
- **M-001/M6-E019** [SURFACE-RELATED] class=`E/TEST` anchor=`M-6:L31-L32` (DEEP-M-001.discovery.md:L413): Conformance fixture pack: valid bundles + one-byte-tampered + missing-field negatives.
- **M-001/M6-E020** [SURFACE] class=`E/SURFACE` anchor=`M-6:L32-L33` (DEEP-M-001.discovery.md:L414): `elai-verify conform <artifact>` (an M-5 binary subcommand extension) validates any artifact against a pinned spec version.
- **M-001/M6-E025** [SURFACE-RELATED] class=`E/COMP` anchor=`M-6:L38-L39` (DEEP-M-001.discovery.md:L419): Slice (d) PUBLICATION + THIRD-PARTY EMITTER STORY: publish spec + schemas + fixtures alongside the elai-verify repo (same license family as M-5a; user picks).
- **M-001/M6-E026** [SURFACE-RELATED] class=`E/COMP` anchor=`M-6:L39-L40` (DEEP-M-001.discovery.md:L420): Short "emit EEF from your tool" doc with one worked example (a minimal third-party emitter producing a conformant verdict row).
- **M-001/M6-E033** [SURFACE-RELATED] class=`E/FAIL` anchor=`M-6:L54-L56` (DEEP-M-001.discovery.md:L427): LOCK-4 FAIL-CLOSED SEMANTICS ARE NORMATIVE: the spec text makes R1 (missing evidence = FAIL, UNVOUCHED never renders clean) a conformance requirement for verifiers, not a recommendation — a permissive third-party verifier is non-conformant by definition.
- **M-001/M6-E034** [SURFACE-RELATED] class=`E/DEP` anchor=`M-6:L58-L62` (DEEP-M-001.discovery.md:L428): COMPOSES-WITH: M-5a elai-verify (gains `conform`; the reference verifier), G-2 RRA + G-8 attestation + K-6 bundle (the emit paths — the schemas describe them, never fork them), E-45 golden-lock pattern (schema evolution), K-1 claims lint (adoption-claim recognizer extension), K-3/K-11 (the verdict/receipt shapes rendered into PRs), gate-manifest R8 (the golden lock joins the frozen set).
- **M-001/M6-E035** [SURFACE-RELATED] class=`E/DATA` anchor=`M-6:L64-L67` (DEEP-M-001.discovery.md:L429): WIRING produces: docs/strategy/eef/ spec v1 (prose + derived schemas + conformance fixtures), the `elai-verify conform` subcommand, the schema golden-lock file (added to .claude/gate-manifest.txt), the K-1 lint adoption-claim recognizer, the third-party emitter example.
- **M-001/M6-E037** [SURFACE-RELATED] class=`E/INT` anchor=`M-6:L70-L71` (DEEP-M-001.discovery.md:L431): reachable_from: `elai-verify conform <artifact>` on an engine-free machine (real entrypoint); the golden lock enforced by gate_freeze_check.py on every run (R8 path).
- **M-001/M6-E038** [SURFACE-RELATED] class=`E/TEST` anchor=`M-6:L72-L73` (DEEP-M-001.discovery.md:L432): trigger: on a CLEAN machine: `elai-verify conform fixtures/run.rra && elai-verify conform fixtures/tampered.rra; test $? -ne 0`.
- **M-001/M6-E040** [SURFACE-RELATED] class=`E/TEST` anchor=`M-6:L75-L76` (DEEP-M-001.discovery.md:L434): A conformant fixture PASSes and a one-byte-tampered / missing-field fixture FAILS on an engine-free machine (R2).
- **M-001/M6-E044** [SURFACE-RELATED] class=`E/TEST` anchor=`M-6:L79-L79` (DEEP-M-001.discovery.md:L438): The worked third-party example emits a row `elai-verify conform` accepts.
- **M-001/M6-E045** [SURFACE-RELATED] class=`E/FAIL` anchor=`M-6:L81-L84` (DEEP-M-001.discovery.md:L439): Risk R1 premature standardization: freezing v1 before the RRA/attestation surfaces stabilize forces a fast v2 and burns credibility. Mitigation: M-6 sits in WAVE M3 behind GATE M-0; v1 scope = only artifacts elai-verify already verifies today.
- **M-001/M6-E046** [SURFACE-RELATED] class=`E/PRIV` anchor=`M-6:L85-L86` (DEEP-M-001.discovery.md:L440): Risk R2 the schema leaks engine detail: review pass against LOCK-3 with the G-9 redaction spec; the conformance fixtures are generated from redacted runs only.
- **M-001/M6-E047** [SURFACE-RELATED] class=`E/CLAIM` anchor=`M-6:L87-L89` (DEEP-M-001.discovery.md:L441): Risk R3 nobody adopts (the honest default): the format still pays for itself as internal rigor (the golden lock + conform subcommand harden ELAI's own emit paths); adoption claims stay gated so non-adoption never becomes a false claim.
- **M-001/M6-I001** [SURFACE-RELATED] class=`I/DATA` anchor=`M-6:L32-L36` (DEEP-M-001.discovery.md:L442): Because every emitted artifact carries spec_version and conform pins a version, multi-version coexistence is implied even though negotiation protocol is not defined.

### Summary CLI / UI / ACTION surfaces (reference by ID)

Summary CLI/UI/ACTION (and closely related commercial surfaces): **44**.

- **SUMMARY/M1-E011** [CLI] [SURFACE] anchor=`M:L28` (13-M-SUMMARY_INDEX.discovery.md:L101): The CLI exposes 'elai license activate'.
- **SUMMARY/M1-E012** [CLI] [SURFACE] anchor=`M:L28` (13-M-SUMMARY_INDEX.discovery.md:L102): The CLI exposes 'elai license status'.
- **SUMMARY/M1-E013** [CLI] [SURFACE] anchor=`M:L28` (13-M-SUMMARY_INDEX.discovery.md:L103): The CLI exposes 'elai license deactivate'.
- **SUMMARY/M1-E021** [ACTION] [SURFACE] anchor=`M:L30-L31` (13-M-SUMMARY_INDEX.discovery.md:L111): The mint endpoint can mint an entitlement.
- **SUMMARY/M1-E022** [ACTION] [SURFACE] anchor=`M:L30-L31` (13-M-SUMMARY_INDEX.discovery.md:L112): The mint endpoint can renew an entitlement.
- **SUMMARY/M1-E023** [ACTION] [SURFACE] anchor=`M:L30-L31` (13-M-SUMMARY_INDEX.discovery.md:L113): The mint endpoint can revoke an entitlement.
- **SUMMARY/M1-E034** [UI] [SURFACE] anchor=`M:L34-L35` (13-M-SUMMARY_INDEX.discovery.md:L124): B-43 offers activation at the end of installation.
- **SUMMARY/M1-E043** [UI] [SURFACE] anchor=`M:L37-L38` (13-M-SUMMARY_INDEX.discovery.md:L133): A license-status line is paired with SettingsWindow.
- **SUMMARY/M2-E016** [UI] [SURFACE] anchor=`M:L48-L49` (13-M-SUMMARY_INDEX.discovery.md:L163): Policy drift is surfaced.
- **SUMMARY/M2-E024** [CLI] [SURFACE] anchor=`M:L53` (13-M-SUMMARY_INDEX.discovery.md:L171): The CLI exposes 'elai org policy init'.
- **SUMMARY/M2-E025** [CLI] [SURFACE] anchor=`M:L53` (13-M-SUMMARY_INDEX.discovery.md:L172): The CLI exposes 'elai org policy sign'.
- **SUMMARY/M2-E026** [CLI] [SURFACE] anchor=`M:L53` (13-M-SUMMARY_INDEX.discovery.md:L173): The CLI exposes 'elai org policy verify'.
- **SUMMARY/M2-E027** [CLI] [SURFACE] anchor=`M:L53` (13-M-SUMMARY_INDEX.discovery.md:L174): The CLI exposes 'elai org policy show'.
- **SUMMARY/M3-E005** [CLI] [SURFACE] anchor=`M:L61` (13-M-SUMMARY_INDEX.discovery.md:L195): Each seat exports with 'elai export compliance --org'.
- **SUMMARY/M3-E012** [CLI] [SURFACE] anchor=`M:L62` (13-M-SUMMARY_INDEX.discovery.md:L202): An admin aggregates with 'elai org report <dir> --verify'.
- **SUMMARY/M4-E013** [CLI] [SURFACE] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L248): The CLI exposes 'elai update check'.
- **SUMMARY/M4-E014** [CLI] [SURFACE] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L249): The CLI exposes 'elai update apply'.
- **SUMMARY/M4-E015** [CLI] [SURFACE] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L250): The CLI exposes 'elai update rollback'.
- **SUMMARY/M4-E016** [CLI] [SURFACE] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L251): The CLI exposes 'elai update channel'.
- **SUMMARY/M4-E023** [CLI] [SURFACE] anchor=`M:L81` (13-M-SUMMARY_INDEX.discovery.md:L258): The CLI exposes 'elai support bundle'.
- **SUMMARY/M4-E031** [UI] [SURFACE] anchor=`M:L82` (13-M-SUMMARY_INDEX.discovery.md:L266): Support-bundle creation prints a summary.
- **SUMMARY/M5-E003** [CAP] [SURFACE] anchor=`M:L94-L95` (13-M-SUMMARY_INDEX.discovery.md:L287): M-5 publishes a standalone open-source elai-verify crate and binary.
- **SUMMARY/M5-E004** [INT] [SURFACE] anchor=`M:L94-L95` (13-M-SUMMARY_INDEX.discovery.md:L288): elai-verify wraps the same G-8 attestation verification path.
- **SUMMARY/M5-E005** [INT] [SURFACE] anchor=`M:L94-L95` (13-M-SUMMARY_INDEX.discovery.md:L289): elai-verify wraps the same G-2 RRA verification path.
- **SUMMARY/M5-E006** [INT] [SURFACE] anchor=`M:L94-L95` (13-M-SUMMARY_INDEX.discovery.md:L290): elai-verify wraps the same K-6 conformance verification path.
- **SUMMARY/M5-E007** [INT] [SURFACE] anchor=`M:L95` (13-M-SUMMARY_INDEX.discovery.md:L291): elai-verify wraps the same M-3 organization-bundle verification path.
- **SUMMARY/M5-E008** [SAFE] [SURFACE] anchor=`M:L95-L96` (13-M-SUMMARY_INDEX.discovery.md:L292): elai-verify has zero engine dependencies.
- **SUMMARY/M5-E011** [COMP] [SURFACE] anchor=`M:L97` (13-M-SUMMARY_INDEX.discovery.md:L295): Commercial packaging includes an EULA draft.
- **SUMMARY/M5-E013** [COMP] [SURFACE] anchor=`M:L97` (13-M-SUMMARY_INDEX.discovery.md:L297): Commercial packaging includes a DPA draft.
- **SUMMARY/M5-E018** [COMP] [SURFACE] anchor=`M:L98` (13-M-SUMMARY_INDEX.discovery.md:L302): The trust-continuity statement offers escrow on lapse.
- **SUMMARY/M5-E019** [COMP] [SURFACE] anchor=`M:L98-L99` (13-M-SUMMARY_INDEX.discovery.md:L303): The trust-continuity statement alternatively offers conversion to OSS on abandonment.
- **SUMMARY/M5-E035** [INT] [SURFACE] anchor=`M:L105` (13-M-SUMMARY_INDEX.discovery.md:L319): The published integration artifact is the elai-verify crate.
- **SUMMARY/M5-E040** [TEST] [SURFACE] anchor=`M:L107-L108` (13-M-SUMMARY_INDEX.discovery.md:L324): On a clean, engine-free machine, elai-verify must verify a fixture attestation.
- **SUMMARY/M5-E041** [TEST] [SURFACE] anchor=`M:L107-L108` (13-M-SUMMARY_INDEX.discovery.md:L325): On a clean, engine-free machine, elai-verify must verify a fixture RRA.
- **SUMMARY/M5-E042** [TEST] [SURFACE] anchor=`M:L107-L108` (13-M-SUMMARY_INDEX.discovery.md:L326): On a clean, engine-free machine, elai-verify must verify a fixture organization bundle.
- **SUMMARY/M5-E043** [TEST] [SURFACE] anchor=`M:L108` (13-M-SUMMARY_INDEX.discovery.md:L327): elai-verify must catch a one-byte artifact tamper under R2.
- **SUMMARY/M5-E046** [TEST] [SURFACE] anchor=`M:L109-L110` (13-M-SUMMARY_INDEX.discovery.md:L330): 'elai license status' must render from tiers.toml.
- **SUMMARY/M6-E011** [INT] [SURFACE] anchor=`M:L118-L119` (13-M-SUMMARY_INDEX.discovery.md:L346): Third-party tools can emit evidence that ELAI can verify.
- **SUMMARY/M6-E012** [CAP] [SURFACE] anchor=`M:L119` (13-M-SUMMARY_INDEX.discovery.md:L347): elai-verify becomes the reference verifier for the format.
- **SUMMARY/M6-E018** [CLI] [SURFACE] anchor=`M:L121` (13-M-SUMMARY_INDEX.discovery.md:L353): elai-verify adds 'conform <artifact>'.
- **SUMMARY/M6-E024** [OPS] [SURFACE] anchor=`M:L124` (13-M-SUMMARY_INDEX.discovery.md:L359): The evidence format is published beside elai-verify.
- **SUMMARY/M6-E035** [TEST] [SURFACE] anchor=`M:L129-L130` (13-M-SUMMARY_INDEX.discovery.md:L370): A conformant fixture must pass 'elai-verify conform' on an engine-free machine.
- **SUMMARY/M6-E036** [TEST] [SURFACE] anchor=`M:L129-L130` (13-M-SUMMARY_INDEX.discovery.md:L371): A one-byte-tampered fixture must fail 'elai-verify conform'.
- **SUMMARY/M6-E037** [TEST] [SURFACE] anchor=`M:L129-L130` (13-M-SUMMARY_INDEX.discovery.md:L372): A missing-field fixture must fail 'elai-verify conform'.

### Summary surfaces-and-artifacts table

- **SUMMARY/SURF-001** [SURFACE] (13-M:L440): surface=`License CLI` — activate, status, deactivate (atoms: M1-E011..M1-E013)
- **SUMMARY/SURF-002** [SURFACE] (13-M:L441): surface=`SettingsWindow` — license-status line; exact presentation decided at edit (atoms: M1-E043..M1-E044)
- **SUMMARY/SURF-003** [SURFACE] (13-M:L442): surface=`Org-policy CLI` — init, sign, verify, show (atoms: M2-E024..M2-E027)
- **SUMMARY/SURF-004** [SURFACE] (13-M:L443): surface=`Fleet CLI` — seat export and admin report with verification (atoms: M3-E005, M3-E012)
- **SUMMARY/SURF-005** [SURFACE] (13-M:L444): surface=`Update CLI` — check, apply, rollback, channel (atoms: M4-E013..M4-E016)
- **SUMMARY/SURF-006** [SURFACE] (13-M:L445): surface=`Support CLI` — local redacted support bundle with printed summary (atoms: M4-E023..M4-E032)
- **SUMMARY/SURF-007** [SURFACE] (13-M:L446): surface=`elai-verify` — standalone verification of attestation, RRA, K-6 conformance, and org bundle (atoms: M5-E003..M5-E009)
- **SUMMARY/SURF-008** [SURFACE] (13-M:L447): surface=`elai-verify conform` — fail-closed evidence-format conformance (atoms: M6-E018..M6-E020)
- **SUMMARY/SURF-009** [SURFACE] (13-M:L448): surface=`Entitlement` — signed org/tier/seat/features/expiry record (atoms: M1-E002..M1-E009)
- **SUMMARY/SURF-010** [SURFACE] (13-M:L449): surface=`OrgPolicyPack` — signed inert file with organization controls and pins (atoms: M2-E003..M2-E018)
- **SUMMARY/SURF-011** [SURFACE] (13-M:L450): surface=`Organization bundle` — per-seat table plus recomputable rollups (atoms: M3-E013..M3-E027)
- **SUMMARY/SURF-012** [SURFACE] (13-M:L451): surface=`Release manifest` — signed, per-artifact blake3, stable/beta metadata (atoms: M4-E003..M4-E009)
- **SUMMARY/SURF-013** [SURFACE] (13-M:L452): surface=`Support bundle` — versions, tier, hashes, doctor, redacted journal tail (atoms: M4-E024..M4-E032)
- **SUMMARY/SURF-014** [SURFACE] (13-M:L453): surface=`Legal/commercial docs` — EULA, license split, DPA, continuity alternatives, custody and support posture (atoms: M5-E011..M5-E023)
- **SUMMARY/SURF-015** [SURFACE] (13-M:L454): surface=`tiers.toml` — Free/Pro/Team and pricing source of truth (atoms: M5-E024..M5-E029)
- **SUMMARY/SURF-016** [SURFACE] (13-M:L455): surface=`Published evidence format` — derived schemas, prose spec, fixtures, example emitter (atoms: M6-E004..M6-E025)

### Curated surface checklist (family M, by product surface)

- **CLI: elai license activate|status|deactivate** — refs: M-001/M1-E022, M-001/M1-E040, M-001/M5-E033, SUMMARY/M1-E011..M1-E013, SUMMARY/M5-E046
- **Status: ExpiredGrace status banner** — refs: M-001/M1-E037, M-001/M1-A002, M-001/AC-005
- **Installer: B-43 install-end activation offer** — refs: M-001/M1-E040, SUMMARY/M1-E034
- **CLI: elai org policy init|sign|verify|show** — refs: M-001/M2-E018, SUMMARY/M2-E024..M2-E027
- **Status: M-2 boot warning (Drift)** — refs: M-001/M2-E016, M-001/M2-A003
- **CLI: elai export compliance --org** — refs: M-001/M3-E019, SUMMARY/M3-E005
- **CLI: elai org report <dir> [--verify]** — refs: M-001/M3-E020, M-001/M3-E023, SUMMARY/M3-E012
- **Render: org report human markdown + machine msgpack/toml** — refs: M-001/M3-E026, M-001/M3-I001
- **CLI: elai update check|apply|rollback|channel** — refs: M-001/M4-E022, M-001/M4-E016, SUMMARY/M4-E013..M4-E016
- **CLI: elai support bundle** (+ printed content summary) — refs: M-001/M4-E028, M-001/M4-E029, SUMMARY/M4-E023, SUMMARY/M4-E031
- **Binary: elai-verify** (attestation, rra, K-6, org bundle) — refs: M-001/M5-E009..M5-E010, SUMMARY/M5-E003..M5-E007
- **CLI: elai-verify conform <artifact>** — refs: M-001/M6-E020, SUMMARY/M6-E018
- **Docs: support-expectation page** — refs: M-001/M5-E020, M-001/M5-I001
- **Docs: EULA/DPA/continuity under docs/strategy/commercial/** — refs: M-001/M5-E014..M5-E019, SUMMARY/M5-E011..M5-E023
- **Docs: EEF prose spec + third-party emitter example** — refs: M-001/M6-E017, M-001/M6-E026, SUMMARY/M6-E016, SUMMARY/M6-E025
- **Config surface: tiers.toml Free/Pro/Team + pricing** — refs: M-001/M5-E021..M5-E022, SUMMARY/M5-E024..M5-E029
- **[superseded as target UI] SettingsWindow license-status line** — SUMMARY/M1-E043..M1-E044 retained under SUP-001; not in numbered M-1

---

## Per-unit atomic records (deep ledger M-001)

Feature and unit-local records grouped by unit. Cross-file interactions, summary deltas, and ambiguity/conflict registers follow in dedicated sections.

## Ledger M-001

Source file: `docs/thefinalglance/feature-discovery/deep/DEEP-M-001.discovery.md`
Extracted: source=360, IX=24, SD=39, AC=12, subtotal=435

### Manifest and method (ledger header, preserved as evidence records)

- **M-001/HDR-001** [E/STATUS] (DEEP-M-001.discovery.md:L3-L11): Processed manifest DEEP-SOURCE-MANIFEST.md:L1195-L1207; family M; 7 files; 542 logical lines; 38,550 bytes; all numbered/index original; mechanical verification matched (0 missing/unexpected/duplicate/SHA/line/byte mismatches).
- **M-001/HDR-002** [E/STATUS] (DEEP-M-001.discovery.md:L8): Read boundary: all seven originals read completely; 13-M-SUMMARY_INDEX consulted only as orientation/delta artifact; no source atom derived solely from summary.
- **M-001/HDR-003** [E/EVIDENCE] (DEEP-M-001.discovery.md:L15-L25): Exact path/SHA/line/byte table for M-0..M-6 originals (aggregate 542/38,550).

### Unit `M-0 — INDEX (commercial shipping / multi-seat doctrine)` — from M-001 (63 records)

- **M-001/M0-E001** [E/STATUS] anchor=`M-0:L1-L1` (ledger-line: DEEP-M-001.discovery.md:L34): M-plan is the COMMERCIAL SHIPPING / MULTI-SEAT closing letter after B..L.
- **M-001/M0-E002** [E/STATUS] anchor=`M-0:L2-L3` (ledger-line: DEEP-M-001.discovery.md:L35): Generated 2026-07-02 in one wave by the main LLM in-band via paper-to-plan PHASE 4-5, with gap analysis at `workflow/reasoning/steal-multiseat-gap-2026-07-02.md`.
- **M-001/M0-E003** [E/STATUS] anchor=`M-0:L3-L4` (ledger-line: DEEP-M-001.discovery.md:L36): Scope is 6 sessions M-1..M-6 plus 7 Section-A execution pointers MA-1..MA-7.
- **M-001/M0-E004** [E/STATUS] anchor=`M-0:L4-L5` (ledger-line: DEEP-M-001.discovery.md:L37): M-6 was appended 2026-07-05 (evidence-format open standard; spec-eval ADD-1, user-directed); the rider’s informal "M-6+" slot was renumbered to M-7+.
- **M-001/M0-E005** [E/STATUS] anchor=`M-0:L5-L6` (ledger-line: DEEP-M-001.discovery.md:L38): Doctrine inherits F-0 W-1..W-10 + G-0 GD-1..GD-5 + K-0 KD-1..KD-5 and ADDS MD-1..MD-5.
- **M-001/M0-E006** [E/STATUS] anchor=`M-0:L8-L11` (ledger-line: DEEP-M-001.discovery.md:L39): USER OVERRIDE of K-10: K-10 defers new plan LETTERS until A-1..A-4 + K-3 are DONE; the user authorized this letter 2026-07-02 for authoring only.
- **M-001/M0-E007** [E/STATUS] anchor=`M-0:L10-L11` (ledger-line: DEEP-M-001.discovery.md:L40): M EXECUTION stays gated behind the K-10 order; M is authored last and runs last.
- **M-001/M0-E008** [E/CAP] anchor=`M-0:L13-L16` (ledger-line: DEEP-M-001.discovery.md:L41): B..L finished equals a sellable SINGLE seat; M makes the seat SELLABLE TO A COMPANY.
- **M-001/M0-E009** [E/CAP] anchor=`M-0:L14-L16` (ledger-line: DEEP-M-001.discovery.md:L42): M packages the verified seat into something an org can buy, deploy, police, and audit.
- **M-001/M0-E010** [E/CAP] anchor=`M-0:L16-L17` (ledger-line: DEEP-M-001.discovery.md:L43): M-1 is a seat entitlement a payment can mint and a machine can verify offline.
- **M-001/M0-E011** [E/CAP] anchor=`M-0:L17-L17` (ledger-line: DEEP-M-001.discovery.md:L44): M-2 is an org policy pack every seat obeys and cannot weaken.
- **M-001/M0-E012** [E/CAP] anchor=`M-0:L17-L18` (ledger-line: DEEP-M-001.discovery.md:L45): M-3 is a fleet evidence rollup that answers the org-side buyer K-7 targets.
- **M-001/M0-E013** [E/CAP] anchor=`M-0:L18-L18` (ledger-line: DEEP-M-001.discovery.md:L46): M-4 is a signed release channel + updater + support path.
- **M-001/M0-E014** [E/CAP] anchor=`M-0:L18-L19` (ledger-line: DEEP-M-001.discovery.md:L47): M-5 is the commercial/legal wrapper including the GTM open-source-verifier decision.
- **M-001/M0-E015** [E/DEP] anchor=`M-0:L19-L21` (ledger-line: DEEP-M-001.discovery.md:L48): Approximately 60% of M is composition of shipped/planned substrate: K-24 billing rail, J-49/J-59 per-seat, B-43 installer, K-6/K-7 evidence pack, G-8/G-9 signing/redaction, K-1 claims lint.
- **M-001/M0-E016** [E/CAP] anchor=`M-0:L20-L22` (ledger-line: DEEP-M-001.discovery.md:L49): The net-new is exactly the commerce machinery no prior letter owns (grep-proof in the gap file).
- **M-001/M0-E017** [E/RECOVERY] anchor=`M-0:L25-L26` (ledger-line: DEEP-M-001.discovery.md:L50): MD-1 NEVER-BRICK: entitlement failure/expiry degrades to the FREE tier — never blocks local work, never touches safety gates.
- **M-001/M0-E018** [E/FAIL] anchor=`M-0:L26-L28` (ledger-line: DEEP-M-001.discovery.md:L51): MD-1 commerce is fail-OPEN by design — the one deliberate, recorded inversion of R1.
- **M-001/M0-E019** [E/SEC] anchor=`M-0:L27-L28` (ledger-line: DEEP-M-001.discovery.md:L52): MD-1 keeps safety gates fail-closed exactly as before.
- **M-001/M0-E020** [E/CFG] anchor=`M-0:L28-L29` (ledger-line: DEEP-M-001.discovery.md:L53): The free tier is the base state, so W-2 is satisfied without an env gate.
- **M-001/M0-E021** [E/SEC] anchor=`M-0:L30-L31` (ledger-line: DEEP-M-001.discovery.md:L54): MD-2 OFFLINE-VERIFIABLE COMMERCE: every entitlement / policy pack / fleet bundle verifies OFFLINE using the G-8 local-Merkle/signature pattern.
- **M-001/M0-E022** [E/LIMIT] anchor=`M-0:L31-L32` (ledger-line: DEEP-M-001.discovery.md:L55): The license service is issue/renew/revoke ONLY — never a runtime dependency.
- **M-001/M0-E023** [E/CAP] anchor=`M-0:L32-L32` (ledger-line: DEEP-M-001.discovery.md:L56): A seat that never phones home keeps working (MD-1 floor).
- **M-001/M0-E024** [E/LIMIT] anchor=`M-0:L33-L33` (ledger-line: DEEP-M-001.discovery.md:L57): MD-3 MINIMAL FIRST-PARTY CLOUD: exactly one tiny endpoint (mint/renew/revoke).
- **M-001/M0-E025** [E/FLOW] anchor=`M-0:L33-L34` (ledger-line: DEEP-M-001.discovery.md:L58): Org policy and fleet evidence stay file/git-distributable.
- **M-001/M0-E026** [E/LIMIT] anchor=`M-0:L34-L35` (ledger-line: DEEP-M-001.discovery.md:L59): NO SSO/SAML/SCIM/RBAC/org-console build — the K-0:531 non-goal + GTM "agencies NOT enterprise" decision stand.
- **M-001/M0-E027** [E/PRIV] anchor=`M-0:L36-L36` (ledger-line: DEEP-M-001.discovery.md:L60): MD-4 SEAT-PRIVACY DEFAULT: per-seat journal/memory isolated (J-59).
- **M-001/M0-E028** [E/PRIV] anchor=`M-0:L36-L37` (ledger-line: DEEP-M-001.discovery.md:L61): Org sharing is opt-in per artifact class (K-0:382 decision upheld).
- **M-001/M0-E029** [E/PRIV] anchor=`M-0:L37-L38` (ledger-line: DEEP-M-001.discovery.md:L62): Rollups aggregate NUMBERS + HASHES, never raw transcripts (G-9 is a precondition of every export).
- **M-001/M0-E030** [E/CLAIM] anchor=`M-0:L39-L40` (ledger-line: DEEP-M-001.discovery.md:L63): MD-5 CLAIMS DISCIPLINE ON MONEY (KD-1 extended): no pricing/value/tier copy ships outside the K-1 claims lint.
- **M-001/M0-E031** [E/CLAIM] anchor=`M-0:L40-L40` (ledger-line: DEEP-M-001.discovery.md:L64): "$20=$200"-class claims stay gated on K-24's measured $/resolved-task number.
- **M-001/M0-E032** [E/DEP] anchor=`M-0:L42-L43` (ledger-line: DEEP-M-001.discovery.md:L65): MA-1 points at K-24 BCC billing/credits — the metering rail M-1's entitlement gates (L1..L4 before M-1 live leg).
- **M-001/M0-E033** [E/DEP] anchor=`M-0:L44-L44` (ledger-line: DEEP-M-001.discovery.md:L66): MA-2 points at J-49 multi-seat local-serving admission — per-user quota on a shared box; M-1 tier feeds it.
- **M-001/M0-E034** [E/DEP] anchor=`M-0:L45-L45` (ledger-line: DEEP-M-001.discovery.md:L67): MA-3 points at J-59 per-seat memory + trust isolation — the data-governance slice; gates M-3's rollup.
- **M-001/M0-E035** [E/DEP] anchor=`M-0:L46-L46` (ledger-line: DEEP-M-001.discovery.md:L68): MA-4 points at B-43 one-line install + B-45 dashboard installer — M-4 builds the updater they defer.
- **M-001/M0-E036** [E/DEP] anchor=`M-0:L47-L47` (ledger-line: DEEP-M-001.discovery.md:L69): MA-5 points at K-3/K-11 PR evidence emitter — the per-repo distribution leg M-3 aggregates above.
- **M-001/M0-E037** [E/DEP] anchor=`M-0:L48-L48` (ledger-line: DEEP-M-001.discovery.md:L70): MA-6 points at K-6/K-7/K-13 compliance export + briefs — the per-seat artifacts M-3 rolls up, M-5 wraps.
- **M-001/M0-E038** [E/DEP] anchor=`M-0:L49-L49` (ledger-line: DEEP-M-001.discovery.md:L71): MA-7 points at K-23 multi-repo workspace — orthogonal axis; noted to prevent scope confusion.
- **M-001/M0-E039** [E/LIMIT] anchor=`M-0:L42-L42` (ledger-line: DEEP-M-001.discovery.md:L72): Section-A pointers sequence EXISTING files only — M never re-authors them (GD-1).
- **M-001/M0-E040** [E/STATUS] anchor=`M-0:L51-L52` (ledger-line: DEEP-M-001.discovery.md:L73): M-1 is seat entitlement + purchase rail [KEYSTONE], WAVE M1.
- **M-001/M0-E041** [E/STATUS] anchor=`M-0:L53-L53` (ledger-line: DEEP-M-001.discovery.md:L74): M-2 is org policy pack + distribution + drift detection, WAVE M2.
- **M-001/M0-E042** [E/STATUS] anchor=`M-0:L54-L54` (ledger-line: DEEP-M-001.discovery.md:L75): M-3 is fleet evidence rollup (`elai org report`), WAVE M3, depends M-1, MA-3, MA-6.
- **M-001/M0-E043** [E/STATUS] anchor=`M-0:L55-L55` (ledger-line: DEEP-M-001.discovery.md:L76): M-4 is signed release channel + auto-updater + support bundle, WAVE M2, depends MA-4.
- **M-001/M0-E044** [E/STATUS] anchor=`M-0:L56-L56` (ledger-line: DEEP-M-001.discovery.md:L77): M-5 is commercial packaging + verifier OSS + trust continuity, WAVE M1(a,d)/M3(b,c).
- **M-001/M0-E045** [E/STATUS] anchor=`M-0:L57-L57` (ledger-line: DEEP-M-001.discovery.md:L78): M-6 is evidence format as a versioned open standard (EEF v1), WAVE M3, depends M-5a.
- **M-001/M0-E046** [E/STATUS] anchor=`M-0:L59-L62` (ledger-line: DEEP-M-001.discovery.md:L79): GATE M-0: no M session EXECUTES until K's A-1..A-4 + K-3 are DONE and K-24 slices L1..L4 are live.
- **M-001/M0-E047** [E/CLAIM] anchor=`M-0:L60-L62` (ledger-line: DEEP-M-001.discovery.md:L80): Selling before the measured number, distribution leg, and metering rail exist is selling an unmeasured claim (KD-1).
- **M-001/M0-E048** [E/STATUS] anchor=`M-0:L63-L63` (ledger-line: DEEP-M-001.discovery.md:L81): WAVE M1: M-1; M-5a (verifier OSS extraction) + M-5d (tiers.toml).
- **M-001/M0-E049** [E/STATUS] anchor=`M-0:L64-L64` (ledger-line: DEEP-M-001.discovery.md:L82): WAVE M2: M-2; M-4.
- **M-001/M0-E050** [E/STATUS] anchor=`M-0:L65-L66` (ledger-line: DEEP-M-001.discovery.md:L83): WAVE M3: M-3; M-5b (EULA/DPA/pricing under lint) + M-5c (continuity statement); M-6 (after M-5a extraction + stable emit paths).
- **M-001/M0-E051** [E/DEFER] anchor=`M-0:L68-L70` (ledger-line: DEEP-M-001.discovery.md:L84): ENTERPRISE RIDER (recorded, NOT planned): SSO/SAML/SCIM/RBAC + hosted org console = FUTURE M-7+ (renumbered 2026-07-05; M-6 = evidence-format standard).
- **M-001/M0-E052** [E/DEFER] anchor=`M-0:L70-L71` (ledger-line: DEEP-M-001.discovery.md:L85): Enterprise rider is DEMAND-GATED: authored only when >=3 paying orgs ask, or a signed deal is blocked on it.
- **M-001/M0-E053** [E/LIMIT] anchor=`M-0:L71-L72` (ledger-line: DEEP-M-001.discovery.md:L86): Until the demand gate fires, K-0:531 non-goal stands and the M-2 file-distributed policy pack is the org story.
- **M-001/M0-E054** [E/FAIL] anchor=`M-0:L72-L73` (ledger-line: DEEP-M-001.discovery.md:L87): Re-proposing an org SaaS before the gate fires is a P0-grade miss (feature-map Section 4 discipline).
- **M-001/M0-E055** [E/EVENT] anchor=`M-0:L75-L77` (ledger-line: DEEP-M-001.discovery.md:L88): LIVE SSB baseline 268/577 (K-plan stamp 2026-06-26, ssb.rs:1508 / action_type_lock.rs:775 — re-grep at edit).
- **M-001/M0-E056** [E/EVENT] anchor=`M-0:L77-L79` (ledger-line: DEEP-M-001.discovery.md:L89): M net +2..+4: +EntitlementEvent (M-1, Pattern-2), +OrgPolicyEvent (M-2, Pattern-2), optional +OrgReportEvent (M-3, prefer-reuse a K-6/G-8 row first), optional +ReleaseChannelEvent (M-4, prefer-reuse).
- **M-001/M0-E057** [E/EVENT] anchor=`M-0:L79-L79` (ledger-line: DEEP-M-001.discovery.md:L90): M-5 = zero SSB (E-23-class + one crate extraction).
- **M-001/M0-E058** [E/EVIDENCE] anchor=`M-0:L81-L83` (ledger-line: DEEP-M-001.discovery.md:L91): Live anchors are carried from K/L 2026-06-26..27 stamps + the 2026-07-02 gap-file greps.
- **M-001/M0-E059** [E/EVIDENCE] anchor=`M-0:L83-L84` (ledger-line: DEEP-M-001.discovery.md:L92): `_grounding/M-GROUNDING-MAP.md` is generated at the FIRST M execution session (before any code) — every anchor re-grepped then.
- **M-001/M0-E060** [E/CLAIM] anchor=`M-0:L84-L85` (ledger-line: DEEP-M-001.discovery.md:L93): Gap-existence claims are grep-proven in the gap file; external market/pricing figures are [EXTERNAL] and live only in M-5's lint-governed copy.
- **M-001/M0-I001** [I/DEP] anchor=`M-0:L42-L49` (ledger-line: DEEP-M-001.discovery.md:L94): MA-1..MA-7 are sequencing pointers only; their targets remain owned by their home letters and must not be reimplemented under M.
- **M-001/M0-A001** [A/STATUS] anchor=`M-0:L59-L62` (ledger-line: DEEP-M-001.discovery.md:L95): Exact DONE criteria for K A-1..A-4, K-3, and K-24 L1..L4 that open GATE M-0 are named but not restated in M-0.
- **M-001/M0-A002** [A/EVENT] anchor=`M-0:L77-L79` (ledger-line: DEEP-M-001.discovery.md:L96): Whether OrgReportEvent and ReleaseChannelEvent are created or reused is deferred to edit-time W-9 decision.

### Unit `M-1 — seat entitlement + purchase rail [KEYSTONE]` — from M-001 (59 records)

- **M-001/M1-E001** [E/STATUS] anchor=`M-1:L1-L3` (ledger-line: DEEP-M-001.discovery.md:L106): M-1 is Track M, lever SELLABLE, grounding STRONG (gap G1/G2, grep-proven; K-24 substrate PLANNED), nature BUILD, effort M-L, WAVE M1.
- **M-001/M1-E002** [E/DEP] anchor=`M-1:L2-L3` (ledger-line: DEEP-M-001.discovery.md:L107): Depends on K-24 slices L1..L4 live (MA-1) and G-8 attestation signing shape shipped.
- **M-001/M1-E003** [E/CAP] anchor=`M-1:L6-L9` (ledger-line: DEEP-M-001.discovery.md:L108): Objective: turn a payment into a verifiable seat — a signed, OFFLINE-verifiable Entitlement that a payment mints, a machine activates, and every tier-gated feature reads.
- **M-001/M1-E004** [E/RECOVERY] anchor=`M-1:L8-L9` (ledger-line: DEEP-M-001.discovery.md:L109): Expiry, grace, and degrade-to-free semantics can never brick a working seat (MD-1).
- **M-001/M1-E005** [E/STATUS] anchor=`M-1:L11-L12` (ledger-line: DEEP-M-001.discovery.md:L110): Gap G1/G2: zero entitlement machinery anywhere in B..L.
- **M-001/M1-E006** [E/DEP] anchor=`M-1:L12-L14` (ledger-line: DEEP-M-001.discovery.md:L111): K-24 is METERING (credits spent per usage) and deliberately leaves "payment webhook OUTSIDE ELAI"; it never binds a paying org to an installed seat.
- **M-001/M1-E007** [E/INT] anchor=`M-1:L15-L17` (ledger-line: DEEP-M-001.discovery.md:L112): The signing/verification pattern EXISTS as G-8 blake3-Merkle/ed25519-signed attestations verified offline by `elai verify-attestation`.
- **M-001/M1-E008** [E/INT] anchor=`M-1:L16-L17` (ledger-line: DEEP-M-001.discovery.md:L113): The Entitlement is the SAME shape pointed at commerce (GD-1: reuse the signing/verify core, do not re-implement).
- **M-001/M1-E009** [E/CLAIM] anchor=`M-1:L18-L18` (ledger-line: DEEP-M-001.discovery.md:L114): Buyer per GTM memory: team/agency seats, $39-49/seat class.
- **M-001/M1-E010** [E/CFG] anchor=`M-1:L18-L18` (ledger-line: DEEP-M-001.discovery.md:L115): Tiers are DATA (M-5d tiers.toml).
- **M-001/M1-E011** [E/RECOVERY] anchor=`M-1:L21-L22` (ledger-line: DEEP-M-001.discovery.md:L116): LOCK-1 MD-1: expiry/verification-failure => DegradedToFree + everything Free-tier keeps working + safety gates untouched.
- **M-001/M1-E012** [E/LIMIT] anchor=`M-1:L22-L22` (ledger-line: DEEP-M-001.discovery.md:L117): No kill-the-binary path exists in the code, period.
- **M-001/M1-E013** [E/LIMIT] anchor=`M-1:L23-L24` (ledger-line: DEEP-M-001.discovery.md:L118): LOCK-2 MD-2/MD-3: the license service does mint/renew/revoke ONLY. No telemetry, no runtime callback, no feature flags served remotely.
- **M-001/M1-E014** [E/SEC] anchor=`M-1:L24-L25` (ledger-line: DEEP-M-001.discovery.md:L119): A seat offline for a year still verifies its entitlement locally (expiry logic is local, from the signed artifact + ClockProvider).
- **M-001/M1-E015** [E/COMP] anchor=`M-1:L26-L28` (ledger-line: DEEP-M-001.discovery.md:L120): LOCK-3 payment provider: Merchant-of-Record (Paddle/Lemon-Squeezy-class) OUTSIDE ELAI, PCI scope zero — the K-24 decision inherited verbatim.
- **M-001/M1-E016** [E/DEFER] anchor=`M-1:L27-L28` (ledger-line: DEEP-M-001.discovery.md:L121): Provider choice is a user decision at execution (E-50's MoR-vs-Stripe fork applies to ELAI itself here).
- **M-001/M1-E017** [E/DATA] anchor=`M-1:L29-L31` (ledger-line: DEEP-M-001.discovery.md:L122): LOCK-4 Entitlement is org-granular with seat slots: Entitlement{org_id, tier, seats_max, features[], issued_at_ms, expires_at_ms, grace_days=14, sig}.
- **M-001/M1-E018** [E/DATA] anchor=`M-1:L30-L31` (ledger-line: DEEP-M-001.discovery.md:L123): A seat ACTIVATION binds {seat_id = blake3(machine-scoped salt), org_id} locally.
- **M-001/M1-E019** [E/LIMIT] anchor=`M-1:L31-L33` (ledger-line: DEEP-M-001.discovery.md:L124): Seat-count enforcement is HONEST best-effort client-side (an offline product cannot hard-enforce concurrency).
- **M-001/M1-E020** [E/OBS] anchor=`M-1:L32-L33` (ledger-line: DEEP-M-001.discovery.md:L125): Over-use shows in the M-3 rollup instead — recorded limit, GD-5.
- **M-001/M1-E021** [E/CAP] anchor=`M-1:L36-L37` (ledger-line: DEEP-M-001.discovery.md:L126): Task 1: Entitlement type + signer/verifier in `crates/orchestrator/src/commerce/entitlement.rs` reusing the G-8 signing core.
- **M-001/M1-E022** [E/SURFACE] [SURFACE] anchor=`M-1:L37-L37` (ledger-line: DEEP-M-001.discovery.md:L127): CLI verbs: `elai license activate <file|key>`, `elai license status`, `elai license deactivate`.
- **M-001/M1-E023** [E/DATA] anchor=`M-1:L37-L38` (ledger-line: DEEP-M-001.discovery.md:L128): Local store under `~/.elai/entitlement/`.
- **M-001/M1-E024** [E/EVENT] anchor=`M-1:L38-L39` (ledger-line: DEEP-M-001.discovery.md:L129): +EntitlementEvent (Pattern-2: Activated/Renewed/ExpiredGrace/DegradedToFree/Revoked/RefusedForged).
- **M-001/M1-E025** [E/CFG] anchor=`M-1:L40-L40` (ledger-line: DEEP-M-001.discovery.md:L130): Task 2: tier gating seam — one `entitlement::tier()` read.
- **M-001/M1-E026** [E/INT] anchor=`M-1:L40-L41` (ledger-line: DEEP-M-001.discovery.md:L131): tier() is consumed by (a) K-24 credit-grant path (Team credits only mint against a live Team entitlement).
- **M-001/M1-E027** [E/INT] anchor=`M-1:L41-L41` (ledger-line: DEEP-M-001.discovery.md:L132): tier() is consumed by (b) M-2 org-pack apply (Team feature).
- **M-001/M1-E028** [E/INT] anchor=`M-1:L41-L42` (ledger-line: DEEP-M-001.discovery.md:L133): tier() is consumed by (c) M-3 `org report` (Team feature).
- **M-001/M1-E029** [E/CFG] anchor=`M-1:L42-L43` (ledger-line: DEEP-M-001.discovery.md:L134): Free tier = everything shipped through L that is not explicitly Team-tier in tiers.toml (M-5d).
- **M-001/M1-E030** [E/LIMIT] anchor=`M-1:L43-L44` (ledger-line: DEEP-M-001.discovery.md:L135): The gate READS data; the LLM never adjudicates tier (TD-3).
- **M-001/M1-E031** [E/CAP] anchor=`M-1:L45-L46` (ledger-line: DEEP-M-001.discovery.md:L136): Task 3: mint service (the ONE first-party cloud piece, MD-3): MoR webhook in, signed Entitlement out (mint/renew/revoke).
- **M-001/M1-E032** [E/SEC] anchor=`M-1:L46-L46` (ledger-line: DEEP-M-001.discovery.md:L137): Mint keys are offline-managed.
- **M-001/M1-E033** [E/OBS] anchor=`M-1:L46-L47` (ledger-line: DEEP-M-001.discovery.md:L138): Every mint is appended to an operator-side ledger file (auditable, GD-2-shaped).
- **M-001/M1-E034** [E/LIMIT] anchor=`M-1:L47-L48` (ledger-line: DEEP-M-001.discovery.md:L139): Mint-service deployment is an ops runbook, not a crate in the workspace.
- **M-001/M1-E035** [E/FLOW] anchor=`M-1:L49-L49` (ledger-line: DEEP-M-001.discovery.md:L140): Task 4: renewal/expiry loop — opportunistic re-check when online (never blocking).
- **M-001/M1-E036** [E/STATE] anchor=`M-1:L49-L50` (ledger-line: DEEP-M-001.discovery.md:L141): Expiry -> 14-day grace (ExpiredGrace rows + status banner) -> DegradedToFree.
- **M-001/M1-E037** [E/SURFACE] [SURFACE] anchor=`M-1:L50-L50` (ledger-line: DEEP-M-001.discovery.md:L142): ExpiredGrace emits a status banner (user-visible).
- **M-001/M1-E038** [E/FLOW] anchor=`M-1:L50-L51` (ledger-line: DEEP-M-001.discovery.md:L143): Revocation applies on next opportunistic check only (offline seats keep grace semantics — honest, GD-5).
- **M-001/M1-E039** [E/DEP] anchor=`M-1:L53-L56` (ledger-line: DEEP-M-001.discovery.md:L144): COMPOSES-WITH: G-8 attestation signing/verify core; K-24 credit ledger + seats.toml; ClockProvider (all expiry math — no SystemTime); K-1 claims lint (any UI copy about tiers); B-43 installer (activation offered at install-end, one line).
- **M-001/M1-E040** [E/SURFACE] [SURFACE] anchor=`M-1:L56-L56` (ledger-line: DEEP-M-001.discovery.md:L145): B-43 installer offers activation at install-end (one line).
- **M-001/M1-E041** [E/DATA] anchor=`M-1:L58-L60` (ledger-line: DEEP-M-001.discovery.md:L146): WIRING produces: commerce/entitlement.rs (Entitlement, activate/status/deactivate, tier()), EntitlementEvent SSB (+1), mint-service runbook + operator ledger format.
- **M-001/M1-E042** [E/INT] anchor=`M-1:L61-L61` (ledger-line: DEEP-M-001.discovery.md:L147): WIRING consumes: G-8 signing core, K-24 grant path, tiers.toml (M-5d), ClockProvider.
- **M-001/M1-E043** [E/INT] anchor=`M-1:L62-L62` (ledger-line: DEEP-M-001.discovery.md:L148): reachable_from: `elai license *` CLI (real entrypoint); tier() read on the K-24 grant path.
- **M-001/M1-E044** [E/TEST] anchor=`M-1:L63-L63` (ledger-line: DEEP-M-001.discovery.md:L149): trigger: `elai license activate fixtures/team-entitlement.signed && elai license status`.
- **M-001/M1-E045** [E/TEST] anchor=`M-1:L64-L65` (ledger-line: DEEP-M-001.discovery.md:L150): expect_ssb: EntitlementEvent(Activated) with org_id + tier=Team; a K-24 fixture grant succeeds ONLY with the live Team entitlement.
- **M-001/M1-E046** [E/TEST] anchor=`M-1:L66-L67` (ledger-line: DEEP-M-001.discovery.md:L151): Fixture Team entitlement activates OFFLINE (network disabled) -> Activated row + Team features unlock.
- **M-001/M1-E047** [E/TEST] anchor=`M-1:L67-L69` (ledger-line: DEEP-M-001.discovery.md:L152): Clock-forwarded expiry fixture (SeededClock) -> ExpiredGrace then DegradedToFree rows AND a Free-tier task still runs green AND every safety gate still fires (MD-1 negative arm).
- **M-001/M1-E048** [E/TEST] anchor=`M-1:L69-L70` (ledger-line: DEEP-M-001.discovery.md:L153): A forged-signature entitlement is REFUSED (RefusedForged, R2).
- **M-001/M1-E049** [E/TEST] anchor=`M-1:L70-L71` (ledger-line: DEEP-M-001.discovery.md:L154): Seat over-activation beyond seats_max is RECORDED (visible in M-3), not silently absorbed.
- **M-001/M1-E050** [E/SEC] anchor=`M-1:L73-L75` (ledger-line: DEEP-M-001.discovery.md:L155): Risk R1 key management: a leaked mint key = infinite seats. Mitigation: offline key ceremony + revoke list shipped in release manifests (M-4) — recorded, not solved, GD-5.
- **M-001/M1-E051** [E/LIMIT] anchor=`M-1:L76-L78` (ledger-line: DEEP-M-001.discovery.md:L156): Risk R2 fail-open abuse: Free-tier floor means piracy of Team features is a licensing matter, not a technical wall. Deliberate (MD-1); paid value concentrates in org pack/rollup which are org-workflow features, low piracy value.
- **M-001/M1-E052** [E/FAIL] anchor=`M-1:L79-L80` (ledger-line: DEEP-M-001.discovery.md:L157): Risk R3 clock tampering: local expiry uses ClockProvider/OsClock; a wound-back clock extends grace. Bounded by opportunistic online re-check; recorded residue (GD-5).
- **M-001/M1-I001** [I/STATE] anchor=`M-1:L37-L39, L58-L60` (ledger-line: DEEP-M-001.discovery.md:L158): deactivate is a first-class CLI verb producing local deactivation state, but its exact local-store mutation and SSB arm are not separately named beyond the activate/status/deactivate surface list.
- **M-001/M1-A001** [A/DATA] anchor=`M-1:L30-L31` (ledger-line: DEEP-M-001.discovery.md:L159): machine-scoped salt for seat_id is named but its storage, rotation, and privacy properties are unspecified.
- **M-001/M1-A002** [A/SURFACE] [SURFACE] anchor=`M-1:L50-L50` (ledger-line: DEEP-M-001.discovery.md:L160): "status banner" for ExpiredGrace is named without panel/window placement, copy, or dismiss rules.
- **M-001/M1-A003** [A/CFG] anchor=`M-1:L18-L18, L42-L43` (ledger-line: DEEP-M-001.discovery.md:L161): Exact Free vs Pro vs Team feature-key inventory is deferred to M-5d tiers.toml content.
- **M-001/M1-A004** [A/FLOW] anchor=`M-1:L49-L51` (ledger-line: DEEP-M-001.discovery.md:L162): Opportunistic online re-check interval, transport, and failure behavior are unspecified.
- **M-001/M1-A005** [A/DATA] anchor=`M-1:L46-L47, L59-L60` (ledger-line: DEEP-M-001.discovery.md:L163): Operator ledger format is produced but its schema fields are not enumerated.
- **M-001/M1-C001** [C/SURFACE] [SURFACE] anchor=`M-1:L37-L37, L50-L50, L56-L56` (ledger-line: DEEP-M-001.discovery.md:L164): Summary discovery asserts a SettingsWindow license-status line (W-4 deferred). Numbered M-1 names `elai license status` + ExpiredGrace status banner and B-43 install-end activation; it does not name SettingsWindow.

### Unit `M-2 — org policy pack + distribution + drift detection` — from M-001 (44 records)

- **M-001/M2-E001** [E/STATUS] anchor=`M-2:L1-L3` (ledger-line: DEEP-M-001.discovery.md:L174): M-2 is Track M, lever CANT-WRECK (org grain), grounding STRONG (gap G3; 3F/R8/E-39 substrate SHIPPED), nature BUILD-compose, effort M, WAVE M2.
- **M-001/M2-E002** [E/DEP] anchor=`M-2:L2-L3` (ledger-line: DEEP-M-001.discovery.md:L175): Depends on M-1 (Team tier), rules-engine 3F lifecycle, gate-manifest freeze (R8) live.
- **M-001/M2-E003** [E/CAP] anchor=`M-2:L6-L7` (ledger-line: DEEP-M-001.discovery.md:L176): One signed, file-distributable OrgPolicyPack every seat applies at boot and cannot weaken.
- **M-001/M2-E004** [E/CFG] anchor=`M-2:L7-L9` (ledger-line: DEEP-M-001.discovery.md:L177): Pack contents: org rules (mutable=0-equivalent tier), gate-manifest pins, spend caps, model/backend allowlist, redaction config, update-channel pin.
- **M-001/M2-E005** [E/CAP] anchor=`M-2:L8-L9` (ledger-line: DEEP-M-001.discovery.md:L178): The org buys ONE control surface instead of policing N laptops by hand.
- **M-001/M2-E006** [E/LIMIT] anchor=`M-2:L10-L10` (ledger-line: DEEP-M-001.discovery.md:L179): No cloud, no console — the pack travels by git/file (MD-3).
- **M-001/M2-E007** [E/STATUS] anchor=`M-2:L12-L15` (ledger-line: DEEP-M-001.discovery.md:L180): Gap G3: mutable=0 rules, gate_freeze_check + .claude/gate-manifest.txt, E-39 caps, B-37 allowlists, G-9 redaction ALL exist — per-install, hand-configured, with no cross-seat distribution or drift story.
- **M-001/M2-E008** [E/LIMIT] anchor=`M-2:L15-L16` (ledger-line: DEEP-M-001.discovery.md:L181): The org layer was an explicit K non-goal (K-0:531); M-2 is its minimal file-based form, NOT an org SaaS (ENTERPRISE RIDER stands).
- **M-001/M2-E009** [E/SEC] anchor=`M-2:L17-L18` (ledger-line: DEEP-M-001.discovery.md:L182): The freeze discipline is the precedent: R8 proves "a run cannot edit its own gate"; M-2 lifts the same property one level — "a seat cannot edit its org's gate".
- **M-001/M2-E010** [E/DATA] anchor=`M-2:L21-L22` (ledger-line: DEEP-M-001.discovery.md:L183): LOCK-1 the pack is DATA: policy.toml + rule rows + manifest pin hashes + caps + allowlist + redaction config + channel pin, signed with the org key (G-8 core again).
- **M-001/M2-E011** [E/LIMIT] anchor=`M-2:L22-L23` (ledger-line: DEEP-M-001.discovery.md:L184): NEVER executable content (the L-13 FS-2 doctrine at org grain).
- **M-001/M2-E012** [E/SEC] anchor=`M-2:L24-L25` (ledger-line: DEEP-M-001.discovery.md:L185): LOCK-2 precedence: org pack > local config, and org rules land in an immune tier a local edit cannot demote (exactly the mutable=0 semantics, org-keyed).
- **M-001/M2-E013** [E/SEC] anchor=`M-2:L25-L26` (ledger-line: DEEP-M-001.discovery.md:L186): A seat may ADD stricter checks, never weaken (monotone-tighten only).
- **M-001/M2-E014** [E/CFG] anchor=`M-2:L27-L28` (ledger-line: DEEP-M-001.discovery.md:L187): LOCK-3 absence of a pack = today's behavior, byte-for-byte. Single-seat users never see M-2 (W-2 satisfied structurally; no env gate).
- **M-001/M2-E015** [E/OBS] anchor=`M-2:L29-L30` (ledger-line: DEEP-M-001.discovery.md:L188): LOCK-4 drift is DETECTED + surfaced, not auto-corrected: boot compares applied-pack hash vs the org-pinned hash -> OrgPolicyEvent(Drift) + boot warning + M-3 rollup visibility.
- **M-001/M2-E016** [E/SURFACE] [SURFACE] anchor=`M-2:L29-L30` (ledger-line: DEEP-M-001.discovery.md:L189): Drift surfaces as a boot warning.
- **M-001/M2-E017** [E/DEFER] anchor=`M-2:L30-L31` (ledger-line: DEEP-M-001.discovery.md:L190): Auto-remediate is a later rider (needs GD-4 FP bounding first).
- **M-001/M2-E018** [E/CAP] anchor=`M-2:L34-L34` (ledger-line: DEEP-M-001.discovery.md:L191): Task 1: pack format + signer/verifier + `elai org policy init|sign|verify|show` (admin authoring side).
- **M-001/M2-E019** [E/INT] anchor=`M-2:L35-L35` (ledger-line: DEEP-M-001.discovery.md:L192): Module: `crates/orchestrator/src/commerce/org_policy.rs`.
- **M-001/M2-E020** [E/EVENT] anchor=`M-2:L35-L36` (ledger-line: DEEP-M-001.discovery.md:L193): +OrgPolicyEvent (Pattern-2: Applied/Drift/RefusedForged/WeakenAttemptBlocked).
- **M-001/M2-E021** [E/FLOW] anchor=`M-2:L37-L39` (ledger-line: DEEP-M-001.discovery.md:L194): Task 2 boot apply seam: pack discovery (repo `.elai/org-policy.signed`, then `~/.elai/org/`), verify, apply-in-order.
- **M-001/M2-E022** [E/FLOW] anchor=`M-2:L38-L39` (ledger-line: DEEP-M-001.discovery.md:L195): Apply-in-order: rules rows via the 3F loader into the org-immune tier; caps into E-39; pins compared against the local frozen set; redaction config into G-9; channel pin read by M-4.
- **M-001/M2-E023** [E/EVENT] anchor=`M-2:L40-L40` (ledger-line: DEEP-M-001.discovery.md:L196): Emits Applied with the pack hash.
- **M-001/M2-E024** [E/FAIL] anchor=`M-2:L41-L42` (ledger-line: DEEP-M-001.discovery.md:L197): Task 3 weaken-attempt gate: a local rule edit / cap raise / allowlist widen that contradicts the pack is REFUSED with WeakenAttemptBlocked naming the pack clause (deterministic compare, TD-3).
- **M-001/M2-E025** [E/OBS] anchor=`M-2:L43-L43` (ledger-line: DEEP-M-001.discovery.md:L198): Task 4 drift check: per-boot + per-`org report` (M-3 consumes Drift rows per seat).
- **M-001/M2-E026** [E/DEP] anchor=`M-2:L45-L48` (ledger-line: DEEP-M-001.discovery.md:L199): COMPOSES-WITH: 3F rules lifecycle + mutable-tier semantics; gate_freeze_check.py / gate-manifest (R8 — the pack PINS it, never replaces it); E-39 cost gate (caps consumer); K-5 GateIntensity policy table (the pack may pin its hash); G-9 redaction; G-8 signing core; M-1 tier() (pack apply = Team feature).
- **M-001/M2-E027** [E/DATA] anchor=`M-2:L50-L52` (ledger-line: DEEP-M-001.discovery.md:L200): WIRING produces: commerce/org_policy.rs (pack type, verify/apply, weaken gate), `elai org policy *` CLI, OrgPolicyEvent SSB (+1).
- **M-001/M2-E028** [E/INT] anchor=`M-2:L53-L53` (ledger-line: DEEP-M-001.discovery.md:L201): WIRING consumes: 3F loader, gate-manifest hashes, E-39 caps, G-9 config, M-1 tier, G-8 signing.
- **M-001/M2-E029** [E/INT] anchor=`M-2:L54-L54` (ledger-line: DEEP-M-001.discovery.md:L202): reachable_from: boot path (pack discovery) + `elai org policy *` CLI.
- **M-001/M2-E030** [E/TEST] anchor=`M-2:L55-L56` (ledger-line: DEEP-M-001.discovery.md:L203): trigger: place fixtures/org-policy.signed (spend cap 0.05, one org GREP rule, stable channel pin) in a fixture repo; boot; run a fixture task that would exceed the cap.
- **M-001/M2-E031** [E/TEST] anchor=`M-2:L57-L58` (ledger-line: DEEP-M-001.discovery.md:L204): expect_ssb: OrgPolicyEvent(Applied) with pack hash; the task blocked by the ORG cap (E-39 row names the org clause, not local config); the org GREP rule fires in SHADOW.
- **M-001/M2-E032** [E/TEST] anchor=`M-2:L59-L59` (ledger-line: DEEP-M-001.discovery.md:L205): Fixture boots with the pack applied + org cap enforced.
- **M-001/M2-E033** [E/TEST] anchor=`M-2:L59-L60` (ledger-line: DEEP-M-001.discovery.md:L206): A tampered pack byte is REFUSED (RefusedForged, R2).
- **M-001/M2-E034** [E/TEST] anchor=`M-2:L60-L61` (ledger-line: DEEP-M-001.discovery.md:L207): A local attempt to raise the org cap -> WeakenAttemptBlocked.
- **M-001/M2-E035** [E/TEST] anchor=`M-2:L61-L61` (ledger-line: DEEP-M-001.discovery.md:L208): A seat with NO pack behaves byte-identically to pre-M-2 (regression arm).
- **M-001/M2-E036** [E/TEST] anchor=`M-2:L61-L62` (ledger-line: DEEP-M-001.discovery.md:L209): A stale applied-pack vs pinned-hash fixture -> Drift row at boot.
- **M-001/M2-E037** [E/FAIL] anchor=`M-2:L64-L67` (ledger-line: DEEP-M-001.discovery.md:L210): Risk R1 org-rule FP friction at fleet scale (the CodeGate death, GD-4): org rules enter SHADOW first, promote per-org on the same fpr<5% discipline — the 3F lifecycle already encodes this; the pack just carries lifecycle state.
- **M-001/M2-E038** [E/SEC] anchor=`M-2:L68-L69` (ledger-line: DEEP-M-001.discovery.md:L211): Risk R2 pack-key compromise = org-wide policy injection. Same key posture as M-1 R1; pack verify also pins the SIGNER identity per org (a foreign org's validly-signed pack is refused).
- **M-001/M2-E039** [E/DET] anchor=`M-2:L70-L71` (ledger-line: DEEP-M-001.discovery.md:L212): Risk R3 precedence bugs (org vs local vs defaults) are silent-failure-prone: the apply order is one deterministic function with a contract test enumerating all three layers (R2 broken-arm test).
- **M-001/M2-I001** [I/CFG] anchor=`M-2:L37-L39, L45-L48` (ledger-line: DEEP-M-001.discovery.md:L213): Because apply order maps each pack clause to a named consumer (3F/E-39/gate-manifest/G-9/M-4), the pack is a binding multi-subsystem configuration object rather than a free-form document.
- **M-001/M2-A001** [A/DATA] anchor=`M-2:L21-L22, L34-L35` (ledger-line: DEEP-M-001.discovery.md:L214): Exact policy.toml field schema, rule-row serialization, and signature envelope layout are not specified.
- **M-001/M2-A002** [A/CFG] anchor=`M-2:L29-L30` (ledger-line: DEEP-M-001.discovery.md:L215): Stale threshold that distinguishes Drift from acceptable lag is not defined beyond applied-pack hash vs org-pinned hash.
- **M-001/M2-A003** [A/SURFACE] [SURFACE] anchor=`M-2:L29-L30` (ledger-line: DEEP-M-001.discovery.md:L216): Boot warning presentation (CLI only vs UI badge) is not specified.
- **M-001/M2-A004** [A/FLOW] anchor=`M-2:L10-L10, L37-L38` (ledger-line: DEEP-M-001.discovery.md:L217): Git distribution workflow, multi-pack conflict, and pack revocation/rotation are not specified.

### Unit `M-3 — fleet evidence rollup (`elai org report`)` — from M-001 (47 records)

- **M-001/M3-E001** [E/STATUS] anchor=`M-3:L1-L3` (ledger-line: DEEP-M-001.discovery.md:L227): M-3 is Track M, lever AUDITABLE (org grain), grounding STRONG (gap G4; K-6/G-2/G-8/K-1 SHIPPED-or-PLANNED per-seat), nature BUILD-compose, effort M, WAVE M3.
- **M-001/M3-E002** [E/DEP] anchor=`M-3:L2-L4` (ledger-line: DEEP-M-001.discovery.md:L228): Depends on M-1 (Team tier), M-2 (Drift rows), MA-3 (J-59 per-seat isolation), MA-6 (K-6 export shape).
- **M-001/M3-E003** [E/CAP] anchor=`M-3:L6-L8` (ledger-line: DEEP-M-001.discovery.md:L229): Answers org-side buyer (K-7 AppSec/GRC persona) questions: what the FLEET did, what reached reviewers unvouched, whether seats are on policy, cost per verified outcome.
- **M-001/M3-E004** [E/CAP] anchor=`M-3:L8-L11` (ledger-line: DEEP-M-001.discovery.md:L230): Aggregates per-seat exports into ONE org evidence bundle whose every number recomputes from the constituent seat journals — GD-2 lifted to fleet grain.
- **M-001/M3-E005** [E/PRIV] anchor=`M-3:L11-L11` (ledger-line: DEEP-M-001.discovery.md:L231): Zero telemetry and zero raw content (MD-4).
- **M-001/M3-E006** [E/STATUS] anchor=`M-3:L13-L15` (ledger-line: DEEP-M-001.discovery.md:L232): Gap G4: K-6 compliance export, G-2 RRA, K-1 dossier, K-4 gate-tax, H-9 spend, G-6 SAFEDELTA are all per-seat files. No aggregation surface exists (grep-proven).
- **M-001/M3-E007** [E/FLOW] anchor=`M-3:L16-L17` (ledger-line: DEEP-M-001.discovery.md:L233): Pull-model by design: seats EXPORT (a file), an admin AGGREGATES (a command). No agent phones home, ever (MD-3/MD-4).
- **M-001/M3-E008** [E/FLOW] anchor=`M-3:L17-L17` (ledger-line: DEEP-M-001.discovery.md:L234): Distribution medium is the org's own file share / git — same as M-2.
- **M-001/M3-E009** [E/PRIV] anchor=`M-3:L18-L19` (ledger-line: DEEP-M-001.discovery.md:L235): K-0:382 upheld: shares evidence NUMBERS + HASHES, not memory/content — does not touch the org-shared-memory decision (which stays with J-49/J-59's later opt-in).
- **M-001/M3-E010** [E/DATA] anchor=`M-3:L21-L23` (ledger-line: DEEP-M-001.discovery.md:L236): LOCK-1 rollup input = the K-6 bundle format extended with seat_id + entitlement org_id + M-2 pack-hash + Drift status.
- **M-001/M3-E011** [E/LIMIT] anchor=`M-3:L23-L24` (ledger-line: DEEP-M-001.discovery.md:L237): One export shape, two consumers (K-6 procurement, M-3 rollup) — never a second bespoke export (GD-1).
- **M-001/M3-E012** [E/PRIV] anchor=`M-3:L25-L26` (ledger-line: DEEP-M-001.discovery.md:L238): LOCK-2 privacy floor: G-9 redaction is a PRECONDITION of export (already K-6's rule).
- **M-001/M3-E013** [E/TEST] anchor=`M-3:L26-L27` (ledger-line: DEEP-M-001.discovery.md:L239): The rollup additionally greps its own output for seeded secret markers and FAILS the build of the bundle on a hit (R2-style self-test).
- **M-001/M3-E014** [E/FAIL] anchor=`M-3:L28-L30` (ledger-line: DEEP-M-001.discovery.md:L240): LOCK-3 broken chains are refused: a seat export whose hash chain / attestation does not verify is EXCLUDED and NAMED in the rollup (RefusedSeat row in the report).
- **M-001/M3-E015** [E/FAIL] anchor=`M-3:L29-L30` (ledger-line: DEEP-M-001.discovery.md:L241): Never silently included or silently dropped (R1: missing/invalid evidence screams).
- **M-001/M3-E016** [E/DATA] anchor=`M-3:L31-L33` (ledger-line: DEEP-M-001.discovery.md:L242): LOCK-4 metrics in the rollup are the ALREADY-DEFINED ones: verified-outcome rate, K-4 gate-tax, K-1 dossier deltas, H-9 $/resolved-task, unvouched-hunk counts from K-3, M-2 Drift.
- **M-001/M3-E017** [E/CLAIM] anchor=`M-3:L32-L33` (ledger-line: DEEP-M-001.discovery.md:L243): M-3 defines ZERO new metrics (KD-1: no new number without its own measurement session).
- **M-001/M3-E018** [E/DATA] anchor=`M-3:L36-L37` (ledger-line: DEEP-M-001.discovery.md:L244): Task 1 seat-export extension: K-6 bundle + {seat_id, org_id, pack_hash, drift_status} fields (#[serde(default)], no migration).
- **M-001/M3-E019** [E/SURFACE] [SURFACE] anchor=`M-3:L37-L37` (ledger-line: DEEP-M-001.discovery.md:L245): CLI arm: `elai export compliance --org`.
- **M-001/M3-E020** [E/SURFACE] [SURFACE] anchor=`M-3:L38-L38` (ledger-line: DEEP-M-001.discovery.md:L246): Task 2 aggregator: `elai org report <dir-of-seat-exports>` -> one org bundle.
- **M-001/M3-E021** [E/DATA] anchor=`M-3:L38-L39` (ledger-line: DEEP-M-001.discovery.md:L247): Org bundle contains: per-seat table + org rollups + a manifest of constituent bundle hashes.
- **M-001/M3-E022** [E/SEC] anchor=`M-3:L39-L40` (ledger-line: DEEP-M-001.discovery.md:L248): Offline-verifiable end-to-end (`elai org report --verify` re-checks every constituent chain).
- **M-001/M3-E023** [E/SURFACE] [SURFACE] anchor=`M-3:L39-L40` (ledger-line: DEEP-M-001.discovery.md:L249): Verify flag form: `elai org report --verify`.
- **M-001/M3-E024** [E/INT] anchor=`M-3:L40-L41` (ledger-line: DEEP-M-001.discovery.md:L250): Module: `crates/orchestrator/src/commerce/org_report.rs`.
- **M-001/M3-E025** [E/EVENT] anchor=`M-3:L41-L42` (ledger-line: DEEP-M-001.discovery.md:L251): SSB: prefer-reuse ComplianceExported/G-8 rows; +OrgReportEvent only if no existing row fits (decide at edit, W-9).
- **M-001/M3-E026** [E/SURFACE] [SURFACE] anchor=`M-3:L43-L44` (ledger-line: DEEP-M-001.discovery.md:L252): Task 3 render: one human sheet (markdown) + one machine file (msgpack/toml) — the K-7 brief's org answer, generated not hand-written.
- **M-001/M3-E027** [E/OBS] anchor=`M-3:L45-L46` (ledger-line: DEEP-M-001.discovery.md:L253): Task 4 seat-count honesty leg: distinct seat_ids per org vs seats_max (M-1 LOCK-4's recorded limit surfaces HERE — over-activation is visible to the org admin, the honest enforcement point).
- **M-001/M3-E028** [E/DEP] anchor=`M-3:L48-L51` (ledger-line: DEEP-M-001.discovery.md:L254): COMPOSES-WITH: K-6 export (input shape — extended, not forked); G-2 RRA + G-8 attestation verify core (the --verify leg); G-9 (precondition); K-1 dossier + K-4 + H-9 (the numbers); K-3/K-11 (unvouched counts); M-1 entitlement (org_id + Team gating); M-2 (Drift).
- **M-001/M3-E029** [E/DATA] anchor=`M-3:L53-L55` (ledger-line: DEEP-M-001.discovery.md:L255): WIRING produces: org_report.rs (aggregate + verify + render), `elai org report` CLI, seat-export field extension; +0..1 SSB.
- **M-001/M3-E030** [E/INT] anchor=`M-3:L56-L56` (ledger-line: DEEP-M-001.discovery.md:L256): WIRING consumes: K-6 bundles, G-8 verify, M-1/M-2 rows, K-1/K-4/H-9 numbers.
- **M-001/M3-E031** [E/INT] anchor=`M-3:L57-L57` (ledger-line: DEEP-M-001.discovery.md:L257): reachable_from: `elai org report` (real entrypoint); `elai export compliance --org` on seats.
- **M-001/M3-E032** [E/TEST] anchor=`M-3:L58-L59` (ledger-line: DEEP-M-001.discovery.md:L258): trigger: 3 fixture seat exports (one clean, one with Drift, one with a broken chain) -> `elai org report fixtures/exports --verify`.
- **M-001/M3-E033** [E/TEST] anchor=`M-3:L60-L61` (ledger-line: DEEP-M-001.discovery.md:L259): expect_ssb: (on the admin seat) the report emitted + verify PASS rows for 2 seats, RefusedSeat naming the broken one.
- **M-001/M3-E034** [E/TEST] anchor=`M-3:L62-L63` (ledger-line: DEEP-M-001.discovery.md:L260): Org bundle's every rollup number recomputes from the 2 valid constituent journals (GD-2 fleet-grain, byte-recomputable under ELAI_SEED fixtures).
- **M-001/M3-E035** [E/TEST] anchor=`M-3:L63-L64` (ledger-line: DEEP-M-001.discovery.md:L261): Broken-chain seat is REFUSED+NAMED (R2).
- **M-001/M3-E036** [E/TEST] anchor=`M-3:L64-L64` (ledger-line: DEEP-M-001.discovery.md:L262): Drift seat shows drift_status in the per-seat table.
- **M-001/M3-E037** [E/TEST] anchor=`M-3:L64-L65` (ledger-line: DEEP-M-001.discovery.md:L263): A seeded secret marker in a constituent bundle FAILS the rollup build (LOCK-2 arm).
- **M-001/M3-E038** [E/TEST] anchor=`M-3:L65-L66` (ledger-line: DEEP-M-001.discovery.md:L264): Grep of the org bundle for raw transcript bytes is EMPTY (MD-4 arm).
- **M-001/M3-E039** [E/OBS] anchor=`M-3:L68-L70` (ledger-line: DEEP-M-001.discovery.md:L265): Risk R1 stale exports: every constituent carries its journal head seq + date; the report prints per-seat freshness and flags exports older than the org-set window.
- **M-001/M3-E040** [E/OBS] anchor=`M-3:L71-L72` (ledger-line: DEEP-M-001.discovery.md:L266): Risk R2 metric gaming by seat selection: the report REQUIRES the org's seat roster (from M-1 activations) and names missing seats — absence is visible, not silent.
- **M-001/M3-E041** [E/PRIV] anchor=`M-3:L73-L74` (ledger-line: DEEP-M-001.discovery.md:L267): Risk R3 privacy regression via new fields: the added fields are ids + hashes only; the LOCK-2 self-test plus the K-6 secret-grep run on the AGGREGATE too.
- **M-001/M3-I001** [I/SURFACE] [SURFACE] anchor=`M-3:L38-L44` (ledger-line: DEEP-M-001.discovery.md:L268): Human markdown sheet and machine msgpack/toml are dual renderings of one org bundle rather than two independent aggregation paths.
- **M-001/M3-A001** [A/CFG] anchor=`M-3:L68-L70` (ledger-line: DEEP-M-001.discovery.md:L269): Org-set freshness window unit/default is named but not valued.
- **M-001/M3-A002** [A/FLOW] anchor=`M-3:L38-L40, L58-L59` (ledger-line: DEEP-M-001.discovery.md:L270): Behavior of `elai org report` without `--verify` is not specified.
- **M-001/M3-A003** [A/DATA] anchor=`M-3:L43-L44, L62-L64` (ledger-line: DEEP-M-001.discovery.md:L271): Exact machine file choice between msgpack and toml, output path, and partial-output after refusals are unresolved.
- **M-001/M3-A004** [A/OBS] anchor=`M-3:L58-L64` (ledger-line: DEEP-M-001.discovery.md:L272): Whether Drift on a verified seat affects an overall org verdict is not specified (fixture expects verify PASS for the Drift seat).
- **M-001/M3-A005** [A/EVENT] anchor=`M-3:L41-L42` (ledger-line: DEEP-M-001.discovery.md:L273): OrgReportEvent variants, if created, are not enumerated.

### Unit `M-4 — signed release channel + auto-updater + support bundle` — from M-001 (47 records)

- **M-001/M4-E001** [E/STATUS] anchor=`M-4:L1-L3` (ledger-line: DEEP-M-001.discovery.md:L283): M-4 is Track M, lever REACH + CANT-WRECK, grounding STRONG (gap G5/G6; B-43 defers the updater BY NAME at B-43:15), nature BUILD, effort M, WAVE M2.
- **M-001/M4-E002** [E/DEP] anchor=`M-4:L2-L4` (ledger-line: DEEP-M-001.discovery.md:L284): Depends on MA-4 (B-43 HostInstaller shipped; B-45 noted), M-2 (channel pin clause), G-9 (redaction).
- **M-001/M4-E003** [E/CAP] anchor=`M-4:L6-L9` (ledger-line: DEEP-M-001.discovery.md:L285): Ship the auto-updater B-43 explicitly deferred: signed release manifests, stable/beta channels with org pinning, staged rollout, and one-command rollback.
- **M-001/M4-E004** [E/CAP] anchor=`M-4:L9-L9` (ledger-line: DEEP-M-001.discovery.md:L286): Plus the missing support path: a G-9-redacted local diagnostic bundle.
- **M-001/M4-E005** [E/STATUS] anchor=`M-4:L11-L13` (ledger-line: DEEP-M-001.discovery.md:L287): Gap G5: B-43:15 names "separate plan auto-updater (delta updates + signature pinning)" — never authored anywhere B..L (grep-proven).
- **M-001/M4-E006** [E/INT] anchor=`M-4:L13-L14` (ledger-line: DEEP-M-001.discovery.md:L288): B-43's blake3-manifest + refuse-on-mismatch pattern (B-43:161-163, :343) is the verification core to REUSE.
- **M-001/M4-E007** [E/STATUS] anchor=`M-4:L15-L16` (ledger-line: DEEP-M-001.discovery.md:L289): Gap G6: G-9 redaction exists with no support-bundle consumer; support requests today would mean hand-copying logs (secret-leak roulette).
- **M-001/M4-E008** [E/CFG] anchor=`M-4:L19-L19` (ledger-line: DEEP-M-001.discovery.md:L290): LOCK-1 update CHECK is default-ON + non-blocking.
- **M-001/M4-E009** [E/SEC] anchor=`M-4:L19-L20` (ledger-line: DEEP-M-001.discovery.md:L291): Update APPLY is user-confirmed (interactive) or org-policy-scheduled (M-2 clause).
- **M-001/M4-E010** [E/LIMIT] anchor=`M-4:L20-L20` (ledger-line: DEEP-M-001.discovery.md:L292): Never mid-session, never silent binary swap.
- **M-001/M4-E011** [E/CFG] anchor=`M-4:L20-L21` (ledger-line: DEEP-M-001.discovery.md:L293): W-2 clean: check-on is the default state.
- **M-001/M4-E012** [E/DATA] anchor=`M-4:L22-L23` (ledger-line: DEEP-M-001.discovery.md:L294): LOCK-2 every release = a signed manifest {version, channel, per-artifact blake3, min-rollback version, revocation-list update (M-1 R1 mitigation lands HERE)}.
- **M-001/M4-E013** [E/FAIL] anchor=`M-4:L23-L24` (ledger-line: DEEP-M-001.discovery.md:L295): Mismatch = refuse, exactly the B-43 posture.
- **M-001/M4-E014** [E/DEFER] anchor=`M-4:L24-L24` (ledger-line: DEEP-M-001.discovery.md:L296): Delta updates are a LATER rider — v1 is whole-artifact (simplest correct thing).
- **M-001/M4-E015** [E/RECOVERY] anchor=`M-4:L25-L26` (ledger-line: DEEP-M-001.discovery.md:L297): LOCK-3 rollback keeps the last-good binary alongside (the J-3 last-good discipline applied to binaries).
- **M-001/M4-E016** [E/SURFACE] [SURFACE] anchor=`M-4:L26-L26` (ledger-line: DEEP-M-001.discovery.md:L298): `elai update rollback` restores it and verifies the hash before exec.
- **M-001/M4-E017** [E/PRIV] anchor=`M-4:L27-L27` (ledger-line: DEEP-M-001.discovery.md:L299): LOCK-4 the support bundle is written LOCALLY and never auto-sent (MD-4).
- **M-001/M4-E018** [E/DATA] anchor=`M-4:L27-L29` (ledger-line: DEEP-M-001.discovery.md:L300): Support bundle contents: versions, entitlement status (tier only, no key), config + pack hashes, `elai doctor` output, last-N journal rows through G-9 redaction.
- **M-001/M4-E019** [E/PRIV] anchor=`M-4:L29-L29` (ledger-line: DEEP-M-001.discovery.md:L301): Sending the support bundle anywhere is a human act.
- **M-001/M4-E020** [E/CAP] anchor=`M-4:L32-L32` (ledger-line: DEEP-M-001.discovery.md:L302): Task 1: release manifest format + signer (ops side) + verifier (client, G-8/B-43 core).
- **M-001/M4-E021** [E/INT] anchor=`M-4:L33-L33` (ledger-line: DEEP-M-001.discovery.md:L303): Module: `crates/cli/src/update.rs`.
- **M-001/M4-E022** [E/SURFACE] [SURFACE] anchor=`M-4:L33-L33` (ledger-line: DEEP-M-001.discovery.md:L304): CLI verbs: `elai update check|apply|rollback|channel`.
- **M-001/M4-E023** [E/EVENT] anchor=`M-4:L33-L34` (ledger-line: DEEP-M-001.discovery.md:L305): +ReleaseChannelEvent (Pattern-2: UpdateAvailable/Applied/RefusedHash/RolledBack) or prefer-reuse (W-9 at edit).
- **M-001/M4-E024** [E/CFG] anchor=`M-4:L35-L36` (ledger-line: DEEP-M-001.discovery.md:L306): Task 2 channels + org pin: stable/beta resolution; M-2 pack clause `update.channel_pin` wins over local.
- **M-001/M4-E025** [E/CFG] anchor=`M-4:L36-L37` (ledger-line: DEEP-M-001.discovery.md:L307): Staged rollout = a rollout_pct field the client hashes its seat_id against (deterministic bucket, no server state).
- **M-001/M4-E026** [E/DET] anchor=`M-4:L70-L71` (ledger-line: DEEP-M-001.discovery.md:L308): Bucket = blake3(seat_id) % 100 — deterministic, journal-explainable (GD-2), no server coordination.
- **M-001/M4-E027** [E/RECOVERY] anchor=`M-4:L38-L38` (ledger-line: DEEP-M-001.discovery.md:L309): Task 3 rollback: retain previous artifact + manifest; restore + hash-verify + re-exec.
- **M-001/M4-E028** [E/SURFACE] [SURFACE] anchor=`M-4:L39-L40` (ledger-line: DEEP-M-001.discovery.md:L310): Task 4 support bundle: `elai support bundle` -> `workflow/support/bundle-{date}.tar.zst`, G-9-redacted.
- **M-001/M4-E029** [E/SURFACE] [SURFACE] anchor=`M-4:L40-L40` (ledger-line: DEEP-M-001.discovery.md:L311): Support bundle prints a content summary so the user sees what it holds BEFORE sharing.
- **M-001/M4-E030** [E/DEP] anchor=`M-4:L42-L46` (ledger-line: DEEP-M-001.discovery.md:L312): COMPOSES-WITH: B-43 HostInstaller + blake3-manifest pattern (installer stays the fresh-install path; updater is its sibling, sharing verification code); B-45 (dashboard artifact rides the same manifest); M-2 (channel pin + scheduled apply); G-9 (bundle redaction); doctor (bundle content); M-1 (revocation-list distribution).
- **M-001/M4-E031** [E/DATA] anchor=`M-4:L48-L50` (ledger-line: DEEP-M-001.discovery.md:L313): WIRING produces: update.rs (check/apply/rollback/channel), release-manifest format + ops signing runbook, `elai support bundle`; +0..1 SSB.
- **M-001/M4-E032** [E/INT] anchor=`M-4:L51-L51` (ledger-line: DEEP-M-001.discovery.md:L314): WIRING consumes: B-43 verification core, M-2 pack clause, G-9, doctor, G-8 signing.
- **M-001/M4-E033** [E/INT] anchor=`M-4:L52-L53` (ledger-line: DEEP-M-001.discovery.md:L315): reachable_from: `elai update *` + `elai support bundle` CLI (real entrypoints); boot-time non-blocking check.
- **M-001/M4-E034** [E/TEST] anchor=`M-4:L54-L55` (ledger-line: DEEP-M-001.discovery.md:L316): trigger: fixture release server dir with v(N+1) manifest (one artifact's blake3 deliberately wrong) -> `elai update check && elai update apply`.
- **M-001/M4-E035** [E/TEST] anchor=`M-4:L56-L57` (ledger-line: DEEP-M-001.discovery.md:L317): expect_ssb: UpdateAvailable row; RefusedHash row on the bad artifact; after fixing the fixture, Applied row; `elai update rollback` -> RolledBack + prior binary hash-verified.
- **M-001/M4-E036** [E/TEST] anchor=`M-4:L58-L59` (ledger-line: DEEP-M-001.discovery.md:L318): Corrupted-artifact fixture is REFUSED before any file lands (R2).
- **M-001/M4-E037** [E/TEST] anchor=`M-4:L59-L59` (ledger-line: DEEP-M-001.discovery.md:L319): Clean fixture applies + rollback restores the exact prior binary (hash match).
- **M-001/M4-E038** [E/TEST] anchor=`M-4:L59-L60` (ledger-line: DEEP-M-001.discovery.md:L320): An org pack pinning stable makes a beta manifest invisible to the seat (M-2 arm).
- **M-001/M4-E039** [E/TEST] anchor=`M-4:L60-L62` (ledger-line: DEEP-M-001.discovery.md:L321): `elai support bundle` on a fixture session with a seeded AWS key + email produces a bundle whose secret-grep is EMPTY (G-9 arm) and prints its content summary.
- **M-001/M4-E040** [E/RECOVERY] anchor=`M-4:L64-L66` (ledger-line: DEEP-M-001.discovery.md:L322): Risk R1 self-update on Windows (running exe cannot replace itself): standard rename-swap-on-next-start pattern; the applied version activates on restart — stated in the UX, not hidden.
- **M-001/M4-E041** [E/SEC] anchor=`M-4:L67-L69` (ledger-line: DEEP-M-001.discovery.md:L323): Risk R2 compromised release key = fleet-wide code push: same key posture as M-1/M-2 (offline ceremony); the manifest's min-rollback + revocation list is also the recovery lever. Recorded residue (GD-5).
- **M-001/M4-E042** [E/DET] anchor=`M-4:L70-L71` (ledger-line: DEEP-M-001.discovery.md:L324): Risk R3 staged-rollout determinism: bucket = blake3(seat_id) % 100.
- **M-001/M4-I001** [I/CFG] anchor=`M-4:L19-L20, L35-L36, L45-L45` (ledger-line: DEEP-M-001.discovery.md:L325): Org-policy-scheduled apply is implied to be an M-2 pack clause peer of `update.channel_pin`, but schedule field names are not listed.
- **M-001/M4-A001** [A/CFG] anchor=`M-4:L19-L21, L35-L37` (ledger-line: DEEP-M-001.discovery.md:L326): Polling interval, failure backoff, download transport, and channel-switch authorization are unspecified.
- **M-001/M4-A002** [A/DATA] anchor=`M-4:L27-L29, L39-L40` (ledger-line: DEEP-M-001.discovery.md:L327): last-N journal row count and exact tar.zst layout internals are unspecified.
- **M-001/M4-A003** [A/SURFACE] [SURFACE] anchor=`M-4:L64-L66` (ledger-line: DEEP-M-001.discovery.md:L328): UX wording for Windows restart-activation is required but not drafted.
- **M-001/M4-A004** [A/EVENT] anchor=`M-4:L33-L34` (ledger-line: DEEP-M-001.discovery.md:L329): Prefer-reuse vs new ReleaseChannelEvent is unresolved at W-9 edit time.

### Unit `M-5 — commercial packaging + verifier OSS + trust continuity` — from M-001 (47 records)

- **M-001/M5-E001** [E/STATUS] anchor=`M-5:L1-L4` (ledger-line: DEEP-M-001.discovery.md:L339): M-5 is Track M, lever SELLABLE + AUDITABLE, grounding STRONG (gap G7; GTM decision on file: project_gtm_playbook_2026 "open-source ONLY the verifier, engine stays closed, escrow for bus-factor"), nature E-23-class docs + ONE crate extraction, effort S-M.
- **M-001/M5-E002** [E/STATUS] anchor=`M-5:L4-L5` (ledger-line: DEEP-M-001.discovery.md:L340): WAVE M1 (a,d) / WAVE M3 (b,c).
- **M-001/M5-E003** [E/DEP] anchor=`M-5:L5-L5` (ledger-line: DEEP-M-001.discovery.md:L341): Depends on K-1 claims lint live; (a) needs G-2/G-8 verify paths stable.
- **M-001/M5-E004** [E/CAP] anchor=`M-5:L7-L9` (ledger-line: DEEP-M-001.discovery.md:L342): Execute commercial decisions that already exist as strategy but have NO executing session in B..L.
- **M-001/M5-E005** [E/CAP] anchor=`M-5:L9-L9` (ledger-line: DEEP-M-001.discovery.md:L343): Publish the standalone OSS verifier (the distribution + anti-rug-pull leg).
- **M-001/M5-E006** [E/COMP] anchor=`M-5:L9-L10` (ledger-line: DEEP-M-001.discovery.md:L344): Write the legal wrapper (EULA/DPA/pricing) under the K-1 claims lint.
- **M-001/M5-E007** [E/COMP] anchor=`M-5:L10-L10` (ledger-line: DEEP-M-001.discovery.md:L345): Publish the continuity/escrow statement.
- **M-001/M5-E008** [E/CFG] anchor=`M-5:L11-L11` (ledger-line: DEEP-M-001.discovery.md:L346): Pin the seat-tier matrix as DATA that M-1 reads.
- **M-001/M5-E009** [E/CAP] anchor=`M-5:L14-L15` (ledger-line: DEEP-M-001.discovery.md:L347): Slice (a) VERIFIER OSS EXTRACTION [WAVE M1 — the one build]: extract standalone `elai-verify` crate/binary.
- **M-001/M5-E010** [E/INT] anchor=`M-5:L15-L16` (ledger-line: DEEP-M-001.discovery.md:L348): elai-verify wraps: verify-attestation (G-8) + `rra verify` action-stream re-check (G-2) + the K-6 bundle conformance check + M-3 org-bundle verify.
- **M-001/M5-E011** [E/LIMIT] anchor=`M-5:L16-L17` (ledger-line: DEEP-M-001.discovery.md:L349): Zero engine deps (the extraction IS the proof the verify path has no engine entanglement — a compile-time boundary test).
- **M-001/M5-E012** [E/COMP] anchor=`M-5:L17-L18` (ledger-line: DEEP-M-001.discovery.md:L350): License MIT-or-Apache (user picks at execution); repo published; the engine repo stays closed.
- **M-001/M5-E013** [E/CAP] anchor=`M-5:L18-L20` (ledger-line: DEEP-M-001.discovery.md:L351): Every evidence artifact ELAI emits becomes third-party-checkable without buying anything — the GTM wedge's credibility leg.
- **M-001/M5-E014** [E/COMP] anchor=`M-5:L21-L24` (ledger-line: DEEP-M-001.discovery.md:L352): Slice (b) LEGAL WRAPPER [WAVE M3]: EULA (engine) + license-split statement (engine EULA vs verifier OSS) + DPA template + refund/termination language for the seat model.
- **M-001/M5-E015** [E/PRIV] anchor=`M-5:L22-L23` (ledger-line: DEEP-M-001.discovery.md:L353): DPA grounds what a seat processes locally and what the mint service sees: org_id + payment metadata ONLY — grounded in M-1 LOCK-2.
- **M-001/M5-E016** [E/COMP] anchor=`M-5:L24-L24` (ledger-line: DEEP-M-001.discovery.md:L354): Drafts for counsel review — authoring, not legal advice; flagged for the user.
- **M-001/M5-E017** [E/COMP] anchor=`M-5:L25-L26` (ledger-line: DEEP-M-001.discovery.md:L355): Slice (c) TRUST CONTINUITY STATEMENT [WAVE M3]: solo-maintainer answer (focus-rescope contradiction #2).
- **M-001/M5-E018** [E/COMP] anchor=`M-5:L26-L28` (ledger-line: DEEP-M-001.discovery.md:L356): Source-escrow-on-lapse commitment OR a license-converts-to-OSS-on-abandonment clause — user decision at execution, both drafted.
- **M-001/M5-E019** [E/COMP] anchor=`M-5:L27-L28` (ledger-line: DEEP-M-001.discovery.md:L357): Key-ceremony/custody note (M-1/M-2/M-4 keys).
- **M-001/M5-E020** [E/SURFACE] [SURFACE] anchor=`M-5:L28-L29` (ledger-line: DEEP-M-001.discovery.md:L358): Support-expectation page (what a seat gets; honest solo-operation posture, GD-5 — no fake SLA).
- **M-001/M5-E021** [E/CFG] anchor=`M-5:L30-L31` (ledger-line: DEEP-M-001.discovery.md:L359): Slice (d) TIER MATRIX AS DATA [WAVE M1]: tiers.toml Free / Pro / Team rows -> feature keys M-1's tier() gates + K-24's credit grants read.
- **M-001/M5-E022** [E/CFG] anchor=`M-5:L31-L32` (ledger-line: DEEP-M-001.discovery.md:L360): Pricing NUMBERS live here once and nowhere else; every rendered surface reads this file.
- **M-001/M5-E023** [E/CLAIM] anchor=`M-5:L32-L33` (ledger-line: DEEP-M-001.discovery.md:L361): Seed framing per GTM memory (team/agency $39-49 class) tagged [EXTERNAL-GTM] until K-24's measured $/resolved-task lands (MD-5).
- **M-001/M5-E024** [E/CLAIM] anchor=`M-5:L36-L38` (ledger-line: DEEP-M-001.discovery.md:L362): LOCK-1 MD-5/KD-1: ALL copy in (b)/(d) passes the K-1 claims lint (extended with a pricing recognizer).
- **M-001/M5-E025** [E/CLAIM] anchor=`M-5:L37-L38` (ledger-line: DEEP-M-001.discovery.md:L363): No unmeasured superlative, no "$20=$200"-class claim before K-24's measured number, every external figure tagged.
- **M-001/M5-E026** [E/INT] anchor=`M-5:L39-L41` (ledger-line: DEEP-M-001.discovery.md:L364): LOCK-2 the verifier is extraction, NOT reimplementation (GD-1): the crate re-exports the same verify code paths the engine uses.
- **M-001/M5-E027** [E/FAIL] anchor=`M-5:L40-L41` (ledger-line: DEEP-M-001.discovery.md:L365): A fork of verify logic = drift = the exact failure the attestation exists to prevent. One source of verify truth.
- **M-001/M5-E028** [E/LIMIT] anchor=`M-5:L42-L43` (ledger-line: DEEP-M-001.discovery.md:L366): LOCK-3 the Article-50-certified hook stays KILLED (GTM record; K-6's "designed to support" wording is the ceiling).
- **M-001/M5-E029** [E/DEP] anchor=`M-5:L45-L48` (ledger-line: DEEP-M-001.discovery.md:L367): COMPOSES-WITH: G-8 attestation verify, G-2 rra verify, K-6 conformance, M-3 org verify (the four legs the OSS binary wraps); K-1 dossier + claims lint; K-7 brief (gains the verifier-OSS line); M-1 (tiers.toml consumer); GTM memory decisions (executed, not re-litigated).
- **M-001/M5-E030** [E/DATA] anchor=`M-5:L50-L52` (ledger-line: DEEP-M-001.discovery.md:L368): WIRING produces: elai-verify crate/binary (published), EULA/DPA/continuity drafts under docs/strategy/commercial/, tiers.toml, claims-lint pricing recognizer extension.
- **M-001/M5-E031** [E/INT] anchor=`M-5:L53-L53` (ledger-line: DEEP-M-001.discovery.md:L369): WIRING consumes: G-8/G-2/K-6/M-3 verify paths, K-1 lint, GTM decision records.
- **M-001/M5-E032** [E/INT] anchor=`M-5:L54-L55` (ledger-line: DEEP-M-001.discovery.md:L370): reachable_from: `elai-verify` standalone binary (real entrypoint, engine-free); tiers.toml read by `elai license status`.
- **M-001/M5-E033** [E/SURFACE] [SURFACE] anchor=`M-5:L54-L55, L62-L63` (ledger-line: DEEP-M-001.discovery.md:L371): `elai license status` renders tier names/features from tiers.toml.
- **M-001/M5-E034** [E/TEST] anchor=`M-5:L56-L57` (ledger-line: DEEP-M-001.discovery.md:L372): trigger: on a CLEAN machine (no engine): `elai-verify attestation fixtures/run.attestation && elai-verify rra fixtures/run.rra`.
- **M-001/M5-E035** [E/EVENT] anchor=`M-5:L58-L58` (ledger-line: DEEP-M-001.discovery.md:L373): expect_ssb: none (standalone binary journals nothing; E-23-class legs have no SSB).
- **M-001/M5-E036** [E/TEST] anchor=`M-5:L59-L60` (ledger-line: DEEP-M-001.discovery.md:L374): Standalone binary verifies fixture attestation + RRA + org bundle on an engine-free machine (PASS) and catches a one-byte tamper (FAIL, R2).
- **M-001/M5-E037** [E/TEST] anchor=`M-5:L60-L61` (ledger-line: DEEP-M-001.discovery.md:L375): The crate compiles with ZERO engine-crate deps (the boundary test).
- **M-001/M5-E038** [E/TEST] anchor=`M-5:L61-L62` (ledger-line: DEEP-M-001.discovery.md:L376): A seeded pricing superlative in the tier copy FAILS the lint (MD-5 arm).
- **M-001/M5-E039** [E/TEST] anchor=`M-5:L62-L63` (ledger-line: DEEP-M-001.discovery.md:L377): `elai license status` renders tier names/features from tiers.toml (fixture, M-1 join).
- **M-001/M5-E040** [E/FAIL] anchor=`M-5:L65-L67` (ledger-line: DEEP-M-001.discovery.md:L378): Risk R1 verifier extraction reveals engine entanglement in verify paths: that is a FINDING, not a blocker — the refactor to a clean boundary is in-scope for (a) and improves the engine.
- **M-001/M5-E041** [E/COMP] anchor=`M-5:L68-L70` (ledger-line: DEEP-M-001.discovery.md:L379): Risk R2 legal drafts treated as final: every (b)/(c) artifact carries a counsel-review-required banner; the user ships nothing external without review (house rule: user reviews before any external use, same as K-7).
- **M-001/M5-E042** [E/CLAIM] anchor=`M-5:L71-L72` (ledger-line: DEEP-M-001.discovery.md:L380): Risk R3 tiers.toml drift vs marketing copy: the lint's pricing recognizer greps rendered copy against tiers.toml values — one source of pricing truth, mechanically enforced.
- **M-001/M5-I001** [I/SURFACE] [SURFACE] anchor=`M-5:L28-L29, L50-L52` (ledger-line: DEEP-M-001.discovery.md:L381): Support-expectation page is a published commercial documentation surface rather than a runtime panel; no CLI verb is named for it.
- **M-001/M5-A001** [A/COMP] anchor=`M-5:L26-L28` (ledger-line: DEEP-M-001.discovery.md:L382): User must choose escrow-on-lapse vs convert-to-OSS-on-abandonment; both are drafted but neither is selected.
- **M-001/M5-A002** [A/COMP] anchor=`M-5:L17-L18` (ledger-line: DEEP-M-001.discovery.md:L383): MIT vs Apache license choice for the verifier is deferred to execution.
- **M-001/M5-A003** [A/CFG] anchor=`M-5:L30-L33` (ledger-line: DEEP-M-001.discovery.md:L384): Exact Free/Pro/Team feature-key matrix and numeric prices are data in tiers.toml and not fixed in this plan.
- **M-001/M5-A004** [A/SURFACE] [SURFACE] anchor=`M-5:L15-L16, L56-L60` (ledger-line: DEEP-M-001.discovery.md:L385): Subcommand grammar for org-bundle verify under elai-verify is implied by the four-leg wrap but not typed as a verb list beyond attestation and rra triggers.

### Unit `M-6 — evidence format as a versioned open standard (EEF v1)` — from M-001 (53 records)

- **M-001/M6-E001** [E/STATUS] anchor=`M-6:L1-L5` (ledger-line: DEEP-M-001.discovery.md:L395): M-6 is Track M, lever REACH + AUDITABLE, grounding STRONG (spec-eval 2026-07-05 §5 ADD-1), nature BUILD-compose + docs, effort M, WAVE M3.
- **M-001/M6-E002** [E/DEP] anchor=`M-6:L5-L6` (ledger-line: DEEP-M-001.discovery.md:L396): Depends on M-5a (elai-verify extracted), G-2/G-8/K-6 emit paths stable.
- **M-001/M6-E003** [E/EVIDENCE] anchor=`M-6:L2-L4` (ledger-line: DEEP-M-001.discovery.md:L397): Grounding cites SLSA/in-toto format-first precedent as [training, unsearched — re-verify at execution].
- **M-001/M6-E004** [E/STATUS] anchor=`M-6:L7-L8` (ledger-line: DEEP-M-001.discovery.md:L398): PROOF_SPINE: none directly (distribution amplifier for the spine's numbers); user-acked 2026-07-05 as spine-exempt.
- **M-001/M6-E005** [E/CAP] anchor=`M-6:L10-L13` (ledger-line: DEEP-M-001.discovery.md:L399): M-5 open-sources the verifier BINARY; M-6 publishes what the binary verifies: a versioned, machine-readable EVIDENCE FORMAT specification ("EEF v1" — ELAI Evidence Format; final name at execution).
- **M-001/M6-E006** [E/DATA] anchor=`M-6:L13-L14` (ledger-line: DEEP-M-001.discovery.md:L400): Coverage: RRA bundle, attestation predicate, compliance bundle, and claim-receipt row shapes.
- **M-001/M6-E007** [E/CAP] anchor=`M-6:L14-L15` (ledger-line: DEEP-M-001.discovery.md:L401): Third-party tools can EMIT ELAI-verifiable evidence and elai-verify becomes the reference verifier of a format rather than the private tool of one engine.
- **M-001/M6-E008** [E/CAP] anchor=`M-6:L15-L17` (ledger-line: DEEP-M-001.discovery.md:L402): Converts the moat from "our binary" to "our grammar": the one ecosystem play a single-author project can defend.
- **M-001/M6-E009** [E/CAP] anchor=`M-6:L17-L18` (ledger-line: DEEP-M-001.discovery.md:L403): Counter-move to K-0's kill-switch ("GitHub ships native evidence -> pivot to be-compatible"): publishing first makes ELAI the thing to be compatible WITH.
- **M-001/M6-E010** [E/CAP] anchor=`M-6:L21-L21` (ledger-line: DEEP-M-001.discovery.md:L404): Slice (a) SCHEMA EXTRACTION [the one build]: generate the spec's machine half FROM THE SHIPPED TYPES.
- **M-001/M6-E011** [E/DATA] anchor=`M-6:L21-L24` (ledger-line: DEEP-M-001.discovery.md:L405): Schema files: JSON-Schema for JSON surfaces; a documented msgpack layout for the RRA; derived from the same Rust structs the engine serializes (schemars-class derivation; verify the crate per anti-slopsquatting at execution), never hand-written twice.
- **M-001/M6-E012** [E/DATA] anchor=`M-6:L24-L26` (ledger-line: DEEP-M-001.discovery.md:L406): Covers: the RRA bundle (G-2 action stream + seed + hardware class + fingerprint + evidence manifest + blake3 root).
- **M-001/M6-E013** [E/DATA] anchor=`M-6:L26-L26` (ledger-line: DEEP-M-001.discovery.md:L407): Covers: the in-toto v1 attestation predicate (G-8).
- **M-001/M6-E014** [E/DATA] anchor=`M-6:L26-L26` (ledger-line: DEEP-M-001.discovery.md:L408): Covers: the K-6 compliance bundle.
- **M-001/M6-E015** [E/DATA] anchor=`M-6:L26-L28` (ledger-line: DEEP-M-001.discovery.md:L409): Covers: verdict/receipt row shapes K-3/K-11 render (gate verdict, stdout-hash-bound test outcome, DSR band, anchored finding).
- **M-001/M6-E016** [E/LIMIT] anchor=`M-6:L28-L28` (ledger-line: DEEP-M-001.discovery.md:L410): One source of schema truth = the code.
- **M-001/M6-E017** [E/COMP] anchor=`M-6:L29-L29` (ledger-line: DEEP-M-001.discovery.md:L411): Slice (b) PROSE SPEC + CONFORMANCE FIXTURES at docs/strategy/eef/ (or a sibling repo at publication).
- **M-001/M6-E018** [E/COMP] anchor=`M-6:L29-L31` (ledger-line: DEEP-M-001.discovery.md:L412): Versioned prose spec: field semantics, integrity rules — what a verifier MUST check, what missing evidence MUST mean: fail, per R1.
- **M-001/M6-E019** [E/TEST] anchor=`M-6:L31-L32` (ledger-line: DEEP-M-001.discovery.md:L413): Conformance fixture pack: valid bundles + one-byte-tampered + missing-field negatives.
- **M-001/M6-E020** [E/SURFACE] [SURFACE] anchor=`M-6:L32-L33` (ledger-line: DEEP-M-001.discovery.md:L414): `elai-verify conform <artifact>` (an M-5 binary subcommand extension) validates any artifact against a pinned spec version.
- **M-001/M6-E021** [E/STATE] anchor=`M-6:L34-L35` (ledger-line: DEEP-M-001.discovery.md:L415): Slice (c) VERSIONING + GOVERNANCE: append-only field evolution locked by a golden file (the E-45 action-type golden-lock pattern applied to the schema).
- **M-001/M6-E022** [E/DATA] anchor=`M-6:L35-L36` (ledger-line: DEEP-M-001.discovery.md:L416): Every emitted artifact carries spec_version.
- **M-001/M6-E023** [E/STATE] anchor=`M-6:L36-L36` (ledger-line: DEEP-M-001.discovery.md:L417): A breaking change requires a major version + a migration note.
- **M-001/M6-E024** [E/SEC] anchor=`M-6:L36-L37` (ledger-line: DEEP-M-001.discovery.md:L418): The golden lock lives in the frozen gate-manifest set (R8) so a run cannot mutate the format it is judged in.
- **M-001/M6-E025** [E/COMP] anchor=`M-6:L38-L39` (ledger-line: DEEP-M-001.discovery.md:L419): Slice (d) PUBLICATION + THIRD-PARTY EMITTER STORY: publish spec + schemas + fixtures alongside the elai-verify repo (same license family as M-5a; user picks).
- **M-001/M6-E026** [E/COMP] anchor=`M-6:L39-L40` (ledger-line: DEEP-M-001.discovery.md:L420): Short "emit EEF from your tool" doc with one worked example (a minimal third-party emitter producing a conformant verdict row).
- **M-001/M6-E027** [E/CLAIM] anchor=`M-6:L41-L43` (ledger-line: DEEP-M-001.discovery.md:L421): Claims discipline: it is a "published format" until >=1 EXTERNAL emitter exists; the words "open standard" / "industry standard" are KD-1-gated on that adoption evidence.
- **M-001/M6-E028** [E/CLAIM] anchor=`M-6:L42-L43` (ledger-line: DEEP-M-001.discovery.md:L422): The K-1 lint gains an adoption-claim recognizer.
- **M-001/M6-E029** [E/LIMIT] anchor=`M-6:L46-L48` (ledger-line: DEEP-M-001.discovery.md:L423): LOCK-1 ONE SOURCE OF SCHEMA TRUTH (GD-1): machine schemas are DERIVED from the engine's serialization types; a hand-forked schema = drift = the exact failure attestation exists to prevent (mirrors M-5 LOCK-2).
- **M-001/M6-E030** [E/CLAIM] anchor=`M-6:L49-L50` (ledger-line: DEEP-M-001.discovery.md:L424): LOCK-2 KD-1 ON ADOPTION: no "standard" claim without an external emitter; no ecosystem-size claim ever; every comparison figure [EXTERNAL].
- **M-001/M6-E031** [E/LIMIT] anchor=`M-6:L51-L51` (ledger-line: DEEP-M-001.discovery.md:L425): LOCK-3 the ENGINE STAYS CLOSED (GTM record): only the format, fixtures, and verifier are open.
- **M-001/M6-E032** [E/PRIV] anchor=`M-6:L52-L53` (ledger-line: DEEP-M-001.discovery.md:L426): Publishing the schema must not leak engine internals beyond what emitted artifacts already show; the G-9 redaction guarantees are restated in the spec as emitter REQUIREMENTS.
- **M-001/M6-E033** [E/FAIL] anchor=`M-6:L54-L56` (ledger-line: DEEP-M-001.discovery.md:L427): LOCK-4 FAIL-CLOSED SEMANTICS ARE NORMATIVE: the spec text makes R1 (missing evidence = FAIL, UNVOUCHED never renders clean) a conformance requirement for verifiers, not a recommendation — a permissive third-party verifier is non-conformant by definition.
- **M-001/M6-E034** [E/DEP] anchor=`M-6:L58-L62` (ledger-line: DEEP-M-001.discovery.md:L428): COMPOSES-WITH: M-5a elai-verify (gains `conform`; the reference verifier), G-2 RRA + G-8 attestation + K-6 bundle (the emit paths — the schemas describe them, never fork them), E-45 golden-lock pattern (schema evolution), K-1 claims lint (adoption-claim recognizer extension), K-3/K-11 (the verdict/receipt shapes rendered into PRs), gate-manifest R8 (the golden lock joins the frozen set).
- **M-001/M6-E035** [E/DATA] anchor=`M-6:L64-L67` (ledger-line: DEEP-M-001.discovery.md:L429): WIRING produces: docs/strategy/eef/ spec v1 (prose + derived schemas + conformance fixtures), the `elai-verify conform` subcommand, the schema golden-lock file (added to .claude/gate-manifest.txt), the K-1 lint adoption-claim recognizer, the third-party emitter example.
- **M-001/M6-E036** [E/INT] anchor=`M-6:L68-L69` (ledger-line: DEEP-M-001.discovery.md:L430): WIRING consumes: the engine serialization types (derivation source), G-2/G-8/K-6 emit paths, M-5a binary, E-45 golden-lock pattern, K-1 lint.
- **M-001/M6-E037** [E/INT] anchor=`M-6:L70-L71` (ledger-line: DEEP-M-001.discovery.md:L431): reachable_from: `elai-verify conform <artifact>` on an engine-free machine (real entrypoint); the golden lock enforced by gate_freeze_check.py on every run (R8 path).
- **M-001/M6-E038** [E/TEST] anchor=`M-6:L72-L73` (ledger-line: DEEP-M-001.discovery.md:L432): trigger: on a CLEAN machine: `elai-verify conform fixtures/run.rra && elai-verify conform fixtures/tampered.rra; test $? -ne 0`.
- **M-001/M6-E039** [E/EVENT] anchor=`M-6:L74-L74` (ledger-line: DEEP-M-001.discovery.md:L433): expect_ssb: none (standalone verifier + docs; E-23-class legs journal nothing).
- **M-001/M6-E040** [E/TEST] anchor=`M-6:L75-L76` (ledger-line: DEEP-M-001.discovery.md:L434): A conformant fixture PASSes and a one-byte-tampered / missing-field fixture FAILS on an engine-free machine (R2).
- **M-001/M6-E041** [E/TEST] anchor=`M-6:L76-L77` (ledger-line: DEEP-M-001.discovery.md:L435): Derived schemas byte-match a regeneration from the engine types (LOCK-1 drift test).
- **M-001/M6-E042** [E/TEST] anchor=`M-6:L77-L77` (ledger-line: DEEP-M-001.discovery.md:L436): A schema field removal trips the golden lock (R8).
- **M-001/M6-E043** [E/TEST] anchor=`M-6:L77-L79` (ledger-line: DEEP-M-001.discovery.md:L437): A seeded "industry standard" phrase in the spec/README FAILS the K-1 lint until the adoption row exists (LOCK-2 arm).
- **M-001/M6-E044** [E/TEST] anchor=`M-6:L79-L79` (ledger-line: DEEP-M-001.discovery.md:L438): The worked third-party example emits a row `elai-verify conform` accepts.
- **M-001/M6-E045** [E/FAIL] anchor=`M-6:L81-L84` (ledger-line: DEEP-M-001.discovery.md:L439): Risk R1 premature standardization: freezing v1 before the RRA/attestation surfaces stabilize forces a fast v2 and burns credibility. Mitigation: M-6 sits in WAVE M3 behind GATE M-0; v1 scope = only artifacts elai-verify already verifies today.
- **M-001/M6-E046** [E/PRIV] anchor=`M-6:L85-L86` (ledger-line: DEEP-M-001.discovery.md:L440): Risk R2 the schema leaks engine detail: review pass against LOCK-3 with the G-9 redaction spec; the conformance fixtures are generated from redacted runs only.
- **M-001/M6-E047** [E/CLAIM] anchor=`M-6:L87-L89` (ledger-line: DEEP-M-001.discovery.md:L441): Risk R3 nobody adopts (the honest default): the format still pays for itself as internal rigor (the golden lock + conform subcommand harden ELAI's own emit paths); adoption claims stay gated so non-adoption never becomes a false claim.
- **M-001/M6-I001** [I/DATA] anchor=`M-6:L32-L36` (ledger-line: DEEP-M-001.discovery.md:L442): Because every emitted artifact carries spec_version and conform pins a version, multi-version coexistence is implied even though negotiation protocol is not defined.
- **M-001/M6-A001** [A/STATUS] anchor=`M-6:L12-L13` (ledger-line: DEEP-M-001.discovery.md:L443): Final EEF product name is deferred to execution.
- **M-001/M6-A002** [A/DATA] anchor=`M-6:L21-L28, L34-L37` (ledger-line: DEEP-M-001.discovery.md:L444): Media type, extension namespaces, unknown-field handling, and compatibility window are unspecified.
- **M-001/M6-A003** [A/STATE] anchor=`M-6:L34-L36` (ledger-line: DEEP-M-001.discovery.md:L445): Whether fields can be deprecated or made optional inside a major version is not stated beyond append-only + major-for-breaking.
- **M-001/M6-A004** [A/CLAIM] anchor=`M-6:L41-L43` (ledger-line: DEEP-M-001.discovery.md:L446): Adoption row schema, authority, and verification process are unspecified.
- **M-001/M6-A005** [A/COMP] anchor=`M-6:L29-L29, L38-L39` (ledger-line: DEEP-M-001.discovery.md:L447): Publication as docs/strategy/eef/ vs sibling repo is unresolved.

## Cross-source interactions (M-001/IX-*)

Interaction records: **24** (`M-001/IX-001`–`M-001/IX-024`). Reference source atoms; not double-counted as features in deep footer, counted once here as relational records.

- **M-001/IX-001** [E/IX] (DEEP-M-001.discovery.md:L455): GATE M-0 is the single execution gate for every M session: K A-1..A-4 + K-3 DONE and K-24 L1..L4 live before any M code lands. | evidence: M0-E046–M0-E047; M1-E002
- **M-001/IX-002** [E/IX] (DEEP-M-001.discovery.md:L456): Wave order is M1 (M-1, M-5a, M-5d) then M2 (M-2, M-4) then M3 (M-3, M-5b, M-5c, M-6). | evidence: M0-E048–M0-E050; M5-E002; M6-E001
- **M-001/IX-003** [E/IX] (DEEP-M-001.discovery.md:L457): MD-1..MD-5 are letter-wide invariants: commerce fail-open to Free, offline-verifiable commerce, one mint endpoint, seat privacy, claims discipline on money. | evidence: M0-E017–M0-E031; M1-E011–M1-E014; M3-E005; M5-E024–M5-E025
- **M-001/IX-004** [E/IX] (DEEP-M-001.discovery.md:L458): M-1 is the keystone: its `tier()` read gates K-24 Team credit grants, M-2 pack apply, and M-3 org report. | evidence: M1-E025–M1-E028; M2-E002, M2-E026; M3-E002
- **M-001/IX-005** [E/IX] (DEEP-M-001.discovery.md:L459): Entitlement is org-granular with seat slots; honest over-activation is only enforced as visibility in M-3's roster honesty leg, not as a hard offline concurrency wall. | evidence: M1-E017–M1-E020, M1-E049; M3-E027, M3-E040
- **M-001/IX-006** [E/IX] (DEEP-M-001.discovery.md:L460): G-8 signing/verify core is shared by M-1 entitlements, M-2 packs, M-3 --verify, M-4 manifests, and M-5/M-6 verification legs. | evidence: M1-E007–M1-E008; M2-E010; M3-E022, M3-E028; M4-E020; M5-E010; M6-E013
- **M-001/IX-007** [E/IX] (DEEP-M-001.discovery.md:L461): M-2 pack apply is a Team feature and feeds M-4 via `update.channel_pin` plus scheduled apply; Drift rows feed M-3. | evidence: M2-E004, M2-E015, M2-E022, M2-E025–M2-E026; M4-E002, M4-E009, M4-E024, M4-E038; M3-E002, M3-E010
- **M-001/IX-008** [E/IX] (DEEP-M-001.discovery.md:L462): M-3 extends K-6 rather than forking it, requires G-9 redaction precondition, and reuses G-2/G-8 for chain verify. | evidence: M3-E010–M3-E013, M3-E028
- **M-001/IX-009** [E/IX] (DEEP-M-001.discovery.md:L463): M-4 distributes M-1's revocation-list mitigation inside release manifests and reuses B-43 refuse-on-mismatch for artifacts. | evidence: M1-E050; M4-E006, M4-E012–M4-E013, M4-E030
- **M-001/IX-010** [E/IX] (DEEP-M-001.discovery.md:L464): M-5d tiers.toml is the single pricing/feature matrix M-1 tier() and K-24 credit grants read; M-5a extracts the four verify legs including M-3 org-bundle verify. | evidence: M1-E010, M1-E029, M1-E042; M5-E009–M5-E010, M5-E021–M5-E022, M5-E033
- **M-001/IX-011** [E/IX] (DEEP-M-001.discovery.md:L465): M-6 depends on M-5a and extends the same binary with `conform`, while schemas describe (never fork) G-2/G-8/K-6 emit paths. | evidence: M6-E002, M6-E005, M6-E020, M6-E034
- **M-001/IX-012** [E/IX] (DEEP-M-001.discovery.md:L466): Claims discipline spans the letter: MD-5/K-1 lint on pricing/EULA (M-5), adoption-claim recognizer (M-6), and ban on unmeasured monetary claims before K-24. | evidence: M0-E030–M0-E031, M0-E047; M5-E024–M5-E025, M5-E042; M6-E027–M6-E028, M6-E043
- **M-001/IX-013** [E/IX] (DEEP-M-001.discovery.md:L467): Privacy chain: J-59 isolation (MA-3) gates M-3; G-9 redacts exports/support; rollups and support bundles never auto-send and never carry raw transcripts. | evidence: M0-E027–M0-E029, M0-E034; M3-E005, M3-E012–M3-E013, M3-E038; M4-E017–M4-E019, M4-E039
- **M-001/IX-014** [E/IX] (DEEP-M-001.discovery.md:L468): Fail-open commerce vs fail-closed safety: MD-1 degrades entitlement to Free while safety gates, pack weaken blocks, forged refuse, and missing-evidence FAIL remain fail-closed. | evidence: M0-E017–M0-E019; M1-E011–M1-E012, M1-E047–M1-E048; M2-E024, M2-E033; M6-E033
- **M-001/IX-015** [E/IX] (DEEP-M-001.discovery.md:L469): Offline-first commerce: license service is never a runtime dependency; mint/renew/revoke only; pack and report travel by file/git. | evidence: M0-E021–M0-E025; M1-E013–M1-E014, M1-E031; M2-E006; M3-E007
- **M-001/IX-016** [E/IX] (DEEP-M-001.discovery.md:L470): SSB budget: mandatory +EntitlementEvent and +OrgPolicyEvent; optional OrgReportEvent/ReleaseChannelEvent prefer-reuse; M-5 and M-6 add zero. | evidence: M0-E055–M0-E057; M1-E024; M2-E020; M3-E025; M4-E023; M5-E035; M6-E039
- **M-001/IX-017** [E/IX] (DEEP-M-001.discovery.md:L471): Enterprise rider M-7+ (SSO/SAML/SCIM/RBAC/hosted console) is demand-gated and deliberately out of M-1..M-6; M-2 file pack is the org story until then. | evidence: M0-E026, M0-E051–M0-E054; M2-E008
- **M-001/IX-018** [E/IX] (DEEP-M-001.discovery.md:L472): Section-A pointers MA-1..MA-7 sequence existing work M must not re-author; MA-3/MA-4/MA-6 are hard deps of M-3/M-4, MA-1 of M-1. | evidence: M0-E032–M0-E039; M1-E002; M3-E002; M4-E002
- **M-001/IX-019** [E/IX] (DEEP-M-001.discovery.md:L473): Key-compromise posture is shared: offline ceremony for mint/pack/release keys; recovery levers include M-4 revocation-list distribution and min-rollback. | evidence: M1-E032, M1-E050; M2-E038; M4-E012, M4-E041; M5-E019
- **M-001/IX-020** [E/IX] (DEEP-M-001.discovery.md:L474): Deterministic client-side math: ClockProvider for expiry; blake3(seat_id)%100 for rollout; blake3 machine-scoped seat_id; SeededClock fixtures. | evidence: M1-E014, M1-E018, M1-E039, M1-E047; M4-E025–M4-E026, M4-E042
- **M-001/IX-021** [E/IX] [SURFACE] (DEEP-M-001.discovery.md:L475): Human-facing commercial surfaces span license CLI, status banner, install-end activation, org policy CLI, boot warning, org report human/machine sheets, update CLI, support bundle summary, elai-verify binary, support-expectation page, EEF prose/spec, and tiers.toml-driven license status. | evidence: M1-E022, M1-E037, M1-E040; M2-E016, M2-E018; M3-E019–M3-E020, M3-E026; M4-E022, M4-E028–M4-E029; M5-E020, M5-E033; M6-E017, M6-E020, M6-E026
- **M-001/IX-022** [I/IX] (DEEP-M-001.discovery.md:L476): Mint service ops ledger (M-1) plus counsel-bannered legal drafts (M-5) together form the operator-side commercial audit trail, separate from seat SSB. | evidence: M1-E033–M1-E034; M5-E014–M5-E016, M5-E041
- **M-001/IX-023** [E/IX] (DEEP-M-001.discovery.md:L477): M-6 R8 golden lock and M-2 gate-manifest pin both freeze evaluator/policy artifacts a run cannot rewrite; they operate at format and org-policy layers respectively. | evidence: M2-E009–M2-E010, M2-E022; M6-E021, M6-E024, M6-E037
- **M-001/IX-024** [E/IX] (DEEP-M-001.discovery.md:L478): First M execution generates `_grounding/M-GROUNDING-MAP.md` and re-greps live anchors before code; M-6 also re-verifies unsearched SLSA/in-toto precedent. | evidence: M0-E058–M0-E059; M6-E003

## Ambiguity and conflict register (M-001/AC-*)

Register records: **12**.

- **M-001/AC-001** [ambiguity/status/AC] (DEEP-M-001.discovery.md:L484): GATE M-0 prerequisite DONE criteria for K A-1..A-4, K-3, K-24 L1..L4 are named by reference only. | refs: M0-A001
- **M-001/AC-002** [ambiguity/event/AC] (DEEP-M-001.discovery.md:L485): OrgReportEvent and ReleaseChannelEvent create-vs-reuse remains an edit-time W-9 decision. | refs: M0-A002; M3-A005; M4-A004
- **M-001/AC-003** [conflict/surface/AC] (DEEP-M-001.discovery.md:L486): Summary SettingsWindow license-status line is not established by numbered M-1; source surfaces are CLI status + ExpiredGrace status banner + B-43 install-end activation. | refs: M1-C001
- **M-001/AC-004** [ambiguity/data/AC] (DEEP-M-001.discovery.md:L487): seat_id salt storage/rotation, operator ledger schema, and exact tiers.toml feature inventory are open. | refs: M1-A001, M1-A003, M1-A005
- **M-001/AC-005** [ambiguity/surface/AC] (DEEP-M-001.discovery.md:L488): ExpiredGrace status banner placement/copy and M-2 boot warning presentation are unspecified. | refs: M1-A002; M2-A003
- **M-001/AC-006** [ambiguity/flow/AC] (DEEP-M-001.discovery.md:L489): Opportunistic re-check interval/transport and deactivate local-state/SSB semantics are incomplete. | refs: M1-A004; M1-I001
- **M-001/AC-007** [ambiguity/data/AC] (DEEP-M-001.discovery.md:L490): OrgPolicyPack serialization, stale threshold, git multi-pack workflow, and pack rotation are open. | refs: M2-A001–M2-A004
- **M-001/AC-008** [ambiguity/flow/AC] (DEEP-M-001.discovery.md:L491): `elai org report` without `--verify`, machine msgpack-vs-toml choice, partial output after refusals, and Drift overall-verdict interaction are open. | refs: M3-A001–M3-A004
- **M-001/AC-009** [ambiguity/config/AC] (DEEP-M-001.discovery.md:L492): Updater polling/backoff/transport, support last-N size, Windows restart UX copy, and org-scheduled-apply field names are open. | refs: M4-I001, M4-A001–M4-A003
- **M-001/AC-010** [ambiguity/comp/AC] (DEEP-M-001.discovery.md:L493): Continuity choice (escrow vs convert-to-OSS), verifier license (MIT vs Apache), and exact elai-verify org-bundle verb list remain open. | refs: M5-A001–M5-A004
- **M-001/AC-011** [ambiguity/data/AC] (DEEP-M-001.discovery.md:L494): Final EEF name, media type, deprecation inside major versions, adoption-row authority, and docs-vs-sibling-repo publication path are open. | refs: M6-A001–M6-A005
- **M-001/AC-012** [ambiguity/claim/AC] (DEEP-M-001.discovery.md:L495): External GTM $39-49 framing remains [EXTERNAL-GTM]-tagged and SLSA/in-toto precedent remains unsearched until re-verified. | refs: M1-E009; M5-E023; M6-E003

## Summary deltas (M-001/SD-*)

Delta records: **39**. Relational ADD/CORRECT/QUALIFY against `13-M-SUMMARY_INDEX.discovery.md`.

- **M-001/SD-M0-001** [Δ] (DEEP-M-001.discovery.md:L505): **ADD** full MA-1..MA-7 Section-A pointer set (summary used MA-3/MA-4/MA-6 by name only; originals enumerate all seven targets and GD-1 non-reauthor rule). | summary-anchor: 13-M:L8, L460-L463, L660-L662 | source-atoms: M0-E032–M0-E039, M0-I001
- **M-001/SD-M0-002** [Δ] (DEEP-M-001.discovery.md:L506): **ADD** exact enterprise demand gate: >=3 paying orgs ask OR a signed deal is blocked; re-proposing org SaaS early is P0-grade miss. | summary-anchor: 13-M:L70, L141-L142, L504-L505 | source-atoms: M0-E051–M0-E054
- **M-001/SD-M0-003** [Δ] (DEEP-M-001.discovery.md:L507): **ADD** live SSB baseline anchors ssb.rs:1508 / action_type_lock.rs:775 and zero-SSB rule for M-5. | summary-anchor: 13-M:L9-L10, L548-L551 | source-atoms: M0-E055–M0-E057
- **M-001/SD-M0-004** [Δ] (DEEP-M-001.discovery.md:L508): **ADD** first-execution grounding map generation path `_grounding/M-GROUNDING-MAP.md` and [EXTERNAL] pricing confinement to M-5. | summary-anchor: 13-M:L5-L7 | source-atoms: M0-E058–M0-E060
- **M-001/SD-M0-005** [Δ] (DEEP-M-001.discovery.md:L509): **CORRECT** enterprise-rider numbering: originals authoritatively renumber informal M-6+ to M-7+ with M-6 = evidence format; summary already noted the header correction. | summary-anchor: 13-M:L2-L4, L141, L585 | source-atoms: M0-E004, M0-E051
- **M-001/SD-M0-006** [Δ] (DEEP-M-001.discovery.md:L510): **ADD** exact wave membership including M-5a/d in M1, M-2/M-4 in M2, M-3/M-5b/c/M-6 in M3. | summary-anchor: 13-M:L463 | source-atoms: M0-E048–M0-E050
- **M-001/SD-M1-001** [Δ] (DEEP-M-001.discovery.md:L516): **ADD** full Entitlement field set including issued_at_ms, expires_at_ms, grace_days=14 default, features[], and seat_id = blake3(machine-scoped salt). | summary-anchor: 13-M:L91-L108, L586-L588 | source-atoms: M1-E017–M1-E018
- **M-001/SD-M1-002** [Δ] (DEEP-M-001.discovery.md:L517): **ADD** local store path `~/.elai/entitlement/`, activate argument form `<file|key>`, mint operator ledger, and ops-runbook-not-crate rule. | summary-anchor: 13-M:L111-L113, L586-L587 | source-atoms: M1-E022–M1-E023, M1-E031–M1-E034
- **M-001/SD-M1-003** [Δ] (DEEP-M-001.discovery.md:L518): **ADD** opportunistic online re-check, revocation-on-next-check-only, and ExpiredGrace status banner. | summary-anchor: 13-M:L114-L116, L587 | source-atoms: M1-E035–M1-E038
- **M-001/SD-M1-004** [Δ] (DEEP-M-001.discovery.md:L519): **ADD** MoR class examples (Paddle/Lemon-Squeezy), E-50 provider-choice fork, and seats.toml composition. | summary-anchor: 13-M:L107-L109, L492 | source-atoms: M1-E015–M1-E016, M1-E039
- **M-001/SD-M1-005** [Δ] (DEEP-M-001.discovery.md:L520): **ADD** exact Free-tier definition (everything through L not Team in tiers.toml), TD-3 no-LLM-tier-adjudication, and three named tier() consumers. | summary-anchor: 13-M:L114-L119 | source-atoms: M1-E025–M1-E030
- **M-001/SD-M1-006** [Δ] (DEEP-M-001.discovery.md:L521): **ADD** acceptance trigger commands, SeededClock expiry arm, offline network-disabled activation, and key-leak/clock-tamper residues. | summary-anchor: 13-M:L134-L142, L558-L562 | source-atoms: M1-E044–M1-E052
- **M-001/SD-M1-007** [Δ] (DEEP-M-001.discovery.md:L522): **CORRECT/REMOVE** summary SettingsWindow license-status pairing: not present in numbered M-1; substitute CLI status + status banner + install-end activation. | summary-anchor: 13-M:L133-L134, L441, L508, L589 | source-atoms: M1-C001, M1-E022, M1-E037, M1-E040
- **M-001/SD-M2-001** [Δ] (DEEP-M-001.discovery.md:L528): **ADD** pack discovery order: repo `.elai/org-policy.signed` then `~/.elai/org/`, plus apply-in-order consumer map (3F/E-39/gate-manifest/G-9/M-4). | summary-anchor: 13-M:L151-L160, L590-L591 | source-atoms: M2-E021–M2-E022
- **M-001/SD-M2-002** [Δ] (DEEP-M-001.discovery.md:L529): **ADD** weaken gate covers rule edit / cap raise / allowlist widen with deterministic pack-clause naming; org GREP rule fires in SHADOW. | summary-anchor: 13-M:L159-L163, L180-L185 | source-atoms: M2-E024, M2-E031, M2-E037
- **M-001/SD-M2-003** [Δ] (DEEP-M-001.discovery.md:L530): **ADD** signer-identity pin refusing foreign-org valid packs; three-layer precedence contract test; fpr<5% SHADOW promotion discipline. | summary-anchor: 13-M:L590-L591 | source-atoms: M2-E037–M2-E039
- **M-001/SD-M2-004** [Δ] (DEEP-M-001.discovery.md:L531): **ADD** fixture details: spend cap 0.05, stable channel pin, E-39 row names org clause not local config. | summary-anchor: 13-M:L180-L185 | source-atoms: M2-E030–M2-E031
- **M-001/SD-M2-005** [Δ] (DEEP-M-001.discovery.md:L532): **ADD** K-5 GateIntensity policy-table hash may be pack-pinned; L-13 FS-2 non-executable doctrine at org grain. | summary-anchor: 13-M:L167-L169 | source-atoms: M2-E011, M2-E026
- **M-001/SD-M3-001** [Δ] (DEEP-M-001.discovery.md:L538): **ADD** dual render outputs: human markdown sheet + machine msgpack/toml as the K-7 org answer. | summary-anchor: 13-M:L201-L211, L450 | source-atoms: M3-E026, M3-I001
- **M-001/SD-M3-002** [Δ] (DEEP-M-001.discovery.md:L539): **ADD** freshness fields (journal head seq + date), org-set window flagging, and required M-1 seat roster with named missing seats. | summary-anchor: 13-M:L576-L577, L592-L593 | source-atoms: M3-E039–M3-E040
- **M-001/SD-M3-003** [Δ] (DEEP-M-001.discovery.md:L540): **ADD** serde(default) no-migration extension, prefer-reuse ComplianceExported/G-8 SSB guidance, and aggregate secret-grep on rollup output. | summary-anchor: 13-M:L198-L201, L221-L223 | source-atoms: M3-E013, M3-E018, M3-E025, M3-E041
- **M-001/SD-M3-004** [Δ] (DEEP-M-001.discovery.md:L541): **ADD** exact trigger fixture trio path and admin-seat expect_ssb wording for 2 PASS + named RefusedSeat. | summary-anchor: 13-M:L224-L230 | source-atoms: M3-E032–M3-E038
- **M-001/SD-M3-005** [Δ] (DEEP-M-001.discovery.md:L542): **QUALIFY** summary’s `--verify` as present on the report command while behavior without the flag remains unspecified in originals. | summary-anchor: 13-M:L201-L202, L576, L592 | source-atoms: M3-E022–M3-E023, M3-A002
- **M-001/SD-M4-001** [Δ] (DEEP-M-001.discovery.md:L548): **ADD** complete release-manifest field set: version, channel, per-artifact blake3, min-rollback version, revocation-list update. | summary-anchor: 13-M:L238-L241, L594 | source-atoms: M4-E012
- **M-001/SD-M4-002** [Δ] (DEEP-M-001.discovery.md:L549): **ADD** exact ReleaseChannelEvent arms UpdateAvailable/Applied/RefusedHash/RolledBack (summary said variants unspecified). | summary-anchor: 13-M:L273-L274, L549 | source-atoms: M4-E023, M4-E035
- **M-001/SD-M4-003** [Δ] (DEEP-M-001.discovery.md:L550): **ADD** support bundle path `workflow/support/bundle-{date}.tar.zst`, entitlement status tier-only (no key), and printed pre-share content summary. | summary-anchor: 13-M:L258-L267 | source-atoms: M4-E018, M4-E028–M4-E029
- **M-001/SD-M4-004** [Δ] (DEEP-M-001.discovery.md:L551): **ADD** pack clause name `update.channel_pin`, rollout_pct field, Windows rename-swap-on-next-start restart UX, and boot-time non-blocking check reachability. | summary-anchor: 13-M:L244-L247, L594-L595 | source-atoms: M4-E024–M4-E026, M4-E033, M4-E040
- **M-001/SD-M4-005** [Δ] (DEEP-M-001.discovery.md:L552): **ADD** fixture acceptance sequence including deliberate wrong blake3, Applied after fix, and RolledBack hash verification. | summary-anchor: 13-M:L275-L279 | source-atoms: M4-E034–M4-E039
- **M-001/SD-M5-001** [Δ] (DEEP-M-001.discovery.md:L558): **ADD** GTM decision source project_gtm_playbook_2026 and explicit MIT-or-Apache user-pick for verifier license. | summary-anchor: 13-M:L285-L290 | source-atoms: M5-E001, M5-E012
- **M-001/SD-M5-002** [Δ] (DEEP-M-001.discovery.md:L559): **ADD** refund/termination language, counsel-review-required banner house rule, and support-expectation page (no fake SLA). | summary-anchor: 13-M:L290-L308 | source-atoms: M5-E014, M5-E016, M5-E020, M5-E041
- **M-001/SD-M5-003** [Δ] (DEEP-M-001.discovery.md:L560): **ADD** key-ceremony/custody note covering M-1/M-2/M-4 keys and K-7 brief gaining the verifier-OSS line. | summary-anchor: 13-M:L300-L301, L317-L318 | source-atoms: M5-E019, M5-E029
- **M-001/SD-M5-004** [Δ] (DEEP-M-001.discovery.md:L561): **ADD** exact clean-machine trigger commands for attestation + rra, pricing-recognizer drift grep against tiers.toml, and entanglement-as-finding risk. | summary-anchor: 13-M:L324-L330 | source-atoms: M5-E034–M5-E042
- **M-001/SD-M5-005** [Δ] (DEEP-M-001.discovery.md:L562): **ADD** docs path docs/strategy/commercial/ and `elai license status` as tiers.toml consumer entrypoint. | summary-anchor: 13-M:L320-L321, L330 | source-atoms: M5-E030, M5-E032–M5-E033
- **M-001/SD-M6-001** [Δ] (DEEP-M-001.discovery.md:L568): **ADD** schema derivation mechanics: JSON-Schema + documented RRA msgpack layout, schemars-class, anti-slopsquatting check. | summary-anchor: 13-M:L348-L350, L601 | source-atoms: M6-E010–M6-E011
- **M-001/SD-M6-002** [Δ] (DEEP-M-001.discovery.md:L569): **ADD** exact RRA field inventory (action stream, seed, hardware class, fingerprint, evidence manifest, blake3 root) and receipt fields (gate verdict, stdout-hash-bound outcome, DSR band, anchored finding). | summary-anchor: 13-M:L341-L346 | source-atoms: M6-E012–M6-E015
- **M-001/SD-M6-003** [Δ] (DEEP-M-001.discovery.md:L570): **ADD** golden lock path into `.claude/gate-manifest.txt`, gate_freeze_check.py enforcement, and major-version + migration-note rule. | summary-anchor: 13-M:L356-L358 | source-atoms: M6-E021–M6-E024, M6-E035, M6-E037
- **M-001/SD-M6-004** [Δ] (DEEP-M-001.discovery.md:L571): **ADD** G-9 redaction restated as emitter REQUIREMENTS; permissive third-party verifiers are non-conformant by definition. | summary-anchor: 13-M:L354-L355 | source-atoms: M6-E032–M6-E033
- **M-001/SD-M6-005** [Δ] (DEEP-M-001.discovery.md:L572): **ADD** K-0 kill-switch counter-move rationale, spine-exempt PROOF_SPINE note, and "pays for itself as internal rigor" non-adoption residual. | summary-anchor: 13-M:L358-L365 | source-atoms: M6-E004, M6-E008–M6-E009, M6-E047
- **M-001/SD-M6-006** [Δ] (DEEP-M-001.discovery.md:L573): **ADD** exact conform trigger on run.rra + tampered.rra, regeneration byte-match, field-removal golden-lock trip, and industry-standard lint arm. | summary-anchor: 13-M:L370-L377 | source-atoms: M6-E038–M6-E044

---

## Summary record (13-M-SUMMARY_INDEX)

Source file: `docs/thefinalglance/feature-discovery/13-M-SUMMARY_INDEX.discovery.md`
Extracted: inventory=304, relational/extra=179, subtotal=483

### Summary scope header (preserved)

- **SUMMARY/HDR-001** [STATUS] (13-M:L3-L12): Primary source M-SUMMARY_INDEX.md 146/146 lines; orientation-only ELAI-SPEC-OVERVIEW; 6/6 sessions; MD-1..MD-5; originals existence-checked but not read (summary-bounded).
- **SUMMARY/HDR-002** [STATUS] (13-M:L15-L25): Coverage map for M-wide and M-1..M-6 with waves and dependency gates.

### SUMMARY unit `M-wide product boundary, doctrine, and sequencing` (42 records)

- **SUMMARY/MX-E001** [CAP] anchor=`M:L1` (13-M-SUMMARY_INDEX.discovery.md:L44): M is a single-track commercial-shipping / multi-seat letter closing B through L.
- **SUMMARY/MX-E002** [STATUS] anchor=`M:L1` (13-M-SUMMARY_INDEX.discovery.md:L45): The index covers six M sessions.
- **SUMMARY/MX-E003** [STATUS] anchor=`M:L2-L4` (13-M-SUMMARY_INDEX.discovery.md:L46): M-6 was appended on 2026-07-05 from spec-eval ADD-1 following explicit user direction.
- **SUMMARY/MX-E004** [STATUS] anchor=`M:L2-L4` (13-M-SUMMARY_INDEX.discovery.md:L47): The enterprise rider’s informal M-6+ slot was renumbered M-7+ in M-0_INDEX.
- **SUMMARY/MX-E005** [STATUS] anchor=`M:L5-L6` (13-M-SUMMARY_INDEX.discovery.md:L48): The original M wave was generated 2026-07-02 with the main LLM in-band via paper-to-plan phases 4-5 and no paper.
- **SUMMARY/MX-E006** [STATUS] anchor=`M:L5-L7` (13-M-SUMMARY_INDEX.discovery.md:L49): The multi-seat gap analysis was grep-proven.
- **SUMMARY/MX-E007** [STATUS] anchor=`M:L7-L8` (13-M-SUMMARY_INDEX.discovery.md:L50): K-10’s logged deviation applies to authoring only; M execution remains behind the K-10 order and M-0 gate.
- **SUMMARY/MX-E008** [STATUS] anchor=`M:L9` (13-M-SUMMARY_INDEX.discovery.md:L51): M-0_INDEX owns doctrine MD-1..MD-5, Section-A pointers MA-1..MA-7, and the enterprise rider.
- **SUMMARY/MX-E009** [STATUS] anchor=`M:L10` (13-M-SUMMARY_INDEX.discovery.md:L52): The recorded live stamp is 268/577 from K on 2026-06-26, with a W-9 re-grep required at edit.
- **SUMMARY/MX-E010** [OBS] anchor=`M:L10` (13-M-SUMMARY_INDEX.discovery.md:L53): M expects only +2 through +4 net-new SSB event families/rows across the letter.
- **SUMMARY/MX-E011** [CAP] anchor=`M:L13-L14` (13-M-SUMMARY_INDEX.discovery.md:L54): B through L are treated as a sellable single-seat product; M makes it sellable to a company.
- **SUMMARY/MX-E012** [CAP] anchor=`M:L14-L15` (13-M-SUMMARY_INDEX.discovery.md:L55): M is deliberately a minimal, offline-first organization layer.
- **SUMMARY/MX-E013** [CAP] anchor=`M:L14-L15` (13-M-SUMMARY_INDEX.discovery.md:L56): M explicitly does not become an organization SaaS.
- **SUMMARY/MX-E014** [INT] anchor=`M:L15-L16` (13-M-SUMMARY_INDEX.discovery.md:L57): M is approximately 60% composition of K-24, J-49/J-59, B-43, K-6/K-7, G-8/G-9, and K-1.
- **SUMMARY/MX-E015** [CAP] anchor=`M:L16-L17` (13-M-SUMMARY_INDEX.discovery.md:L58): Net-new commerce machinery includes entitlement.
- **SUMMARY/MX-E016** [CAP] anchor=`M:L16-L17` (13-M-SUMMARY_INDEX.discovery.md:L59): Net-new commerce machinery includes an organization policy pack.
- **SUMMARY/MX-E017** [CAP] anchor=`M:L16-L17` (13-M-SUMMARY_INDEX.discovery.md:L60): Net-new commerce machinery includes a fleet rollup.
- **SUMMARY/MX-E018** [CAP] anchor=`M:L16-L17` (13-M-SUMMARY_INDEX.discovery.md:L61): Net-new commerce machinery includes an updater.
- **SUMMARY/MX-E019** [CAP] anchor=`M:L16-L17` (13-M-SUMMARY_INDEX.discovery.md:L62): Net-new commerce machinery includes a legal wrapper.
- **SUMMARY/MX-E020** [FALLBACK] anchor=`M:L17-L18` (13-M-SUMMARY_INDEX.discovery.md:L63): MD-1 is the never-brick rule: commerce fails open to the Free tier.
- **SUMMARY/MX-E021** [SAFE] anchor=`M:L17-L19` (13-M-SUMMARY_INDEX.discovery.md:L64): MD-1 does not weaken or disable safety gates.
- **SUMMARY/MX-E022** [STATUS] anchor=`M:L17-L19` (13-M-SUMMARY_INDEX.discovery.md:L65): MD-1 is the one recorded inversion of the R1 fail-closed rule.
- **SUMMARY/MX-E023** [CAP] anchor=`M:L19` (13-M-SUMMARY_INDEX.discovery.md:L66): MD-2 requires commerce to be offline-verifiable, reusing the G-8 pattern.
- **SUMMARY/MX-E024** [SAFE] anchor=`M:L19` (13-M-SUMMARY_INDEX.discovery.md:L67): The license service must never be a runtime dependency.
- **SUMMARY/MX-E025** [CAP] anchor=`M:L19-L20` (13-M-SUMMARY_INDEX.discovery.md:L68): MD-3 permits exactly one first-party cloud endpoint, the mint endpoint.
- **SUMMARY/MX-E026** [FLOW] anchor=`M:L20-L21` (13-M-SUMMARY_INDEX.discovery.md:L69): Organization policy and evidence travel by file or git rather than a hosted control plane.
- **SUMMARY/MX-E027** [DEFER] anchor=`M:L20-L21` (13-M-SUMMARY_INDEX.discovery.md:L70): SSO and SCIM are demand-gated rather than part of the minimal first-party cloud.
- **SUMMARY/MX-E028** [PRIV] anchor=`M:L21-L22` (13-M-SUMMARY_INDEX.discovery.md:L71): MD-4 defaults to per-seat isolation by composing J-59.
- **SUMMARY/MX-E029** [PRIV] anchor=`M:L21-L22` (13-M-SUMMARY_INDEX.discovery.md:L72): Fleet rollups contain numbers and hashes.
- **SUMMARY/MX-E030** [PRIV] anchor=`M:L21-L22` (13-M-SUMMARY_INDEX.discovery.md:L73): Fleet rollups never contain transcripts.
- **SUMMARY/MX-E031** [CLAIM] anchor=`M:L22-L23` (13-M-SUMMARY_INDEX.discovery.md:L74): MD-5 applies K-1 claims lint to all pricing and EULA copy.
- **SUMMARY/MX-E032** [CLAIM] anchor=`M:L22-L23` (13-M-SUMMARY_INDEX.discovery.md:L75): Monetary value claims are blocked until K-24 supplies a measured number.
- **SUMMARY/MX-E033** [CAP] anchor=`M:L135-L138` (13-M-SUMMARY_INDEX.discovery.md:L76): At organization grain, the finished product is purchasable, policeable, and auditable without being an org SaaS.
- **SUMMARY/MX-E034** [FLOW] anchor=`M:L137-L138` (13-M-SUMMARY_INDEX.discovery.md:L77): Apart from the one mint endpoint, organization commerce, policy, and evidence use offline-verifiable files.
- **SUMMARY/MX-E035** [FALLBACK] anchor=`M:L138-L139` (13-M-SUMMARY_INDEX.discovery.md:L78): Commerce is the sole deliberately fail-open layer and degrades a seat rather than bricking it.
- **SUMMARY/MX-E036** [SAFE] anchor=`M:L138-L139` (13-M-SUMMARY_INDEX.discovery.md:L79): Every safety gate remains fail-closed even when commerce fails open.
- **SUMMARY/MX-E037** [AUDIT] anchor=`M:L139-L141` (13-M-SUMMARY_INDEX.discovery.md:L80): Organization answers are computed from seat journals, not vendor dashboards.
- **SUMMARY/MX-E038** [DEFER] anchor=`M:L141-L142` (13-M-SUMMARY_INDEX.discovery.md:L81): SSO, SAML, SCIM, RBAC, and a hosted console are demand-gated enterprise-rider capabilities.
- **SUMMARY/MX-E039** [CLAIM] anchor=`M:L141-L142` (13-M-SUMMARY_INDEX.discovery.md:L82): The demand-gated rider preserves the recorded agencies-not-enterprise go-to-market posture.
- **SUMMARY/MX-E040** [STATUS] anchor=`M:L142-L143` (13-M-SUMMARY_INDEX.discovery.md:L83): M is authored as the closing letter and executes last.
- **SUMMARY/MX-E041** [STATUS] anchor=`M:L143-L144` (13-M-SUMMARY_INDEX.discovery.md:L84): GATE M-0 holds every M session behind K A-1..A-4, K-3, and K-24 L1..L4.
- **SUMMARY/MX-E042** [CLAIM] anchor=`M:L143-L144` (13-M-SUMMARY_INDEX.discovery.md:L85): Selling before K-24’s measured number exists is forbidden by KD-1/MD-5 claim discipline.

### SUMMARY unit `M-1 — seat entitlement + purchase rail` (52 records)

- **SUMMARY/M1-E001** [STATUS] anchor=`M:L25-L26` (13-M-SUMMARY_INDEX.discovery.md:L91): M-1 is the M keystone, lever SELLABLE, STRONG-grounded by gaps G1/G2, BUILD, effort M-L, wave M1.
- **SUMMARY/M1-E002** [DATA] anchor=`M:L27-L28` (13-M-SUMMARY_INDEX.discovery.md:L92): Entitlements are signed.
- **SUMMARY/M1-E003** [SAFE] anchor=`M:L27-L28` (13-M-SUMMARY_INDEX.discovery.md:L93): Entitlement validity is verifiable offline.
- **SUMMARY/M1-E004** [DATA] anchor=`M:L27` (13-M-SUMMARY_INDEX.discovery.md:L94): Entitlement carries org_id.
- **SUMMARY/M1-E005** [DATA] anchor=`M:L27` (13-M-SUMMARY_INDEX.discovery.md:L95): Entitlement carries tier.
- **SUMMARY/M1-E006** [DATA] anchor=`M:L27` (13-M-SUMMARY_INDEX.discovery.md:L96): Entitlement carries seats_max.
- **SUMMARY/M1-E007** [DATA] anchor=`M:L27` (13-M-SUMMARY_INDEX.discovery.md:L97): Entitlement carries features.
- **SUMMARY/M1-E008** [DATA] anchor=`M:L27` (13-M-SUMMARY_INDEX.discovery.md:L98): Entitlement carries expiry.
- **SUMMARY/M1-E009** [DATA] anchor=`M:L27` (13-M-SUMMARY_INDEX.discovery.md:L99): Entitlement carries sig.
- **SUMMARY/M1-E010** [INT] anchor=`M:L27-L28` (13-M-SUMMARY_INDEX.discovery.md:L100): Entitlement signing reuses the G-8 signing core.
- **SUMMARY/M1-E011** [CLI] [SURFACE] anchor=`M:L28` (13-M-SUMMARY_INDEX.discovery.md:L101): The CLI exposes 'elai license activate'.
- **SUMMARY/M1-E012** [CLI] [SURFACE] anchor=`M:L28` (13-M-SUMMARY_INDEX.discovery.md:L102): The CLI exposes 'elai license status'.
- **SUMMARY/M1-E013** [CLI] [SURFACE] anchor=`M:L28` (13-M-SUMMARY_INDEX.discovery.md:L103): The CLI exposes 'elai license deactivate'.
- **SUMMARY/M1-E014** [LIFE] anchor=`M:L28-L29` (13-M-SUMMARY_INDEX.discovery.md:L104): An entitlement has a 14-day grace period.
- **SUMMARY/M1-E015** [LIFE] anchor=`M:L28-L29` (13-M-SUMMARY_INDEX.discovery.md:L105): Expiration after grace transitions the seat to DegradedToFree.
- **SUMMARY/M1-E016** [FALLBACK] anchor=`M:L29` (13-M-SUMMARY_INDEX.discovery.md:L106): Entitlement expiry never bricks the seat; it retains Free-tier operation.
- **SUMMARY/M1-E017** [SAFE] anchor=`M:L29` (13-M-SUMMARY_INDEX.discovery.md:L107): Entitlement degradation leaves safety gates untouched.
- **SUMMARY/M1-E018** [FLOW] anchor=`M:L29-L30` (13-M-SUMMARY_INDEX.discovery.md:L108): A merchant-of-record payment webhook sits outside ELAI.
- **SUMMARY/M1-E019** [COMP] anchor=`M:L29-L30` (13-M-SUMMARY_INDEX.discovery.md:L109): Keeping the payment webhook outside ELAI gives ELAI zero PCI handling under the inherited K-24 decision.
- **SUMMARY/M1-E020** [FLOW] anchor=`M:L29-L31` (13-M-SUMMARY_INDEX.discovery.md:L110): Payment fulfillment flows from the MoR webhook to the one first-party mint endpoint and then to seat activation.
- **SUMMARY/M1-E021** [ACTION] [SURFACE] anchor=`M:L30-L31` (13-M-SUMMARY_INDEX.discovery.md:L111): The mint endpoint can mint an entitlement.
- **SUMMARY/M1-E022** [ACTION] [SURFACE] anchor=`M:L30-L31` (13-M-SUMMARY_INDEX.discovery.md:L112): The mint endpoint can renew an entitlement.
- **SUMMARY/M1-E023** [ACTION] [SURFACE] anchor=`M:L30-L31` (13-M-SUMMARY_INDEX.discovery.md:L113): The mint endpoint can revoke an entitlement.
- **SUMMARY/M1-E024** [CAP] anchor=`M:L30-L31` (13-M-SUMMARY_INDEX.discovery.md:L114): The mint endpoint is limited to mint, renew, and revoke.
- **SUMMARY/M1-E025** [CFG] anchor=`M:L31` (13-M-SUMMARY_INDEX.discovery.md:L115): A tier() read determines commercial feature access.
- **SUMMARY/M1-E026** [INT] anchor=`M:L31` (13-M-SUMMARY_INDEX.discovery.md:L116): tier() gates K-24 credit grants.
- **SUMMARY/M1-E027** [INT] anchor=`M:L31-L32` (13-M-SUMMARY_INDEX.discovery.md:L117): tier() gates M-2 Team features.
- **SUMMARY/M1-E028** [INT] anchor=`M:L31-L32` (13-M-SUMMARY_INDEX.discovery.md:L118): tier() gates M-3 Team features.
- **SUMMARY/M1-E029** [CFG] anchor=`M:L32` (13-M-SUMMARY_INDEX.discovery.md:L119): Tier behavior is data-driven through M-5d tiers.toml under TD-3.
- **SUMMARY/M1-E030** [LIFE] anchor=`M:L32-L33` (13-M-SUMMARY_INDEX.discovery.md:L120): Seat counting is an honest client-side best-effort mechanism.
- **SUMMARY/M1-E031** [OBS] anchor=`M:L32-L33` (13-M-SUMMARY_INDEX.discovery.md:L121): Seat over-use is surfaced by M-3.
- **SUMMARY/M1-E032** [FAIL] anchor=`M:L32-L33, L41-L42` (13-M-SUMMARY_INDEX.discovery.md:L122): Over-activation is recorded rather than silently absorbed.
- **SUMMARY/M1-E033** [INT] anchor=`M:L34-L35` (13-M-SUMMARY_INDEX.discovery.md:L123): ClockProvider owns all entitlement-expiry math.
- **SUMMARY/M1-E034** [UI] [SURFACE] anchor=`M:L34-L35` (13-M-SUMMARY_INDEX.discovery.md:L124): B-43 offers activation at the end of installation.
- **SUMMARY/M1-E035** [INT] anchor=`M:L36` (13-M-SUMMARY_INDEX.discovery.md:L125): The planned integration module is crates/orchestrator/src/commerce/entitlement.rs.
- **SUMMARY/M1-E036** [OBS] anchor=`M:L36-L37` (13-M-SUMMARY_INDEX.discovery.md:L126): M-1 adds an EntitlementEvent family.
- **SUMMARY/M1-E037** [OBS] anchor=`M:L36-L37` (13-M-SUMMARY_INDEX.discovery.md:L127): EntitlementEvent has Activated.
- **SUMMARY/M1-E038** [OBS] anchor=`M:L36-L37` (13-M-SUMMARY_INDEX.discovery.md:L128): EntitlementEvent has Renewed.
- **SUMMARY/M1-E039** [OBS] anchor=`M:L36-L37` (13-M-SUMMARY_INDEX.discovery.md:L129): EntitlementEvent has ExpiredGrace.
- **SUMMARY/M1-E040** [OBS] anchor=`M:L36-L38` (13-M-SUMMARY_INDEX.discovery.md:L130): EntitlementEvent has DegradedToFree.
- **SUMMARY/M1-E041** [OBS] anchor=`M:L36-L38` (13-M-SUMMARY_INDEX.discovery.md:L131): EntitlementEvent has Revoked.
- **SUMMARY/M1-E042** [OBS] anchor=`M:L36-L38` (13-M-SUMMARY_INDEX.discovery.md:L132): EntitlementEvent has RefusedForged.
- **SUMMARY/M1-E043** [UI] [SURFACE] [superseded as target UI by SUP-001] anchor=`M:L37-L38` (13-M-SUMMARY_INDEX.discovery.md:L133): A license-status line is paired with SettingsWindow.
- **SUMMARY/M1-E044** [STATUS] [superseded as target UI by SUP-001] anchor=`M:L37-L38` (13-M-SUMMARY_INDEX.discovery.md:L134): Exact SettingsWindow presentation is deferred to edit time under W-4.
- **SUMMARY/M1-E045** [TEST] anchor=`M:L39` (13-M-SUMMARY_INDEX.discovery.md:L135): A Team entitlement fixture must activate offline.
- **SUMMARY/M1-E046** [TEST] anchor=`M:L39` (13-M-SUMMARY_INDEX.discovery.md:L136): Offline activation must emit Activated.
- **SUMMARY/M1-E047** [TEST] anchor=`M:L39` (13-M-SUMMARY_INDEX.discovery.md:L137): Offline activation must unlock Team features.
- **SUMMARY/M1-E048** [TEST] anchor=`M:L40` (13-M-SUMMARY_INDEX.discovery.md:L138): SeededClock expiry must produce ExpiredGrace followed by DegradedToFree.
- **SUMMARY/M1-E049** [TEST] anchor=`M:L40-L41` (13-M-SUMMARY_INDEX.discovery.md:L139): After degradation, a Free task must still complete green.
- **SUMMARY/M1-E050** [TEST] anchor=`M:L40-L41` (13-M-SUMMARY_INDEX.discovery.md:L140): After degradation, every safety gate must still fire.
- **SUMMARY/M1-E051** [TEST] anchor=`M:L41` (13-M-SUMMARY_INDEX.discovery.md:L141): A forged signature must be refused under R2.
- **SUMMARY/M1-E052** [TEST] anchor=`M:L41-L42` (13-M-SUMMARY_INDEX.discovery.md:L142): An over-activation fixture must produce a recorded signal.

### SUMMARY unit `M-2 — org policy pack + distribution + drift detection` (38 records)

- **SUMMARY/M2-E001** [STATUS] anchor=`M:L44-L45` (13-M-SUMMARY_INDEX.discovery.md:L148): M-2 has lever CANT-WRECK at organization grain, STRONG grounding from gap G3 and shipped 3F/R8/E-39, BUILD-compose, effort M, wave M2.
- **SUMMARY/M2-E002** [STATUS] anchor=`M:L45` (13-M-SUMMARY_INDEX.discovery.md:L149): M-2 depends on M-1.
- **SUMMARY/M2-E003** [DATA] anchor=`M:L46` (13-M-SUMMARY_INDEX.discovery.md:L150): The organization policy is one signed OrgPolicyPack.
- **SUMMARY/M2-E004** [FLOW] anchor=`M:L46` (13-M-SUMMARY_INDEX.discovery.md:L151): OrgPolicyPack is distributable as a file.
- **SUMMARY/M2-E005** [CFG] anchor=`M:L46-L47` (13-M-SUMMARY_INDEX.discovery.md:L152): The pack maps organization rules into an org-immune tier.
- **SUMMARY/M2-E006** [CFG] anchor=`M:L46-L47` (13-M-SUMMARY_INDEX.discovery.md:L153): The pack pins the gate manifest.
- **SUMMARY/M2-E007** [CFG] anchor=`M:L47` (13-M-SUMMARY_INDEX.discovery.md:L154): The pack carries E-39 spend caps.
- **SUMMARY/M2-E008** [CFG] anchor=`M:L47` (13-M-SUMMARY_INDEX.discovery.md:L155): The pack carries the B-37 allowlist.
- **SUMMARY/M2-E009** [CFG] anchor=`M:L47` (13-M-SUMMARY_INDEX.discovery.md:L156): The pack carries G-9 configuration.
- **SUMMARY/M2-E010** [CFG] anchor=`M:L47` (13-M-SUMMARY_INDEX.discovery.md:L157): The pack carries the M-4 release-channel pin.
- **SUMMARY/M2-E011** [LIFE] anchor=`M:L47` (13-M-SUMMARY_INDEX.discovery.md:L158): OrgPolicyPack is applied at boot.
- **SUMMARY/M2-E012** [SAFE] anchor=`M:L47-L48` (13-M-SUMMARY_INDEX.discovery.md:L159): A seat may add checks stricter than the organization pack.
- **SUMMARY/M2-E013** [SAFE] anchor=`M:L48` (13-M-SUMMARY_INDEX.discovery.md:L160): A seat may never weaken organization checks.
- **SUMMARY/M2-E014** [SAFE] anchor=`M:L48` (13-M-SUMMARY_INDEX.discovery.md:L161): The organization merge is monotone-tighten, lifting R8 to organization grain.
- **SUMMARY/M2-E015** [OBS] anchor=`M:L48-L49` (13-M-SUMMARY_INDEX.discovery.md:L162): Policy drift is detected.
- **SUMMARY/M2-E016** [UI] [SURFACE] anchor=`M:L48-L49` (13-M-SUMMARY_INDEX.discovery.md:L163): Policy drift is surfaced.
- **SUMMARY/M2-E017** [FAIL] anchor=`M:L49` (13-M-SUMMARY_INDEX.discovery.md:L164): Policy drift is not auto-corrected before GD-4.
- **SUMMARY/M2-E018** [SAFE] anchor=`M:L49-L50` (13-M-SUMMARY_INDEX.discovery.md:L165): OrgPolicyPack is data, never executable content.
- **SUMMARY/M2-E019** [FALLBACK] anchor=`M:L50` (13-M-SUMMARY_INDEX.discovery.md:L166): With no organization pack, behavior is byte-identical to current seat behavior.
- **SUMMARY/M2-E020** [INT] anchor=`M:L51` (13-M-SUMMARY_INDEX.discovery.md:L167): M-2 composes the 3F lifecycle and mutable tier.
- **SUMMARY/M2-E021** [INT] anchor=`M:L51-L52` (13-M-SUMMARY_INDEX.discovery.md:L168): M-2 composes gate_freeze_check and the gate manifest, which the pack pins.
- **SUMMARY/M2-E022** [INT] anchor=`M:L51-L52` (13-M-SUMMARY_INDEX.discovery.md:L169): M-2 composes E-39, K-5 policy-table hash, G-9, G-8 signing, and M-1 tier.
- **SUMMARY/M2-E023** [INT] anchor=`M:L53` (13-M-SUMMARY_INDEX.discovery.md:L170): The planned integration module is commerce/org_policy.rs.
- **SUMMARY/M2-E024** [CLI] [SURFACE] anchor=`M:L53` (13-M-SUMMARY_INDEX.discovery.md:L171): The CLI exposes 'elai org policy init'.
- **SUMMARY/M2-E025** [CLI] [SURFACE] anchor=`M:L53` (13-M-SUMMARY_INDEX.discovery.md:L172): The CLI exposes 'elai org policy sign'.
- **SUMMARY/M2-E026** [CLI] [SURFACE] anchor=`M:L53` (13-M-SUMMARY_INDEX.discovery.md:L173): The CLI exposes 'elai org policy verify'.
- **SUMMARY/M2-E027** [CLI] [SURFACE] anchor=`M:L53` (13-M-SUMMARY_INDEX.discovery.md:L174): The CLI exposes 'elai org policy show'.
- **SUMMARY/M2-E028** [OBS] anchor=`M:L53-L54` (13-M-SUMMARY_INDEX.discovery.md:L175): M-2 adds an OrgPolicyEvent family.
- **SUMMARY/M2-E029** [OBS] anchor=`M:L53-L54` (13-M-SUMMARY_INDEX.discovery.md:L176): OrgPolicyEvent has Applied.
- **SUMMARY/M2-E030** [OBS] anchor=`M:L53-L54` (13-M-SUMMARY_INDEX.discovery.md:L177): OrgPolicyEvent has Drift.
- **SUMMARY/M2-E031** [OBS] anchor=`M:L53-L54` (13-M-SUMMARY_INDEX.discovery.md:L178): OrgPolicyEvent has RefusedForged.
- **SUMMARY/M2-E032** [OBS] anchor=`M:L53-L54` (13-M-SUMMARY_INDEX.discovery.md:L179): OrgPolicyEvent has WeakenAttemptBlocked.
- **SUMMARY/M2-E033** [TEST] anchor=`M:L55` (13-M-SUMMARY_INDEX.discovery.md:L180): A fixture pack must boot into Applied.
- **SUMMARY/M2-E034** [TEST] anchor=`M:L55-L56` (13-M-SUMMARY_INDEX.discovery.md:L181): The applied organization cap must block a task and name the responsible pack clause.
- **SUMMARY/M2-E035** [TEST] anchor=`M:L56` (13-M-SUMMARY_INDEX.discovery.md:L182): A tampered pack must be refused.
- **SUMMARY/M2-E036** [TEST] anchor=`M:L56` (13-M-SUMMARY_INDEX.discovery.md:L183): A local cap raise must produce WeakenAttemptBlocked.
- **SUMMARY/M2-E037** [TEST] anchor=`M:L56-L57` (13-M-SUMMARY_INDEX.discovery.md:L184): A no-pack seat must be byte-identical to baseline.
- **SUMMARY/M2-E038** [TEST] anchor=`M:L57` (13-M-SUMMARY_INDEX.discovery.md:L185): A stale pack must produce Drift at boot.

### SUMMARY unit `M-3 — fleet evidence rollup` (40 records)

- **SUMMARY/M3-E001** [STATUS] anchor=`M:L59-L60` (13-M-SUMMARY_INDEX.discovery.md:L191): M-3 has lever AUDITABLE at organization grain, STRONG grounding from gap G4, BUILD-compose, effort M, wave M3.
- **SUMMARY/M3-E002** [STATUS] anchor=`M:L60` (13-M-SUMMARY_INDEX.discovery.md:L192): M-3 depends on M-1 and M-2.
- **SUMMARY/M3-E003** [STATUS] anchor=`M:L60` (13-M-SUMMARY_INDEX.discovery.md:L193): M-3 also depends on MA-3/J-59 and MA-6/K-6.
- **SUMMARY/M3-E004** [FLOW] anchor=`M:L61` (13-M-SUMMARY_INDEX.discovery.md:L194): Fleet aggregation uses a pull model.
- **SUMMARY/M3-E005** [CLI] [SURFACE] anchor=`M:L61` (13-M-SUMMARY_INDEX.discovery.md:L195): Each seat exports with 'elai export compliance --org'.
- **SUMMARY/M3-E006** [INT] anchor=`M:L61-L62, L67` (13-M-SUMMARY_INDEX.discovery.md:L196): The seat export extends the K-6 bundle rather than forking it.
- **SUMMARY/M3-E007** [DATA] anchor=`M:L61-L62` (13-M-SUMMARY_INDEX.discovery.md:L197): The organization export adds seat_id.
- **SUMMARY/M3-E008** [DATA] anchor=`M:L61-L62` (13-M-SUMMARY_INDEX.discovery.md:L198): The organization export adds org_id.
- **SUMMARY/M3-E009** [DATA] anchor=`M:L61-L62` (13-M-SUMMARY_INDEX.discovery.md:L199): The organization export adds pack_hash.
- **SUMMARY/M3-E010** [DATA] anchor=`M:L61-L62` (13-M-SUMMARY_INDEX.discovery.md:L200): The organization export adds drift.
- **SUMMARY/M3-E011** [DATA] anchor=`M:L62` (13-M-SUMMARY_INDEX.discovery.md:L201): The added export fields use serde defaults for compatibility.
- **SUMMARY/M3-E012** [CLI] [SURFACE] anchor=`M:L62` (13-M-SUMMARY_INDEX.discovery.md:L202): An admin aggregates with 'elai org report <dir> --verify'.
- **SUMMARY/M3-E013** [DATA] anchor=`M:L62-L63` (13-M-SUMMARY_INDEX.discovery.md:L203): One organization bundle is produced per report invocation.
- **SUMMARY/M3-E014** [DATA] anchor=`M:L63` (13-M-SUMMARY_INDEX.discovery.md:L204): The organization bundle contains a per-seat table.
- **SUMMARY/M3-E015** [DATA] anchor=`M:L63` (13-M-SUMMARY_INDEX.discovery.md:L205): The bundle rolls up verified-outcome rate.
- **SUMMARY/M3-E016** [DATA] anchor=`M:L63-L64` (13-M-SUMMARY_INDEX.discovery.md:L206): The bundle rolls up K-4 gate tax.
- **SUMMARY/M3-E017** [DATA] anchor=`M:L63-L64` (13-M-SUMMARY_INDEX.discovery.md:L207): The bundle rolls up K-1 deltas.
- **SUMMARY/M3-E018** [DATA] anchor=`M:L63-L64` (13-M-SUMMARY_INDEX.discovery.md:L208): The bundle rolls up H-9 dollars per resolved task.
- **SUMMARY/M3-E019** [DATA] anchor=`M:L64` (13-M-SUMMARY_INDEX.discovery.md:L209): The bundle rolls up K-3 unvouched counts.
- **SUMMARY/M3-E020** [DATA] anchor=`M:L64` (13-M-SUMMARY_INDEX.discovery.md:L210): The bundle rolls up M-2 policy drift.
- **SUMMARY/M3-E021** [DATA] anchor=`M:L64` (13-M-SUMMARY_INDEX.discovery.md:L211): The bundle reports seat-roster honesty against entitlement seats_max.
- **SUMMARY/M3-E022** [AUDIT] anchor=`M:L64-L65` (13-M-SUMMARY_INDEX.discovery.md:L212): Every fleet number is recomputable from constituent seat journals.
- **SUMMARY/M3-E023** [PRIV] anchor=`M:L65-L66` (13-M-SUMMARY_INDEX.discovery.md:L213): Fleet reporting sends zero telemetry.
- **SUMMARY/M3-E024** [PRIV] anchor=`M:L65-L66` (13-M-SUMMARY_INDEX.discovery.md:L214): Fleet reporting includes zero raw content.
- **SUMMARY/M3-E025** [FAIL] anchor=`M:L66` (13-M-SUMMARY_INDEX.discovery.md:L215): A seat with a broken evidence chain is refused.
- **SUMMARY/M3-E026** [OBS] anchor=`M:L66` (13-M-SUMMARY_INDEX.discovery.md:L216): A refused broken-chain seat is named.
- **SUMMARY/M3-E027** [CLAIM] anchor=`M:L66` (13-M-SUMMARY_INDEX.discovery.md:L217): M-3 defines zero new metrics; it only aggregates existing metrics.
- **SUMMARY/M3-E028** [INT] anchor=`M:L67` (13-M-SUMMARY_INDEX.discovery.md:L218): M-3 composes G-2/G-8 verification.
- **SUMMARY/M3-E029** [SAFE] anchor=`M:L67-L68` (13-M-SUMMARY_INDEX.discovery.md:L219): G-9 is both a precondition and aggregate self-test for the report.
- **SUMMARY/M3-E030** [INT] anchor=`M:L67-L68` (13-M-SUMMARY_INDEX.discovery.md:L220): M-3 composes K-1, K-4, H-9, K-3, M-1, and M-2.
- **SUMMARY/M3-E031** [INT] anchor=`M:L69` (13-M-SUMMARY_INDEX.discovery.md:L221): The planned integration module is commerce/org_report.rs.
- **SUMMARY/M3-E032** [OBS] anchor=`M:L69` (13-M-SUMMARY_INDEX.discovery.md:L222): Existing SSB events should be reused for organization reporting when possible.
- **SUMMARY/M3-E033** [OBS] anchor=`M:L69` (13-M-SUMMARY_INDEX.discovery.md:L223): OrgReportEvent may be added only if no existing event fits.
- **SUMMARY/M3-E034** [TEST] anchor=`M:L70` (13-M-SUMMARY_INDEX.discovery.md:L224): Acceptance uses three fixture exports: clean, drift, and broken.
- **SUMMARY/M3-E035** [TEST] anchor=`M:L70` (13-M-SUMMARY_INDEX.discovery.md:L225): The three fixtures must aggregate into one bundle.
- **SUMMARY/M3-E036** [TEST] anchor=`M:L70-L71` (13-M-SUMMARY_INDEX.discovery.md:L226): The clean and drift seats must verify PASS.
- **SUMMARY/M3-E037** [TEST] anchor=`M:L70-L71` (13-M-SUMMARY_INDEX.discovery.md:L227): The broken seat must be refused and named as RefusedSeat under R2.
- **SUMMARY/M3-E038** [TEST] anchor=`M:L71` (13-M-SUMMARY_INDEX.discovery.md:L228): Rollup numbers must recompute byte-identically.
- **SUMMARY/M3-E039** [TEST] anchor=`M:L71-L72` (13-M-SUMMARY_INDEX.discovery.md:L229): A seeded secret in any constituent must fail the build.
- **SUMMARY/M3-E040** [TEST] anchor=`M:L72` (13-M-SUMMARY_INDEX.discovery.md:L230): A byte-level transcript grep over the generated bundle must be empty.

### SUMMARY unit `M-4 — signed release channel + auto-updater + support bundle` (44 records)

- **SUMMARY/M4-E001** [STATUS] anchor=`M:L74-L75` (13-M-SUMMARY_INDEX.discovery.md:L236): M-4 has lever REACH plus CANT-WRECK, STRONG grounding from gaps G5/G6 and B-43’s named updater deferral, BUILD, effort M, wave M2.
- **SUMMARY/M4-E002** [STATUS] anchor=`M:L75` (13-M-SUMMARY_INDEX.discovery.md:L237): M-4 depends on MA-4, M-2, and G-9.
- **SUMMARY/M4-E003** [DATA] anchor=`M:L76` (13-M-SUMMARY_INDEX.discovery.md:L238): Release manifests are signed.
- **SUMMARY/M4-E004** [DATA] anchor=`M:L76` (13-M-SUMMARY_INDEX.discovery.md:L239): Each release artifact carries a blake3 digest.
- **SUMMARY/M4-E005** [INT] anchor=`M:L76-L77` (13-M-SUMMARY_INDEX.discovery.md:L240): Artifact verification reuses B-43’s refuse-on-mismatch core.
- **SUMMARY/M4-E006** [SAFE] anchor=`M:L77` (13-M-SUMMARY_INDEX.discovery.md:L241): Revocation-list distribution provides M-1’s key-compromise lever.
- **SUMMARY/M4-E007** [CFG] anchor=`M:L77` (13-M-SUMMARY_INDEX.discovery.md:L242): The updater supports a stable channel.
- **SUMMARY/M4-E008** [CFG] anchor=`M:L77` (13-M-SUMMARY_INDEX.discovery.md:L243): The updater supports a beta channel.
- **SUMMARY/M4-E009** [CFG] anchor=`M:L77-L78` (13-M-SUMMARY_INDEX.discovery.md:L244): An M-2 organization-policy clause can pin the release channel.
- **SUMMARY/M4-E010** [FLOW] anchor=`M:L78` (13-M-SUMMARY_INDEX.discovery.md:L245): Staged rollout is deterministic.
- **SUMMARY/M4-E011** [FLOW] anchor=`M:L78` (13-M-SUMMARY_INDEX.discovery.md:L246): Rollout assignment is blake3(seat_id) modulo 100.
- **SUMMARY/M4-E012** [OBS] anchor=`M:L78` (13-M-SUMMARY_INDEX.discovery.md:L247): A staged-rollout assignment must be explainable from the journal.
- **SUMMARY/M4-E013** [CLI] [SURFACE] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L248): The CLI exposes 'elai update check'.
- **SUMMARY/M4-E014** [CLI] [SURFACE] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L249): The CLI exposes 'elai update apply'.
- **SUMMARY/M4-E015** [CLI] [SURFACE] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L250): The CLI exposes 'elai update rollback'.
- **SUMMARY/M4-E016** [CLI] [SURFACE] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L251): The CLI exposes 'elai update channel'.
- **SUMMARY/M4-E017** [OPS] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L252): Update checking is default-on.
- **SUMMARY/M4-E018** [OPS] anchor=`M:L79` (13-M-SUMMARY_INDEX.discovery.md:L253): Update checking is non-blocking.
- **SUMMARY/M4-E019** [SAFE] anchor=`M:L79-L80` (13-M-SUMMARY_INDEX.discovery.md:L254): An update apply requires user confirmation unless organization-scheduled.
- **SUMMARY/M4-E020** [OPS] anchor=`M:L79-L80` (13-M-SUMMARY_INDEX.discovery.md:L255): An organization policy may schedule update application.
- **SUMMARY/M4-E021** [SAFE] anchor=`M:L80` (13-M-SUMMARY_INDEX.discovery.md:L256): Update application is never silent.
- **SUMMARY/M4-E022** [LIFE] anchor=`M:L80-L81` (13-M-SUMMARY_INDEX.discovery.md:L257): Rollback retains the last-good binary.
- **SUMMARY/M4-E023** [CLI] [SURFACE] anchor=`M:L81` (13-M-SUMMARY_INDEX.discovery.md:L258): The CLI exposes 'elai support bundle'.
- **SUMMARY/M4-E024** [PRIV] anchor=`M:L81` (13-M-SUMMARY_INDEX.discovery.md:L259): A support bundle is assembled locally.
- **SUMMARY/M4-E025** [SAFE] anchor=`M:L81` (13-M-SUMMARY_INDEX.discovery.md:L260): Support-bundle diagnostics are G-9-redacted.
- **SUMMARY/M4-E026** [DATA] anchor=`M:L81` (13-M-SUMMARY_INDEX.discovery.md:L261): A support bundle includes versions.
- **SUMMARY/M4-E027** [DATA] anchor=`M:L81` (13-M-SUMMARY_INDEX.discovery.md:L262): A support bundle includes tier.
- **SUMMARY/M4-E028** [DATA] anchor=`M:L81-L82` (13-M-SUMMARY_INDEX.discovery.md:L263): A support bundle includes hashes.
- **SUMMARY/M4-E029** [DATA] anchor=`M:L82` (13-M-SUMMARY_INDEX.discovery.md:L264): A support bundle includes doctor output.
- **SUMMARY/M4-E030** [DATA] anchor=`M:L82` (13-M-SUMMARY_INDEX.discovery.md:L265): A support bundle includes a redacted journal tail.
- **SUMMARY/M4-E031** [UI] [SURFACE] anchor=`M:L82` (13-M-SUMMARY_INDEX.discovery.md:L266): Support-bundle creation prints a summary.
- **SUMMARY/M4-E032** [PRIV] anchor=`M:L82` (13-M-SUMMARY_INDEX.discovery.md:L267): A support bundle is never automatically sent.
- **SUMMARY/M4-E033** [STATUS] anchor=`M:L82-L83` (13-M-SUMMARY_INDEX.discovery.md:L268): Version 1 updates whole artifacts.
- **SUMMARY/M4-E034** [DEFER] anchor=`M:L83` (13-M-SUMMARY_INDEX.discovery.md:L269): Delta updates are deferred to a later rider.
- **SUMMARY/M4-E035** [INT] anchor=`M:L84` (13-M-SUMMARY_INDEX.discovery.md:L270): M-4 is a sibling of B-43 HostInstaller and shares its manifest-verification code.
- **SUMMARY/M4-E036** [INT] anchor=`M:L84-L85` (13-M-SUMMARY_INDEX.discovery.md:L271): M-4 also composes B-45, M-2, G-9, doctor, and G-8.
- **SUMMARY/M4-E037** [INT] anchor=`M:L86` (13-M-SUMMARY_INDEX.discovery.md:L272): The planned CLI integration module is crates/cli/src/update.rs plus a support-bundle arm.
- **SUMMARY/M4-E038** [OBS] anchor=`M:L86-L87` (13-M-SUMMARY_INDEX.discovery.md:L273): M-4 may add a Pattern-2 ReleaseChannelEvent.
- **SUMMARY/M4-E039** [OBS] anchor=`M:L86-L87` (13-M-SUMMARY_INDEX.discovery.md:L274): Existing event types should be preferred over ReleaseChannelEvent when they fit.
- **SUMMARY/M4-E040** [TEST] anchor=`M:L88` (13-M-SUMMARY_INDEX.discovery.md:L275): A corrupted artifact must be refused before any file lands.
- **SUMMARY/M4-E041** [TEST] anchor=`M:L88` (13-M-SUMMARY_INDEX.discovery.md:L276): A clean artifact must apply successfully.
- **SUMMARY/M4-E042** [TEST] anchor=`M:L88-L89` (13-M-SUMMARY_INDEX.discovery.md:L277): Rollback after a clean apply must restore the prior binary with a matching hash.
- **SUMMARY/M4-E043** [TEST] anchor=`M:L89` (13-M-SUMMARY_INDEX.discovery.md:L278): An organization stable-channel pin must hide beta manifests.
- **SUMMARY/M4-E044** [TEST] anchor=`M:L89-L90` (13-M-SUMMARY_INDEX.discovery.md:L279): A support bundle built from a seeded-secret fixture session must have an empty secret grep.

### SUMMARY unit `M-5 — commercial packaging + verifier OSS + trust continuity` (46 records)

- **SUMMARY/M5-E001** [STATUS] anchor=`M:L92-L93` (13-M-SUMMARY_INDEX.discovery.md:L285): M-5 has levers SELLABLE and AUDITABLE, STRONG grounding from gap G7, E-23-class nature plus one extraction, and effort S-M.
- **SUMMARY/M5-E002** [STATUS] anchor=`M:L93` (13-M-SUMMARY_INDEX.discovery.md:L286): M-5 slices a and d are wave M1; slices b and c are wave M3.
- **SUMMARY/M5-E003** [CAP] anchor=`M:L94-L95` (13-M-SUMMARY_INDEX.discovery.md:L287): M-5 publishes a standalone open-source elai-verify crate and binary.
- **SUMMARY/M5-E004** [INT] anchor=`M:L94-L95` (13-M-SUMMARY_INDEX.discovery.md:L288): elai-verify wraps the same G-8 attestation verification path.
- **SUMMARY/M5-E005** [INT] anchor=`M:L94-L95` (13-M-SUMMARY_INDEX.discovery.md:L289): elai-verify wraps the same G-2 RRA verification path.
- **SUMMARY/M5-E006** [INT] anchor=`M:L94-L95` (13-M-SUMMARY_INDEX.discovery.md:L290): elai-verify wraps the same K-6 conformance verification path.
- **SUMMARY/M5-E007** [INT] anchor=`M:L95` (13-M-SUMMARY_INDEX.discovery.md:L291): elai-verify wraps the same M-3 organization-bundle verification path.
- **SUMMARY/M5-E008** [SAFE] anchor=`M:L95-L96` (13-M-SUMMARY_INDEX.discovery.md:L292): elai-verify has zero engine dependencies.
- **SUMMARY/M5-E009** [TEST] anchor=`M:L95-L96` (13-M-SUMMARY_INDEX.discovery.md:L293): Extracting the verifier forms a compile-time proof of the engine boundary.
- **SUMMARY/M5-E010** [CAP] anchor=`M:L96` (13-M-SUMMARY_INDEX.discovery.md:L294): The ELAI engine remains closed-source.
- **SUMMARY/M5-E011** [COMP] anchor=`M:L97` (13-M-SUMMARY_INDEX.discovery.md:L295): Commercial packaging includes an EULA draft.
- **SUMMARY/M5-E012** [COMP] anchor=`M:L97` (13-M-SUMMARY_INDEX.discovery.md:L296): Commercial packaging includes a license-split draft.
- **SUMMARY/M5-E013** [COMP] anchor=`M:L97` (13-M-SUMMARY_INDEX.discovery.md:L297): Commercial packaging includes a DPA draft.
- **SUMMARY/M5-E014** [COMP] anchor=`M:L97` (13-M-SUMMARY_INDEX.discovery.md:L298): Each legal draft carries a counsel-review banner.
- **SUMMARY/M5-E015** [PRIV] anchor=`M:L97-L98` (13-M-SUMMARY_INDEX.discovery.md:L299): The mint service sees org_id.
- **SUMMARY/M5-E016** [PRIV] anchor=`M:L97-L98` (13-M-SUMMARY_INDEX.discovery.md:L300): The mint service sees payment metadata.
- **SUMMARY/M5-E017** [PRIV] anchor=`M:L97-L98` (13-M-SUMMARY_INDEX.discovery.md:L301): The mint service sees only org_id and payment metadata.
- **SUMMARY/M5-E018** [COMP] anchor=`M:L98` (13-M-SUMMARY_INDEX.discovery.md:L302): The trust-continuity statement offers escrow on lapse.
- **SUMMARY/M5-E019** [COMP] anchor=`M:L98-L99` (13-M-SUMMARY_INDEX.discovery.md:L303): The trust-continuity statement alternatively offers conversion to OSS on abandonment.
- **SUMMARY/M5-E020** [STATUS] anchor=`M:L98-L99` (13-M-SUMMARY_INDEX.discovery.md:L304): Both continuity alternatives are drafted and the user selects between them.
- **SUMMARY/M5-E021** [COMP] anchor=`M:L99` (13-M-SUMMARY_INDEX.discovery.md:L305): Commercial documentation includes a key-custody note.
- **SUMMARY/M5-E022** [CLAIM] anchor=`M:L99-L100` (13-M-SUMMARY_INDEX.discovery.md:L306): Commercial documentation states an honest solo-support posture.
- **SUMMARY/M5-E023** [CLAIM] anchor=`M:L99-L100` (13-M-SUMMARY_INDEX.discovery.md:L307): Commercial documentation must not advertise a fake SLA.
- **SUMMARY/M5-E024** [CFG] anchor=`M:L100` (13-M-SUMMARY_INDEX.discovery.md:L308): tiers.toml defines Free as data.
- **SUMMARY/M5-E025** [CFG] anchor=`M:L100` (13-M-SUMMARY_INDEX.discovery.md:L309): tiers.toml defines Pro as data.
- **SUMMARY/M5-E026** [CFG] anchor=`M:L100` (13-M-SUMMARY_INDEX.discovery.md:L310): tiers.toml defines Team as data.
- **SUMMARY/M5-E027** [CFG] anchor=`M:L100-L101` (13-M-SUMMARY_INDEX.discovery.md:L311): tiers.toml is the one pricing source of truth.
- **SUMMARY/M5-E028** [INT] anchor=`M:L100-L101` (13-M-SUMMARY_INDEX.discovery.md:L312): M-1 reads tiers.toml.
- **SUMMARY/M5-E029** [INT] anchor=`M:L100-L101` (13-M-SUMMARY_INDEX.discovery.md:L313): K-24 reads tiers.toml.
- **SUMMARY/M5-E030** [CLAIM] anchor=`M:L101` (13-M-SUMMARY_INDEX.discovery.md:L314): The $39-49-per-seat go-to-market framing is tagged [EXTERNAL-GTM].
- **SUMMARY/M5-E031** [CLAIM] anchor=`M:L101` (13-M-SUMMARY_INDEX.discovery.md:L315): That price framing remains externally qualified until K-24 supplies a measured number.
- **SUMMARY/M5-E032** [DEFER] anchor=`M:L102` (13-M-SUMMARY_INDEX.discovery.md:L316): The Article-50-certified hook remains killed under LOCK-3.
- **SUMMARY/M5-E033** [INT] anchor=`M:L103-L104` (13-M-SUMMARY_INDEX.discovery.md:L317): M-5 composes G-8/G-2/K-6/M-3 verification, K-1 dossier and claims lint, K-7 brief, and M-1.
- **SUMMARY/M5-E034** [CLAIM] anchor=`M:L103-L104` (13-M-SUMMARY_INDEX.discovery.md:L318): K-1 claims lint gains a pricing recognizer.
- **SUMMARY/M5-E035** [INT] anchor=`M:L105` (13-M-SUMMARY_INDEX.discovery.md:L319): The published integration artifact is the elai-verify crate.
- **SUMMARY/M5-E036** [INT] anchor=`M:L105` (13-M-SUMMARY_INDEX.discovery.md:L320): Legal and commercial content lives under docs/strategy/commercial/.
- **SUMMARY/M5-E037** [INT] anchor=`M:L105` (13-M-SUMMARY_INDEX.discovery.md:L321): Integration includes tiers.toml.
- **SUMMARY/M5-E038** [INT] anchor=`M:L105-L106` (13-M-SUMMARY_INDEX.discovery.md:L322): Integration includes a claims-lint extension.
- **SUMMARY/M5-E039** [OBS] anchor=`M:L106` (13-M-SUMMARY_INDEX.discovery.md:L323): M-5 adds no SSB event.
- **SUMMARY/M5-E040** [TEST] anchor=`M:L107-L108` (13-M-SUMMARY_INDEX.discovery.md:L324): On a clean, engine-free machine, elai-verify must verify a fixture attestation.
- **SUMMARY/M5-E041** [TEST] anchor=`M:L107-L108` (13-M-SUMMARY_INDEX.discovery.md:L325): On a clean, engine-free machine, elai-verify must verify a fixture RRA.
- **SUMMARY/M5-E042** [TEST] anchor=`M:L107-L108` (13-M-SUMMARY_INDEX.discovery.md:L326): On a clean, engine-free machine, elai-verify must verify a fixture organization bundle.
- **SUMMARY/M5-E043** [TEST] anchor=`M:L108` (13-M-SUMMARY_INDEX.discovery.md:L327): elai-verify must catch a one-byte artifact tamper under R2.
- **SUMMARY/M5-E044** [TEST] anchor=`M:L108-L109` (13-M-SUMMARY_INDEX.discovery.md:L328): The crate must compile with zero engine-crate dependencies.
- **SUMMARY/M5-E045** [TEST] anchor=`M:L109` (13-M-SUMMARY_INDEX.discovery.md:L329): A seeded pricing superlative must fail claims lint under MD-5.
- **SUMMARY/M5-E046** [TEST] anchor=`M:L109-L110` (13-M-SUMMARY_INDEX.discovery.md:L330): 'elai license status' must render from tiers.toml.

### SUMMARY unit `M-6 — evidence format as a versioned open standard` (42 records)

- **SUMMARY/M6-E001** [STATUS] anchor=`M:L112-L116` (13-M-SUMMARY_INDEX.discovery.md:L336): M-6 has levers REACH and AUDITABLE, STRONG grounding from spec-eval ADD-1, BUILD-compose plus docs, effort M, wave M3.
- **SUMMARY/M6-E002** [STATUS] anchor=`M:L113-L115` (13-M-SUMMARY_INDEX.discovery.md:L337): SLSA/in-toto precedent is training-memory only and must be re-verified at execution.
- **SUMMARY/M6-E003** [STATUS] anchor=`M:L115-L116` (13-M-SUMMARY_INDEX.discovery.md:L338): M-6 depends on M-5a extraction and stable G-2/G-8/K-6 emit paths.
- **SUMMARY/M6-E004** [CAP] anchor=`M:L117` (13-M-SUMMARY_INDEX.discovery.md:L339): M-6 publishes a versioned machine-readable evidence format provisionally called EEF v1.
- **SUMMARY/M6-E005** [STATUS] anchor=`M:L117` (13-M-SUMMARY_INDEX.discovery.md:L340): The final EEF name is selected at execution.
- **SUMMARY/M6-E006** [DATA] anchor=`M:L117-L118` (13-M-SUMMARY_INDEX.discovery.md:L341): The format specifies the RRA bundle shape.
- **SUMMARY/M6-E007** [DATA] anchor=`M:L117-L118` (13-M-SUMMARY_INDEX.discovery.md:L342): The format specifies the attestation predicate shape.
- **SUMMARY/M6-E008** [DATA] anchor=`M:L118` (13-M-SUMMARY_INDEX.discovery.md:L343): The format specifies the compliance-bundle shape.
- **SUMMARY/M6-E009** [DATA] anchor=`M:L118` (13-M-SUMMARY_INDEX.discovery.md:L344): The format specifies verdict-row shapes.
- **SUMMARY/M6-E010** [DATA] anchor=`M:L118` (13-M-SUMMARY_INDEX.discovery.md:L345): The format specifies receipt-row shapes.
- **SUMMARY/M6-E011** [INT] anchor=`M:L118-L119` (13-M-SUMMARY_INDEX.discovery.md:L346): Third-party tools can emit evidence that ELAI can verify.
- **SUMMARY/M6-E012** [CAP] anchor=`M:L119` (13-M-SUMMARY_INDEX.discovery.md:L347): elai-verify becomes the reference verifier for the format.
- **SUMMARY/M6-E013** [DATA] anchor=`M:L119-L120` (13-M-SUMMARY_INDEX.discovery.md:L348): Schemas are derived from engine serialization types.
- **SUMMARY/M6-E014** [SAFE] anchor=`M:L120` (13-M-SUMMARY_INDEX.discovery.md:L349): Engine serialization types are the single source of schema truth.
- **SUMMARY/M6-E015** [SAFE] anchor=`M:L120` (13-M-SUMMARY_INDEX.discovery.md:L350): Schema definitions must never be maintained as a hand-written fork.
- **SUMMARY/M6-E016** [COMP] anchor=`M:L121` (13-M-SUMMARY_INDEX.discovery.md:L351): Publication includes a prose format specification.
- **SUMMARY/M6-E017** [TEST] anchor=`M:L121` (13-M-SUMMARY_INDEX.discovery.md:L352): Publication includes conformance fixtures.
- **SUMMARY/M6-E018** [CLI] [SURFACE] anchor=`M:L121` (13-M-SUMMARY_INDEX.discovery.md:L353): elai-verify adds 'conform <artifact>'.
- **SUMMARY/M6-E019** [SAFE] anchor=`M:L121-L122` (13-M-SUMMARY_INDEX.discovery.md:L354): Format conformance is fail-closed.
- **SUMMARY/M6-E020** [FAIL] anchor=`M:L122` (13-M-SUMMARY_INDEX.discovery.md:L355): Missing required evidence is normatively a conformance FAIL.
- **SUMMARY/M6-E021** [LIFE] anchor=`M:L123` (13-M-SUMMARY_INDEX.discovery.md:L356): Schema evolution is append-only.
- **SUMMARY/M6-E022** [SAFE] anchor=`M:L123-L124` (13-M-SUMMARY_INDEX.discovery.md:L357): Schema evolution is protected by a golden-file lock.
- **SUMMARY/M6-E023** [SAFE] anchor=`M:L123-L124` (13-M-SUMMARY_INDEX.discovery.md:L358): The schema golden-file lock joins the frozen gate manifest under R8/E-45.
- **SUMMARY/M6-E024** [OPS] anchor=`M:L124` (13-M-SUMMARY_INDEX.discovery.md:L359): The evidence format is published beside elai-verify.
- **SUMMARY/M6-E025** [OPS] anchor=`M:L124` (13-M-SUMMARY_INDEX.discovery.md:L360): Publication includes a worked third-party-emitter example.
- **SUMMARY/M6-E026** [CLAIM] anchor=`M:L124-L125` (13-M-SUMMARY_INDEX.discovery.md:L361): The phrase 'open standard' is KD-1-gated on at least one external emitter.
- **SUMMARY/M6-E027** [CLAIM] anchor=`M:L125` (13-M-SUMMARY_INDEX.discovery.md:L362): Before one external emitter exists, the permitted phrase is 'published format'.
- **SUMMARY/M6-E028** [CLAIM] anchor=`M:L125-L126` (13-M-SUMMARY_INDEX.discovery.md:L363): K-1 claims lint gains an adoption-claim recognizer.
- **SUMMARY/M6-E029** [STATUS] anchor=`M:L126` (13-M-SUMMARY_INDEX.discovery.md:L364): M-6 has no direct PROOF_SPINE contribution.
- **SUMMARY/M6-E030** [STATUS] anchor=`M:L126` (13-M-SUMMARY_INDEX.discovery.md:L365): M-6 is recorded as a distribution amplifier with a user-acknowledged spine exemption.
- **SUMMARY/M6-E031** [INT] anchor=`M:L127` (13-M-SUMMARY_INDEX.discovery.md:L366): M-6 extends M-5a’s verifier binary.
- **SUMMARY/M6-E032** [INT] anchor=`M:L127-L128` (13-M-SUMMARY_INDEX.discovery.md:L367): M-6 describes, but never forks, the G-2/G-8/K-6 emit paths.
- **SUMMARY/M6-E033** [INT] anchor=`M:L127-L128` (13-M-SUMMARY_INDEX.discovery.md:L368): M-6 composes E-45’s golden lock, K-1 lint, K-3/K-11 rendered shapes, and gate-manifest R8.
- **SUMMARY/M6-E034** [CAP] anchor=`M:L128` (13-M-SUMMARY_INDEX.discovery.md:L369): The engine stays closed while the evidence format and verifier are published.
- **SUMMARY/M6-E035** [TEST] anchor=`M:L129-L130` (13-M-SUMMARY_INDEX.discovery.md:L370): A conformant fixture must pass 'elai-verify conform' on an engine-free machine.
- **SUMMARY/M6-E036** [TEST] anchor=`M:L129-L130` (13-M-SUMMARY_INDEX.discovery.md:L371): A one-byte-tampered fixture must fail 'elai-verify conform'.
- **SUMMARY/M6-E037** [TEST] anchor=`M:L129-L130` (13-M-SUMMARY_INDEX.discovery.md:L372): A missing-field fixture must fail 'elai-verify conform'.
- **SUMMARY/M6-E038** [TEST] anchor=`M:L130-L131` (13-M-SUMMARY_INDEX.discovery.md:L373): Derived schemas must byte-match regeneration from engine types.
- **SUMMARY/M6-E039** [TEST] anchor=`M:L131` (13-M-SUMMARY_INDEX.discovery.md:L374): Removing a schema field must trip the R8 golden lock.
- **SUMMARY/M6-E040** [TEST] anchor=`M:L131-L132` (13-M-SUMMARY_INDEX.discovery.md:L375): A seeded 'industry standard' phrase must fail K-1 lint while no adoption row exists.
- **SUMMARY/M6-E041** [TEST] anchor=`M:L132-L133` (13-M-SUMMARY_INDEX.discovery.md:L376): The worked third-party example must emit a row accepted by conform.
- **SUMMARY/M6-E042** [OBS] anchor=`M:L133` (13-M-SUMMARY_INDEX.discovery.md:L377): M-6 adds no SSB event because it consists of the standalone verifier and documentation.

### SUMMARY end-to-end workflow steps

Workflow step records: **22**.

- **SUMMARY/WF-001** [FLOW] workflow=`Purchase, activation, expiry, and continued safe use` step=1 (13-M:L383): An external merchant of record handles payment and calls ELAI’s sole first-party mint endpoint. The endpoint only mints, renews, or revokes an entitlement. [M1-E018..M1-E024]
- **SUMMARY/WF-002** [FLOW] workflow=`Purchase, activation, expiry, and continued safe use` step=2 (13-M:L384): A signed entitlement is delivered to a seat and verified offline; installation may offer activation at its end. [M1-E002..M1-E010, M1-E034]
- **SUMMARY/WF-003** [FLOW] workflow=`Purchase, activation, expiry, and continued safe use` step=3 (13-M:L385): tier() reads the data-defined tier and gates K-24 credit grants plus M-2/M-3 Team functionality. [M1-E025..M1-E029]
- **SUMMARY/WF-004** [FLOW] workflow=`Purchase, activation, expiry, and continued safe use` step=4 (13-M:L386): All expiry math uses ClockProvider. After a 14-day grace, the lifecycle emits ExpiredGrace and DegradedToFree. [M1-E014..M1-E017, M1-E033, M1-E039..M1-E040]
- **SUMMARY/WF-005** [FLOW] workflow=`Purchase, activation, expiry, and continued safe use` step=5 (13-M:L387): A degraded seat can still run Free work and every safety gate remains active. Forgery is refused, while excess activation is honestly recorded for M-3 rather than bricking the seat. [M1-E030..M1-E032, M1-E049..M1-E052]
- **SUMMARY/WF-006** [FLOW] workflow=`Policy creation, boot application, tightening, and drift` step=1 (13-M:L391): An administrator initializes, signs, verifies, and shows one file-distributable OrgPolicyPack. [M2-E003..M2-E004, M2-E024..M2-E027]
- **SUMMARY/WF-007** [FLOW] workflow=`Policy creation, boot application, tightening, and drift` step=2 (13-M:L392): At boot, the pack supplies rules, manifest pins, caps, allowlists, redaction configuration, and release-channel pinning. [M2-E005..M2-E011]
- **SUMMARY/WF-008** [FLOW] workflow=`Policy creation, boot application, tightening, and drift` step=3 (13-M:L393): A seat may tighten these controls but cannot weaken them; a raise/weaken attempt becomes WeakenAttemptBlocked. [M2-E012..M2-E014, M2-E032, M2-E036]
- **SUMMARY/WF-009** [FLOW] workflow=`Policy creation, boot application, tightening, and drift` step=4 (13-M:L394): Tampering is refused. Staleness creates surfaced Drift but is not auto-corrected. No pack preserves byte-identical baseline behavior. [M2-E015..M2-E019, M2-E030..M2-E031, M2-E035, M2-E037..M2-E038]
- **SUMMARY/WF-010** [FLOW] workflow=`Private fleet evidence aggregation` step=1 (13-M:L398): Each seat produces an extended K-6 compliance export with seat and policy identifiers but no raw content. [M3-E005..M3-E011, M3-E023..M3-E024]
- **SUMMARY/WF-011** [FLOW] workflow=`Private fleet evidence aggregation` step=2 (13-M:L399): An administrator pulls exports into a directory and runs the verifying organization-report command. [M3-E004, M3-E012]
- **SUMMARY/WF-012** [FLOW] workflow=`Private fleet evidence aggregation` step=3 (13-M:L400): The command verifies evidence chains, refuses and names broken seats, and builds one per-seat table plus existing-metric rollups. [M3-E013..M3-E027]
- **SUMMARY/WF-013** [FLOW] workflow=`Private fleet evidence aggregation` step=4 (13-M:L401): Every number can be rederived from constituent journals, with secrets and transcript bytes excluded. [M3-E022, M3-E029, M3-E038..M3-E040]
- **SUMMARY/WF-014** [FLOW] workflow=`Safe update and local support` step=1 (13-M:L405): A seat non-blockingly checks signed stable/beta manifests by default. An organization pin may filter the visible channel, while blake3(seat_id)%100 deterministically assigns staged rollout. [M4-E003..M4-E018]
- **SUMMARY/WF-015** [FLOW] workflow=`Safe update and local support` step=2 (13-M:L406): Application requires explicit user confirmation or an organization schedule and is never silent. Artifact mismatch is refused before disk write. [M4-E019..M4-E021, M4-E040]
- **SUMMARY/WF-016** [FLOW] workflow=`Safe update and local support` step=3 (13-M:L407): The previous known-good binary is retained so rollback can restore its exact hash. [M4-E022, M4-E041..M4-E042]
- **SUMMARY/WF-017** [FLOW] workflow=`Safe update and local support` step=4 (13-M:L408): A user can assemble a local, redacted support bundle, inspect its printed summary, and decide whether to transmit it; ELAI never auto-sends it. [M4-E023..M4-E032]
- **SUMMARY/WF-018** [FLOW] workflow=`Independent verification and format adoption` step=1 (13-M:L412): The same engine verification paths are extracted into an open-source, engine-independent elai-verify boundary while the engine remains closed. [M5-E003..M5-E010]
- **SUMMARY/WF-019** [FLOW] workflow=`Independent verification and format adoption` step=2 (13-M:L413): On a clean machine, the binary verifies attestations, RRAs, conformance artifacts, and fleet bundles and rejects one-byte tampering. [M5-E040..M5-E044]
- **SUMMARY/WF-020** [FLOW] workflow=`Independent verification and format adoption` step=3 (13-M:L414): M-6 derives evidence schemas from engine serialization types and publishes them with prose, fixtures, the conform command, and a third-party example. [M6-E004..M6-E025]
- **SUMMARY/WF-021** [FLOW] workflow=`Independent verification and format adoption` step=4 (13-M:L415): External emitters can target the format. 'Open standard' remains a prohibited adoption claim until at least one external emitter exists. [M6-E011..M6-E012, M6-E026..M6-E028]
- **SUMMARY/WF-022** [FLOW] workflow=`Independent verification and format adoption` step=5 (13-M:L416): Schema changes are append-only and golden-locked; field removal, missing evidence, or tampering fails closed. [M6-E019..M6-E023, M6-E035..M6-E040]

### SUMMARY cross-feature invariants

Invariant records: **15**.

- **SUMMARY/INV-001** [INVAR] (13-M:L420): Commerce availability and safety have deliberately different failure semantics: commerce degrades to Free; safety gates never degrade. [MX-E020..MX-E024, MX-E035..MX-E036]
- **SUMMARY/INV-002** [INVAR] (13-M:L421): Runtime work never depends on a licensing service. Once obtained, entitlement proof is local and offline-verifiable. [MX-E023..MX-E025, M1-E002..M1-E003]
- **SUMMARY/INV-003** [INVAR] (13-M:L422): The only first-party commercial cloud surface is the mint endpoint, limited to mint/renew/revoke. Policy and evidence move through files or git. [MX-E025..MX-E026, M1-E020..M1-E024]
- **SUMMARY/INV-004** [INVAR] (13-M:L423): Organization controls are monotonic: a seat can tighten but cannot weaken a signed organization pack. [M2-E012..M2-E014]
- **SUMMARY/INV-005** [INVAR] (13-M:L424): Organization pack content is inert data, never executable code. [M2-E018]
- **SUMMARY/INV-006** [INVAR] (13-M:L425): Absence of an organization pack must preserve byte-identical single-seat behavior. [M2-E019, M2-E037]
- **SUMMARY/INV-007** [INVAR] (13-M:L426): Fleet reporting is pull-based, locally recomputable, and contains neither telemetry nor raw content/transcripts. [M3-E004, M3-E022..M3-E024, M3-E040]
- **SUMMARY/INV-008** [INVAR] (13-M:L427): Broken or forged trust chains are refused and named where applicable. [M1-E042, M1-E051, M2-E031, M2-E035, M3-E025..M3-E026, M3-E037]
- **SUMMARY/INV-009** [INVAR] (13-M:L428): Update integrity is checked before any artifact lands, and a last-good binary remains rollbackable. [M4-E022, M4-E040..M4-E042]
- **SUMMARY/INV-010** [INVAR] (13-M:L429): Support material is local, redacted, summarized to the user, and never auto-sent. [M4-E024..M4-E032]
- **SUMMARY/INV-011** [INVAR] (13-M:L430): tiers.toml is data and the single pricing truth consumed by entitlement and billing. [M1-E029, M5-E024..M5-E029]
- **SUMMARY/INV-012** [INVAR] (13-M:L431): Monetary, pricing, adoption, SLA, and certification language remains evidence-gated rather than promotional by default. [MX-E031..MX-E032, M5-E022..M5-E023, M5-E030..M5-E032, M6-E026..M6-E028]
- **SUMMARY/INV-013** [INVAR] (13-M:L432): The external verifier reuses the same verification paths but has zero engine dependencies; publishing it does not open the engine. [M5-E003..M5-E010]
- **SUMMARY/INV-014** [INVAR] (13-M:L433): Evidence schemas derive from engine types, evolve append-only, and fail closed on missing evidence. [M6-E013..M6-E023]
- **SUMMARY/INV-015** [INVAR] (13-M:L434): SSB additions are intentionally bounded: EntitlementEvent and OrgPolicyEvent are explicit; M-3/M-4 prefer reuse; M-5/M-6 add none. [MX-E010, M1-E036..M1-E042, M2-E028..M2-E032, M3-E032..M3-E033, M4-E038..M4-E039, M5-E039, M6-E042]

### SUMMARY dependencies and ordering

Ordering + reuse rows: **33**.

- **SUMMARY/ORDER-001** [DEP] (13-M:L461): All M execution is held behind GATE M-0. The named prerequisites are K A-1..A-4, K-3, and K-24 L1..L4. [MX-E007, MX-E040..MX-E042]
- **SUMMARY/ORDER-002** [DEP] (13-M:L462): Within M, M-1 precedes M-2; M-3 requires M-1/M-2 plus J-59/K-6 pointers; M-4 requires M-2/G-9 plus MA-4; M-6 requires M-5a and stable upstream emit paths. [M2-E002, M3-E002..M3-E003, M4-E002, M6-E003]
- **SUMMARY/ORDER-003** [DEP] (13-M:L463): Wave grouping: M1 contains entitlement plus tiers/verifier extraction; M2 contains policy and updater; M3 contains fleet rollup, legal/continuity work, and evidence-format publication. [coverage map; M5-E002]
- **SUMMARY/REUSE-001** [DEP] (13-M:L469): 3F → policy lifecycle and mutable tier
- **SUMMARY/REUSE-002** [DEP] (13-M:L470): B-37 → organization allowlist
- **SUMMARY/REUSE-003** [DEP] (13-M:L471): B-43 → installer-end activation and shared artifact mismatch verification
- **SUMMARY/REUSE-004** [DEP] (13-M:L472): B-45 → updater composition
- **SUMMARY/REUSE-005** [DEP] (13-M:L473): E-23 → commercial-document class/pattern
- **SUMMARY/REUSE-006** [DEP] (13-M:L474): E-39 → spend caps
- **SUMMARY/REUSE-007** [DEP] (13-M:L475): E-45 → schema/golden-lock evolution pattern
- **SUMMARY/REUSE-008** [DEP] (13-M:L476): G-2 → RRA verification and evidence format
- **SUMMARY/REUSE-009** [DEP] (13-M:L477): G-8 → signing, attestation verification, and offline verification pattern
- **SUMMARY/REUSE-010** [DEP] (13-M:L478): G-9 → redaction, report precondition/self-test, support-bundle safety
- **SUMMARY/REUSE-011** [DEP] (13-M:L479): H-9 → dollars-per-resolved-task rollup
- **SUMMARY/REUSE-012** [DEP] (13-M:L480): J-3 → last-good binary rollback discipline
- **SUMMARY/REUSE-013** [DEP] (13-M:L481): J-49 → pre-existing per-seat admission/quota composition
- **SUMMARY/REUSE-014** [DEP] (13-M:L482): J-59 / MA-3 → per-seat memory/trust isolation
- **SUMMARY/REUSE-015** [DEP] (13-M:L483): K-0:531 → recorded exclusion of org/SSO/RBAC/multi-tenant build
- **SUMMARY/REUSE-016** [DEP] (13-M:L484): K-1 → dossier/claims lint, pricing recognizer, adoption-claim recognizer, K-1 deltas
- **SUMMARY/REUSE-017** [DEP] (13-M:L485): K-3 → unvouched counts and rendered evidence shapes
- **SUMMARY/REUSE-018** [DEP] (13-M:L486): K-4 → gate-tax metric
- **SUMMARY/REUSE-019** [DEP] (13-M:L487): K-5 → policy-table hash
- **SUMMARY/REUSE-020** [DEP] (13-M:L488): K-6 / MA-6 → compliance export and conformance path
- **SUMMARY/REUSE-021** [DEP] (13-M:L489): K-7 → buyer/positioning questions and commercial brief
- **SUMMARY/REUSE-022** [DEP] (13-M:L490): K-10 → authoring/execution order constraint
- **SUMMARY/REUSE-023** [DEP] (13-M:L491): K-11 → rendered evidence shapes
- **SUMMARY/REUSE-024** [DEP] (13-M:L492): K-24 → payment architecture, credit grants, pricing measurement, seats data
- **SUMMARY/REUSE-025** [DEP] (13-M:L493): L-13 → inert-data / non-executable organization policy rule
- **SUMMARY/REUSE-026** [DEP] (13-M:L494): ClockProvider / SeededClock → deterministic expiry lifecycle
- **SUMMARY/REUSE-027** [DEP] (13-M:L495): gate_freeze_check / gate manifest → pinned organization policy and schema evolution
- **SUMMARY/REUSE-028** [DEP] (13-M:L496): doctor → support diagnostics
- **SUMMARY/REUSE-029** [DEP] (13-M:L497): R1, R2, R8 → failure semantics, tamper refusal, frozen/golden-lock properties
- **SUMMARY/REUSE-030** [DEP] (13-M:L498): GD-2, GD-4, GD-5, KD-1, LOCK-3, TD-3, W-2, W-4, W-9 → explainability, drift timing, honesty, claims, killed scope, data-driven tiers, clean behavior, UI decision, event reuse/re-grep

### SUMMARY deferred / killed / conditional

Defer rows: **13**.

- **SUMMARY/DEFER-001** [DEFER] (13-M:L504): item=`SSO, SAML, SCIM, RBAC, hosted console` state=`Demand-gated enterprise rider, not minimal M` anchor=`M:L20-L21, L141-L142`
- **SUMMARY/DEFER-002** [DEFER] [superseded numbering note: see SUP-002; target M-7+] (13-M:L505): item=`Enterprise rider numbering` state=`Header says former M-6+ became M-7+; cross-cutting text still says M-6+` anchor=`M:L2-L4 vs M:L141`
- **SUMMARY/DEFER-003** [DEFER] (13-M:L506): item=`Automatic drift correction` state=`Deferred until at least GD-4` anchor=`M:L49`
- **SUMMARY/DEFER-004** [DEFER] (13-M:L507): item=`Exact license-status presentation` state=`Decide at edit under W-4` anchor=`M:L37-L38`
- **SUMMARY/DEFER-005** [DEFER] (13-M:L508): item=`OrgReportEvent` state=`Add only if no existing SSB event fits` anchor=`M:L69`
- **SUMMARY/DEFER-006** [DEFER] (13-M:L509): item=`ReleaseChannelEvent` state=`Optional; prefer an existing event if it fits` anchor=`M:L86-L87`
- **SUMMARY/DEFER-007** [DEFER] (13-M:L510): item=`Delta updates` state=`Later rider; v1 is whole-artifact` anchor=`M:L82-L83`
- **SUMMARY/DEFER-008** [DEFER] (13-M:L511): item=`Trust-continuity choice` state=`User must choose escrow-on-lapse or convert-to-OSS-on-abandonment; both drafted` anchor=`M:L98-L99`
- **SUMMARY/DEFER-009** [DEFER] (13-M:L512): item=`Article-50-certified hook` state=`Killed under LOCK-3` anchor=`M:L102`
- **SUMMARY/DEFER-010** [DEFER] (13-M:L513): item=`Final EEF name` state=`Chosen at execution` anchor=`M:L117`
- **SUMMARY/DEFER-011** [DEFER] (13-M:L514): item=`'Open standard' claim` state=`Deferred until at least one external emitter exists` anchor=`M:L124-L126`
- **SUMMARY/DEFER-012** [DEFER] (13-M:L515): item=`SLSA/in-toto precedent grounding` state=`Must be re-verified at execution` anchor=`M:L113-L115`
- **SUMMARY/DEFER-013** [DEFER] (13-M:L516): item=`Direct proof-spine contribution from M-6` state=`None; explicitly exempted` anchor=`M:L126`

### SUMMARY failure / fallback matrix

Failure rows: **19**.

- **SUMMARY/FAIL-001** [FAIL] (13-M:L522): trigger=`Entitlement passes expiry and grace` → result=`ExpiredGrace then DegradedToFree` consequence=`Free task remains usable; safety gates still fire` anchor=`M:L28-L29, L40-L41`
- **SUMMARY/FAIL-002** [FAIL] (13-M:L523): trigger=`Forged entitlement` → result=`RefusedForged / refusal` consequence=`No Team unlock` anchor=`M:L36-L42`
- **SUMMARY/FAIL-003** [FAIL] (13-M:L524): trigger=`Excess seat activation` → result=`Record it` consequence=`Surface through fleet report; do not silently absorb or brick` anchor=`M:L32-L33, L41-L42`
- **SUMMARY/FAIL-004** [FAIL] (13-M:L525): trigger=`Forged/tampered org pack` → result=`RefusedForged / refusal` consequence=`Pack is not applied` anchor=`M:L53-L56`
- **SUMMARY/FAIL-005** [FAIL] (13-M:L526): trigger=`Local weakening or cap raise` → result=`WeakenAttemptBlocked` consequence=`Organization floor remains effective` anchor=`M:L47-L48, L53-L56`
- **SUMMARY/FAIL-006** [FAIL] (13-M:L527): trigger=`Stale org pack` → result=`Drift` consequence=`Surface at boot; do not auto-correct yet` anchor=`M:L48-L49, L57`
- **SUMMARY/FAIL-007** [FAIL] (13-M:L528): trigger=`No org pack` → result=`Baseline behavior` consequence=`Byte-identical single-seat behavior` anchor=`M:L50, L56-L57`
- **SUMMARY/FAIL-008** [FAIL] (13-M:L529): trigger=`Broken seat evidence chain` → result=`Refuse and name seat` consequence=`Exclude it from verified seats; no anonymous absorption` anchor=`M:L65-L66, L70-L71`
- **SUMMARY/FAIL-009** [FAIL] (13-M:L530): trigger=`Secret in fleet constituent` → result=`Fail report build` consequence=`Prevent secret-bearing organization bundle` anchor=`M:L71-L72`
- **SUMMARY/FAIL-010** [FAIL] (13-M:L531): trigger=`Transcript bytes in fleet bundle` → result=`Test failure` consequence=`Fleet output must remain content-free` anchor=`M:L72`
- **SUMMARY/FAIL-011** [FAIL] (13-M:L532): trigger=`Corrupt update artifact` → result=`Refuse before file lands` consequence=`Preserve current binary/files` anchor=`M:L88`
- **SUMMARY/FAIL-012** [FAIL] (13-M:L533): trigger=`Failed/new binary rollback request` → result=`Restore last-good hash` consequence=`Deterministic recovery` anchor=`M:L80-L81, L88-L89`
- **SUMMARY/FAIL-013** [FAIL] (13-M:L534): trigger=`Seeded secret in support session` → result=`Redact; grep stays empty` consequence=`Bundle remains local and is never auto-sent` anchor=`M:L81-L82, L89-L90`
- **SUMMARY/FAIL-014** [FAIL] (13-M:L535): trigger=`One-byte verifier artifact tamper` → result=`Verification failure` consequence=`Works without engine installation` anchor=`M:L107-L109`
- **SUMMARY/FAIL-015** [FAIL] (13-M:L536): trigger=`Pricing superlative without evidence` → result=`Claims-lint failure` consequence=`Blocks unsupported pricing claim` anchor=`M:L101, L109`
- **SUMMARY/FAIL-016** [FAIL] (13-M:L537): trigger=`Missing conformance evidence` → result=`FAIL` consequence=`Normative fail-closed format behavior` anchor=`M:L121-L123, L129-L130`
- **SUMMARY/FAIL-017** [FAIL] (13-M:L538): trigger=`Evidence artifact one-byte tamper` → result=`conform failure` consequence=`Engine-free independent detection` anchor=`M:L129-L130`
- **SUMMARY/FAIL-018** [FAIL] (13-M:L539): trigger=`Schema field removal` → result=`Golden-lock failure` consequence=`Append-only format evolution preserved` anchor=`M:L123-L124, L131`
- **SUMMARY/FAIL-019** [FAIL] (13-M:L540): trigger=`'Industry standard' without adoption row` → result=`Claims-lint failure` consequence=`Only 'published format' is allowed` anchor=`M:L124-L126, L131-L132`

### SUMMARY observability / evidence inventory

Obs rows: **8**.

- **SUMMARY/OBS-001** [OBS] (13-M:L546): family=`EntitlementEvent` contents=`Activated, Renewed, ExpiredGrace, DegradedToFree, Revoked, RefusedForged` ssb=`New Pattern-2 family`
- **SUMMARY/OBS-002** [OBS] (13-M:L547): family=`OrgPolicyEvent` contents=`Applied, Drift, RefusedForged, WeakenAttemptBlocked` ssb=`New Pattern-2 family`
- **SUMMARY/OBS-003** [OBS] (13-M:L548): family=`OrgReportEvent` contents=`No variants specified` ssb=`Prefer reuse; conditional new family only if nothing fits`
- **SUMMARY/OBS-004** [OBS] [superseded detail by SUP-003; deep names arms] (13-M:L549): family=`ReleaseChannelEvent` contents=`No variants specified` ssb=`Optional; prefer reuse`
- **SUMMARY/OBS-005** [OBS] (13-M:L550): family=`M-5 verifier/package` contents=`Verification outcomes exist, but summary says NO SSB` ssb=`No new SSB`
- **SUMMARY/OBS-006** [OBS] (13-M:L551): family=`M-6 format/docs` contents=`Conformance verdicts/receipts are specified as format shapes, but summary says NO SSB for this work` ssb=`No new SSB`
- **SUMMARY/OBS-007** [OBS] (13-M:L552): family=`Fleet bundle` contents=`Seat table; verified-outcome, gate-tax, K-1 delta, cost, unvouched, drift, roster-honesty rollups` ssb=`Recomputed from constituent journals`
- **SUMMARY/OBS-008** [OBS] (13-M:L553): family=`Support bundle` contents=`Versions, tier, hashes, doctor, redacted journal tail` ssb=`Local diagnostic; never auto-sent`

### SUMMARY acceptance / test coverage index

Accept rows: **6**.

- **SUMMARY/ACCEPT-001** [TEST] (13-M:L561): cap=`M-1` pos=`Offline Team activation and unlock` neg=`Forged signature refusal; over-activation recorded` det=`Seeded expiry chain; Free task remains green; all safety gates fire`
- **SUMMARY/ACCEPT-002** [TEST] (13-M:L562): cap=`M-2` pos=`Pack Applied; org cap blocks and names clause` neg=`Tampered pack refused; local weakening blocked; stale pack drift` det=`No-pack seat byte-identical`
- **SUMMARY/ACCEPT-003** [TEST] (13-M:L563): cap=`M-3` pos=`Clean and drift exports pass into one bundle` neg=`Broken chain refused/named; seeded secret fails; transcript grep empty` det=`Rollups recompute byte-identically`
- **SUMMARY/ACCEPT-004** [TEST] (13-M:L564): cap=`M-4` pos=`Clean apply succeeds; support bundle generated` neg=`Corrupt artifact refused before disk; beta hidden by stable pin; secret grep empty` det=`Rollback binary hash equals prior binary`
- **SUMMARY/ACCEPT-005** [TEST] (13-M:L565): cap=`M-5` pos=`Engine-free verification of attestation/RRA/org bundle; license status renders tier data` neg=`One-byte tamper caught; pricing superlative linted` det=`Crate compiles with zero engine dependencies`
- **SUMMARY/ACCEPT-006** [TEST] (13-M:L566): cap=`M-6` pos=`Conformant fixture and third-party example pass conform` neg=`Tamper, missing field, field removal, and unsupported adoption claim fail` det=`Derived schema byte-matches regeneration on engine-free verifier path`

### SUMMARY inferred behavior (non-normative)

Inference records: **11**.

- **SUMMARY/INF-SSB-NOTE** [I] (13-M:L555): The summary’s +2..+4 SSB estimate is consistent with two mandatory families plus two conditional/reuse-first families; this consistency statement is an inference, not an additional requirement.
- **SUMMARY/INF-001** [I] (13-M:L572): The +2..+4 SSB estimate likely means EntitlementEvent and OrgPolicyEvent are mandatory while OrgReportEvent and ReleaseChannelEvent account for the optional upper bound. This follows from M:L10, L36-L37, L53-L54, L69, and L86-L87.
- **SUMMARY/INF-002** [I] (13-M:L573): 'DegradedToFree' likely removes paid/Team capabilities because tier() gates them, but the exact feature-by-feature removal list is not stated.
- **SUMMARY/INF-003** [I] (13-M:L574): A revoked entitlement likely changes tier availability, but the summary names Revoked without stating the resulting seat state or grace behavior.
- **SUMMARY/INF-004** [I] (13-M:L575): 'deactivate' likely removes the local activation, but its local-state semantics and whether it emits Revoked or another event are not stated.
- **SUMMARY/INF-005** [I] (13-M:L576): The M-3 command’s '--verify' likely controls mandatory chain verification, but behavior without the flag is not described.
- **SUMMARY/INF-006** [I] (13-M:L577): The roster-honesty rollup likely consumes M-1’s recorded over-activation signal; the exact reconciliation algorithm is not specified.
- **SUMMARY/INF-007** [I] (13-M:L578): An organization-scheduled apply likely comes from OrgPolicyPack, but schedule fields are not enumerated among the listed pack clauses.
- **SUMMARY/INF-008** [I] (13-M:L579): The revocation list likely covers entitlement and/or release keys, but its file format, refresh transport, and offline staleness semantics are absent.
- **SUMMARY/INF-009** [I] (13-M:L580): 'published format' likely includes a distribution location and version negotiation, but neither registry nor URL is specified.
- **SUMMARY/INF-010** [I] (13-M:L581): Legal-document generation may be documentation-only rather than runtime behavior; no command surface is stated.

### SUMMARY ambiguities and missing decisions

Ambiguity records: **20**.

- **SUMMARY/AMB-001** [A] [partially resolved numbering by SUP-002 / deep M-7+] (13-M:L585): Enterprise-rider numbering conflicts inside the summary: M:L2-L4 says M-6+ was renumbered M-7+, but M:L141 still calls it M-6+. Treat M-7+ as the newer header correction until M-0 is inspected.
- **SUMMARY/AMB-002** [A] (13-M:L586): The entitlement signature algorithm, canonical byte representation, key rotation, revocation-list structure, and local entitlement storage location are absent.
- **SUMMARY/AMB-003** [A] (13-M:L587): Grace-period start conditions, renewal timing, clock-skew policy, offline expiry messaging, and deactivation semantics are absent.
- **SUMMARY/AMB-004** [A] (13-M:L588): 'Seat-count best-effort' does not define seat identity issuance, duplicate-device rules, reconciliation cadence, or privacy-preserving roster representation.
- **SUMMARY/AMB-005** [A] [SettingsWindow: see SUP-001] (13-M:L589): The SettingsWindow license-status layout and states are deliberately left to edit-time W-4.
- **SUMMARY/AMB-006** [A] (13-M:L590): OrgPolicyPack serialization, schema version, signer/key custody, precedence ordering, file discovery path, git workflow, and stale threshold are absent.
- **SUMMARY/AMB-007** [A] (13-M:L591): 'Drift' does not specify which comparisons trigger it, how pack_hash relates to effective configuration, or the user remediation flow.
- **SUMMARY/AMB-008** [A] (13-M:L592): M-3 does not specify organization-bundle serialization, output path, command behavior without '--verify', partial-output handling after refusals, or whether a broken seat is omitted versus included as refused.
- **SUMMARY/AMB-009** [A] (13-M:L593): The summary names '2 verify PASS' for clean/drift fixtures; it does not say whether Drift affects an overall organization verdict.
- **SUMMARY/AMB-010** [A] (13-M:L594): M-4 does not define manifest schema/version, download transport, polling interval, failure backoff, channel-switch authorization, install atomicity, platform support, or retention count beyond one last-good binary.
- **SUMMARY/AMB-011** [A] (13-M:L595): ReleaseChannelEvent and OrgReportEvent are unresolved reuse decisions; their event variants are absent.
- **SUMMARY/AMB-012** [A] (13-M:L596): Support-bundle file format, destination, size/tail limit, exact redaction policy, encryption, retention, and user transmission flow are absent.
- **SUMMARY/AMB-013** [A] (13-M:L597): The selected trust-continuity model is unresolved between escrow-on-lapse and convert-to-OSS-on-abandonment.
- **SUMMARY/AMB-014** [A] (13-M:L598): Legal drafts explicitly require counsel review; they are not represented as legally approved final documents.
- **SUMMARY/AMB-015** [A] (13-M:L599): Exact tier entitlements, pricing, billing interval, taxes, currency, and K-24 credit-grant values are absent.
- **SUMMARY/AMB-016** [A] (13-M:L600): 'K-6 conformance' versus M-6 format conformance naming may represent different layers; originals must disambiguate.
- **SUMMARY/AMB-017** [A] (13-M:L601): The final name, media type, version-negotiation rules, canonical encoding, extension namespaces, unknown-field handling, and compatibility window for EEF v1 are absent.
- **SUMMARY/AMB-018** [A] (13-M:L602): 'Append-only schema evolution' does not state whether fields can be deprecated, made optional, or moved into a new major version.
- **SUMMARY/AMB-019** [A] (13-M:L603): The adoption row’s schema, authority, and verification process are absent.
- **SUMMARY/AMB-020** [A] (13-M:L604): M-6 precedent is explicitly unsearched and requires execution-time verification.

### SUMMARY deeper-source manifest rows

Deeper-source / missing-candidate records: **16**.

- **SUMMARY/DEEPER-001** [DEEPER] (13-M:L612): Doctrine, MA-1..MA-7 pointers, gate, enterprise rider | docs/thefinalglance/M-plan/M-0_INDEX.md | Resolve doctrine details, prerequisites, rider numbering, source map
- **SUMMARY/DEEPER-002** [DEEPER] (13-M:L613): M-1 original | docs/thefinalglance/M-plan/M-1_seat_entitlement_and_purchase_rail.md | Entitlement schema/lifecycle, keys, CLI and seat accounting details
- **SUMMARY/DEEPER-003** [DEEPER] (13-M:L614): M-2 original | docs/thefinalglance/M-plan/M-2_org_policy_pack_distribution.md | Pack schema, precedence, distribution, drift and event details
- **SUMMARY/DEEPER-004** [DEEPER] (13-M:L615): M-3 original | docs/thefinalglance/M-plan/M-3_fleet_evidence_rollup.md | Bundle schema, verifier behavior, metrics fields, partial failure
- **SUMMARY/DEEPER-005** [DEEPER] (13-M:L616): M-4 original | docs/thefinalglance/M-plan/M-4_signed_release_channel_updater_support.md | Manifest/update protocol, rollback, support-bundle format
- **SUMMARY/DEEPER-006** [DEEPER] (13-M:L617): M-5 original | docs/thefinalglance/M-plan/M-5_commercial_packaging_verifier_oss.md | Verifier crate boundary, legal artifacts, tier truth, continuity choice
- **SUMMARY/DEEPER-007** [DEEPER] (13-M:L618): M-6 original | docs/thefinalglance/M-plan/M-6_evidence_schema_open_standard.md | Schema generation, conformance contract, evolution and publication
- **SUMMARY/DEEPER-008** [DEEPER] (13-M:L626): M-6 spec-eval ADD-1 | workflow/reasoning/elai-spec-eval-ambition-2026-07-05.md | M:L2-L4, L113-L115
- **SUMMARY/DEEPER-009** [DEEPER] (13-M:L627): Multi-seat gap analysis | workflow/reasoning/steal-multiseat-gap-2026-07-02.md | M:L5-L7
- **SUMMARY/DEEPER-MISS-001** [DEEPER] (13-M:L662): MA-1 through MA-7 are mentioned but only MA-3, MA-4, and MA-6 are used by name in the summary; their exact targets and the unused MA pointers require M-0 inspection.
- **SUMMARY/DEEPER-MISS-002** [DEEPER] (13-M:L663): The enterprise rider’s authoritative identifier needs M-0 verification because M-6+ and M-7+ both appear.
- **SUMMARY/DEEPER-MISS-003** [DEEPER] (13-M:L664): B-45 and K-10 lack matching top-level plan filenames in their respective directory inventories.
- **SUMMARY/DEEPER-MISS-004** [DEEPER] (13-M:L665): K A-1..A-4 and K-24 L1..L4 are prerequisite substeps, not standalone paths in this summary; resolve within K originals.
- **SUMMARY/DEEPER-MISS-005** [DEEPER] (13-M:L666): seats.toml is named as a K-24 composition artifact at M:L34, while tiers.toml is M-5’s pricing truth; originals must clarify their distinct roles.
- **SUMMARY/DEEPER-MISS-006** [DEEPER] (13-M:L667): 'Pattern-2', frontend_pair, org-immune tier, and the named rule families need canonical definitions from M-0/upstream sources.
- **SUMMARY/DEEPER-MISS-007** [DEEPER] (13-M:L668): Implementation destinations named in the summary are planned targets, not confirmed source files: crates/orchestrator/src/commerce/entitlement.rs, commerce/org_policy.rs, commerce/org_report.rs, crates/cli/src/update.rs, docs/strategy/commercial/, tiers.toml, and the elai-verify crate.

---

## Deep ledger exhaustiveness footer (preserved)

- **M-001/AUDIT-001** [E/EVIDENCE] (DEEP-M-001.discovery.md:L576-L578): Source coverage 7/7 files, 542/542 lines, 38,550/38,550 bytes; SHA mismatches 0.
- **M-001/AUDIT-002** [E/EVIDENCE] (DEEP-M-001.discovery.md:L578-L580): Atomic source ledger 360 unique = 323 E + 7 I + 29 A + 1 C; per-file M-0=63 M-1=59 M-2=44 M-3=47 M-4=47 M-5=47 M-6=53.
- **M-001/AUDIT-003** [E/EVIDENCE] (DEEP-M-001.discovery.md:L580-L581): Relational IX=24, SD=39, AC=12; not double-counted as features.
- **M-001/AUDIT-004** [E/EVIDENCE] (DEEP-M-001.discovery.md:L581-L582): ID audit: unique IDs matching M0..M6-[EIAC]nnn; duplicate/missing/nonconforming atomic IDs = 0.
- **M-001/AUDIT-005** [E/EVIDENCE] (DEEP-M-001.discovery.md:L586-L588): output_lines=0588; output_bytes=81798; canonical normalized SHA-256 6e89c168aebda2d7ea262996eb61e9d5f0b48ecca42989c543847fe77faf9c34.

---

## Closing audit block

- Deep ledgers merged: **1/1** (DEEP-M-001)
- Summary record merged: **1/1** (`13-M-SUMMARY_INDEX.discovery.md`)
- Total inputs: **2/2** (1/1 deep + summary)

### Per-ledger atom counts

| Ledger | Source | IX | SD | AC | Meta/header/footer | Subtotal |
|---|---:|---:|---:|---:|---:|---:|
| M-001 | 360 | 24 | 39 | 12 | 8 | **443** |
| SUMMARY | 304 inv + 179 extra | — | — | — | 2 | **485** |
| **Aggregate distinct record count** | | | | | | **928** |

### Deep-ledger source-atom breakdown (traceable)

- **M-0** = 63
- **M-1** = 59
- **M-2** = 44
- **M-3** = 47
- **M-4** = 47
- **M-5** = 47
- **M-6** = 53
- Sum source atoms = **360** (declared 360 — exact)
- IX=24, SD=39, AC=12

### Summary breakdown (traceable)

- Inventory `M-wide product boundary, doctrine, and sequencing` = 42
- Inventory `M-1 — seat entitlement + purchase rail` = 52
- Inventory `M-2 — org policy pack + distribution + drift detection` = 38
- Inventory `M-3 — fleet evidence rollup` = 40
- Inventory `M-4 — signed release channel + auto-updater + support bundle` = 44
- Inventory `M-5 — commercial packaging + verifier OSS + trust continuity` = 46
- Inventory `M-6 — evidence format as a versioned open standard` = 42
- Inventory total = **304** (declared 304 — exact)
- Workflow steps = 22
- Cross-feature invariants = 15
- Surfaces table rows = 16
- Dep/order/reuse rows = 33
- Defer rows = 13
- Failure matrix rows = 19
- Observability rows = 8
- Acceptance rows = 6
- Inferences = 11
- Ambiguities = 20
- Deeper-source rows = 16

- Aggregate distinct record count: **928**
  - Core (deep source+IX+SD+AC + summary inv+extra): **918**
  - Plus preserved header/footer meta records: **10**
- Superseded notes count: **4**
- User-facing surface index entries (deep): **73**
- User-facing surface index entries (summary CLI/UI/ACTION-related): **44**
- User-facing surface table rows (summary): **16**
- Cross-file interaction index entries: **24**
- Summary-delta index entries: **39**
- Ambiguity/conflict index entries (deep AC): **12**

### Zero-omission statement

ZERO-OMISSION: The single deep ledger DEEP-M-001 and the family summary record 13-M-SUMMARY_INDEX.discovery.md were merged (1/1 + summary).
Every source-local atomic ID extracted from the deep ledger (360 = M-0..M-6 [EIAC] atoms) is present under namespace M-001/.
Cross-file interactions (IX-*), summary deltas (SD-*), and ambiguity/conflict registers (AC-*) are namespaced as M-001/IX-*, M-001/SD-*, M-001/AC-*.
Every summary inventory atom (304) is present under namespace SUMMARY/.
Summary relational sections (workflows, invariants, surfaces table, deps, defer, failure matrix, observability, acceptance, inferences, ambiguities, deeper-source) are preserved as distinct SUMMARY/* records.
Losing target-state descriptions are retained under Superseded notes rather than deleted.
No deep-ledger atom was compressed into prose-only form; each remains an ID-bearing bullet with source anchor.

ZERO-DUPLICATE: Plan-local IDs remain unique within their ledger namespace (M-001/ vs SUMMARY/).
Summary-layer atoms are orientation records and may restate deep content; deep records remain authoritative for implement (SUP-004).

### Count reconciliation

- **M-001 source atoms** exact match to ledger footer 360: **YES**
- **M-001 IX/SD/AC** exact match to ledger footer 24/39/12: **YES**
- **SUMMARY inventory** vs declared 304: **304/304** (exact)
- Per-file deep: M-0=63 M-1=59 M-2=44 M-3=47 M-4=47 M-5=47 M-6=53

### Output path

`docs/thefinalglance/feature-discovery/aggregates/M-AGGREGATE.md`
