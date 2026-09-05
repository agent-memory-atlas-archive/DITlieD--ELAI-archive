Source: NVIDIA SkillSpector
Reliability: HIGH (official NVIDIA open-source repo, reviewed Python source, Apache-2.0, actively maintained)
Version: 2.1.2
Repo: https://github.com/NVIDIA/skillspector
Files examined:
  src/skillspector/graph.py
  src/skillspector/state.py
  src/skillspector/models.py
  src/skillspector/nodes/meta_analyzer.py
  src/skillspector/nodes/analyzers/__init__.py
  src/skillspector/nodes/report.py
  src/skillspector/nodes/resolve_input.py
  src/skillspector/nodes/build_context.py
  src/skillspector/input_handler.py
  src/skillspector/llm_analyzer_base.py
  src/skillspector/nodes/analyzers/behavioral_taint_tracking.py
  src/skillspector/nodes/analyzers/mcp_rug_pull.py
  src/skillspector/nodes/analyzers/semantic_security_discovery.py
  pyproject.toml
  README.md


Public API:

graph.invoke | (state: dict) -> dict | Synchronous entry point; runs full LangGraph pipeline
  Required input keys: input_path (str) OR skill_path (str)
  Optional input keys: output_format ("terminal"|"json"|"markdown"|"sarif"), use_llm (bool, default True), yara_rules_dir (str|None), zip_bytes (bytes|None)
  Returns: SkillspectorState dict with keys: risk_score (int 0-100), risk_severity (str), risk_recommendation (str), filtered_findings (list[Finding]), report_body (str), sarif_report (dict), component_metadata (list[dict])

InputHandler.resolve | (input_path: str) -> tuple[Path, str] | Normalizes any input to local directory; source_type one of "git", "url", "zip", "file", "directory"
  Side effect: may create a temp directory (query handler.temp_dir_for_cleanup() to get path for deletion)

Finding.to_dict | () -> dict | Serializes a finding to JSON-serializable shape with keys: id, category, pattern, severity, confidence, location, finding, explanation, remediation, code_snippet, intent

CLI entrypoint: skillspector scan <path|url> [--format terminal|json|markdown|sarif] [--output PATH] [--no-llm] [-V]


Behavioral contracts:

graph.invoke -> resolves input, builds file cache + manifest + AST cache, runs 20 analyzer nodes in parallel (all fan out from build_context), collects findings via operator.add reducer, runs meta_analyzer (LLM filter/enrich), runs report node to compute score + format
meta_analyzer -> when use_llm=True and LLM creds available: one async LLM call per file-with-findings (chunked when file exceeds token budget); LLM verdict filters static findings, only keeps those with is_vulnerability=True AND confidence>=0.6
meta_analyzer -> when use_llm=False: pass-through, all static findings kept with default remediations
report._compute_risk_score -> CRITICAL+50, HIGH+25, MEDIUM+10, LOW+5; 1.3x multiplier if has_executable_scripts; clamped to [0,100]
mcp_rug_pull -> STUB (returns no findings); implements RP1-RP3 patterns but body is empty; marked TODO(SADD B.3.3)
InputHandler -> uses subprocess git clone for Git URLs; uses httpx for file URL downloads; uses zipfile for zips; single .md files are wrapped in a temp directory
OSV.dev lookup (SC4) -> batch HTTPS POST to api.osv.dev; in-memory 1-hour cache; falls back to static allowlist when offline; no API key required
LLM call -> uses LangChain structured output (Pydantic schema validation); anti-jailbreak system prompt embedded; prompt includes CRITICAL INSTRUCTIONS block that flags "ignore security warnings" text as a RED FLAG
Semantic analyzers (semantic_security_discovery, semantic_developer_intent, semantic_quality_policy) -> LLM-only, return empty findings when use_llm=False
mcp_tool_poisoning -> hybrid: TP1-TP3 static, TP4 (description-behavior mismatch) LLM-powered; TP4 skipped when use_llm=False


Platform constraints:

Python 3.12+ required (uses StrEnum, TypedDict total=False, match, modern typing)
yara-python>=4.5.0 requires native YARA C library; install may fail on systems without libssl/libyara (Windows: use prebuilt wheel)
LangChain structured output uses langchain-openai; Anthropic provider uses openai>=2.25.0 with base_url override (not langchain-anthropic)
OSV.dev SC4 check requires outbound HTTPS to api.osv.dev; silent fallback to static list when blocked
git must be installed and on PATH for Git URL input resolution
asyncio.run() called inside meta_analyzer node (not async-native); cannot be called from within an already-running event loop (LangGraph runs graph.invoke synchronously)
Temp directory cleanup is the CALLER's responsibility: state["temp_dir_for_cleanup"] must be deleted by the invoking code


Gotchas:

mcp_rug_pull is a stub; emits zero findings regardless of input; comparing previous vs current manifest (RP1-RP3) not implemented
LLM filter uses confidence threshold 0.6 (hardcoded in meta_analyzer.apply_filter); findings below 0.6 are silently dropped even if is_vulnerability=True
Coarse vs granular key fallback: LLM responses without start_line use (file, rule_id) key; if two findings share the same rule_id in one file and LLM omits line numbers, BOTH or NEITHER are kept (ambiguous)
Risk score can exceed 100 before clamping; a single CRITICAL finding + executable scripts multiplier is clamped to 100 so scores saturate quickly; a skill with 2+ CRITICAL findings looks identical to one with 10
Executable scripts detection uses file extension only (.py, .sh, .js, etc.); a malicious .md file is not flagged as executable regardless of content
YARA rules are bundled (4 rule files: malware.yar, webshells.yar, cryptominers.yar, hacktools.yar); custom rules can be added via --yara-rules-dir but only appended, not replacing bundled rules
Non-English content, image-based attacks, encrypted/binary code, and dynamic runtime behavior are explicitly out of scope (stated in README Limitations)
Static analysis has moderate precision (some false positives per README); LLM stage improves to ~87%
Semantic discovery LLM analyzers (semantic_*) run independently from meta_analyzer; they produce findings directly (not filtered by static first), then those findings also pass through meta_analyzer for LLM confirmation
typer version pinned <0.24 due to click conflict with semgrep; this is a known constraint in pyproject.toml


Critical assertions (testable):

assert result["risk_score"] >= 0 and result["risk_score"] <= 100
assert result["risk_severity"] in ("LOW", "MEDIUM", "HIGH", "CRITICAL")
assert result["risk_recommendation"] in ("SAFE", "CAUTION", "DO_NOT_INSTALL")
assert all(f["severity"] in ("LOW", "MEDIUM", "HIGH", "CRITICAL") for f in result.get("issues", []))
assert all(0.0 <= f["confidence"] <= 1.0 for f in result.get("issues", []))
assert (result["risk_score"] >= 81) == (result["risk_severity"] == "CRITICAL")
assert (21 <= result["risk_score"] <= 50) == (result["risk_severity"] == "MEDIUM")
# LLM filter confidence gate
assert all(f.confidence >= 0.6 for f in filtered_findings)  # when use_llm=True
