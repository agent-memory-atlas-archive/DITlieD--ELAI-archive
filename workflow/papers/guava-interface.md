Algorithm: Guava — Effective and Universal Harness for Embodied Manipulation
ArXiv: 2606.18363
Paper: Guava: An Effective and Universal Harness for Embodied Manipulation (Liu, Li, Yao, Shi, Zhou, Huang, Huang, Mao; June 16, 2026)
Reliability: preprint (arXiv, not yet peer-reviewed at time of ingest)
Domain: embodied manipulation / agent harness design
ELAI lens: ignore gripper/grasp geometry; extract harness-design abstractions and small-model distillation method as model-agnostic agent-harness principles


DESIGN-SPACE STUDY (Section 3.1, Figure 2)
Three axes explored systematically using GPT-5.4 as oracle on six long-horizon Robosuite tasks:

Axis 1: Agent Workflow
Variants tried:
  one-shot / single-turn: model generates entire plan upfront, executes without replanning
  iterative ReAct loop: model issues one tool call per turn, receives observation feedback, replans

Winner: iterative ReAct loop
Delta: substantial improvement across all six tasks; paper states "significantly improving the reliability of frontier VLMs" (Section 3.1). Figure 2 left panel shows consistent win. Finding 4 confirms: "closed-loop execution is critical for long-horizon manipulation."
Key mechanism: after each action the model receives updated visual + textual state; it can detect grasp failures, object drift, and off-trajectory states and replan before the next step. One-shot generation has no recovery path.

Axis 2: Action Space
Variants tried:
  low-level geometric interface: single fused primitive move(x, y, z, roll, pitch, yaw, width) plus get_position, get_position_and_size, home_pose. Model must reason about absolute Cartesian poses, grasp configurations, gripper orientations.
  semantic action abstractions: 9-tool set (Table 1, Appendix B) where each tool encapsulates a skill and hides geometry behind named parameters

Winner: semantic action abstractions
Delta: Figure 2 middle panel shows consistent win. Appendix B states "low-level tool interface consistently results in worse performance" and "semantic-level action spaces reduce the low-level geometric and physical reasoning burden placed on the VLM."
Key mechanism: model focuses on WHAT to do (task decomposition, tool selection), not HOW (joint angles, clearance distances). Motion planning and low-level execution are delegated to external controllers behind the tool API.

Axis 3: Observation Space
Variants tried:
  visual only: RGB image of scene
  textual only: symbolic state description (gripper xyz, rpy, opening%)
  multimodal: visual + textual combined

Winner: multimodal
Delta: Figure 2 right panel shows multimodal higher performance than either alone. Section 3.1: "combining both modalities improves grounding and reduces ambiguity during sequential decision making."
Key mechanism: visual captures spatial relationships and object configurations; text provides compact symbolic gripper state and task progress. Neither alone is sufficient for robust embodied reasoning.

Bottom line from design study: ALL THREE AXES MUST BE COMBINED. The paper validates multimodal setting in iterative workflow as the configuration used for Guava.


THREE KEY INGREDIENTS (Section 3.1)

Ingredient 1: Iterative Perception-Reasoning-Action Loop (ReAct-style)
Structure per step:
  - model receives: (a) current RGB image, (b) textual state [gripper xyz, rpy, opening%], (c) conversation history of prior tool calls and their results
  - model outputs: <think>...</think> reasoning trace then exactly ONE tool call in JSON format <tool_call>{"name":"...", "arguments":{...}}</tool_call>
  - environment executes the tool, returns result (e.g., "grasped", "closed", position [x,y,z], or error)
  - loop repeats with updated observation

Termination condition: model outputs "Task complete" or "Task failed" after its <think> block. No fixed step limit cited; loop continues until terminal output or execution horizon.

Replan triggers (implicit): any tool result indicating failure ("closed" when "grasped" expected = missed grasp), unexpected position, or model reasoning detects off-trajectory state. Recovery behaviors include home_pose() followed by retry, or replanning target poses.

