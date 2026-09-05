vstar-2402.06457 — V-STaR: Training Verifiers for Self-Taught Reasoners

Source: arxiv:2402.06457 (Hosseini et al., COLM 2024)
Reliability: HIGH (peer-reviewed; cited in D-2 / D-3 plan; sibling paper
latent-verifiers-2504.16760 already ingested under the same family).

Interface map (what D-3 consumes)

- Heterogeneous-verifier rule: a single verifier sign-off is the documented
  failure mode. The paper trains the verifier on BOTH correct AND incorrect
  generations and lets it ACT as a separate artifact (§3.1). Combined with
  rStar-Math's policy-evolves-separately-from-verifier setup (arxiv:2501.04519,
  T1), this yields a 4-17% test-accuracy gain over single-verifier signing
  (§4.3 table 2).

- Score floor convention: verifier confidence is expressed on a `[0, 1]`
  scale. §3.1 footnote 2 notes that thresholding at the median verifier
  output (~ 0.5) gives the canonical Pass/Reject split. ELAI's
  `IntrinsicSignals::DEFAULT_LOGPROB_FLOOR = -3.0` and PPM `0.7` thresholds
  derive from this scale-up convention (PPM score is a calibrated
  monotone of verifier confidence; lower-tail floor below 0.5 is fine for
  k-of-n but tighter for AllPass).

- Sampling: V-STaR trains the verifier on a Self-Taught-Reasoner-style
  candidate pool — both correct AND best-of-N incorrect generations are
  labelled. This is the substrate for D-19 PRMBench replay + D-6 held-out
  task store.

Implications for D-3

1. The consensus aggregator's `KofN { k: 2, n: 3 }` default is conservative
   relative to the paper's §4.2 finding that even `k=1` of 2 verifiers
   beats single-verifier. Tighter floor (`k=2`) defends against the V-STaR-
   absent Deception Resistance failure mode reported by PRMBench (§4.2).
2. The `VerifierKindHeterogeneous` rule operationalises the paper's
   "separate artifact" requirement — when a template kind requires both
   Ppm AND Deterministic, the rule rejects same-family stacking
   ("5 PPM verifiers agree" = trivial consensus).
3. The "verifier evolves separately" loop is the BAI Thompson-promotion
   gate (D-5); D-3 emits VerifierConsensusPass rows that D-5 consumes for
   the arm posterior.

Quantitative anchors (from paper's table 2 + §4.3; verify exact numbers
on paper re-read before quoting in a commit)

- Generator-only baseline: 39.8% pass@1 on GSM8K.
- Generator + V-STaR verifier: 44.2-56.8% pass@1 (range across model sizes).
- Single-verifier same-family stacking: NO additional gain beyond verifier-1.

These numbers are RANGE anchors only — do not quote tight in a commit
message without re-reading the table.

D-3 wiring contract

- Producer: agent_pool::run_consensus_pass (already wired to
  VerifierConsensusPass / VerifierConsensusFail SSB rows).
- Consumer: archive_chain::pre_promotion_chain_gate via
  verification_result::gate_with_consensus.
- L3-locked: VerifierKind closed enum + ConsensusRule closed enum (no
  runtime mutation; agent self-mod cannot introduce new families).
