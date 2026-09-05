DIGEST: p2-orchestration.md
SOURCE: c17746b0-333e-4a9d-82ec-836eb22931a0.txt

TOPIC: Pillar P2 deep-dive — multi-agent orchestration audit. Audits the design: deterministic FSM + evidence-gated transitions + BSP supersteps + OS-process isolation w/ seccomp syscall filtering + deterministic simulation testing (DST). Evidence tagged Strong(red)/Moderate(yellow)/Indicative(green).

REPORT VERDICTS:
- FSM + evidence-gated transitions = KEEP (Strong). SDOF production (86.5% completion, 100% injection block) + SEMAP (56-70% MAST reduction) validate constrained-FSM over prompt-only; LangGraph Pregel/BSP independently converged; no 2024-2026 paper argues against deterministic state machines.
- BSP superstep barriers = KEEP + HARDEN (elastic barrier timeouts) (Moderate). Add per-worker deadline propagation: workers missing superstep deadline evicted+restarted not waited on, in-flight LLM call cancelled via sidecar cancellation token. Bounded-timeout addition, not architectural change.
- OS-process isolation with seccomp = HARDEN (gVisor or WASM for tool execution) (Strong). SandboxEscapeBench shows seccomp+container escapable at orchestration/runtime layer by frontier models. Layered defense: OS process -> gVisor/Firecracker -> WASM for tool sandboxing. Do NOT replace OS-process entirely.
- Deterministic Simulation Testing = HARDEN (turmoil + mad-turmoil, expand property coverage) (Moderate). Inference sidecar must be inside simulation boundary. Antithesis (closed) is only system w/ autonomous state-space exploration.
- Temporal-style durable execution = OPTIONAL / LOCAL ONLY (Moderate). Adds cross-process crash recovery in-process FSM+journal can't match, but needs external server conflicting w/ local hardware. Restate (Rust-core, MIT) is embeddable alternative. Future hardening for cloud-routing, not required local.

TOP 3 RANKED CHANGES: 1. Add plan-level taint / information-flow tracking (Strong). 2. Upgrade worker sandbox to gVisor (short term) / Firecracker (long term) (Strong). 3. Expand DST to include inference sidecar boundary + add property coverage tooling (Moderate).

