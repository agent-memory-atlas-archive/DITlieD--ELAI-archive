# E-plan exhaustive feature discovery — E-SUMMARY_INDEX

## Coverage

- **Primary source read completely:** docs/thefinalglance/E-plan/E-SUMMARY_INDEX.md, all 615 lines, including premise, all 55 per-session records, closure map, hard rules, coverage totals, schema-version note, and ship-sequence sketch.
- **Orientation only:** docs/thefinalglance/ELAI-SPEC-OVERVIEW.md, especially the system thesis/core architecture and E overview. No feature below is promoted from the overview if it is absent from the E summary.
- **Source semantics:** The source describes designed capabilities, not verified implementation. “SHIPPED”, “DONE”, “PLANNED”, “DEFERRED”, and “POSITIONING” are retained exactly where the summary states them.
- **Traceability convention:** Every item in the exhaustive inventory is **[EXPLICIT]** and cites the exact summary line range for its session. Anything synthesized from multiple entries is isolated later as **[INFERRED]**.
- **Atomic extraction total:** 753 uniquely keyed explicit feature/behavior/artifact/event/smoke atoms (E-1.F01 through the per-session final keys), with no duplicate keys or numbering gaps.
- **Corpus accounting:** 55/55 E session summaries covered; 11/11 summary waves covered; 55/55 corresponding E-N originals found on disk; E-0_INDEX.md also found. The summary says it was synthesized from 56 source files (E-0_INDEX + 55 E-N files) and reports 57 authored files when this summary is included (lines 1-4, 548-555).
- **Global proof contract:** Each E-N is intended to end in an observable-effect smoke chain of command/action → SSB row → artifact (lines 8-12). Exceptions explicitly called out by the summary are E-23 (no new SSB) and E-47 (no SSB).

## Exhaustive atomic inventory

### E-1 — Infinite-Context Chat Illusion

**Anchor:** E-SUMMARY_INDEX.md lines 30-36. **Closes:** 01-infinite-context-chat-illusion. **Prerequisites/status:** E-5 memory_facts substrate and B-37 SHIPPED; mostly Wave-1+-parallel-safe but conflicts with E-5 schema.

- **E-1.F01 [EXPLICIT]** Add an A-MEM Zettelkasten memory layer.
- **E-1.F02 [EXPLICIT]** Add MemGPT Core memory.
- **E-1.F03 [EXPLICIT]** Add MemGPT Archival memory.
- **E-1.F04 [EXPLICIT]** Add a sleep-time digest over E-5’s memory_facts table.
- **E-1.F05 [EXPLICIT]** Read voice_style facts into a unified response preamble.
- **E-1.F06 [EXPLICIT]** Gate memory promotion with BeliefShift/CTIM-Rover risk, subject to the original plan’s Open questions.
- **E-1.F07 [EXPLICIT]** Add the new crates/memory/ surface.
- **E-1.F08 [EXPLICIT]** Journal memory_promoted.
- **E-1.F09 [EXPLICIT]** Journal memory_evicted.
- **E-1.F10 [EXPLICIT]** After five turns, emit a digest.
- **E-1.F11 [EXPLICIT]** On turn six, retrieve/cite an Archival fact with a [memory:K] tag.

### E-2 — Local LLM Speed Tricks

**Anchor:** lines 38-44. **Closes:** 02-local-llm-speed-tricks. **Prerequisite/status:** B-14 DONE dashboard-v2 streaming surface; parallel-safe with E-3, E-4, and Wave 2+.

- **E-2.F01 [EXPLICIT]** Frame-pace SSE rendering in CommandWindow with smoothStream-style behavior.
- **E-2.F02 [EXPLICIT]** Show a skeleton response bubble while generation begins.
- **E-2.F03 [EXPLICIT]** Show a cursor sentinel during streamed generation.
- **E-2.F04 [EXPLICIT]** Add prompt-lookup decoding to the inference sidecar.
- **E-2.F05 [EXPLICIT]** Configure Hugging Face prompt_lookup_num_tokens=10.
- **E-2.F06 [EXPLICIT]** Preserve the DoLa/ETC LogitsProcessor chain when prompt lookup is active.
- **E-2.F07 [EXPLICIT]** Change CommandWindow.svelte and python/elai_inference/server.py.
- **E-2.F08 [EXPLICIT]** Journal stream_smoothed.
- **E-2.F09 [EXPLICIT]** Render chat replies at a steady 40–60 characters/second in smoke verification.
- **E-2.F10 [EXPLICIT]** Demonstrate 2–4× sidecar throughput on input-grounded code-edit prompts.

### E-3 — Model Handoff UX

**Anchor:** lines 46-52. **Closes:** 03-model-handoff-ux. **Prerequisite/status:** B-37 SHIPPED PerRoleRuntime; parallel-safe with E-2 and E-4.

- **E-3.F01 [EXPLICIT]** Show the agent ROLE in chat by default.
- **E-3.F02 [EXPLICIT]** Hide the underlying MODEL in chat by default.
- **E-3.F03 [EXPLICIT]** Expose model identity in a drawer for power users.
- **E-3.F04 [EXPLICIT]** Read E-5 voice_style facts into the preamble.
- **E-3.F05 [EXPLICIT]** Document a CUDA-graph warm pool.
- **E-3.F06 [EXPLICIT]** Defer that warm pool on Hopper.
- **E-3.F07 [EXPLICIT]** Extend SettingsWindow and ResponseCardPayload.
- **E-3.F08 [EXPLICIT]** Journal role_shown_model_hidden.
- **E-3.F09 [EXPLICIT]** Display a “Designer” role badge without a model name in smoke verification.
- **E-3.F10 [EXPLICIT]** Reveal the model when the Settings toggle is enabled.

### E-4 — Chat UX Critique

**Anchor:** lines 54-60. **Closes:** 04-chat-ux-critique. **Prerequisite/status:** E-5 expertise facts; parallel-safe with E-3.

- **E-4.F01 [EXPLICIT]** Detect sycophancy.
- **E-4.F02 [EXPLICIT]** Alert on persona drift.
- **E-4.F03 [EXPLICIT]** Count repair resistance.
- **E-4.F04 [EXPLICIT]** Port 14 CommandWindow UX rules from the original report; the summary does not enumerate those rules.
- **E-4.F05 [EXPLICIT]** Calibrate response terseness from E-5 fact_type=expertise.
- **E-4.F06 [EXPLICIT]** Add the dashboard intent surface at crates/dashboard-v2/src/lib/intent/.
- **E-4.F07 [EXPLICIT]** Journal sycophancy_detected.
- **E-4.F08 [EXPLICIT]** Journal persona_drift_alert.
- **E-4.F09 [EXPLICIT]** When a user says “you’re wrong”, do not flip a high-confidence answer solely to agree.
- **E-4.F10 [EXPLICIT]** Detect drift across ten turns and show an alert chip in StatusBar.

### E-5 — User Modeling & Memory

**Anchor:** lines 62-68. **Closes:** 05-user-modeling-and-memory. **Prerequisite/status:** foundational with no prerequisite; E-1/E-3/E-4 depend on it; parallel-safe only with E-2.

- **E-5.F01 [EXPLICIT]** Dispatch memory operations using Mem0 ADD.
- **E-5.F02 [EXPLICIT]** Dispatch memory operations using Mem0 UPDATE.
- **E-5.F03 [EXPLICIT]** Dispatch memory operations using Mem0 DELETE.
- **E-5.F04 [EXPLICIT]** Dispatch memory operations using Mem0 NOOP.
- **E-5.F05 [EXPLICIT]** Add Zep bi-temporal scaffolding.
- **E-5.F06 [EXPLICIT]** Defend memory ingestion against poisoning with MINJA.
- **E-5.F07 [EXPLICIT]** Assign memory facts a trust score.
- **E-5.F08 [EXPLICIT]** Apply temporal decay to memory facts.
- **E-5.F09 [EXPLICIT]** Persist facts in a SQLite memory_facts table.
- **E-5.F10 [EXPLICIT]** Add a new approximately 600-LOC crates/memory/ Rust crate.
- **E-5.F11 [EXPLICIT]** Journal memory_fact_added.
- **E-5.F12 [EXPLICIT]** Journal memory_fact_updated.
- **E-5.F13 [EXPLICIT]** Journal memory_fact_deleted.
- **E-5.F14 [EXPLICIT]** Journal memory_fact_noop.
- **E-5.F15 [EXPLICIT]** Convert “I prefer terse responses” into a voice_style memory fact.
- **E-5.F16 [EXPLICIT]** Read that fact on a later turn.
- **E-5.F17 [EXPLICIT]** Block an adversarial memory injection at the MINJA gate.

### E-6 — Multi-Agent Failure Modes

**Anchor:** lines 70-76. **Closes:** 06-multi-agent-failure-modes. **Prerequisite/status:** D-13 Adversary PLANNED plus a new E-6 iso-budget harness; parallel-safe with E-7/E-8/E-9/E-10.

- **E-6.F01 [EXPLICIT]** Classify multi-agent failures with the 14-mode MAST classifier.
- **E-6.F02 [EXPLICIT]** Require an Adversary from a different model family.
- **E-6.F03 [EXPLICIT]** Audit each agent role under an iso-budget comparison.
- **E-6.F04 [EXPLICIT]** Delete a role that does not improve results by more than one standard deviation.
- **E-6.F05 [EXPLICIT]** Supersede/extend D-13’s same-family blind spot.
- **E-6.F06 [EXPLICIT]** Add crates/orchestrator/src/mast.rs.
- **E-6.F07 [EXPLICIT]** Journal mast_mode_classified.
- **E-6.F08 [EXPLICIT]** Journal iso_budget_run.
- **E-6.F09 [EXPLICIT]** Journal adversary_disagreement.
- **E-6.F10 [EXPLICIT]** Make a synthetic same-family executor/auditor trace produce adversary_disagreement under cross-family review.
- **E-6.F11 [EXPLICIT]** Demonstrate an iso-budget run pruning one role without loss.

### E-7 — Darwinian Self-Improvement

**Anchor:** lines 78-84. **Closes:** 07-darwinian-self-improvement. **Prerequisite/status:** D-5 BAI, D-8 archive, and D-9 L3 frozen, all PLANNED; parallel-safe with E-6 and E-8.

- **E-7.F01 [EXPLICIT]** Guard self-improvement with a bounded-capacity Two-Gate.
- **E-7.F02 [EXPLICIT]** Maintain a MAP-Elites archive.
- **E-7.F03 [EXPLICIT]** Quarantine candidates with EST.
- **E-7.F04 [EXPLICIT]** Bind the improvement lineage with an Ed25519 chain.
- **E-7.F05 [EXPLICIT]** Add undeferred trap fixtures.
- **E-7.F06 [EXPLICIT]** Close the Sakana DGM reward-hacking incident gap.
- **E-7.F07 [EXPLICIT]** Add two_gate.rs and map_elites.rs.
- **E-7.F08 [EXPLICIT]** Journal two_gate_blocked.
- **E-7.F09 [EXPLICIT]** Journal map_elites_promoted.
- **E-7.F10 [EXPLICIT]** Journal est_quarantined.
- **E-7.F11 [EXPLICIT]** Block and quarantine a synthetic reward-hacking variant.
- **E-7.F12 [EXPLICIT]** Promote a clean variant.

### E-8 — Self-Evolving Prompts

**Anchor:** lines 86-92. **Closes:** 08-self-evolving-prompts. **Prerequisite/status:** D-1 ReasoningTemplate PLANNED and an available cloud Sonnet route; parallel-safe with E-6/E-7.

- **E-8.F01 [EXPLICIT]** Admit GEPA-generated templates through the SHADOW lifecycle.
- **E-8.F02 [EXPLICIT]** Use cloud Sonnet as the prompt optimizer.
- **E-8.F03 [EXPLICIT]** Use local Qwen as the optimized target.
- **E-8.F04 [EXPLICIT]** Do not rely on a small local LM to self-optimize.
- **E-8.F05 [EXPLICIT]** Run the optimizer as an offline Python pipeline.
- **E-8.F06 [EXPLICIT]** Hash generated templates with blake3.
- **E-8.F07 [EXPLICIT]** Ship accepted templates into ELAI’s archive.
- **E-8.F08 [EXPLICIT]** Add workflow/optimizer/.
- **E-8.F09 [EXPLICIT]** Journal prompt_optimizer_proposed.
- **E-8.F10 [EXPLICIT]** Journal prompt_optimizer_shadow.
- **E-8.F11 [EXPLICIT]** Journal prompt_optimizer_promoted.
- **E-8.F12 [EXPLICIT]** Require a 24-hour offline run to yield at least one GEPA-promoted template in smoke verification.
- **E-8.F13 [EXPLICIT]** Use the promoted template on a live cloud /task.

### E-9 — Agent Eval Benchmarks

**Anchor:** lines 94-100. **Closes:** 09-agent-eval-benchmarks. **Prerequisite/status:** C-51 Aider polyglot PLANNED; parallel-safe with E-6/E-7/E-49.

- **E-9.F01 [EXPLICIT]** Stop reporting SWE-bench Verified.
- **E-9.F02 [EXPLICIT]** Adopt SWE-Live.
- **E-9.F03 [EXPLICIT]** Adopt SWE-Pro.
- **E-9.F04 [EXPLICIT]** Adopt tau2-bench.
- **E-9.F05 [EXPLICIT]** Add a private held-out evaluation set.
- **E-9.F06 [EXPLICIT]** Compute paired-bootstrap confidence intervals.
- **E-9.F07 [EXPLICIT]** Record a harness-vs-model attribution tuple.
- **E-9.F08 [EXPLICIT]** Extend crates/cli/src/benchmark.rs.
- **E-9.F09 [EXPLICIT]** Journal bench_swelive_run.
- **E-9.F10 [EXPLICIT]** Journal bench_tau2_run.
- **E-9.F11 [EXPLICIT]** Journal harness_attribution_split.
- **E-9.F12 [EXPLICIT]** Support the CLI command “elai benchmark --suite swelive”.
- **E-9.F13 [EXPLICIT]** Write the paired-bootstrap interval and attribution tuple to phase_*.toml.

### E-10 — Critic of ELAI Design

**Anchor:** lines 102-108. **Closes:** 10-critic-of-elai-design. **Prerequisite/status:** E-6 iso-budget harness; parallel-safe with E-7 and E-8.