One tool call per turn is enforced: "In every response, think step-by-step inside <think></think> tags first, then call exactly one tool." (Appendix A, system prompt for data generation)

Ingredient 2: Semantic Action Abstractions (Table 1 + Appendix B)
"Semantic" means: tool names and parameters use task/object vocabulary, not robot kinematics.
Tool set (9 tools):
  grasp(object)                   -- perception-guided pick via SAM3 segmentation + 6-DoF grasp planner
  align(object, direction, clearance) -- direction in {top, left, right, front, back}; clearance in {small, medium, large}
  get_position(object)            -- returns 3D centroid in robot base frame
  get_position_and_size(object)   -- returns centroid + AABB dimensions
  move(x, y, z)                   -- Cartesian end-effector target (lower-level; available for fine-grained control)
  rotate(angle_deg, axis)         -- axis in {x, y, z}
  close_gripper()
  release()
  home_pose()                     -- recovery primitive; returns to safe default configuration

Abstraction level: highly abstracted (grasp, align) mixed with lower-level (move, rotate) to cover both coarse skill invocation and fine-grained adjustments. The mix is deliberate: high-level first, fall back to low-level when precise geometry is needed.
Parameters are semantic (direction names, clearance sizes) not numerical poses, so the model does not need to compute grasp orientations from first principles.

Ingredient 3: Multimodal Observations
Modalities provided per turn:
  visual: RGB image from fixed external camera (Intel RealSense D435 in real world)
  textual: current gripper state [position xyz, rotation rpy, opening percentage], tool call history, task prompt

Fusion method: concatenated into a single multi-turn conversation message (image + text in one turn). No separate fusion network; the VLM processes both natively.
Limitation: single fixed camera view; occlusion and perspective effects are cited as a limitation (Section 5, Limitations).


DISTILLATION / TRAINING PIPELINE (Sections 3.2, Appendix A, C)

Data collection (Appendix A):
  Teacher: GPT-5.4 operating under the Guava harness in Robosuite (MuJoCo-based simulation)
  Environment: Robosuite with scene randomization (pose, lighting, camera views)
  Raw trajectories: each trajectory = sequence of (image, textual state, <think> reasoning, tool call, tool result) tuples
  Data processing: (1) keep only successful completions, (2) filter invalid tool parameters and bad sim init, (3) manual inspection to remove off-task dialogue and excessive self-reflection, (4) de-duplicate similar trajectories
  Recovery data (Appendix A.3): from successful trajectories, inject perturbations (missed grasp, object dropping, wrong alignment), continue generation from perturbed state; same filtering applied; only valid counterfactuals added per task type
  Final dataset: 1,934 trajectories, 237 unique task prompts; 1,191 successful (62%) + 743 recovery (38%)
  Scale: fewer than 2K trajectories total; entirely from simulation

Base model: Qwen3.5-4B (Section 4.1). Vision encoder and aligner are frozen during training.

Training pipeline: two-stage, ORDER MATTERS:
  Stage 1: SFT
    dataset: all 1,934 trajectories (successful + recovery)
    objective: supervised fine-tuning on full conversation format including <think> traces and tool calls
    purpose: teach manipulation skills + basic recovery behaviors
    hyperparams (Appendix C.1): LR 1e-5, effective batch 32, 3 epochs
  Stage 2: GRPO (RL)
    dataset: only the two hardest long-horizon tasks (shell game + place all red objects in basket)
    reward: sparse task-success binary reward
    algorithm: Group Relative Policy Optimization, K=4 rollouts per prompt, KL penalty beta=0.04
    purpose: strengthen sequential planning, failure recovery, long-range task completion on hard tasks
    hyperparams: LR 5e-6, effective batch 12, 3 epochs
    rationale (Appendix C.2): applying RL to full suite too expensive (each rollout = multimodal inference + sim interaction); targeted RL on bottleneck tasks more compute-efficient

