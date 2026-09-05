Algorithm: Coscientist — Autonomous Chemical Research Agent
ArXiv: 2304.05332
Paper: Emergent autonomous scientific research capabilities of large language models (Boiko, MacKnight, Kline, Gomes; CMU; published Nature vol.624 Dec 2023, DOI 10.1038/s41586-023-06792-0)

Summary: Multi-LLM orchestration system (GPT-4 Planner + specialized submodules) that autonomously designs, plans, and executes chemical experiments using internet search, API documentation lookup, code execution, and robotic hardware control. Demonstrated on six tasks culminating in physical Suzuki-Miyaura and Sonogashira catalytic cross-coupling reactions confirmed by GC-MS.

Inputs:
user_goal | natural language string | unconstrained | mandatory; plain English task description (e.g. "perform multiple Suzuki reactions")
api_documentation | text corpus | vector-embedded chunks | optional; Opentrons OT-2 API pages + ECL SLL functions; absent = no hardware execution
reagent_inventory | physical lab consumables | source plate layout | mandatory for Tasks 3-5; plates moved manually in current impl
optimization_dataset | tabular CSV | rows=reaction conditions, col=yield float [0,1] | optional; required for Task 6 (Perera Suzuki or Doyle BH datasets)
max_iterations | int | [1, inf) default=20 | optional; bounds Task 6 optimization game

Outputs:
synthesis_plan | structured text | step-by-step procedure with quantities | Task 1; quality scored 1-5
hardware_code | Python (Opentrons API) or SLL | executable script | Tasks 3-5; sent to EXPERIMENT command
observation_record | hardware response or execution log | stdout + instrument data files | raw hardware feedback (UV-Vis spectra, GC-MS chromatograms)
optimization_action | JSON dict | keys: condition selections (ligand, base, solvent, reagent) | Task 6; must be parseable JSON or task fails
nma_score | float | Normalized Maximum Advantage in [-inf, 1.0]; NMA=1.0 = max yield found, NMA=0 = random baseline | Task 6 performance metric

Hyperparameters:
planner_model | GPT-4 (chat completion) | GPT-4 only validated; GPT-3.5 fails JSON task | high sensitivity; GPT-3.5 cannot reliably output JSON schema
web_searcher_model | GPT-4 or GPT-3.5-turbo | GPT-4 preferred; GPT-3.5 shows lower synthesis accuracy | moderate sensitivity
embedding_model | OpenAI ada | used for documentation vector search | not varied in experiments
max_optimization_iterations | 20 | integer; 5.2-6.9% of full condition space explored | not sensitivity-tested
synthesis_score_threshold | 3 | [1,5] integer; below=failure | authors treat 3 as minimum acceptable
temperature | not stated | not disclosed; default presumed | unknown sensitivity

Critical invariants (testable assertions):
assert json.loads(optimization_action) is not None  # JSON format mandatory; failure alerts player and aborts task (Task 6 invariant, paper Sec. "Chemical reasoning game")
assert hardware_code does not contain network calls to external services beyond Opentrons/ECL API  # Docker isolation contract: code execution isolated, no LLM dependency at execution time
assert physical_observation is only valid after GC-MS or UV-Vis instrument output file is returned  # observation validity: product confirmation requires instrument signal match (biphenyl at 9.53 min RT, diphenylacetylene at 12.92 min RT)
assert nma_score increases monotonically on average across Task 6 iterations  # paper Fig.4: "normalized advantage values increase over time"
assert synthesis_plan_score >= 3 or synthesis_plan is classified as failure  # scoring rubric: score <3 = incorrect or infeasible

Sandbox vs Production observation taxonomy (D-18 relevant):
SANDBOX observation: code execution in isolated Docker container with no external side effects; RDKit/NumPy computations; lookup tables from known datasets (Task 6 condition space); no physical state change; output = Python stdout or return value
PRODUCTION observation: physical hardware API call through EXPERIMENT command; instrument returns file (spectra, chromatogram); irreversible physical/chemical state change; output = instrument data file (UV-Vis .csv, GC-MS signal peak)
BOUNDARY CASE: ECL cloud lab — remote physical execution; human does not touch hardware but state is real; treated as PRODUCTION; HPLC air bubble incident is a PRODUCTION failure not a sandbox failure
KEY DISTINCTION the paper draws: "The PYTHON command performs code execution using an isolated Docker container to protect the users machine from any unexpected actions" — Docker = sandbox boundary; EXPERIMENT command = production boundary; crossing EXPERIMENT boundary means real-world irreversibility

