# G-9: SSB/log write-time secret+PII redaction gate + DB-file-granularity TTL

STATUS: PLANNED (WAVE 2; G-2 precondition; LOCAL-only hand-rolled redactor, no cloud egress, no new dep)

Objective: a LOCAL-ONLY (no cloud egress, unlike ggshield/Nightfall) fail-closed redactor on the single-writer
SSB commit path AND the hook-log sink, replacing detected secrets/PII with stable blake3-keyed tokens BEFORE the
row is committed, so the append-only journal stays valid (ssb.rs:5 "no UPDATE or DELETE" forces WRITE-TIME
redaction — you cannot scrub a row after it lands). Retention is by purging whole expired session-scoped sinks
(the per-session hook-executions.log file; for the single shared ssb.db, a session-scoped row purge or a
whole-file TTL — see the honest caveat below), since individual rows cannot be deleted. The redactor REUSES the
shipped F-1 secret regex set (vibegate_secrets.rs::SECRET_PATTERNS) — it does NOT re-implement detection and adds
NO Presidio/ggshield dependency. This is the precondition for shipping the G-2 RRA / G-8 attestation to a client
without being a secrets-exposure liability. Grounding is MODERATE: PII/secret regex detection is inherently
high-recall / low-precision, so the false-positive (over-redaction) rate is bounded before default-on (GD-4).
Estimated effort: ~M (~8-12 h). Pure in-process, fail-closed at the write path.

Pre-requisites: the SSB single-writer commit path SHIPPED (grep-confirmed 2026-06-02: write_payload ssb.rs:14265
-> write:14249 -> write_loop:14354 -> execute_write:14400, INSERT binds request.payload at :14429 between BEGIN
IMMEDIATE :14417 and COMMIT :14449; append-only invariant ssb.rs:5). The F-1 secret regex set SHIPPED
(vibegate_secrets.rs::SECRET_PATTERNS:54). The hook-log sink SHIPPED (skill_telemetry.rs::log_skill_invocation:166,
hook-executions.log path :172-176). blake3 = "1.8" workspace dep (Cargo.toml:65; source_blake3_prefix:148 is the
token precedent). Presidio/ggshield/gitguardian ABSENT (hand-rolled confirmed). G-2 RRA is a FORWARD DEPENDENCY
(rra.rs absent — G-9 is its precondition).
Parallel-safe with: G-3, G-4, G-8 (disjoint files). NOT parallel-safe with G-2 (G-9 is its precondition — coordinate)
or any plan editing ssb.rs's execute_write / write path in the same window.
Source: workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R9. Doctrine: G-0_INDEX.md (W-1..W-10 + GD-1..GD-5).

## Context

Per the trust-max synthesis R9: the SSB payload carries the raw prompt + tool I/O, and the hook log carries the
raw hook input — both can contain secrets / PII (GitGuardian found 24,008 MCP-config secrets in the wild and a
3.2% Claude-Code leak rate, CONFIRMED). The append-only invariant (ssb.rs:5 "no UPDATE or DELETE") means you
CANNOT scrub a row after it lands — redaction MUST happen at write-time, before the INSERT. The redactor is
LOCAL-ONLY (no cloud egress, unlike ggshield/Nightfall — a redactor that phones home is itself a leak), fail-closed
(if detection errors, the field is redacted, not passed through), and replaces each detected secret/PII span with a
stable blake3-keyed token so the row stays structurally valid and two occurrences of the same secret map to the same
token (correlatable without exposure). Retention is by purging whole expired session sinks, since rows cannot be
deleted.

The redactor REUSES the shipped F-1 detection (vibegate_secrets.rs::SECRET_PATTERNS, a LazyLock<RegexSet> of
AWS-key / PEM / secret-assignment shapes) — G-9 adds the WRITE-TIME mutation + the token substitution + the TTL
purge, not a new detector (GD-1 COMPOSES-WITH). It adds NO Presidio/ggshield dependency (both ABSENT in-tree;
hand-rolled is the posture). Grounding is MODERATE: regex/pattern secret+PII detection is high-recall but
low-precision, so the OVER-REDACTION rate is the GD-4 release gate — bounded on >=3 real runs before default-on.