Infrastructure: 8 x NVIDIA H100 80GB, bfloat16, FlashAttention-2, DeepSpeed ZeRO-3, ms-swift framework

Headline result (Table 2):
  Guava-Agent-4B: 75.6% overall (13 tasks, 15 episodes each)
  GPT-5.4 with same harness: 70.2%
  CaP-Agent0 (one-shot code gen, frontier model): 62.7%
  Qwen3.5-4B base (no post-training): 23.1%
  => distilled 4B OUTPERFORMS frontier GPT-5.4 by 5.4pp overall

RL impact (Finding 3, Figure 6):
  shell game: SFT 6.7% -> SFT+RL 60.0% (+53.3pp)
  place all red objects: SFT 0.0% -> SFT+RL 93.3% (+93.3pp)
  Confirms: RL is load-bearing for long-horizon tasks; SFT alone insufficient

Real-world transfer (Finding 2, Figure 4):
  zero-shot sim-to-real (no additional real-world fine-tuning)
  Guava-Agent-4B: ID 86%, OOD 92% in real world
  GPT-5.4: lower on several tasks (push basket: 60% vs 40%)
  Key insight (Appendix E.3): "agentic formulation separates high-level semantic planning from low-level perception and control" -- this abstraction gap is WHY sim-to-real works without adaptation


MODEL-AGNOSTIC / UNIVERSAL CLAIM (Appendix D, Table 3)
Models tested with Guava harness (same tool set, same harness structure):
  GPT-5.4: 70.2% overall (primary frontier baseline)
  Gemini-3.1-Pro: 89.8% overall (HIGHEST of frontier models tested)
  Claude-Sonnet-4.6: 88.9% overall (near-frontier level)
  Qwen3.5-4B (no post-training): 23.1%
  Qwen3.5-2B: 2.2% overall -- FAILURE CASE

