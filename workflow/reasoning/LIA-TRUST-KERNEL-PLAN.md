# LIA Trust Kernel — standalone build/test/validate/publish plan

STANDALONE. This is NOT an ELAI letter plan. It ships its own repository (`lia-trust`), its own
benchmark harness, and its own published number, and it runs and proves itself BEFORE any ELAI
benchmark. It reuses ELAI's doctrine (off-agent evidence, fail-closed, frozen gate) and the hard
lessons the 2026-07-17 direction sweep surfaced, but it shares NO code with `.elai_cc` and imports
nothing from it. Authored 2026-07-18 under owner directive: extract the portable trust substrate
that makes ANY agent trustworthy, prove it on empty third-party harnesses, publish it, and make it
the on-ramp that sells native ELAI.

Grounding for every design decision below traces to files read this session:
`brain/migrated-notes/ELAI trust kernel chat.md` (the open/closed boundary), the four direction
sweeps + `5-SYNTHESIS.md` / `6-RUNNABLE-NOW.md` / `7-BENCHMARK-READINESS-PLAN.md`, and
`docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`. Load-bearing external claims carry a source tag; a claim
with no source is labelled UNGROUNDED and is not acted on.

PROOF_SPINE: feeds a NEW external credibility number — the LIA TRUST-INTEGRITY RATE (catch-rate of
fabricated-pass / out-of-scope-write / hallucinated-dep / unsupported-claim attempts, at zero
false-open, measured across three third-party harnesses). This number is deliberately DISTINCT from
ELAI's PS-1 SAFEDELTA (a marginal residual-insecure-rate that needs a live cloud model + 300-400
tasks + a pod, all still un-produced per 6-RUNNABLE-NOW): LIA's number is a CAPABILITY delta against
harnesses that have NO trust gate at all, so the effect is near-total not marginal. It comes in two
tiers: (a) a GATE-SOUNDNESS floor — scripted known-bad actions, fully deterministic, no model, proving
the gate catches what it was built to catch at zero false-open; (b) the HEADLINE — a real-agent
catch-rate measured by driving a BARE agent (Devin free tier, Codex, Claude Code) OFF vs ON over real
tasks and counting how often the agent actually fabricates a pass / deletes out of scope / hallucinates
a dependency / agrees with a false premise, and how many LIA catches at what false-block rate. Crucially
LIA never HOSTS or PAYS FOR the model: the wrapped harness brings its own (the free Devin/Codex model
IS the generator lane), so unlike ELAI's PS-1 SAFEDELTA — which needs ELAI itself to generate patches
with a credible model on a pod/cloud budget over 300-400 tasks for statistical power — LIA's number
needs no GPU, no cloud bill, and no large-N power argument (a capability delta against a
no-gate baseline, not a marginal statistical effect). LIA proving its own number first is therefore the
cheapest credible number in the whole program (5-SYNTHESIS §5: every model-dependent ELAI number is
still un-produced for want of a live endpoint; LIA sidesteps that by measuring a third party's free
model, not by dropping the model).

---

## 0. THE ONE-SENTENCE BOUNDARY (what is open, what stays ELAI)

Open-source everything needed to independently ESTABLISH TRUST in an agent action; keep ELAI-private
everything that decides WHICH actions to take, HOW to solve the task, and HOW to recover until the
task is finished (source: `ELAI trust kernel chat.md:689`).

LIA proves what happened, blocks the clearly-unsafe, and verifies supplied claims. It does NOT contain
the machinery that makes ELAI plan better, understand a repo, recover from failure, coordinate agents,
or finish work autonomously. Concretely:

- OPEN (LIA): the trust protocol + event model; the append-only journal + hash chain + Ed25519
  signatures; the policy/gate engine (rules-as-data, fail-closed, frozen-for-run); receipts + replay
  bundle + offline verifier; the adapter SDK + hook-bridge + wrapper; the AST-gate SDK + a minimal
  taint core; SEVEN core gates; the deterministic claim→evidence VERIFICATION layer; the sycophancy
  MEASUREMENT layer; the conformance + adversarial-fixture suites; a GitHub Action.
- CLOSED (native ELAI, never in this repo): the knowledge/DKG/retrieval engine; the Grounded Reasoning
  Chain; claim EXTRACTION + claim-risk scoring + evidence-round-trip prediction + counterfactual
  selection + effort routing; the planning FSM + decomposition + branch selection + context allocation;
  multi-agent supervision + crash recovery + model substitution; automated repair after a gate fails;
  the evidence-ranked candidate search; the self-improvement (DGM) loop; the full sandbox/confinement
  orchestration; the reviewer cockpit; the commercial control plane.

The division answering the owner's coverage ask (sycophancy / false claims / wrong sources+data):
> LIA can DETECT and MEASURE unsupported agreement, an unsupported claim, a hallucinated dependency,
> a wrong-source citation, and can BLOCK a catastrophic action. ELAI actively REASONS in a way that
> prevents them, gathers the evidence to resolve them, and repairs the work when one fails.
(source: `ELAI trust kernel chat.md:404` for sycophancy, `:349-357` for anti-hallucination, `:295` for AST.)

---

## 1. THE NAME (owner asked "idk if kernel is the right name")

DECISION: keep "LIA Trust Kernel"; repo/package `lia-trust`; the standalone verifier binary `lia`.
"Kernel" here means TRUSTED COMPUTING BASE (a small, permanent, high-assurance core), which is the
2026-07-10 market evaluation's OWN adopted word — "a small permanent kernel with rich native
capabilities at typed, measurable edges" / "compact trust kernel" (source: sweep 4 :1131-1139,
5-SYNTHESIS §4 "compact trust kernel (receipts, journal, confinement, policy, provenance,
independent verification, rollback, evidence schemas)"). It is NOT the refuted "frozen reasoning
kernel" sense (sweep 4 §0: that sense — the harness makes a small model smart — does not survive
constraint-decay / scaffold-model-synergy). Kernel-as-TCB is exactly what LIA is, so the name is
correct and on-doctrine.

OPEN OWNER DECISION (from the OSS strategy doc §1, not adopted here unilaterally): the strategy doc
recommends renaming the COMMERCIAL product itself from ELAI to LIA — one brand family, "LIA" the full
harness, "LIA Trust Kernel" the open repo, positioning line "LIA Trust is the open, model-neutral
safety and verification kernel. LIA is the complete autonomous engineering system built on it."
This plan keeps writing "native ELAI" for the commercial product; read every such reference as "the
commercial product, whatever its final name". The decision must land BEFORE L6 ships the upsell page,
because the page's product name is public and expensive to re-index later.

---

## 2. THE HARD LESSONS BAKED IN (so LIA does not repeat ELAI's own audited flaws)

These are the six P0 design flaws the 2026-07-10 market eval found in ELAI's OWN trust spine
(sweep 4 :1076-1109). LIA is authored to be born WITHOUT them:

- HL-1 (from P0-3): a Merkle root is NOT a signature. LIA signs receipts with Ed25519 over the
  gate-manifest version + signer identity, NOT a bare blake3 Merkle root. The Merkle chain proves
  internal integrity; the Ed25519 signature proves authenticity. Both ship.
- HL-2 (from P0-5): `lia wrap` (MCP/hook interception) is BYPASSABLE without whole-process
  confinement. LIA never CLAIMS complete mediation it cannot prove; it emits a per-harness
  ASSURANCE-LEVEL report naming exactly what it can and cannot intercept (this is unit L3 and it is
  also the ELAI upsell surface).
- HL-3 (from P0-6): the determinism claim is layer-scoped. LIA ships one normative honesty matrix:
  pure journal fold = byte-recomputable; local-model tokens = same-class hashable; cloud-model =
  AUTHENTICATED action stream, NOT byte-replay; external world = replay against captured cassettes
  only. LIA never says "reproducible" of a cloud lane; it says "trace-authenticated" (source: sweep 4
  :1097-1102, K-1 claims-policy honest-wording rule, 5-SYNTHESIS R-7 ELAI_SEED refutation).