ELAI ground truth (grep+source-confirmed 2026-06-02): the SSB single-writer chokepoint is write_payload
(ssb.rs:14265, typed front door) -> Ssb::write (:14249, mpsc enqueue) -> write_loop (:14354, the SOLE writer task) ->
execute_write (:14400) where INSERT INTO ssb (... payload ...) binds request.payload at :14429 between BEGIN IMMEDIATE
(:14417) and COMMIT (:14449) — so the redactor mutates request.payload inside execute_write before the INSERT. The
append-only no-UPDATE-no-DELETE invariant is at ssb.rs:5. The destructive migration DROP (F-13's site) is in
run_migration (ssb.rs:14334), executing `DROP TABLE IF EXISTS ssb_files; DROP TABLE IF EXISTS ssb;` at :14342 only
when an on-disk schema version mismatches. The hook-log sink is skill_telemetry.rs::log_skill_invocation (:166),
append-writing JSON lines to sessions/<session_id>/hook-executions.log (path :172-176, append :190, write :191-192,
wire-dark via is_enabled()/ELAI_SKILL_SYSTEM_ENABLE :89); read side commands.rs::get_hook_executions (:4367/:4403).
The F-1 detector is vibegate_secrets.rs::SECRET_PATTERNS (:54, PAT_AWS_KEY AKIA[0-9A-Z]{16}:27, PAT_PEM_TYPED:30,
PAT_PEM_GENERIC:33, PAT_SECRET_ASSIGN:42, SECRET_LABELS:62), with CWE-95 in cwe_classifier.rs (:58/:70/:95) and
CWE-798/321/532 surfaced via vibegate cwe_from_rule_id (NOT a standalone secret-regex file — VERIFY the exact
CWE-798 seed-rule location before citing a line). Stable blake3 tokens follow vibegate.rs::source_blake3_prefix
(:148-149, blake3::hash().to_hex() first-16) + ssb.rs:25728 blake3::hash(label); blake3 is a workspace dep
(Cargo.toml:65). The session-DB layout is a SINGLE shared <base>/.elai_cc/sessions/ssb.db (SSB_DB_FILE_NAME ssb.rs:14745,
SSB_SESSIONS_SUBDIR ".elai_cc/sessions":14741, default_ssb_path_for_base:14758) with session_id as a COLUMN — NOT
per-session ssb.db files; only the hook-executions.log is per-session. An optional wire-dark advisory secret_redacted
action_type is precedented (ELAI_*_ENABLE gating, ssb.rs:1869/:1882/:1890). Presidio/ggshield/gitguardian ABSENT.
rra.rs ABSENT (G-9 is the G-2 precondition). SCHEMA_VERSION=205 (ssb.rs:2898), TOTAL_VARIANTS=482 (tests/action_type_lock.rs:534).

## Existing state (B/C/D/E/F coverage)
- SSB single-writer path (crates/orchestrator/src/ssb.rs) — SHIPPED. write_payload:14265 -> write:14249 -> write_loop:14354
  -> execute_write:14400 (INSERT binds payload :14429, BEGIN IMMEDIATE :14417, COMMIT :14449). Append-only ssb.rs:5.
  G-9 mutates request.payload inside execute_write before the INSERT; re-implements no part of the write path (GD-1).
- run_migration (ssb.rs:14334, DROP :14342) — SHIPPED. The destructive schema-bump DROP; relevant to the DB-file TTL
  (distinct from the per-session row purge — see caveat). F-13 cites this site.
- Hook-log sink: skill_telemetry.rs::log_skill_invocation:166 (hook-executions.log :172-176, append :190, wire-dark :89) +
  get_hook_executions (commands.rs:4367) — SHIPPED. The second sink G-9 redacts (read back unredacted today).
- F-1 detector: vibegate_secrets.rs::SECRET_PATTERNS:54 (AWS/PEM/secret-assign, SECRET_LABELS:62) + cwe_classifier.rs:58
  (CWE-95 :70; CWE-798/321/532 via vibegate cwe_from_rule_id) — SHIPPED. The detection G-9 REUSES (no new detector, no dep).
- blake3 token precedent: vibegate.rs::source_blake3_prefix:148 + ssb.rs:25728 blake3::hash(label). blake3 Cargo.toml:65.
- Session-DB layout: SINGLE shared <base>/.elai_cc/sessions/ssb.db (ssb.rs:14741/:14745/:14758), session_id a COLUMN.
  hook-executions.log IS per-session. Presidio/ggshield/gitguardian ABSENT (hand-rolled).
- G-2 RRA (rra.rs) — ABSENT. G-9 is its PRECONDITION (the artifact must not export unredacted secrets).
- ssb.rs — SCHEMA 205 (:2898), TOTAL_VARIANTS 482 (tests/action_type_lock.rs:534). Wire-dark advisory precedent ssb.rs:1869/:1882/:1890.

## Reads (mandatory before coding)
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R9 + GD-4 (the over-redaction FP gate) + GD-5 (no over-claim)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:1-10,14245-14460 (the append-only header + write_payload/write/write_loop/execute_write + run_migration)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:14740-14770 (SSB_DB_FILE_NAME / SSB_SESSIONS_SUBDIR / default_ssb_path_for_base — the DB layout)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\vibegate_secrets.rs (full — SECRET_PATTERNS + SECRET_LABELS, the detector to reuse)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\vibegate.rs:140-160 (source_blake3_prefix — the stable token precedent)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\skill_telemetry.rs:80-200 (log_skill_invocation + the hook-log path + is_enabled gate)
- microsoft.github.io/presidio docs (for any qualitative PII-detection framing — do NOT cite the REFUTED PRvL numbers)
- docs/strategy/ELAI-2026-POSTURE.md (the EU AI Act penalty tier — E-38 HARD RULE)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, anti-slopsquatting (NO Presidio/ggshield), SCHEMA baseline+N.

## Tasks (WAVE 2)

1. **Add the local redactor (~150 LOC in a new crates/orchestrator/src/redaction.rs).** `pub fn redact(text: &str) ->
   (String, u32)` returning the text with every SECRET_PATTERNS (vibegate_secrets.rs:54) match replaced by a stable
   `[REDACTED:<label>:<blake3-16>]` token (the blake3 prefix of the secret value, so identical secrets map to identical
   tokens — correlatable without exposure; the source_blake3_prefix:148 precedent) plus the count of redactions. `pub fn
   redact_payload(payload: &mut SsbPayload)` walks the payload's text-bearing fields and redacts in place. LOCAL-ONLY (no
   network), fail-closed (a detection error redacts the field rather than passing it). NO `.unwrap()`/`.expect()`. REUSES
   SECRET_PATTERNS — does NOT define a new regex set; adds NO Presidio/ggshield dependency.

