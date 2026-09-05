Source: ouroboros (github.com/Q00/ouroboros, MIT license)
Reliability: HIGH (official source, reviewed)
Session: s-37672
Fetched: 2026-05-10
Files examined:
  src/ouroboros/core/lineage.py (lines ~110-180, OntologyDelta.compute)
  src/ouroboros/evolution/convergence.py (lines ~107-126, 156-171, 306-321, 323-346)
  src/ouroboros/evolution/regression.py (lines ~39-112, RegressionDetector.detect)
Local cache:
  workflow/papers/ouroboros-src/lineage.py
  workflow/papers/ouroboros-src/convergence.py
  workflow/papers/ouroboros-src/regression.py

ALGORITHM A: OntologyDelta.compute (lineage.py ~110-180)
Phase target: 4H.7B ConfigVariantSimilarity

Inputs:
  old | OntologySchema | fields: list of OntologyField(name, field_type, description) | mandatory
  new | OntologySchema | fields: list of OntologyField(name, field_type, description) | mandatory

Outputs:
  OntologyDelta | frozen Pydantic model | similarity: float in [0.0, 1.0] | invariant: similarity always in [0.0, 1.0]
  OntologyDelta.added_fields | tuple[OntologyField, ...] | fields in new not in old | sorted by name
  OntologyDelta.removed_fields | tuple[str, ...] | field names in old not in new | sorted
  OntologyDelta.modified_fields | tuple[FieldModification, ...] | same name, different type or description

Similarity formula (exact, verified from source):
  all_names = old_names | new_names
  if not all_names: similarity = 1.0   (empty schemas are identical)
  else:
    name_score  = len(common_names) / len(all_names)        # key_overlap
    type_score  = count(same_type in common) / len(all_names)   # type_match
    exact_score = count(same_type AND same_desc in common) / len(all_names)  # exact_match
    similarity  = 0.5 * name_score + 0.3 * type_score + 0.2 * exact_score

Critical detail: denominators all use len(all_names) (union), NOT len(common_names).
This means type_score and exact_score are bounded by key overlap -- they cannot exceed name_score.
Maximum similarity = 1.0 only when all_names is empty OR all fields match on name+type+desc.

Adaptation note for 4H.7B ConfigVariantSimilarity:
  Replace OntologyField with ConfigKey (name=key_path, field_type=value_type, description=value_repr).
  Formula weights are FIXED at 0.5/0.3/0.2 -- do not alter weights without spec change.

Hyperparameters: none (formula is fixed, no thresholds in compute itself)

Critical assertions (testable):
  assert 0.0 <= delta.similarity <= 1.0
  assert abs((0.5 + 0.3 + 0.2) - 1.0) < 1e-9   # weights sum to 1
  # empty schemas
  assert OntologyDelta.compute(empty, empty).similarity == 1.0
  # identical schemas
  assert OntologyDelta.compute(schema, schema).similarity == 1.0
  # disjoint schemas -> name_score=0, type_score=0, exact_score=0 -> similarity=0
  assert OntologyDelta.compute(schema_a, schema_b_disjoint).similarity == 0.0
  # name overlap only (same names, different types) -> similarity = 0.5*name_score
  # type_score and exact_score both 0 because type differs

Failure modes:
  If denominator uses common_names instead of all_names: type_score inflated, similarity > correct value
  If weights altered: invariant 0.5+0.3+0.2=1.0 breaks, similarity range no longer [0,1]
  If description comparison uses identity not equality: exact_match understated

Implementation pitfalls:
  Do NOT use len(common_names) as denominator for type_score or exact_score -- source uses len(all_names)
  Do NOT skip the empty-schema guard (returns 1.0, not 0/NaN)
  Do NOT sort by insertion order -- added and removed fields are sorted lexicographically

---

ALGORITHM B: ConvergenceCriteria._check_oscillation and _check_stagnation (convergence.py)
Phase target: 4H.7C (OSC), 4H.7D (EVO gate)