Observable confounds cited in paper:
1. Training data leakage: "It is unclear whether GPT-4s training data contained information from the optimization datasets (Suzuki: Perera et al., BH: Doyle)"; GPT-4 cutoff overlaps with published datasets; cannot exclude memorization
2. Non-deterministic reagent selection: "strategy of Coscientist changes among runs... base DBU selected more often with PEPPSI-IPr complex, with preference switching in Sonogashira"; no chemical rationale given; run-to-run variance unquantified
3. Common compound search bias: web search GPT-4 "performed lower than some other models for ethylacetate and benzoic acid, possibly because of widespread nature of these compounds"; high-frequency compounds contaminate search results
4. Partial automation gap: "plates were moved manually; no human decision-making was involved"; manual transfer introduces unmeasured physical variability (contamination, volume error)
5. HPLC parameter delegation: "Other parameters (column, mobile phases, gradients) were determined by ECLs internal software"; agent did not control full experiment; partial automation inflates apparent capability
6. Single-run confirmation: Task 5 cross-coupling confirmed in single run per reaction type; no statistical replication reported; n=1 for physical experiments

Safety failure modes cited in paper:
1. Dual-use potential: authors performed a brief dual-use study (Supplementary Information); full details withheld pending US AI regulation development; simplified implementation released instead of full system
2. Hallucination of synthesis routes: "all non-browsing models incorrectly synthesized ibuprofen" (direct nitration producing undesired mixture); ungrounded LLMs invent plausible-sounding but incorrect or dangerous procedures
3. JSON schema non-compliance: GPT-3.5 unable to output correct JSON; optimization task incomplete; structured output failure mode for weaker models
4. API method hallucination: incorrect heater-shaker module method name generated initially; self-corrected via Docs searcher; if not corrected: hardware command failure or silent no-op
5. HPLC quality control gap: "air bubble was injected along with analyte"; physical execution defects not detected by agent; demonstrates need for automated quality-control sensors
6. No atomic safety guarantee: paper states no hard constraint prevents hazardous synthesis requests; relies on LLM alignment and access control at model level (GPT-4 content policy) rather than system-level chemical safety filters
7. Irreversible physical action: once EXPERIMENT command fires, chemical reaction cannot be undone; no rollback mechanism exists

Computational complexity:
synthesis_planning | O(k * web_search_latency) where k = search iterations; GPT-4 API calls dominant cost | no parallelism mentioned
documentation_search | O(n_docs * embedding_dim) for vector similarity; one-time indexing cost + O(log n) per query | parallelizable
code_generation_and_fix | O(e * api_call) where e = number of error correction rounds; worst case unbounded | sequential (fix loop)
optimization_game | O(max_iterations * api_call); 20 iterations x GPT-4 call = ~20 API calls per run | sequential

Known failure modes cited in paper:
multistep synthesis planning fails without reaction database (Reaxys/SciFinder); web search insufficient for complex routes
heterogeneous robotic ecosystem (ECL vs Opentrons) requires separate documentation; scalability challenge
older models (GPT-3.5, Falcon-40B) fail at structured JSON output required for optimization tasks
without web search, all tested models hallucinate synthesis routes for common pharmaceuticals (ibuprofen, aspirin variants)
hardware released after model training cutoff (heater-shaker module) requires documentation lookup; cannot rely on parametric knowledge
no automated quality control: physical defects (bubbles, contamination) pass silently unless instrument output is analyzed

Implementation pitfalls (do NOT do this):
do NOT treat Docker-sandboxed Python output as equivalent to physical instrument observation; sandbox = computational state only, no real-world side effects
do NOT trust LLM-generated synthesis plans without web-search grounding; hallucination rate is high without retrieval augmentation
do NOT use GPT-3.5 or weaker models for structured JSON output tasks; GPT-4 required for reliable schema compliance
do NOT assume hardware API method names from training; post-cutoff hardware (heater-shaker) requires runtime documentation lookup
do NOT count manual human actions (plate movement) as agent capability; partial automation inflates success claims
do NOT treat single-run GC-MS confirmation as statistical proof; n=1 physical experiments are necessary but not sufficient for reproducibility claims
do NOT use optimization NMA scores as proof of reasoning over memorization; training data overlap with published datasets (Perera, Doyle) cannot be excluded
do NOT skip instrument output parsing; air bubble events and similar defects are only detectable from raw instrument data files, not from execution logs

D-18 type taxonomy grounding:
SandboxObservation: type=computational; source=Docker isolated PYTHON command; side_effects=none; rollback=trivially; examples: RDKit fingerprint, NumPy array, dataset lookup
ProductionObservation: type=physical; source=EXPERIMENT command (Opentrons OT-2 or ECL); side_effects=irreversible physical/chemical state; rollback=impossible; requires=instrument data file for validity; examples: GC-MS chromatogram, UV-Vis absorbance spectrum, plate reader output
CloudLabObservation: sub-type of ProductionObservation; source=ECL remote execution; physical=true; human_touch=false; instrument_defects=possible and not auto-detected
PartialObservation: mixed; code correct but hardware action delegated to human (plate movement); agent cannot claim full observation ownership
