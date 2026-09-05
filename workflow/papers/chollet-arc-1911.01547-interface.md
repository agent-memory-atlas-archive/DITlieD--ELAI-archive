Algorithm: On the Measure of Intelligence (Chollet ARC Framework)
ArXiv: 1911.01547
Paper: On the Measure of Intelligence, Francois Chollet, Google, November 5 2019

Scope for D-15: novelty-class labeling taxonomy grounded in Core Knowledge vs Acquired vs Derivative distinction


CENTRAL DEFINITION (Section II.2.1)

"The intelligence of a system is a measure of its skill-acquisition efficiency over a scope of tasks, with respect to priors, experience, and generalization difficulty."

Intelligence formula (sufficient case):
I_{IS,scope}^{theta_T} = Avg_{T in scope}[ omega_T * theta_T * Sum_{C in Cur_T^{theta_T}}[ P_C * GD_{IS,T,C}^{theta_T} / (P_{IS,T}^{theta_T} + E_{IS,T,C}^{theta_T}) ] ]

Schematic per task: Expectation[ skill * generalization / (priors + experience) ]


FORMAL COMPONENTS (Section II.2, grounded in Algorithmic Information Theory)

Algorithmic Complexity H(s): length of shortest program that outputs binary string s on a fixed universal Turing machine. Machine-independent to a constant.

Relative Algorithmic Complexity H(s1|s2): length of shortest program that, taking s2 as input, produces s1.

Generalization Difficulty GD_{T,C}^{theta} (system-centric):
  = H(Sol_T^{theta} | TrainSol_{T,C}^{opt}) / H(Sol_T^{theta})
  Value in [0, 1] by construction.
  0 = no adaptation needed (zero generalization); 1 = maximal edit distance from training solution.

Generalization Difficulty GD_{IS,T,C}^{theta} (developer-aware):
  = H(Sol_T^{theta} | TrainSol_{T,C}^{opt}, IS_{t=0}) / H(Sol_T^{theta})
  Accounts for system initial state (priors baked in at IS_{t=0}).

Priors P_{IS,T}^{theta}:
  = (H(Sol_T^{theta}) - H(Sol_T^{theta} | IS_{t=0})) / H(Sol_T^{theta})
  Value in [0, 1]. Fraction of solution complexity explained by the system's initial state.
  High priors = system starts close to the solution.

Experience E_{IS,T,C}^{theta} (per step t):
  E_{IS,T,t}^{theta} = H(Sol_T^{theta} | IS_t) - H(Sol_T^{theta} | IS_t, data_t)
  Total: E_{IS,T,C}^{theta} = (1/H(Sol_T^{theta})) * sum_t E_{IS,T,t}^{theta}
  Measures relevant, novel information received -- NOT raw curriculum size.
  Penalizes slow learners; does not penalize noisy or repetitive curricula.


GENERALIZATION SPECTRUM (Section I.3.2) -- D-15 taxonomy anchor

1. Local generalization (Robustness):
   "Adaptation to known unknowns within a single task or well-defined set of tasks."
   Handling new points from a known distribution after dense sampling.
   This is what machine learning has primarily addressed since the 1950s.

2. Broad generalization (Flexibility):
   "Adaptation to unknown unknowns across a broad category of related tasks."
   Handle situations not foreseeable by system creators, without further human intervention.
   No current AI system clearly belongs here (as of 2019).

3. Extreme generalization (Generality):
   "Adaptation to unknown unknowns across an unknown range of tasks and domains."
   Human-centric extreme generalization = "generality" in Chollet's usage.
   Requires both system-centric generalization (quick adaptation from little experience)
   AND developer-aware generalization (ability to handle situations never encountered by
   previous humans).


NOVELTY AS OUT-OF-DISTRIBUTION GENERALIZATION (Section II.3, III.1.1)

Key framing: ARC evaluation set tasks are explicitly assumed unknown to the developer.
"Focus on measuring developer-aware generalization, rather than task-specific skill,
by only featuring novel tasks in the evaluation set (assumed unknown to the developer
of a test-taker)."