- **E-10.F01 [EXPLICIT]** Convert 14 hostile-critic punches into five concrete FSM/role ablations; exact members require the original.
- **E-10.F02 [EXPLICIT]** Preserve five conceded strengths; exact members require the original.
- **E-10.F03 [EXPLICIT]** Use E-6’s iso-budget delta test as the role-deletion gate.
- **E-10.F04 [EXPLICIT]** Add the workflow/reasoning/ablations/ documentation and harness.
- **E-10.F05 [EXPLICIT]** Journal ablation_pruned_role.
- **E-10.F06 [EXPLICIT]** When a role’s measured delta is below one standard deviation, emit ablation_pruned_role.
- **E-10.F07 [EXPLICIT]** Remove that role from agent_pool in the resulting PR.

### E-11 — Idea-to-SaaS Prior Art

**Anchor:** lines 110-116. **Closes:** 11-idea-to-saas-prior-art; extends B-40 SHIPPED. **Prerequisite/status:** B-40 and B-41 SHIPPED; parallel-safe with E-12/E-50.

- **E-11.F01 [EXPLICIT]** Score scope clarity in IdeaCritic.
- **E-11.F02 [EXPLICIT]** Add a ProductionGrade auditor mode as a B-41 mode extension.
- **E-11.F03 [EXPLICIT]** Verify package identity with PackageVerifier to block slopsquatting.
- **E-11.F04 [EXPLICIT]** Run a critic at each level.
- **E-11.F05 [EXPLICIT]** Enforce ANTI_SPIRAL_HARD_ABORT.
- **E-11.F06 [EXPLICIT]** Produce Aider-compatible unified diffs.
- **E-11.F07 [EXPLICIT]** Extend crates/orchestrator/src/idea_framing.rs.
- **E-11.F08 [EXPLICIT]** Journal scope_score_below_threshold.
- **E-11.F09 [EXPLICIT]** Journal slopsquat_block.
- **E-11.F10 [EXPLICIT]** Journal production_grade_block.
- **E-11.F11 [EXPLICIT]** Let a sufficiently clear calculator idea pass the scope threshold.
- **E-11.F12 [EXPLICIT]** Block a vague idea.
- **E-11.F13 [EXPLICIT]** Block the suspicious npm package “react-icns”.

### E-12 — Production Fullstack Codegen

**Anchor:** lines 118-124. **Closes:** 12-production-fullstack-codegen. **Prerequisite/status:** E-11, B-30 PLANNED, B-37 SHIPPED; parallel-safe with E-50.

- **E-12.F01 [EXPLICIT]** Provide seven framework-convention manifests; names require the original.
- **E-12.F02 [EXPLICIT]** Define a FrameworkAdapter trait.
- **E-12.F03 [EXPLICIT]** Run a 15-item production_checklist; the summary names only part of it.
- **E-12.F04 [EXPLICIT]** Include gitleaks in the production checklist.
- **E-12.F05 [EXPLICIT]** Include OWASP Semgrep in the production checklist.
- **E-12.F06 [EXPLICIT]** Include an RLS probe in the production checklist.
- **E-12.F07 [EXPLICIT]** Include axe/lighthouse in the production checklist.
- **E-12.F08 [EXPLICIT]** Include OTel seed generation in the production checklist.
- **E-12.F09 [EXPLICIT]** Include Biome in the deny-quality runner.
- **E-12.F10 [EXPLICIT]** Include Ruff in the deny-quality runner.
- **E-12.F11 [EXPLICIT]** Include Clippy deny in the deny-quality runner.
- **E-12.F12 [EXPLICIT]** Block assertion-only diffs.
- **E-12.F13 [EXPLICIT]** Add crates/orchestrator/src/production_grade/.
- **E-12.F14 [EXPLICIT]** Journal production_check_failed per checker.
- **E-12.F15 [EXPLICIT]** Journal rls_probe_missing.
- **E-12.F16 [EXPLICIT]** Journal biome_deny_violation.
- **E-12.F17 [EXPLICIT]** Require all 15 checks to pass before COMMITTING a greenfield SaaS.
- **E-12.F18 [EXPLICIT]** Block a deliberate missing-RLS variant.

### E-13 — TDD Verification Agents

**Anchor:** lines 126-132. **Closes:** 13-tdd-verification-agents; augments C-28/C-29. **Prerequisite/status:** C-28 mutation and C-29 IntentPair PLANNED; parallel-safe with E-12/E-14.

- **E-13.F01 [EXPLICIT]** Gate changes on properties.
- **E-13.F02 [EXPLICIT]** Run targeted cargo-mutants checks.
- **E-13.F03 [EXPLICIT]** Cache targeted mutant survivals.
- **E-13.F04 [EXPLICIT]** Detect over-mocking after tool use.
- **E-13.F05 [EXPLICIT]** Add a mutant-hunter agent.
- **E-13.F06 [EXPLICIT]** Gate at Fuzz4All CONCEPT level.
- **E-13.F07 [EXPLICIT]** Require at least one non-LLM verifier.
- **E-13.F08 [EXPLICIT]** Keep mutation checks targeted rather than workspace-wide.
- **E-13.F09 [EXPLICIT]** Extend crates/orchestrator/src/mutation_runner.rs.
- **E-13.F10 [EXPLICIT]** Journal mutant_survived.
- **E-13.F11 [EXPLICIT]** Journal anti_over_mock_block.
- **E-13.F12 [EXPLICIT]** Journal fuzz4all_finding.
- **E-13.F13 [EXPLICIT]** Block a PR containing an over-mocked test.
- **E-13.F14 [EXPLICIT]** Make a clean PR pass its targeted mutant run.

### E-14 — Browser E2E Agents

**Anchor:** lines 134-140. **Closes:** 14-browser-e2e-agents; extends B-24 and B-42 SHIPPED. **Prerequisite/status:** B-24/B-42 and E-15’s sandbox profile for npx playwright; parallel-safe with E-12/E-15.

- **E-14.F01 [EXPLICIT]** Use AXTree-first Playwright validation.
- **E-14.F02 [EXPLICIT]** Persist Playwright snapshots to disk.
- **E-14.F03 [EXPLICIT]** Gate accessibility with axe-core.
- **E-14.F04 [EXPLICIT]** Use constrained decoding for Tier-2 VLM validation.
- **E-14.F05 [EXPLICIT]** Use an SSIM-first hybrid visual diff.
- **E-14.F06 [EXPLICIT]** Apply an asymmetric veto.
- **E-14.F07 [EXPLICIT]** Add a Cosmos DeployPreviewWorker.
- **E-14.F08 [EXPLICIT]** Exercise a multi-context SaaS journey.
- **E-14.F09 [EXPLICIT]** Detect Cloudflare/Turnstile.
- **E-14.F10 [EXPLICIT]** Skip protected flows when detected and never bypass the protection.
- **E-14.F11 [EXPLICIT]** Add crates/orchestrator/src/browser_e2e/.
- **E-14.F12 [EXPLICIT]** Journal axtree_snapshot.
- **E-14.F13 [EXPLICIT]** Journal axe_violation.
- **E-14.F14 [EXPLICIT]** Journal deploy_preview_ready.
- **E-14.F15 [EXPLICIT]** Commit the AXTree snapshot diff for a greenfield build.
- **E-14.F16 [EXPLICIT]** Make the deploy preview ready.
- **E-14.F17 [EXPLICIT]** Block an intentional accessibility defect.

### E-15 — Sandboxed Exec

**Anchor:** lines 142-148. **Closes:** 15-sandboxed-exec; extends B-18/C-11 PLANNED. **Prerequisite/status:** B-18/C-11 PLANNED; E-14 consumes it and E-22 is a sandbox-pool sibling.

- **E-15.F01 [EXPLICIT]** Maintain a Firecracker hot snapshot-restore pool.
- **E-15.F02 [EXPLICIT]** Argument-check seccomp socket() calls.
- **E-15.F03 [EXPLICIT]** Permit socket address families AF_INET and AF_INET6 only.
- **E-15.F04 [EXPLICIT]** Apply sandbox hardening in NO_NEW_PRIVS → CAPBSET_DROP → seccomp order.
- **E-15.F05 [EXPLICIT]** Remove the chroot code path.
- **E-15.F06 [EXPLICIT]** Fall back to gVisor where KVM is unavailable.
- **E-15.F07 [EXPLICIT]** Give researcher networking a veth+iptables allowlist.
- **E-15.F08 [EXPLICIT]** Fork each baby step from a snapshot.
- **E-15.F09 [EXPLICIT]** Require the AppContainer Windows trio; exact trio members require the original.
- **E-15.F10 [EXPLICIT]** Extend crates/orchestrator/src/sandbox/.
- **E-15.F11 [EXPLICIT]** Journal firecracker_snapshot_restored.
- **E-15.F12 [EXPLICIT]** Journal seccomp_arg_blocked.
- **E-15.F13 [EXPLICIT]** Restore/fork a novel baby step in 28 ms in smoke verification.
- **E-15.F14 [EXPLICIT]** Block the CVE-2025-31133 proof of concept.

### E-16 — VLM Image Validation

**Anchor:** lines 150-156. **Closes:** 16-vlm-image-validation; a delta on B-24 SHIPPED. **Prerequisite/status:** B-24 SHIPPED; parallel-safe with E-14.

- **E-16.F01 [EXPLICIT]** Add AXTree validation to the existing media validator.
- **E-16.F02 [EXPLICIT]** Add anti-sycophancy validation.
- **E-16.F03 [EXPLICIT]** Add RCE convergence.
- **E-16.F04 [EXPLICIT]** Preserve the existing tier1 implementation.
- **E-16.F05 [EXPLICIT]** Preserve the existing feedback_loop implementation.
- **E-16.F06 [EXPLICIT]** Act as the intra-modal sibling of D-13’s cross-family Adversary.
- **E-16.F07 [EXPLICIT]** Extend crates/orchestrator/src/media_validator/.
- **E-16.F08 [EXPLICIT]** Journal rce_convergence_step.
- **E-16.F09 [EXPLICIT]** Journal intra_modal_disagreement.
- **E-16.F10 [EXPLICIT]** Produce intra_modal_disagreement for a deliberate-fail screenshot.
- **E-16.F11 [EXPLICIT]** Preserve an honest VLM verdict.

### E-17 — RAG Retrieval Quality ★ THE BIG ONE

**Anchor:** lines 158-164. **Closes:** 17-rag-retrieval-quality and the B-4 degraded-runtime gap. **Prerequisite/status:** B-1/B-4 SHIPPED; parallel-safe with most of E-plan; named the highest-leverage single fix in the sweep.

- **E-17.F01 [EXPLICIT]** Stage a GraphCodeBERT ONNX model under .elai_cc/models/onnx/.
- **E-17.F02 [EXPLICIT]** Stage a Granite ONNX model under .elai_cc/models/onnx/.
- **E-17.F03 [EXPLICIT]** Stage a cross-encoder ONNX model under .elai_cc/models/onnx/.
- **E-17.F04 [EXPLICIT]** Activate the code_vec query path that is currently opt-in through ELAI_CODE_VEC=1.
- **E-17.F05 [EXPLICIT]** Add Anthropic Contextual Retrieval.
- **E-17.F06 [EXPLICIT]** Add Jina-rerank-v3.
- **E-17.F07 [EXPLICIT]** Calibrate Found-in-the-Middle for an expected 15 percentage-point lift.
- **E-17.F08 [EXPLICIT]** Correct the DEEP MRR < STANDARD MRR monotonicity violation.
- **E-17.F09 [EXPLICIT]** Add scripts/setup_onnx.sh.
- **E-17.F10 [EXPLICIT]** Update knowledge-engine configuration.
- **E-17.F11 [EXPLICIT]** Journal onnx_model_staged.
- **E-17.F12 [EXPLICIT]** Journal rerank_active.
- **E-17.F13 [EXPLICIT]** Journal fitm_calibrated.
- **E-17.F14 [EXPLICIT]** Verify staged models with blake3.
- **E-17.F15 [EXPLICIT]** Demonstrate DEEP MRR > STANDARD MRR in phase_3K_rag.toml.

### E-18 — Tool-Use Reliability

**Anchor:** lines 166-172. **Closes:** 18-tool-use-reliability. **Prerequisite/status:** must sequence after E-35 XGrammar; also depends on B-7 SHIPPED via B-7b; parallel-safe with E-17/E-19/E-20.

- **E-18.F01 [EXPLICIT]** Enforce Tier-2 XGrammar constraints at tool-call sites.
- **E-18.F02 [EXPLICIT]** Fence SEARCH/REPLACE operations outside Git.
- **E-18.F03 [EXPLICIT]** Wrap tool calls in a Hermes tool envelope.
- **E-18.F04 [EXPLICIT]** Apply a path-allowlist mask.
- **E-18.F05 [EXPLICIT]** Maintain a reliability counter per tool.
- **E-18.F06 [EXPLICIT]** Reuse E-35’s grammar substrate rather than recreate it.
- **E-18.F07 [EXPLICIT]** Extend crates/tools/src/grammar.rs.
- **E-18.F08 [EXPLICIT]** Apply the XGrammar mask in python/elai_inference/server.py.
- **E-18.F09 [EXPLICIT]** Journal xgrammar_constraint_enforced.
- **E-18.F10 [EXPLICIT]** Journal search_replace_fenced.
- **E-18.F11 [EXPLICIT]** Reshape malformed tool-call JSON into valid JSON.
- **E-18.F12 [EXPLICIT]** Make a SEARCH/REPLACE patch land on its first attempt.

### E-19 — Reasoning Quality

**Anchor:** lines 174-180. **Closes:** 19-reasoning-quality; extends B-39 SHIPPED. **Prerequisite/status:** B-39 SHIPPED, D-2 PPM PLANNED/future; parallel-safe with E-18/E-20.

- **E-19.F01 [EXPLICIT]** Gate L2 Micro-Reflection behind an L1 confidence check.
- **E-19.F02 [EXPLICIT]** Use self-certainty for that confidence check.
- **E-19.F03 [EXPLICIT]** Score L4 AB-MCTS branches with PRM-lite.
- **E-19.F04 [EXPLICIT]** Treat PRM-lite as a stopgap until D-2 ships.
- **E-19.F05 [EXPLICIT]** Represent Typed-CoT as JSON triples.
- **E-19.F06 [EXPLICIT]** Constrain Typed-CoT through E-18’s XGrammar mask.
- **E-19.F07 [EXPLICIT]** Extend crates/orchestrator/src/reasoning/.
- **E-19.F08 [EXPLICIT]** Extend crates/orchestrator/src/chat_reflect.rs.
- **E-19.F09 [EXPLICIT]** Journal prm_lite_score.
- **E-19.F10 [EXPLICIT]** Journal l2_confidence_gate.
- **E-19.F11 [EXPLICIT]** Skip L2 for an easy task with high L1 confidence.
- **E-19.F12 [EXPLICIT]** Run L2 for a hard task.
- **E-19.F13 [EXPLICIT]** Score AB-MCTS branches with PRM-lite in smoke verification.

