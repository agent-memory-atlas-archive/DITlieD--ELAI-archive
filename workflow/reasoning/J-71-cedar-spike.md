# J-71 Cedar Policy-as-Code SPIKE Report

**Status:** SPIKE ONLY — time-boxed measurement, NOT production wire.
**Date:** 2026-08-05
**Gate:** C-42 `check_authz` at `.elai_cc/crates/plugins/src/authz/checker.rs`

---

## 1. Gate Chosen + Why

**C-42 `check_authz(role, tool, doc_context, policy)`** — a pure-function
authorization check with deterministic verdicts and no side effects. It
implements a 4-step decision order (role-not-declared → role-not-permitted-tool
→ role-not-permitted-owner → Allow) with BOLA defense (ignores
`agent_claimed_owner`, consults only `db_owner`).

Selected because it is the smallest self-contained RBAC gate in the codebase,
has existing unit tests with a sample policy TOML, and maps naturally to
Cedar's principal/action/resource model.

---

## 2. Authoring Cost

| Metric | Value |
|--------|-------|
| Wall-clock authoring time | ~25 min (schema + 3 policies + entities + 8 fixtures + script) |
| Cedar artifact LOC | 161 (schema 21 + policy_v1 48 + v2_refactor 45 + v2_broken 47) |
| Fixture LOC | 112 (8 request files × 14 lines) |
| Entities LOC | 8 (structured JSON) |
| Measurement script LOC | 229 (`run_spike.py`) |
| **Total scratch LOC** | **510** |

---

## 3. SMT / Analysis Runtime

**Full Thm 4.2 SMT equivalence API not exposed in cedarpy 4.8.x.** The
`cedarpy` package exposes: `is_authorized`, `is_authorized_batch`,
`is_authorized_partial`, `validate_policies`, `PolicySet`, `Schema`,
`Entities`. No `policy_equivalence`, `check_equivalence`, or SMT-level
analysis function is available. The Cedar Rust crate's
`PolicyAnalysisEngine` / equivalence checking (Thm 4.2, CVC5-based) is not
bound to Python.

**Measured substitute (authorize + validate):**

| Metric | Value |
|--------|-------|
| Schema parse | 0.000134 s |
| Entities parse | 0.000080 s |
| Policy v1 parse | 0.000604 s |
| Policy v1 validate | 0.000907 s |
| Policy v2_refactor parse | 0.000093 s |
| Policy v2_refactor validate | 0.000253 s |
| Policy v2_broken parse | 0.000081 s |
| Policy v2_broken validate | 0.000259 s |
| Avg authorize per call (100-call avg) | 0.000015 s (15 µs) |
| Total parse (3 policy sets) | 0.000778 s |
| Total validate (3 policy sets) | 0.001419 s |

All policies validated successfully against the schema (`validation_passed: true`).

The paper's reported SMT encode+solve time is 75.1 ms (CVC5, Section 5.4).
Our measured per-call authorize is 15 µs — but this is **not** the same
operation. Authorize evaluates one request; SMT equivalence checks all
possible requests. The equivalence API gap is the key finding.

**Cedar CLI attempt:** Not attempted — `cargo install cedar-policy-cli`
would require network access and a local Cargo home in scratch. The CLI
exposes `authorize` and `validate` but the equivalence analysis is also
not in the CLI's standard subcommands. The Rust crate
`cedar-policy-analysis` (used by the paper) is a separate crate not
bound to cedarpy.

---

## 4. Expressiveness Fit

### Mapped cleanly:
- **Role → principal:** `AgentRole::"planner"`, `AgentRole::"executor"` — direct.
- **Tool → action:** `Action::"knowledge.search"`, etc. — direct (Cedar requires
  action type = `Action`).
- **Tool permission list → permit statements:** one `permit` per (role, tool) pair.
- **Default deny:** Cedar's implicit default is deny (no `forbid` needed); matches
  Rust's `default = "deny"`.
- **Ownership "self" sentinel:** encoded as `when { resource has owner && resource.owner == "executor" }`
  — the "self" sentinel maps to checking `db_owner == role_string`.

