Source: alphaproof-nexus-results (google-deepmind)
Reliability: HIGH (official Google DeepMind GitHub repo; Apache-2.0 software / CC-BY-4.0 materials)
Repo: https://github.com/google-deepmind/alphaproof-nexus-results
Commit: 0647711a71183c1ea492ad60860776617ce1ea88 (HEAD, branch main), cloned 2026-06-06
Local clone: F:\.ELAI_workflow\docs\.source\alphaproof-nexus-results
Map artifacts:
  manual map (substance): F:\.ELAI_workflow\docs\.source\alphaproof-nexus-results\alphaproof.MANUAL-MAP.txt
  auto-mapper output (EMPTY, kept for provenance): alphaproof.map + alphaproof_graph.json (0 records -- mapper has no Lean support)

WHAT THE REPO IS (from its README)
  Mechanically formalized Lean 4 proof results plus companion informal natural-language
  (PDF) proofs accompanying the "AlphaProof Nexus" paper (Google DeepMind). It contains
  ONLY problems for which AlphaProof Nexus discovered a SUCCESSFUL proof. It is not a
  runnable program -- it is a lake-buildable Lean library whose deliverable is the set of
  machine-checked proof terms. README explicitly: "This is not an official Google product."

WHAT IT IS FOR
  Evidence / artifact release backing the paper's claims that an automated theorem prover
  (AlphaProof Nexus) produced Lean-verified proofs across four problem sources:
    - AICollaborator: AI<->human research collaborations across AdditiveCombinatorics,
      AlgebraicGeometry (Hilbert functions), Graphs, Optimization, QuantumOptics
      (section "Deployment in Mathematics Research")
    - ErdosProblems: Erdos open problems (section "Systematic Evaluation on Open Problems")
    - OEIS: conjectures about Online Encyclopedia of Integer Sequences entries
    - StacksProject: algebraic-geometry / commutative-algebra theorems
  NaturalLanguageProofs/ holds human-written prose proofs mirroring the structure of the
  Lean proofs (12 PDFs). erdos_problems_attempted.txt (352 lines) lists every Erdos problem
  attempted (solved + unsolved). Unsolved formalizations live in the sibling
  google-deepmind/formal-conjectures repo, not here.

STRUCTURE (see manual map for full per-file detail)
  APNOutputs/                  71 .lean files, 44,857 lines  (the Lean proofs)
    AICollaborator/{AdditiveCombinatorics,AlgebraicGeometry,Graphs,Optimization,QuantumOptics}
    ErdosProblems/ (9 files)   OEIS/ (38 files)   StacksProject/ (11 files)
  NaturalLanguageProofs/       12 .pdf files (AICollaborator 2, ErdosProblems 9, OEIS 1)
  Build: lakefile.toml (lib APNOutputs), lean-toolchain leanprover/lean4:v4.27.0,
         lake-manifest.json pins formal_conjectures (google-deepmind) + mathlib4 + 8 more.

ARTIFACT-SHAPE FACTS (relevant if anyone consumes this corpus)
  Lean declaration totals: 2339 lemma, 483 def, 156 theorem, 22 abbrev, 12 instance.
  Every .lean imports FormalConjectures.Util.ProblemImports.
  Theorems carry @[category ...] / @[AMS ...] attributes (formal-conjectures convention).
  Proof gaps: 128 `sorry` markers, ALL inside 2 QuantumOptics files
    (MonochromaticQuantumGraph.lean 88, 35.lean 40) -- stated-but-unproven AME benchmark
    cases + `answer(sorry)` open-question placeholders. The other 69 files have no sorry.
  Build requires network fetch of mathlib cache (`lake exe cache get`) before `lake build`;
    full verification is heavy (set_option maxHeartbeats tuned per file; LEAN_NUM_THREADS cap advised).

Behavioral contracts (n/a -- no callable runtime API; this is a proof library)
  - "build" contract: `lake exe cache get && lake build` machine-checks all non-sorry proofs.
  - A green build is the proof-of-correctness; there is no exported function surface to call.

Platform constraints
  - Requires Lean 4 toolchain v4.27.0 exactly (lean-toolchain pin) + elan/lake.
  - Network access needed for mathlib cache + formal_conjectures dependency resolution.
  - Heavy memory/CPU; README warns about open-file limits at high parallelism.

Gotchas
  - codebase_mapper.py cannot map this repo (no Lean support) -> auto map is empty; use the manual map.
  - 2 QuantumOptics files contain `sorry`; do not treat those specific AME cases as proven.
  - This repo is the SOLVED subset only; the full attempt set (incl. failures) is in
    google-deepmind/formal-conjectures (auto_oeis branch + ErdosProblems dir).

Critical assertions (testable, about the artifact)
  assert clone_commit == "0647711a71183c1ea492ad60860776617ce1ea88"
  assert num_lean_files == 71 and num_pdf_files == 12
  assert total_lean_lines == 44857
  assert sorry_files == {"APNOutputs/AICollaborator/QuantumOptics/35.lean",
                         "APNOutputs/AICollaborator/QuantumOptics/MonochromaticQuantumGraph.lean"}
  assert lean_toolchain == "leanprover/lean4:v4.27.0"