### E-20 — Local/Cloud Routing

**Anchor:** lines 182-188. **Closes:** 20-local-cloud-routing; extends B-27 PLANNED, B-37 SHIPPED, and C-5 PLANNED. **Prerequisite/status:** B-37 SHIPPED and B-27 PLANNED; parallel-safe with E-17/E-18/E-19.

- **E-20.F01 [EXPLICIT]** Add xRouter routing policy support.
- **E-20.F02 [EXPLICIT]** Add RouterArena routing policy support.
- **E-20.F03 [EXPLICIT]** Add C3PO routing policy support.
- **E-20.F04 [EXPLICIT]** Enforce data-leakage rules shared conceptually with E-37.
- **E-20.F05 [EXPLICIT]** Mask execution paths according to privacy tier.
- **E-20.F06 [EXPLICIT]** Share E-18’s path-allowlist primitive.
- **E-20.F07 [EXPLICIT]** Keep compute-location routing orthogonal to D-10 verification-mode routing.
- **E-20.F08 [EXPLICIT]** Extend crates/orchestrator/src/routing.rs.
- **E-20.F09 [EXPLICIT]** Journal route_decision.
- **E-20.F10 [EXPLICIT]** Journal data_leakage_blocked.
- **E-20.F11 [EXPLICIT]** Route prompts mentioning an API key locally.
- **E-20.F12 [EXPLICIT]** Require a redaction check before any cloud route.

### E-21 — Peer Systems / LLM-as-Worker

**Anchor:** lines 190-196. **Closes:** 21-peer-systems-llm-as-worker. **Prerequisite/status:** B-22 SHIPPED and an AGENTLESS evidence pull; parallel-safe with E-22/E-23.

- **E-21.F01 [EXPLICIT]** Build a PageRank repository map.
- **E-21.F02 [EXPLICIT]** Run spectrum-based fault localization on failing tests.
- **E-21.F03 [EXPLICIT]** Rerank candidates with a critic.
- **E-21.F04 [EXPLICIT]** Audit ACI command shape.
- **E-21.F05 [EXPLICIT]** Use the repository as a state store.
- **E-21.F06 [EXPLICIT]** Expose multi-provider routing through B-37 PerRoleRuntime.
- **E-21.F07 [EXPLICIT]** Add a SWE-bench-Verified subcommand, notwithstanding E-9’s rule to stop reporting it as the headline benchmark.
- **E-21.F08 [EXPLICIT]** Add crates/orchestrator/src/sbfl.rs.
- **E-21.F09 [EXPLICIT]** Journal sbfl_localized.
- **E-21.F10 [EXPLICIT]** Journal ace_audit_run.
- **E-21.F11 [EXPLICIT]** Produce a suspiciousness ranking from failing tests.
- **E-21.F12 [EXPLICIT]** Let the agent choose the top-ranked file.

### E-22 — GitHub Trending May 2026

**Anchor:** lines 198-204. **Closes:** 22-github-trending-may2026. **Prerequisite/status:** B-18/C-11 PLANNED; coordinates SKILL.md export with E-43 and is parallel-safe with E-21.

- **E-22.F01 [EXPLICIT]** Export skills through a SKILL.md compatibility shim.
- **E-22.F02 [EXPLICIT]** Boot a Linux MicroVm sandbox with microsandbox/libkrun.
- **E-22.F03 [EXPLICIT]** Fall back to Process sandboxing on Windows.
- **E-22.F04 [EXPLICIT]** Add a flag for deferred MCP schema loading.
- **E-22.F05 [EXPLICIT]** Audit Streamable HTTP transport.
- **E-22.F06 [EXPLICIT]** Benchmark elai.map against claude-context.
- **E-22.F07 [EXPLICIT]** Run a 12-factor audit.
- **E-22.F08 [EXPLICIT]** Add scripts/export_skills.py.
- **E-22.F09 [EXPLICIT]** Extend crates/orchestrator/src/sandbox/.
- **E-22.F10 [EXPLICIT]** Journal skill_exported.
- **E-22.F11 [EXPLICIT]** Journal microsandbox_pool_size.
- **E-22.F12 [EXPLICIT]** Journal twelve_factor_audit.
- **E-22.F13 [EXPLICIT]** Support “elai skill export”.
- **E-22.F14 [EXPLICIT]** Produce a signed claude.skills.io payload.
- **E-22.F15 [EXPLICIT]** Boot the sandbox pool in under 100 ms.

### E-23 — Developer Discourse 2026 [POSITIONING]

**Anchor:** lines 206-212. **Closes:** 23-developer-discourse-2026. **Prerequisite/status:** POSITIONING-only; at least one of E-21/E-24/E-25 must land first; parallel-safe with every other E-N because it has no shared code surface.

- **E-23.F01 [EXPLICIT]** Add a ContextStatePanel.
- **E-23.F02 [EXPLICIT]** Add a TrustTelemetryPanel.
- **E-23.F03 [EXPLICIT]** Publish a productivity baseline.
- **E-23.F04 [EXPLICIT]** Do not claim “speedup proven” from that baseline.
- **E-23.F05 [EXPLICIT]** Rewrite CommandWindow first-run copy in maintainer voice.
- **E-23.F06 [EXPLICIT]** Improve README.md citation hygiene.
- **E-23.F07 [EXPLICIT]** Improve .claude/AGENTS.md citation hygiene.
- **E-23.F08 [EXPLICIT]** Show a pricing-transparency badge.
- **E-23.F09 [EXPLICIT]** Surface prompt-injection-check status.
- **E-23.F10 [EXPLICIT]** Add no new code surface.
- **E-23.F11 [EXPLICIT]** Add no new SSB action.
- **E-23.F12 [EXPLICIT]** Show the new copy on first dashboard run.
- **E-23.F13 [EXPLICIT]** Publish AGENTS.md.

### E-24 — Agent Protocols / MCP

**Anchor:** lines 214-220. **Closes:** 24-agent-protocols-mcp; extends B-3’s MCP bridge. **Prerequisite/status:** B-3 SHIPPED; parallel-safe with E-25/E-30/E-37.

- **E-24.F01 [EXPLICIT]** Pin MCP schemas with a blake3 hash.
- **E-24.F02 [EXPLICIT]** Sanitize Trail-of-Bits line-jumping attacks.
- **E-24.F03 [EXPLICIT]** Enforce a token budget for each tool.
- **E-24.F04 [EXPLICIT]** Migrate MCP transport from SSE to Streamable HTTP.
- **E-24.F05 [EXPLICIT]** Run an LLM-decision guard before an MCP call.
- **E-24.F06 [EXPLICIT]** Guard explicit gateway rejection.
- **E-24.F07 [EXPLICIT]** Add six contract tests; their individual cases require the original.
- **E-24.F08 [EXPLICIT]** Extend crates/plugins/src/mcp_bridge.rs.
- **E-24.F09 [EXPLICIT]** Journal mcp_tool_blake3_pinned.
- **E-24.F10 [EXPLICIT]** Journal line_jumping_sanitized.
- **E-24.F11 [EXPLICIT]** Journal streamable_http_active.
- **E-24.F12 [EXPLICIT]** Block an MCP call when a server mutates a pinned tool description.
- **E-24.F13 [EXPLICIT]** Operate with Streamable HTTP active.

### E-25 — Claude Code Architecture

**Anchor:** lines 222-228. **Closes:** 25-claude-code-architecture; extends Phase 3H session handoff. **Prerequisite/status:** Phase 3H session handoff in code; E-24 consumes prompt_decision; parallel-safe with E-44.

- **E-25.F01 [EXPLICIT]** Add a six-tier compaction ladder above SESSION_HANDOFF; tier definitions require the original.
- **E-25.F02 [EXPLICIT]** Add a dashboard surface for deferred MCP state.
- **E-25.F03 [EXPLICIT]** Support shell-command skill templating with the !command-style syntax named in the summary.
- **E-25.F04 [EXPLICIT]** Live-reload skill templates.
- **E-25.F05 [EXPLICIT]** Provide a shared prompt_decision LLM-decision hook primitive.
- **E-25.F06 [EXPLICIT]** Let E-24’s pre-MCP-call guard consume prompt_decision.
- **E-25.F07 [EXPLICIT]** Overlay an opusplan-style substate on B-37 PerRoleRuntime.
- **E-25.F08 [EXPLICIT]** Fork sessions.
- **E-25.F09 [EXPLICIT]** Create pre-edit checkpoints.
- **E-25.F10 [EXPLICIT]** Enforce seven anti-pattern guards; exact guards require the original.
- **E-25.F11 [EXPLICIT]** Extend crates/orchestrator/src/compaction.rs.
- **E-25.F12 [EXPLICIT]** Add CompactionDrawer.svelte.
- **E-25.F13 [EXPLICIT]** Journal compaction_tier_N.
- **E-25.F14 [EXPLICIT]** Journal opusplan_substate.
- **E-25.F15 [EXPLICIT]** Journal prompt_decision_evaluated.
- **E-25.F16 [EXPLICIT]** Trigger tier-1 compaction near 85% context use.
- **E-25.F17 [EXPLICIT]** Escalate through subsequent tiers as needed.
- **E-25.F18 [EXPLICIT]** Reconstruct the full session from any tier.

### E-26 — Inference Engines 2026

**Anchor:** lines 230-236. **Closes:** 26-inference-engines-2026; augments B-37 SHIPPED. **Prerequisite/status:** B-37 SHIPPED; must sequence shared server.py edits with E-27/E-28.

- **E-26.F01 [EXPLICIT]** Add an SGLang RuntimeAdapter slot.
- **E-26.F02 [EXPLICIT]** Add an XGrammar LogitsProcessor.
- **E-26.F03 [EXPLICIT]** Deprecate Ollama.
- **E-26.F04 [EXPLICIT]** Preserve the DoLa/ETC LogitsProcessor chain.
- **E-26.F05 [EXPLICIT]** Track vLLM V1 LogitsProcessor breakage as an Open question.
- **E-26.F06 [EXPLICIT]** Add crates/providers/src/runtime/sglang.rs.
- **E-26.F07 [EXPLICIT]** Extend python/elai_inference/server.py.
- **E-26.F08 [EXPLICIT]** Journal sglang_health_changed.
- **E-26.F09 [EXPLICIT]** Journal ollama_deprecated.
- **E-26.F10 [EXPLICIT]** Make the SGLang health probe green.
- **E-26.F11 [EXPLICIT]** Warn and skip when constructing an Ollama backend.

### E-27 — Quantization 2026 ★ HIGH-ROI

**Anchor:** lines 238-244. **Closes:** 27-quantization-2026. **Prerequisite/status:** B-37 SHIPPED; shared server.py edits must sequence with E-26/E-28.

- **E-27.F01 [EXPLICIT]** Switch serving quantization from NF4 to AWQ+Marlin.
- **E-27.F02 [EXPLICIT]** Preserve HumanEval at the stated 51.8% while switching.
- **E-27.F03 [EXPLICIT]** Target 4.4× serving throughput.
- **E-27.F04 [EXPLICIT]** Use an FP8 KV cache.
- **E-27.F05 [EXPLICIT]** Use native FP8 KV behavior on Ada and newer GPUs.
- **E-27.F06 [EXPLICIT]** Software-simulate FP8 KV on Ampere.
- **E-27.F07 [EXPLICIT]** Use FP8 KV to free approximately 2× context-window capacity.
- **E-27.F08 [EXPLICIT]** Reserve NF4 for QLoRA training only.
- **E-27.F09 [EXPLICIT]** Extend python/elai_inference/server.py.
- **E-27.F10 [EXPLICIT]** Add workflow/quantize/.
- **E-27.F11 [EXPLICIT]** Journal awq_loaded.
- **E-27.F12 [EXPLICIT]** Journal fp8_kv_active.
- **E-27.F13 [EXPLICIT]** Journal nf4_qlora_only.
- **E-27.F14 [EXPLICIT]** Boot the sidecar with AWQ+Marlin.
- **E-27.F15 [EXPLICIT]** Measure at least 4× throughput over NF4 at the same HumanEval result.

### E-28 — Attention Kernels 2026

**Anchor:** lines 246-252. **Closes:** 28-attention-kernels-2026. **Prerequisite/status:** B-37 SHIPPED; shared server.py edits must sequence with E-26/E-27.

- **E-28.F01 [EXPLICIT]** Make FlashAttention 2 mandatory.
- **E-28.F02 [EXPLICIT]** Pin/bump flash-attn to 2.8.3.
- **E-28.F03 [EXPLICIT]** Add StreamingLLM sinks.
- **E-28.F04 [EXPLICIT]** Gate YaRN on evaluation results.
- **E-28.F05 [EXPLICIT]** Block FA3 on Ampere.
- **E-28.F06 [EXPLICIT]** Block FA4 on Ampere.
- **E-28.F07 [EXPLICIT]** Refuse to boot the 7B SDPA path without FA2.
- **E-28.F08 [EXPLICIT]** Extend python/elai_inference/server.py.
- **E-28.F09 [EXPLICIT]** Journal fa2_pinned.
- **E-28.F10 [EXPLICIT]** Journal streamingllm_sink_active.
- **E-28.F11 [EXPLICIT]** Refuse sidecar startup when FA2 is absent.
- **E-28.F12 [EXPLICIT]** Emit fa2_pinned at startup when FA2 is active.

### E-29 — Agentic Observability

**Anchor:** lines 254-260. **Closes:** 29-agentic-observability; extends Phase 3B SSB. **Prerequisite/status:** Phase 3B SSB shipped; E-30 consumes its SSB types.

- **E-29.F01 [EXPLICIT]** Add OTel GenAI semantic-convention 1.40.0 columns to SSB.
- **E-29.F02 [EXPLICIT]** Export SSB records through a new ssb-otel-export crate.
- **E-29.F03 [EXPLICIT]** Add a Transcript drawer.
- **E-29.F04 [EXPLICIT]** Add a Failure-modes drawer.
- **E-29.F05 [EXPLICIT]** Add a Spend drawer.
- **E-29.F06 [EXPLICIT]** Add a Query drawer.
- **E-29.F07 [EXPLICIT]** Replay/resume from ssb_resume_from.
- **E-29.F08 [EXPLICIT]** Integrate the MAST 14-mode classifier as E-6’s observability sibling.
- **E-29.F09 [EXPLICIT]** Add crates/ssb_otel_export/.
- **E-29.F10 [EXPLICIT]** Extend dashboard-v2 drawers.
- **E-29.F11 [EXPLICIT]** Journal otel_span_emitted.
- **E-29.F12 [EXPLICIT]** Emit GenAI-conformant OTLP spans from a live session to a local collector.
- **E-29.F13 [EXPLICIT]** Replay a past session in a drawer.

