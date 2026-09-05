Source: google/agents-cli
Reliability: HIGH (official Google open-source repo, Apache-2.0, actively maintained, last commit 2026-06-22)
Version: 0.5.1
Files examined:
  src/google/agents/cli/main.py
  src/google/agents/cli/_trust.py
  src/google/agents/cli/_runner.py
  src/google/agents/cli/_project.py
  src/google/agents/cli/_experiments.py
  src/google/agents/cli/_skills_check.py
  src/google/agents/cli/eval/cmd_eval_group.py
  src/google/agents/cli/eval/_inference_runner.py
  src/google/agents/cli/eval/_synthesize_runner.py
  src/google/agents/cli/eval/cmd_grade.py
  src/google/agents/cli/eval/cmd_optimize.py
  src/google/agents/cli/eval/cmd_submit.py
  src/google/agents/cli/eval/optimize_utils.py
  src/google/agents/cli/deploy/cmd_deploy.py
  src/google/agents/cli/run/cmd_run.py
  src/google/agents/cli/setup/cmd_setup.py
  src/google/agents/cli/_tools.py
  .claude-plugin/plugin.json

What this is:
  The official Google CLI for the Gemini Enterprise Agent Platform (GEAP) / ADK lifecycle.
  It is NOT a general-purpose agent runner; it is a DevOps / MLOps toolchain specifically for
  ADK (Agent Development Kit) agents deployed to Google Cloud (Vertex AI Agent Runtime,
  Cloud Run, GKE). Target model family: Gemini (all generation via Vertex AI / AI Studio).
  It bundles 7 SKILL.md files (agentskills format) that get injected into coding agents
  (Claude Code, Gemini CLI, Codex) to enable those agents to drive the lifecycle CLI.

Public API (CLI surface):
  main | Click group (LazyGroup) | entry: agents-cli | root command dispatcher
  setup | cmd_setup | installs 7 skills to detected coding agents via npx skills@1.4.8
  update | cmd_update | force-reinstall skills
  login | cmd_auth | Google Cloud ADC or AI Studio auth
  scaffold (create) | cmd_scaffold_group / create | cookiecutter templates: adk, adk_a2a, agentic_rag; outputs agents-cli-manifest.yaml
  playground | cmd_playground | starts local ADK dev server
  run MESSAGE | cmd_run | single-turn query; auto-starts local server or routes to --url; supports mode=adk (ADK SSE) or mode=a2a (Agent-to-Agent protocol via a2a SDK)
  lint | cmd_lint | runs ruff / project-specific linting via uv
  install | cmd_install | uv sync
  data-ingestion | cmd_data_ingestion | triggers Vertex Pipelines RAG ingestion for agentic_rag template
  eval generate | _inference_runner.py staged into project; calls vertexai.Client.evals.run_inference per case; fresh agent reload per case to avoid asyncio.Lock cross-loop contamination
  eval grade | cmd_grade | loads traces JSON, calls vertexai.Client.evals.evaluate with LLMMetric / custom_function / CodeExecutionMetric metrics; local custom metrics run in-process
  eval run | chains generate+grade
  eval dataset | manage trace files (synthesize = LlmBackedUserSimulator via ADK EvaluationGenerator)
  eval metric | discover/manage metrics
  eval compare | diff two result JSON files
  eval analyze | cluster failures from result JSON
  eval optimize | cmd_optimize | translates EvaluationDataset to ADK EvalSet, then calls "uv run adk optimize" with GEPA/ProTeGi/BayesianSearch/RandomSearch/MetaPrompt/PromptWizard samplers
  eval submit | cmd_submit | submits eval run to Vertex AI Eval Service cloud-side (asynchronous, GCS-backed output)
  eval results | fetches cloud eval run results
  deploy | cmd_deploy | dispatches to agent_runtime / cloud_run / gke; supports --no-wait, --status, PSC network attachment + DNS peering for Agent Runtime
  publish | cmd_publish_group | publish agents to various targets
  infra | cmd_infra | Terraform + Cloud CI-CD provisioning
  info | cmd_info | show project config, paths, CLI version