Class defaults (ConvergenceCriteria dataclass):
  convergence_threshold: float = 0.95   (similarity >= this => "same" ontology)
  stagnation_window: int = 3            (consecutive unchanged gens = stagnation)
  min_generations: int = 2
  max_generations: int = 30
  enable_oscillation_detection: bool = True
  eval_gate_enabled: bool = False
  eval_min_score: float = 0.7
  ac_gate_mode: str = "all"            # "all" | "ratio" | "off"
  ac_min_pass_ratio: float = 1.0
  regression_gate_enabled: bool = True
  validation_gate_enabled: bool = True

_check_stagnation (lines 306-321):
  Input: lineage (OntologyLineage with completed generations list)
  Returns: bool -- True if stagnation detected
  Logic:
    if len(completed_gens) < stagnation_window: return False
    window = last stagnation_window completed gens
    for each consecutive pair in window:
      compute OntologyDelta.similarity
      if similarity < convergence_threshold: return False  (change found, no stagnation)
    return True  (all pairs similar -> stagnation)
  Invariant: requires len >= stagnation_window (default 3) before triggering

_check_oscillation (lines 323-346):
  Input: lineage
  Returns: bool -- True if period-2 oscillation detected
  Logic (4-gen case, PREFERRED -- full period-2 verification):
    sim_n_n2  = similarity(gens[-3], gens[-1])   # N vs N-2
    sim_n1_n3 = similarity(gens[-4], gens[-2])   # N-1 vs N-3
    if BOTH >= convergence_threshold: return True
  Logic (3-gen fallback -- partial check):
    sim = similarity(gens[-3], gens[-1])          # N vs N-2 only
    if sim >= convergence_threshold: return True
  Fallback note: 3-gen check is weaker (only one half-period verified). Source comments call 4-gen the "full" check.
  Returns False if fewer than 3 gens.

_count_evolved_generations (lines 107-126):
  Input: lineage
  Returns: int -- number of consecutive-generation pairs where similarity < convergence_threshold
  A return of 0 means ontology never changed across all completed generation transitions.

Evolution gate withholding clause (lines 156-171):
  Triggered inside evaluate() AFTER similarity threshold already satisfied.
  If _count_evolved_generations == 0:
    return ConvergenceSignal(converged=False, reason="Convergence withheld: ...")
  Semantics: similarity alone is insufficient for convergence; the ontology must have ACTUALLY EVOLVED
  at least once (i.e., at least one generation transition crossed below convergence_threshold) before
  convergence is accepted. This prevents false-positive convergence on a static (never-mutating) ontology.

Critical assertions (testable):
  # stagnation requires window full
  assert not criteria._check_stagnation(lineage_with_1_gen)
  # stagnation: all pairs in window must be similar
  # oscillation 4-gen: both sim_n_n2 and sim_n1_n3 must be >= threshold
  # withholding: if evolved_count == 0 and similarity >= threshold, converged must be False
  assert convergence_signal.converged == False when evolved_count == 0
  # oscillation fallback is strictly weaker than 4-gen check

Failure modes:
  Using only N~N-2 for oscillation (3-gen check) on 4+ gen lineages: misses A->B->B->A patterns
  Not implementing the evolution gate: static lineage falsely converges on first high-similarity generation
  stagnation_window off-by-one: triggers one generation too early or too late

Implementation pitfalls:
  Do NOT omit the evolved_count == 0 withholding check -- it is a separate gate from similarity threshold
  Do NOT use len(all gens) instead of len(completed gens) -- incomplete generations must be excluded
  Do NOT implement only the 4-gen oscillation path -- the 3-gen fallback is required for early detection
  Do NOT hardcode threshold -- must use self.convergence_threshold (field, not module constant)

---

ALGORITHM C: RegressionDetector.detect (regression.py ~39-112)
Phase target: 4H.7C REG

Inputs:
  lineage | OntologyLineage | .generations: list of GenerationRecord | mandatory
  Each GenerationRecord has:
    .generation_number: int
    .evaluation_summary.ac_results: list of ACResult
    ACResult has: .ac_index: int, .passed: bool, .ac_content: str

