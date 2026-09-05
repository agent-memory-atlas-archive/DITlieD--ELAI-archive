F-2c DECISIONS — four low-FP CWE rules (798/321/532/95) + the first measured SeCodePLT static-gate baseline
Session: s-6312 (orchestrate Phase C). Date: 2026-06-03. FSM: EXECUTING.

SCOPE SHIPPED
Four CWE classes added to the operational `security` rule domain, scoped to the already-registered
Python + TypeScript grammars, plus the FIRST published before/after static-gate insecure-rate baseline
over the now-20-row ruleset. No multi-language expansion, no new crate, no ast_adapter edit, no SSB
variant, no SCHEMA bump.

DECISION 1 — GREP-ONLY, NOT GREP+AST COMPOSITE (controller design decision, implemented as directed)
The plan premised the four rules as `GrepPattern AND AstPattern` Composites. That premise is wrong for the
LIVE gate. evaluate_vibegate (vibegate.rs:133) builds its SECURITY_RULES from load_security_seed_rules_builtin
using a NoOpAst evaluator (vibegate.rs:104) whose evaluate_query ALWAYS returns Ok(vec![]). In a
Composite AND[GrepPattern, AstPattern] the AstPattern child therefore resolves to zero matches, the
predicate is false, and the WHOLE rule never fires on the production gate — it would be wire-dark.
TreeSitterAstEvaluator (crates/knowledge/ast_adapter.rs) is real and registers python+typescript, but it is
NOT the evaluator wired into evaluate_vibegate.
Resolution: ship all four CWE classes as GrepPattern-ONLY rules (anchored regex predicates), mirroring the
12 existing F-2a security rules exactly — all 12 are grep-only and all DO fire on the live gate. AST-precision
is NOT attempted here.
AST-PRECISION DEFERRED (future F-2 follow-up): wiring TreeSitterAstEvaluator into evaluate_vibegate's
SECURITY_RULES LazyLock (replacing NoOpAst) is the prerequisite for any Grep+AST Composite security rule.
That is a vibegate.rs change crossing crates/orchestrator -> crates/knowledge and is out of F-2c scope. Until
it lands, every security seed rule MUST be grep-only or it is wire-dark. The four F-2c rules are GREP-precision,
not AST-precision; FP is bounded by regex anchoring (Decision 3), measured (Decision 5), not claimed a priori.

