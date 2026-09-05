# Cryptographic Archive Verify CLI (D-8)

- **ELAI ids:** D-8 / Phase 4H.IX / companion §3.8
- **Spec status (as designed):** PLANNED. Feature-map tags PARTIAL (`elai archive verify`; CI workflow absent) (`ELAI-FEATURE-MAP.md:1688-1689`).
- **Crate / module (spec):** `crates/cli/src/archive_verify.rs`; dispatch in `cli/src/commands.rs`; CI `.github/workflows/archive-verify.yml`
- **Purpose of this file:** enough architecture to rebuild the user-facing hash-chain verifier, tamper contracts, and CI gate. Not a status ledger.

## 1. What it is

**Cryptographic archive verify** is the user-facing and CI wrapping of D-1 `ArchiveChain::verify_chain`. Command: `elai archive verify [--path PATH] [--json] [--fail-fast]`. Default path `.elai_cc/archive/reasoning-templates.jsonl`. Exit **0** on a clean chain, **1** on any integrity error. Always emits `ArchiveVerifyResult`. This is the checksum defense from arxiv:2507.05619 plus DGM §5 lineage (`D-8_crypto_archive_verify.md:3-8`, `D-SUMMARY_INDEX.md:64-68`).

## 2. Why it exists

A self-improver that can rewrite its audit history has lobotomized the evidence of the hack (DGM §H hallucination-marker-removal). If this organ is missing, D-1 `verify_chain` exists only as a library call an agent can skip. If it is faked (the model “reviews” the JSONL), the model evaluates its own modification. CI must fail the PR; CLI must fail the shell (`D-8_crypto_archive_verify.md:16-31`, `:146-150`).

## 3. How it works (runtime)

1. **Who calls it.** Operator CLI; GitHub Actions `archive-verify.yml` on push/PR/nightly; dev `cargo run --bin elai -- archive verify`. **Not** the promoting model (`D-8_crypto_archive_verify.md:71-77`, `:142-145`).
2. **Load (deterministic).** `ArchiveVerifyCmd` loads `ArchiveChain` from `archive_path` (`:62-64`).
3. **Walk (deterministic).** For each `ArchiveEntry`:
   - Recompute blake3 of the concatenation named in companion §3.8 (see hash box below).
   - Compare to `self_hash` → else `HashMismatch`.
   - `parent_hash` must name an existing prior entry → else `BrokenParent`.
   - Duplicate `self_hash` → `DuplicateHash`.
   - Unparseable JSONL line → `MalformedJson`.
   (`:25-28`, `:58-61`)
4. **Collect vs fail-fast.** Default: keep walking, aggregate all errors. `--fail-fast`: stop on first (`:65-66`, `:126-128`).
5. **Report.** Human text, or `--json` parseable (`jq` round-trip). `ArchiveVerifyReport { total_entries, verified, errors }` (`:56-57`, `:67`, `:205`).
6. **Exit.** Zero errors → **0**. Any error → **1**. Clean must not “always 1”; tampered must not “always 0” (`:68`, `:147-150`).
7. **Journal.** `ArchiveVerifyResult` **regardless of exit code** (`:69`, `:152-155`).
8. **CI.** windows-latest + ubuntu-latest matrix; `elai archive verify --path … --fail-fast`; skip if commit message contains `[ci skip]` (`:84-92`).
9. **Perf bound.** 10K entries, `verify_chain` **≤ 5 s**, **O(n) not O(n²)** (`:108-113`).
10. **Recovery (docs, not auto-heal).** If verify fails, last N agent actions are suspect; forensic walk the chain; do **not** delete JSONL (`:116-119`).

The model may have appended Candidate rows earlier. It may **not** recompute hashes, skip CI, or mark a tampered file clean.

**Hash concatenation (copy both; do not merge):**

| Source | Bytes hashed |
|---|---|
| D-1 `compute_content_hash` | `body \|\| parent_hash_or_32_zero_bytes` (`D-1_reasoning_template_schema.md:114-116`) |
| D-8 / companion §3.8 walk | `template_body \|\| parent_hash \|\| mutation_proposal_text` (`D-8_crypto_archive_verify.md:16-17`, `:25-27`) |

D-8 also claims it **does not extend schema** (`:11-12`). D-1 `ArchiveEntry` has **no** `mutation_proposal_text` field. Rebuild: verify **whatever D-1 actually stored**; treat the third operand as UNCERTAIN (section 16). Algorithm remains **blake3**, never sha256/blake2 (`:123-126`).

