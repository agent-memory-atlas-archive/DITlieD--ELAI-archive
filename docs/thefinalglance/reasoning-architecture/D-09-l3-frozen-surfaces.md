# L3 Frozen Surfaces + Locked-Keys Access Control (D-9)

- **ELAI ids:** D-9 / companion §2.3 + §4 / current-RF (Everitt) / Boxed Myopic (Cohen) / mesa-opt (Hubinger)
- **Spec status (as designed):** PLANNED. Feature-map tags DARK / WIRED_STATIC 06-07b (CI added; registry 0 entries at audit) (`ELAI-FEATURE-MAP.md:1690-1692`).
- **Crate / module (spec):** `crates/orchestrator/src/locked_keys_auditor.rs`; registry `.elai_cc/scripts/locked_keys_registry.toml`; hook `scripts/locked_keys_check.sh`; CI `locked-keys-audit.yml`
- **Purpose of this file:** enough architecture to rebuild the three-layer lock on the function that grades the agent. Not a status ledger.

## 1. What it is

**L3 frozen-surface access control** is the structural ban on the agent editing the thing that scores it. Companion §2.3 / §4 name fourteen surfaces (harness, tripwires, BAI, archive, held-out, canary families, blake3 choice, `AllowedConfigKey` bound, `verifier_required`, `Domain`, and the four toml blocks). Enforcement is documentation + pre-commit hook + runtime `LockedKeysAuditor`. A human may commit with `L3_OVERRIDE:<key>:` and ≥40 characters of justification. Git push / pre-commit **BLOCK** otherwise (`D-9_l3_frozen_surfaces.md:3-8`, `:19-46`, `D-SUMMARY_INDEX.md:70-75`).

## 2. Why it exists

Everitt current-RF, Cohen Boxed Myopic, and Hubinger mesa-optimization agree: the agent must not modify the function that grades it. If this organ is missing, D-1..D-8 exist as files the prompt-engineer agent can patch (`kl_spec_tau = 10`, `VerifierKind::Refuse`, extra `Domain` variant). If it is faked (a comment “do not edit tripwires”), the model evaluates its own modification. **Promotion still needs ≥2 verifiers (D-3); this organ stops the agent from deleting that rule** (`D-9_l3_frozen_surfaces.md:37-40`, `:190-192`).

## 3. How it works (runtime)

1. **Who enforces.** (a) local `.git/hooks/pre-commit` → `locked_keys_check.sh`. (b) CI `locked-keys-audit.yml` on push/PR. (c) orchestrator PreToolUse / pre-`Tool::Write` → `LockedKeysAuditor::audit_runtime_write`. Coordinate hook **order** with C-27 StubBanAuditor (`D-9_l3_frozen_surfaces.md:12-15`, `:108-134`, `:219-222`).
2. **Registry load (deterministic).** Parse `locked_keys_registry.toml` into `LockedKeysRegistry { paths, symbols, toml_keys }`. **Fail closed** if unreadable: treat as locked (`:77-106`, `:224-225`).
3. **Pre-commit path layer.** `git diff --cached --name-only` ∩ `[[locked_path]]`. Hit and no valid override → exit 1 (`:110-113`).
4. **Symbol layer.** Grep cached diff body for `[[locked_symbol]]` names (e.g. `AllowedConfigKey`) (`:114`).
5. **Toml-key layer.** Parse toml diff for `section.key` in `[[locked_toml_key]]` (`:115-116`).
6. **Override parse.** Commit body must contain `L3_OVERRIDE:<key>:` matching a registry `human_override_key` **and** ≥40-char justification. Then hook **allows**. `L3OverrideRequested` SSB is a **log of the human decision, not a permission grant in code** (`:140-159`, `:193-196`).
7. **Runtime write (deterministic).** `audit_runtime_write(PendingWrite)` on orchestrator disk/config edits. Event → **BLOCK the write** + `LockedKeysTouch` with `override_present=false` (`:125-134`, `:294-296`).
8. **Diff auditor.** `audit_diff(&GitDiff) -> Vec<LockedKeysTouchEvent>` for tests and CI (`:122-124`).
9. **Journal.** `LockedKeysTouch` on every detected touch (override or not). `L3OverrideRequested` when the tag is present (`:136-143`).
10. **Fail-closed.** Short justification BLOCK. Registry parse error BLOCK. CI catches hook bypass (`:183-187`, `:226-229`).