Key datastructures:
  ProjectConfig | dataclass | read from agents-cli-manifest.yaml (primary) or pyproject.toml [tool.agents-cli] (legacy fallback)
    fields: project_name, deployment_target (none/agent_runtime/cloud_run/gke), agent_directory, is_a2a, requires_data_ingestion, region, base_template, acli_version, language, datastore, session_type, cicd_runner, agent_guidance_filename
  EvaluationDataset | vertexai._genai.types.common.EvaluationDataset | Pydantic model; JSON wire format {"eval_cases": [...]}
  EvalCase | vertexai.types.EvalCase | per-case struct; agent_data.turns[].events; responses[].response
  Experiment | NamedTuple(label, value_type, default_value) | feature-flag registry; env override via AGENTS_CLI_EXPERIMENTS JSON dict

Behavioral contracts:
  _runner.run() -> streams subprocess output by default; capture=True to capture; check=True raises ClickException on nonzero exit; env merged with os.environ
  _runner.run_resolved() -> resolves executable via _tools.require_tool before subprocess.run; raises ToolNotFoundError if missing
  _runner.popen_resolved_detached() -> cross-platform detached process: POSIX uses start_new_session=True; Windows uses CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP; stdin must be DEVNULL
  _trust.require_confirmation() -> decorator; -i flag prompts user; -y / --auto-approve skips; no flags = silent proceed (programmatic mode)
  _inference_runner.main() -> staged script: fresh agent loaded per case (AgentLoader.remove_agent_from_cache then load_agent) to avoid asyncio.Lock cross-event-loop errors; zero-success = exit 1 and no artifact written; partial success = artifact written with successful cases only + stderr summary
  _inference_runner._ensure_eval_compatible() -> monkey-patches missing agent.tools=[] on SequentialAgent/ParallelAgent/LoopAgent (workaround for googleapis/python-aiplatform#6865)
  cmd_optimize -> translates EvaluationDataset to ADK EvalSet format in .tmp/ subdir; calls "uv run adk optimize" with sampler_config_file_path; cleans up .tmp on exit (finally block)
  cmd_deploy (agent_runtime) -> calls deploy_agent_runtime() from agent_runtime.py; PSC network attachment requires --network-attachment flag; --num-workers is Agent Runtime only
  cmd_deploy (cloud_run) -> constructs gcloud run deploy; auto-injects AGENT_VERSION from pyproject.toml; auto-injects APP_URL from gcloud projects describe; --service-name forbidden on GKE (Terraform owns resource names)
  cmd_deploy (gke) -> terraform init + apply (targeted resources list) -> docker build -> kubectl set image -> kubectl set env -> kubectl rollout status; --no-wait not supported for GKE
  cmd_run (local) -> ensure_server() starts or reuses a background ADK server (pidfile-based); always mode=adk (ADK SSE); dispatches to http://127.0.0.1:{port}
  cmd_run (remote --url) -> mode required; a2a mode uses a2a.client.ClientFactory + AgentCard from /.well-known/agent.json; adk mode uses ADK SSE endpoint; auto-selects access_token (Agent Runtime URLs) vs id_token (Cloud Run/GKE)
  check_skills_version() -> rate-limited to once per 12h via ~/.agents/.acli_skills_check stamp; scans ~/.agents/skills/google-agents-cli-*/SKILL.md for version frontmatter; warns on mismatch; never blocks

Platform constraints:
  Python 3.11+ required (uses tomllib stdlib, match-based type hints)
  uv required as package manager (all subprocess calls use uv run / uv sync)
  npx (Node.js) required for skills installation (npx skills@1.4.8 list/install)
  gcloud SDK required for cloud_run and GKE deployments
  kubectl required for GKE
  terraform required for GKE local dev (not CI/CD mode)
  vertexai SDK (google-cloud-aiplatform) required for eval and agent_runtime deploy
  a2a Python SDK required for run --mode a2a
  GOOGLE_CLOUD_PROJECT / GOOGLE_CLOUD_LOCATION env vars consumed by vertexai.Client
  AGENTS_CLI_EXPERIMENTS env var: JSON dict for feature-flag overrides
  CLOUDSDK_CORE_DISABLE_PROMPTS=1 injected automatically (suppresses gcloud interactive prompts)
  Windows: popen_resolved_detached uses CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP
  POSIX: popen_resolved_detached uses start_new_session=True

Gotchas:
  Agent reload per inference case is mandatory: ADK AgentLoader caches agents; stale locks from prior event loops cause silent failures on subsequent cases. The CLI explicitly calls remove_agent_from_cache before each load_agent call.
  SequentialAgent/ParallelAgent/LoopAgent have no .tools attribute; vertexai eval SDK calls agent.tools unconditionally (bug aiplatform#6865). The CLI patches object.__setattr__(agent, "tools", []) on affected agents recursively before eval.
  thought_signature fields from Gemini are stripped from eval events before artifact write; leaving them in causes downstream grade failures.
  GKE service names are Terraform-owned (var.project_name); --service-name is rejected for GKE deploys to prevent kubectl/Terraform state divergence.
  Local custom metrics (custom_function) run in-process with full CLI privileges; the CLI warns the user explicitly.
  CodeExecutionMetric runs server-side in a Vertex AI sandbox; requires GCP project + region even though local metrics do not.
  eval optimize writes sampler_config.json and optional optimizer_config.json to .tmp/ in the agent dir; these are cleaned up in a finally block but may persist if the process is killed.
  cmd_run auto-detects Agent Runtime URLs vs Cloud Run/GKE by checking for "aiplatform.googleapis.com" in the URL to select the correct token type (access vs identity).
  Skills version check uses a 12h stamp file; first run after install always checks.
  Manifest format: agents-cli-manifest.yaml is primary; pyproject.toml [tool.agents-cli] is a legacy fallback that emits a deprecation warning.

Critical assertions (testable):
  assert cfg.deployment_target in ("none", "agent_runtime", "cloud_run", "gke")
  # Fresh agent reload per case: cache must be cleared
  # assert loader.remove_agent_from_cache called before load_agent on each inference case
  # Zero-success invariant: if n_succeeded == 0, no artifact file is written, exit code is 1
  assert n_succeeded == 0 or Path(output_path).exists()  # artifact written iff success > 0
  # PSC DNS peering requires network_attachment
  assert not (dns_peering_domain and not network_attachment)
  # --service-name rejected for GKE
  assert not (service_name_override and cfg.deployment_target == "gke")
  # mode required when using --url
  assert not (url and not mode)

Staleness verdict:
  NOT STALE. Last commit 2026-06-22 (3 days ago from 2026-06-25). Targets Google ADK +
  Vertex AI Agent Runtime, both of which are active and expanding as of mid-2026. The
  GEPA prompt optimizer (6 optimizer backends including GEPA, ProTeGi, PromptWizard)
  is actively documented in Google Cloud 2026 materials. The a2a SDK integration
  (Agent-to-Agent protocol) is a 2026 addition. This is a current, production-grade CLI.

Tests present:
  No unit tests for the CLI itself in the repo. Only scaffold template tests exist:
  tests/integration/test_agent.py (stamped into user projects by scaffold). The CLI
  relies on manual integration testing via the skills workflow.

License: Apache-2.0

ELAI steal assessment (sharp question: what does agents-cli have that ELAI does not):

1. GEPA / multi-backend prompt optimizer (eval optimize)
   What: CLI wrapper around ADK's "adk optimize" which runs GEPA/ProTeGi/BayesianSearch/
   RandomSearch/MetaPrompt/PromptWizard optimization loops against an EvaluationDataset.
   Translates EvaluationDataset format to ADK EvalSet, writes temp configs, invokes uv run adk optimize.
   Where: src/google/agents/cli/eval/cmd_optimize.py + optimize_utils.py
   ELAI coverage: ELAI ships H-39 abstention-aware promotion and B-41 per-level eval, but has NO
   automated prompt optimization loop of any kind. The GEPA wrapper is 150 LOC of pure translator
   logic (EvaluationDataset->EvalSet format conversion) + subprocess dispatch; the real optimizer
   is in ADK itself, not in agents-cli. Not directly portable (ADK-specific).

2. Eval generate: per-case fresh agent reload pattern
   What: _inference_runner.py explicitly calls AgentLoader.remove_agent_from_cache before
   each eval case to avoid asyncio.Lock cross-event-loop contamination. This is a non-obvious
   correctness requirement when running multiple eval cases in sequence.
   Where: src/google/agents/cli/eval/_inference_runner.py lines 76-87
   ELAI coverage: ELAI's eval harness (B-41, E-6) does not have an explicit cache-flush mechanism
   documented. If ELAI ever runs ADK-style sequential inference the pattern is worth copying.
   The underlying bug (aiplatform#6865 tools attribute) is ADK-specific but the cache-flush
   discipline is general.

3. A2A transport dispatch in run command
   What: cmd_run dispatches via two protocols: ADK SSE (local/remote ADK) and A2A (Agent-to-Agent
   protocol) using google/a2a Python SDK. The A2A path fetches an AgentCard from
   /.well-known/agent.json, constructs ClientFactory, handles file/text parts, and streams
   TaskArtifactUpdateEvents. Token selection is automatic: access_token for Agent Runtime URLs
   (detected by "aiplatform.googleapis.com"), id_token for Cloud Run/GKE.
   Where: src/google/agents/cli/run/cmd_run.py
   ELAI coverage: ELAI ships an ACP transport + MCP bridge + vendor CLI subprocess bridges.
   ELAI does NOT have A2A protocol support. A2A is a Google-specific inter-agent standard; the
   AgentCard discovery + TransportProtocol dispatch pattern differs from ELAI's ACP model.
   Steal value: LOW for ELAI unless targeting Google ecosystem; A2A and ACP are competing standards.

4. Skills-to-coding-agent installation mechanism
   What: agents-cli setup runs npx skills@1.4.8 (pinned version) to install 7 SKILL.md files
   into ~/.agents/skills/ where Claude Code, Gemini CLI, Codex, etc. auto-discover them.
   The version drift check (12h rate-limited, SKILL.md frontmatter version comparison vs
   running CLI __version__) keeps skills in sync with the installed CLI.
   Where: src/google/agents/cli/_skills_check.py + setup/cmd_setup.py
   ELAI coverage: ELAI has a richer skills system (11 ELAI-authored skills, skill-import-policy.md,
   skillspector gate). The agents-cli pattern of CLI-distributing skills via npx + auto-version-check
   is simpler but is a different distribution model (npm-based vs git-based). Not novel for ELAI.

5. Deployment target abstraction (agent_runtime / cloud_run / gke)
   What: Single deploy command dispatches to three targets from one config. Per-target flag
   validation is explicit (e.g., --num-workers only for agent_runtime, --cpu/--memory forbidden
   for GKE which uses Terraform HPA). --no-wait + --status for async deploys.
   Where: src/google/agents/cli/deploy/cmd_deploy.py
   ELAI coverage: ELAI ships B-37 multi-runtime backend abstraction for inference (11 backends,
   RuntimeAdapter). Deployment targets are a different concern (cloud infra not inference runtime);
   ELAI has no equivalent multi-target deploy command. This is GCP-ecosystem-specific and not
   directly stealable for ELAI's current scope.

6. Experiment flag registry
   What: Central _REGISTRY dict of Experiment(label, value_type, default_value); resolved via
   AGENTS_CLI_EXPERIMENTS env var JSON override. Clean, minimal feature-flag system with typed
   defaults and env override.
   Where: src/google/agents/cli/_experiments.py
   ELAI coverage: ELAI uses ELAI_*_ENABLE env vars per-feature; no central typed registry.
   Steal value: LOW (trivial to replicate; ELAI's pattern is already established and works).
