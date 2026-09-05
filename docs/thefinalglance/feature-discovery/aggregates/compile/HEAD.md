# ELAI exhaustive super feature list

This document is the compiled, source-traceable super feature list for ELAI. It is derived from **76 deep chunk ledgers** over **635 manifest-pinned sources**, rolled into **17 family aggregates** (PHASE, B–Q), then into section files under `aggregates/compile/`, with **3 cross-family passes** (interaction map, shared invariants, status conflicts). Generated **2026-08-12**. Manifest SHA-256: `4aafb23fa4ec917813cc3b629f13cea84049194f4a3b91d340a545826ac2aa93`.

**L-5 pin provenance (single recovery case):** the live L-plan L-5 file was modified after the manifest freeze (now **272 lines**, sha256 `7d13e7a1d9182070e7bb460e29e0d5420f63a5d84fbd34c4c74f0f65f1142005`). The exact manifest-pinned image (**103 lines**, sha256 `b254e847133d4d7cbb095f2f49dfa07d8cd73a8d0b3aff23238c125c595ea14f`) was recovered bit-exact from pre-drift session transcripts, hash-verified against the manifest row, stored at `feature-discovery/_recovered/L-5_counterfactual_do_test_on_model_claims.PINNED-b254e847.md`, and DEEP-L-003 was extracted from that pinned image. The newer live file was NOT extracted. All 635 assignments therefore trace to their exact manifest-pinned fingerprints.

Body sections are assembled mechanically from `SECTION-*.md` files in this directory. This HEAD file supplies title, provenance, reading rules, and the systems table of contents. Synergy, shared invariants, and status-conflict registers live in `SYNERGY.md`.

---

## Status legend

Statuses are **plan-recorded** unless an auditor independently verified live code. Do not treat a plan banner as a runtime proof of shipment.

| Status word | Meaning in this list |
|---|---|
| **shipped** / **DONE** / **WIRED** | Plan or deep ledger asserts the unit (or named seam) is implemented and wired. Prefer deep measurement banners and production-caller notes over summary inventory. |
| **planned** / **PLANNED** / **BUILD** | Design is accepted; implementation is still open or only partially landed. |
| **superseded** | An older claim lost under NEWER-PLAN PRECEDENCE. The losing text is retained as a **Superseded:** note, never deleted. |
| **mixed** / **partial** / **PARTIAL** | Some legs shipped or host-wired; others deferred, dark, or research-blocked. Read the Status line and Superseded notes together. |
| **REJECTED-by-measurement** / **VALID REFUTED** / **VETO** | Explicit disposition: do not ship or reopen as if still live. Design text may remain as non-ship history. |
| **wire-dark** / **DARK** | Path exists or is designed but is not live default-on protection (or is gated dark). Honesty rule: do not advertise fail-closed live guarantees for dark paths. |
| **DEFERRED** / **OUT-OF-K** / **BENCH-PENDING** | Explicitly postponed or owned elsewhere. |

**Mixed statuses:** When one unit shows both SHIPPED and PLANNED (e.g. Q-14 header PLANNED with E-1 SHIPPED, or D-28 shipping-pass host vs training deferred), **per-sub-unit / per-leg banners control**. A coarse file header must not erase a more specific measurement banner. See `SYNERGY.md` status conflicts (CF-001, CF-015–017, CF-019).

---

## Newer-plan precedence

When two sources disagree about target state for the same feature or interaction:

1. An explicit **SUPERSEDED** / replacement / **CORRECT** / **REJECTED** / **SHIPPED** (measurement) / **VETO** / **VALID REFUTED** marker in a source beats everything.
2. Else later **letter family** wins (B oldest → Q newest). PHASE is foundation substrate, not a letter-order peer.
3. Within a family, higher plan number is newer.
4. Deep ledgers supersede summary orientation where they conflict (every family contract states this).
5. **Losing claims stay recorded** as superseded notes with anchors. Never delete losers.

Cross-family conflict resolutions: `SYNERGY.md` § Status conflicts (CF-001..CF-043). Residual cases precedence cannot close: RA-001..RA-010 (UNRESOLVED).

---

## Systems table of contents

One row per system. Unit/entry counts come from each section’s audit block. Behavior bullets are atom re-emissions with IDs. Surfaces are user-facing surface atoms.