DECISION 2 — SEVERITY: ALL FOUR ARE BLOCK, NOT WARN (deviation from the controller's CWE-532 WARN guidance)
The controller instruction suggested CWE-532 (secrets-in-logs) ship as WARN because it is a higher-FP shape.
Grounded reason to override: evaluate_vibegate's run_code_vuln_axis (vibegate.rs:236-254) only converts a
VerdictResult::Block into a HookFinding; a WARN verdict produces NO finding. severity_verdict (rule_engine.rs:739)
maps the rule's `severity` field "WARN" -> VerdictResult::Warn. Consequently a WARN security rule is:
  (a) gate-silent on the live PreToolUse gate (it never blocks a write, never emits a security_rule_blocked row), and
  (b) invisible to the seccodeplt bench — gate_blocks (seccodeplt_runner.rs:397) counts only Block findings, so a
      WARN rule leaves insecure_residual == insecure for its class => gate_recall = 0 for CWE-532.
Both the f2c block contract (each pattern must BLOCK a vulnerable snippet) and the per-CWE before!=after
requirement demand a Block. A WARN rule satisfies neither. The WARN guidance was premised on the Python
`print()` high-FP shape; instead of demoting the whole class to a gate-silent WARN, CWE-532 ships as BLOCK with
FP bounded by a conjunction (Decision 3). The print() shape is kept but only when a secret-named identifier is
also present in the same call. This is logged as a deliberate deviation; precedent for the WARN-is-gate-silent
fact is SEC-AUTHZ-001/002 (WARN), which never produce a gate Block.

DECISION 3 — PER-RULE FP-BOUNDING (grep-precision anchors; the Rust regex crate has NO lookahead/backref)
The regex crate cannot express "literal that is NOT a placeholder" with negative lookahead, so each rule uses a
positive structural anchor instead:
  CWE-798 (SEC-CREDS-001 py / -002 ts): a secret-named variable (password|passwd|secret|api_key|apikey|token,
    case-insensitive) assigned a DOUBLE-QUOTED string literal that CONTAINS A DIGIT and is >= ~3 chars:
      (?i)(password|...|token)\s*=\s*"[^"\n]{2,}[0-9][^"\n]*"
    The digit requirement is the FP bound: pure-alpha placeholders (CHANGEME, example, your-password) and the
    empty literal "" do NOT match; real credentials almost always carry a digit. Double-quoted only (mirrors F-2a
    and the F-1 secret axis, which excludes single-quoted strings to avoid SQL-fragment FPs). Complements the
    secret axis (vibegate_secrets.rs PAT_SECRET_ASSIGN), which only fires on >=20-char high-entropy literals;
    this rule catches the SHORTER hardcoded-credential shape the secret axis misses.
  CWE-321 (SEC-KEY-001 py / -002 ts): anchored to the crypto SINK plus a literal-key argument — the narrowest
    sink of the four. py: (AES|DES|ARC4)\.new\s*\(\s*b?"[^"\n]{8,}" | Fernet\s*\(\s*b?"[^"\n]{8,}".
    ts: createCipheriv\s*\(\s*"[^"\n]+"\s*,\s*"[0-9a-fA-F]{16,}". A key sourced from a variable
    (AES.new(key, ...) / createCipheriv("aes-256-cbc", key, iv)) does NOT fire. Disjoint from weak_crypto.toml,
    which gates the ALGORITHM, not key-provenance.
  CWE-532 (SEC-LOG-001 py / -002 ts): a CONJUNCTION in one regex — a high-signal log sink
    (logging.(info|debug|warning|error|critical) | logger.<m> | print | console.<m>) AND a secret-named
    identifier (password|passwd|secret|token|api_key|apikey) inside the same call argument list. A benign log
    line with no secret identifier does NOT fire. The sink+secret conjunction IS the FP bound (replacing the
    demoted WARN).
  CWE-95 (SEC-EVAL-001 py / -002 ts): grep the eval/exec sink, require the FIRST non-space char after the
    opening paren to be an identifier start [A-Za-z_], NOT a string-literal quote. So eval("1 + 1") (a constant —
    not injection) does NOT fire, while eval(user_input) / new Function(userCode) DO. py: (eval|exec)\s*\(\s*[A-Za-z_].
    ts: (eval\s*\(\s*[A-Za-z_] | new\s+Function\s*\(\s*[A-Za-z_]).
RESIDUAL FALSE NEGATIVES (documented, not fixed — grep precision limit):
  - CWE-95: a concatenation whose first token is a literal, e.g. eval("safe_" + user), is a residual FN
    (the [A-Za-z_] anchor sees the quote). AST would close this; deferred with Decision 1.
  - CWE-798: a single-quoted literal or a digitless real secret is a residual FN by the double-quote + digit anchors.
  - CWE-321: a base64/hex key shorter than the {8,}/{16,} floors, or a key built then passed by variable, is an FN.
All file_context = "production" (is_test_file excludes tests/, *_test.py, __tests__/, etc.).

DECISION 4 — LOCKSTEP REGISTRY EDITS (all five sites; TOML-only is INERT — the loader reads CANONICAL_ROWS)
The DB seed path reads the compile-time CANONICAL_ROWS array, NOT the seed/security/ directory, so authoring
TOMLs alone would not seed or enforce. Updated, in lockstep, byte-identical predicate_json across all:
  1. crates/rules/seed/security/{hardcoded_creds,hardcoded_key,secrets_in_logs,eval_injection}.toml — 8 new
     [[rules]] (py+ts per CWE), domain=security, tier=1, severity=BLOCK, phase_mask=4294967295.
  2. crates/orchestrator/src/rule_loader.rs SECURITY_SEED_TOMLS — appended the 4 include_str! entries (the
     LIVE-GATE load path; without this the live gate never sees the rules).
  3. crates/rules/src/vibesec_seed.rs — added 8 `const PRED_*` + 8 CANONICAL_ROWS tuples, kept the array sorted
     by rule_id ASC (sort test enforces it). Made CANONICAL_ROWS `pub` (was pub(crate)) so the contracts crate
     can assert the new rows are grep-only.
  4. crates/rules/src/project_rule_seeder.rs security_rule_meta — added a match arm per new rule_id returning
     (title-with-CWE, "BLOCK", cwe_comment); without it seeding falls through to ("Unknown security rule","WARN").
  5. COUNT CONSTANTS bumped 12 -> 20 at all THREE sites: f2_immune_rule_contract.rs:69 and :168, plus
     vibesec_seed.rs canonical_row_count_matches_expected.
SEED INTEGRITY: verify_vibesec_seed_integrity computes the expected blake3 digest from CANONICAL_ROWS at
runtime (canonical_hex_digest) and compares it to the digest of the seeded DB rows — there is NO hard-coded
digest literal. Adding rows recomputes BOTH sides from the same source, so the check stays consistent with no
hash literal to regenerate; the only content-derived constant that had to move is the row COUNT (done above).
The blake3 (never blake2/sha2) hashing is unchanged from F-2a.

DECISION 5 — ACTUAL ROW COUNT
12 F-2a rows + 8 F-2c rows (CWE-798 ×2, CWE-321 ×2, CWE-532 ×2, CWE-95 ×2) = 20 CANONICAL_ROWS rows. The plan's
"16" assumed one row per CWE; py+ts variants make it 20.

DECISION 6 — THE MEASURED BASELINE (the keystone number; RunPod-INDEPENDENT, offline static gate, NO model)
Command: ELAI_SEED=42 elai bench seccodeplt --json (drives the REAL vibegate::evaluate_vibegate over the
embedded corpus). Corpus extended with 8 new samples (py+ts per new CWE; vulnerable_code trips the rule,
patched_code does not) => 48 samples / 96 candidates. Deterministic (static gate is model-independent).
Report: .elai_cc/benchmarks/seccodeplt/seccodeplt-1780464568613.json. corpus_blake3 =
e162f48b02b0c86b32749a85f9543b269fb71ecc4c1de96514a55b3c57e328e6. One InsecureRateMeasured SSB row emitted.

  insecure_rate_before = 0.5000   (corpus is balanced: every sample contributes one insecure + one secure candidate)
  insecure_rate_after  = 0.15625  (the static gate's false-negative residual)
  gate_recall          = 0.6875   ((before - after)/before)
  n_samples (candidates) = 96, seed = 42

Per-CWE for the four NEW classes (insecure_candidates / insecure_blocked / insecure_residual / secure_false_blocks):
  CWE-798: 2 / 2 / 0 / 0   (both insecure caught; ZERO measured false positives on patched/placeholder)
  CWE-321: 2 / 2 / 0 / 0   (both insecure caught; ZERO measured false positives)
  CWE-532: 2 / 2 / 0 / 0   (both insecure caught; ZERO measured false positives)
  CWE-95:  2 / 2 / 0 / 0   (both insecure caught; ZERO measured false positives)
MEASURED FALSE-POSITIVE RATE for the four new rules on this corpus = 0/8 secure candidates = 0.00. This is the
MEASURED number on a small synthetic corpus, NOT an a-priori "low-FP" claim and NOT a transfer claim — the
placeholder/env-read/variable-key/benign-log negatives were authored INTO the corpus and the rules cleared them.
The non-zero residual after=0.156 comes from out-of-scope classes with no F-2c rule (CWE-78/285/502/639/697/862),
exactly as expected.

HONESTY CAVEATS (mandatory, section 1.4)
- This is a STATIC-gate insecure-rate (the gate's false-negative residual over ELAI's OWN synthetic corpus),
  DISTINCT from SeCodePLT's DYNAMIC sandbox-executed secure-coding rate (arXiv:2410.11096) and from any
  live-model number. No transfer claim. No live-Qwen claim (that stays a separate RunPod-gated checkbox).
- The FP=0 result is on 8 secure candidates the rules were tuned against; it bounds nothing about real-world FP.
  Release gate per the plan: bound FP on >= 3 real orchestrate runs before promoting confidence; do NOT publish
  "low-FP" as a property until that evidence exists.
- "below 15%" target and GitGuardian-replay framing intentionally kept OUT (section 1.4 corrections inherited).

TESTS AUTHORED (auditor runs them; main LLM ran only the bench)
- crates/contracts/tests/f2c_cwe_rule_block_contract.rs — per CWE: a vulnerable production snippet BLOCKs via the
  real evaluate_vibegate path; its patched/placeholder/env-read counterpart does NOT block; the same vulnerable
  snippet in a test-context file does NOT fire.
- crates/contracts/tests/f2c_predicate_shape_contract.rs — the 8 new CANONICAL_ROWS rows deserialize to
  GrepPattern (not composite), file_context=Production, registered language (python|typescript); count==20;
  blake3 seed integrity passes over the 20-row seeded DB; every new rule_id has a real security_rule_meta arm
  (severity BLOCK, title names the CWE).
- crates/orchestrator/tests/f2c_seccodeplt_baseline_smoke.rs — the offline runner produces n_samples>=80,
  before>after, gate_recall in (0,1), a per-CWE breakdown INCLUDING 798/321/532/95 each with insecure_blocked>0
  and not-all-secure-false-blocked; and (tokio runtime) one InsecureRateMeasured row lands in a tempdir SSB store.

W-10: F-2c is entirely Tier-1 in-process (grep rules + an offline static-gate bench). No network, no subprocess,
no RunPod at gate time or bench time. Multi-language expansion, ast-grep-core, AST-precision, and the live-model
secure-rate run are all explicitly deferred to future F-Ns.

cargo check --workspace --tests: GREEN (no errors from the F-2c changes; pre-existing unused-import warnings only).
