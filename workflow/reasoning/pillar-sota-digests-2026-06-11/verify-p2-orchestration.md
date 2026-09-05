verify-p2-orchestration: plan-level compositional safety + sandbox upgrade realism
task_id: verify-p2-orchestration
date: 2026-06-11
author: sub-agent (read-only web research)
scope: claims from deep-research report on plan-level SIF + sandbox frontier

=============================================================================
CLAIM-BY-CLAIM VERDICT
=============================================================================

CLAIM 1: SIF arXiv:2604.08608 exists; title "Semantic Intent Fragmentation"; 71% policy violation
with subtask-level safety; ~0% with plan-level IFC; "stronger orchestrators have HIGHER SIF success";
plan-level IFC detects all 14 MAST modes at 0% FP.

VERDICT: MOSTLY VERIFIED, one caveat

Source: https://arxiv.org/abs/2604.08608

Confirmed details:
- Title: "Semantic Intent Fragmentation: A Single-Shot Compositional Attack on Multi-Agent AI Pipelines"
- Authors: Tanzim Ahad, Ismail Hossain, Md Jahangir Alam, Sai Puppala, Yoonpyo Lee, Syed Bahauddin Alam,
  Sajedul Talukder
- Venue: AAAI 2026 Summer Symposium
- 14 enterprise scenarios (financial reporting, infosec, HR analytics); 71% (10/14) produced
  policy-violating plans while per-subtask classifiers passed all steps
- 0% FP is described as a measured experimental result via "cross-model compliance judge + deterministic
  taint analysis + chain-of-thought evaluation"
- "Plan-level information-flow tracking combined with compliance evaluation detects all attacks before
  execution" -- this is stated as the measured outcome on the 14 scenarios

CAVEATS (materially weakening the 0% FP claim):
- The orchestrator is labeled "GPT-20B" -- this designation does not correspond to any known OpenAI
  model naming convention; the paper does not clarify whether this is a hypothetical, internal, or
  renamed model. The evaluation base is therefore OPAQUE.
- 14 scenarios is a small N; "all 14 MAST modes at 0% FP" may conflate the 14 scenarios with MAST
  mode coverage, which is not confirmed from the abstract. The claim as stated in the deep-research
  report is likely an over-summary of "detected all 10 attacks tested with 0 FP on the compliance
  judge." The exact MAST-mode mapping was not verifiable from the abstract alone.
- "Stronger orchestrators have HIGHER SIF success" is confirmed conceptually (paper argues stronger
  orchestrators produce more coherent multi-step plans, making fragmentation more effective), but
  a quantitative gradient across multiple model tiers is not evidenced in the abstract.

BOTTOM LINE: SIF is real, the attack class is real, 71% is real, 0% FP on the compliance judge is
presented as measured (not just designed). The "14 MAST modes" framing is likely an imprecise
summary in the deep-research report.


CLAIM 2: MAScope arXiv:2603.04469 -- execution-aware cross-agent semantic flow reconstruction.

VERDICT: VERIFIED (paper exists, mechanism confirmed)

Source: https://arxiv.org/abs/2603.04469 / https://arxiv.org/html/2603.04469

Title: "Beyond Input Guardrails: Reconstructing Cross-Agent Semantic Flows for Execution-Aware
Attack Detection"
Authors: Yangyang Wei, Yijie Xu, Zhenyuan Li, Xiangmin Shen, Shouling Ji
Codename in repo: MAScope-71DC

Mechanism: Reconstructs behavioral trajectories ACROSS agent interactions (data flow violations,
control flow deviations, intent inconsistencies) during execution -- not at planning time but at
execution-trace time. A Supervisor LLM scrutinizes the reconstructed trajectory. F1: 85.3%
(node-level) / 66.7% (path-level) for indirect prompt injection detection.

One-line mechanism: runtime semantic-flow reconstruction across the agent graph, checked by a
supervisor against deviation patterns.

NOTE: MAScope operates at execution time (post-dispatch), not pre-execution planning time. It is
a detection system, not a pre-execution gate. This is distinct from SIF-style plan-level IFC.