2. **Wire the redactor into the single write path (~40 LOC delta in ssb.rs execute_write).** Call redact_payload on
   request.payload at the TOP of execute_write (ssb.rs:14400), BEFORE the INSERT binds it (:14429) — so no unredacted
   secret ever lands in ssb.db (the append-only invariant ssb.rs:5 forbids fixing it later). Default-ON (W-2): kill-switch
   ELAI_REDACT_DISABLE only; the observable_effect runs with NO override.

3. **Wire the redactor into the hook-log sink (~25 LOC delta in skill_telemetry.rs).** Call redact() on the serialized
   LogLine before the append-write (skill_telemetry.rs:191) so the per-session hook-executions.log never lands a raw secret.

4. **DB-file-granularity TTL (~70 LOC in redaction.rs / a retention module).** `pub fn purge_expired(base: &Path, ttl:
   Duration) -> PurgeReport` purging whole expired per-session sinks: the per-session sessions/<id>/hook-executions.log is
   file-granular (delete the whole expired file). HONEST CAVEAT (the single-shared-ssb.db reality): ssb.db is ONE shared
   file per project base with session_id as a column (ssb.rs:14741/:14758), NOT a per-session DB — so SSB retention is
   either a session-scoped row purge that respects the append-only invariant (a controlled vacuum at a maintenance
   boundary, NOT mid-session UPDATE/DELETE) or a whole-file TTL across all sessions; the plan states which it does and does
   NOT pretend ssb.db is per-session. NO mid-session row DELETE (stub_ban — violates ssb.rs:5).