### E-30 — Public Agent Incidents ★ SAFETY FLOOR

**Anchor:** lines 262-268. **Closes:** 30-public-agent-incidents. **Prerequisite/status:** B-19 hook unification PLANNED; parallel-safe with E-24/E-29; named a safety floor.

- **E-30.F01 [EXPLICIT]** Run a non-LLM PreToolUse argv-regex guard.
- **E-30.F02 [EXPLICIT]** Enforce an allowlist around destructive verbs.
- **E-30.F03 [EXPLICIT]** Put the trust dialog in CVE-safe ordering.
- **E-30.F04 [EXPLICIT]** Pin MCP dependencies to exact versions.
- **E-30.F05 [EXPLICIT]** Pin MCP content with blake3.
- **E-30.F06 [EXPLICIT]** Block auditor-lineage collisions.
- **E-30.F07 [EXPLICIT]** Label content sources by trust.
- **E-30.F08 [EXPLICIT]** Require a 15/15 red-team contract.
- **E-30.F09 [EXPLICIT]** Close the Replit ignored-ALL-CAPS incident class.
- **E-30.F10 [EXPLICIT]** Close the Cursor Plan-Mode bypass incident class.
- **E-30.F11 [EXPLICIT]** Close the repeated Claude Code rm -rf incident class.
- **E-30.F12 [EXPLICIT]** Add crates/plugins/src/pretool_argv.rs.
- **E-30.F13 [EXPLICIT]** Journal destructive_verb_denied.
- **E-30.F14 [EXPLICIT]** Journal mcp_pin_violation.
- **E-30.F15 [EXPLICIT]** Journal auditor_lineage_collision.
- **E-30.F16 [EXPLICIT]** Deny “rm -rf ~/” regardless of the LLM’s verdict.

### E-31 — Coding Model Leaderboard

**Anchor:** lines 270-276. **Closes:** 31-coding-model-leaderboard; strategic positioning. **Prerequisite/status:** B-37 SHIPPED; parallel-safe with E-32/E-34.

- **E-31.F01 [EXPLICIT]** Maintain a strategic model matrix by VRAM tier.
- **E-31.F02 [EXPLICIT]** Add an EscalationPolicy module.
- **E-31.F03 [EXPLICIT]** Flag Qwen3’s FA2 incompatibility.
- **E-31.F04 [EXPLICIT]** Document MoE as ELAI’s riskiest strategic bet.
- **E-31.F05 [EXPLICIT]** Provide a MoE-compatible path without DoLa.
- **E-31.F06 [EXPLICIT]** Provide a MoE-compatible path without steering.
- **E-31.F07 [EXPLICIT]** Add crates/orchestrator/src/escalation_policy.rs.
- **E-31.F08 [EXPLICIT]** Refresh CLAUDE.md’s tier table.
- **E-31.F09 [EXPLICIT]** Journal tier_model_swap.
- **E-31.F10 [EXPLICIT]** Resolve the correct model/backend tuple for 12 GB.
- **E-31.F11 [EXPLICIT]** Resolve the correct model/backend tuple for 24 GB.
- **E-31.F12 [EXPLICIT]** Resolve the correct model/backend tuple for 48 GB.

### E-32 — Context Window Extension

**Anchor:** lines 278-284. **Closes:** 32-context-window-extension. **Prerequisite/status:** B-37 SHIPPED; parallel-safe with E-17/E-31.

- **E-32.F01 [EXPLICIT]** Raise ELAI_LOCAL_MAX_CONTEXT from 25K to Qwen2.5-Coder-7B’s native 32K.
- **E-32.F02 [EXPLICIT]** Calibrate Found-in-the-Middle.
- **E-32.F03 [EXPLICIT]** Target a 15 percentage-point RAG-recall lift from that calibration.
- **E-32.F04 [EXPLICIT]** Skip YaRN because of the stated 15–20% quality cost.
- **E-32.F05 [EXPLICIT]** Skip StreamingLLM for this context-lift path.
- **E-32.F06 [EXPLICIT]** Skip RingAttention for this context-lift path.
- **E-32.F07 [EXPLICIT]** Update the sidecar environment documentation in CLAUDE.md.
- **E-32.F08 [EXPLICIT]** Update max_context in python/elai_inference/server.py.
- **E-32.F09 [EXPLICIT]** Journal context_lifted_32k.
- **E-32.F10 [EXPLICIT]** Journal fitm_recall_lift_measured.
- **E-32.F11 [EXPLICIT]** Fit a 32K-context prompt.
- **E-32.F12 [EXPLICIT]** Measure FITM RAG-recall lift on a 30-query corpus.

### E-33 — Dashboard UX Agentic

**Anchor:** lines 286-292. **Closes:** 33-dashboard-ux-agentic; a UX delta on B-26 DONE. **Prerequisite/status:** B-26 DONE; parallel-safe with non-dashboard E-Ns.

- **E-33.F01 [EXPLICIT]** Add five Svelte components; the summary names only a subset.
- **E-33.F02 [EXPLICIT]** Include a Kiro-style Activity ribbon.
- **E-33.F03 [EXPLICIT]** Include skeleton screens.
- **E-33.F04 [EXPLICIT]** Use the Wong palette.
- **E-33.F05 [EXPLICIT]** Respect the reduced-motion preference.
- **E-33.F06 [EXPLICIT]** Add footer mode.
- **E-33.F07 [EXPLICIT]** Keep the work as a frontend pair with no backend overlap.
- **E-33.F08 [EXPLICIT]** Preserve DESIGN.md’s outlined-only chrome.
- **E-33.F09 [EXPLICIT]** Extend crates/dashboard-v2/src/lib/windows/.
- **E-33.F10 [EXPLICIT]** Journal activity_ribbon_frame.
- **E-33.F11 [EXPLICIT]** Journal skeleton_screen_shown.
- **E-33.F12 [EXPLICIT]** Render skeleton bubbles on dashboard load.
- **E-33.F13 [EXPLICIT]** Scroll the Activity ribbon smoothly at 60 fps.
- **E-33.F14 [EXPLICIT]** Continue to honor reduced motion during that behavior.

### E-34 — Rust AI Ecosystem 2026

**Anchor:** lines 294-300. **Closes:** 34-rust-ai-ecosystem-2026; amends CLAUDE.md anti-stale guidance. **Prerequisite/status:** none; parallel-safe with all E-Ns; should land before B-16.

- **E-34.F01 [EXPLICIT]** Correct the anti-stale rule to recognize ort 2.0.0-rc.12 as production in ELAI.
- **E-34.F02 [EXPLICIT]** Adopt rmcp 1.7.0.
- **E-34.F03 [EXPLICIT]** Adopt genai.
- **E-34.F04 [EXPLICIT]** Adopt insta.
- **E-34.F05 [EXPLICIT]** Adopt proptest.
- **E-34.F06 [EXPLICIT]** Append to outdated-tech-audit.md.
- **E-34.F07 [EXPLICIT]** Correct B-PLAN-SUMMARY.md.
- **E-34.F08 [EXPLICIT]** Apply that correction before B-16 runs.
- **E-34.F09 [EXPLICIT]** Journal ort_rc_amended.
- **E-34.F10 [EXPLICIT]** Journal rmcp_adopted.
- **E-34.F11 [EXPLICIT]** Ensure B-16 enforcement does not regress the ort dependency after E-34 lands.

### E-35 — Structured Output 2026 ★ SUBSTRATE

**Anchor:** lines 302-308. **Closes:** 35-structured-output-2026; substrate consumed by E-18/E-19/E-20. **Prerequisite/status:** none; parallel-safe with all non-sidecar E-Ns; must land before E-18.

- **E-35.F01 [EXPLICIT]** Add an XGrammar sidecar grammar-mask substrate.
- **E-35.F02 [EXPLICIT]** Add INSTRUCTOR mode.
- **E-35.F03 [EXPLICIT]** Add CodeAct mode.
- **E-35.F04 [EXPLICIT]** Target a 20% CodeAct accuracy gain.
- **E-35.F05 [EXPLICIT]** Target a 30% CodeAct turn-count reduction.
- **E-35.F06 [EXPLICIT]** Add SEARCH/REPLACE fences for non-Git operation.
- **E-35.F07 [EXPLICIT]** Parse Qwen tool calls with qwen_tools.
- **E-35.F08 [EXPLICIT]** Make downstream E-18 tool-call wiring thin by landing the substrate first.
- **E-35.F09 [EXPLICIT]** Extend crates/providers/src/decoding.rs.
- **E-35.F10 [EXPLICIT]** Add the XGrammar mask to python/elai_inference/server.py.
- **E-35.F11 [EXPLICIT]** Journal xgrammar_substrate_loaded.
- **E-35.F12 [EXPLICIT]** Journal codeact_step.
- **E-35.F13 [EXPLICIT]** Complete a JSON-schema structured-output task with zero retries.
- **E-35.F14 [EXPLICIT]** Measure a 30% turn-count reduction in CodeAct mode.

### E-36 — Document Vision RAG

**Anchor:** lines 310-316. **Closes:** 36-document-vision-rag; adds the eighth GroundingMethod. **Prerequisite/status:** B-35 ArtifactCheck, B-42 CosmosCheck, and Phase 2B grounding stubs; parallel-safe with most E-plan work.

- **E-36.F01 [EXPLICIT]** Add GroundingMethod::VisualArtifactCheck.
- **E-36.F02 [EXPLICIT]** Keep it alongside FILE grounding.
- **E-36.F03 [EXPLICIT]** Keep it alongside SOURCE grounding.
- **E-36.F04 [EXPLICIT]** Keep it alongside TEST grounding.
- **E-36.F05 [EXPLICIT]** Keep it alongside WEB grounding.
- **E-36.F06 [EXPLICIT]** Keep it alongside API grounding.
- **E-36.F07 [EXPLICIT]** Keep it alongside API_REGISTRY grounding.
- **E-36.F08 [EXPLICIT]** Keep it alongside Artifact grounding.
- **E-36.F09 [EXPLICIT]** Keep it alongside Cosmos grounding.
- **E-36.F10 [EXPLICIT]** Use ColQwen2.5, with the cited ViDoRe v2 score of 0.597.
- **E-36.F11 [EXPLICIT]** Use PaddleOCR-VL 0.9B.
- **E-36.F12 [EXPLICIT]** Route claims about design.pdf to the vision verifier.
- **E-36.F13 [EXPLICIT]** Route claims about api-spec.pdf to the vision verifier.
- **E-36.F14 [EXPLICIT]** Extend crates/reasoning/src/grounding.rs.
- **E-36.F15 [EXPLICIT]** Add workflow/vision_rag/.
- **E-36.F16 [EXPLICIT]** Journal visual_artifact_check.
- **E-36.F17 [EXPLICIT]** Extract and verify a PDF claim through the VLM.
- **E-36.F18 [EXPLICIT]** Fall through to existing grounding methods for a non-PDF claim.

### E-37 — Prompt Injection Defense

**Anchor:** lines 318-324. **Closes:** 37-prompt-injection-defense; extends E-24 and the visibility filter. **Prerequisite/status:** E-24 MCP hardening; parallel-safe with E-39/E-50.

- **E-37.F01 [EXPLICIT]** Wrap ingested untrusted content in a spotlighting envelope.
- **E-37.F02 [EXPLICIT]** Pin MCP tool descriptions with blake3.
- **E-37.F03 [EXPLICIT]** Do not use SHA-256 for that pin.
- **E-37.F04 [EXPLICIT]** Classify prompt injection with PromptArmor off-model.
- **E-37.F05 [EXPLICIT]** Restrict egress with an allowlist.
- **E-37.F06 [EXPLICIT]** Strip Markdown images.
- **E-37.F07 [EXPLICIT]** Close the data-plane prompt-injection gap.
- **E-37.F08 [EXPLICIT]** Treat the visibility filter and MCP deny-list as control-plane defenses, not sufficient data-plane defenses.
- **E-37.F09 [EXPLICIT]** Add crates/plugins/src/prompt_injection.rs.
- **E-37.F10 [EXPLICIT]** Journal spotlighting_envelope.
- **E-37.F11 [EXPLICIT]** Journal promptarmor_verdict.
- **E-37.F12 [EXPLICIT]** Journal egress_blocked.
- **E-37.F13 [EXPLICIT]** Log a PromptArmor verdict for a hostile injected PDF page.
- **E-37.F14 [EXPLICIT]** Block egress to a URL outside the allowlist.

### E-38 — AI Roadmap 2026 Signals

**Anchor:** lines 326-332. **Closes:** 38-ai-roadmap-2026-signals; compliance posture. **Prerequisite/status:** Phase 3B SSB; parallel-safe with most E-plan work.

- **E-38.F01 [EXPLICIT]** Add an EU AI Act May-2026 audit-trail liveness gate for Article 13.
- **E-38.F02 [EXPLICIT]** Document the MoE strategic bet.
- **E-38.F03 [EXPLICIT]** Maintain a RAG-first posture through 2027.
- **E-38.F04 [EXPLICIT]** Compose compliance evidence with E-40 symbol-graph audit logs.
- **E-38.F05 [EXPLICIT]** Compose compliance evidence with E-39’s LayeredCapManualOverride SSB row.
- **E-38.F06 [EXPLICIT]** Add workflow/compliance/eu_ai_act.md.
- **E-38.F07 [EXPLICIT]** Add crates/orchestrator/src/compliance_gate.rs.
- **E-38.F08 [EXPLICIT]** Journal eu_ai_act_audit.
- **E-38.F09 [EXPLICIT]** Emit an Article-13-conformant audit row on PLANNING → TESTING_FIRST.

### E-39 — Cost Token Economics ★ HARD CEILING

**Anchor:** lines 334-340. **Closes:** 39-cost-token-economics; extends B-27/C-5 PLANNED. **Prerequisite/status:** B-27 and C-5 PLANNED; E-20 consumes it.

- **E-39.F01 [EXPLICIT]** Enforce hard cost ceilings by blocking, not warning.
- **E-39.F02 [EXPLICIT]** Allow at most one retry.
- **E-39.F03 [EXPLICIT]** Escalate after the one-shot retry is exhausted.
- **E-39.F04 [EXPLICIT]** Gate spend separately per tool.
- **E-39.F05 [EXPLICIT]** Estimate task difficulty before selecting a cloud route.
- **E-39.F06 [EXPLICIT]** Close the $47K wallet-loop incident class.
- **E-39.F07 [EXPLICIT]** Close the 2.3M retry-storm incident class.
- **E-39.F08 [EXPLICIT]** Allow a user-driven LayeredCapManualOverride.
- **E-39.F09 [EXPLICIT]** Extend crates/orchestrator/src/cost_gate.rs.
- **E-39.F10 [EXPLICIT]** Journal hard_ceiling_blocked.
- **E-39.F11 [EXPLICIT]** Journal retry_budget_exhausted.
- **E-39.F12 [EXPLICIT]** With cap=$0.05 and warning=80%, block at exactly $0.05.
- **E-39.F13 [EXPLICIT]** Escalate rather than retry when the retry budget is exhausted.