Novelty in this framework = task or situation that falls outside the system's
training distribution AND outside the developer's anticipated scope.
This is developer-aware GD: the system must adapt beyond both its priors and its training.

Implication for D-15 novelty-class labeling:
  CORE_KNOWLEDGE class: The item requires only the four innate Core Knowledge priors.
    No training data needed beyond what any human would have innately.
  ACQUIRED class: The item requires knowledge beyond Core Knowledge priors --
    learned language, domain facts, cultural symbols, task-specific practice.
  DERIVATIVE class: The item is reducible to a combination of Core Knowledge operations
    or previously solved tasks (GD approaches 0; generalization difficulty is minimal).


CORE KNOWLEDGE PRIORS TAXONOMY (Section III.1.2 + III.1.3) -- D-15 canonical reference

Chollet grounds ARC in Spelke & Kinzler 2007 Core Knowledge theory [ref 85].
Four systems, evolutionarily ancient, largely shared across species:

1. Objectness and elementary physics (Objectness priors in ARC)
   - Object cohesion: objects move as continuous, connected, bounded wholes
   - Object persistence: objects do not cease to exist or materialize suddenly
   - Contact: objects do not act at a distance, cannot interpenetrate
   ARC implementation: parse grids into objects via color continuity or spatial
   contiguity; objects persist despite noise/occlusion; physical contact between objects.

2. Agentness and goal-directedness (Goal-directedness prior in ARC)
   - Inanimate vs agent distinction
   - Agents possess intentions, act to achieve goals
   - Agents show efficiency in goal-directed actions, act contingently and reciprocally
   ARC implementation: input/output grids modeled as start/end of intentional process;
   not strictly necessary but likely useful for ARC solving.

3. Natural numbers and elementary arithmetic (Numbers and Counting priors in ARC)
   - Innate abstract number representations for small numbers (all < ~10 in ARC)
   - Operations: addition, subtraction, comparison, sorting
   - Applicable across sensory modalities
   ARC implementation: counting, sorting, comparing object counts; addition/subtraction.

4. Elementary geometry and topology (Basic Geometry and Topology priors in ARC)
   - Notions of distance, orientation, in/out relationships
   - Spatial navigation in 2D and 3D
   ARC implementation: lines, rectangular shapes; symmetries, rotations, translations;
   upscaling/downscaling, elastic distortions; containing/being contained;
   drawing lines, connecting points, orthogonal projections; copying/repeating objects.

WHAT IS NOT A CORE KNOWLEDGE PRIOR (critical for D-15 ACQUIRED class):
   - Language, learned symbols (e.g. arrows)
   - Concepts requiring cultural learning (e.g. "cat", "dog")
   - Tasks requiring task-specific practice (e.g. chess)
   - Crystallized intelligence (domain knowledge accumulated through education/experience)


ARC BENCHMARK FORMAT (Section III.1.1)

Dataset: 400 training tasks + 400 public evaluation + 200 private evaluation = 1000 total
Task structure: N demonstration examples (avg 3.3) + 1-3 test examples
Grid format: 2D grid, 1x1 to 30x30, 10 unique symbols (colors, 0-9)
Success criterion: exact match on all test examples (binary, 0/1 per task)
Test-taker is allowed 3 trials per test example
Evaluation tasks are assumed disjoint from training tasks and unknown to the developer
Experience controlled: fixed number of training examples; tasks do not lend well to
  data augmentation.


FAILURE MODES AND KNOWN WEAKNESSES (Section III.2)

1. Generalization difficulty is not quantified per task. No formal GD score is provided
   for each ARC task; Chollet notes this as future work.

2. Test validity (predictive validity) not established via large psychometric studies.

3. Dataset size/diversity limited (1000 tasks); vulnerable to shortcut strategies that
   bypass genuine intelligence. Authors note competitions help expose shortcuts.

4. Binary evaluation format lacks granularity. Partial credit not captured.