5. **(Optional) wire-dark advisory secret_redacted action_type (~50 LOC in ssb.rs, W-9).** G-0 says NO new MANDATORY
   action_type (redaction mutates the write path). An OPTIONAL advisory `secret_redacted` row (count + sink + session_id,
   NO secret value) gated wire-dark by ELAI_REDACT_ADVISORY_ENABLE follows the ssb.rs:1869/:1882/:1890 wire-dark precedent.
   If shipped: sql_str + parse + CHECK + golden; SCHEMA bump baseline 205 (ssb.rs:2898) -> 206, TOTAL_VARIANTS 482
   (tests/action_type_lock.rs:534) -> 483 + golden regen. (This is the ONE deliberate W-2 deviation: an ADVISORY row may be
   env-gated; the REDACTION ITSELF is default-ON.)

6. **Default-ON kill-switch + GD-4 over-redaction FP gate (~30 LOC).** ELAI_REDACT_DISABLE kill-switch only; default ON.
   The release gate (GD-4, MODERATE grounding): measure the over-redaction (false-positive) rate on >=3 real orchestrate
   runs; a contract test asserts 0 false redactions on a 10-benign-payload corpus (no real-but-not-secret string is
   tokenized). Over-redaction that corrupts a payload is the failure mode to bound.

7. **Frontend pair: a redaction status surface (~140 LOC svelte + ~90 LOC tauri) (W-4).** A RedactionPanel (or extend the
   security drawer) showing redaction counts per session + the (optional) secret_redacted advisory rows (NEVER the secret
   value); Tauri `get_redaction_stats`. cross_lang_diff.py PASS.

8. **Decision doc workflow/reasoning/G-9-decisions.md (~110 LOC).** Records: (a) WRITE-TIME redaction is forced by the
   append-only invariant (ssb.rs:5) — cannot scrub after; (b) REUSE the F-1 SECRET_PATTERNS, hand-rolled, NO Presidio/ggshield
   dep; (c) stable blake3-keyed tokens (source_blake3_prefix precedent); (d) the single-shared-ssb.db reality — TTL is
   per-session-file for the hook log + session-scoped/whole-file for ssb.db, NOT per-session-DB; (e) CITATION HYGIENE — the
   PRvL Presidio 0.83/0.88/0.85 + GPT-4o numbers are REFUTED, use the qualitative high-recall/low-precision framing,
   re-source from Presidio docs; EU penalty EUR 15M/3% (Art-99), NOT 35M/7%; (f) MODERATE grounding -> the over-redaction
   rate is bounded before default-on (GD-4); (g) G-9 is the G-2/G-8 precondition (the artifact must not export unredacted secrets).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/redaction.rs::redact
  - crates/orchestrator/src/redaction.rs::redact_payload
  - crates/orchestrator/src/redaction.rs::purge_expired
  - crates/dashboard-v2/src-tauri/src/commands_g9.rs::get_redaction_stats
  - crates/dashboard-v2/src/lib/drawers/RedactionPanel.svelte
  - (optional) crates/orchestrator/src/ssb.rs::SsbActionType::SecretRedacted + SecretRedactedPayload
