Algorithm: BeliefShift Benchmark (temporal belief consistency and opinion drift evaluation)
ArXiv: 2603.23848
Paper: BeliefShift: Benchmarking Temporal Belief Consistency and Opinion Drift in LLM Agents

AUTHORS: Praveen Kumar Myakala, Manan Agrawal, Rahul Manche
SUBMITTED: March 25, 2026

VERIFICATION STATUS: VERIFIED
The arXiv ID resolves exactly to "BeliefShift: Benchmarking Temporal Belief Consistency and
Opinion Drift in LLM Agents." Confirmed via arxiv.org/abs/2603.23848.

CLAIMED USE IN H-30: "BeliefShift, a belief-revision protocol (present fact X, then present
superseding fact X', measure whether system uses X' not X). H-30's labeled fixture mirrors it."

FIXTURE PROTOCOL VERIFICATION:
  BeliefShift does NOT implement a single-turn "present X, then X'" fixture.
  It uses LONGITUDINAL MULTI-SESSION TRAJECTORIES with transition operators at session boundaries.
  The Evidence-Driven Revision operator injects evidence across sessions, not within one turn.
  H-30 can draw inspiration from the BeliefShift protocol but the fixture shape differs:
    BeliefShift: multi-session, belief state vector tracks across sessions, operator applied at boundary
    H-30 labeled fixture (as described): single-turn or within-context X-then-X' probe
  This is a PARTIAL MATCH. The conceptual grounding (test whether system uses X' not X) is valid;
  the implementation protocol differs significantly.

EXACT PROTOCOL (Evidence-Driven Revision track):

  Belief State Vector (BSV):
    Each trajectory session annotated with BSV: b_t in [-1, +1]^d
    Each dimension d_i = user's stance on one topic sub-dimension (e.g., vaccine efficacy, trust)
    Sequence: {b_1, b_2, ..., b_T} tracked across T sessions per trajectory

  Initialization:
    "Each trajectory is initialized with a Belief Seed: a user profile specifying an initial
    stance vector b_0 in R^d"

  Scaffolding Engine (4 transition operators at session boundaries):
    tau_p: Perspective-Shift (peer influence or social pressure changes belief)
    tau_e: Evidence-Driven Revision (external evidence event injected, triggers principled update)
    tau_c: Contradiction (user expresses position conflicting with prior session stance)
    tau_s: Stability (belief reinforcement, no change)

  Evidence-Driven Revision (tau_e) -- the H-30-relevant operator:
    An external evidence event is injected at a session boundary
    This triggers a "principled update to the belief vector"
    The model must predict the updated BSV b_t given the session transcript + evidence log
    Measurement: compare predicted b_t against human-annotated ground truth b_t

  Evidence log:
    "Each session carries a human-annotated Belief Seed... the Scaffolding Engine then applies
    one of four transition operators at each session boundary."
    "each session in BeliefShift carries a human-annotated BSV... an explicit evidence log
    recording any new factual information introduced during the session"

  What the evaluating model receives:
    Full session transcript
    Explicit evidence log
    Instruction to score each BSV dimension on [-1, +1]
    Prior session BSV is available (cross-session context)

  What is measured:
    Whether predicted b_t matches ground truth b_t after evidence injection
    The BRA metric quantifies accuracy of this tracking

Metrics (four novel metrics):
  BRA (Belief Revision Accuracy):
    BRA = (1/|S_e|) * sum(1 - ||b_hat_t - b_t||_2 / (||b_{t-1} - b_t||_2 + epsilon))
    Range [0, 1]; 1.0 = perfect tracking of evidence-driven updates
    Measures whether the model correctly UPDATES its representation after evidence injection

  DCS (Drift Coherence Score):
    Measures whether belief changes are internally consistent across dimensions

  CRR (Contradiction Resolution Rate):
    Fraction of contradictions the agent explicitly acknowledges and reconciles

  ESI (Evidence Sensitivity Index):
    Whether the model distinguishes evidence-grounded changes from conversational pressure

Dataset:
  2,400 human-annotated multi-session interaction trajectories
  Topics: health, politics, personal values, product preferences
  7 models evaluated: GPT-4o, Claude 3.5 Sonnet, Gemini 1.5 Pro, LLaMA-3 70B/8B, Mistral-Large, Gemini 1.5 Flash

Results (RAG setting):
  GPT-4o: BRA 0.83, highest BRA
  Claude 3.5 Sonnet: BRA 0.74, DCS 0.81 (highest), ESI 0.63 (highest)
  Gemini 1.5 Pro: BRA 0.79
  LLaMA-3 70B: BRA 0.70
  LLaMA-3 8B: BRA 0.55

Key finding (tradeoff):
  "Models that personalize aggressively resist drift poorly; factually grounded models miss
  legitimate belief updates."

Inputs:
  session_trajectory | list[Session] | multi-session conversation history | >=2 sessions
  evidence_log | list[str] | factual events injected at session boundaries
  initial_belief_seed | BSV | b_0 in [-1,+1]^d | human-authored starting stance
  transition_operator | enum | tau_p / tau_e / tau_c / tau_s | one per session boundary

Outputs:
  predicted_bsv | float[d] | b_hat_t in [-1,+1]^d | per-dimension stance prediction
  bra_score | float | [0,1] | accuracy of belief update tracking
  dcs_score | float | coherence of drift
  crr_score | float | contradiction resolution rate
  esi_score | float | sensitivity to evidence vs pressure

Critical invariants:
  assert b_t in [-1.0, 1.0] for all t and all dimensions, "BSV is bounded"
  assert len(trajectory) >= 2, "at least two sessions needed for a transition"
  assert evidence_log is not empty when operator == tau_e, "evidence-driven revision requires evidence"
  assert bra_score >= 0.0 and bra_score <= 1.0, "BRA is bounded"

Failure modes cited:
  Aggressive personalizers: high DCS but low ESI (follow conversational pressure, miss evidence)
  Factually grounded models: low DCS but high ESI (respond to evidence, resist legitimate updates)
  No model dominates all four metrics simultaneously

Implementation pitfalls (do NOT do this):
  Do NOT implement H-30's labeled fixture as a single-session BeliefShift replica;
    BeliefShift is longitudinal multi-session, H-30's fixture can be a single-turn analog
  Do NOT conflate BRA (evidence-driven update accuracy) with CRR (contradiction resolution)
  Do NOT use tau_p (social pressure) or tau_s (stability) when testing belief REVISION;
    only tau_e (evidence) and tau_c (contradiction) are revision operators
  Do NOT omit the evidence log when evaluating tau_e; without it BRA cannot be computed

H-30 GROUNDING VERDICT:
  BeliefShift GROUNDS the conceptual claim that "present state X, inject superseding X',
  measure whether system uses X' not X" is a valid evaluation protocol.
  It does NOT provide a single-turn fixture; it uses multi-session trajectories.
  H-30 can cite 2603.23848 for the CONCEPT of belief-revision evaluation but must NOT claim
  the BeliefShift fixture is identical to H-30's labeled fixture. Label: PARTIAL GROUNDING.