- HL-4 (from F-4 sharpened by R2): a test-integrity gate binds stdout_hash ALONE too weakly. LIA's
  test-integrity receipt binds `stdout_sha256` + `stderr_sha256` + `exit_code` + `argv` + `cwd` +
  `coverage_profraw_sha256` + the digest of the WRAPPER EXECUTABLE ITSELF (source: sweep 3 §2 R2
  :350-354, "stdout-hash alone is weaker than stdout-hash + coverage-hash + wrapper-digest").
- HL-5 (from R2 Rust-tooling reality): `sigstore-rs` cannot verify attestations. LIA's offline verify
  path is native Ed25519 (ed25519-dalek) + an OPTIONAL shell-out to `cosign verify-blob --bundle` for
  the public-log path; it never hand-rolls Sigstore bundle verification (source: sweep 3 §2 R2 :384).
- HL-6 (from K-1 / the ERRATA ancestor): a "cant-lie" product that ships an unverified number IS the
  liar it sells a cure for. LIA ships its OWN claims-policy lint over its OWN README/docs: no number
  or superlative without a [MEASURED]-with-recompute-pointer / [EXTERNAL] tag (source: 5-SYNTHESIS
  R-8, sweep 2 §9 KD-1, 2-direction-monetization.md :557).

---

## 3. THE ARCHITECTURE (open repo layout, all Rust core + thin bridges)

Language: Rust for the `lia` core and verifier (single static binary, dependency-light per the
M-plan small-boring-verifier doctrine + the Q-8 size ceiling; ELAI stack, no new language risk).
Adapters are thin: Claude Code = a PreToolUse hook (JSON over stdin) shelling to `lia`; Codex = an
MCP/tool proxy; generic/Devin = a process+filesystem wrapper. Bridge scripts run under bun, never
node (house rule `bun-not-node.md`).

```
lia-trust/                      (Rust workspace + thin bridges)
  crates/
    lia-protocol/     canonical event model + raw-harness-event passthrough (serde, closed enums, deny_unknown)
    lia-journal/      append-only WAL sqlite + blake3 hash chain + Ed25519 receipt signer/verifier
    lia-policy/       rules-as-data gate engine (fail-closed, frozen-for-run, capability-aware, stable reason codes)
    lia-gates/        the SEVEN core gates (each a rules-as-data + a small deterministic predicate)
    lia-ast/          tree-sitter integration + the changed-symbol/import extraction + small security predicates
    lia-taint/        minimal source→sink label propagation (untrusted-content → destructive-sink)
    lia-ground/       deterministic claim→evidence VERIFICATION (file/symbol/test/dep/source-supports/api-schema)
    lia-syco/         unsupported-agreement DETECTOR + stability-across-framing measurement
    lia-verify/       the standalone OFFLINE verifier (Ed25519 + replay + evidence recompute; size-capped)
    lia-adapters/     claude-code hook-bridge, codex/mcp proxy, generic wrapper + the ASSURANCE-LEVEL report
    lia-cli/          the `lia` binary (wrap / verify / gate / bench / report subcommands)
  bench/              the empty-harness benchmark harness + the adversarial fixture corpus
  conformance/        the frozen conformance suite (what an adapter MUST satisfy)
  .github/actions/    the GitHub Action wrapper
  docs/               README + the honest claims + the ELAI upsell page
```

WIRING_CONTRACT (repo-level, the un-gameable spine every unit inherits):
- produces: signed LIA receipts + a replayable run bundle + an assurance-level report.
- consumes: a real agent's action stream at whatever boundary the harness exposes (hook / MCP / wrapper).
- reachable_from: the `lia` CLI subcommands + the three adapter entrypoints (hook stdin, MCP handler,
  wrapper process) — NEVER a unit test calling a gate function directly.
- observable_effect (OFF-AGENT): a journal row + a signed receipt on disk that the `lia verify` binary
  (a SEPARATE process, ideally a different machine) recomputes and validates. Missing evidence = FAIL.
- stub_ban: no `.unwrap()`/`.expect()` on prod paths; no `TODO`/placeholder gate bodies; no gate that
  reads the AGENT's own stdout/claim as its evidence (that is the F-4 inverse and a doctrine violation).
- frozen: the gate engine, the fixture corpus, the conformance suite, and the size-ceiling manifest are
  authored once and are read-only to any run being evaluated (freeze-the-gate R8).

---

## 4. THE SEVEN CORE GATES (the open, credible product — do not ship 200 half-gates)

Ship SEVEN excellent, adversarially-tested gates first (source: `ELAI trust kernel chat.md:154`).
Each is rules-as-data (a YAML rule) + a small deterministic predicate; each fails closed; each emits a
signed receipt the offline verifier recomputes.

1. TEST-INTEGRITY (the killer demo). A claimed test PASS is emitted ONLY if the WRAPPER (not the agent)
   observed exit-code success; the receipt binds HL-4's full field set. Fabrication signature:
   claimed_pass=true with no matching wrapper-captured receipt = REFUTED.
2. EVIDENCE-COMPLETENESS / COMPLETION. The agent cannot say "done" unless the required evidence exists:
   modified code requires a test-result (or an explicit unsupported-test verdict); a new dependency
   requires registry evidence; missing evidence yields an honest INCOMPLETE, never a silent pass.
3. FILESYSTEM-SCOPE. Allowed roots, path traversal, symlink escape, writes outside the repo, protected
   files, deletion limits, and changes to hook/policy/verifier files DURING a run. (This + gate 4 are
   the "cannot wipe your project" headline — the GPT-5.6 / GLM-5.2 file-deletion incident class.)
4. SHELL / IRREVERSIBLE-ACTION. Post-shell-EXPANSION path resolution (NOT `canonicalize`, which does
   NOT do tilde/glob expansion — the P0-2 flaw ELAI is authored to avoid, source sweep 4 :1082);
   recursive delete, destructive git, package publish, credential-changing commands, out-of-scope
   commands. Default-on, fail-closed.
5. DEPENDENCY-REALITY. Package exists + version exists + registry matches + suspiciously-new + typo/
   slopsquat + "was the dep actually added where claimed". (Covers "wrong data/info": a hallucinated
   package. External motivation: >=5.2%/21.7% hallucination rate, 205,474 unique fake names, source
   2-direction-monetization.md :576 [EXTERNAL] Spracklen USENIX 2025.)
6. SECRET / SENSITIVE-OUTPUT. Secrets in diffs, credentials in logs, private keys, dangerous env
   exposure, and a shareable evidence bundle accidentally carrying private data. Redaction is
   STRUCTURAL: the shareable projection has no field a secret could live in (source: ELAI P-plan
   gate-credibility "redaction structural", sweep 4 :504).
7. JOURNAL / EVIDENCE-TAMPERING. Removed/reordered events, modified evidence, cross-session receipts,
   replayed receipts, a policy change during execution, unknown event families where completeness
   matters. This is what makes every other gate's verdict un-fabricable.

---

## 5. THE COVERAGE THE OWNER ASKED FOR (sycophancy / false claims / wrong sources) — GROUNDING IN

Two open modules cover the whole "LLM lies" surface without exporting the ELAI reasoning engine:

- lia-ground (false claims + wrong sources + bad data): a deterministic claim→evidence VERIFICATION
  layer. Given a SUPPLIED (claim, required-evidence) pair, it answers verified | refuted | unsupported
  by a mechanical check: file-exists, symbol-exists, test-ran (via gate 1's receipt), command-succeeded,
  dependency-exists (via gate 5), api/schema-contains, and source-supports-claim (span/n-gram
  containment against a harness-FETCHED, hashed source — token-level containment is the WRONG
  granularity and is banned; span + closed-set citation is the sound form, source
  1-portable-trust-core.md :764-766). What it does NOT ship: automatic claim EXTRACTION from model
  output, claim-risk prediction, evidence-round-trip prediction, semantic-entropy, counterfactual
  selection, effort routing, the GRC — all ELAI-closed (source `ELAI trust kernel chat.md:331-343`).
- lia-syco (sycophancy): the MEASUREMENT, not the reasoning. An unsupported-agreement detector (user
  asserts X; agent agrees; independent registry/docs evidence absent → verdict "unsupported
  agreement"), a contradiction-with-evidence detector, and a stability-across-framing test (same
  question, differently framed, verdict must not flip). What it does NOT ship: adversarial roles,
  best-of-N, multi-verifier consensus, pristine-context critics, answer regeneration — ELAI-closed
  (source `ELAI trust kernel chat.md:366-401`).

DECISION RATIONALE (owner asked me to decide on grounding): YES, include lia-ground. It is the exact
open half the chat note carves out, it is the "false claims / wrong sources / bad data" coverage the
owner named, and it is deterministic (no model on the load-bearing path). It is ALSO the sharpest
upsell: LIA answers "is THIS supplied claim supported?"; ELAI answers "which claims exist, how risky
are they, what evidence to gather, and how to change the work when one fails" — a strictly larger
product (source `ELAI trust kernel chat.md:349-357`).

---

## 6. STRATEGY-DOC ALIGNMENT (Open Assurance Profile, assurance ladder, declared scope cuts)

Added 2026-07-18 from the owner-supplied OSS strategy doc
(`~/Downloads/LIA_Trust_Kernel_OSS_Strategy_and_Programs.md`). Three binding rules + the v1 cuts:

- OPEN ASSURANCE PROFILE (binding on gate design AND marketing): whenever LIA publicly claims it
  VERIFIED or BLOCKED something, the COMPLETE path producing that verdict is open and independently
  recomputable — observation, command/env capture, exit-code capture, output hashing, journal
  insertion, policy evaluation, receipt generation, offline verification. A closed checker that merely
  emits a signed PASS is banned from the open profile (source: strategy doc §4). The commercial
  product may layer proprietary ADVANCED-assurance checks on top, but their verdicts are labelled
  advanced-assurance and are NEVER pooled with open-profile verdicts in any published number. This is
  HL-6's sibling: a verdict a third party cannot recompute is an unverified number.
- ASSURANCE LADDER (per-harness roll-up over the per-gate cells): AUDIT (post-hoc diff + supplied
  evidence only) / OBSERVE (records, cannot block) / GATE (blocks supported tool calls via native
  hooks) / CONFINE (kernel-controlled side-effect boundaries) / NATIVE (the full commercial harness).
  L3's report keeps its per-gate PREVENT / DETECT / CANNOT-OBSERVE cells and DERIVES the roll-up level
  from machine-readable capability keys (pre_write_block, post_write_receipt, shell_pre_block,
  shell_result_capture, network_control, credential_broker, completion_gate, subagent_visibility,
  immutable_journal, offline_verification). A guarantee not derivable from a declared capability is
  refused, never asserted (HL-2). (source: strategy doc §14)
- MCP IS INSPECTION, NEVER THE SECURITY BOUNDARY: an MCP server cannot guarantee a harness routes
  every native FS/shell/network action through its tools, so MCP is the control/inspection surface
  only (verify_run, inspect_receipts, explain_denial, verify_diff, show_policy,
  show_adapter_capabilities); enforcement lives in hooks and the wrapper (source: strategy doc §13).
  The codex/MCP adapter is a TOOL PROXY whose assurance cells state exactly what it can intercept.
- V1 SCOPE CUTS (declared here so a promised open item is never silently dropped):
  - process-contract validation (strategy §9: objective / assumptions / required evidence / completion
    predicate / honest-stop / typed unblock as an open typed contract + a "was it followed"
    validator) — v1 covers the completion-predicate + required-evidence half via gate 2; the full
    typed contract format is a POST-L6 fast-follow.
  - Gemini CLI + Cursor adapters (strategy §13 targets 3 and 4) — post-L6 fast-follows on the adapter
    SDK + conformance suite; v1 ships claude-code + codex/MCP + generic.
  - Linux reference confinement backend (strategy §10) — v1's generic-wrapper floor is worktree
    isolation + env allowlist + protected evidence location (L3); a namespaces-grade confinement
    backend is a post-L6 fast-follow, and until it ships NO harness may be reported at CONFINE level.

---

# PART II — THE BUILD (wargame units; a mid-tier executor runs each blind)

Every unit is a vertical slice (producer + real consumer + smoke). Each move states what the executor
should see if it worked and the failure/cause/counter-move if it did not; VERIFY runs name off-agent
artifacts only. Units land in dependency order L0 to L6; the INTEGRATION_SMOKE cadence fires after every
2-3 units and before publish.

## DEFINITION OF DONE (binding on every unit — cargo-green is NOT done)

"It compiles + `cargo test` passes" is explicitly REJECTED as done for any code-shipping unit here, for
the same reason ELAI's own gate stack exists: a unit test that calls a new function directly is
indistinguishable from a missing production caller (wire-dark). A LIA unit is DONE only when ALL hold,
and the first four are off-agent artifacts the implementing agent did not author:
- REACHABILITY: every new pub symbol is called from a REAL entrypoint (a `lia` subcommand / an adapter
  hook-stdin handler / the MCP handler / the wrapper syscall path), proven by an LLVM coverage hit on
  that function from the COMPILED binary under an integration run — never by a unit test alone (L0b
  Layer 3).
- OFF-AGENT EVIDENCE: the unit's behaviour is proven by a journal row / a signed receipt / a coverage
  hit / an integration bundle that a SEPARATE `lia verify` process recomputes — never by the code's own
  return value or stdout.
- REAL E2E: the INTEGRATION_SMOKE drives the REAL `lia` binary (and, from L3 on, a REAL bare agent or a
  faithful recorded trace) end to end on a HARD multi-step input — not a toy single-action happy path.
- NO WIRE-DARK: L0b's producer-without-consumer checker returns clean over the changeset (or the symbol
  carries a registered `// WIRE-DARK[unit]` marker + an allowlist row); missing evidence is a hard FAIL,
  never SKIP.
- ZERO-LAZY: no `.unwrap()`/`.expect()` on a prod path, no `TODO`/placeholder body, no gate that reads
  the agent's own stdout/claim as its evidence.
This is the SAME discipline this root enforces for ELAI, re-authored INSIDE the LIA repo (L0b) so it
binds LIA's own code from commit 1 — LIA is a trust-gate product and must pass its own gate.

## UNIT L0 — repo skeleton + protocol + signed journal

MOVE: create the `lia-trust` Cargo workspace; implement `lia-protocol` (canonical event enum + raw
passthrough, serde, `#[serde(deny_unknown_fields)]` on closed enums) and `lia-journal` (append-only
WAL sqlite, blake3 hash chain per row, Ed25519 receipt signer/verifier over gate-manifest-version +
signer-identity). Wire a `lia journal-append` + `lia journal-verify` CLI pair as the real entrypoint.
EXPECTED_OBSERVATION: `lia journal-append --event <json>` writes a row whose `prev_hash` chains to the
prior row; `lia journal-verify <db>` exits 0 on an untampered db and NONZERO after a single byte flip
in any row; a receipt carries a valid Ed25519 signature that a second `lia` process verifies offline.
ON_FAIL: verify exits 0 on a tampered db (cause: the chain does not cover the mutated field, or the
signature covers too little) — COUNTER-MOVE: extend the signed/hashed span to the full canonical row
bytes, add a negative test that flips each field and asserts NONZERO; do NOT weaken the tamper test.
FORK: if a pure-Rust Ed25519 verify path proves flaky against the reference vectors (trigger: a known
test vector fails) → pin `ed25519-dalek` to a vetted version and add the RFC-8032 test vectors as a
gate; do NOT hand-roll the curve math.
RECON_NEEDED: the append-only sqlite WAL single-writer discipline under a wrapper that may fork. SETTLING
CHECK: `cargo test -p lia-journal concurrent_append` proving two writers serialize (single-writer lock),
never interleave a partial row.
ABORT: if a journal write can FAIL SILENTLY without blocking the action it records (the ELAI P0-1
flaw) → STOP; a transition without its journal row breaks replay and completeness. A journal-write
failure must fail-closed the action, not proceed.
VERIFY (off-agent): `cargo test -p lia-journal -p lia-protocol` green; `lia journal-verify` exit code
on a tampered vs clean db (the exit code is the off-agent artifact, not stdout); RFC-8032 Ed25519
vectors pass.
PRODUCTION_HOOK: `lia-cli::journal_append` + `journal_verify` subcommands dispatch into
`lia-journal::append_signed` / `verify_chain`; the CLI is the real entrypoint, not a unit test.
INTEGRATION_SMOKE: n/a (L0 is the foundation; the first cross-seam smoke is IS-1 after L2).