### E-40 — Symbol Graph Tools

**Anchor:** lines 342-348. **Closes:** 40-symbol-graph-tools; extends B-29/C-24 PLANNED. **Prerequisite/status:** B-29/C-24 PLANNED; parallel-safe with E-52.

- **E-40.F01 [EXPLICIT]** Use a SCIP plus tree-sitter symbol-graph hybrid already present in elai.map.
- **E-40.F02 [EXPLICIT]** Skip Stack Graphs because it was archived in September 2025.
- **E-40.F03 [EXPLICIT]** Flag arxiv:2506.11419 as a misattributed FocalAD citation.
- **E-40.F04 [EXPLICIT]** Treat that as the third bad-citation pattern after B-40/B-42.
- **E-40.F05 [EXPLICIT]** Feed the issue to feedback_paper_citation_verify.
- **E-40.F06 [EXPLICIT]** Add a SymbolGraphDSL query layer.
- **E-40.F07 [EXPLICIT]** Add crates/knowledge/src/symbol_dsl.rs.
- **E-40.F08 [EXPLICIT]** Journal symbol_graph_dsl_query.
- **E-40.F09 [EXPLICIT]** Support “elai symbol dsl callers(SsbActionType)”.
- **E-40.F10 [EXPLICIT]** Return a deterministic ranked list from that query.

### E-41 — CI/CD PR Review

**Anchor:** lines 350-356. **Closes:** 41-ci-cd-pr-review; extends C-40 PLANNED GitHub Adapter. **Prerequisite/status:** C-40 PLANNED; pairs with E-45 at PR time.

- **E-41.F01 [EXPLICIT]** Defend changes in-flow.
- **E-41.F02 [EXPLICIT]** Defend changes at PR review.
- **E-41.F03 [EXPLICIT]** Defend changes at deployment.
- **E-41.F04 [EXPLICIT]** Defend changes through observation.
- **E-41.F05 [EXPLICIT]** Assign a PR reviewer from a different model family than the author.
- **E-41.F06 [EXPLICIT]** Run the Semgrep AI ruleset.
- **E-41.F07 [EXPLICIT]** Expose dependency verification as a tool call.
- **E-41.F08 [EXPLICIT]** Commit every baby step.
- **E-41.F09 [EXPLICIT]** Merge with --no-ff.
- **E-41.F10 [EXPLICIT]** Put AI attribution only in Git trailers under commit-cadence.md.
- **E-41.F11 [EXPLICIT]** Extend crates/github_adapter/.
- **E-41.F12 [EXPLICIT]** Journal pr_review_dispatched.
- **E-41.F13 [EXPLICIT]** Dispatch a cross-family review comment when a PR opens.
- **E-41.F14 [EXPLICIT]** Surface an SSRF finding from Semgrep AI rules.
- **E-41.F15 [EXPLICIT]** Have ELAI fix the finding before merge.

### E-42 — Tree-Sitter Grammars 2026

**Anchor:** lines 358-364. **Closes:** 42-tree-sitter-grammars-2026; extends B-30 PLANNED. **Prerequisite/status:** B-30 PLANNED; parallel-safe with most E-plan work.

- **E-42.F01 [EXPLICIT]** Pin tree-sitter 0.26.x.
- **E-42.F02 [EXPLICIT]** Probe grammar ABI 15.
- **E-42.F03 [EXPLICIT]** Put 15 production languages in TIER-A.
- **E-42.F04 [EXPLICIT]** Put experimental languages in cfg-gated TIER-B.
- **E-42.F05 [EXPLICIT]** Measure Svelte 5 runes ERROR-rate telemetry.
- **E-42.F06 [EXPLICIT]** Add injection.scm queries.
- **E-42.F07 [EXPLICIT]** Close the Svelte-5-runes-ungrammared gap.
- **E-42.F08 [EXPLICIT]** Extend crates/knowledge/src/grammars.rs.
- **E-42.F09 [EXPLICIT]** Pin dependencies in Cargo.toml.
- **E-42.F10 [EXPLICIT]** Journal tier_b_grammar_failed.
- **E-42.F11 [EXPLICIT]** Keep Svelte 5 runes ERROR rate below 30% in smoke verification.
- **E-42.F12 [EXPLICIT]** Fall back to the line-window chunker when a TIER-B grammar fails.

### E-43 — Agent Skill Systems

**Anchor:** lines 366-372. **Closes:** 43-agent-skill-systems; extends/coordinates E-22. **Prerequisite/status:** B-19/B-20 PLANNED; coordinates SKILL.md export with E-22.

