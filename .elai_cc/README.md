> [!WARNING]
> Historical documentation for an abandoned experiment. This text is not evidence
> that the described behavior works. Read the [archive warning](../README.md) first.

# ELAI core (`.elai_cc/`)

This is the project root for the ELAI deterministic development engine. The
top-level repo README at `../README.md` covers product positioning; this file
documents operational concerns the binaries surface.

## Tamper-Recovery (D-8 Archive Verify)

The reasoning-template ArchiveChain (`.elai_cc/archive/reasoning-templates.jsonl`)
is the append-only ledger of every promoted, frozen, and retired
ReasoningTemplate the orchestrator has ever produced. Each entry carries a
blake3 `self_hash` over `(template_body, parent_hash)` plus a `parent_hash`
reference to its predecessor. The chain is intentionally line-diffable in
git review (DGM §5 archive practice) so any out-of-band edit is visible at
PR time.

### Detecting tampering

Two equivalent detection paths land the same diagnosis:

1. **Developer side — `elai archive verify`**
   ```
   elai archive verify --path .elai_cc/archive/reasoning-templates.jsonl
   elai archive verify --json --fail-fast    # CI mode
   ```
   Exit `0` = clean chain. Exit `1` = at least one of:
   - `HashMismatch` — a `template_body` was rewritten without recomputing
     `self_hash` (canonical lobotomy attack).
   - `BrokenParent` — an entry's `parent_hash` references a hash that was
     never appended earlier (forged ancestry or chain-truncation attack).
   - `DuplicateHash` — two entries share a `self_hash` (accidental
     double-append, or a cryptographically improbable collision attempt).
   - `MalformedJson` — a JSONL line failed to deserialise; the verify
     driver uses the lenient loader so subsequent lines continue to be
     walked.

   The `--json` flag prints the typed `ArchiveVerifyReport` (jq-parseable
   single object). The `--fail-fast` flag short-circuits on the first
   error — recommended for CI; the collect-all default is recommended for
   local debugging so you see every problem in one pass.

2. **CI side — `.github/workflows/archive-verify.yml`**
   Runs `cargo build --release --bin elai` then invokes
   `elai archive verify --fail-fast` against the in-repo archive on every
   push and pull_request to any branch. The workflow skips itself if the
   archive file is absent so PRs that don't touch the archive aren't
   blocked by a missing fixture.

Every invocation — pass or fail — also writes a single
`SsbActionType::ArchiveVerifyResult` row to the project's session SSB DB
so dashboards can render a verify-history timeline without re-walking the
JSONL.

### Recovery procedure (verify failed)

If `elai archive verify` reports a failure, **do not edit the JSONL by
hand**. The repair path:

1. **Identify the tampered range.** Run the verify driver in collect-all
   mode (no `--fail-fast`) and note every `entry_index`. The lowest
   index is the earliest divergence from a trusted state; everything
   from there forward is suspect because hash-chain semantics make
   later entries depend on earlier ones.

2. **Cross-check against the RAG / sidecar snapshots.** The audit-log
   retention policy keeps a 30-day window of SSB rows for every
   `ArchiveEntryAdded` and `ArchiveEntryRetired` event. Each row carries
   the `template_id`, `content_hash`, and `parent_hash` at the moment of
   the append. Reconstruct the trusted state by walking the SSB log
   forward from the most recent verified-clean checkpoint.

3. **Rebuild the chain from the trusted snapshot.** Append-only
   semantics mean repair is *not* in-place editing — it is a NEW chain
   that replays the trusted history forward. The old (tampered) chain
   is preserved verbatim under `archive/quarantine/<timestamp>.jsonl`
   so forensic analysis can compare the divergence.

4. **Re-run `elai archive verify` on the rebuilt chain.** Exit `0` is
   the gate; the SSB row for that invocation is the audit trail.

### Trust positioning

The chain rationale is supported by two pieces of literature (neither
specifies the implementation; see
`workflow/reasoning/D-8-decisions.md` DECISION-5):

* arXiv:2507.05619 (T1 cryptographic-checksum recommendation) frames the
  threat model — agents rewriting their own audit history to suppress
  reward-hacking evidence.
* DGM §5 (archive practice) frames the design pattern — append-only
  lineage exposes diff so the §H hallucination-marker-removal incident
  could not have gone undetected.

Future hardening tracks (orthogonal to D-8):
* **SLSA Build L3** — generate a provenance attestation for every
  release that pins the source commit + build environment, so
  consumers can verify the binary was built from the audited chain.
  Status: deferred to a release-engineering wave.
* **Sigstore signing** — cosign-sign the JSONL on every committed
  promotion so the chain is verifiable against a public transparency
  log even outside ELAI's session SSB. Status: deferred to D-9 (Locked
  Keys Auditor) integration.

## Honest positioning (E-23)