consumes_symbols:
  - crates/orchestrator/src/ssb.rs::execute_write (the single-writer INSERT chokepoint, :14400/:14429)
  - crates/orchestrator/src/ssb.rs::write_payload (the typed front door, :14265)
  - crates/orchestrator/src/vibegate_secrets.rs::SECRET_PATTERNS (the F-1 detector to REUSE, :54) + SECRET_LABELS (:62)
  - crates/orchestrator/src/vibegate.rs::source_blake3_prefix (the stable-token precedent, :148)
  - crates/orchestrator/src/skill_telemetry.rs::log_skill_invocation (the hook-log sink, :166/:191)
  - crates/orchestrator/src/ssb.rs::SSB_SESSIONS_SUBDIR (:14741) + default_ssb_path_for_base (:14758) (the DB layout for TTL)
  - the G-2 RRA export path (FORWARD DEPENDENCY — rra.rs absent; G-9 is its precondition, W-7 do-not-cite-as-present)
reachable_from:
  - crates/orchestrator/src/ssb.rs::execute_write (:14400, the live single-writer) -> redact_payload -> redacted INSERT  [PRODUCTION write path]
  - crates/orchestrator/src/skill_telemetry.rs::log_skill_invocation (:191, the live hook-log writer) -> redact -> redacted append  [PRODUCTION log path]
  - crates/dashboard-v2/src-tauri/src/commands_g9.rs::get_redaction_stats -> RedactionPanel.svelte  [dashboard]
stub_ban:
  - NO cloud egress in the redactor (LOCAL-only; a redactor that phones home is itself a leak)
  - NO new detector / regex set — REUSE vibegate_secrets.rs::SECRET_PATTERNS; NO Presidio/ggshield dependency
  - NO mid-session UPDATE/DELETE of SSB rows (violates ssb.rs:5) — redaction is WRITE-TIME; TTL purges whole expired sinks
  - NO silent pass-through: a detection error redacts the field (fail-closed), never emits the raw value
  - NO default-off ENABLE flag on the REDACTION itself (only ELAI_REDACT_DISABLE kill-switch; default ON, W-2) — only the
    OPTIONAL advisory secret_redacted row may be env-gated wire-dark
  - NO claim that ssb.db is per-session (it is a single shared file; session_id is a column) — the TTL framing must be honest
  - blake3 only (the redaction tokens); no blake2/sha2
  - do NOT cite the REFUTED PRvL Presidio 0.83/0.88/0.85 + GPT-4o numbers; use qualitative high-recall/low-precision
  - do NOT cite crates/orchestrator/src/rra.rs as present (G-2 forward dep, absent)
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/RedactionPanel.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_g9.rs::get_redaction_stats
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <orchestrate a fixture task whose prompt/tool-output carries an
    AWS key (AKIA...) + an email; then trigger a hook-log write; run with NO env override>"
  expect_ssb: "the committed ssb row's payload carries `[REDACTED:aws_key:<blake3-16>]` not the raw AKIA key; the raw
    secret never appears in ssb.db (a grep of the db for AKIA returns nothing); (optional) a secret_redacted advisory row if enabled"
  expect_artifact: "the per-session hook-executions.log carries the redaction token not the raw secret; an expired
    session hook-log is purged whole by purge_expired; RedactionPanel shows the redaction count; a benign payload (no
    secret) is byte-unchanged (FP guard)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/G-plan/G-9_journal_redaction_ttl.md --run-trigger