### Did NOT map cleanly:
- **Unknown role → `role-not-declared` deny reason:** Cedar has no concept of
  "undeclared role." A request with `AgentRole::"ghost"` simply matches no permit
  and gets `Deny` (or `NoDecision` if no policy matches at all). The specific deny
  *reason* (`role-not-declared` vs `role-not-permitted-tool`) is lost — Cedar only
  reports which policies matched or didn't, not a structured reason string.
- **Structured deny reasons:** Rust `AuthzVerdict::Deny` carries a reason string
  (`role-not-permitted-owner: role 'executor' not granted access to docs owned by 'alice'`).
  Cedar's diagnostics report matched policy IDs, not semantic reason codes. Mapping
  back to the Rust reason taxonomy would require post-processing Cedar's diagnostics.
- **BOLA ignore of `agent_claimed_owner`:** Cedar has no concept of "ignored fields."
  The BOLA defense works by *not including* `agent_claimed_owner` in the Cedar
  request context at all — which is a wiring concern, not a policy-language feature.
  The policy itself cannot express "ignore this field."
- **`doc_owners = []` (empty = no constraint):** For `planner` with `doc_owners = []`,
  the Rust check passes ownership for any doc. In Cedar, this maps to permits with
  no `when` guard on ownership — which works, but the semantics are inverted: Rust
  says "no owner constraint needed," Cedar says "permit regardless of owner." Same
  behavior, different mental model.
- **`doc_owners = ["self"]` with missing DocContext:** Rust denies (no context → no
  db_owner → owner check fails). Cedar denies because `resource has owner` is false
  on a Doc with no `owner` attr. Same result, but Cedar's is "attribute missing"
  while Rust's is "context missing."

---

## 5. Second-Source-of-Truth Cost

If Cedar policies were adopted alongside the Python/Rust checker, the system
would have **two gate homes** that can drift:

| Risk Factor | Assessment |
|-------------|------------|
| **Dual-edit surface** | Every policy change (TOML in Rust + `.cedar` files) must be manually synchronized. No auto-sync mechanism exists. |
| **Drift detection** | Would require a fixture parity suite (the 8 fixtures here) run on every change. At 8 fixtures, parity is cheap; at 50+ roles × 10+ tools × owner variants, the fixture matrix grows combinatorially. |
| **No auto-translation** | TOML policy → Cedar policy is a manual translation. The mapping rules (role→principal, tool→action, self→owner check) are codified only in this spike, not in tooling. |
| **Deny reason divergence** | Cedar cannot produce Rust's structured deny reasons. Any caller relying on `deny_reason()` would need a Cedar-to-Rust-reason mapping layer. |
| **Schema maintenance** | Cedar schema (entity types, actions) must track the Rust `AuthzPolicy` struct. New roles/tools require edits in both places. |

**Quantified risk:** For the current 2-role, 4-tool policy, the dual surface is
~50 LOC in each home. The fixture parity suite needs ≥8 cases (current) to
cover the decision matrix. Adding a role or tool roughly doubles the fixture
count (new role × existing tools + new tool × existing roles). At 5 roles ×
10 tools, the fixture matrix is ~50+ cases, and each policy edit requires
updating 2 files + verifying 50+ fixtures.

---

## 6. Fixture Parity Table

| Request Fixture | Rust Expected | Cedar v1 | Cedar v2_refactor | Cedar v2_broken |
|----------------|:-------------:|:--------:|:-----------------:|:---------------:|
| `allow_planner_search` | Allow | Allow ✓ | Allow ✓ | Allow ✓ |
| `deny_planner_write` | Deny | Deny ✓ | Deny ✓ | **Allow ✗** |
| `deny_unknown_role` | Deny | Deny ✓ | Deny ✓ | Deny ✓ |
| `allow_executor_write_self` | Allow | Allow ✓ | Allow ✓ | Allow ✓ |
| `deny_executor_write_alice` | Deny | Deny ✓ | Deny ✓ | Deny ✓ |
| `deny_executor_write_no_ctx` | Deny | Deny ✓ | Deny ✓ | Deny ✓ |
| `allow_executor_edit_self` | Allow | Allow ✓ | Allow ✓ | Allow ✓ |
| `deny_executor_edit_alice` | Deny | Deny ✓ | Deny ✓ | **Allow ✗** |