5. Core Knowledge priors not exhaustively verified. "The exact nature of innate human
   prior knowledge is still an open problem."


IMPLEMENTATION PITFALLS FOR D-15 (do NOT do this)

Do NOT conflate skill with intelligence. Chollet explicitly: "skill is merely the output
  of the process of intelligence." A system that solves a task via memorization of
  training examples has skill, not intelligence.

Do NOT treat "novel" as merely "not in training set." Novelty in this framework is
  developer-aware: the task must not be anticipatable by the developer from known
  distributions. A new chess puzzle is not novel in this sense.

Do NOT label items as CORE_KNOWLEDGE if they require language, learned cultural symbols,
  or domain facts. Those are ACQUIRED regardless of apparent simplicity.

Do NOT assume Core Knowledge = easy. The difficulty of a task is orthogonal to whether
  it uses only Core Knowledge priors. A task can be hard and still only require objectness
  + geometry priors.

Do NOT ignore the distinction between system-centric GD and developer-aware GD.
  D-15 novelty labeling is about developer-aware GD (the right concept for benchmarking).

Do NOT label as DERIVATIVE a task that merely looks similar to a solved task.
  Similarity in appearance does not imply low GD. Use the formal GD criterion:
  how much must the shortest training-time solution be edited to produce the evaluation
  solution?

Do NOT allow acquired knowledge to sneak into CORE_KNOWLEDGE labels via indirect routes
  (e.g., numerical sequences that require culturally-learned counting beyond ~10,
  or shapes that require knowing named geometric vocabulary).


CRITICAL INVARIANTS (testable assertions for D-15 novelty labeling)

assert novelty_class in {"CORE_KNOWLEDGE", "ACQUIRED", "DERIVATIVE"}
# GD for DERIVATIVE must be near-zero: evaluation behavior reducible from training
assert not (novelty_class == "DERIVATIVE" and requires_new_abstraction)
# CORE_KNOWLEDGE items must not require language, cultural symbols, domain knowledge
assert not (novelty_class == "CORE_KNOWLEDGE" and requires_acquired_knowledge)
# ACQUIRED items require at least one knowledge type beyond the four Core Knowledge systems
assert not (novelty_class == "ACQUIRED" and set(knowledge_types).issubset(CORE_KNOWLEDGE_SYSTEMS))
# Priors in [0, 1]
assert 0.0 <= prior_score <= 1.0
# GD in [0, 1] by construction (Section II.2 formal definition)
assert 0.0 <= generalization_difficulty <= 1.0
# Intelligence formula denominator must be positive (priors + experience > 0)
assert (prior_score + experience_score) > 0


COMPLEXITY NOTES

Intelligence formula: not directly computable (requires Kolmogorov complexity, which
  is uncomputable). Paper acknowledges this and positions ARC as an approximation.
Approximate operationalization: use human performance as proxy for GD estimation.
ARC tasks: fixed-size grids (max 30x30 = 900 cells), 10 symbols, small demo sets.
No parallelism constraint stated for the benchmark itself.


RELATED PRIOR DEFINITIONS DISTINGUISHED

C-Test (Hernandez-Orallo): AIT-based, different formulation.
AIXI model (Hutter): AIT-based, focuses on reward maximization not skill-acquisition.
Universal Intelligence (Legg & Hutter): "agent ability to achieve goals in wide range
  of environments" -- does not separate priors from intelligence.
Chollet's key difference: explicitly separates and controls for priors + experience,
  enabling fair comparison between systems with different inductive biases.


KEY CITATIONS FROM PAPER
[85] Spelke & Kinzler. Core knowledge. Developmental Science, 2007. (Core Knowledge theory)
[40] Hernandez-Orallo. C-Test. (Prior AIT intelligence definition)
[43] Hutter. AIXI. (Prior AIT intelligence definition)
[54] Legg & Hutter. Universal Intelligence. (Prior AIT intelligence definition)
[15,14,30,55] Algorithmic Information Theory references.