Finding: harness works well across GPT-5.4, Gemini-3.1-Pro, Claude-Sonnet-4.6. Qwen3.5-2B fails due to "poor instruction following and tool calling ability; frequently hallucinates wrong reasoning and invalid tool calls." (Appendix D text)
Universality claim is supported for sufficiently capable models (frontier or post-trained 4B). Below a capability threshold the harness cannot compensate.
Note: citation attributes in the HTML are mislabeled for Claude-Sonnet-4.6 and Qwen3.5-2B (paper's bib IDs are inconsistent), but the model names are clear in the table headers.


HARNESS VS END-TO-END (VLA) COMPARISON (Sections 1, 2, S2)
Paper contrasts harness approach vs Vision-Language-Action (VLA) end-to-end models:
  VLA pros: strong learned policies, broad manipulation skills
  VLA cons: requires large robot demonstration data (expensive, embodiment-specific, hard to scale); action heads are opaque (cannot inspect/repair plan); limited failure recovery
  Harness pros: modular (swap perception/control tools independently), leverages general VLM knowledge, explicit plan visible and repairable, sim-to-real via abstraction gap, data-efficient (< 2K trajectories vs thousands for VLA)
  Harness cons: cannot handle dexterous manipulation (current tool primitives); cannot correct tool-level errors (SAM3 segmentation failures); single-view occlusion limits; VLM spatial reasoning is the bottleneck

Key tradeoff reported: harness requires capable base model (>=4B well-trained) to follow structured tool-call format reliably; VLA is more robust to weak base model but needs more data and robot-specific training.


KNOWN FAILURE MODES AND LIMITATIONS (Section 5, Appendix E)
1. Spatial reasoning: push and arrange-by-order tasks have lowest success rates across all models. VLMs use coordinate heuristics (left=negative x-axis) instead of grounding direction in image. Appendix E.1.
2. Tool-level errors: invalid grasp proposals or incorrect SAM3 segmentations cannot be directly corrected; model can only detect failure and retry. Section 5.
3. Single-view occlusion: fixed camera with potential occlusion / perspective ambiguity. Section 5.
4. Dexterous manipulation: not supported by current action primitive set. Section 5.
5. Small model failure: models below ~4B (Qwen3.5-2B at 2.2% overall) hallucinate tool calls and cannot follow structured format. Appendix D.
6. SFT-only insufficient for hard long-horizon tasks: shell game 6.7%, place all red 0.0% without RL. Finding 3.
7. Simulation contact dynamics: imperfect sim physics can introduce failures not seen in real world; real-world eval remains essential. Appendix E.3.


IMPLEMENTATION PITFALLS (do NOT do this in analog implementations)
1. Do NOT use one-shot code generation or single-turn planning for multi-step tasks. The paper explicitly shows iterative loops are the critical ingredient. CaP-Agent0 (one-shot) scores 62.7% vs 75.6%.
2. Do NOT expose raw joint/Cartesian poses as the action interface to the LLM. Low-level geometric interface (move(x,y,z,roll,pitch,yaw,width)) consistently underperforms semantic tools. Appendix B.
3. Do NOT use visual-only or text-only observations. Both modalities are jointly necessary. Figure 2 right.
4. Do NOT skip the recovery trajectory data in SFT. Recovery trajectories are 38% of the dataset (743/1934) and directly teach failure recovery. Section 3.2 / Appendix A.3.
5. Do NOT apply RL uniformly across all tasks. Targeted RL on hard tasks only is more compute-efficient and effective. Appendix C.2.
6. Do NOT skip SFT before RL. Paper uses SFT first, then GRPO. Order matters; GRPO on randomly initialized weights would not converge. Section 3.2.
7. Do NOT assume a 2B model can follow the structured tool-call format. Capability threshold exists; sub-threshold models hallucinate and produce invalid calls. Appendix D.
8. Do NOT report sim results as sufficient for real-world claim. Paper notes sim contact dynamics differ and real-world eval is essential. Appendix E.3.
9. Do NOT count test-file callers as production wiring (ELAI internal rule, not from paper).


COMPLEXITY AND EFFICIENCY
Per-step complexity: one VLM forward pass (multimodal) + one tool execution. Token cost per step is moderate (image + conversation history + one tool call).
Efficiency finding (Appendix D, Figure 9): Guava-Agent-4B uses fewer tokens per episode than GPT-5.4 with same harness. Smaller distilled model is more token-efficient at inference.
Dataset size: 1,934 trajectories. Collection via GPT-5.4 API calls in simulation (cost not quantified in paper).
Training: 8 x H100-80GB; wall-clock time not stated.
Episode horizon: not stated explicitly; "execution horizon" is the termination condition. Tasks vary from single-step to multi-step long-horizon.


ELAI-SPECIFIC TRANSFER PRINCIPLES (ELAI lens, not robotics)
1. Iterative loop = ELAI's GRC observe->hypothesize->plan->act->verify cycle. One action per turn, update state, replan on failure. Direct analog.
2. Semantic action abstractions = ELAI's tool abstraction pattern: expose named skills (search, synthesize, audit) not raw API calls. Hides execution complexity behind semantic contracts.
3. Multimodal observations = ELAI's dual-channel context: code graph (visual/structured) + textual plan/memory. Neither alone is sufficient for accurate reasoning.
4. Recovery trajectory injection = ELAI's recovery behavior data: explicitly training on failure+recovery sequences, not just happy path. Analog: augmenting ELAI training with failed-then-recovered agent traces.
5. Targeted RL on hard tasks only = ELAI's selective RL: apply expensive RL budget to bottleneck tasks (long-horizon orchestration), not simple ones already solved by SFT.
6. Small distilled model beats frontier on in-distribution = ELAI's local model strategy: a task-specialized 4B can beat GPT-5.4 if the harness and training data are right. Validates Qwen2.5-Coder-7B direction.
7. Capability floor: models below a threshold (Qwen3.5-2B) cannot follow structured tool-call format. ELAI sidecar must be above this floor for reliable tool use. 3B-NF4 (ELAI's current local) is at the borderline; 7B is safer.