## 4. Types, enums, objects

Copy from `D-8_crypto_archive_verify.md:52-61`. Do not invent fields.

```
struct ArchiveVerifyCmd {
  archive_path: PathBuf,
  json_output: bool,
  fail_fast: bool,
}

struct ArchiveVerifyReport {
  total_entries: u32,
  verified: u32,
  errors: Vec<ArchiveVerifyError>,
}

enum ArchiveVerifyError {
  HashMismatch { entry_index: u32, expected: String, computed: String },
  BrokenParent { entry_index: u32, parent_hash: String },
  DuplicateHash { entry_index: u32, hash: String, prior_index: u32 },
  MalformedJson { entry_index: u32, error: String },
}

fn run(&self) -> Result<ExitCode, ArchiveVerifyError>
```

D-1 types consumed, not redefined: `ArchiveEntry { self_hash, parent_hash, template, staged_eval_result, promoted_at, retired_at, retirement_reason }`, `ArchiveChain::verify_chain` (`:21-24`). D-SUMMARY names `eval_result` (`D-SUMMARY_INDEX.md:65`) — same organ, D-1 field name wins.

**CLI:** `SlashCommand::ArchiveVerify`; flags `--path`, `--json`, `--fail-fast` (`:71-77`).

**SSB SCHEMA 53→54** (`:79-82`):

```
ArchiveVerifyResult { audit_id: Uuid, total_entries: u32, verified: u32, error_count: u32, errors: Vec<String> }
```

## 5. Control flow / state machine

**Owner of the loop:** CLI process / CI job. Orchestrator `verify_chain` is the library. No agent FSM.

```
load JSONL
for each line i:
  parse → MalformedJson
  recompute blake3 → HashMismatch
  parent exists → BrokenParent
  self_hash unique → DuplicateHash
  if fail_fast and error: stop
emit ArchiveVerifyResult
exit 0 iff errors.is_empty()
```

**Forced:**

- Append-only store still has **no delete** (D-1). Verify never “repairs” by rewrite.
- Default path = D-1 default (`:76-77`).
- SSB even when exit 1 (`:69`).

## 6. Grounding, receipts, gates

The receipt **is** the recomputed blake3 vs stored `self_hash`, plus parent-link existence, plus uniqueness.

**Never allowed as a confidence signal:** LLM reading the archive and saying it looks fine; comparing cached hashes without recompute (D-1 stub-ban); sha256; exit-code ignoring.

