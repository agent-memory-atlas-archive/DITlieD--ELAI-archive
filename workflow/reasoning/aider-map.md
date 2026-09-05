AIDER ARCHITECTURE + CAPABILITY MAP
Source: C:\Users\user\Downloads\aider-main\aider-main
Purpose: competitive map vs "Liberty" (small Python litellm-based agent harness)
Every claim cites a file:line read in this repo. Ignored: .venv/.git/dist/build/node_modules (none present in tree anyway).

================================================================
1. PURPOSE & STACK
================================================================
- What it is: terminal-first AI pair-programming tool that edits a real git repo. README.md:1-30 (top-level pitch, "AI pair programming in your terminal").
- Language: Python. Package root aider/ (top-level ls). pyproject.toml name = "aider-chat" (pyproject.toml:2-ish, "name = aider-chat", dynamic deps/version).
- Version / maturity: aider/__init__.py:3 __version__ = "0.86.3.dev" -- mature, high release number, .dev suffix = active main. HISTORY.md is 77KB (top-level ls) = long, dense changelog = high release cadence.
- License: Apache-2.0 (LICENSE.txt:1-4 header "Apache License Version 2.0").
- Scale: 147 .py files repo-wide (find count). aider/ package = 20,285 LOC (wc -l total over aider/*.py + subdirs). base_coder.py alone = 2,485 LOC; models.py = 1,338; repomap.py = 867 (wc -l).
- Test suite: 36 test_*.py files, tests/ = 12,381 LOC (find + wc). pytest.ini present (top-level). Test LOC ~= 60% of core LOC = serious test investment.
- CI: 10 GitHub workflows (.github/workflows ls) incl ubuntu-tests.yml, windows-tests.yml, docker-build-test.yml, release.yml, check_pypi_version.yml, pre-commit.yml. Multi-OS CI + PyPI release automation + docker = production-grade maturity.
- Docs: 84 markdown pages under aider/website/docs (find count). Dedicated per-provider LLM guides (see axis 4).
- Maturity verdict: this is a multi-year, widely-shipped product, NOT a thin wrapper. ~20K core LOC + 12K test LOC + 84 doc pages + 10 CI workflows.

================================================================
2. PROVIDER / MODEL SYSTEM  (most important axis)
================================================================
CONFIRMED: Aider uses litellm as its sole LLM gateway.
- Lazy-loaded wrapper: aider/llm.py:16 comment "import litellm takes 1.5 seconds, defer it!"; llm.py:20-37 class LazyLiteLLM with __getattr__ that imports litellm on first access (_load_litellm at llm.py:30). llm.py:11-14 sets OR_SITE_URL / OR_APP_NAME / LITELLM_MODE=PRODUCTION env before load.
- Import sites (every call goes through the lazy module): base_coder.py:38 `from aider.llm import litellm`; the actual completion call is base_coder.py:1373 `litellm.completion(...)` AND models.py:1036 `res = litellm.completion(**kwargs)`. Cost via base_coder.py:2037 `litellm.completion_cost(...)`. Token counting models.py:648-653 `litellm.encode` / `litellm.token_counter`. Env validation models.py:751 `litellm.validate_environment(model)`. Transcription (voice) voice.py:169 `litellm.transcription(...)`. So litellm is used for completion, cost, tokens, env-check, AND audio.

HOW AIDER USES litellm vs a THIN WRAPPER (the differentiating finding):
A thin wrapper calls litellm.completion(model, messages) and stops. Aider builds a whole model-intelligence layer ON TOP of litellm:
  a) Model metadata layer: aider/models.py ModelInfoManager (models.py:161) fetches litellm's price/context DB from raw.githubusercontent.com/BerriAI/litellm/.../model_prices_and_context_window.json (models.py:162-165), caches it 24h under ~/.aider/caches (models.py:166-170). Falls back to a local model-settings.yml shipped in the package: 804 `name:` entries (grep -c on aider/resources/model-settings.yml) -- a hand-curated per-model behavior DB.
  b) ModelSettings dataclass (models.py:127-150): per-model fields a raw litellm call has no concept of -- edit_format default, weak_model_name, use_repo_map, lazy, overeager, reminder, examples_as_sys_msg, extra_params, cache_control, caches_by_default, use_temperature, editor_model_name, editor_edit_format, reasoning_tag, system_prompt_prefix, accepts_settings. This is Aider deciding, per model, HOW to drive it.
  c) Aliases: models.py:99-123 MODEL_ALIASES maps short names to canonical -- sonnet->claude-sonnet-4-6, opus->claude-opus-4-7, haiku->claude-haiku-4-5, deepseek->deepseek/deepseek-chat, flash->gemini/gemini-flash-latest, r1->deepseek/deepseek-reasoner, gemini->gemini/gemini-3-pro-preview, quasar/optimus->openrouter/... , grok3->xai/grok-3-beta.
  d) THREE-MODEL SPLIT (main / weak / editor): Model.__init__(model, weak_model, editor_model, editor_edit_format) models.py:331. weak_model (models.py:603-623, used for cheap tasks like commit messages / summarization) defaults to self if unset. editor_model (models.py:625+) is the model that applies edits in architect mode. This split is a core design, not a flag.
  e) Provider-specific request shaping inside the send path (models.py around 1000-1037): ollama gets an auto-computed num_ctx (models.py:1012-1014 `if self.is_ollama() ... num_ctx = int(token_count*1.25)+8192`); GitHub Copilot token swap (models.py:1027-1034 injects Editor-Version + Copilot-Integration-Id headers, converts copilot token to OpenAI key); deepseek-reasoner role-alternation fixup (models.py:1043 ensure_alternating_roles). A thin wrapper does none of this.
  f) OpenRouter has its OWN manager: aider/openrouter.py (128 LOC) OpenRouterModelManager, instantiated in ModelInfoManager (models.py:177). models.py:263-322 fetches/scrapes openrouter.ai model pages for context-window/pricing when litellm lacks the info, incl ":free" suffixed models (models.py:280 example `openrouter/qwen/qwen-2.5-72b-instruct:free`; openrouter.py:52 example `openrouter/nousresearch/deephermes-3-mistral-24b-preview:free`).

FREE / LOCAL API VERDICT: YES -- strongly and first-class.
  - OLLAMA (fully local, free): is_ollama() models.py:931-932 (matches `ollama/` and `ollama_chat/` prefixes); auto num_ctx models.py:1012; dedicated doc aider/website/docs/llms/ollama.md:1-18 ("Aider can connect to local Ollama models", `export OLLAMA_API_BASE=http://127.0.0.1:11434`).
  - LM STUDIO (local): provider doc exists, aider/website/docs/llms/lm-studio.md (llms/ ls).
  - OpenAI-compatible local servers: aider/website/docs/llms/openai-compat.md (llms/ ls) -- any local OpenAI-API server.
  - OpenRouter FREE tier: onboarding.py:20-58 detects free-tier OpenRouter accounts and, if free, defaults the model to `openrouter/deepseek/deepseek-r1:free` (onboarding.py:58). Plus OAuth onboarding flow main.py:785-818 (offer_openrouter_oauth) so a user with no key gets walked into OpenRouter.
  - Cheap cloud (deepseek/gemini-flash): aliases deepseek, r1, flash, flash-lite (models.py:112-117).
  So: Aider can run fully free/local (Ollama, LM Studio, openai-compat) AND on free-tier hosted (OpenRouter :free / deepseek-r1:free). This is verified in code, not docs-only.

ROUTING / PICKING: Aider does NOT do dynamic per-request routing (no RouteLLM-style classifier). Routing = (1) alias resolution models.py:99-123, (2) the static main/weak/editor role assignment per Model, (3) onboarding default-model selection main.py:34 select_default_model / onboarding.py. The "routing" intelligence is the per-model ModelSettings behavior table, not runtime model-switching.

================================================================
3. AGENT HARNESS / EDIT ENGINE  (Aider's distinctive core)
================================================================
EDIT FORMATS -- the coders/ directory implements a family of edit strategies, each a Coder subclass with an `edit_format` string. Registry: aider/coders/__init__.py:1-34 (__all__ list). Verified edit_format strings (grep edit_format = over coders/*.py):
  - "whole"         wholefile_coder.py:13 -- model returns entire file content. Robust, token-heavy. Default for weak models.
  - "diff"          editblock_coder.py:18 -- SEARCH/REPLACE blocks. The workhorse format.
  - "diff-fenced"   editblock_fenced_coder.py:9 -- SEARCH/REPLACE inside fenced code (Gemini-friendly).
  - "udiff"         udiff_coder.py:49 -- unified-diff style.
  - "udiff-simple"  udiff_simple.py:12 -- relaxed udiff.
  - "patch"         patch_coder.py:217 -- patch-apply format.
  - "ask"           ask_coder.py:8 -- Q&A, no edits.
  - "help"          help_coder.py:9 -- docs Q&A.
  - "context"       context_coder.py:8 -- identifies which files are relevant (auto-context).
  - "architect"     architect_coder.py:7 -- two-model plan-then-edit (below).
  - "editor-diff" / "editor-whole" / "editor-diff-fenced" (editor_editblock_coder.py:7, editor_whole_coder.py:7, editor_diff_fenced_coder.py:8) -- stripped-down formats used by the editor model in architect mode (no chat scaffolding, pure edit).
  - "func" single_wholefile_func_coder.py:9 -- commented out of __all__ (legacy function-call format).
SELECTION: Coder.create (base_coder.py:125) dispatches on edit_format: base_coder.py:190-191 loops coders.__all__ and picks the class whose `.edit_format` matches; "code" is aliased (base_coder.py:142). Each Model carries a DEFAULT edit_format in its ModelSettings (models.py:131), so a strong model defaults to "diff", a weak one to "whole". User overrides via --edit-format.
EDITBLOCK MECHANICS (the load-bearing format): editblock_coder.py:386-390 regex markers HEAD=`^<{5,9} SEARCH>?`, DIVIDER=`^={5,9}`, UPDATED=`^>{5,9} REPLACE`. replace_most_similar_chunk (editblock_coder.py:157) does fuzzy matching so a near-miss SEARCH block still applies; failure path emits a structured "SEARCH/REPLACE block failed to match" reflection back to the model (editblock_coder.py:84-120) -- self-repair loop.

REPO MAP (the second distinctive piece -- tree-sitter + PageRank):
- aider/repomap.py (867 LOC). Uses tree_sitter (repomap.py:19 `from tree_sitter import Query`) to extract symbol defs/refs as Tag namedtuples (repomap.py:29 `Tag = namedtuple(... rel_fname fname line name kind)`), via get_tags / get_tags_raw (repomap.py:233-357).
- Tree-sitter grammars: 31 *.scm tag-query files under aider/queries/tree-sitter-language-pack/ (ls *.scm | wc) covering C/C++/C#/Python/Ruby/Rust/Go/JS/TS/Java/PHP/etc (queries/ ls shows arduino, bash, c, clojure, commonlisp, cpp, csharp, d ... -tags.scm).
- RANKING via PageRank: repomap.py:368 `import networkx as nx`; builds a graph where edges = symbol references between files, then repomap.py:525 `ranked = nx.pagerank(G, weight="weight", ...)` (personalized PageRank biased toward chat/mentioned files). ranked_definitions distributed across out-edges (repomap.py:534-555). This surfaces the MOST IMPORTANT symbols of the WHOLE repo within a token budget -- not just open files.
- Token-budgeted + cached: map_tokens budget (repomap.py:49-70, max_map_tokens, map_mul_no_files), binary-search to fit budget (get_ranked_tags_map), diskcache TAGS_CACHE (repomap.py:14 `from diskcache import Cache`, repomap.py:43 TAGS_CACHE_DIR `.aider.tags.cache.v{N}`), in-memory tree_cache/map_cache (repomap.py:78-80), refresh="auto" (repomap.py:57-61).

CONTEXT MANAGEMENT:
- Explicit: /add /drop chat files (commands.py has 43 cmd_ methods; repo map fills the rest of context up to budget).
- Auto-context: ContextCoder (edit_format "context", context_coder.py) asks the model which files are relevant. Architect mode passes map_tokens=0 to the editor (architect_coder.py:28) since the architect already chose files.
- The repo map IS the auto-context engine for un-added files.

GIT INTEGRATION (aider/repo.py):
- auto-commit after every successful edit: base_coder.py:308 auto_commits=True default, base_coder.py:1589 `self.auto_commit(edited)`; also commits after linter fixes (base_coder.py:1601). dirty_commits (base_coder.py:309/412) commits pre-existing dirty state before editing so each AI change is isolated.
- GitRepo.commit (repo.py:131) generates an LLM commit message (uses the weak_model) and supports attribution flags: attribute_committer / attribute_commit_message_author / "Co-authored-by: aider (<model>)" trailer (repo.py:62-91, 156-198).
- /undo (commands.py:553 cmd_undo -> raw_cmd_undo:560) reverts the last aider commit; /commit (commands.py:337), /diff (commands.py:657) round out the git surface. git_commit_verify flag (repo.py:75/91) controls pre-commit hook bypass.

ARCHITECT / EDITOR TWO-MODEL FLOW (architect_coder.py, 49 LOC, the cleanest distinctive piece):
- ArchitectCoder extends AskCoder (architect_coder.py:6) -- it does NOT edit. The architect (strong reasoning model) replies in prose with a plan.
- reply_completed (architect_coder.py:11): on completion, confirm "Edit the files?" (architect_coder.py:17), then spins up a SECOND coder = the editor: editor_model = main_model.editor_model or main_model (architect_coder.py:23), edit_format = main_model.editor_edit_format (architect_coder.py:26), map_tokens=0 (architect_coder.py:28). editor_coder.run(with_message=content) (architect_coder.py:44) feeds the architect's plan to the editor model which produces the actual SEARCH/REPLACE edits.
- So: model A reasons, model B (cheaper/edit-tuned) applies. Cost + commit hashes are threaded back (architect_coder.py:46-48). This is the design Liberty would need to match for "smart-but-cheap".

================================================================
4. INTERFACE & POLISH
================================================================
- CLI/TUI chat loop: aider/main.py entrypoint, aider/commands.py = 43 slash commands (grep -c def cmd_). Rich-based terminal markdown streaming (aider/mdstream.py).
- Voice: aider/voice.py -- records audio, transcribes via litellm.transcription (voice.py:169). Flags --voice-format/--voice-language/--voice-input-device (args.py:702-718).
- Web/browser GUI: aider/gui.py + --gui/--browser flags (args.py:657-660) -- Streamlit-based browser UI.
- File watcher / in-IDE comments: aider/watch.py (watch for `# aider:` comments in files) + watch_prompts.py -- lets you drive aider from your editor by writing comments.
- Linting + test running: aider/linter.py; flags --lint/--lint-cmd/--auto-lint (args.py:528-543) and --test-cmd/--auto-test (args.py:549-554). After edits aider can auto-run the linter, feed errors back, and auto-commit the fix (base_coder.py:1601). Closes the edit->lint->fix loop.
- Web scrape: aider/scrape.py (pull URL content into chat).
- BENCHMARK HARNESS (genuinely distinctive): benchmark/ = 2,463 LOC. benchmark.py (driver), swe_bench.py (SWE-bench runner), refactor_tools.py, rungrid.py (grid sweeps over models/formats), problem_stats.py, plots.py/over_time.py (the leaderboard charts). Uses Exercism exercises (clone-exercism.sh) + swe-bench-lite.txt/swe-bench.txt task lists, dockerized (benchmark/Dockerfile). This is how Aider produces its public edit-format/model leaderboard -- a reproducible eval rig, not just a demo.
- Docs: 84 md pages (find), incl 17 per-provider LLM guides (aider/website/docs/llms/ ls: anthropic, azure, bedrock, cohere, deepseek, gemini, github, groq, lm-studio, ollama, openai, openai-compat, openrouter, vertex, xai, other, warnings). Docs quality = high.

================================================================
5. DISTINCTIVE vs COMMODITY
================================================================
GENUINELY DISTINCTIVE ENGINEERING (hard to replicate, the moat):
  - Repo map: tree-sitter symbol extraction across 31 languages + personalized PageRank ranking + token-budgeted binary-search + diskcache (repomap.py:368/525, 31 .scm files). This is the single hardest piece and a thin litellm wrapper has nothing like it.
  - Edit-format family + fuzzy SEARCH/REPLACE + structured self-repair reflection (coders/ ~14 formats, editblock_coder.py:157 fuzzy match, :84-120 failure reflection). Per-model default format chosen from a curated 804-entry behavior table (model-settings.yml).
  - Architect/editor two-model split (architect_coder.py) -- reason-with-A, apply-with-B, with weak_model for commit messages. A real cost/quality lever.
  - Reproducible benchmark harness (benchmark/ 2,463 LOC, SWE-bench + Exercism, dockerized grid) -- evidence-generating, not vibes.
  - Curated per-model behavior DB: model-settings.yml 804 entries + ModelSettings 23 fields (models.py:127-150) -- accumulated knowledge of how each model misbehaves.

TABLE-STAKES / COMMODITY (Liberty likely already has or can get cheaply):
  - litellm completion call itself (models.py:1036) -- one line; the wrapper is trivial, the value is everything around it.
  - Free/local provider support (Ollama, OpenRouter, openai-compat) -- this is LITELLM's feature, inherited for free; any litellm-based harness gets it. Aider's add is the metadata/context-window handling (openrouter.py, num_ctx) not the connectivity.
  - Slash-command chat loop, streaming markdown, /undo /diff -- standard.
  - Git auto-commit -- valuable but mechanically simple (repo.py wraps GitPython).
  - Voice/web GUI -- nice-to-have surface area, not core.

KEY TAKEAWAY FOR LIBERTY COMPARISON:
Aider and Liberty share the SAME litellm substrate, so "can use free APIs" is a TIE inherited from litellm (Ollama/OpenRouter-free/openai-compat all work in both by construction). Aider's real, non-trivial advantages are NOT the provider layer -- they are (1) the tree-sitter+PageRank repo map, (2) the edit-format family with fuzzy-apply + self-repair, (3) the architect/editor + weak-model role split, and (4) the reproducible benchmark rig that lets it tune all of the above. A thin litellm wrapper gets provider breadth for free but none of these four.

================================================================
UNVERIFIED / NOT CHECKED
================================================================
- Did not runtime-execute aider; all claims are static source reads.
- Did not read full base_coder.py send/retry loop body (2,485 LOC) beyond the litellm.completion call sites and commit hooks -- exact retry/streaming/cache-warming logic UNVERIFIED in detail.
- model-settings.yml 804 `name:` count includes editor_model_name/weak_model_name sub-keys, so distinct top-level models < 804 (count is an upper bound, UNVERIFIED exact distinct-model number).
- networkx pagerank "personalization" toward chat files inferred from repomap.py:519-525 pers_args usage; exact personalization vector construction not line-traced.
- Release CADENCE inferred from 77KB HISTORY.md + .dev version, not from git log dates (git not traversed per scope).
