VOYAGER — grounded technical dossier (SHARD prior-art, skill-acquisition / skill-library pillar)

Compiled 2026-06-01 (oracle-mode, grounded-by-default). All architecture, API, and benchmark claims trace to fetched primary sources (paper + repo + ar5iv full text). Anything not grounded is labelled UNGROUNDED in the ledger.

================================================================
1. IDENTITY & PROVENANCE
================================================================

Full name: Voyager: An Open-Ended Embodied Agent with Large Language Models.
Authors: Guanzhi Wang, Yuqi Xie, Yunfan Jiang, Ajay Mandlekar, Chaowei Xiao, Yuke Zhu, Linxi "Jim" Fan, Anima Anandkumar.
Organizations (per official site): NVIDIA, Caltech, UT Austin, Stanford, ASU. [site does not bind each author to a single institution]
Paper id: arXiv:2305.16291 — VERIFIED, resolves. cs.AI.
  v1 submitted 2023-05-25; v2 submitted 2023-10-19 (the canonical citeable version is v2, 19 Oct 2023).
Repo: github.com/MineDojo/Voyager — VERIFIED via api.github.com. description "An Open-Ended Embodied Agent with Large Language Models". default_branch main. archived=false.
Site: voyager.minedojo.org — VERIFIED, resolves and serves the project page + hosted PDF.
License: MIT. LICENSE file first lines: "MIT License / Copyright (c) 2023 MineDojo Team". spdx_id from GitHub API = "MIT". NO license/version mismatch found — paper, repo metadata, and LICENSE file all agree (MIT, 2023, MineDojo).
Package registry: there is NO PyPI/npm published package. Voyager is installed from source (setup.py + `pip install -e .`), not from a registry. So "current version" is a git ref, not a SemVer release — there are no tagged releases driving a version number. Treat "version" as: repo HEAD, last code push 2024-04-03.
Verified links:
  paper  https://arxiv.org/abs/2305.16291  (PDF https://arxiv.org/pdf/2305.16291)
  repo   https://github.com/MineDojo/Voyager
  site   https://voyager.minedojo.org/  (hosted PDF /assets/documents/voyager.pdf)
  parent framework MineDojo https://minedojo.org/

VERSION/STALENESS FLAG: the code is effectively frozen — last push 2024-04-03 (GitHub `pushed_at`), no releases. The repo `updated_at` 2026-05-31 reflects stars/watch metadata churn, not code. So "current version" = a 2024 snapshot; nothing newer despite the recent updated_at timestamp.

================================================================
2. PROBLEM IT SOLVES
================================================================

Target failure mode (abstract framing): open-ended *lifelong* learning for an embodied agent without human intervention and without model fine-tuning. Specifically three coupled failures of prior LLM-agent / RL approaches:
  (a) Catastrophic forgetting / non-compounding skills — prior agents re-derive behaviour each episode. Voyager's claim: skills are "temporally extended, interpretable, and compositional, which compounds the agent's abilities rapidly and alleviates catastrophic forgetting." (abstract)
  (b) No stored, reusable behaviour artifact — prior NLP-derived agents (ReAct, Reflexion, AutoGPT) keep reasoning in a transient context and cannot transfer learned competence to a new world. Voyager stores executable code as the durable artifact.
  (c) Exploration that stalls — without a curriculum, the agent does not keep proposing novel-but-achievable goals. Voyager's automatic curriculum exists to "maximize exploration."
The headline transfer result it sells: the learned skill library, loaded into a *new* Minecraft world, solves novel tasks from scratch "while other techniques struggle to generalize." (abstract)

================================================================
3. HOW IT WORKS — ARCHITECTURE + KEY MECHANISMS
================================================================

Three named components (abstract + ar5iv method section, grounded):
  1) Automatic Curriculum (maximizes exploration)
  2) Ever-growing Skill Library of executable code (store + retrieve)
  3) Iterative Prompting Mechanism (code-gen with environment feedback, execution errors, self-verification)