The 2026 developer-discourse data is bimodal. capability is up, trust is
down. ELAI is built for the user who sits in the ship-with-it cohort
with the broken-and-getting-worse cohorts hygiene rules. these are the
load-bearing claims, every one anchored to a source.

### what ELAI is

ELAI is an FSM-gated, grounded, audited code-work harness. it is not a
chat assistant. every state transition writes an SSB row, every claim
cites its source, every audit cycle runs a real test pass. the trust
telemetry plane (ContextStatePanel + TrustTelemetryPanel) surfaces the
five signals the report flagged as the trust gap: tasks-per-hour,
auditor findings per task, revert rate, ungrounded-claim count, and
stagnation distribution.

Reference: METR RCT (arXiv:2507.09089, Becker et al., 16 senior OSS
devs, 246 tasks, Feb-Jun 2025) found Cursor Pro + Sonnet 3.5/3.7 made
developers 19% slower while they felt 20% faster. ELAI doesn't claim to
beat that number. the productivity-baseline-2026-05-19 artifact is a
baseline only; paired-task data is what would make it a comparison.
TrustTelemetryPanel exists to let a user build that data themselves.

### what ELAI is not

* not a full-autonomous YOLO agent. every irreversible action passes
  through the user-confirm gate.
* not a surprise-priced product. the runtime_choice_badge in the
  status strip reads `local` or `cloud - billed` at all times, derived
  from the B-37 runtime_fallback row. when the local sidecar OOMs and
  falls back to Anthropic, the badge transitions visibly.
  Reference: Cursor June 2025 backlash + Stack Overflow 2025
  (29% trust on AI-generated code, survey.stackoverflow.co/2025).
* not a thick SDK over the raw API. the CLI speaks the Anthropic
  protocol natively.

### where the moats sit

* **rules-as-data, not prompt-soup.** CLAUDE.md + `.claude/rules/` +
  per-skill SKILL.md files are loaded as data the harness routes
  around, not strings concatenated into a system prompt. the
  PreCompact hook saves FSM state to SESSION_HANDOFF before
  truncation, so context-loss is visible (Anthropic issue #13171 +
  #19567 grounding).
* **per-level auditor.** every level of decomposition (concept /
  phase / step / babystep) gets its own auditor mode with scoped
  recall criteria. matches Cloudflares 7-specialised-reviewer
  pattern (blog.cloudflare.com/ai-code-review). grounded by
  arXiv:2404.03602 (Kamoi COLM, prompt-scope recall lift 17-37pp).
* **local-first hybrid.** sidecar (NF4 Qwen2.5-Coder-7B on RTX 3090,
  ~1.92GB VRAM, 25-28K context production cap) handles the bulk of
  inference; cloud Sonnet/Opus is opt-in per role via PerRoleRuntime
  (B-37 SHIPPED). the moat is by-construction: ELAI runs without a
  cloud account.
* **cross-session FSM.** the FSM state persists across compaction
  via SESSION_HANDOFF. activeContext.md + the current plan file are
  re-read after compaction without losing work.
* **grounded reasoning chain.** every mcp_think call requires a
  grounding source (file path, grep, search, tool result, or
  explicit `training:unverified`). the ungrounded counter on
  TrustTelemetryPanel surfaces drift.

### capability curve (third-party verified)

* JetBrains AI Pulse April 2026
  (blog.jetbrains.com/research/2026/04/which-ai-coding-tools-do-
  developers-actually-use-at-work) reports 90% regular AI use and
  Claude Code 91% CSAT / NPS 54 — the model layer ELAI consumes is
  measurably stronger than 2024 baselines.
* Birchler Feb 2026 (birchtree.me/blog/the-data-has-changed) reports
  Sonnet 3.5 ~40% pass to Opus 4.6 ~90% on entire-feature tasks. the
  raw model improved; the harness moat is hygiene + verification
  around the model, not the model itself.
* Cloudflare AI code review blog (blog.cloudflare.com/ai-code-review)
  reports 7-specialised-reviewer composition gives sharper signal
  than one monolithic reviewer. ELAIs per-level auditor matches the
  shape.

### prompt-injection visibility

prompt-injection is a real production attack vector (CVE-2026-26268
Cursor IDE CVSS 9.8; Copilot CVSS 9.3; Willison Jan 2026 "Challenger
disaster" prediction). E-23 does not implement injection defence
(that is the C-42 fuzzer + E-37 PromptArmor track). E-23 surfaces a
`prompt_injection_check_status` row in the CommandWindow status strip
so the user can see whether the fuzzer has run in the current
session. status reads `passed` / `blocked` / `not_run`. when `not_run`
for more than 24h, the row reads yellow.

### caveat on third-party tool blocking

Anthropics April 2026 policy change may block third-party CLI tools
that pretend to be Claude Code over the Anthropic Console API. ELAIs
local-first claim survives that policy because the sidecar speaks the
Anthropic protocol natively and is not a wrapper over the Console API.
the cloud path (`runtime: cloud - billed`) does go through the Console
API and is subject to the same policy.