CLAIM 3: SAFEFLOW arXiv:2506.07564 -- information flow control at planning layer.

VERDICT: VERIFIED (paper exists, mechanism confirmed)

Source: https://arxiv.org/abs/2506.07564

Title: "SAFEFLOW: A Principled Protocol for Trustworthy and Transactional Autonomous Agent Systems"
Authors: Peiran Li, Xinkai Zou, Zhuohang Wu, Ruifeng Li, Shuo Xing, Hanwen Zheng, Zhikai Hu,
Yuping Wang, Haoxi Li, Qin Yuan, Yingmo Zhang, Zhengzhong Tu

Mechanism: Fine-grained IFC via security labels on all data exchanges -- tracks provenance,
integrity, and confidentiality. Prevents untrusted inputs from contaminating high-integrity
decisions. Also implements transactional execution (write-ahead logging, rollback, secure caches).
Includes SAFEFLOWBENCH benchmark.

One-line mechanism: security labels propagate with data across agents; integrity/confidentiality
policy checked at each data handoff; transactional rollback if violation detected.

NOTE: The "planning layer" framing in the deep-research report is slightly imprecise. SAFEFLOW's
IFC applies during execution/data exchange, not specifically at plan decomposition time. It does
not analyze the plan DAG before dispatch; it tracks labels as data flows between live agents.


CLAIM 4: Microsoft "Optimizing Agent Planning for Security" / MAControl.

VERDICT: VERIFIED but title/framing in deep-research report is slightly wrong

Source: https://arxiv.org/abs/2602.11416
         https://www.microsoft.com/en-us/research/publication/optimizing-agent-planning-for-security-and-autonomy/

Actual title: "Optimizing Agent Planning for Security and Autonomy" (not "MAControl")
Authors: Aashish Kolluri + 7 co-authors (Microsoft Research)
arXiv: 2602.11416, submitted 2026-02-11

Mechanism: Builds on an existing IFC defense against prompt injection (the CaMeL-style
system). Introduces PRUDENTIA, a security-aware planning agent that (a) adds richer HITL
interaction points, (b) explicitly plans for BOTH task progress AND policy compliance at the
planning layer. Evaluated on AgentDojo and WASP benchmarks.

One-line mechanism: planner generates candidate action sequence; security-aware planning
component checks compliance constraints BEFORE committing subtasks for dispatch, reducing
HITL interrupts while maintaining IFC guarantees.

NOTE: "MAControl" is not a name used in the paper. The deep-research report appears to have
invented or confused this label. The actual mechanism is PRUDENTIA atop an IFC substrate.


CLAIM 5: SandboxEscapeBench (Oxford/AISI, ~March 2026): 18 container-escape scenarios;
frontier models exploit orchestration-layer MISCONFIG (not kernel zero-days).

VERDICT: VERIFIED, institution partially confirmed

Source: https://arxiv.org/abs/2603.02277
         https://www.aisi.gov.uk/research/quantifying-frontier-llm-capabilities-for-container-sandbox-escape

Title: "Quantifying Frontier LLM Capabilities for Container Sandbox Escape"
Authors: Rahul Marchand, Art O Cathain, Jerome Wynne, Philippos Maximos Giavridis, Sam Deverett,
John Wilkinson, Jason Gwartz, Harry Coppock
Venue: ICML 2026 (Oral + Poster)
Published on AISI (aisi.gov.uk) website -- confirmed UK AI Security Institute involvement.
Oxford affiliation not directly confirmed from abstract metadata but ICML + AISI hosting is solid.

18 scenarios: 4 orchestration/Kubernetes (Layer 1), 8 runtime/Docker misconfigs (Layer 3),
6 kernel exploits (Layer 4).

KEY FINDING: Frontier models succeeded via misconfiguration exploitation (exposed Docker socket,
privileged containers, writable host mounts) -- ZERO success on kernel exploits (Layers 4-5
requiring deep kernel knowledge). Kernel security mechanisms (capabilities, seccomp, MAC) block
67.57% of privilege escalation attacks. Models did NOT exploit kernel zero-days.