LLM backbone: GPT-4 (gpt-4-0314) for code generation and self-verification; GPT-3.5-turbo for cheaper NLP sub-tasks (description generation, curriculum QA) — per ar5iv method/experiment section. Embeddings via OpenAI embeddings (langchain `OpenAIEmbeddings`). NO fine-tuning — "blackbox queries" only (abstract).

Embodiment substrate: the LLM emits JavaScript that drives a Mineflayer bot against a modded Minecraft instance (Fabric). So skills are JS programs, not Python.

--- 3a. SKILL LIBRARY (the SHARD-comparison core) ---
Implementation: voyager/agents/skill.py, class SkillManager (grounded from raw source).
  __init__ params (from source): model_name="gpt-3.5-turbo", temperature=0, retrieval_top_k=5, request_timout=120, ckpt_dir="ckpt", resume=False. LLM = ChatOpenAI; embeddings = OpenAIEmbeddings().
  In-memory store: self.skills is a dict keyed by skill name, value = {"code": program_code, "description": skill_description}.
  Vector store: Chroma vectordb, collection_name="skill_vectordb", persisted to {ckpt_dir}/skill/vectordb.
  Methods: programs (property), add_new_skill(info), generate_skill_description(program_name, program_code), retrieve_skills(query).
  ADD path — add_new_skill(info): takes task info, calls the LLM (generate_skill_description) to produce a natural-language description of the program, then vectordb.add_texts() with the description embedded and the skill *name* stored as metadata, and writes the raw code + description to files on disk.
  RETRIEVE path — retrieve_skills(query): vectordb.similarity_search_with_score(query, k=k) where k = min(#skills, retrieval_top_k); returns the matching program code. Per the paper this is top-5 cosine-similarity retrieval.
  Key/value semantics (ar5iv, verbatim): "The key is the embedding vector of the program description (generated by GPT-3.5), while the value is the program itself." Retrieval query embedding is built from self-generated context (the agent describes what it is trying to do), not the raw task string.
On-disk format (grounded from skill_library/trial1/.../mineWoodLog description): each learned skill is persisted as THREE artifacts under skill_library/trial{N}/skill/ — code/ (the JS function), description/ (a natural-language summary, e.g. mineWoodLog -> "The function is about mining a single wood log block. It searches ... until it finds one of the seven types of wood logs. If a wood log block is found, it is mined and a success message is sent."), and vectordb/ (the Chroma persistence). Three shipped trials (trial1/2/3) are checkpointed skill libraries.

--- 3b. AUTOMATIC CURRICULUM ---
Implementation: voyager/agents/curriculum.py, class CurriculumAgent (grounded).
  Params: mode ("auto" | "manual"), warm_up dict (gates how much state is shown to the LLM as a function of #completed tasks — e.g. nearby entities after 5 tasks, full inventory after 7 tasks), qa_cache (caches self-generated Q/A).
  Methods: propose_next_task(), propose_next_ai_task(), run_qa(), get_task_context().
  Mechanism (ar5iv): bottom-up curriculum. Inputs to the proposal prompt: directives for diverse behaviour; full agent state (inventory, equipment, nearby blocks/entities, biome, time, health, hunger, position); completed_tasks + failed_tasks lists; and extra context from a self-ask QA loop (run_qa generates biome-relevant questions, answers them with GPT-3.5 + wiki knowledge, caches in qa_cache). GPT-4 then proposes the next task under criteria specificity / achievability / novelty. Early progress is hard-coded (e.g. "Mine 1 wood log" at progress 0) before handing off to LLM proposals.

--- 3c. ITERATIVE PROMPTING MECHANISM (action + critic) ---
Action: voyager/agents/action.py, class ActionAgent (grounded).
  Methods: render_system_message() (loads prompt templates + control primitives + retrieved skills), render_human_message() (builds the observation: biome, entities, inventory, health, position; folds in execution_error, render_chest_observation(), chat log, and a critique parameter), process_ai_message() (parses JS functions out of the LLM reply with Babel, validates async structure, returns executable code), summarize_chatlog().
  Feedback folded into the next prompt (grounded, verbatim fragments): "Execution error:\n{error}", chest observations "{chest_position}: {chest}", "Chat log: {chat_log}", and "Critique: {critique}".
Critic / self-verification: voyager/agents/critic.py, class CriticAgent (grounded).
  Methods: check_task_success(), ai_check_task_success(), human_check_task_success().
  ai mode prompts GPT-4 with game-state + task; the LLM returns JSON {"success": true|false, "critique": "..."} (code asserts response['success'] in [True, False]; defaults missing critique to ""). Returns tuple (success: bool, critique: str). human mode prompts "Success? (y/n)" + "Enter your critique:".
Control loop (ar5iv): generate code -> execute in the JS interpreter -> collect environment feedback + execution errors -> critic self-verifies -> if not success, re-prompt with all feedback; "If the agent gets stuck after 4 rounds of code generation, then we query the curriculum for another task." On success, the verified program is handed to SkillManager.add_new_skill.

CONTROL/DATA FLOW (one cycle):
  CurriculumAgent.propose_next_task() -> task
  -> SkillManager.retrieve_skills(context) -> top-5 JS programs injected into ActionAgent system message
  -> ActionAgent generates JS -> Mineflayer executes -> env feedback/errors
  -> CriticAgent.check_task_success() -> (success, critique)
  -> if success: SkillManager.add_new_skill() (embed description, persist code+desc+vectordb)
  -> else: re-prompt up to 4 rounds, then curriculum proposes a different task.

================================================================
4. CORE API / USAGE (verbatim-in-spirit from README)
================================================================

Minimal driver (README, quoted):

  from voyager import Voyager

  azure_login = {
      "client_id": "YOUR_CLIENT_ID",
      "redirect_url": "https://127.0.0.1/auth-response",
      "secret_value": "[OPTIONAL] YOUR_SECRET_VALUE",
      "version": "fabric-loader-0.14.18-1.19",
  }
  openai_api_key = "YOUR_API_KEY"

  voyager = Voyager(
      azure_login=azure_login,
      openai_api_key=openai_api_key,
  )

  voyager.learn()

Skill-library surface (skill.py, grounded):
  sm = SkillManager(retrieval_top_k=5, ckpt_dir="ckpt", resume=True)
  skills = sm.retrieve_skills(query)        # top-k cosine over skill_vectordb
  sm.add_new_skill(info)                     # describe -> embed -> persist code/desc/vectordb
Resume / transfer: README states checkpoints can be resumed and a learned skill library can be loaded to run learned skills in a new world (the zero-shot-generalization experiment).

Install/runtime requirements (README, grounded): Python >= 3.9; Node.js >= 16.13.0; a Minecraft instance with Fabric mods (tested fabric-loader-0.14.18-1.19); Node packages mineflayer + mineflayer-collectblock. Python deps (requirements.txt, grounded): chromadb==0.3.29 (the ONLY pinned version), plus unpinned langchain, openai, tiktoken, tqdm, chardet, cchardet, requests, setuptools, gymnasium, psutil, javascript (the `javascript` bridge), minecraft_launcher_lib.

DEPENDENCY-ROT FLAG (relevant if ELAI ever runs this): chromadb is hard-pinned to 0.3.29 (an old pre-0.4 Chroma API) while langchain/openai are UNPINNED — a fresh install in 2026 pulls current langchain/openai against a 2023 chromadb, which is a near-certain break. This is a maturity/reproducibility risk, not a design feature.

================================================================
5. MEASURED NUMBERS / BENCHMARKS
================================================================

All from the paper (abstract + ar5iv) and the official site. Vendor/author-reported (single-paper, NOT independently reproduced here):
  - 3.3x more unique items collected vs prior SOTA. Site states the absolute as "63 unique items" discovered by Voyager.
  - 2.3x longer map distance traversed.
  - Tech-tree unlock speedups vs baselines: wooden 15.3x, stone 8.5x, iron 6.4x faster.
  - Diamond: Voyager is the ONLY method to unlock the diamond tier; site/ar5iv qualify it as 1/3 runs succeeding, at ~102 iterations. So diamond is a 1-of-3 success, not a deterministic result — read it as "reached at all," not "reliably."
  - Zero-shot generalization (new world): Voyager solved all the held-out tasks; baselines solved none within 50 iterations (site).
Eval setup (ar5iv): baselines = ReAct, Reflexion, AutoGPT, all NLP agents adapted to Minecraft embodied control. Action/critic LLM = GPT-4 (gpt-4-0314); GPT-3.5-turbo for cheaper sub-tasks. Iteration retry cap = 4 code-generation rounds per task before the curriculum reassigns.
SAMPLE SIZE CAVEAT: the multipliers are reported as ratios; the fetched material gives no large-N variance bars — the diamond "1/3 at 102 iters" figure is the clearest signal that run counts are small (single-digit seeds). Treat all multipliers as point estimates from few seeds. Mark ALL of section 5 as VENDOR/AUTHOR-REPORTED, UNREPRODUCED-here.

================================================================
6. MATURITY SIGNALS
================================================================

GitHub (api.github.com, fetched 2026-06-01, all verbatim):
  stars 6943, forks 673, open_issues 8, created 2023-05-25, last code push 2024-04-03, metadata updated 2026-05-31, archived=false, MIT.
Code activity: last *code* push 2024-04-03 -> ~2 years stale at time of writing. No tagged releases. Effectively a research-artifact repo in maintenance/frozen state, not an actively developed product.
Funding / org backing: NVIDIA-led research (Jim Fan, Anima Anandkumar) plus academic co-authors; no commercial product spun out of the repo itself (the repo is the research deliverable). Influence flowed into successors (e.g. Eureka and the broader "skill-library agent" line) rather than into Voyager-the-repo.
Citations: HIGH (widely cited foundational agent paper) but the EXACT count is UNGROUNDED here — Semantic Scholar API returned HTTP 429 (rate-limited) on two attempts, so no number is stated from memory.
Production users: NONE verifiable. No evidence of Voyager running in production anywhere; it is a Minecraft research agent. Do not claim production adoption.

================================================================
7. VERSUS SHARD'S SKILL-ACQUISITION / SKILL-LIBRARY PILLAR
================================================================

Mapping the two skill libraries.

WHERE VOYAGER IS STRONGER / HAS A MECHANISM SHARD LACKS OR SIMPLIFIES:
  - Closed-loop *acquisition with self-verification*: Voyager has a real generate -> execute -> critic(success,critique) -> retry(<=4) -> commit loop (CriticAgent + ActionAgent + 4-round cap). A skill is only added after the GPT-4 critic returns success=true. SHARD's stated design has safety-constrained self-improvement with .bak rollback + 0-3 scoring, BUT per the requester's note SHARD's repo has NO self-improvement loop implemented. So Voyager *ships* a working acquisition-with-verification loop where SHARD currently has only the spec. This is Voyager's clearest edge: an executable, persisted gate (success+critique) on what enters the library.
  - Description-embedding retrieval is *semantic by construction*: Voyager's library key is the GPT-3.5 description embedding, value is the program; retrieval is top-5 cosine over a self-generated context query (skill.py + ar5iv). It is content-addressed by *what the skill does*, which lets a brand-new task pull in compositional sub-skills. SHARD's T1-T4 trust tiers gate *whether* to trust a skill but are a coarser, tier-level filter than a learned semantic index; nothing in the SHARD pillar described here is a vector-similarity retrieval.
  - Compositionality as a first-class outcome: Voyager skills call earlier skills (the JS programs invoke previously-learned functions), so the library compounds. The paper's whole thesis is that this compounding + interpretable code is what beats forgetting. SHARD's composition.py is a *cross-subsystem self-healing cascade* (a different axis — system-level healing, not skill-on-skill program composition).

WHERE SHARD IS STRONGER / HAS MECHANISMS VOYAGER LACKS:
  - Staleness / freshness model: SHARD has staleness = (now - last_verified) / expected_lifetime, flag>1.0, quarantine>2.0. Voyager has NOTHING here — once a skill is verified and added, it is never re-validated or aged out. There is no last_verified field, no decay, no quarantine. A Voyager skill that silently rots (e.g. game/API change) stays in the library forever. This is the single biggest gap in Voyager's library design relative to SHARD.
  - Trust tiers: SHARD's T1-T4 trust tiers stratify provenance/reliability. Voyager skills are flat — every verified skill is equally trusted; there is no provenance tier, no "this came from a low-trust source" marking.
  - Conflict detection / coordinated edits: SHARD has DECLARE-REVIEW-EXECUTE coordination with conflict detection (note: SHARD's repo implements only SYNTACTIC conflict detection, not the spec'd semantic detection). Voyager has neither — skills are added independently, keyed by name; a new skill with the same name overwrites, and there is no review/declare phase and no conflict check at all (syntactic OR semantic). On *coordination*, SHARD (even at syntactic-only) is ahead of Voyager's none.
  - Safety-constrained rollback: SHARD specs .bak rollback + 0-3 scoring for self-improvement. Voyager's "rollback" is implicit (a failed task just isn't added; the agent retries up to 4 rounds) — there is no scored, reversible self-edit of the library; skills are append/overwrite-by-name with no graded acceptance beyond the binary critic success.

NET FOR ELAI: Voyager validates the *acquisition+verification+retrieval* loop SHARD only specs (so it is strong evidence the SHARD acquisition loop is buildable and worth building). But Voyager is the cautionary case for the *maintenance* side SHARD already designed: no staleness, no trust tiers, no conflict detection, no aging. SHARD's staleness/trust/coordination machinery is exactly the layer Voyager never built and that a long-lived (non-game, non-resettable) skill library needs. The honest read: borrow Voyager's critic-gated description-embedding retrieval; do NOT borrow its append-forever, never-revalidate library hygiene — that is precisely what SHARD's staleness+T1-T4+DECLARE-REVIEW-EXECUTE layer is for. The one place SHARD must catch up to Voyager is shipping an actual self-improvement/verification loop (SHARD's is spec-only per the requester's note), where Voyager has a working one.

================================================================
8. CITATIONS (tier-marked)
================================================================

[1] T1  Voyager paper, arXiv abstract — https://arxiv.org/abs/2305.16291  (title, authors, abstract, 3 components, 3.3x/2.3x/15.3x figures, v1 2023-05-25 / v2 2023-10-19)
[2] T1  Voyager full text (ar5iv) — https://ar5iv.labs.arxiv.org/abs/2305.16291  (method: curriculum inputs, skill key=GPT-3.5 desc embedding / value=program, top-5 cosine, 4-round retry cap, GPT-4 gpt-4-0314, baselines ReAct/Reflexion/AutoGPT, wood 15.3x/stone 8.5x/iron 6.4x/diamond 1-of-3 @102)
[3] T1  GitHub API repo metadata — https://api.github.com/repos/MineDojo/Voyager  (MIT, stars 6943, forks 673, open_issues 8, created 2023-05-25, pushed 2024-04-03, updated 2026-05-31, archived=false)
[4] T1  skill.py source — https://raw.githubusercontent.com/MineDojo/Voyager/main/voyager/agents/skill.py  (SkillManager, retrieval_top_k=5, self.skills dict, Chroma collection skill_vectordb, add_new_skill/retrieve_skills/generate_skill_description)
[5] T1  critic.py source — https://raw.githubusercontent.com/MineDojo/Voyager/main/voyager/agents/critic.py  (CriticAgent, check_task_success/ai_check_task_success/human_check_task_success, JSON {success,critique}, tuple return)
[6] T1  curriculum.py source — https://raw.githubusercontent.com/MineDojo/Voyager/main/voyager/agents/curriculum.py  (CurriculumAgent, mode/warm_up/qa_cache, propose_next_task/propose_next_ai_task/run_qa/get_task_context)
[7] T1  action.py source — https://raw.githubusercontent.com/MineDojo/Voyager/main/voyager/agents/action.py  (ActionAgent, render_system/human_message, process_ai_message via Babel, feedback fragments execution error / chest / chat log / critique)
[8] T1  README — https://raw.githubusercontent.com/MineDojo/Voyager/main/README.md  (Voyager(azure_login, openai_api_key).learn(), Python>=3.9, Node>=16.13.0, Fabric fabric-loader-0.14.18-1.19, mineflayer)
[9] T1  requirements.txt — https://raw.githubusercontent.com/MineDojo/Voyager/main/requirements.txt  (chromadb==0.3.29 pinned; langchain/openai/tiktoken unpinned)
[10] T1 LICENSE — https://raw.githubusercontent.com/MineDojo/Voyager/main/LICENSE  (MIT, (c) 2023 MineDojo Team)
[11] T1 skill description sample — https://raw.githubusercontent.com/MineDojo/Voyager/main/skill_library/trial1/skill/description/mineWoodLog.txt  (on-disk description format)
[12] T1 official site — https://voyager.minedojo.org/  (63 unique items, stone 8.5x / iron 6.4x, diamond-only, zero-shot all-vs-none-in-50, org list)

================================================================
9. GROUNDING LEDGER
================================================================

- Title / authors / abstract / 3 components -> [1]. GROUNDED.
- Org list (NVIDIA/Caltech/UT Austin/Stanford/ASU) -> [12]. GROUNDED (site does not bind each author to one org; do not over-attribute).
- arXiv id resolves; v1 2023-05-25, v2 2023-10-19 -> [1]. GROUNDED.
- License MIT, (c) 2023 MineDojo Team, no mismatch -> [10]+[3]. GROUNDED.
- No PyPI/npm package; source install only -> [8]+[9] (install-from-source, no registry artifact). GROUNDED by absence + README install path.
- "current version" = repo HEAD, last code push 2024-04-03; updated_at 2026-05-31 is metadata not code -> [3]. GROUNDED.
- SkillManager: retrieval_top_k=5, self.skills={code,description}, Chroma collection skill_vectordb, add_new_skill/retrieve_skills, similarity_search_with_score top-k -> [4]. GROUNDED.
- Skill key=GPT-3.5 description embedding, value=program; top-5 cosine; query from self-generated context -> [2]+[4]. GROUNDED.
- On-disk 3-artifact format (code/ description/ vectordb/), mineWoodLog description text -> [11]+[4]. GROUNDED.
- CurriculumAgent inputs/methods, warm_up schedule, 4-round retry cap -> [6]+[2]. GROUNDED.
- ActionAgent JS-via-Mineflayer, Babel parse, feedback fragments (execution error/chest/chat/critique) -> [7]+[2]. GROUNDED.
- CriticAgent self-verification, JSON {success,critique}, tuple return -> [5]. GROUNDED.
- LLM backbone GPT-4 gpt-4-0314 (action/critic) + GPT-3.5-turbo (NLP subtasks) -> [2]+[4]. GROUNDED.
- Benchmarks 3.3x / 2.3x / 63 items / wood 15.3x / stone 8.5x / iron 6.4x / diamond 1-of-3 @102 / zero-shot all-vs-none-in-50 -> [1]+[2]+[12]. GROUNDED but VENDOR/AUTHOR-REPORTED, unreproduced here; small-seed caveat is inferred from the diamond 1/3 figure.
- Stars 6943 / forks 673 / open_issues 8 / push 2024-04-03 -> [3]. GROUNDED.
- chromadb==0.3.29 pinned, langchain/openai unpinned; dependency-rot risk -> [9]. GROUNDED (rot risk is an inference from the pin pattern, labelled as such).
- SHARD comparison facts (staleness formula, T1-T4, DECLARE-REVIEW-EXECUTE, semantic-vs-syntactic conflict, composition.py, .bak/0-3 scoring, SHARD self-improvement is spec-only) -> taken from the requester's SHARD specification as given; NOT independently re-verified against SHARD's repo in this dossier. Labelled: SHARD-SPEC-AS-PROVIDED.
- UNGROUNDED: exact Semantic Scholar / Google Scholar citation COUNT for arXiv:2305.16291 — Semantic Scholar API returned HTTP 429 on two attempts; no number stated rather than cite from memory. (Influence is qualitatively HIGH but the integer is ungrounded.)
- UNGROUNDED: the published *venue* — the v2 is the citeable arXiv version; whether it appeared at a specific conference/journal was not confirmed from a primary source in this pass (search results showed only arXiv/HF/ADS mirrors, no venue field). Stated as arXiv-canonical only.
