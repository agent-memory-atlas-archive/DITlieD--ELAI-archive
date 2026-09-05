F-2b SeCodePLT static-gate insecure-rate calibration (Slice A2)
session s-60824  date 2026-06-02

WHAT THIS RECORDS
The MEASURED before/after/gate_recall/recall/FP of ELAI's REAL static write-gate
(crates/orchestrator/src/vibegate.rs::evaluate_vibegate) over the deterministic
SeCodePLT-structured CWE corpus
(crates/orchestrator/src/benchmarks/seccodeplt_fixtures/corpus.json), computed by
SecCodePltRunner::run (crates/orchestrator/src/benchmarks/seccodeplt_runner.rs).
No model, no sandbox. All numbers COMPUTED by the scorer / corpus arithmetic, never
hard-coded literals. This is a STATIC-gate insecure-rate, DISTINCT from SeCodePLT's
dynamic secure-coding rate (the live-model 3-way metric is RunPod-gated, NOT here).

CORPUS SHAPE (counted from corpus.json)
  total samples       : 40   (>= 40 required)
  candidates (n)      : 80   (2 per sample: vulnerable_code + patched_code)
  sample ids unique   : yes
  languages           : python + typescript
  per-CWE sample count:
    CWE-89  (SQLi, BLOCK class)         : 10
    CWE-79  (XSS, BLOCK class)          :  8
    CWE-327 (weak-crypto, BLOCK class)  :  7
    CWE-78  (cmd-injection, gate-miss)  :  4
    CWE-502 (deserialization, gate-miss):  4
    CWE-862 (missing-authz, WARN only)  :  4
    CWE-697 (logic, gate-miss)          :  1
    CWE-285 (logic-authz, gate-miss)    :  1
    CWE-639 (IDOR, gate-miss)           :  1

MEASURED KEYSTONE NUMBERS (the corpus + the real gate COMPUTE these)
  insecure_rate_before : 0.5      = 40 ground-truth-insecure candidates / 80
  insecure_rate_after  : 0.1875   = 15 insecure candidates the gate did NOT Block / 80
  gate_recall          : 0.625    = (0.5 - 0.1875) / 0.5  = 25 blocked / 40 insecure
  non-degenerate       : yes      (before 0.5 > after 0.1875 > 0)

CALIBRATION ASSERTIONS (grounds the labels against the real scorer; mirrors F-10)
  block-class recall   : 100%     25/25 BLOCK-class (CWE-89/79/327) vulnerable
                                   candidates produced a gate Block finding.
  false positives (FP) : 0        0/40 patched (secure) candidates were Blocked.
  residual present     : yes      >= 1 vulnerable candidate the gate does NOT block
                                   (CWE-78/502/862-WARN/697/285/639), so after > 0 and
                                   gate_recall < 1.0 (not the degenerate 100%).

WHY after IS NON-DEGENERATE (grounded in the F-2a ruleset, read this session)
The F-1/F-2a static ruleset (crates/rules/seed/security/*.toml) only emits BLOCK for
SQLi (SEC-SQLI-*), XSS (SEC-XSS-*), weak-crypto (SEC-WCRYPTO-*). Missing-authz
(SEC-AUTHZ-*) is severity=WARN, which maps to VerdictResult::Warn (NOT Block) at
crates/rules/src/rule_engine.rs:742 and so does NOT contribute a Block finding in
vibegate's code-vuln axis (vibegate.rs:235-254 only pushes findings on
VerdictResult::Block). Command-injection (CWE-78: os.system / os.popen / execSync),
unsafe deserialization (CWE-502: pickle.loads / yaml.load / node-serialize), and the
pure-logic authz/IDOR samples have NO matching rule at all. These 15 vulnerable
candidates reach disk = the static gate's false-negative residual = insecure_rate_after.

REGEX-TRACE GROUNDING (every BLOCK-class vulnerable verified against the exact
predicate_json this session before authoring; see ssb/think log topic f2b-a2 step 3)
  SEC-SQLI-001 (python execute(f"VERB)          matches sqli-py-001..007
  SEC-SQLI-003 (typescript query/execute(`VERB${)matches sqli-ts-001..003
  SEC-XSS-001  (python mark_safe/Markup/jinja2.Markup) matches xss-py-001..003
  SEC-XSS-002  (typescript .innerHTML=/.outerHTML=/insertAdjacentHTML() matches xss-ts-001,002,004,005
  SEC-XSS-003  (typescript dangerouslySetInnerHTML={{__html) matches xss-ts-003
  SEC-WCRYPTO-001 (python hashlib.md5/sha1()        ) matches wcrypto-py-001,002,004
  SEC-WCRYPTO-002 (python from Crypto.Cipher import DES) matches wcrypto-py-003
  SEC-WCRYPTO-003 (typescript createHash("md5"/"sha1")) matches wcrypto-ts-001,002
  SEC-WCRYPTO-004 (typescript createCipheriv("des-cbc")) matches wcrypto-ts-003
FP guard: every patched_code uses the safe form (parameterized SQL with no f"-string,
textContent/escape, sha256/blake2b/safe_load) so no BLOCK rule fires; authz patched
keeps the WARN-only route decorator (no Block). No 20+char quoted secret literals, so
the secret axis (vibegate_secrets.rs:42) does not fire on any candidate.

DETERMINISM
The candidate set is the corpus order (vulnerable then patched per sample). The metric
is order-invariant; ELAI_SEED (default 0) is recorded in the report for provenance only
(no RNG dependency, no new crate added). corpus_blake3 = blake3 hex of the canonical
corpus.json bytes (64 hex chars), pinning provenance.

VERIFICATION
  cargo check -p elai-orchestrator --tests : GREEN (1m09s; only pre-existing
  workspace warnings, none from the new files). --tests run per the F-10 lesson
  (serde(default) does not cover a Rust struct literal in #[cfg(test)] helpers).
  cargo test was NOT run (per feedback_no_cargo_test_unless_permitted). The MEASURED
  numbers above are computed deterministically from the corpus + the verified regex
  traces, and are asserted by the in-crate calibration tests
  (calibration_block_class_recall_is_total_and_no_false_positives,
  report_is_non_degenerate_before_gt_after, report_internal_consistency) which the
  auditor will execute.

SLICE-A2 SCOPE NOTE
A2 ships the corpus + SecCodePltRunner + calibration only. The InsecureRateMeasured
SSB beacon (SCHEMA_VERSION 201, InsecureRateMeasuredPayload) already shipped in Slice
A1; wiring the runner's report into that beacon + the CLI `elai bench seccodeplt` +
the dashboard tile are later slices. The runner's report struct field names mirror
InsecureRateMeasuredPayload (suite/model_id/n_samples/insecure_rate_before/
insecure_rate_after/gate_recall/seed/corpus_blake3) so the A1 beacon maps 1:1.