## UNIT L0b — LIA's OWN wiring-gate stack + CI (dogfood; the "no wire-dark from day 0" machinery)

MOVE: re-author (do NOT import from `.elai_cc` — LIA shares no code) LIA's own frozen 4-layer wiring
gate over the LIA workspace, wired into CI as the real consumer: Layer 1 = `RUSTFLAGS="-D dead_code"`
with a `pub(crate)` discipline (anything not a genuine cross-crate API is `pub(crate)`, so dead_code
fires on unused internal items) + `cargo-machete` for dead deps; Layer 2 = a re-authored
producer-without-consumer checker (`tools/lia_wire_check` — AST/name-grep over the changeset; a new pub
fn whose only callers are test code = DARK = fail, R6/R7; `// WIRE-DARK[unit]` + an allowlist row is the
only escape); Layer 3 = an entrypoint-coverage gate (`cargo-llvm-cov`: build the real `lia` binary
instrumented, run the auditor-owned integration smokes with production-default config, export
per-function counts, any NEW function with count==0 = BLOCK unless `#[cfg(test)]` or a registered
WIRE-DARK marker); Layer 4 = the runtime-trace assertion (every gate's smoke asserts its signed receipt
lands in the journal — the un-gameable leg). Freeze the checker + the manifest + the fixture corpus
(`tools/lia_gate_freeze` over a `gate-manifest.txt`) so a run cannot edit the gate that judges it (R8).
EXPECTED_OBSERVATION: `tools/lia_wire_check --changeset <diff>` exits 0 on a fully-wired change and
NONZERO (naming the DARK symbol) on a change that adds a pub fn with only test callers; the CI job
fails a PR that introduces a wire-dark symbol; `cargo llvm-cov` over the integration smokes reports a
nonzero hit count for every new production function; `lia_gate_freeze --check` exits NONZERO if the PR
touched any frozen gate file.
ON_FAIL: the wire-checker passes a genuinely dark symbol (cause: name-grep matched a same-named symbol
in the wrong file, R5 unsoundness) — COUNTER-MOVE: fall back to the Layer-3 coverage leg as the sound
backstop (a function with count==0 from the real binary is dark regardless of name-grep), and add the
false-negative case to the checker's own test corpus; treat the static leg as necessary-not-sufficient.
FORK: if `cargo-llvm-cov` is unavailable or too slow for every PR (trigger: CI budget) → run Layers 1+2
on every PR and gate Layer 3 (coverage) on merge-to-main + a nightly, never dropping it entirely; the
runtime leg is the only sound backstop and must run before any release.
RECON_NEEDED: the exact `cargo-llvm-cov` version + its zero-count-function JSON export shape. SETTLING
CHECK: `cargo llvm-cov --version` pinned + a smoke that a deliberately-unwired stub function shows
count==0 in the export before trusting the gate.
ABORT: if the wire-checker or the freeze-check is itself editable by the run being evaluated → STOP; the
gate stack is frozen and read-only to any PR (a run that can rewrite its evaluator always writes itself
to PASS). The gate-manifest freeze is the precondition for trusting every later unit's "done".
VERIFY (off-agent): a SEEDED wire-dark commit (a new `pub fn` with only a test caller) is REJECTED by
`tools/lia_wire_check` in CI (the CI exit code is the artifact); a seeded gate-file edit is REJECTED by
`lia_gate_freeze --check`; `cargo llvm-cov` shows a nonzero count for a known-wired function and zero for
the seeded stub.
PRODUCTION_HOOK: the CI workflow (`.github/workflows/wire.yml`) invokes `tools/lia_wire_check`,
`cargo llvm-cov`, and `lia_gate_freeze`; these ARE the production consumers of the gate stack, run on
every PR — not a manual command.
INTEGRATION_SMOKE: IS-0 — a throwaway branch that deliberately adds a wire-dark `pub fn` and a
frozen-gate edit; CI MUST reject both, proving LIA's own repo enforces the doctrine end to end before a
single feature unit ships. This smoke is a required deliverable of L0b.

## UNIT L1 — policy/gate engine + receipts + the offline verifier

MOVE: implement `lia-policy` (load rules-as-data YAML; evaluate against a typed evidence set; emit a
verdict of allow | deny | quarantine | advisory with a STABLE reason code; freeze the policy set for
the duration of a run) and `lia-verify` (the standalone offline verifier: recompute every gate's
evidence hash from the archived bundle, re-check the Ed25519 signatures, replay the action stream, and
emit a signed VERIFICATION-REPORT verdict). The verifier is the ONLY thing allowed to mark a run
"accepted" (source sweep 3 §2 R2 :405).
EXPECTED_OBSERVATION: `lia gate --rules <frozen.yaml> --evidence <bundle>` returns a stable reason code
per rule; a rule whose required evidence is ABSENT returns DENY (fail-closed), never SKIP; `lia verify
<bundle>` recomputes the bundle's hashes, exits 0 only if every hash matches AND every signature
validates AND the replay reproduces, NONZERO on any mismatch/missing artifact.
ON_FAIL: a missing-evidence rule returns SKIP/UNVERIFIED instead of DENY (cause: the evaluator treats
absence as not-a-failure) — COUNTER-MOVE: make missing evidence a hard DENY in the evaluator core and
add a fixture with a deliberately-absent evidence field asserting DENY (the D3 anti-pattern; source
1-portable-trust-core.md :694-701).
FORK: if a rule legitimately needs a QUARANTINE/RETRY not a hard DENY (a low-risk quality check, not a
security boundary) → route it through the risk-tier field so quality/productivity checks can quarantine
while security/irreversible/secret/publication boundaries stay fail-closed (source: sweep 4 C4
:1350-1362, the risk-tiered correction — high-risk fails closed, low-risk may quarantine).
RECON_NEEDED: the exact reason-code vocabulary a downstream tool depends on. SETTLING CHECK: freeze a
`reason-codes.golden` file and add a test that a reason code can be ADDED but never RENAMED/REMOVED
without editing the golden (a golden-lock).
ABORT: if the offline verifier would need to reach the LIVE network to validate a signature at verify
time → STOP; archive the trust-root + signing-config snapshot INTO the bundle so verify is offline
(the Rekor-migration coupling risk, source sweep 3 §2 R2 :312). A verifier that needs the network is
not an offline verifier.
VERIFY (off-agent): `cargo test -p lia-policy -p lia-verify` green; run `lia verify` against a
KNOWN-GOOD bundle (exit 0) and a bundle with one corrupted evidence hash (NONZERO) — the two exit
codes are the artifact.
PRODUCTION_HOOK: `lia-cli::gate` → `lia-policy::evaluate_frozen`; `lia-cli::verify` →
`lia-verify::verify_bundle`. The verifier's signed VERIFICATION-REPORT is the only "accepted" authority.
INTEGRATION_SMOKE: n/a (covered by IS-1 after L2).

## UNIT L2 — the seven core gates