**Summary:**
- v1 matches Rust on all 8/8 fixtures ✓
- v2_refactor matches v1 on all 8/8 fixtures (equivalent) ✓
- v2_broken mismatches v1 on 2/8 fixtures (planner fs.write granted, executor fs.edit owner guard removed) ✓

---

## 7. Recommendation: **keep-Python**

### Rationale (numbers-backed):

1. **SMT equivalence API gap:** The primary theoretical advantage of Cedar
   (Thm 4.2 SMT policy-equivalence checking) is **not accessible** via
   `cedarpy 4.8.7`. The Python binding exposes only `is_authorized` and
   `validate_policies`. Without the equivalence API, Cedar offers no
   analysis advantage over fixture-based testing of the existing Rust
   checker. The paper's 75.1 ms SMT equivalence check is measured on the
   Rust crate, not the Python binding.

2. **Authoring cost vs. benefit:** 510 LOC of Cedar artifacts + 229 LOC
   measurement script to replicate a 55-line Rust function. The Cedar
   policies are more declarative but the dual-maintenance cost (§5)
   exceeds the readability gain for a 2-role, 4-tool policy.

3. **Deny reason loss:** Cedar cannot produce the structured deny reasons
   (`role-not-declared`, `role-not-permitted-tool`, `role-not-permitted-owner`)
   that the Rust checker returns. Any caller switch would lose diagnostic
   granularity.

4. **BOLA is a wiring concern, not a policy concern:** Cedar's policy
   language cannot express "ignore `agent_claimed_owner`." The BOLA defense
   must be enforced at the wiring layer (don't pass the field to Cedar),
   which is exactly what the Rust checker already does in code.

5. **Authorize performance is not a bottleneck:** 15 µs/call (Cedar) vs.
   sub-microsecond (Rust pure function). Cedar adds a Python↔Rust FFI
   boundary for no performance gain.

### When to revisit:
- If the policy grows to 10+ roles with complex ownership rules and
  refactoring safety becomes critical, the Cedar Rust crate's equivalence
  API (not cedarpy) would be worth evaluating via a native Rust integration,
  not a Python binding.
- If `cedarpy` exposes the equivalence API in a future version, re-spike.

---

## 8. `test_cedar_contract.py` Premature Stub — OPEN Owner Decision

No `test_cedar_contract.py` was created. The spike's `run_spike.py` is a
throwaway measurement script in scratch, not a committed test. 

**OPEN decision for orchestrator:** If a future step creates
`test_cedar_contract.py` as a scaffold (e.g., to assert fixture parity
between Rust and Cedar), it should be **kept-as-scaffold only** with a
clear marker that it is not a production gate, OR **removed** entirely
if the keep-Python recommendation is accepted. The spike does NOT adopt
it as committed.

**Lean:** remove — a scaffold test for a declined policy engine adds
maintenance burden without value. If Cedar is revisited later, the test
can be re-created from this spike's fixtures.

---

## 9. Explicit: NO Production Wiring Performed

- **NO** edits under `.elai_cc/crates/` or any production path.
- **NO** `cedar-policy` or `cedarpy` added to any workspace `Cargo.toml`
  or `pyproject.toml`.
- **NO** production seam invented or wired.
- **NO** git commits, pushes, or branch mutations.
- All artifacts are in `conductor/opus_walk/scratch/j71-cedar/` (throwaway)
  and this report file.

---

## Appendix: Scratch Artifacts

Location: `conductor/opus_walk/scratch/j71-cedar/`

| File | Purpose | LOC |
|------|---------|-----|
| `schema.cedarschema` | Cedar JSON schema (AgentRole, Doc, 4 actions) | 21 |
| `policy_v1.cedar` | v1 policies matching check_authz semantics | 48 |
| `policy_v2_refactor.cedar` | Permission-equivalent refactor (reordered) | 45 |
| `policy_v2_broken.cedar` | Deliberate non-equivalent (planner fs.write + BOLA) | 47 |
| `entities.json` | 6 entities (3 roles, 3 docs) | 8 |
| `request_*.json` (×8) | Request fixtures covering allow/deny cases | 112 |
| `run_spike.py` | Measurement script (cedarpy 4.8.7 in isolated venv) | 229 |
| `results.json` | Measured timings + fixture matrix results | 163 |