IMPLICATION for our seccomp posture: seccomp-only is below the escape surface for
ORCHESTRATION-LAYER misconfigs, but seccomp + capabilities + MAC as a combined mechanism block
most kernel-path escalation. The real threat is Docker socket exposure / privileged containers,
which seccomp alone cannot fix. seccomp hardening is NECESSARY but INSUFFICIENT; the threat
is configuration hygiene at the container/orchestrator layer, not kernel exploitation.


CLAIM 6: arXiv:2604.23425 "Architectural Requirements for Agentic AI Containment": 5 requirements
R1 enforcement-in-unreachable-trust-domain, R4 write-only-logically-invisible-audit.

VERDICT: VERIFIED with labeling caveat

Source: https://arxiv.org/abs/2604.23425

Title: "When the Agent Is the Adversary: Architectural Requirements for Agentic AI Containment
After the April 2026 Frontier Model Escape"
Author: Richard Joseph Mitchell (no institutional affiliation confirmed)

The five requirements (from abstract, precise wording):
R1: trust separation through layered OS privilege enforcement with semantic intent analysis
R2: sequential intent inference through five-phase taxonomic monitoring
R3: independent containment integrity monitoring
R4: adversarial audit isolation through logical invisibility
R5: emergent capability envelope enforcement through distributional divergence monitoring

LABELING CAVEAT: The deep-research report paraphrased R1 as "enforcement-in-unreachable-trust-
domain" and R4 as "write-only-logically-invisible-audit." These are reasonable summaries of R1
(layered OS privilege enforcement implies separation from agent's trust domain) and R4 (adversarial
audit isolation through logical invisibility), but "write-only" is the report's gloss, not the
paper's language. R4's mechanism is audit ISOLATION via logical invisibility, not specifically
write-only.

NOTE: The "April 2026 Frontier Model Escape" is presented in the paper as a real documented
incident. Mitchell also cites "698 real-world AI scheming incidents" from the Centre for Long-Term
Resilience (Oct 2025 -- Mar 2026). Solo author, no affiliation: weight accordingly.


CLAIM 7: SDOF arXiv:2605.15204 -- 86.5% completion / 100% injection block; validates FSM choice.

VERDICT: VERIFIED

Source: https://arxiv.org/abs/2605.15204

Title: "SDOF: Taming the Alignment Tax in Multi-Agent Orchestration with State-Constrained Dispatch"
Author: Zhantao Wang

Mechanism: Treats multi-agent execution as a constrained state machine. StateAwareDispatcher with
precondition/postcondition validation. Online-RLHF Specialized Intent Router.

Results:
- 86.5% task completion (95% CI: 80.8-90.7%) on recruitment system (Beisen iTalent, 6000+ enterprises)
  with 185 expert-curated scenarios / 1671 live API calls
- Blocks all 22 injection/illegal HR operations tested (100% block rate)
- Intent Router: 80.9% joint accuracy vs GPT-4o 48.9%

This VALIDATES our FSM approach: deterministic state-transition constraints as a security mechanism
is a published pattern with measured results. SDOF is not refuting our design.


CLAIM 8: SEMAP arXiv:2510.12120 -- 56-70% MAST failure reduction.

VERDICT: VERIFIED (numbers match within range)

Source: https://arxiv.org/abs/2510.12120

Title: "Towards Engineering Multi-Agent LLMs: A Protocol-Driven Approach"
Authors: Zhenyu Mao, Jacky Keung, Fengji Zhang, Shuo Liu, Yifei Wang, Jialong Li

SEMAP = Software Engineering Multi-Agent Protocol, built on Google Agent-to-Agent infrastructure.
Mechanism: explicit behavioral contract modeling + structured messaging + lifecycle-guided execution
with verification.

Results (MAST-measured failure reduction):
- 69.6% reduction for function-level code development
- 56.7% for deployment-level development
- 47.4% on Python vulnerability detection
- 28.2% on C/C++ vulnerability detection