The model may propose a Candidate template (D-1) inside L1A. It may **not** edit tripwire τ, BAI δ, n-gram n, canary k, blake3→sha256, `Domain` variants, or `verifier_required`.

## 4. Types, enums, objects

Copy from `D-9_l3_frozen_surfaces.md:77-143`. Do not invent fields.

```
struct LockedKeysAuditor {
  registry: LockedKeysRegistry,
  ssb_tx: SsbSender,
  audit_id: Uuid,
}

struct LockedKeysRegistry {
  paths: Vec<LockedPath>,
  symbols: Vec<LockedSymbol>,
  toml_keys: Vec<LockedTomlKey>,
}

// registry TOML shapes:
[[locked_path]]
  path, reason, human_override_key   # e.g. "L3_OVERRIDE:tripwire"

[[locked_symbol]]
  symbol, file, reason               # e.g. AllowedConfigKey

[[locked_toml_key]]
  file, section, keys = [...], reason

fn audit_diff(diff: &GitDiff) -> Vec<LockedKeysTouchEvent>
fn audit_runtime_write(write: &PendingWrite) -> Option<LockedKeysTouchEvent>
```

**The 14 L3 surfaces (companion §2.3 + §4; registry must cover all)** (`:19-35`, `:105-106`):

| # | Surface | Typical lock |
|---|---|---|
| 1 | Staged-eval harness code | path |
| 2 | Tripwire thresholds **and** detection code (D-4) | path `tripwire.rs` + toml `[tripwire]` |
| 3 | MSI / BAI scoring function code (D-5) | path `bai.rs` |
| 4 | Archive maintenance + parent-selection (D-1 + D-8) | path `archive_chain.rs` / verify |
| 5 | Held-out task **selection algorithm** (D-6) | path `held_out.rs` |
| 6 | Cross-domain canary **family definitions** (D-7) | `CanaryFamily` / Domain |
| 7 | Cryptographic hash function selection (**blake3 only**) | symbol / hash call sites |
| 8 | Bound on `AllowedConfigKey` itself (cannot self-add variants) | `[[locked_symbol]] AllowedConfigKey` |
| 9 | `verifier_required` on every `ReasoningTemplate` (cannot self-weaken Prm→Refuse) | field/symbol |
| 10 | `Domain` enum (cannot self-add domains) | symbol |
| 11 | `[tripwire]` keys: `kl_spec_tau`, `rho_delta_threshold`, `iqr_outlier_multiplier`, `ppl_anomaly_sigma`, `ddm_warning_threshold`, `ddm_drift_threshold`, `consensus_fire_count`, `window_size` | toml |
| 12 | `[bai]` `eval_budget`, `ci_alpha`, per-family delta | toml |
| 13 | `[held_out]` `ttl_months`, `ngram_size`, `jaccard_threshold` | toml |
| 14 | `[canary]` `regression_threshold`, `min_samples` | toml |

**SSB SCHEMA 54→55** (`:136-143`):

```
LockedKeysTouch { path, symbol: Option<String>, toml_section: Option<String>,
                  toml_key: Option<String>, override_present: bool, audit_id }
L3OverrideRequested { override_key, requester: String, justification: BoundedString<512> }
```

Override example body (`:149-157`): subject line + `L3_OVERRIDE:tripwire.kl_spec_tau:` + ≥40-char rationale (PRMBench false-positive story in the spec).

## 5. Control flow / state machine

**Owner of the loop:** hooks + auditor. Not BAI, not the model.

