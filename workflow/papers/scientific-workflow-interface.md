Algorithm: Skills-Driven Agentic Scientific Workflow Generation (Three-Layer Architecture)
ArXiv: 2604.21910
Paper: From Research Question to Scientific Workflow: Leveraging Agentic AI for Science Automation
Authors: Bartosz Balis, Michal Orzechowski, Piotr Kica, Michal Dygas, Michal Kuszewski
Reliability: HIGH (peer-style preprint with measured ablation table on Claude Opus 4.6 + GPT-5.4 + GPT-4.1-mini, 150-query stratified test set)
Local cache: .claude/papers/2604.21910.html
Domain: agentic-workflow, skills-formalization, deterministic-DAG-generation, scientific-computing

System architecture (three-layer):
  semantic_layer | LLM interprets natural language into structured ResearchIntent | Section 3
  deterministic_layer | validated generators transform intent to DAGs (Hyperflow WMS on Kubernetes) | Section 3
  knowledge_layer | Skills documents encode vocabulary mappings, parameter constraints, optimization strategies | Section 4

Pipeline (Section 3.2, Figure 2):
  step1_routing | Conductor classifies domain, routes to Workflow Composer
  step2_planning | Composer + LLM + Skills produce structured intent + workflow plan
  step3_user_validation | scientist approves intent before execution
  step4_provisioning | Deployment Service provisions K8s, measures data sizes
  step5_deferred_generation | Deployment Service returns measurements, Composer generates final DAG (deferred = measurement-aware)
  step6_execution_approval | Conductor presents summary, scientist approves

Skill manifest format (paper Section 4 — plain markdown, NOT a formal schema):
  Each Skill is a markdown document containing:
    - definition tables (vocabulary)
    - synonym mapping tables (term-to-code, e.g. "European" -> EUR)
    - decision logic (interpretation guidelines)
  Note: paper does NOT publish a strict YAML/JSON schema. Author claim: "Skills are plain markdown that anyone can read and verify, unlike embeddings or model weights."

Skill categories (paper-cited examples, NOT a generalized taxonomy):
  Populations | term-to-code mappings (European->EUR, African->AFR)
  Genomic regions | gene-name to GRCh37 coordinate mappings
  Research contexts | topic to (regions, analysis types) routing
  Data sources | location, extraction patterns, transfer size estimates
  Workflow Composer | tools, parameter ranges, interpretation rules
  Author warning: "Each new domain requires its own set of Skills and its own deterministic generator." — NOT a portable taxonomy.

Inputs (ResearchIntent contract — Listing 1):
  analysis_type | enum | str | mandatory
  populations | set[str] | population codes (EUR, AFR, ...) | mandatory
  chromosomes | set[str] | chromosome IDs | mandatory
  regions | list[tuple] | coordinate tuples on GRCh37 | mandatory
  focus | str | research focus modifier | optional

Outputs:
  workflow_dag | DAG | Hyperflow workflow JSON | invariant: deterministic given fixed ResearchIntent
  data_size_measurements | dict | feeds deferred-generation loop | mandatory before final DAG

Skill selection / trigger predicate (CRITICAL GAP):
  Paper Section 3 states: "The Conductor receives a natural-language research query and classifies its domain to select the appropriate Workflow Composer and associated Skills."
  Paper Section 2.3 specifies routing is "by domain routing, not similarity search" — i.e. deterministic classification, NOT embedding retrieval.
  Paper does NOT specify: the classifier prompt, the scoring function, the activation threshold, or the fallback if no Skill matches.
  Implication for C-41: the trigger-predicate spec must be authored locally (paper does not ground it). Mark as UNGROUNDED in implementation.

Ablation table (Section 5.1, Table 2 — full-match intent accuracy on 150 queries stratified T1-T5):
  Model              | S0 (none) | S1 (vocab) | S2 (strategy) | S3 (all)
  Claude Opus 4.6    | 44.0%     | 80.0%      | 57.3%         | 83.3%
  GPT-5.4            | 39.3%     | 78.7%      | 48.7%         | 80.0%
  GPT-4.1-mini       | 27.4%     | 70.7%      | 40.0%         | 62.0%
  Definition: full-match intent accuracy = ALL fields correct simultaneously (populations, chromosomes, regions, analysis_type, focus). Per-field uses "exact set matching"; regions compared as coordinate tuples.
  Test composition: 5 tiers x 30 queries each:
    T1 explicit codes | T2 synonyms | T3 implicit inference | T4 underspecified | T5 adversarial
  Headline finding: vocabulary Skills (S1) drive +36pp on Opus (44 -> 80). All Skills (S3) = 83.3% on Opus.
  T1-T2: models reach 100% with S1. T3-T5: sharp drops without Skills.