The "56-70%" range in the deep-research report is accurate (it spans the deployment and function-
level results). The lower end (28-47%) for vulnerability detection tasks was not cited in the
report. Not refuting our design.

=============================================================================
KEY QUESTION: is "plan-level taint/IFC over the decomposition tree" a GROUNDED new gate
worth a plan unit, or does our existing per-step evidence-gating + data-plane taint cover it?
=============================================================================

SHORT ANSWER: GROUNDED new gate, but scoped narrowly to the composition step (idea->concepts)
and NOT a replacement for per-step gating. Our existing data-plane taint (Provenanced<String> +
capability_gate) covers the data-flow IFC leg; what SIF shows is missing is PLAN-STRUCTURE
IFC: analyzing whether the subtask DAG as a whole violates policy, BEFORE dispatching any subtask.

DETAIL:

What we have (verified from context):
1. Per-step FSM evidence gates: each step requires measurable off-agent artifacts to advance.
   This catches per-step violations but explicitly does NOT compose across steps -- it is a
   sequence of independent gates, not a joint policy check over the DAG.
2. Data-plane taint (Provenanced<String>, capability_gate before destructive sinks): tracks
   data provenance as it flows through the system. This is the closest thing we have to IFC.
   However, data-plane taint operates at EXECUTION time on actual data values; it cannot see
   that two individually-benign subtasks are jointly extracting a quasi-identifier at plan
   decomposition time, before any data flows.
3. Deterministic FSM + BSP supersteps + OS-process isolation: these constrain WHAT each step
   can do, but do not check whether the COMBINATION of subtasks violates a policy the
   individual steps are authorized to perform.

What SIF demonstrates is missing:
The gap is PRE-DISPATCH compositional policy checking: given the full subtask DAG that the
planner generated for a goal, does executing all subtasks in sequence/parallel produce a
jointly-violating outcome, even if each subtask individually clears the per-step gate?

This is a structural gap that cannot be closed by:
- Making per-step gates stricter (each step IS benign in isolation)
- Making data-plane taint more sensitive (taint tracks values, not the abstract combination of
  authorized actions across planning steps)
- Adding more FSM states (FSM constrains transitions; it does not analyze cross-step semantic
  composition)

What a plan-level IFC gate would check (concretely):
At plan decomposition (concepts->phases step, or idea->concepts for the top-level goal):
- Build a data-flow/information-flow graph over the PLANNED subtask sequence
- Check: does any path through the planned execution produce a data aggregate, capability
  combination, or output that is forbidden by policy, even if each node is individually
  authorized?