```
Write or commit
  → load registry (unreadable ⇒ locked)
  → match path ∪ symbol ∪ toml_key
  → if miss: allow
  → if hit and valid L3_OVERRIDE (≥40 chars, known key):
        allow; log L3OverrideRequested; optionally LockedKeysTouch override_present=true
  → if hit else:
        BLOCK; LockedKeysTouch override_present=false; hook exit 1; no disk write
```

**Forced:**

- Three layers: docs, pre-commit, runtime (`:42-46`). CI is the fourth guardrail against `--no-verify` (`:187`).
- Override is **human**; code must not treat an agent-written tag as sufficient without the hook seeing a real commit message (`:140-143`).
- C-21 hook already exists; D-9 **extends** paths, does not replace (`:48-50`, `:108-110`).

**UNCERTAIN:** unit test allows “0 events **OR** 1 event with `override_present=true`” on tagged diffs (`:171-172`).

## 6. Grounding, receipts, gates

Receipts: registry row + `LockedKeysTouch` + hook exit code + CI status. Papers are the why, not a confidence score.

**Never allowed as a confidence signal:** agent “I needed to tune τ”; documentation-only L3; empty registry; `L3_OVERRIDE:yes`; self-weakening `verifier_required`; adding a `Domain` variant from inside a Candidate template.