rg "pub fn redact|pub fn redact_payload|pub fn purge_expired" .elai_cc/crates/orchestrator/src/redaction.rs
rg "redact_payload" .elai_cc/crates/orchestrator/src/ssb.rs        # wired into execute_write before the INSERT
rg "redact\(" .elai_cc/crates/orchestrator/src/skill_telemetry.rs  # wired into the hook-log append
rg -c "ELAI_REDACT_ENABLE" .elai_cc/crates/orchestrator/src/redaction.rs   # MUST be 0 (kill-switch DISABLE only; redaction default-ON)
rg -ci "presidio|ggshield" .elai_cc/Cargo.toml .elai_cc/crates/orchestrator/Cargo.toml   # MUST be 0 (hand-rolled, no dep)
rg -c "0.83|0.88|0.85" .elai_cc/crates/orchestrator/src/redaction.rs   # MUST be 0 (PRvL numbers REFUTED — qualitative only)
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/redaction.rs      # MUST be 0
test -f .elai_cc/crates/dashboard-v2/src/lib/drawers/RedactionPanel.svelte
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/G-9-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS; the observable_effect redacted the AWS key at the SSB
  write path AND the hook-log sink, and an expired session log was purged. cargo-green != DONE.
- [ ] W-2 default-ON: a secret-carrying payload is redacted with NO env override; ELAI_REDACT_DISABLE=1 disables it; no
  default-off ENABLE flag on the redaction (only the OPTIONAL advisory row may be env-gated).
- [ ] W-3 production entrypoint: redaction fires from execute_write (ssb.rs:14400) AND log_skill_invocation (skill_telemetry.rs:191),
  not a test-only path.
- [ ] W-6 dashboard E2E: RedactionPanel shows the redaction count via dashboard-smoker (NEVER the secret value).
- [ ] WRITE-TIME proof: a contract test asserts the raw secret NEVER lands in ssb.db (grep the db after commit) — the
  append-only invariant (ssb.rs:5) means it cannot be scrubbed later; redaction must be pre-INSERT.
- [ ] Stable token: a contract test asserts two occurrences of the same secret map to the same blake3 token (correlatable
  without exposure); a benign payload is byte-unchanged.
- [ ] GD-4 FP gate (MODERATE grounding): a 10-benign-payload contract test shows 0 false redactions; the over-redaction
  rate is MEASURED + recorded on >=3 real runs before default-on.
- [ ] TTL: purge_expired deletes a whole expired per-session hook-executions.log; the ssb.db TTL is session-scoped/whole-file
  (NOT a mid-session row DELETE — a contract test asserts no UPDATE/DELETE on a live ssb row).
- [ ] (if shipped) secret_redacted advisory round-trips + action_type_lock golden; SCHEMA bump named baseline 205 -> 206,
  TOTAL_VARIANTS 482 -> 483.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p contracts.
- [ ] No new dependency (SECRET_PATTERNS + blake3 in-tree; NO Presidio/ggshield — anti-slopsquatting).

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check --run-trigger is ACCEPTANCE 1; redact_payload (produced) is consumed by execute_write (W-8)
  reached by the live single-writer (W-3).
- W-2: only ELAI_REDACT_DISABLE; redaction default ON; observable_effect runs with no override (the optional advisory row
  is the sole env-gated piece — a deliberate, documented W-2 deviation per G-0 "no new mandatory action_type").
- W-3: execute_write (ssb.rs:14400) + log_skill_invocation (skill_telemetry.rs:191); not test-only.
- W-4: redaction stats are human-surfaceable -> RedactionPanel + get_redaction_stats; cross_lang_diff PASS.
- W-5: observable_effect is a runnable orchestrate command -> a redacted ssb.db row + a redacted log + a purged file; in-process.
- W-6: ACCEPTANCE verifies the redaction count THROUGH RedactionPanel via dashboard-smoker.
- W-7: ssb.rs write path (write_payload:14265, execute_write:14400/:14429, append-only :5, run_migration:14334,
  SSB_SESSIONS_SUBDIR:14741), vibegate_secrets.rs SECRET_PATTERNS:54, vibegate.rs source_blake3_prefix:148,
  skill_telemetry.rs log_skill_invocation:166/:191 grep-confirmed 2026-06-02; Presidio/ggshield + rra.rs confirmed ABSENT
  and NOT cited; the exact CWE-798 seed-rule line is VERIFY-before-cite; the validator re-greps.