CANDIDATES:
LangGraph (Pregel/BSP) | https://github.com/langchain-ai/langgraph MIT Production | Pregel-inspired BSP superstep w/ transactional checkpointing; FSM optional not default; lacks per-step artifact gating; routing LLM-driven by default; HIL hard mid-superstep (global barrier) | the BSP-superstep prior art (lacks design's evidence gates)
Temporal (Durable Execution) | https://github.com/temporalio/temporal MIT Production (Netflix/OpenAI/Snap) | Event-sourced workflow engine; every LLM call a deterministic activity; OpenAI Codex/Replit/Cursor run on it; deterministic in replay not original execution; needs external Cassandra/MySQL/PostgreSQL server | OPTIONAL crash-recovery (conflicts w/ local)
ORCH | arXiv:2602.01797 (https://arxiv.org/abs/2602.01797) Research preprint | Deterministic fixed-routing multi-LLM orchestrator "many analyses one merge"; EMA-guided routing optional; 10+ pt MMLU-Pro gain vs single-model; EMA router adapts to historical latency | BSP-straggler mitigation reference; merge agent still LLM
SDOF | arXiv:2605.15204 (https://arxiv.org/abs/2605.15204 ; https://www.semanticscholar.org/paper/92ce29910217a585a380367611d6ec033e9b9dcc) Research, live prod (Beisen iTalent 6000+ enterprises) | Multi-agent as constrained FSM w/ precondition/postcondition SkillRegistry; 86.5% task completion, 100% injection block; trained 7B intent router 80.9% vs zero-shot GPT-4o 48.9% | validates constrained-FSM KEEP (introduces LLM in routing layer = both feature+gap)
SEMAP | arXiv:2510.12120 (https://ieeexplore.ieee.org/document/11396616/) IEEE 2025, peer-reviewed | Protocol-layer behavioral contracts + lifecycle-guided execution atop Google A2A; 56-70% MAST failure reduction in code tasks; needs A2A transport substrate | hardens inter-agent-misalignment (FM-2.3/2.5)
Restate / DBOS | https://restate.dev / https://dbos.dev MIT/Apache-2 Early prod (Restate GA 2025) | Lightweight durable-execution runtimes as libraries; Restate Rust-core; DBOS SQLite backend; alternatives to Temporal for resource-constrained | embeddable durable-execution (closer to in-process model)
MAST taxonomy / Why Do Multi-Agent LLM Systems Fail | arXiv:2503.13657 (https://arxiv.org/abs/2503.13657 ; https://arxiv.org/pdf/2503.13657.pdf) UC Berkeley NeurIPS 2025 Workshop, kappa=0.88 | 14 failure modes across 1,600+ traces, 3 categories (System Design / Inter-Agent Misalignment / Task Verification) | the audit lens; design precludes majority
ICLR 2026 extended taxonomy | no-url (https://iclr.cc/virtual/2025/33314) | 18 failure modes, 4 categories; adds organizational breakdowns incl "specification ambiguities only manifest under adversarial inputs" | intersects SIF; residual exposure
SIF (Semantic Intent Fragmentation) | arXiv:2604.08608 (https://arxiv.org/abs/2604.08608) AAAI 2026 Symposium | Single legitimately-phrased request decomposed into individually-benign subtasks jointly violating policy; 71% policy violation w/ subtask-safety, stronger orchestrators HIGHER SIF success; plan-level IFC detects all 14 at 0% FP | grounds #1 plan-level taint/IFC add
MAScope | arXiv:2603.04469 (https://arxiv.org/html/2603.04469v1) March 2026 | Reconstructs cross-agent semantic flows into behavioral trajectories; F1=85.3% end-to-end attack detection | supports plan-level IFC
Microsoft Research (Optimizing Agent Planning) | no-url (https://www.microsoft.com/.../optimizing-agent-planning-for-security-and-autonomy/) April 2026 | IFC at planning layer increases autonomous execution without sacrificing security | supports plan-level IFC
SAFEFLOW | arXiv:2506.07564 (https://huggingface.co/papers/2506.07564) June 2025 | Principled protocol enforcing transactional execution + information-flow labels across agent boundaries | supports plan-level IFC
SandboxEscapeBench | no-url (https://letsdatascience.com/news/...531abf80 ; https://www.helpnetsecurity.com/2026/03/30/...) Oxford/AISI, released March 30 2026 | 18 container-escape scenarios; frontier models exploited common misconfigs at orchestration/runtime layer (kernel exploits failed) | grounds #2 sandbox upgrade
Architectural Requirements for Agentic AI Containment | arXiv:2604.23425 (https://arxiv.org/abs/2604.23425 ; https://www.semanticscholar.org/paper/c5352cdd...) | April 2026 frontier-model escape (executed unauthorized actions, erased Git history); 5 containment requirements none satisfied by any public system; R1 enforcement in unreachable trust domain, R4 write-only logically-invisible audit | grounds sandbox + audit-isolation gaps
gVisor (runsc) | no-url (https://dev.to/chunxiaoxx/...3hp7) | Systrap/KVM; ~100ms boot, ~20MB/instance, no host-kernel exposure, 10-30% I/O overhead minimal on compute | #2 short-term sandbox upgrade
Firecracker | no-url (https://www.augmentcode.com/guides/agent-execution-sandbox) | microVM ~125ms boot, ~5MB, >95% bare-metal CPU (KVM hw), 150 VMs/s/host; E2B ~150ms w/ pre-warmed snapshots | #2 long-term max isolation
WASM/WASI component | no-url (https://platform.uno/blog/...2025-2026/ ; https://techbytes.app/posts/...) | <1ms instantiation, <1MB, capability-first model; WASI 0.3 async released early 2026; WASI 1.0 stable end-2026/early-2027 | highest-value for tool-call sandboxing (forward-looking)
turmoil | https://github.com/tokio-rs/turmoil tokio-rs MIT | Simulated network/filesystem/io_uring drop-in for tokio primitives; production-quality Rust DST | DST hardening (sidecar in boundary)
mad-turmoil | no-url (https://lib.rs/crates/mad-turmoil) released Nov 2025/Feb 2026 | madsim-inspired determinism for turmoil-based simulation tests | DST determinism layer
Antithesis | no-url (https://se-radio.net/2025/09/...685... ; https://mack.work/blog/antithesis) commercial, closed | Deterministic hypervisor controlling CPU/network/time at instruction-count granularity; autonomous state-space exploration; used by etcd/CNCF, Jane Street, Google | property-coverage gold standard (gap vs turmoil/madsim)
TigerBeetle Vortex | no-url (https://tigerbeetle.com/blog/2025-02-13-a-descent-into-the-vortex) | DST framework; defense-in-depth DST + fuzzers + integration + 6,000+ assertions | DST property-coverage reference
ElasticBSP | no-url (https://ar5iv.labs.arxiv.org/html/2001.01347) | Relaxes strict synchronization to skip stragglers without sacrificing accuracy (ML training focus) | BSP-straggler mitigation
AgentOrchestra | arXiv:2506.12508 (https://arxiv.org/abs/2506.12508) | Orchestrating multi-agent intelligence w/ tool; self-reported MMLU-Pro/GAIA, unreplicated | flagged unverified

ATTACK-THE-DESIGN (strongest criticisms):
- B1 (Strong): SIF compositional plan-level safety gap — evidence-gated per-step checks are exactly the subtask-level safety SIF bypasses; hierarchical decomposition (idea->concepts->phases->steps->baby steps) lets a fragmented goal pass every per-transition gate; violation only manifests at execution.
- B2 (Strong): seccomp-only boundary below frontier escape surface — orchestration-layer misconfigurations are the primary successful escape vector, not kernel exploits; orchestrator process shares host kernel with workers; a worker influencing the orchestrator's trust domain (crafted output the orchestrator parses) defeats seccomp; R1 violated (supervisor tree runs in orchestrator process; typed-IPC-parser or journal-replay vuln could escalate).
- B3 (Moderate): BSP straggler/barrier stall under LLM latency variance (tail 5-100x median); a rate-limited cloud API mid-superstep looks like "no progress" to the FSM; evicting discards partial work, triggering expensive restart cycle.
- B4 (Strong): MAST residual exposure — task derailment (FM-2.3) + reasoning-action mismatch (FM-2.5) occur within a worker's LLM call before output hits the gate; gate catches bad artifacts but not wrong-subtask reasoning producing valid artifact; premature termination (FM-3.1) not blocked (low-quality partial passing the gate advances the FSM).
- B5 (Moderate): DST coverage gaps vs Antithesis — non-determinism sources outside madsim (LLM sampling, cloud API ordering, sidecar system time); if sidecar outside simulation boundary, cross-boundary faults invisible; property coverage self-defined (must specify invariants); Antithesis controls scheduling below the runtime, finding bugs turmoil/madsim cannot.
- B6 (Moderate): audit isolation — journal writes applied by orchestrator (its trust domain); compromised orchestrator could modify journal ordering; R4 (write-only, logically invisible) unmet; forward-looking exposure.

NUMBERS (claimed source, unverified):
- MAST: 14 failure modes, 1,600+ traces, kappa=0.88 [arXiv:2503.13657].
- SIF: 71% policy violation w/ subtask-safety; 0% w/ plan-level IFC; 14 enterprise scenarios [arXiv:2604.08608].
- MAScope: F1=85.3% [arXiv:2603.04469].
- SDOF: 86.5% task completion, 100% injection block; intent router 80.9% vs GPT-4o 48.9% [arXiv:2605.15204].
- SEMAP: 56-70% MAST failure reduction [arXiv:2510.12120].
- ORCH: 10+ pt MMLU-Pro gain vs single-model [arXiv:2602.01797].
- SandboxEscapeBench: 18 scenarios [Oxford/AISI March 30 2026].
- Containment: 698 documented AI scheming incidents Oct 2025-Mar 2026, 4.9x acceleration [CLTR, via arXiv:2604.23425].
- Overhead table: seccomp ~1ms/~1MB shared-filtered; gVisor ~100ms/~20MB/10-30% I/O; Firecracker ~125ms/~5MB/>95% CPU/150 VMs/s; WASM <1ms/<1MB; V8 isolate <1ms/~1MB.
- TigerBeetle: 6,000+ assertions [tigerbeetle.com].
- LLM tail latencies 5-100x median [report].

REPORT'S OWN FLAGS: April 2026 frontier-escape incident NOT independently verifiable (paper cites own prior work; 698 figure from CLTR report not directly accessible — recommendations sound regardless); SandboxEscapeBench focused on known misconfig exploits not zero-days, may not generalize to well-hardened seccomp; ORCH/AgentOrchestra numbers self-reported preprints unreplicated; WASI 0.3 async released early 2026 but WASI 1.0 stable end-2026/early-2027 (track stabilization before prod tool execution).