| System | One-line name | Units / entries | Behavior atoms | Surfaces | Section file |
|---|---|---:|---:|---:|---|
| **PHASE** | Implementation-guide phase steps (walking skeleton through empire integration) | 44 | 1677 | 243 | `SECTION-PHASE-D.md` |
| **B** | Dashboard V2 harness wiring, producers, knowledge/runtime surfaces, and B-plan delivery | 49 | 2587 | 672 | `SECTION-B.md` |
| **C** | Verification gap, testing trust, code completeness, and paper-grounded extensions (C-0..C-59) | 60 | 3205 | 366 | `SECTION-C.md` |
| **D** | Typed self-improvement, inference-time grounding, benchmarks, dashboards, deferred research | 31 | 1215 | 88 | `SECTION-PHASE-D.md` |
| **E** | Deep feature discovery sweep (UX/memory, orchestration, codegen, validation/RAG, inference, observability, vision/security/economics, CI/skills, planning/RL) | 56 plan units (58 entry sections) | 3620 | 2023 | `SECTION-E.md` |
| **F** | VibeSec/SHARD wiring-enforcement sessions (F-0..F-13, F-2c) and wiring doctrine | 18 | (F+G total 2627) | (F+G total 524) | `SECTION-F-G.md` |
| **G** | Trust READ/EVIDENCE/RUNTIME surfaces (taint, RRA, blast radius, MCP re-pin, cockpit, SAFEDELTA, confidence, attestation, redaction, model-drift) | 12 | (F+G total 2627) | (F+G total 524) | `SECTION-F-G.md` |
| **H** | Efficiency, canvas-workbench, trust-hardening, UX/memory, grounded reasoning, DGM verification, reversibility, context lifecycle, review anchoring, external steals | 71 | 3826 | 721 | `SECTION-H.md` |
| **I** | Feature engineering over wiring bedrock (Tracks 12–20: fault isolation, evidence-ranked search, cache economics, deterministic gates, routing/budgets, open-weight flywheel, deterministic-over-prompt, design critique/trust, competitor-steal ports) | 62 (55 plan units I-0..I-54 + specials) | 3829 | 445 | `SECTION-I.md` |
| **J** | Workflow harness redesign (perception, MCP control plane, retrieval, determinism, doctrine, canvas, voice, journal integrity, eval, security, multi-seat) | 113 (J-0..J-112) | 5208 | 1038 | `SECTION-J.md` |
| **K** | Sequencing, composition, proof, distribution, affordability, packaging, discipline (plus net-new K-23 workspace and K-24 billing) | 33 | (K+L total 3532) | (K+L total 762) | `SECTION-K-L.md` |
| **L** | Causal Grounding Gate + Output Quality/Familiarity + Context Core + Measured Competitor/Eval + DeepReason/Exploratory Reasoning (L-0..L-24) | 28 | (K+L total 3532) | (K+L total 762) | `SECTION-K-L.md` |
| **M** | Commercial shipping / multi-seat (entitlement, org policy, fleet evidence, release channel, packaging, EEF) | 10 | 1000 | 153 | `SECTION-M-N-O.md` |
| **N** | INTERLACE / UNIFICATION (measurement spine, composition, promotion, claim/receipt, arbitration, provenance, render, governance, counterfactual, journal-fold, consistency, OutputBoundary, edit-loop, framing stability, closure) | 19 | 1388 | 166 | `SECTION-M-N-O.md` |
| **O** | PERSONAL RESEARCH MACHINE (research daemon, TaskSpec scheduler, brain vault, canvas WORKFLOW) | 7 | 982 | 239 | `SECTION-M-N-O.md` |
| **P** | Personal-software incarnation + command-center + computer-use-steal + openclaw-steal + creativity falsifier (P-0..P-25) | 26 | 761 | 240 | `SECTION-P-Q.md` |
| **Q** | Trust-spine durability + routing-economics + distribution + research-conditioned-creativity + AIDE² instrument economics + autoresearch synergy + e2e functional suite (Q-0..Q-16) | 18 | 477 | 159 | `SECTION-P-Q.md` |

**Combined section audits (for assembly math):** PHASE+D entries 75; F+G entries 30; K+L entries 61; M+N+O entries 36; P+Q entries 44. Cross-family map: 881 structural edges, 115 hub systems (≥3 consumer families), 426 distinct provider feature IDs. Shared invariants: 34 (INV-A01..G02). Status conflicts: CF-001..043 (43 top-level); residual ambiguities: RA-001..010 (10 UNRESOLVED).

How systems compose: see `SYNERGY.md`.

---

## How to read an entry

Each plan unit in the body sections uses a fixed shape:

### Unit heading
`### <Unit-ID>: <short name>` (e.g. `### B-37: RuntimeAdapter multi-backend substrate`).

### Status
Plan-recorded disposition of the unit or its legs. May be mixed (shipped + deferred + wire-dark on different seams). Prefer deep banners and Superseded notes over a single word in a summary table.

### Anchors
Atom IDs and source line anchors that ground the entry. Form examples: `B0-025` @ DEEP-B-001…; `Q-001/Q14-E002` @ Q-14:L4; `SUMMARY/SUM-0389` @ B-AGGREGATE. Cross-family edges also cite aggregate lines (`G-AGGREGATE.md:L1236`).

### Composes with
Cross-unit and cross-family dependencies: consumes, depends, extends, gates, closes, supersedes, reuses. Prefer IX-* atoms; DEP/INT/WIRING atoms also appear. For hub fan-in and multi-hop chains, use `SYNERGY.md` first, then drill into the unit.

### Behaviors
Bulleted atomic behaviors, each carrying its **atom ID** in backticks or parentheses. Every source atom from the family aggregate unit section is re-emitted (zero-omission compile policy). Surface inventories and register buckets may appear after unit entries.

### Superseded
When present: **winner** claim, **loser** (retained) claim, precedence rule, and anchors. Losers are history for implement recon, not deleted inventory.

### Status conflicts
If a unit is named in CF-001..043 or RA-001..010, apply the RESOLUTION in `SYNERGY.md` before treating summary text as target state. Residual RA rows mean stop and recon; do not paper over with letter order alone.