**CI is a gate** on push/PR. Cloud B: `sed` one `template_body` → exit 1 `HashMismatch` (`:214-215`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| hash | blake3, 32 bytes | every entry |
| 10K walk | ≤ 5 s wall | perf test, ignored by default |
| complexity | O(n) | anti-DoS (`:112-113`) |
| JSONL scale | ~10K clean (D-1); compaction deferred | (`:235-236`) |
| SCHEMA | 53 → 54 | SSB |
| CI OS | windows-latest + ubuntu-latest | (`:90-91`) |
| tamper contract | 4 tampers + 1 clean | (`:94-106`) |
| D-25 later | 10 scenarios / 5 s | out of scope (`:232`) |

**IO:** no `unwrap`; `thiserror` (`:147`).

## 8. Roles and neighbors

CLI + CI. D-4 auto-revert and D-5/D-6/D-7 promotions **append**; D-8 **only reads** (`:11-12`). Parallel-safe with D-2..D-7.

**Consumes:** D-1 `verify_chain`, `SlashCommand`, `SsbWriter` (`:138-141`). Pattern models: C-27 `elai wiring audit` + `wiring-audit.yml` (`:35-36`).

**Produces:** report, exit code, SCHEMA 54, CI workflow.

**Must not fork:** a second in-agent “verify” that can return 0 on HashMismatch; compaction that rewrites hashes.

## 9. Config / flags / defaults

- Path default: `.elai_cc/archive/reasoning-templates.jsonl`.
- `--json` / `--fail-fast` real, not stubs (`:151`).
- `[ci skip]` skips the workflow (`:92`).
- No new `elai_cc.toml` section in D-8 (path already under `[reasoning.templates]`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `cli/src/archive_verify.rs` | cmd/report/error (`:52-69`) |
| `SlashCommand::ArchiveVerify` | `commands.rs` + `main.rs` (`:71-75`) |
| `ssb.rs` SCHEMA 54 | (`:79-82`) |
| CI | `.elai_cc/.github/workflows/archive-verify.yml` (`:84-92`) |
| contract | `contracts/tests/archive_verify_contract.rs` (`:94-106`) |
| perf | `orchestrator/tests/archive_verify_perf.rs` ignored (`:108-113`) |
| docs | README or docs/ tamper-recovery (`:115-119`) |

## 11. Journal / SSB events

One variant `ArchiveVerifyResult`. One row **per run** (`:152-155`). Clean: `error_count=0`. Tampered: `error_count>=1`. `errors: Vec<String>` is a flattened form of `ArchiveVerifyError` (exact stringification UNCERTAIN).

## 12. Tests and observables

Contract (`:94-106`):

| Id | Assert |
|---|---|
| T-A | mutate entry 3 `template_body` → HashMismatch index 3, exit 1 |
| T-B | parent_hash of 4 → missing → BrokenParent, exit 1 |
| T-C | duplicate self_hash 2 and 5 → DuplicateHash, exit 1 |
| T-D | malformed line → MalformedJson; `!fail_fast` continues |
| T-clean | exit 0, `error_count=0` |

Acceptance also: two tampers + default collect → ≥2 errors in one report (`:206-207`). `--json` `jq` round-trip (`:205`).

Cloud A: after a real promotion, verify exit 0 (`:212-213`). Cloud B: `sed` body → exit 1 HashMismatch (`:214-215`).

Perf: 10K ≤5s, `--ignored` (`:208-209`).

Deterministic-check includes a live append-corrupt-restore cycle (`:182-186`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- User command `elai archive verify` over D-1 JSONL.
- Recompute blake3; check parent exists; check unique `self_hash`; parse errors typed.
- Exit 0 clean / 1 dirty; SSB every run.
- `--json` and `--fail-fast` work.
- CI runs the same binary with `--fail-fast`.
- 4 named tampers detected; O(n) ≤5s at 10K.
- No sha256; no LLM judge; no auto-rewrite of a dirty chain.

A `sha256sum` of the file, a git diff, or “the agent confirmed the archive” is a cousin, not this organ.

## 14. Non-goals / banned designs

- D-25 ten-scenario bench (`:232`).
- D-9 LockedKeysAuditor (`:233-234`).
- Compaction / sled migration (`:235-236`).
- Schema writes (D-8 read-only vs ArchiveChain) (`:11-12`).
- `unwrap` on IO (`:147`).
- Always-exit-1 or always-exit-0 (`:147-150`).
- Stub flags (`:151`).
- sha256/sha3 (decision b) (`:123-126`).
- Model self-certifying archive integrity.

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-8_crypto_archive_verify.md` (full file). Index: `D-SUMMARY_INDEX.md:64-68`, `:276`. Feature map: `ELAI-FEATURE-MAP.md:1688-1689`. Companion (not in this tree): §3.8 lines 319-337 as cited `:40-41`. D-1 hash + entry: `D-1_reasoning_template_schema.md:114-132`. Papers: 2507.05619, DGM §5. C-27 CLI/CI pattern: `C-27_testing_trust_t0_wiring_contract.md` as cited `:44-48`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Third hash operand `mutation_proposal_text` vs D-1 two-operand hash and “no schema extension” (`:11-12`, `:16-17`, `:25-27` vs D-1 `:114-116`, `:125-128`).
- `run() -> Result<ExitCode, ArchiveVerifyError>` vs aggregating `Vec<ArchiveVerifyError>` — which errors become `Err` vs report (`:62-68`).
- `ArchiveVerifyResult.errors: Vec<String>` formatting (`:80-81`).
- Whether `BrokenLink` (prose `:28`) is an alias of `BrokenParent` or a fifth variant (enum has four).
- D-SUMMARY `eval_result` vs D-1 `staged_eval_result` (`D-SUMMARY_INDEX.md:65`).
- Tamper-recovery “last N actions” value of N (`:118-119`).
- How CI obtains a built `elai` binary on windows-latest (`:87-89`).
- Whether `DuplicateHash` / `MalformedJson` are inside D-1 `verify_chain` or only the CLI wrapper (`:25-28` vs `:64`).
- Feature-map “CI workflow absent” vs D-8 requiring `archive-verify.yml` (`ELAI-FEATURE-MAP.md:1689`).