- **E-43.F01 [EXPLICIT]** Make skill frontmatter comply with agentskills.io v1.
- **E-43.F02 [EXPLICIT]** Add an eval/ directory per skill.
- **E-43.F03 [EXPLICIT]** Record skill invocation telemetry.
- **E-43.F04 [EXPLICIT]** Hash skill telemetry with blake3.
- **E-43.F05 [EXPLICIT]** Publish skills for Claude.
- **E-43.F06 [EXPLICIT]** Publish skills for Codex.
- **E-43.F07 [EXPLICIT]** Publish skills for Gemini.
- **E-43.F08 [EXPLICIT]** Gate imported skills with mcp-scan.
- **E-43.F09 [EXPLICIT]** Add .claude/rules/skill-import-policy.md.
- **E-43.F10 [EXPLICIT]** Close the ToxicSkills incident class.
- **E-43.F11 [EXPLICIT]** Close the ClawHavoc incident class.
- **E-43.F12 [EXPLICIT]** Add frontmatter to .claude/skills/*.
- **E-43.F13 [EXPLICIT]** Add scripts/skill_publish.sh.
- **E-43.F14 [EXPLICIT]** Journal skill_imported.
- **E-43.F15 [EXPLICIT]** Journal skill_invocation_logged.
- **E-43.F16 [EXPLICIT]** Support “elai skill publish”.
- **E-43.F17 [EXPLICIT]** Produce a signed payload accepted by claude.skills.io.
- **E-43.F18 [EXPLICIT]** Block an unsigned skill import with mcp-scan.

### E-44 — Checkpointing Durability

**Anchor:** lines 374-380. **Closes:** 44-checkpointing-durability; extends B-9 SHIPPED. **Prerequisite/status:** B-9 SHIPPED; parallel-safe with E-25/E-45.

- **E-44.F01 [EXPLICIT]** Wrap tool calls with idempotency keys.
- **E-44.F02 [EXPLICIT]** Record compensating actions in a SAGA log.
- **E-44.F03 [EXPLICIT]** Roll SessionDag over with Continue-As-New.
- **E-44.F04 [EXPLICIT]** Test distributed-system behavior with a madsim DST harness.
- **E-44.F05 [EXPLICIT]** Lazily migrate checkpoints from SCHEMA_VERSION 40 through 45.
- **E-44.F06 [EXPLICIT]** Close the ACRFence semantic-rollback attack class.
- **E-44.F07 [EXPLICIT]** Add crates/orchestrator/src/idempotency.rs.
- **E-44.F08 [EXPLICIT]** Add crates/orchestrator/src/saga.rs.
- **E-44.F09 [EXPLICIT]** Journal idempotency_replay.
- **E-44.F10 [EXPLICIT]** Journal saga_compensated.
- **E-44.F11 [EXPLICIT]** Journal continue_as_new.
- **E-44.F12 [EXPLICIT]** Return the same result for a retried tool call with the same idempotency key.
- **E-44.F13 [EXPLICIT]** Run SAGA compensation after partial failure.

### E-45 — Cross-Language Drift

**Anchor:** lines 382-388. **Closes:** 45-cross-language-drift; extends B-3 ts-rs base. **Prerequisite/status:** B-3 SHIPPED; pairs with E-41 at PR time.

- **E-45.F01 [EXPLICIT]** Snapshot generated types with ts-rs v12.
- **E-45.F02 [EXPLICIT]** Apply schemars at the sidecar boundary.
- **E-45.F03 [EXPLICIT]** Lock action_type in a golden file.
- **E-45.F04 [EXPLICIT]** Make action_type append-only in the Protobuf style.
- **E-45.F05 [EXPLICIT]** Extend .claude/tools/cross_lang_diff.py with --strict mode.
- **E-45.F06 [EXPLICIT]** Add a prek pre-commit hook.
- **E-45.F07 [EXPLICIT]** Require every new SSB action type to pass through the golden file when landed.
- **E-45.F08 [EXPLICIT]** Add crates/contracts/tests/ts_rs_snapshot.rs.
- **E-45.F09 [EXPLICIT]** Add crates/contracts/tests/action_type_golden.rs.
- **E-45.F10 [EXPLICIT]** Journal action_type_golden_drift.
- **E-45.F11 [EXPLICIT]** Make a renamed Rust SsbActionType cause cross_lang_diff.py to exit 1.
- **E-45.F12 [EXPLICIT]** Block until Rust and counterpart types both update.

### E-46 — Planning Decomposition

**Anchor:** lines 390-396. **Closes:** 46-planning-decomposition; extends B-22b 4G.9 and B-40 SHIPPED. **Prerequisite/status:** B-22/B-40 SHIPPED; parallel-safe with E-48/E-49.

- **E-46.F01 [EXPLICIT]** Maintain a dual ledger during decomposition.
- **E-46.F02 [EXPLICIT]** Shortcut leaf tasks through AGENTLESS.
- **E-46.F03 [EXPLICIT]** Maintain a Reflexion buffer.
- **E-46.F04 [EXPLICIT]** Detect goal drift with Jaccard similarity.
- **E-46.F05 [EXPLICIT]** Detect goal drift with cosine similarity.
- **E-46.F06 [EXPLICIT]** Stop decomposition on entropy criteria.
- **E-46.F07 [EXPLICIT]** Score plan branches with a PRM scalar.
- **E-46.F08 [EXPLICIT]** Cascade first to REPLAN_PHASE.
- **E-46.F09 [EXPLICIT]** Cascade onward to LEDGER_RESET.
- **E-46.F10 [EXPLICIT]** Widen the 4G.9 canary.
- **E-46.F11 [EXPLICIT]** Extend crates/orchestrator/src/decomposition.rs.
- **E-46.F12 [EXPLICIT]** Journal goal_drift_canary_fired.
- **E-46.F13 [EXPLICIT]** Journal replan_phase.
- **E-46.F14 [EXPLICIT]** Journal ledger_reset.
- **E-46.F15 [EXPLICIT]** Make a synthetic drifting trajectory fire the canary and REPLAN_PHASE.
- **E-46.F16 [EXPLICIT]** Let a clean trajectory complete.

### E-47 — LoRA Finetuning [DEFERRED]

**Anchor:** lines 398-404. **Closes:** 47-lora-finetuning. **Prerequisite/status:** entirely DEFERRED/research-grade; requires RunPod retraining and D-30 PLANNED; parallel-safe with all because it does not ship runtime code.

- **E-47.F01 [EXPLICIT]** Defer LoRA implementation because the report observes 5–19 percentage-point catastrophic forgetting.
- **E-47.F02 [EXPLICIT]** Permit only a SPEC.md for the Python subpackage.
- **E-47.F03 [EXPLICIT]** Ship no Python .py implementation.
- **E-47.F04 [EXPLICIT]** Make CI require the DEFERRED_PER_E47_REPORT ship-gate token.
- **E-47.F05 [EXPLICIT]** Document an Unsloth rung.
- **E-47.F06 [EXPLICIT]** Document a QLoRA rung.
- **E-47.F07 [EXPLICIT]** Document a PiSSA rung.
- **E-47.F08 [EXPLICIT]** Document a DoRA rung.
- **E-47.F09 [EXPLICIT]** Do not build those ladder rungs.
- **E-47.F10 [EXPLICIT]** Create only workflow/research/lora/SPEC.md.
- **E-47.F11 [EXPLICIT]** Add no SSB action.
- **E-47.F12 [EXPLICIT]** Pass CI when the ship-gate token is present.
- **E-47.F13 [EXPLICIT]** Fail CI when a .py file is added without an unlock.

### E-48 — Web Research Orchestrator

**Anchor:** lines 406-412. **Closes:** 48-web-research-orchestrator; extends B-7 SHIPPED and C-48 SafeHarbor PLANNED. **Prerequisite/status:** B-7/B-39 SHIPPED; parallel-safe with E-46.

- **E-48.F01 [EXPLICIT]** Let the research plan be editable.
- **E-48.F02 [EXPLICIT]** Fan search out to Brave in a tiered policy.
- **E-48.F03 [EXPLICIT]** Fan search out to SearXNG in a tiered policy.
- **E-48.F04 [EXPLICIT]** Fan search out to Mojeek in a tiered policy.
- **E-48.F05 [EXPLICIT]** Extract content with crawl4ai.
- **E-48.F06 [EXPLICIT]** Extract content with Jina.
- **E-48.F07 [EXPLICIT]** Rerank results with a cross-encoder.
- **E-48.F08 [EXPLICIT]** Prune results with FlashResearch.
- **E-48.F09 [EXPLICIT]** Apply a conformal triple threshold.
- **E-48.F10 [EXPLICIT]** Represent contradictions with ContradictionNode.
- **E-48.F11 [EXPLICIT]** Verify citations with a four-stage CitationVerifier; stages require the original.
- **E-48.F12 [EXPLICIT]** Harden against GhostCite.
- **E-48.F13 [EXPLICIT]** Lock attribution to the responsible sub-agent.
- **E-48.F14 [EXPLICIT]** Extend crates/orchestrator/src/web_research_orchestrator.rs.
- **E-48.F15 [EXPLICIT]** Journal citation_verified.
- **E-48.F16 [EXPLICIT]** Journal contradiction_surfaced.
- **E-48.F17 [EXPLICIT]** Reach at least an 80% citation_verified pass rate on a research query.
- **E-48.F18 [EXPLICIT]** Surface cross-source contradictions in the dashboard.

### E-49 — Coding Agent Metrics

**Anchor:** lines 414-420. **Closes:** 49-coding-agent-metrics; extends B-23/C-50/C-51 PLANNED. **Prerequisite/status:** C-50/C-51 PLANNED; parallel-safe with E-9/E-46.

- **E-49.F01 [EXPLICIT]** Measure STABLE-Pass@1 with n=5.
- **E-49.F02 [EXPLICIT]** Measure dollars per CTS.
- **E-49.F03 [EXPLICIT]** Measure the METR 50% task horizon.
- **E-49.F04 [EXPLICIT]** Record a harness-vs-model attribution tuple.
- **E-49.F05 [EXPLICIT]** Evaluate on a private held-out set.
- **E-49.F06 [EXPLICIT]** Run a 30-day revert canary.
- **E-49.F07 [EXPLICIT]** Measure prompt-injection attack success rate.
- **E-49.F08 [EXPLICIT]** Measure evaluation awareness.
- **E-49.F09 [EXPLICIT]** Measure context rot.
- **E-49.F10 [EXPLICIT]** Measure Cover@τ.
- **E-49.F11 [EXPLICIT]** Enforce STOP-Verified through CI lint.
- **E-49.F12 [EXPLICIT]** Extend crates/cli/src/metrics.rs.
- **E-49.F13 [EXPLICIT]** Journal stable_pass_at_1.
- **E-49.F14 [EXPLICIT]** Journal harness_attribution_tuple.
- **E-49.F15 [EXPLICIT]** Journal stop_verified.
- **E-49.F16 [EXPLICIT]** Emit per-task pass_rate.
- **E-49.F17 [EXPLICIT]** Emit per-task cost_per_token.
- **E-49.F18 [EXPLICIT]** Emit per-task horizon_min.
- **E-49.F19 [EXPLICIT]** Emit per-task harness_attr.
- **E-49.F20 [EXPLICIT]** Emit per-task ASR.
- **E-49.F21 [EXPLICIT]** Emit per-task eval_aware.
- **E-49.F22 [EXPLICIT]** Emit per-task cover_tau.

### E-50 — SaaS Architecture Patterns

**Anchor:** lines 422-428. **Closes:** 50-saas-architecture-patterns; extends/consumes E-11/E-12. **Prerequisite/status:** E-11/E-12; parallel-safe with most E-plan work.

- **E-50.F01 [EXPLICIT]** Define the B2C SaaS archetype.
- **E-50.F02 [EXPLICIT]** Define the B2B SaaS archetype.
- **E-50.F03 [EXPLICIT]** Define the API-first SaaS archetype.
- **E-50.F04 [EXPLICIT]** Define the AI-app SaaS archetype.
- **E-50.F05 [EXPLICIT]** Emit a mandatory RLS BabyStep.
- **E-50.F06 [EXPLICIT]** Gate production readiness with six checks; exact checks require the original.
- **E-50.F07 [EXPLICIT]** Enforce scoped tokens.
- **E-50.F08 [EXPLICIT]** Scaffold OpenTelemetry.
- **E-50.F09 [EXPLICIT]** Scaffold SLOs.
- **E-50.F10 [EXPLICIT]** Scaffold a runbook.
- **E-50.F11 [EXPLICIT]** Choose between Merchant-of-Record and direct Stripe integration.
- **E-50.F12 [EXPLICIT]** Supply four archetype seed TOMLs.
- **E-50.F13 [EXPLICIT]** Let the Designer consume the archetype catalog.
- **E-50.F14 [EXPLICIT]** Add workflow/archetypes/*.toml.
- **E-50.F15 [EXPLICIT]** Add crates/orchestrator/src/archetype.rs.
- **E-50.F16 [EXPLICIT]** Journal archetype_selected.
- **E-50.F17 [EXPLICIT]** Journal rls_baby_step_emitted.
- **E-50.F18 [EXPLICIT]** Select B2C for “build me a B2C SaaS”.
- **E-50.F19 [EXPLICIT]** Insert the RLS baby step into that pipeline.

### E-51 — Edge Inference + MLX

**Anchor:** lines 430-436. **Closes:** 51-edge-inference-and-mlx; extends B-37 SHIPPED. **Prerequisite/status:** B-37 SHIPPED; parallel-safe with most E-plan work.

- **E-51.F01 [EXPLICIT]** Add RuntimeBackend::Mlx.
- **E-51.F02 [EXPLICIT]** Use MLX as an Apple-Silicon fast path.
- **E-51.F03 [EXPLICIT]** Record the cited M5 Max result of 158 tokens/second on Qwen 1.7B.
- **E-51.F04 [EXPLICIT]** Add RuntimeBackend::LlamaCpp.
- **E-51.F05 [EXPLICIT]** Use llama.cpp as a universal GGUF fallback.
- **E-51.F06 [EXPLICIT]** Expose MLX through a /v1/messages adapter.
- **E-51.F07 [EXPLICIT]** Expose llama.cpp through a /v1/messages adapter.
- **E-51.F08 [EXPLICIT]** Hard-defer NPU support.
- **E-51.F09 [EXPLICIT]** Document consideration of Tauri webview.
- **E-51.F10 [EXPLICIT]** Document consideration of transformers.js.
- **E-51.F11 [EXPLICIT]** Document consideration of WebGPU.
- **E-51.F12 [EXPLICIT]** Add crates/providers/src/runtime/mlx.rs.
- **E-51.F13 [EXPLICIT]** Add crates/providers/src/runtime/llamacpp.rs.
- **E-51.F14 [EXPLICIT]** Journal mlx_loaded.
- **E-51.F15 [EXPLICIT]** Journal llamacpp_loaded.
- **E-51.F16 [EXPLICIT]** Resolve the MLX adapter on a macOS M-series host.
- **E-51.F17 [EXPLICIT]** Resolve the llama.cpp adapter on any host with GGUF.

### E-52 — LSP Integration

**Anchor:** lines 438-444. **Closes:** 52-lsp-integration; extends B-19/B-29 PLANNED. **Prerequisite/status:** B-19/B-29 PLANNED; parallel-safe with E-40.

- **E-52.F01 [EXPLICIT]** Use LSP as a verifier rather than a retriever.
- **E-52.F02 [EXPLICIT]** Correct the cited “LSP doesn’t help” conclusion as applying only to retrieval.
- **E-52.F03 [EXPLICIT]** Run rust-analyzer analysis-stats headlessly.
- **E-52.F04 [EXPLICIT]** Run rust-analyzer scip headlessly.
- **E-52.F05 [EXPLICIT]** Run rust-analyzer lsif headlessly.
- **E-52.F06 [EXPLICIT]** Run rust-analyzer ssr headlessly.
- **E-52.F07 [EXPLICIT]** Run rust-analyzer diagnostics headlessly.
- **E-52.F08 [EXPLICIT]** Use multilspy as a multi-LSP client.
- **E-52.F09 [EXPLICIT]** Document jdtls as memory-heavy.
- **E-52.F10 [EXPLICIT]** Document rust-analyzer as memory-heavy.
- **E-52.F11 [EXPLICIT]** Run a diagnostic probe before AUDITING.
- **E-52.F12 [EXPLICIT]** Trigger the diagnostic probe from a PostToolUse hook.
- **E-52.F13 [EXPLICIT]** Add crates/plugins/src/lsp_verifier.rs.
- **E-52.F14 [EXPLICIT]** Journal lsp_diagnostic_probed.
- **E-52.F15 [EXPLICIT]** Run rust-analyzer diagnostics after a Write/Edit tool call.
- **E-52.F16 [EXPLICIT]** Surface a deliberate type error before AUDITING.

### E-53 — Formal Verification Codegen

**Anchor:** lines 446-452. **Closes:** 53-formal-verification-codegen; extends D-3/C-52 PLANNED. **Prerequisite/status:** D-3 PLANNED; parallel-safe with E-52/E-54.

- **E-53.F01 [EXPLICIT]** Add narrow Z3 SMT checks to D-3 verifier consensus.
- **E-53.F02 [EXPLICIT]** Gate Z3 use on CSQ Critical classification.
- **E-53.F03 [EXPLICIT]** Run Kani on Rust hot paths during AUDITING.
- **E-53.F04 [EXPLICIT]** Generate AutoVerus-style Verus annotations for FSM invariants.
- **E-53.F05 [EXPLICIT]** Generate AutoVerus-style Verus annotations for IPC invariants.
- **E-53.F06 [EXPLICIT]** Gate COMMITTING in CI on those Verus invariants.
- **E-53.F07 [EXPLICIT]** Do not run the Verus gate on every audit.
- **E-53.F08 [EXPLICIT]** Use Dafny as a verification-aware IR for safety-critical Python.
- **E-53.F09 [EXPLICIT]** Hard-avoid Lean theorem-prover loops as a routine audit gate because of TPU-day latency.
- **E-53.F10 [EXPLICIT]** Add verifier modules z3.rs, kani.rs, verus.rs, and dafny.rs.
- **E-53.F11 [EXPLICIT]** Journal z3_consensus.
- **E-53.F12 [EXPLICIT]** Journal kani_proved.
- **E-53.F13 [EXPLICIT]** Journal autoverus_proved.
- **E-53.F14 [EXPLICIT]** Require Z3 consensus before promoting a CSQ-Critical financial-data function.
- **E-53.F15 [EXPLICIT]** Reject a Lean-loop attempt through CI lint.

### E-54 — Eval Corpus Design

**Anchor:** lines 454-460. **Closes:** 54-eval-corpus-design; replaces D-6’s 9-gram filter and extends D-23. **Prerequisite/status:** D-6 PLANNED; parallel-safe with E-49.

- **E-54.F01 [EXPLICIT]** Replace 9-gram contamination matching with 13-gram matching.
- **E-54.F02 [EXPLICIT]** Shard held-out tasks into per-family directories.
- **E-54.F03 [EXPLICIT]** Maintain _index/{family}.jsonl indexes.
- **E-54.F04 [EXPLICIT]** Avoid NTFS performance cliffs at 5,000 tasks.
- **E-54.F05 [EXPLICIT]** Run matched-pair evaluation-vs-deployment sycophancy probes.
- **E-54.F06 [EXPLICIT]** Feed those probes into D-23.
- **E-54.F07 [EXPLICIT]** Detect/verbalize evaluation awareness.
- **E-54.F08 [EXPLICIT]** Maintain a held-out test-triplet ORM.
- **E-54.F09 [EXPLICIT]** Refactor .elai_cc/held-out/.
- **E-54.F10 [EXPLICIT]** Add scripts/eval_corpus_audit.py.
- **E-54.F11 [EXPLICIT]** Journal ngram13_filtered.
- **E-54.F12 [EXPLICIT]** Journal eval_aware_flag.
- **E-54.F13 [EXPLICIT]** Journal per_family_shard_loaded.
- **E-54.F14 [EXPLICIT]** Catch a CodeForces contamination sample missed by D-6’s 9-gram filter.
- **E-54.F15 [EXPLICIT]** Load a 5,000-task fixture in under one second.

### E-55 — Agentic RL and Reward

**Anchor:** lines 462-468. **Closes:** 55-agentic-rl-and-reward; extends D-5 PLANNED and D-28 DEFERRED. **Prerequisite/status:** D-5 PLANNED; parallel-safe with E-46/E-49; RL ladder deferred, practical SFT rung active.

- **E-55.F01 [EXPLICIT]** Do not fine-tune ELAI with RL yet.
- **E-55.F02 [EXPLICIT]** Treat D-5 BAI Thompson sampling as existing soft contextual RL with logarithmic regret.
- **E-55.F03 [EXPLICIT]** Hard-avoid DPO because of the cited 3–7 percentage-point out-of-distribution drop.
- **E-55.F04 [EXPLICIT]** If escalation is later unlocked, use KTO.
- **E-55.F05 [EXPLICIT]** If escalation is later unlocked, use RLVR.
- **E-55.F06 [EXPLICIT]** Prefer SFT on successful rollouts as the practical next rung.
- **E-55.F07 [EXPLICIT]** Base that rung on the SWE-Gym pattern.
- **E-55.F08 [EXPLICIT]** Record the cited 12–14 percentage-point result from 491 traces as rationale, not as an ELAI result.
- **E-55.F09 [EXPLICIT]** Require three unlock conditions before any RL ladder; the summary does not enumerate them.
- **E-55.F10 [EXPLICIT]** Keep workflow/research/rl_ladder/SPEC.md as a deferred spec.
- **E-55.F11 [EXPLICIT]** Add workflow/training/sft_on_rollouts/ as the practical implementation.
- **E-55.F12 [EXPLICIT]** Journal sft_on_rollouts_promoted.
- **E-55.F13 [EXPLICIT]** Require a 491-trace SFT run to improve private-held-out results by at least five percentage points.
- **E-55.F14 [EXPLICIT]** Keep the full RL ladder deferred until all three conditions are satisfied.

## Interactions and workflows

### Explicit end-to-end compositions

- **[EXPLICIT; lines 30-68] Memory/UX:** E-5 owns memory_facts, voice_style, expertise, trust/decay, and poisoning defense. E-1 layers long-term digest/Core/Archival behavior on it; E-3 consumes voice_style in role-oriented handoff UX; E-4 consumes expertise to calibrate terseness.
- **[EXPLICIT; lines 70-108, 254-260] Multi-agent quality:** E-6 supplies MAST classification, cross-family disagreement, and iso-budget deletion evidence; E-10 uses that evidence as its ablation gate; E-29 visualizes/replays MAST failures; E-7 and E-8 add guarded evolutionary changes around the same orchestration layer.
- **[EXPLICIT; lines 110-148, 422-428] Idea-to-production SaaS:** E-11 frames and critiques the idea; E-12 adapts it to framework conventions and runs production checks; E-50 selects an archetype and inserts mandatory RLS/operations scaffolding; E-14 exercises the built app through browser E2E; E-15 supplies the sandbox profile needed to run Playwright safely.
- **[EXPLICIT; lines 158-188, 302-308] Structured tool/reasoning/routing path:** E-35 lands the grammar-mask substrate first. E-18 wires it into tool-call sites and shares a path-allowlist primitive with E-20. E-19 uses the mask for Typed-CoT. E-20 then chooses local/cloud execution under privacy and leakage rules.
- **[EXPLICIT; lines 158-164, 278-284] Retrieval lift:** E-17 activates/stages ONNX code retrieval, contextual retrieval, reranking, and FITM; E-32 adds the native 32K window and also calibrates FITM. Both prove lift through recall/MRR measurements rather than prose.
- **[EXPLICIT; lines 214-228, 318-324] MCP decision/security:** E-25 supplies prompt_decision; E-24 consumes it before MCP calls and adds schema pinning/sanitization/budget/transport guards; E-37 extends that control-plane hardening with data-plane spotlighting, off-model classification, egress restriction, and image stripping.
- **[EXPLICIT; lines 230-252, 430-436] Runtime backend stack:** E-26 adds SGLang and deprecates Ollama while preserving LogitsProcessors; E-27 changes quantization/cache policy; E-28 enforces FA2; E-51 adds MLX and llama.cpp adapters. E-26/E-27/E-28 must sequence their shared python/elai_inference/server.py edits.
- **[EXPLICIT; lines 262-268, 334-340] Runtime safety floor:** E-30 blocks destructive commands before the model can authorize them; E-39 blocks runaway cost/retries; both emit typed denial/exhaustion evidence.
- **[EXPLICIT; lines 342-348, 438-452] Static/semantic verification:** E-40 exposes deterministic symbol-graph queries; E-52 uses language servers as post-edit verifiers; E-53 escalates CSQ-Critical/hot-path work into Z3, Kani, Verus, or Dafny checks while rejecting routine Lean loops.
- **[EXPLICIT; lines 350-388] PR and schema integrity:** E-41 performs heterogeneous PR/security/dependency review; E-45 catches Rust/sidecar/action-type drift at pre-commit and PR time; all new action types must pass through the append-only golden file.
- **[EXPLICIT; lines 390-420, 454-468] Planning/evaluation loop:** E-46 detects drift and replans; E-48 performs evidence-ranked research with citation/contradiction checks; E-49 records stable, cost, horizon, attribution, safety, awareness, and coverage metrics; E-54 hardens held-out corpus contamination and eval-awareness probes; E-55 permits practical SFT-on-rollouts only while the full RL ladder remains locked.
- **[EXPLICIT; lines 374-380, 222-228] Durable sessions:** E-25 compacts, forks, checkpoints, and reconstructs sessions; E-44 makes tool retries idempotent, compensates partial failure, rolls SessionDag over, and migrates old checkpoints.
- **[EXPLICIT; lines 8-12, 254-260] Evidence pattern:** A live behavior is expected to produce a typed SSB event and a durable/replayable artifact; E-29 maps those records to OTel spans and dashboard drawers.

### Integration conclusions

- **[INFERRED from E-5/E-1/E-3/E-4]** A fully built chat turn can personalize presentation from trusted facts while the archival-memory promotion path remains separately risk-gated. This is a composition inference; the summary does not specify one combined function or transaction.
- **[INFERRED from E-11/E-12/E-50/E-14/E-15/E-41]** A fully built product-generation run likely flows idea framing → archetype/framework selection → baby-step implementation in sandbox → checklist and browser gates → heterogeneous PR review. The ordering is consistent with dependencies/smokes, but no single line defines a canonical pipeline.
- **[INFERRED from E-35/E-18/E-20/E-24/E-37/E-39]** A fully hardened external tool call likely passes structured decoding, path/privacy routing, MCP decision/schema checks, prompt-injection controls, and cost ceilings. Exact guard order beyond explicitly stated local orderings is not defined in this summary.
- **[INFERRED from E-29/E-45 and the global proof contract]** New E action types should be journaled, schema/golden-file synchronized, exportable as OTel, and replayable. The summary explicitly requires the individual pieces but does not state one combined acceptance test.
- **[INFERRED from E-9/E-49/E-54/E-55]** Any future learning promotion should be judged on the private held-out/sharded corpus with stable multi-run metrics and attribution, rather than legacy SWE-bench Verified alone. The exact promotion formula remains in deeper plans.

## Constraints and invariants

### Cross-cutting hard rules

All items in this subsection are **[EXPLICIT]** from lines 532-544.

- Use blake3 only where a cryptographic/content hash is specified; E-37 explicitly rewrites SHA-256 to blake3.
- Do not use .unwrap() or .expect() in production Rust.
- FA2 is mandatory; a 7B sidecar without it must refuse to boot.
- Use model.generate() plus LogitsProcessor only; the SGLang adapter must preserve that chain.
- Block slopsquatting through E-11 PackageVerifier.
- Evict post-install state through E-15’s anti-slopsquatting handling; this behavior appears only in the cross-cutting rules and requires the E-15 original for detail.
- Correct the stale ort rule as E-34 specifies.
- Detect sycophancy through E-4.
- Surface every E-N report’s flagged Open questions.
- Add no new external dependency without confirmation. The summary specifically marks E-22, E-26, E-27, E-33, E-37, E-43, and E-51 as requiring explicit approval.
- Run CLI subprocesses in the Tier-C sandbox. E-15 implements it; E-22 and E-41 consume it.
- Keep E-47 behind its hard deferred ship token.
- Keep E-55’s RL ladder behind all three unlock conditions.
- Hard-avoid routine Lean loops under E-53.
- Keep E-23 positioning-only with README/AGENTS.md/UX copy and no new code surface.

### Session-local invariants and negative requirements

- **[EXPLICIT]** E-3: role visible and model hidden by default; model exposure is opt-in (lines 46-52).
- **[EXPLICIT]** E-4: a high-confidence answer must not flip merely because the user asserts it is wrong (lines 54-60).
- **[EXPLICIT]** E-6/E-10: a role deletion requires an iso-budget measured lift no greater than 1σ (lines 70-76, 102-108).
- **[EXPLICIT]** E-12: all 15 production checks pass before COMMITTING; assertion-only diffs block (lines 118-124).
- **[EXPLICIT]** E-13: at least one verifier must be non-LLM; mutation work is targeted, not workspace-wide (lines 126-132).
- **[EXPLICIT]** E-14: detect and skip Cloudflare/Turnstile; never bypass it (lines 134-140).
- **[EXPLICIT]** E-15: sandbox privilege order is NO_NEW_PRIVS → CAPBSET_DROP → seccomp; chroot is removed (lines 142-148).
- **[EXPLICIT]** E-16: extend, but do not rewrite, tier1/feedback_loop (lines 150-156).
- **[EXPLICIT]** E-17: success requires DEEP MRR > STANDARD MRR after blake3-verified ONNX staging (lines 158-164).
- **[EXPLICIT]** E-20: cloud routing requires redaction first; API-key-bearing prompts route locally (lines 182-188).
- **[EXPLICIT]** E-23: productivity language must say baseline, not “speedup proven” (lines 206-212).
- **[EXPLICIT]** E-24: a mutated pinned MCP description blocks the call (lines 214-220).
- **[EXPLICIT]** E-27: NF4 is training-only, not serving (lines 238-244).
- **[EXPLICIT]** E-28: FA3/FA4 are blocked on Ampere; YaRN is evaluation-gated (lines 246-252).
- **[EXPLICIT]** E-32: native 32K plus FITM is preferred; YaRN/StreamingLLM/RingAttention are skipped on this path (lines 278-284).
- **[EXPLICIT]** E-33: reduced-motion preferences and outlined-only chrome remain authoritative (lines 286-292).
- **[EXPLICIT]** E-39: the cap blocks at the cap; only one retry occurs before escalation (lines 334-340).
- **[EXPLICIT]** E-42: TIER-B grammar failure falls back to line-window chunking (lines 358-364).
- **[EXPLICIT]** E-45: SSB action-type evolution is append-only and locked by a golden file (lines 382-388).
- **[EXPLICIT]** E-47: no .py ships before explicit unlock; adding one fails CI (lines 398-404).
- **[EXPLICIT]** E-52: LSP is a verifier, not a retriever, and runs before AUDITING after edits (lines 438-444).
- **[EXPLICIT]** E-53: Verus gates COMMITTING in CI rather than every audit; routine Lean loops are rejected (lines 446-452).
- **[EXPLICIT]** E-55: DPO is hard-avoided; RL remains deferred pending three conditions (lines 462-468).

## Dependency, status, and deferred work

### Program-level status

- **[EXPLICIT; lines 8-12, 546-555]** There are 55 session slots. The summary classifies 53 as executable-now CPU/cloud work, one as entirely DEFERRED (E-47), and one as POSITIONING-only (E-23).
- **[EXPLICIT; lines 8-12, 546-555]** Estimated implementation effort is approximately 6–12 hours per session and 330–660 hours total, excluding deferred RunPod compute.
- **[EXPLICIT; lines 8-12, 546-555]** The evidence sweep claims 55 grounded reports, about 770 web fetches, about 75 cached PDFs, and URL/arXiv grounding, with conjecture labeled “Ungrounded:”.
- **[EXPLICIT; lines 557-559]** E-0_INDEX.md, not this summary, owns the complete per-E-N SCHEMA_VERSION bump-site list. Final cumulative version depends on ship order above the post-C/D 47+ baseline.

### Original topical waves

All are **[EXPLICIT]** from lines 14-26.

- Wave 1 UX + memory: E-1 through E-5.
- Wave 2 orchestration quality + critique: E-6 through E-10.
- Wave 3 idea-to-product: E-11 through E-15.
- Wave 4 validation + retrieval + routing: E-16 through E-20.
- Wave 5 peer landscape + Claude Code internals: E-21 through E-25.
- Wave 6 inference + kernels + observability + incidents: E-26 through E-30.
- Wave 7 model + UI + Rust ecosystem + structured output: E-31 through E-35.
- Wave 8 vision RAG + security + strategy + economics + graphs: E-36 through E-40.
- Wave 9 CI/CD + grammars + skills + durability + drift: E-41 through E-45.
- Wave 10 planning + fine-tune + deep research + metrics + SaaS: E-46 through E-50.
- Wave 11 edge + LSP + formal verification + eval + RL: E-51 through E-55.

### Recommended ship sequence

All are **[EXPLICIT]** from lines 561-611.

1. **Wave A, foundational/no-prerequisite:** E-5 memory; E-17 one-day RAG ONNX fix; E-2 one-day frame-paced SSE; E-34 anti-stale correction to unblock B-16; E-35 XGrammar substrate for E-18/E-19.
2. **Wave B, orchestration safety:** E-30 PreToolUse argv allowlist; E-15 Firecracker sandbox; E-37 data-plane prompt-injection defense; E-24 MCP hardening, which E-37’s landing path needs.
3. **Wave C, serving/inference:** E-27 AWQ+Marlin, sequenced with E-26/E-28; E-26 SGLang; E-28 mandatory FA2; E-32 32K context; E-51 MLX/llama.cpp.
4. **Wave D, production SaaS:** E-11 idea framing; E-12 production checklist; E-50 archetypes; E-14 browser E2E.
5. **Wave E, evaluation:** E-9 benchmark suites; E-13 TDD/mutants; E-49 metrics; E-54 corpus hardening.
6. **Wave F, reasoning/cross-family:** E-6 Adversary/iso-budget; E-19 confidence/PRM-lite; E-8 offline GEPA; E-7 Two-Gate DGM.
7. **Wave G, durability/drift:** E-44 idempotency/SAGA; E-45 cross-language snapshots; E-25 compaction.
8. **Wave H, parallel remainder:** E-3, E-4, E-10, E-16, E-18, E-20, E-21, E-22, E-23, E-29, E-31, E-33, E-36, E-38, E-39, E-40, E-41, E-42, E-43, E-46, E-48, E-52, E-53.
9. **Wave I, deferred/research:** E-47 fully deferred; E-55 RL ladder deferred while SFT-on-rollouts is the practical next rung.

### Complete B/C/D closure map

This is **[EXPLICIT]** from lines 472-530. “—” relations are omitted, but every nonempty relation is retained.

| E ID | B/phase surface | C surface | D surface |
|---|---|---|---|
| E-1 | — | — | D-1 ReasoningTemplate consumer |
| E-2 | B-14 | — | — |
| E-3 | B-37 | — | — |
| E-4 | — | C-7 | — |
| E-5 | Foundational | — | — |
| E-6 | — | — | D-13 supersede |
| E-7 | — | — | D-5, D-8, D-9 |
| E-8 | — | — | D-1 |
| E-9 | — | C-51 | — |
| E-10 | — | — | — |
| E-11 | B-40, B-41 | C-21, C-31, C-32 | — |
| E-12 | B-30 extension, B-37 | C-31, C-32 | — |
| E-13 | — | C-28, C-29 | D-3 |
| E-14 | B-24, B-42 | C-7, C-16 | — |
| E-15 | B-18 via C-11 | C-11 | — |
| E-16 | B-24 | — | — |
| E-17 | B-1, B-4 | C-8, C-17 | — |
| E-18 | B-7 | C-13 | — |
| E-19 | B-39 | — | D-2 stopgap, D-3 |
| E-20 | B-37 | C-5 | D-10 |
| E-21 | B-22 | — | — |
| E-22 | — | C-11 | — |
| E-23 | POSITIONING | — | — |
| E-24 | B-3 MCP bridge | C-35, C-39, C-42 | — |
| E-25 | Phase-3H handoff | — | — |
| E-26 | B-37 | — | — |
| E-27 | B-37 | — | — |
| E-28 | B-37 | — | — |
| E-29 | Phase-3B SSB | — | — |
| E-30 | — | C-44 | D-9 |
| E-31 | B-37 | — | — |
| E-32 | B-37 | — | — |
| E-33 | B-26 | C-6, C-7 | — |
| E-34 | CLAUDE.md amendment | — | — |
| E-35 | Substrate for E-18+ | — | — |
| E-36 | B-35, B-42 | — | — |
| E-37 | — | C-42, C-48 | — |
| E-38 | Phase-3B SSB | — | — |
| E-39 | — | C-5 | — |
| E-40 | — | C-24 | — |
| E-41 | — | C-40 | — |
| E-42 | B-30 PLANNED extension | — | — |
| E-43 | B-19/B-20 PLANNED extensions | — | — |
| E-44 | B-9 | — | — |
| E-45 | B-3 ts-rs | — | — |
| E-46 | B-22, B-40 | — | — |
| E-47 | DEFERRED | — | D-30 PLANNED/DEFERRED |
| E-48 | B-7, B-39 | C-48 | — |
| E-49 | B-23 | C-50, C-51 | — |
| E-50 | Consumes E-11/E-12 | — | — |
| E-51 | B-37 | — | — |
| E-52 | B-19/B-29 PLANNED extensions | — | — |
| E-53 | — | C-52 | D-3 |
| E-54 | — | — | Replaces D-6 9-gram; extends D-23 |
| E-55 | — | — | D-5; D-28 DEFERRED |

### Explicitly deferred, blocked, conditional, or open

- **E-3:** CUDA-graph warm pool is documented but Hopper-deferred (lines 46-52).
- **E-8:** requires a cloud Sonnet route and D-1 ReasoningTemplate (lines 86-92).
- **E-19:** PRM-lite is temporary until D-2 PPM ships (lines 174-180).
- **E-25:** the MCP dashboard surface is described as deferred (lines 222-228).
- **E-26:** vLLM V1 LogitsProcessor breakage remains an Open question (lines 230-236).
- **E-28:** YaRN is conditional on evaluation; FA3/FA4 are blocked on Ampere (lines 246-252).
- **E-31:** MoE remains a documented risky strategic bet (lines 270-276).
- **E-47:** all executable LoRA work is deferred; only SPEC.md and CI lock are in scope (lines 398-404).
- **E-51:** NPU is hard-deferred; webview/transformers.js/WebGPU are considerations, not committed implementation (lines 430-436).
- **E-53:** Lean loops are hard-avoided, not deferred implementation (lines 446-452).
- **E-55:** full RL is deferred until three unnamed unlock conditions; SFT-on-rollouts is the practical next rung (lines 462-468).

## Deeper-source manifest

### E-plan originals

All 56 source-plan files named by the summary are present at the paths below. Presence was checked; contents were intentionally not read in this summary-only pass because later agents own original-file expansion.

- **Index/schema source:** docs/thefinalglance/E-plan/E-0_INDEX.md
- **E-1:** docs/thefinalglance/E-plan/E-1_infinite_context_chat_illusion.md
- **E-2:** docs/thefinalglance/E-plan/E-2_local_llm_speed_tricks.md
- **E-3:** docs/thefinalglance/E-plan/E-3_model_handoff_ux.md
- **E-4:** docs/thefinalglance/E-plan/E-4_chat_ux_critique.md
- **E-5:** docs/thefinalglance/E-plan/E-5_user_modeling_and_memory.md
- **E-6:** docs/thefinalglance/E-plan/E-6_multi_agent_failure_modes.md
- **E-7:** docs/thefinalglance/E-plan/E-7_darwinian_self_improvement.md
- **E-8:** docs/thefinalglance/E-plan/E-8_self_evolving_prompts.md
- **E-9:** docs/thefinalglance/E-plan/E-9_agent_eval_benchmarks.md
- **E-10:** docs/thefinalglance/E-plan/E-10_critic_of_elai_design.md
- **E-11:** docs/thefinalglance/E-plan/E-11_idea_to_saas_prior_art.md
- **E-12:** docs/thefinalglance/E-plan/E-12_production_fullstack_codegen.md
- **E-13:** docs/thefinalglance/E-plan/E-13_tdd_verification_agents.md
- **E-14:** docs/thefinalglance/E-plan/E-14_browser_e2e_agents.md
- **E-15:** docs/thefinalglance/E-plan/E-15_sandboxed_exec.md
- **E-16:** docs/thefinalglance/E-plan/E-16_vlm_image_validation.md
- **E-17:** docs/thefinalglance/E-plan/E-17_rag_retrieval_quality.md
- **E-18:** docs/thefinalglance/E-plan/E-18_tool_use_reliability.md
- **E-19:** docs/thefinalglance/E-plan/E-19_reasoning_quality.md
- **E-20:** docs/thefinalglance/E-plan/E-20_local_cloud_routing.md
- **E-21:** docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md
- **E-22:** docs/thefinalglance/E-plan/E-22_github_trending_may2026.md
- **E-23:** docs/thefinalglance/E-plan/E-23_developer_discourse_2026.md
- **E-24:** docs/thefinalglance/E-plan/E-24_agent_protocols_mcp.md
- **E-25:** docs/thefinalglance/E-plan/E-25_claude_code_architecture.md
- **E-26:** docs/thefinalglance/E-plan/E-26_inference_engines_2026.md
- **E-27:** docs/thefinalglance/E-plan/E-27_quantization_2026.md
- **E-28:** docs/thefinalglance/E-plan/E-28_attention_kernels_2026.md
- **E-29:** docs/thefinalglance/E-plan/E-29_agentic_observability.md
- **E-30:** docs/thefinalglance/E-plan/E-30_public_agent_incidents.md
- **E-31:** docs/thefinalglance/E-plan/E-31_coding_model_leaderboard.md
- **E-32:** docs/thefinalglance/E-plan/E-32_context_window_extension.md
- **E-33:** docs/thefinalglance/E-plan/E-33_dashboard_ux_agentic.md
- **E-34:** docs/thefinalglance/E-plan/E-34_rust_ai_ecosystem_2026.md
- **E-35:** docs/thefinalglance/E-plan/E-35_structured_output_2026.md
- **E-36:** docs/thefinalglance/E-plan/E-36_document_vision_rag.md
- **E-37:** docs/thefinalglance/E-plan/E-37_prompt_injection_defense.md
- **E-38:** docs/thefinalglance/E-plan/E-38_ai_roadmap_2026_signals.md
- **E-39:** docs/thefinalglance/E-plan/E-39_cost_token_economics.md
- **E-40:** docs/thefinalglance/E-plan/E-40_symbol_graph_tools.md
- **E-41:** docs/thefinalglance/E-plan/E-41_ci_cd_pr_review.md
- **E-42:** docs/thefinalglance/E-plan/E-42_tree_sitter_grammars_2026.md
- **E-43:** docs/thefinalglance/E-plan/E-43_agent_skill_systems.md
- **E-44:** docs/thefinalglance/E-plan/E-44_checkpointing_durability.md
- **E-45:** docs/thefinalglance/E-plan/E-45_cross_language_drift.md
- **E-46:** docs/thefinalglance/E-plan/E-46_planning_decomposition.md
- **E-47:** docs/thefinalglance/E-plan/E-47_lora_finetuning.md
- **E-48:** docs/thefinalglance/E-plan/E-48_web_research_orchestrator.md
- **E-49:** docs/thefinalglance/E-plan/E-49_coding_agent_metrics.md
- **E-50:** docs/thefinalglance/E-plan/E-50_saas_architecture_patterns.md
- **E-51:** docs/thefinalglance/E-plan/E-51_edge_inference_and_mlx.md
- **E-52:** docs/thefinalglance/E-plan/E-52_lsp_integration.md
- **E-53:** docs/thefinalglance/E-plan/E-53_formal_verification_codegen.md
- **E-54:** docs/thefinalglance/E-plan/E-54_eval_corpus_design.md
- **E-55:** docs/thefinalglance/E-plan/E-55_agentic_rl_and_reward.md

### Referenced cross-plan and orientation sources

The summary explicitly depends on or points readers to these source families; they were not mined for new E features in this pass.

- docs/thefinalglance/ELAI-SPEC-OVERVIEW.md — design orientation.
- E-0_INDEX.md — full SCHEMA_VERSION bump sites and presumably E landing coordination (line 559).
- B-PLAN-SUMMARY.md and B IDs B-1, B-3, B-4, B-7/B-7b, B-9, B-14, B-18, B-19, B-20, B-22/B-22b, B-23, B-24, B-26, B-27, B-29, B-30, B-35, B-37, B-39, B-40, B-41, B-42.
- C IDs C-5, C-6, C-7, C-8, C-11, C-13, C-16, C-17, C-21, C-24, C-28, C-29, C-31, C-32, C-35, C-39, C-40, C-42, C-44, C-48, C-50, C-51, C-52.
- D IDs D-1, D-2, D-3, D-5, D-6, D-8, D-9, D-10, D-13, D-23, D-28, D-30.
- Core phases/surfaces Phase 2B, Phase 3B SSB, Phase 3H session handoff, Phase 3K RAG, and post-C/D schema baseline 47+.
- elai.map — current-code grounding for the wiring contracts and the SCIP/tree-sitter state.
- CLAUDE.md — inherited production rules, model/context tier configuration, and E-34 correction.
- DESIGN.md — E-33 outlined-only dashboard chrome.
- .claude/AGENTS.md — E-23 positioning/citation output.
- .claude/rules/commit-cadence.md — E-41 commit/attribution policy.
- .claude/rules/skill-import-policy.md — new E-43 policy.
- README.md and the sweep README/per-batch agent reports — named synthesis inputs/outputs, but the summary does not provide exact paths for the sweep README or per-batch reports.
- Approximately 75 cached PDFs and approximately 770 web fetches — evidence corpus claimed by the summary, without a manifest in this file.

### Summary-only gaps and ambiguity queue for original-plan agents

These are not invented missing features; they are explicit counts/references whose members or exact behavior are not recoverable from the summary.

- **Global:** Full SCHEMA_VERSION bump sites are delegated to E-0_INDEX.md; per-E report Open questions are said to exist but are not enumerated; the sweep README, per-batch agent reports, web-fetch manifest, and cached-PDF manifest are not path-resolved.
- **E-1:** Exact BeliefShift/CTIM-Rover promotion threshold and Open questions.
- **E-4:** The 14 CommandWindow UX rules and repair-resistance event/UI behavior beyond its counter.
- **E-6:** Names/definitions of all 14 MAST modes and exact iso-budget statistics.
- **E-7:** Two-Gate criteria, EST definition, MAP-Elites dimensions, trap fixtures, and Ed25519 chain schema.
- **E-8:** SHADOW admission/promotion thresholds and offline optimizer details.
- **E-9:** Exact benchmark attribution tuple schema and private-held-out composition.
- **E-10:** The 14 hostile punches, five ablations, and five preserved strengths.
- **E-11:** Per-level critic levels, scope threshold, PackageVerifier algorithm, and ANTI_SPIRAL_HARD_ABORT trigger.
- **E-12:** Seven framework manifests; all 15 checklist items (only named families are in the summary); FrameworkAdapter API; assertion-only-diff definition.
- **E-13:** Property-gate policy, cache key/lifetime, over-mock definition, and Fuzz4All CONCEPT criterion.
- **E-14:** Tier-2 VLM schema, asymmetric-veto direction, multi-context journey cases, and diff thresholds.
- **E-15:** AppContainer Windows trio, exact seccomp rules, allowed networking, post-install state eviction detail, and platform fallback policy.
- **E-17:** Exact model artifacts/URLs/hashes, query-path configuration, reranking weights, FITM calibration, and the 30-query/corpus specification.
- **E-18:** Hermes envelope schema, mask semantics, per-tool reliability formula, and fence implementation.
- **E-19:** L1 confidence threshold, PRM-lite formula, AB-MCTS integration, and Typed-CoT triple schema.
- **E-20:** Policy arbitration among xRouter/RouterArena/C3PO, privacy tiers, redaction rules, and leakage taxonomy.
- **E-21:** ACI audit specifics and the relationship between the retained SWE-bench-Verified subcommand and E-9’s “stop reporting” rule.
- **E-22:** Export payload schema/signature, deferred-schema flag semantics, 12-factor checklist, and benchmark method.
- **E-24:** Six contract-test cases, line-jumping sanitizer rules, per-tool token budgets, and gateway-rejection semantics.
- **E-25:** Six compaction tiers, seven anti-pattern guards, exact shell-templating syntax/escaping, reconstructability format, and deferred-MCP dashboard behavior.
- **E-26:** RuntimeAdapter API and vLLM V1 Open-question resolution.
- **E-27:** Hardware matrix, accuracy/throughput measurement protocol, and FP8 software-simulation behavior.
- **E-29:** Exact OTel columns, drawer field/layout definitions, and ssb_resume_from replay contract.
- **E-30:** Destructive-verb allowlist/regex, trust-dialog order, trust-label taxonomy, lineage collision definition, and 15 red-team cases.
- **E-31:** VRAM tier matrix entries, escalation policy, Qwen3 incompatibility handling, and MoE route.
- **E-33:** Three unnamed members of the five-component set and exact Wong palette/footer behavior.
- **E-34:** Exact dependency-edit versions/configuration beyond rmcp 1.7.0 and the B-summary correction text.
- **E-35:** INSTRUCTOR/CodeAct configuration, grammar-mask API, qwen_tools parser contract, and benchmark protocol.
- **E-36:** Model routing/installation details and extraction/verdict schema.
- **E-37:** Spotlighting format, classifier thresholds, egress allowlist policy, and Markdown-image handling.
- **E-38:** Audit-row schema and exact Article-13 gate requirements.
- **E-39:** Cap layers/defaults, override authorization/audit behavior, difficulty estimator, per-tool gates, and escalation target.
- **E-40:** SymbolGraphDSL grammar/ranking and citation-verification feedback contract.
- **E-41:** Semgrep AI rules, dependency verification, four-leg gate detail, cross-family selection, and merge/commit enforcement.
- **E-42:** TIER-A’s 15 languages, TIER-B members, ABI probe, ERROR-rate calculation, and injection queries.
- **E-43:** agentskills.io frontmatter fields, eval layout, signing/publish protocol, telemetry schema, and mcp-scan policy.
- **E-44:** Idempotency-key derivation/retention, compensation model, Continue-As-New threshold, DST scenarios, and schema migration detail.
- **E-45:** Snapshot/golden-file formats, sidecar schemars boundary, and exact strict-diff/pre-commit behavior.
- **E-46:** Ledger schemas, goal-drift thresholds, entropy stop, PRM scalar, and cascade conditions.
- **E-47:** Explicit unlock authority/process and documented ladder details.
- **E-48:** Search-tier policy, extractor fallback, reranker/pruner formulas, three thresholds, four CitationVerifier stages, contradiction schema, and attribution lock.
- **E-49:** Definitions/formulas for CTS, horizon, attribution, context rot, Cover@τ, STOP-Verified, and 30-day revert policy.
- **E-50:** Four seed TOML contents, six readiness checks, RLS emission details, scoped-token rules, and MoR/Stripe decision criteria.
- **E-51:** Adapter protocol/health configuration, exact platform detection, GGUF discovery, and what “consideration documented” commits to.
- **E-52:** multilspy language matrix, RAM safeguards, diagnostic severity gate, and PostToolUse hook policy.
- **E-53:** Supported proof obligations, tool invocation/timeout/fallback, CSQ Critical definition, CI cadence, and Dafny IR path.
- **E-54:** 13-gram normalization, shard/index schema, matched-pair construction, ORM meaning/schema, and perf-fixture method.
- **E-55:** The three RL unlock conditions, rollout selection/data hygiene, promotion criteria beyond the smoke threshold, and KTO/RLVR ladder details.

### Missing-source assessment

- **No missing E originals:** E-0_INDEX.md and all E-1 through E-55 filenames are present.
- **Unresolved by this summary:** Exact sweep README/per-batch-report paths and evidence-cache manifests are not named; this is a traceability gap in the summary, not proof that the files are absent.
- **Potential wording inconsistency:** The header says “Source files: 56” (E-0 + 55 plans), while the coverage section says “Files authored: 57” because it includes the summary. These figures are reconcilable, not contradictory.
- **Potential status ambiguity:** “53 executable now” coexists with many prerequisites marked PLANNED. Read it as nominal in-scope capability, not confirmation that all prerequisites are already implemented.
- **Potential benchmark tension:** E-9 stops reporting SWE-bench Verified, while E-21 adds/retains a SWE-bench-Verified subcommand. The likely distinction is headline reporting versus compatibility tooling, but that is **[INFERRED]** and must be resolved from E-9/E-21 originals.