Other measured numbers:
  data_transfer_reduction | 92% | from deferred (measurement-aware) workflow generation | Section 5.2
  llm_overhead | < 15s | end-to-end per query | abstract, Section 5.3
  cost | < $0.001 per query | abstract, Section 5.3

Hyperparameters / configuration:
  num_skills | 5 in evaluation | 1000-Genomes domain
  test_set_size | 150 queries | stratified 30/tier x 5 tiers
  models_evaluated | Claude Opus 4.6, GPT-5.4, GPT-4.1-mini | API models

Critical invariants (testable assertions):
  assert intent.populations.issubset(known_population_codes), "populations must map to known codes"
  assert all(len(r) == 2 and r[0] < r[1] for r in intent.regions), "regions are valid (start, end) coordinate tuples"
  assert intent.analysis_type in REGISTERED_ANALYSIS_TYPES, "analysis_type from closed enum"
  assert generated_dag(intent_A) == generated_dag(intent_A), "identical intent -> identical DAG (determinism)"
  assert full_match_score(model_with_S1, T1_T2_subset) >= 0.95, "with vocab skills, explicit/synonym tiers reach near-100%"

Complexity:
  time | LLM call cost dominates; deterministic generator is O(intent_size) | <15s end-to-end measured
  space | Skills are markdown (KB-scale per skill); 5 skills total in eval | trivial
  parallelism | deferred generation requires sequential: provision -> measure -> generate; no parallelism inside one workflow

Failure modes cited by authors:
  no_domain_match | Conductor cannot route -> no Skills loaded -> falls back to bare LLM (S0 baseline = 44% accuracy)
  underspecified_query | T4 tier — LLM must infer missing fields; even with Skills accuracy drops
  adversarial_query | T5 tier — contradictory or misleading phrasing; lowest accuracy
  vocab_drift | new gene aliases not in synonym table -> miss
  domain_lock_in | "Each new domain requires its own set of Skills and its own deterministic generator" — no zero-shot domain transfer

Implementation pitfalls (do NOT do this):
  do_not_use_embedding_retrieval | paper specifies deterministic domain classification, not similarity search
  do_not_skip_user_validation | step3 + step6 are explicit human-in-the-loop gates; removing them breaks the safety story
  do_not_collapse_layers | the semantic/deterministic/knowledge split is the core contribution; merging LLM with generator loses determinism
  do_not_treat_skills_as_optional | S0 (no Skills) = 44% on Opus; the paper's claim REQUIRES Skills for the headline number
  do_not_assume_skill_schema | paper publishes NO formal schema; the "vocabulary tables + synonym tables + decision logic" structure is descriptive, not normative — local implementer must author concrete schema
  do_not_assume_trigger_predicate_grounded | paper specifies "domain classification" but NOT the classifier internals; the local C-41 implementation must author trigger logic, not cite this paper for it

Open questions for C-41 (paper does NOT answer):
  - what exact prompt template the Conductor uses for domain classification
  - how Skills are ranked when multiple match (paper implies 1:1 domain->skill set, no ranking)
  - how to handle SSB/event/IPC vocabulary — paper's vocabulary is internal to ResearchIntent fields, not an event bus
  - how Skills are versioned or hot-reloaded

Grounding for C-41 plan body claims:
  CLAIM "44% -> 83% effect size" | GROUNDED | Section 5.1 Table 2, Claude Opus 4.6, S0 vs S3 | reproduces in paper
  CLAIM "skills formalization" | PARTIALLY GROUNDED | paper describes Skills as markdown with three internal structures (vocab, synonyms, decision logic) but publishes no formal schema; C-41's "manifest schema" is locally authored
  CLAIM "trigger predicate spec" | UNGROUNDED | paper acknowledges domain routing but does not specify predicate; C-41 must author and label as locally derived
  CLAIM "scoring/ranking method" | NOT IN PAPER | paper's design is 1:1 domain->skill set; if C-41 needs ranking, the design is local
  CLAIM "SSB / event vocabulary" | NOT IN PAPER | paper has no event bus; ResearchIntent is a static structured object passed between layers
