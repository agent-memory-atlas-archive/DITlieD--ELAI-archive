# Grok prompt — ELAI trust-spine prioritization (Team of Experts)

Paste the text below as your message to Grok (set mode to **Heavy / Team of Experts**), and ATTACH the 8 files listed at the bottom. If file uploads misbehave, also paste the contents of `06-research-distilled-bundle.md` inline under section (B).

---

You are a Team of Experts (product strategy, AppSec, ML-eval methodology, developer-tools GTM, distributed-systems). Reason hard and adversarially. I am giving you (A) a brief on an existing system "ELAI" and what it has already built/planned, and (B) eight attached files: five deep-research reports (R1–R5) and three ELAI plan summaries (F/G/H). Your job is NOT to re-recommend generic best practices — ELAI already has most of them as designs. Your job is to reason about PRIORITIZATION, GAPS, REDUNDANCY, and RISK, given A + the attachments.

## A) ELAI BRIEF

ELAI = a Rust autonomous-coding agent harness wrapped around CLOUD frontier models (Claude Sonnet/Opus), with an optional local Qwen2.5-Coder sidecar. Core thesis (already settled): buyers no longer pay for "code quality/taste," but WILL pay for provable TRUST in AI-generated code (security + correctness + reproducibility + "the agent cannot fake its own tests"), because untrustworthy autonomous code causes expensive incidents. Intended wedge: agencies cleaning up "vibe-coded" software; "believe-the-number" via a reproducible security-delta benchmark.

ELAI's trust spine is DESIGNED across plan tracks F/G/H (attached). Many pieces are PLANNED or wire-dark; only ~28% is shipped-and-wired today. Key pieces:

- DOCTRINE: gates must read an OFF-AGENT artifact the agent did not author (a runtime event row in an append-only SQLite "Shared State Board"/SSB, a coverage hit, a span); missing evidence = hard FAIL; the gate is frozen/read-only to the run it evaluates.
- F-1 VibeGate: one fail-closed PreToolUse write-gate fanning a file-write across code-vuln + secret + hallucinated-dep + malicious-skill axes; "security" rules seeded mutable=0 (per-project edits cannot disable security).
- F-2 / F-2b: write-time code-vuln ruleset (SQLi/XSS/weak-crypto/authz) + the FIRST measured pre-write insecure-rate number via the shipped SeCodePLT runner (RUNPOD-PENDING; the keystone deliverable).
- F-3: CaMeL/Tool-Filter data-plane defense (lock researcher tool-set to a minimal subset before any untrusted byte is read).
- F-4: test-integrity gate — bind every claimed test PASS to an orchestrator-captured stdout_hash (blake3); auditor cannot emit passed=true without a matching captured-hash row (fabrication signature = passed=true + stdout_hash=None).
- F-9: hallucinated-dependency existence+age gate firing at WRITE time (registry HTTP check on import lines; cargo/npm/pypi).
- F-10: per-iteration security-MONOTONICITY gate (block if REFINE iteration N introduces a CWE class absent at N-1).
- G-1: harness-wide TAINT propagation — RTBAS/FIDES-grounded Provenance/Provenanced<T> + capability_gate already SHIPPED but constructed only in tests (dead wiring); G-1 lights up the 4 real ingest sites (web/MCP/file/sandbox) and uses DEPENDENCY-SCREENED join (NOT global join, which collapsed Slack utility to 33%).
- G-2 [KEYSTONE]: Reproducible-Run Artifact (RRA) = portable msgpack+zstd bundle of the SSB action stream + ELAI_SEED + HardwareClass + system_fingerprint + model pin + an evidence manifest of every F-gate's blake3 verdict hash + a blake3 root; `elai run --emit-rra` / `elai replay --verify`. Honest scope: cloud-lane runs are reproducible-ACTION-STREAM, not reproducible-TOKENS (Anthropic fingerprint opacity), so the guarantee for body-reading steps is verify-DIVERGENCE not full re-exec.
- G-5: Reviewer Cockpit + Diff-Surface-to-Review (DSR) metric = (lines needing full attention)/(total changed lines), computed deterministically from SSB; surface a finding only when backed by a deterministic gate verdict or a failing test (no LLM-opinion noise).
- G-6: SAFEDELTA = gate-ON-vs-gate-OFF ablation measuring marginal residual-insecure-rate reaching a human, on git-stripped held-out tasks (SUSVIBES/BaxBench/SeCodePLT/AgentDojo), per-CWE, bootstrap CI; never a SWE-bench rank.
- G-7: per-change confidence producer (logprob + self-consistency + oracle-free generated-test execution) feeding a shipped conformal-refusal consumer; + Dual-LLM lethal-trifecta quarantine.
- G-8: signed in-toto provenance attestation + offline `elai verify-attestation`; positioned as "designed to support EU AI Act Article-50 marking," not "certified."
- G-9: write-time secret/PII redaction on the SSB commit path (append-only rows force WRITE-TIME redaction).
- CONSTRAINTS: blake3 only; no .unwrap() in prod; anti-slopsquatting; small solo team; Anthropic-dependency is a known root risk; everything currently gates on ONE unproven SAFEDELTA number.

## B) RESEARCH (attached R1–R5; distilled overview in 06-research-distilled-bundle.md)

- R1 — demand/GTM
- R2 — technical trust spine (in-toto/SLSA/Sigstore, CaMeL/FIDES/RTBAS, coverage/OTel off-agent evidence)
- R3 — measurement / safety-delta benchmark methodology
- R4 — competitive landscape
- R5 — human-as-verifier role + reviewer-cockpit evidence

## YOUR TASK — deliver, specific and unhedged, citing R#/F#/G#:

1. THE SINGLE HIGHEST-LEVERAGE MOVE for ELAI given A + the attachments, and why (one decisive paragraph).
2. PRIORITIZED SEQUENCE: rank ELAI's existing F/G pieces by leverage-to-effort for becoming the credible independent trust layer. Name what to ship FIRST, what to DEFER, what to CUT.
3. GAPS the research reveals that ELAI's plans do NOT cover — name the specific missing predicate / artifact / metric.
4. REDUNDANCY / OVER-ENGINEERING: where is ELAI building more than the market will reward?
5. THE 3 BIGGEST RISKS that kill this thesis, and the cheapest falsifying test for each.
6. A concrete CONTRARIAN take: one thing in ELAI's plan OR in the research consensus that you think is WRONG.

---

## FILES TO ATTACH (all in this folder)

1. `R1-demand-gtm.md`
2. `R2-technical-trust-spine.md`
3. `R3-measurement-safetydelta.md`
4. `R4-competitive-landscape.md`
5. `R5-human-as-verifier.md`
6. `ELAI-F-plan-summary.md`
7. `ELAI-G-plan-summary.md`
8. `ELAI-H-plan-summary.md`

(Optional 9th if uploads work: `06-research-distilled-bundle.md` — a condensed overview of all five reports.)