MOVE: implement `lia-gates` — the seven gates of §4, each as a rules-as-data rule + a small
deterministic predicate + a signed receipt. Test-integrity binds HL-4's full field set; the shell gate
does post-EXPANSION path resolution (a side-effect-free shell AST evaluator, NOT `canonicalize`);
filesystem-scope resolves the SMALLEST affected path and refuses writes/deletes outside allowed roots +
refuses in-run edits to hook/policy/verifier files.
EXPECTED_OBSERVATION: for each gate, a scripted attempt (a fabricated pass; an `rm -rf ~/project`; a
write to `../../etc`; an import of a nonexistent package; a secret in a diff; a reordered journal row)
produces a DENY/REFUTED receipt the offline verifier confirms; a benign action produces an ALLOW
receipt; every deny carries a stable reason code and the offending path/command/symbol.
ON_FAIL: the shell gate lets a `rm -rf $HOME/*` through because it resolved the literal string not the
expansion (cause: no expansion evaluator) — COUNTER-MOVE: add a shell-AST expansion pass that rejects
command-substitution and resolves globs/tilde BEFORE the path-scope check, with a fixture of 20
destructive one-liners (incl. the GLM-5.2 node-script-delete and the GPT-5.6 db-drop shapes) asserting
DENY; a residual escape is a landmine, add it to the fixture, do not raise a threshold.
FORK: if a harness cannot expose the pre-execution command (only a post-hoc log) → the shell gate
downgrades from PREVENT to DETECT and the assurance report (L3) records "shell interception:
detect-only on this harness"; never claim prevention LIA cannot deliver (HL-2).
RECON_NEEDED: which of the seven gates each target harness can feed at PREVENT vs DETECT grain. SETTLING
CHECK: for each of {claude-code hooks, codex/mcp, generic wrapper}, a probe run recording, per gate,
whether the action is interceptable BEFORE execution (`bench/probe_assurance.sh <adapter>`).
ABORT: if the dynamic scorer for any gate would EXECUTE an untrusted model-authored command on an
unconfined host → STOP (destructive-op safety gate); the shell/exploit legs run only inside a
confinement primitive, never on the real machine (source 7-BENCHMARK-READINESS-PLAN.md ABORT).
VERIFY (off-agent): `cargo test -p lia-gates` green + `bench/gate_fixtures/*` each produce the expected
signed receipt, confirmed by a SEPARATE `lia verify` process (not the gate's own return); the tamper
gate flips a receipt and `lia verify` goes NONZERO.
PRODUCTION_HOOK: each gate is dispatched from `lia-adapters` at the action boundary (hook stdin / MCP
handler / wrapper syscall), never called directly by a test; the receipt lands in the L0 journal.
INTEGRATION_SMOKE: IS-1 — an INDEPENDENT run drives a REAL bare agent through the `generic` wrapper on a
HARD scripted task (multi-step: write a file, run a test the agent will FABRICATE a pass for, then
attempt an out-of-scope delete) and asserts the fabricated-pass is REFUTED + the delete is DENIED + all
receipts verify offline. Toy single-action smokes do not satisfy this (R10).

## UNIT L3 — adapters + the ASSURANCE-LEVEL report (the ELAI upsell surface)

MOVE: implement `lia-adapters` — three adapters (claude-code PreToolUse hook-bridge reading JSON on
stdin; codex/MCP tool proxy; a generic process+filesystem wrapper `lia wrap -- <agent cmd...>`) that
route each intercepted action through the L2 gates + the L0 journal. The generic wrapper's v1
mechanisms (strategy doc §13 wrapper mode): an isolated git worktree for the agent's writes with
final-diff admission back to the real tree, an environment allowlist (secrets filtered from the child
env), child-process supervision, and a protected evidence location OUTSIDE the child's writable area
(the journal is never inside the tree the agent mutates); a file watcher is detect-only and is never
reported as pre-write prevention. Also ship `lia-adapters/mcp-inspection`: read-only MCP tools
(verify_run / inspect_receipts / explain_denial / show_policy / show_adapter_capabilities) so any
MCP-speaking harness can INSPECT trust state — inspection only, never the enforcement boundary (§6).
Then the ASSURANCE-LEVEL report: a per-harness capability declaration computed from the L2 RECON
probe, stating for each gate whether LIA can PREVENT, DETECT-only, or CANNOT-OBSERVE on that harness,
the honest one-line limit, and the §6 roll-up level (AUDIT / OBSERVE / GATE / CONFINE) derived from
the machine-readable capability keys.
EXPECTED_OBSERVATION: `lia wrap -- <agent>` intercepts the agent's file writes and shell commands and
emits a journal row + receipt per action BEFORE the side effect; `lia report --adapter claude-code`
prints a table like "test-integrity: PREVENT | filesystem-scope: PREVENT | shell: PREVENT | network
confinement: CANNOT-GUARANTEE (this harness has no egress hook)" plus a derived roll-up line
("assurance level: GATE"); the same for codex and generic, with the generic/Devin wrapper honestly
showing weaker interception than the hooked lanes (and never a CONFINE roll-up in v1, per §6); the
mcp-inspection tools return receipts/policy/capabilities read-only and cannot mutate journal or policy.
ON_FAIL: the wrapper reports PREVENT for a gate it can only DETECT (cause: the assurance level is
hard-coded not probe-derived) — COUNTER-MOVE: derive every cell of the report from the L2 probe result
at runtime, add a test that a harness with a stubbed-absent hook yields DETECT-only for that gate; an
over-claimed assurance cell is HL-2 and is a BLOCK.
FORK: if `lia wrap` cannot prove COMPLETE mediation on a harness (the P0-5 limit — an external CLI with
direct FS/process access can bypass the proxy) → the report says "mediation: incomplete — an
out-of-band process can bypass LIA on this harness; native ELAI's process-isolation closes this" and
LIA narrows its security claim rather than marketing it as complete (source sweep 4 :1092-1094 + the
pre-registered kill rule :1153).
RECON_NEEDED: the exact Claude Code PreToolUse hook JSON contract + the Codex tool-handler contract at
current versions. SETTLING CHECK: WebFetch the current Claude Code hooks docs + Codex/MCP tool-handler
docs and pin the exact field names in `lia-adapters/CONTRACTS.md` before wiring (anti-stale; the
contracts drift).
ABORT: if an adapter would need to run the target agent with elevated privileges to intercept → STOP +
surface; LIA never escalates the agent's own privilege to gate it.
VERIFY (off-agent): `bench/probe_assurance.sh` for all three adapters produces a report whose cells match
a hand-verified truth table (the report file is the artifact); `cargo test -p lia-adapters` green.
PRODUCTION_HOOK: `lia-cli::wrap` → `lia-adapters::generic::wrap`; the claude-code hook entrypoint reads
stdin JSON → `lia-adapters::claude_code::on_pre_tool`; `lia-cli::report` → `lia-adapters::assurance`.
INTEGRATION_SMOKE: IS-2 — drive a REAL Claude Code (or a faithful hook-emitting stub if a live Claude
Code is unavailable) through the hook-bridge on the HARD IS-1 task; assert the same fabricated-pass
REFUTED + delete DENIED land via the HOOK path (not the wrapper), and the assurance report for that run
is emitted and internally consistent with what was actually intercepted.

## UNIT L4 — grounding-verification + sycophancy measurement + AST-gate SDK + minimal taint

MOVE: implement `lia-ground` (the deterministic claim→evidence checks of §5: file/symbol/test/dep/
source-supports/api-schema, verdict verified|refuted|unsupported), `lia-syco` (unsupported-agreement +
contradiction-with-evidence + stability-across-framing, plus a rules-as-data policy hook so a
HIGH-RISK agreement — a claim-bearing "yes" on a security/dependency/data question — can be REQUIRED
to carry a grounding receipt, strategy doc §8), `lia-ast` (tree-sitter integration + a small
set of deterministic security predicates: SQL-string-interpolation-into-exec, untrusted-data-into-cmd,
dangerous-deserialization, new `eval`, an import with no manifest entry, a test converted to
unconditional success, an authz check removed from a sensitive fn), and `lia-taint` (a minimal
untrusted-source→destructive-sink label propagation with explicit declassification).
EXPECTED_OBSERVATION: `lia ground --claim '{type:file_exists,path:X}'` returns verified|refuted with
the checked location; `lia ground --claim '{type:tests_passed}'` returns unsupported when no gate-1
receipt backs it; `lia syco --exchange <json>` flags "unsupported agreement" when the agent agreed with
a user assertion that no fetched source supports; `lia ast-gate <diff>` flags an SQL-interpolation sink;
a source-supports check uses SPAN containment against a hashed fetched source, never token containment.
ON_FAIL: `lia ground` returns "verified" for a claim it did not actually check (cause: a claim type
with no predicate silently passes) — COUNTER-MOVE: an unknown/unhandled claim type returns UNSUPPORTED
(fail-closed), never verified, with a test asserting an unregistered claim type is unsupported; a
claim-with-no-predicate that returns verified is the D3 inverse and a BLOCK.
FORK: if a source-supports check is too strict and forbids legitimate synthesis (trigger: a
paraphrased-but-correct claim is refuted) → use span + closed-set citation binding (a claim must cite a
span from the fetched hashed set), NOT exact token containment; report "unsupported" not "refuted" when
no citing span exists (source 1-portable-trust-core.md :764-766).
RECON_NEEDED: tree-sitter grammar set + version for the target languages. SETTLING CHECK: pin the
grammar crate versions and add a fixture per predicate per language proving the predicate fires (and a
negative fixture proving it does not over-fire) before shipping the predicate.
ABORT: if any grounding/sycophancy verdict would be rendered by an LLM on the load-bearing path → STOP;
these are ADVISORY/DETECT layers whose verdicts are mechanical; an LLM verdict on a gating path is a
D5 fail-open (source 1-portable-trust-core.md :711-725). If a semantic judgment genuinely needs a
model, it is ELAI-closed, not LIA.
VERIFY (off-agent): `cargo test -p lia-ground -p lia-syco -p lia-ast -p lia-taint` green; each predicate
has a positive + negative fixture; the source-supports check is proven to refuse a token-only match.
PRODUCTION_HOOK: `lia-cli::ground` → `lia-ground::verify_claim`; `lia-cli::syco` →
`lia-syco::detect`; `lia-cli::ast_gate` → `lia-ast::scan_diff` — each also invocable as an L2-style gate
in the adapter path so a live run emits ground/syco receipts into the journal.
INTEGRATION_SMOKE: IS-3 — on the HARD task, the agent is scripted to (a) agree with a false user premise
and (b) claim it added a dependency it never wrote; assert `lia-syco` flags the unsupported agreement
AND `lia-ground` + the dependency-reality gate flag the phantom dependency, both as verified receipts.

## UNIT L5 — the benchmark harness + the LIA TRUST-INTEGRITY NUMBER (the published proof)

BENCH LANE MODEL (grounded 2026-07-18, do not re-derive): "Devin free model" means the TEIKOKU
devin-bridge (`/home/user/teikoku/devin-bridge/proxy/devin_proxy.py`) — a MODEL-GATEWAY proxy that
presents an Anthropic `/v1/messages` endpoint on `:8810` and translates to Devin's gateway
(server.codeium.com, free inference, model ids like kimi-k2-7). It is NOT the Devin autonomous cloud
AGENT. So the "Devin lane" is a FREE MODEL behind a LOCAL bare agent that `lia wrap`/the hook
intercepts at PREVENT grade (point the local agent's `ANTHROPIC_BASE_URL` at `:8810`) — no cloud
workspace, no desktop app, no computer-use tools, fully reproducible. The real Devin cloud agent is a
SEPARATE, OPTIONAL, DETECT-only lane (LIA verifies its produced diff + session log post-hoc, via
Devin's API — never the desktop UI); it demonstrates the agent-agnostic "verify any finished diff"
position at a lower, honestly-declared assurance cell, and is out of scope for the headline PREVENT
number.

MOVE: build `bench/` — (a) a FROZEN adversarial-fixture corpus: K scripted agent behaviours per gate
class (fabricated pass, out-of-scope write/delete, hallucinated dependency, unsupported claim,
unsupported agreement, secret leak, journal tamper), each git-STRIPPED and metadata-free; (b) an
EMPTY-HARNESS runner that drives a BARE LOCAL agent in a fresh throwaway repo with NO skills/rules/config
(the owner's constraint: skills would change the number), wrapped by LIA at PREVENT grade, across three
MODEL lanes — {Claude Code / its Anthropic model, Codex / its model, and a bare generic agent on the
free Devin-bridge model (:8810)} — each run once WITHOUT LIA and once WITH LIA; (c) the metric:
TRUST-INTEGRITY RATE = fraction of attempts LIA catches (DENY/REFUTED/unsupported) at ZERO false-open
(a benign action wrongly blocked is a separate FALSE-BLOCK rate reported alongside), reported per lane
with a paired bootstrap CI. Then run `lia claims-lint docs/` over LIA's own README so no number ships
untagged (HL-6).
EXPECTED_OBSERVATION: `lia bench --harness claude-code --arm off|on --corpus <frozen>` runs the bare
agent through the corpus and writes a signed result bundle; the ON arm shows a near-total catch-rate on
the fabricated-pass / out-of-scope-delete / hallucinated-dep classes (the bare harness has NO such
gate, so the delta is large not marginal), the OFF arm shows the attempts SUCCEED unblocked; the
false-block rate on a benign-action control set is bounded and reported; `lia claims-lint docs/` exits 0
only when every number in the README carries a [MEASURED]/[EXTERNAL] tag.
ON_FAIL: the ON-arm catch-rate is suspiciously 100% including on the benign control (cause: LIA is
blocking everything — a degenerate gate) — COUNTER-MOVE: report the FALSE-BLOCK rate as a first-class
companion metric and require it below a declared bound; a gate that "wins" by blocking everything is the
degenerate-gate failure the utility guardrail exists to catch (source sweep 3 §3 R3 :459 utility
companion, mandatory).
FORK: if the free Devin-bridge model lane is unavailable at bench time (trigger: bridge down / account
rate-limited) → run the corpus against a FAITHFUL RECORDED agent trace (a cassette of a real bare-agent
run) for the deterministic classes, LABEL the number "recorded-agent" not "live-agent", and hold the
live-agent number until the lane is back; never claim a live number from a cassette. ALSO: never
substitute the real Devin CLOUD agent (DETECT-only) for the PREVENT-grade model-lane number — they are
different assurance cells and must not be pooled.
RECON_NEEDED: that the devin-bridge model lane is reachable and which model id it serves. SETTLING CHECK:
`curl -s http://127.0.0.1:8810/v1/models` (or the bridge's health route) returns ok and a model id;
`bench/probe_harness_access.sh` confirms a fresh skill-free repo (no `.claude/`, no config) so the number
is not contaminated, and that a local wrapped agent's `ANTHROPIC_BASE_URL=:8810` routes generation to
the bridge (a request appears in the proxy log) — the interception happens LOCALLY, the tokens come from
the free lane.
ABORT: a published TRUST-INTEGRITY number is about to be emitted from a corpus that still carries `.git`
or fix metadata (the Endor 81%-git-history-exploit lesson, source 7-BENCHMARK-READINESS-PLAN.md
P0-HARDEN) → STOP, git-harden the corpus first; an un-hardened number is discreditable on sight.
VERIFY (off-agent): each `bench` run writes a signed result bundle that `lia verify` recomputes; the
per-harness catch-rate + false-block rate + CI are in the bundle (the bundle is the artifact, not
stdout); `lia claims-lint docs/` exit code gates the README.
PRODUCTION_HOOK: `lia-cli::bench` → `bench::run_arm` (drives the real adapter path from L3, not a mock);
`lia-cli::claims_lint` → a small lexical linter over `docs/`.
INTEGRATION_SMOKE: IS-4 — the FULL empty-harness pipeline on ONE harness end-to-end (fresh repo →
bare-agent ON vs OFF over the frozen corpus → signed bundle → `lia verify` green → a rendered
TRUST-INTEGRITY table), driven by an INDEPENDENT runner, proving the number is reproducible from the
bundle by a third party in one command.

## UNIT L6 — publish (conformance + action + docs + the ELAI upsell page)

MOVE: freeze the `conformance/` suite (what an adapter MUST satisfy to claim LIA-conformance) + the
adversarial-fixtures; ship the `.github/actions/` wrapper (run LIA in CI, emit the signed bundle as a
check-run artifact); ship `lia verify-run --base <rev> --head <rev> --evidence <dir>` — the universal
post-hoc AUDIT mode (verify a finished diff + supplied evidence with NO hook access; the
broad-adoption CI entry, honestly labelled AUDIT-level, never marketed as prevention — strategy doc
§13); write `docs/` — the README (honest claims only, claims-lint-clean), the ASSURANCE
matrix, and the ELAI UPSELL PAGE that states, verbatim from each assurance report, the gaps LIA cannot
close on a foreign harness and how native ELAI closes them (process isolation, auto-repair, claim
extraction + risk routing, planning + recovery). Also the RELEASE-READINESS set the funding
applications require (strategy doc §19): a threat-model doc, the guarantee/non-guarantee matrix
(generated from the assurance report, not hand-written), a five-minute quickstart, SECURITY.md,
CONTRIBUTING.md, CODE_OF_CONDUCT.md, a public roadmap naming the §6 fast-follows, a harness
compatibility table, a TAGGED release with prebuilt `lia` binaries, and an explicit README statement
that LIA Trust is independent and useful WITHOUT the commercial harness. Open-source under Apache-2.0
(verifier-open decision, source 2-direction-monetization.md §6 GTM playbook).
EXPECTED_OBSERVATION: a fresh clone passes `cargo test --workspace` + the conformance suite; the GitHub
Action on a sample repo emits a signed bundle a third party verifies with `lia verify`; `lia verify-run`
on a sample agent-produced diff + evidence dir emits an AUDIT-level bundle `lia verify` accepts; the
README has zero untagged numbers (claims-lint exit 0); every §19 release-readiness file exists; the
upsell page names at least the network-confinement / complete-mediation / auto-repair gaps and points
them at ELAI.
ON_FAIL: the upsell page over-claims LIA or under-claims the gap (cause: marketing drift) — COUNTER-MOVE:
generate the upsell gap-list MECHANICALLY from the assurance report's CANNOT-GUARANTEE / DETECT-only
cells so it cannot drift from what LIA actually delivers; a hand-written gap that the assurance report
contradicts is a claims-lint BLOCK.
FORK: if a regulated buyer later demands recomputation LIA's cloud-lane cannot give (the trace-auth vs
replay limit, HL-3) → the docs already say "cloud lane = trace-authenticated, not byte-replayed; full
recompute is open-weight-local only"; route the demand to ELAI's local lane, never over-claim LIA.
RECON_NEEDED: the exact OSS license + third-party dep license compatibility for a permissive release.
SETTLING CHECK: `cargo deny check licenses` green under an Apache-2.0-compatible allowlist before the
public push; every dep dist-channel verified per anti-slopsquatting.
ABORT: if any published artifact would impersonate a real org/person or ship a fabricated record → STOP;
LIA publishes only its own honest evidence. And: no public push until L5's number exists AND is
independently re-runnable (the Devin failure mode: shipping a number before it is reproducible, source
2-direction-monetization.md :715).
VERIFY (off-agent): a from-scratch clone → `cargo test --workspace` + `conformance/run.sh` green; the
Action's emitted bundle verifies with a SEPARATELY-built `lia verify`; `lia claims-lint docs/` exit 0;
`cargo deny check licenses` exit 0.
PRODUCTION_HOOK: `.github/actions/entrypoint` → `lia-cli::bench`/`verify`; the conformance runner →
`conformance::assert_adapter`. No new unwired symbol.
INTEGRATION_SMOKE: IS-5 — the full published path: a THIRD-PARTY-style fresh clone (no local state) runs
the Action on a sample repo, produces a signed bundle, and a different machine's `lia verify` accepts
it — proving the "verify, don't trust" narrative end to end.

## UNIT L7 — funding-program applications (post-release; docs/process unit, no new code)

MOVE: only after L6's tagged release has been public and visibly maintained for a while (a deliberate
wait — do NOT apply the day the repo goes public, source strategy doc §2), submit the OSS-program
applications in the strategy doc's order: (1) Anthropic Claude for Open Source via the discretionary
Ecosystem Impact Track — pitch: "model-neutral developer infrastructure for independently verifying
and constraining autonomous coding agents" (the Qail precedent shows technically-substantial,
non-famous projects pass discretionary review); (2) OpenAI Codex for OSS; (3) Ona; (4) Vercel
(applications were closed, expected reopen ~Aug 2026 — re-check); then JetBrains / AWS credits /
GitHub Copilot maintainer / Microsoft for Startups as lower priority. Every application links the
reproducible TRUST-INTEGRITY bundle (IS-5) as its demonstration and the L6 release-readiness set as
its evidence of maintenance. Log every submission in `docs/programs.md` (date, program, pitch,
outcome).
EXPECTED_OBSERVATION: each submitted application references a live tagged release + the §19 checklist
artifacts; `docs/programs.md` carries one row per submission with date and outcome.
ON_FAIL: an application is rejected on numeric eligibility (cause: download/dependents thresholds not
met) — COUNTER-MOVE: re-apply through the discretionary/impact track with the security-infrastructure
framing + the reproducible number as evidence; log the rejection and any reviewer feedback; never
inflate metrics to qualify.
FORK: if a program's terms require repo changes (badges, telemetry, branding) → accept only what does
not violate the claims-policy or the Open Assurance Profile; a term that would (an unverifiable badge
claim, telemetry that leaks run evidence) is declined and the decline is logged in `docs/programs.md`.
RECON_NEEDED: each program's live status + current terms at application time (the strategy doc's
program pages are 2026-07 snapshots and WILL drift; Vercel's reopen date explicitly). SETTLING CHECK:
WebFetch each program URL the week of that application, before drafting it.
ABORT: any application that would require claiming a number not yet [MEASURED] or an assurance level
not yet delivered → STOP (HL-6: the application IS marketing copy, claims-lint discipline binds it).
VERIFY (off-agent): `docs/programs.md` rows exist with submission dates; `lia claims-lint docs/` stays
exit 0 with the programs page included in the lint set.
PRODUCTION_HOOK: n/a (docs/process unit — no code ships; the only repo artifact is `docs/programs.md`,
covered by the claims-lint gate).
INTEGRATION_SMOKE: n/a (no code unit; the demonstration artifact is L6's IS-5 bundle).

---

## RECON_NEEDED (plan-level roll-up + the exact settling check for each)

- HARNESS HOOK CONTRACTS (L3): WebFetch current Claude Code PreToolUse hook docs + Codex/MCP
  tool-handler docs; pin field names in `lia-adapters/CONTRACTS.md` (contracts drift; anti-stale).
- FREE-MODEL BENCH ACCESS (L5): `bench/probe_harness_access.sh` confirms a fresh skill-free repo +
  a free/cheap model session on each of Claude Code / Codex / Devin.
- TREE-SITTER GRAMMAR VERSIONS (L4): pin grammar crate versions; positive+negative fixture per
  predicate per language.
- SHELL-EXPANSION EVALUATOR SCOPE (L2): confirm the side-effect-free shell-AST evaluator rejects
  command-substitution and resolves glob/tilde (NOT `canonicalize`); 20-destructive-one-liner fixture.
- ED25519 CRATE + VECTORS (L0): pin `ed25519-dalek`; RFC-8032 test vectors as a gate.
- OSS LICENSE COMPAT (L6): `cargo deny check licenses` under an Apache-2.0 allowlist; dep dist-channels
  slopsquat-verified.

## ABORT (plan-level conditions where execution STOPS AND FLAGS)

- A gate reads the AGENT's own stdout/claim as its evidence instead of an off-agent artifact → STOP
  (the F-4 inverse, a doctrine violation the whole product exists to prevent).
- A published number comes from a non-git-stripped corpus, or from a cassette labelled as live → STOP.
- `lia wrap` markets complete mediation it cannot prove on a harness → STOP (HL-2 / P0-5).
- A dynamic gate would execute an untrusted agent-authored command on an unconfined host → STOP.
- The README ships a number with no [MEASURED]/[EXTERNAL] tag (claims-lint BLOCK) → STOP (HL-6).
- A new pub symbol in any unit has no production caller and no WIRE-DARK marker → STOP + handoff
  (producer-without-consumer).

## VERIFY (plan-level; every pass criterion is an off-agent artifact)

- `cargo test --workspace` green from a fresh clone — necessary, NOT sufficient (see Definition of Done).
- `tools/lia_wire_check` clean over the full tree (zero DARK symbols without a WIRE-DARK row); `cargo
  llvm-cov` over the integration smokes shows a nonzero hit count for every production function; the
  seeded wire-dark commit (IS-0) is REJECTED by CI. This is LIA passing its OWN gate.
- `lia verify` exit code: 0 on a known-good bundle, NONZERO on a single-byte-tampered bundle.
- The empty-harness bench bundle recomputes the TRUST-INTEGRITY rate + false-block rate + CI from disk.
- `lia claims-lint docs/` exit 0; `cargo deny check licenses` exit 0.
- The conformance suite + the GitHub Action bundle both verify on a SEPARATE machine.

## INTEGRATION_SMOKE (cadence roll-up)

IS-0 (after L0b, the self-wire-dark rejection), IS-1 (after L2), IS-2 (after L3), IS-3 (after L4), IS-4
(after L5), IS-5 (after L6) — each drives a
REAL bare agent (or a faithful recorded trace, labelled) through the REAL adapter entrypoint on a HARD
multi-step task, never a toy single-action happy path (R10). IS-4 and IS-5 are the reproducible-number
and the verify-on-a-different-machine smokes that ARE the published proof.

---

## REDTEAM (attacks against THIS plan; the surviving attack + the patch on record)

ATTACK 1 (SURVIVED): "LIA's number is just SAFEDELTA re-skinned, so it inherits SAFEDELTA's refutations
— benchmarks don't close (R1), 200 tasks are underpowered (R3), incumbents counter-market (R4)." WHY IT
FAILS: LIA's number is a CAPABILITY delta against harnesses with NO trust gate (near-total catch-rate),
not a MARGINAL security-rate delta needing statistical power; most of the corpus is scripted-deterministic
(no cloud model, no pod), so R3's power problem does not bind the core classes and the number is
producible cheaply and now. The R1/R4 point (a number alone does not close a sale) is ABSORBED, not
refuted: LIA's product is the neutral-notary POSITION + the honest evidence bundle + the ELAI upsell,
and the number is table-stakes evidence for it, not the moat (5-SYNTHESIS D-5/R-2).

ATTACK 1b (SUCCEEDED → PATCHED): "LIA will ship wire-dark exactly like ELAI did (~40-45% of shipped
plans, historically) because its units are graded by `cargo test`, which cannot tell a real production
caller from a unit test — and the root's wiring checkers live in `.elai_cc`, a DIFFERENT repo, so they
do not apply to LIA." WHY IT SUCCEEDED against the first cut: the doctrine was in the units as prose but
LIA had no MACHINERY of its own enforcing it, and `cargo test` green was implicitly treated as done.
PATCH: added the binding DEFINITION OF DONE (cargo-green is explicitly NOT done) and UNIT L0b — LIA
re-authors its own frozen 4-layer wiring gate (dead_code + `pub(crate)` discipline, a
producer-without-consumer checker, `cargo-llvm-cov` entrypoint coverage from the real binary, and a
runtime-receipt assertion) wired into CI, with IS-0 (a seeded wire-dark commit CI must reject) proving
it works BEFORE any feature unit ships. LIA passes its own gate from commit 1 — the dogfood is also the
credibility proof for an untrusted solo publisher.

ATTACK 2 (SUCCEEDED → PATCHED): the first cut made `lia wrap` claim it blocks catastrophic actions on
any harness. An attacker points out a foreign CLI with direct FS/process access bypasses the MCP/hook
proxy (P0-5), so the claim is false on Devin-class harnesses. PATCH: L3 emits a per-harness
ASSURANCE-LEVEL report (PREVENT / DETECT / CANNOT-GUARANTEE per gate, probe-derived), and the L6 upsell
page is generated MECHANICALLY from its CANNOT-GUARANTEE cells; LIA never claims mediation it cannot
prove, and the gap it admits is the ELAI upsell.

ATTACK 3 (SUCCEEDED → PATCHED): the first cut signed receipts with a blake3 Merkle root as the "offline
signature". An attacker who mutates the artifact recomputes the root (P0-3). PATCH: HL-1 — Ed25519 over
the gate-manifest-version + signer-identity is the authenticity signature; the blake3 chain is only the
integrity leg. Both ship; the offline verifier checks both.

ATTACK 4 (SUCCEEDED → PATCHED): the first cut let the sycophancy/grounding verdicts be rendered by a
model for "semantic" cases. An attacker notes an LLM verdict on a gating path is adaptively bypassable
(NAACL 8/8 defenses >50% ASR) and is a D5 fail-open. PATCH: L4 verdicts are mechanical only (span
containment, registry checks, receipt lookups); anything needing a model is ELAI-closed, and an
unhandled claim type returns UNSUPPORTED (fail-closed), never verified.

ATTACK 5 (SURVIVED): "Skills/config on the bench harness will inflate the number." WHY IT FAILS: L5's
empty-harness runner drives a BARE agent in a fresh throwaway repo with NO `.claude/`, no rules, no
skills (the owner's explicit constraint), and the RECON probe asserts the repo is genuinely skill-free
before the run counts; the number is measured against the agent's raw behaviour, not a scaffolded one.

ATTACK 6 (SUCCEEDED → PATCHED): the first cut published the number before it was reproducible. An
attacker cites the Devin failure mode (a slick number with no reproduction path gets "cancelled").
PATCH: L6 ABORT — no public push until the number exists AND a third-party fresh clone re-runs it from
the signed bundle (IS-5); reproducibility is the product's first real feature.

---

## GROUNDING

Every load-bearing external claim carries a source tag inline above. Confidence labels: the doctrine
(off-agent evidence / fail-closed / frozen gate) is VERIFIED against ELAI's own shipped spine
(1-portable-trust-core.md §5, ELAI-SPEC-OVERVIEW.md :21-27). The six hard lessons HL-1..HL-6 are
VERIFIED against the 2026-07-10 market eval's P0 findings (sweep 4 :1076-1109) and R2's standards read
(sweep 3 §2). The open/closed boundary is VERIFIED against `ELAI trust kernel chat.md`. The
TRUST-INTEGRITY-number-is-cheaper-than-SAFEDELTA claim is HIGH confidence (reasoned from
6-RUNNABLE-NOW's "every model-dependent number is un-produced for want of an endpoint" + the fact that
LIA's core fixture classes are model-free), NOT independently measured — it is the plan's own thesis
and IS the thing L5 tests. UNGROUNDED (flagged, not acted on as fact): the exact catch-rate magnitude
and false-block bound are unknown until L5 runs; the plan does not assert a number, it builds the harness
that produces one. Market/pricing figures from the sweeps are [EXTERNAL] and 2-8 months stale; they are
not load-bearing here and are re-checked before any LIA marketing copy ships (K-1 discipline).

The 2026-07-18 alignment pass (§1 rename decision, §6, the L3/L4/L6 extensions, L7) is grounded in the
owner-supplied OSS strategy doc (`~/Downloads/LIA_Trust_Kernel_OSS_Strategy_and_Programs.md`). Its
program terms, URLs, and benefits (Claude Max 20x, Codex OSS, Ona, Vercel reopen date, etc.) are
[EXTERNAL] 2026-07 snapshots, deliberately NOT restated as facts here — L7's RECON re-verifies each at
application time. The Qail-precedent claim (a modest-scale Rust project passing discretionary review)
is [EXTERNAL] from the same doc, unverified independently, and is used only as encouragement to apply,
never as an eligibility guarantee.