- W-8: redact_payload (produced) called in execute_write (Task 2); redact() called in the hook-log writer (Task 3);
  get_redaction_stats consumes the (optional) advisory rows / a redaction counter (Task 7). No produced symbol caller-less.
- W-9: (optional) ONE wire-dark advisory SecretRedacted action_type (ssb.rs:1869 wire-dark precedent); action_type golden
  + cross_lang_diff; bump 205->206 / 482->483 only if shipped.
- W-10: entirely Tier-1 in-process (regex detect + token substitute at the write path) — no network/subprocess/RunPod;
  genuinely fail-closed (detection error -> redact).
- GD-1: COMPOSES-WITH F-1 (reuses SECRET_PATTERNS) + G-2 (G-9 is its precondition); re-implements no detector. GD-4: the
  over-redaction FP rate is bounded before default-on (MODERATE grounding). GD-5: no over-claim — PII detection is
  high-recall/low-precision (qualitative), the refuted PRvL numbers are not cited.

## Tests to add
- crates/contracts/tests/g9_write_time_redaction_contract.rs (~180 LOC): (a) a payload with an AWS key + email -> the
  committed row carries `[REDACTED:...:<blake3-16>]`, NOT the raw value; (b) a grep of the test ssb.db after commit returns
  no AKIA match (write-time proof); (c) two occurrences of the same secret -> identical tokens; (d) a benign payload is
  byte-unchanged (FP); (e) a detection error fails CLOSED (redacts).
- crates/contracts/tests/g9_ttl_purge_contract.rs (~120 LOC): (a) purge_expired deletes a whole expired hook-executions.log;
  (b) NO mid-session UPDATE/DELETE on a live ssb row (the append-only invariant holds); (c) the ssb.db TTL is session-scoped/whole-file.
- crates/contracts/tests/fixtures/g9/ (secret-carrying + benign payloads, expired/fresh session sinks).

