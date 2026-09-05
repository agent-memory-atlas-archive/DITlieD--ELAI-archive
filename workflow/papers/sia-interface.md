Source: SIA (Self-Improving AI framework)
Reliability: MEDIUM (reviewed source, shallow clone of https://github.com/hexo-ai/sia, tag v0.5.1)
Files examined:
  F:\.ELAI_workflow\docs\.source\sia\pyproject.toml
  F:\.ELAI_workflow\docs\.source\sia\sia\orchestrator.py
  F:\.ELAI_workflow\docs\.source\sia\sia\cli.py
  F:\.ELAI_workflow\docs\.source\sia\sia\config.py
  F:\.ELAI_workflow\docs\.source\sia\sia\agent_impls\base.py
  F:\.ELAI_workflow\docs\.source\sia\sia\agent_impls\claude.py
  F:\.ELAI_workflow\docs\.source\sia\sia\profiles.py
  F:\.ELAI_workflow\docs\.source\sia\sia\providers.py
  F:\.ELAI_workflow\docs\.source\sia\sia\results.py
  F:\.ELAI_workflow\docs\.source\sia\sia\layout.py
  F:\.ELAI_workflow\docs\.source\sia\sia\run_setup.py
  F:\.ELAI_workflow\docs\.source\sia\sia\context_manager.py
  F:\.ELAI_workflow\docs\.source\sia\sia\agent_reference.py
  F:\.ELAI_workflow\docs\.source\sia\sia\prompts.py
  F:\.ELAI_workflow\docs\.source\sia\sia\web\server.py
  F:\.ELAI_workflow\docs\.source\sia\sia\tasks\gpqa\data\public\evaluate.py
  F:\.ELAI_workflow\docs\.source\sia\sia\tasks\lawbench\data\public\evaluate.py
  F:\.ELAI_workflow\docs\.source\sia\docs\architecture.md
  F:\.ELAI_workflow\docs\.source\sia\README.md

Package: sia-agent==0.5.1 (PyPI)
Paper: arXiv:2605.27276 "SIA: Self Improving AI with Harness & Weight Updates" (Hebbar et al., 2026)
Python: >=3.11


Public API:

orchestrator.main() | () -> None | None | CLI entry point; dispatches run or web sub-command
orchestrator.run_generation() | (current_gen, max_gen, run_setup, task_files, abs_dataset_dir, dataset_dir, meta_profile, sandbox, env_config, task_model, target_provider, focus, training_sandbox, resolved_ref) -> None | None | runs one full generation: target agent + eval + feedback agent
orchestrator.run_evaluation() | (gen_directory, task_dir, venv_dir, config) -> dict | dict | runs evaluate.py as subprocess with --gen-dir; returns status/log_path/results_path or error dict
orchestrator.load_agent_execution() | (gen_directory, config) -> tuple[dict|list, bool] | (data, is_multi_trajectory) | loads agent_execution.json or agent_execution/ folder; is_multi_trajectory=True when folder form
orchestrator.setup_run_directory() | -> RunSetup | RunSetup | creates run/venv/context; fails with sys.exit(1) if run_id dir already exists
orchestrator.load_task_files() | (task_dir, shared_dir, resolved_ref) -> TaskFiles | TaskFiles | reads task.md, sample descriptions, sample execution, reference seed
prompts.build_meta_prompt() | (task_files, task_model, working_dir, provider, reference_dir, focus, training_sandbox) -> str | str | builds the initial meta-agent prompt; OpenAI-compat provider prepends client-setup block
prompts.build_feedback_prompt() | (current_gen, max_gen, task_files, agent_py, task, execution_status, execution_section, run_dir, next_gen_dir, previous_gens, task_model, provider, requirements_dir, focus) -> str | str | builds feedback prompt; focus="weights" selects RL variant
agent_impls.base.run_agent() | (model_name, max_turns, prompt, agent_working_directory, agent_impl, provider) -> Awaitable[None] | coroutine | dispatches to registered impl; raises ValueError on unknown impl
agent_impls.base.register() | (name, runner) -> AgentRunner | AgentRunner | registers an async runner; called at module import time by each impl module
profiles.load_meta_agent_profile() | (name_or_path) -> MetaAgentProfile | MetaAgentProfile | reads JSON profile; sys.exit on missing keys or impl/provider mismatch
profiles.load_target_agent_profile() | (name_or_path) -> TargetAgentProfile | TargetAgentProfile | reads JSON profile; sys.exit on missing keys
providers.load_provider() | (name_or_path) -> Provider | Provider | reads JSON provider; sys.exit on invalid client_kind
web.server.create_app() | (runs_dir) -> FastAPI | FastAPI | REST API serving run artifacts; routes: /api/runs, /api/runs/{run}/gens/{gen}/eval, /api/runs/{run}/gens/{gen}/artifact/{label}, /api/runs/{run}/gens/{gen}/trajectory/{qid}
web.server.serve_in_background() | (host, port, runs_dir) -> Thread|None | Thread|None | starts uvicorn in daemon thread; silently returns None if fastapi/uvicorn missing
config.Config.from_env() | () -> Config | Config | reads SIA_* env vars; silently ignores invalid values via contextlib.suppress
run_setup.install_requirements() | (venv_dir, requirements_path) -> None | None | runs uv pip or pip install -r; subprocess.run(check=True) so raises CalledProcessError on failure
agent_reference.resolve_agent_reference() | (ref, task_layout) -> ResolvedAgentReference | ResolvedAgentReference | resolves AgentReference to inline seed or ref_dir; sys.exit if entrypoint not found
agent_reference.copy_reference_into() | (resolved, gen_dir) -> None | None | copies ref_dir tree or sibling requirements.txt into gen working dir


Key dataclasses / types:

Config | sia.config | dataclass; all fields have defaults; override via SIA_* env vars or from_env()
TaskFiles | sia.run_setup | sample_task_descriptions:str, reference_target_agent_py:str, sample_agent_execution:dict, task_md:str
RunSetup | sia.run_setup | run_directory:str, meta_agent_working_directory:str, venv_dir:str, context_mgr:ContextManager
MetaAgentProfile | sia.profiles | frozen dataclass: profile_id, name, agent_impl, model, provider:Provider
TargetAgentProfile | sia.profiles | frozen dataclass: profile_id, name, model, provider:Provider, agent_reference:AgentReference
Provider | sia.providers | frozen dataclass: provider_id, name, client_kind (anthropic|openai|google), base_url:str|None, api_key_env:str
AgentReference | sia.agent_reference | frozen dataclass: kind (default|file|dir), source:Path|None, entrypoint:str|None
ResolvedAgentReference | sia.agent_reference | inline_seed:str|None, ref_dir:Path|None, entrypoint:str, requirements:Path|None
TargetAgentResult | sia.results | success:bool, stdout:str, stderr:str, error_msg:str; as_tuple() -> (bool,str,str,str)
FeedbackContext | sia.results | execution_status:str, execution_section:str; as_tuple() -> (str,str)
RunLayout | sia.layout | frozen dataclass; builds paths under run_dir; key: gen_dir(n), venv_dir, target_agent(n), stdout_log(n, focus), agent_execution_dir(n)
TaskLayout | sia.layout | frozen dataclass; builds paths under task_dir+shared_dir; key: dataset_dir, abs_dataset_dir, sample_descriptions, reference_agent, evaluate_script()


Behavioral contracts:

main() -> loads Config.from_env(), parses argv, starts web thread unless --no-web, builds meta prompt, runs meta agent via asyncio.run(run_agent(...)), then loops run_generation() 1..max_gen
run_generation() -> installs gen requirements.txt if present, runs target agent subprocess, calls run_evaluation(), calls ContextManager.add_generation(), if not last gen runs _run_feedback_agent()
_run_target_agent() -> on subprocess exit!=0: logs error but returns (False,...) and CONTINUES (does not abort); target agent failure does not stop the feedback cycle
run_evaluation() -> runs evaluate.py as subprocess with --gen-dir; if evaluate.py absent: returns {"status":"skipped"}; if results.json not created: returns {"status":"warning"}; timeout=600s default
_run_feedback_agent() -> reads target_agent.py or train.py from gen_dir, builds feedback prompt, copies reference into next_gen_dir, saves prompt to feedback_agent_prompt.txt, calls asyncio.run(run_agent(...))
ContextManager.add_generation() -> appends to context.md; for gen>=2 calls _generate_llm_summary() which spawns a second run_agent() call with max_turns=5 in a temp dir; this is a live LLM call, not free
setup_run_directory() -> creates run dir; sys.exit(1) if dir already exists (no resume); creates venv via uv if available else stdlib venv; installs VENV_PACKAGES at setup time
load_agent_execution() -> detects multi-trajectory by checking if agent_execution/ folder exists; loads up to MAX_EXECUTION_LOG_SIZE bytes per file; oversized files return {"error":"File too large"}
claude impl (run_agent_claude) -> uses claude_agent_sdk.query() with permission_mode="bypassPermissions"; allowed_tools=["Bash","Read","Write","Edit","Glob"]; provider arg accepted but IGNORED (always uses ANTHROPIC_API_KEY)
openhands impl -> separate optional dep openhands-ai>=1.6.0; multi-provider aware (unlike claude impl)
pydantic-ai impl -> separate optional dep pydantic-ai>=1.0; multi-provider aware
target_agent.py contract (generated code) -> must accept --dataset_dir and --working_dir args; must write agent_execution.json or agent_execution/ folder; dataset_dir is READ-ONLY; working_dir is READ-WRITE
evaluate.py contract -> must accept --gen-dir arg; must write results.json to gen_dir on success; exit code 0 = success, nonzero = failure; orchestrator reads results.json to populate metrics
weights mode (focus="weights") -> generates train.py instead of target_agent.py; requires TINKER_API_KEY; optionally MODAL_TOKEN_ID+MODAL_TOKEN_SECRET (modal sandbox) or SANDBOX_URL (sandboxfusion); early-stop when COMPLETED file appears in next_gen_dir


Platform constraints:

Python >= 3.11 required (uses native async trait syntax, match not used but walrus etc.)
venv python path is hardcoded as venv/bin/python (POSIX path); will fail on Windows without WSL (no bin/ on Windows venv)
uv preferred over stdlib venv for speed; falls back silently to stdlib venv if uv not on PATH
Docker sandbox (--sandbox docker) requires Docker on PATH; mounts dataset_dir read-only, working_dir read-write, network=none
context.md LLM summary call uses asyncio.run() inside add_generation() which is called from a synchronous context; will raise RuntimeError if already inside a running event loop
live dashboard uses threading.Thread (daemon=True); uvicorn and fastapi are NOT in core deps -- optional install only
claude impl ignores provider arg entirely; cannot route to non-Anthropic endpoints even if a provider with client_kind=openai is passed; validated and rejected at profile load time
run_id directory must not exist; no resume/append; --run_id must be a new integer each run


Gotchas:

run_setup.setup_run_directory() calls sys.exit(1) if run dir exists -- not an exception, a process exit; cannot be caught by callers
evaluate.py result detection: orchestrator checks for results.json in gen_dir; if evaluate.py writes elsewhere it silently returns status=warning and no metrics flow to context.md
metrics extraction fallback chain: results.json > detailed_results.json > stdout regex; regex patterns are case-insensitive but brittle (accuracy/correct/total patterns only)
agent_execution multi-trajectory: only first 3 trajectories are shown in feedback prompt; rest available via path reference in the prompt; feedback agent must read them via tools
run_agent() in claude impl does NOT return trajectory data to caller; trajectories go to ~/.claude/projects/ (Claude Code SDK behavior) and to agent_execution.json only if the generated target_agent.py writes them
requirements.txt evolution: if the profile's agent_reference is "default" or a single file with no sibling requirements.txt, the feedback agent cannot add dependencies unless it also creates requirements.txt in next_gen_dir (orchestrator copies it forward only if resolved_ref.requirements is not None initially)
weights mode GRPO: reward constant at 0.0 or 0.5 is a training failure (zero variance = failed GRPO); implement majority voting when ground truth is missing (per embedded RL guide in prompts.py)
Config.EVAL_TIMEOUT default 600s; large datasets or slow models will timeout and return status=error
sandbox=docker with --network none: target agents cannot make network calls; any agent that calls an external API will silently fail or raise inside the container
claude impl max_turns is passed as str (not int) to ClaudeAgentOptions; this is intentional per the SDK's API


Critical assertions (testable):

assert provider.client_kind in ("anthropic", "openai", "google"), "provider client_kind must be one of three valid values"
assert not (meta_profile.agent_impl == "claude" and meta_profile.provider.client_kind != "anthropic"), "claude impl requires anthropic provider"
assert os.path.isdir(run_setup.run_directory), "run directory must exist after setup_run_directory()"
assert not os.path.exists(os.path.join(gen_dir, "agent_execution.json")) or os.path.getsize(os.path.join(gen_dir, "agent_execution.json")) <= 50_000_000, "execution log must not exceed MAX_EXECUTION_LOG_SIZE"
assert task_files.task_md != "", "task.md must be non-empty (required for meta prompt construction)"
assert resolved_ref.inline_seed is not None or resolved_ref.ref_dir is not None, "resolved reference must have either inline seed or ref_dir"