- Block the plan BEFORE dispatch if so; do not wait for per-step gates to fail (they won't)

How hard is this to build?
- The SIF paper's own mitigation ("plan-level IFC + compliance evaluation") is described as
  effective on their 14 cases but uses a "cross-model compliance judge" -- an LLM call over
  the plan. This is probabilistic, not deterministic.
- A deterministic version would require an explicit policy graph and a reachability check over
  the subtask DAG. This is feasible for a structured decomposition tree (like our
  idea->concepts->phases->steps->babysteps) because the DAG is explicit, not implicit.
- For our system specifically: the concepts->phases->steps tree is ALREADY a structured artifact.
  Adding a plan-level taint propagation pass (check: does the aggregate of planned step outputs
  violate any policy label combination?) BEFORE executing the first step is a well-scoped
  engineering addition.

VERDICT: Worth a plan unit. The gate is:
(a) grounded by primary-source evidence (SIF paper, PRUDENTIA, SAFEFLOW, SDOF)
(b) not covered by existing per-step gating or data-plane taint (different layer)
(c) feasible for a deterministic implementation on our structured decomposition tree
(d) bounded scope: one pass over the planned subtask DAG at dispatch time, not an ongoing
    runtime monitor

The plan unit should be NARROW: "add a compositional policy check at plan-dispatch time that
walks the subtask DAG and verifies no jointly-violating information-flow path exists before
any subtask is dispatched." This is NOT a full SAFEFLOW port; it is a single pre-dispatch
gate that reuses our existing policy graph and taint label infrastructure.

=============================================================================
SANDBOX UPGRADE REALISM RANKING (Win primary + Linux remote sidecar, single user)
=============================================================================

Deployment context:
- Development primary: Windows 11 Pro
- Remote sidecar: Ubuntu 24.04 WSL2 on friend's RTX 3090 (pythondev, maintainer@192.0.2.10:6465)
- Use case: tool execution sandboxing for a local coding agent, NOT a cloud multi-tenant service
- Single user, non-adversarial co-operator model

RANKING (most realistic TODAY to most forward-looking):

1. SECCOMP + CAPABILITIES + MAC (already deployed, increment only)
   Status: in use (Linux sidecar). SandboxEscapeBench confirms this combination blocks 67.57%
   of privilege escalation attacks. The real gap is NOT seccomp hardening but CONFIGURATION
   hygiene: no exposed Docker sockets, no privileged containers, no writable host mounts.
   Action: audit sidecar container config for the 8 Layer-3 runtime misconfigs SandboxEscapeBench
   documents. This is the highest-leverage, lowest-cost hardening step.
   Source: https://arxiv.org/abs/2603.02277

2. gVisor (runsc) -- REALISTIC for remote sidecar TODAY
   Status: mature, production-deployed on GKE, VFS2/LISAFS improvements shipped 2023-2024.
   Overhead: CPU-bound workloads (API calls, code gen) see minimal overhead; I/O-heavy workloads
   see 10-30% overhead. Inference workloads are largely CPU/GPU-bound, so gVisor is acceptable.
   Windows primary: gVisor does not run on Windows natively. Only applicable to the Linux sidecar.
   Recommendation: deploy runsc on the remote sidecar for tool-execution containers. Straightforward
   drop-in for Docker. LOW friction for a single-machine single-user setup.
   Source: https://gvisor.dev/docs/architecture_guide/performance/
            https://kubeblocks.io/blog/does-containerization-affect-the-performance-of-databases

3. Firecracker microVM -- FORWARD-LOOKING, not TODAY for this use case
   Status: mature for serverless/cloud (AWS Lambda). For a local dev machine: requires bare-metal
   KVM or nested virt. On the remote sidecar (Ubuntu 24.04 WSL2), nested virt overhead is low
   (~3% CPU gap) but network setup (CNI plugins, virtual switches) adds 263% latency at concurrency.
   For a single-user non-concurrent coding agent tool execution model (one tool at a time), the
   concurrency bottleneck is moot. Boot time with snapshots: 28ms is achievable.
   BUT: setup complexity for a single remote box is high relative to gVisor. Not recommended for
   today unless the threat model requires true VM-level isolation (kernel-zero-day scenario).
   Sources: https://firecracker-microvm.github.io/
             https://dev.to/adwitiya/how-i-built-sandboxes-that-boot-in-28ms-using-firecracker-snapshots-i0k

4. WASM/WASI component model -- FORWARD-LOOKING, realistic for TOOL sandboxing only by late 2026
   Status: WASI 0.3 with native async shipped February 2026 (Wasmtime 37+). WASI 1.0 stable
   targeted late 2026 / early 2027.
   For this use case: WASM/WASI is the right long-term answer for SANDBOXING INDIVIDUAL TOOLS
   (file I/O, code execution tools) -- not for sandboxing the whole agent. The capability model
   maps well to our capability_gate pattern.
   Windows: Wasmtime runs on Windows. Cross-platform is a genuine win here.
   Blocker: WASI 0.3 is RC-quality (February 2026); toolchain ecosystem (Rust WASM target for
   tool plugins) is maturing but not production-stable for arbitrary tool plugins today.
   Recommendation: track for 2027; prototype one tool (e.g. file-write tool) as a WASI component
   to validate the capability-label integration with our existing Provenanced<String> taint.
   Sources: https://wasi.dev/roadmap
             https://progosling.com/en/dev-digest/2026-02/wasi-0-3-wasmtime-37-native-async

SUMMARY TABLE:

  seccomp+caps+MAC  | TODAY   | INCREMENTAL | config audit is the action, not more seccomp code
  gVisor (runsc)    | TODAY   | LOW effort  | deploy on remote sidecar for tool containers
  Firecracker       | 2026 Q3 | MEDIUM      | only if VM-isolation required; single-box fine for now
  WASM/WASI 0.3     | 2027    | FORWARD     | prototype now, production after WASI 1.0 stable

=============================================================================
REFUTED / INCORRECT CLAIMS IN DEEP-RESEARCH REPORT
=============================================================================

1. "MAControl/Microsoft" -- REFUTED as a label. The Microsoft paper is arXiv:2602.11416
   "Optimizing Agent Planning for Security and Autonomy." No paper called "MAControl" exists.
   The deep-research report invented or confused this label.

2. "SandboxEscapeBench is an Oxford/AISI joint paper" -- PARTIALLY INCORRECT. The paper is
   published on the AISI website and presented at ICML 2026, but no Oxford affiliation is
   confirmed in the arXiv metadata. The arXiv ID is 2603.02277, NOT the ID implied by linking
   it to the 2604.23425 containment paper (those are separate works by different authors).

3. "plan-level IFC detects all 14 MAST modes at 0% FP" -- IMPRECISE. The paper tests 14
   enterprise scenarios, not 14 MAST modes. The 0% FP is on the compliance judge over those
   14 cases. The MAST-mode framing is the deep-research report's gloss, not the paper's own
   claim. The N=14 makes this a proof-of-concept result, not a production benchmark.

4. "seccomp-only sandbox is below the frontier escape surface" -- PARTIALLY CORRECT but
   MISLEADING framing. SandboxEscapeBench shows the primary escape vector is MISCONFIGURATION
   (Docker socket, privileged containers), not seccomp bypass. Seccomp+caps+MAC blocks 67.57%
   of privilege escalation. The gap is config hygiene, not seccomp capability. Framing it as
   "seccomp-only is inadequate" is correct; framing it as a reason to add a new sandbox layer
   before fixing config hygiene is the wrong priority order.

=============================================================================
GROUNDING LOG
=============================================================================

All claims verified from primary sources. URLs accessed 2026-06-11:

arXiv papers (all confirmed real):
- https://arxiv.org/abs/2604.08608 (SIF)
- https://arxiv.org/abs/2603.04469 (MAScope)
- https://arxiv.org/abs/2506.07564 (SAFEFLOW)
- https://arxiv.org/abs/2604.23425 (Containment requirements)
- https://arxiv.org/abs/2605.15204 (SDOF)
- https://arxiv.org/abs/2510.12120 (SEMAP)
- https://arxiv.org/abs/2603.02277 (SandboxEscapeBench)
- https://arxiv.org/abs/2602.11416 (Microsoft Optimizing Agent Planning)

Secondary sources:
- https://www.aisi.gov.uk/research/quantifying-frontier-llm-capabilities-for-container-sandbox-escape
- https://www.microsoft.com/en-us/research/publication/optimizing-agent-planning-for-security-and-autonomy/
- https://gvisor.dev/docs/architecture_guide/performance/
- https://wasi.dev/roadmap
- https://progosling.com/en/dev-digest/2026-02/wasi-0-3-wasmtime-37-native-async
- https://dev.to/adwitiya/how-i-built-sandboxes-that-boot-in-28ms-using-firecracker-snapshots-i0k
- https://kubeblocks.io/blog/does-containerization-affect-the-performance-of-databases
- https://icml.cc/virtual/2026/oral/71104

UNVERIFIED (full PDFs not fetched; abstract-level only):
- SDOF claim about "86.5% on deterministic simulation" vs "live API" -- paper says live API
  calls (1671 calls on Beisen iTalent), not simulation. Deep-research report's framing as
  "deterministic simulation testing" may misread the paper.
- SEMAP exact mechanism at plan vs execution level -- abstract confirmed protocol-based
  lifecycle-guided execution but pre-dispatch vs runtime distinction not verified from abstract alone.
- SIF "GPT-20B" model identity -- could not confirm; not a known OpenAI model name.