## Files touched
- CREATE: crates/orchestrator/src/redaction.rs (~220 LOC: redact + redact_payload + purge_expired)
- CREATE: crates/dashboard-v2/src-tauri/src/commands_g9.rs (~90 LOC: get_redaction_stats)
- CREATE: crates/dashboard-v2/src/lib/drawers/RedactionPanel.svelte (~140 LOC)
- CREATE: crates/contracts/tests/g9_write_time_redaction_contract.rs (~180 LOC)
- CREATE: crates/contracts/tests/g9_ttl_purge_contract.rs (~120 LOC)
- CREATE: crates/contracts/tests/fixtures/g9/*.json (fixtures)
- CREATE: workflow/reasoning/G-9-decisions.md (~110 LOC)
- MODIFY: crates/orchestrator/src/lib.rs (~2 LOC: pub mod redaction;)
- MODIFY: crates/orchestrator/src/ssb.rs (~40 LOC: redact_payload call at the top of execute_write before the INSERT; + ~50 LOC optional SecretRedacted advisory variant)
- MODIFY: crates/orchestrator/src/skill_telemetry.rs (~25 LOC: redact() before the hook-log append)
- MODIFY: crates/orchestrator/tests/action_type_lock.rs + tests/action_type_lock.golden (~2 LOC + 1 row, ONLY if the optional advisory row ships)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + DrawerHost.svelte (~6 LOC: DrawerKey 'redaction')

## Citations / Grounding
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R9. OWASP AI Agent Cheat Sheet "audit memory before
  persistence" + "redact log data" CONFIRMED; GitGuardian 24,008 MCP-config secrets + 3.2% Claude-Code leak rate CONFIRMED
  (WebSearch-verify at write time per the paper-citation-verify memory); ggshield 3-moment block/notify CONFIRMED (prior-art
  for write-time fail-closed, NOT a dependency). CORRECTIONS (binding): PRvL (arXiv:2508.05545) Presidio 0.83/0.88/0.85 +
  GPT-4o numbers are REFUTED — use the qualitative "regex/pattern secret+PII detection is high-recall, low-precision"
  framing, re-source from microsoft.github.io/presidio docs; EU penalty EUR 15M/3% (Art-99) NOT 35M/7% (Art-5); NO
  Presidio/ggshield dependency (hand-rolled local redactor).
- ELAI ground truth (grep+source-confirmed 2026-06-02): write_payload ssb.rs:14265 -> write:14249 -> write_loop:14354 ->
  execute_write:14400 (INSERT binds payload :14429, BEGIN IMMEDIATE :14417, COMMIT :14449); append-only ssb.rs:5;
  run_migration:14334 (DROP :14342); SSB_DB_FILE_NAME:14745 / SSB_SESSIONS_SUBDIR:14741 / default_ssb_path_for_base:14758
  (single shared ssb.db, session_id a column); skill_telemetry.rs log_skill_invocation:166 (hook-executions.log :172-176,
  append :190, wire-dark :89) + get_hook_executions commands.rs:4367; vibegate_secrets.rs SECRET_PATTERNS:54 (PAT_AWS_KEY:27,
  PAT_PEM_TYPED:30, PAT_PEM_GENERIC:33, PAT_SECRET_ASSIGN:42, SECRET_LABELS:62); cwe_classifier.rs:58 (CWE-95 :70; CWE-798/321/532
  via vibegate cwe_from_rule_id — exact seed-rule line VERIFY-before-cite); vibegate.rs source_blake3_prefix:148; blake3
  Cargo.toml:65. Presidio/ggshield/gitguardian + rra.rs ABSENT. SCHEMA_VERSION 205 (ssb.rs:2898), TOTAL_VARIANTS 482
  (tests/action_type_lock.rs:534). Wire-dark advisory precedent ssb.rs:1869/:1882/:1890.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, anti-slopsquatting (NO Presidio/ggshield), SCHEMA baseline+N.

## Open questions / risks
- MODERATE grounding (the load-bearing caveat): PII/secret regex detection is high-recall/low-precision. The TOP failure
  mode (GD-4) is OVER-redaction corrupting a payload (a real-but-not-secret string tokenized). MITIGATION: the 10-benign FP
  guard + the measured over-redaction rate on >=3 runs before default-on; anchored credential-shape patterns (the SECRET_PATTERNS
  are already low-FP by design, vibegate_secrets.rs:5-6). Do NOT cite the refuted PRvL precision numbers.
- The single-shared-ssb.db reality: ssb.db is ONE file per project base (session_id a column), NOT per-session — so the
  TTL purge for SSB is a session-scoped row purge at a maintenance boundary (NOT mid-session UPDATE/DELETE, which violates
  ssb.rs:5) or a whole-file TTL across all sessions. State which it does; do NOT pretend ssb.db is per-session. The
  hook-executions.log IS per-session and IS file-granular-purgeable.
- The CWE-798/321/532 reuse surface: SECRET_PATTERNS covers AWS-key/PEM/secret-assignment shapes (the de-facto CWE-798
  detector) but the exact CWE-798 seed-rule line was NOT opened — cite SECRET_PATTERNS (vibegate_secrets.rs:54) +
  cwe_from_rule_id as the reuse surface and VERIFY the exact CWE-798 seed location before citing a line (W-7).
- redact_payload must walk EVERY text-bearing field of EVERY payload variant — a missed field is a leak hole. The contract
  test must assert redaction across the payload variants that carry raw prompt/tool I/O.

## Next
G-9 is the precondition for G-2 (the RRA must not export unredacted secrets) and G-8 (the attestation/sidecar must not leak).
With G-9 in place, the reproducible artifact + signed attestation are shippable to a client without being a secrets-exposure
liability — closing the last gap between "auditable" and "safe to hand over." G-9 reuses F-1's detector, so a secret blocked
at write-time by F-1's gate and a secret redacted at the journal by G-9 share one detection set (one source of truth).

End of G-9.