**Selector hierarchy:** path, then symbol, then toml key — **all three must be implemented** (`:227`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| surfaces | ≥14 registry entries | `[[locked_*]]` count (`:246-249`) |
| justification | ≥40 chars | L3_OVERRIDE body (`:159`, `:193-196`) |
| `justification` type | `BoundedString<512>` | SSB (`:141`) |
| SCHEMA | 54 → 55 | SSB |
| unit tests | ≥8 | orchestrator (`:288`) |
| CI trigger | push + PR to main | (`:183-184`) |

**Overflow:** justification >512 → bounded-string error (D-1 pattern); spec does not say truncate. Fail closed.

## 8. Roles and neighbors

Human operator owns overrides. Orchestrator owns runtime block. **No agent role** may edit the registry.

**Consumes:** C-21 `locked_keys_check.sh` + pre-commit chain; C-27 StubBanAuditor pattern; D-1..D-8 surfaces; `GitDiff` / git2 or `git` shell-out (`:212-218`).

**Produces:** auditor, registry, SCHEMA 55, CI workflow, docs.

**Must not fork:** a second “soft L3” in prompts; agent-writable registry; signing deferred (`:323`).

**Not parallel-safe with C-27** (hook order) (`:12-15`). Parallel-safe with Wave 2 D-10..D-18 (they consume, do not write, these surfaces) (`:12-13`).

## 9. Config / flags / defaults

Authoritative file: `locked_keys_registry.toml` (`:163`). No “L3 off” env in D-9.

Feature-map `WIRED_STATIC` / empty registry at audit is neighbor — rebuild **≥14 populated entries**, not an empty file (`ELAI-FEATURE-MAP.md:1690-1692`).

Crypto **signing** of the registry is deferred; relies on git commit signing (`:323`). Multi-party approval deferred (`:324-325`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `scripts/locked_keys_registry.toml` | canonical 14 (`:77-106`) |
| `scripts/locked_keys_check.sh` | C-21 extended (`:108-116`) |
| `locked_keys_auditor.rs` | runtime (`:118-127`) |
| `task_runner.rs` PreToolUse | (`:129-134`) |
| `ssb.rs` SCHEMA 55 | (`:136-144`) |
| CI | `.github/workflows/locked-keys-audit.yml` (`:183-187`) |
| hooks | `.git/hooks/pre-commit` or `install-hooks.sh` (`:258-260`) |
| docs | CLAUDE.md or `docs/L3-FROZEN-SURFACES.md` (`:161-166`) |
| papers | Everitt 1908.04734, Hubinger 1906.01820, Cohen AI Mag 43(3) (`:66-73`) |

## 11. Journal / SSB events

Two variants (section 4). Observable (`:230-235`): commit touching `tripwire.rs` without tag → 1 `LockedKeysTouch`, hook exit 1, commit blocked; runtime tool write same. Valid tag → `L3OverrideRequested`.

Cloud C: simulated `Tool::Write` to `bai.rs` → BLOCK, `override_present=false` (`:294-296`).

## 12. Tests and observables

Named unit tests (`:168-175`):

| Id | Assert |
|---|---|
| T-path | diff `tripwire.rs`, no override → 1 touch event |
| T-override | same + `L3_OVERRIDE:tripwire` → 0 events **or** 1 with `override_present=true` |
| T-toml | `elai_cc.toml` `[tripwire] kl_spec_tau` → event with section+key |
| T-other | unrelated path → 0 events |

Integration (`:177-181`): tmpdir git; staged locked path → `locked_keys_check.sh` exit 1; with valid tag → exit 0.

Cloud A/B/C as section 11 (`:290-296`).

Done-gates: ≥14 registry items covering the 14 surfaces; ≥8 tests; docs; papers (`:276-298`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- A parsed registry of **paths + symbols + toml keys** covering all 14 surfaces.
- Pre-commit **and** CI **and** runtime pre-write all fail closed.
- Unreadable registry = locked, not open.
- `L3_OVERRIDE` needs a known key and ≥40-char human justification; SSB log is not the grant.
- Agent cannot add `AllowedConfigKey` / `Domain` variants or weaken `verifier_required`.
- blake3 remains the only hash selection.
- Model that is being graded cannot edit tripwire/BAI/held-out/canary knobs.

A markdown “L3 list”, a prompt, or an empty `locked_keys_registry.toml` is a cousin, not this organ.

## 14. Non-goals / banned designs

- D-25 combined tamper+lock bench (`:321-322`).
- Registry cryptographic signing (`:323`).
- Multi-party override (`:324-325`).
- Documentation-only L3 (decision a) (`:190-192`).
- Trivial override strings (decision b) (`:193-196`).
- Pre-commit without runtime auditor (decision c) (`:197-199`).
- Silent allow on registry parse error (`:224-225`).
- Covering only paths, skipping symbols or toml keys (`:227`).
- Model self-certifying an L3 edit.

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md` (full file). Index: `D-SUMMARY_INDEX.md:70-75`, `:277`. Feature map: `ELAI-FEATURE-MAP.md:1690-1692`. Companion (not in this tree): §2.3 lines 182-194, §4 lines 347-362 as cited `:56-57`. Papers: 1908.04734, 1906.01820, Cohen/Hutter/Osborne AI Mag 43(3) DOI:10.1002/aaai.12064. C-21/C-27 neighbors: `:48-52`. D-1 `verifier_required` / `Domain`: `D-1_reasoning_template_schema.md:96-101`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Full per-file path list beyond the two examples (`tripwire.rs`, `bai.rs`) — “populate from companion §4” (`:81-106`) and companion file is not in this tree.
- Held-out toml keys named in item 13 (`ttl_months`, `ngram_size`, `jaccard_threshold`) vs D-6 also locking `uniform_pass_threshold`, `min_evaluations_for_retire`, `regression_threshold_pct` (`:34` vs `D-6:140-144`).
- Canary item 14 `min_samples` vs toml `min_samples_per_family` (`:35` vs `D-7:114`).
- Override test 0-vs-1 event (`:171-172`).
- `GitDiff` / `PendingWrite` / `LockedKeysTouchEvent` field lists (`:122-127`).
- `requester: String` source (git author? role?) (`:140`).
- Hook chain **order** vs C-27 (`:12-15`).
- Whether D-7’s Domain +Planning/+KnowledgeQa required an L3_OVERRIDE at the time (human session vs agent).
- Feature-map empty registry vs ≥14 required (`ELAI-FEATURE-MAP.md:1691` vs `:280`).
- MSI scoring lock (item 3) vs D-5 demoting MSI to display-only (`:23` vs `D-5:168-170`).
- git2 vs shell-out (`:216-217`).