Outputs:
  RegressionReport | .regressions: tuple[ACRegression, ...] | empty tuple if no regressions
  ACRegression fields:
    ac_index: int
    ac_text: str
    passed_in_generation: int   (last gen where AC passed)
    failed_in_generation: int   (latest gen number)
    consecutive_failures: int   (streak from latest backwards)

Algorithm (no thresholds -- pure boolean pass/fail):
  1. Guard: if fewer than 2 generations, return empty RegressionReport
  2. Guard: if latest gen has no evaluation_summary or no ac_results, return empty
  3. Build ac_history dict: ac_index -> [(gen_number, passed), ...] across ALL generations
     NOTE: generations with no evaluation_summary are SKIPPED in history build
  4. For each AC in latest gen that is NOT passed:
     a. Get history for that ac_index
     b. Scan history in REVERSE to find last gen where passed == True
     c. If no prior pass found: NOT a regression (persistent failure), skip
     d. Count consecutive failures from latest backwards (stop at first pass)
     e. Append ACRegression
  5. Return RegressionReport(regressions=tuple(regressions))

Key invariant: an AC is a REGRESSION only if it has BOTH failed in latest AND passed in some prior gen.
An AC that has NEVER passed is a persistent failure, not a regression, and is excluded.

Critical assertions (testable):
  # fewer than 2 gens -> empty report
  assert detector.detect(single_gen_lineage).regressions == ()
  # never-passed AC is not a regression
  assert all(r.passed_in_generation is not None for r in report.regressions)
  # consecutive_failures >= 1 for all reported regressions
  assert all(r.consecutive_failures >= 1 for r in report.regressions)
  # ac currently passing -> not in regressions
  assert all(r.ac_index not in currently_passing_indices for r in report.regressions)

Failure modes:
  Treating persistent failures (never passed) as regressions: false positives
  Not scanning ALL generations for history (only recent N): misses regressions after a long pass streak
  Using gen insertion order not gen_number for "last passed": wrong attribution if gens stored out of order
  Not guarding empty evaluation_summary: AttributeError on None

Implementation pitfalls:
  Do NOT use a threshold for regression -- the algorithm is purely boolean (passed or not)
  Do NOT skip the "last_passed_gen is None" guard -- this is the persistent-failure exclusion
  Do NOT reverse only the latest-gen results -- reverse the FULL history for consecutive count
  Do NOT count from oldest forward for consecutive failures -- count must go BACKWARDS from latest

Complexity:
  OntologyDelta.compute: O(|old_fields| + |new_fields|) time, O(|union|) space
  _check_stagnation: O(stagnation_window * field_count) time
  _check_oscillation: O(4 * field_count) time (constant number of delta computations)
  RegressionDetector.detect: O(|gens| * |acs_per_gen|) time, O(|unique_acs|) space
  All algorithms: single-threaded, no parallelism in source

Platform constraints:
  Pure Python (3.10+ type hints: tuple[X,...], dict[X,Y])
  Pydantic BaseModel with frozen=True for OntologyDelta (immutable after construction)
  dataclass for ConvergenceCriteria (mutable fields, not frozen)
  No I/O side effects in any of the three algorithms
  Thread safety: OntologyDelta is frozen/immutable (safe). ConvergenceCriteria is a plain dataclass (not thread-safe for concurrent field mutation, but detect/check methods are read-only on lineage).

Rust port notes for 4H.7B/4H.7C/4H.7D:
  similarity formula: use f64 not f32 (precision matters for threshold comparisons at 0.95)
  convergence_threshold: store as f64, default 0.95
  stagnation_window: usize, default 3
  empty-schema guard (similarity = 1.0) must be explicit before division
  OntologyDelta equivalent should be a plain struct (not frozen -- Rust immutability via &)
  _check_oscillation 4-gen path: index as gens[len-1], gens[len-2], gens[len-3], gens[len-4]
  consecutive_failures count: iterate backwards, break on first pass; type should be u32 or usize
