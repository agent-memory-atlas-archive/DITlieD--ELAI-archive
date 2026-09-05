Goose Repository Partition Map: Recipes, Skills, Permissions, Security, and Runtime

(Scope: recipe/, skills/, slash_commands/, permission/, security/, hints/, goose_apps/, dictation/, bin/)

=== SECTION 1: RECIPES (10 files)

DEFINITION AND SCHEMA

A Recipe is a YAML/JSON file (via serde) that declaratively defines an agent run. Each recipe has:
  - Required: version (sem ver, default "1.0.0"), title (string), description (string)
  - At least one of: instructions (model instructions) OR prompt (session starter)
  - Optional: extensions (array of ExtensionConfig), settings (goose_provider/model/temperature/max_turns), 
    parameters (RecipeParameter array with key/input_type/requirement/description/default/options),
    sub_recipes (array of SubRecipe with name/path/values/sequential_when_repeated),
    response (JSON schema), author (contact/metadata), activities (pill array), retry (RetryConfig)

File format: YAML (preferred, reformatted for multiline fields like "prompt" and "instructions") or JSON.
Extensions to RECIPE_FILE_EXTENSIONS: .yaml, .json.
Location discovery: local_recipe_dirs() walks CWD, GOOSE_RECIPE_PATH env var (: or ; separated),
  ~/.goose/recipes, .goose/recipes, ~/.agents/recipes, .agents/recipes. Duplicates canonicalized and deduped.

PARSING AND VALIDATION

Recipe::from_content(content) - main entry point (recipe/mod.rs:323):
  1. serde_yaml::from_str on raw content
  2. Handles nested {"recipe": {...}} wrapper (for structured embedding)
  3. Strips location info from parse errors (strip_error_location removes "at line X" prefix)
  4. Auto-injects "analyze" extension if legacy "developer" exists (ensure_analyze_for_developer)
  5. Auto-injects "summon" if sub_recipes present (ensure_summon_for_subrecipes)

Recipe::from_file_path(path) - wraps read_recipe_file() then from_content().

Validation: validate_recipe_template_from_content() (recipe/validate_recipe.rs:39):
  - parse_and_validate_parameters: template expansion check + parameter inference
  - validate_prompt_or_instructions: at least one must be non-empty
  - validate_json_schema: if response.json_schema present, run jsonschema::validator_for() (rejects invalid schemas)
  - validate_retry_config: if retry present, call retry.validate()

RecipeParameter.input_type enum: String, Number, Boolean, Date, File (no defaults to prevent sensitive imports), Select.
RecipeParameter.requirement enum: Required, Optional, UserPrompt (filled from user input before run).

TEMPLATE AND PARAMETERIZATION

Template system: minijinja (template/recipe.rs). Recipes support {{variable}} syntax.
  - parse_recipe_content() substitutes parameters before loading Recipe struct
  - BUILT_IN_RECIPE_DIR_PARAM ("recipe_dir"): auto-populated with recipe file's parent directory
  - Complex template variables (non-alphanumeric) are pre-processed separately
  - Minijinja with UndefinedBehavior::Lenient (missing params don't error, substitute raw text)

SCANNING AND DISCOVERY

scan_local_recipes() pattern: walk all directories from local_recipe_dirs(), find *.yaml and *.json files,
  parse each, return Vec<Recipe>. (Not exposed directly; wrapped via load_local_recipe_file which uses same dirs.)

SUB-RECIPES

SubRecipe: name (string), path (string to another recipe file), values (Map<String, String> of parameters to pass),
  sequential_when_repeated (bool: if true, multiple invocations run sequentially not parallel),
  description (optional).

Summon extension auto-added when sub_recipes present (implies sub-recipe dispatch capability).

SECURITY AND RUNTIME CHECKS

Recipe::check_for_security_warnings() - scans instructions/prompt/activities for unicode tags
  (contains_unicode_tags() detects HTML/entity injection payloads). Returns bool.

BUILDER PATTERN

RecipeBuilder: fluent API to construct Recipe instances programmatically.
  - build() validates title and description are set, and at least one of prompt/instructions exists
  - Used in tests; not a primary entry point for normal operation.

PORTABLE IDEA #1: RECIPE FRAMEWORK
  - Declarative YAML/JSON schema with parameter substitution (minijinja-based)
  - Auto-discovery from multiple hierarchical dirs (project + global + env-var)
  - Security pre-check on sensitive fields (unicode tags)
  - Builder pattern for programmatic construction
  - [File evidence: recipe/mod.rs:318-341 (from_file_path/from_content), 
    recipe/template_recipe.rs:1-60 (minijinja integration), 
    recipe/local_recipes.rs:13-46 (discovery dirs)]

=== SECTION 2: SKILLS (4 files)

DEFINITION AND DISCOVERY

A Skill is a MarkDown file (SKILL.md) with frontmatter YAML. Each skill:
  - Frontmatter: name (required, no /), description, metadata (free-form JSON mapping, per agentskills.io spec)
  - Body: markdown/prose content (the skill implementation)
  - Supporting files: other files in the skill directory (loaded on-demand)

Discovery: discover_skills(working_dir) walks all_skill_dirs() looking for SKILL.md files.
  all_skill_dirs() precedence (project first, then global, then plugins):
    .agents/skills, .goose/skills, .claude/skills (project-scoped)
    ~/.agents/skills, ~/.goose/skills, ~/.claude/skills, ~/.config/agents/skills (global)
    installed_plugin_skill_dirs() (from plugin system)

FRONTMATTER AND PROPERTIES

SkillFrontmatter: name (optional but validated at load), description (default empty),
  metadata (HashMap<String, serde_json::Value>, arbitrary metadata per agentskills.io).

When loaded, metadata is flattened into SourceEntry.properties. Two special keys:
  - "arguments": array of argument names (fed to apply_skill_arguments)
  - "argument-hint": string describing the expected arguments

Validation: skill_name must not be empty, <=64 chars, lowercase/digit/hyphen only, no leading/trailing hyphen.
  (validate_skill_name, mod.rs:74-100). Names with / are rejected at parse time.

LOADING AND SUBSTITUTION

loaded_skill_context_with_args(skill, args_str) - replaces $ARGUMENTS placeholder in skill body with the args_str.
  If args is Some, apply_skill_arguments() parses args and substitutes (skill/arguments.rs).
  If args is None, returns raw skill content.

skill_argument_names(skill) - extracts argument list from metadata["arguments"].
skill_argument_hint(skill) - extracts metadata["argument-hint"] (optional user-facing hint).

DISTINCTION: RECIPES VS SKILLS VS EXTENSIONS

- Recipe: Declarative YAML/JSON with parameters, prompt/instructions, extensions list, response schema.
  Entry point for an agent session. (recipe/mod.rs)
- Skill: Markdown with frontmatter (name/description/metadata). Content is prose/instructions.
  Callable from within a recipe via summon extension or via /skill command.
  (skills/mod.rs)
- Extension: A tool/capability that runs during agent execution (part of agents/extension module, not in scope here).
  Can be Builtin (analyze, developer, summon), Platform (external MCP), Stdio (subprocess), or InlinePython.
  Recipe declares extensions it needs; extensions are invoked during run.

Relationship: Recipe -> includes Extensions + references Skills. Skills and Extensions are orthogonal.

SKILL DIRECTORY STRUCTURE

Global discoverable root: ~/.agents/skills
Project discoverable root: <project>/.agents/skills or .goose/skills or .claude/skills
Each skill is a directory: <root>/<skill-name>/ containing SKILL.md + supporting files.

Canonical name: inferred_discoverable_skill_root(path) walks ancestors to confirm path is under a known root.
  If under ~/.agents/skills or similar, it's discoverable.

PORTABLE IDEA #2: SKILL FRONTMATTER + MARKDOWN BODY
  - SkillFrontmatter with name/description/metadata (agentskills.io-compliant)
  - Argument substitution via $ARGUMENTS placeholder
  - Multi-tier discovery (project -> global -> plugins)
  - No-op caching layer (supporting files discovered on load)
  - [File evidence: skills/mod.rs:319-356 (parse_skill_content, discovery loop),
    skills/mod.rs:129-162 (apply_skill_arguments, skill_argument_names),
    skills/client.rs (runtime MCP dispatch, not fully read but referenced at 9)]

=== SECTION 3: SLASH COMMANDS (6 files)

SURFACE AND DISPATCH

Slash commands are actions prefixed with / in user input (e.g., /skills, /compact, /doctor).
Three sources merge in precedence: Builtin > Recipe > Skill.

list_builtin_commands() - returns 8 hardcoded SlashCommandEntry items (mod.rs:7):
  prompts, prompt, compact, clear, skills, doctor, goal, grind (from agents/execute_commands.rs).

list_acp_commands(working_dir) merges builtins + recipe commands + skill commands:
  - recipe_slash_command::commands_from_mappings() generates commands from recipe library
  - skill_slash_command::list_commands(working_dir) walks skill dirs to list /skill <name> entries

merge_command_sources() resolves collisions by taking Builtin (never overridden), then Recipes (override Skills),
  then Skills (if name not reserved). (slash_command.rs:29-52, test at line 98-101)

SlashCommandEntry: name (string), description, source (enum: Builtin|Recipe|Skill), input_hint (optional).

INPUT PARSING AND NORMALIZATION

normalize_command_name(name) - converts to lowercase, normalizes whitespace, used for collision detection.

Recipe slash commands: derived from Recipe.title (treated as /recipe_name). Comes from recipe_slash_command.rs.
  Recipes that parse successfully become /recipe_name commands with recipe description.

PORTABLE IDEA #3: THREE-TIER COMMAND DISPATCH
  - Builtin commands: static hardcoded enum (fast path)
  - Recipe commands: derived from loaded recipes in discovery dirs
  - Skill commands: load_skill(name) entries mapped to /skill name
  - Precedence: Builtin > Recipe > Skill (prevents user mistakes, reserved names protected)
  - Collision detection via normalize_command_name (case-insensitive)
  - [File evidence: slash_commands/slash_command.rs:7-52 (merge logic, tests),
    slash_commands/types.rs (SlashCommandEntry struct and enum),
    slash_commands/util.rs (normalize_command_name)]

=== SECTION 4: PERMISSION (5 files)

PERMISSION MODEL

Permission = explicit per-tool allow/deny decision, stored with context hash (to differentiate identical-name calls with different args).

ToolPermissionStore: HashMap<tool_name, Vec<ToolPermissionRecord>>.
  ToolPermissionRecord: tool_name, allowed (bool), context_hash (blake3 of args), readable_context (optional human-readable summary),
    timestamp (unix sec), expiry (optional unix sec, for time-limited grants).

Storage: ~/.goose/permissions/tool_permissions.json (TOML backup at ~/.goose/permissions/tool_permissions.toml).
  Atomic write: write to .tmp, rename to target (transaction-safe).

APPROVAL FLOW

Permission modes (inferred from source code structure, not explicit enum):
  1. AUTO (default): pre-approved tools (read-only detected by PermissionJudge)
  2. APPROVE: user must approve each tool call (permission_judge.rs creates "Check tool operation" LLM task)
  3. CHAT: user is asked in conversation (permission_confirmation.rs, deferred)

PermissionJudge: uses an LLM to classify tool calls as read-only or write.
  create_read_only_tool() defines the classifier tool (permission_judge.rs:19-65).
  extract_read_only_tools(response) parses LLM output to get safe tools.

Permission confirmation flow (permission_confirmation.rs):
  - User is prompted: "Tool X wants to run, approve?" with options [Allow, Allow Once, Deny, Always Deny]
  - Confirmation stored as ToolPermissionRecord with optional expiry
  - On next identical call, check_permission() returns cached decision (no re-prompt)

TOOL CONTEXT AND HASHING

context_hash: blake3(tool_call.name + serde_json::to_string(tool_call.inputs)).
  Different args = different hash = may trigger re-confirmation.
  readable_context: human-friendly summary (e.g., "rm -rf /tmp/foo", not a hash).

Expiry: Optional i64 unix timestamp. cleanup_expired() runs on load() and after every permission save.
  Expired entries are silently removed.

PERMISSION INSPECTOR

PermissionInspector: queries tool permissions by name or lists all.
  get_permission(tool_name, context_hash) -> Option<bool> (Some(true) = allowed, Some(false) = denied, None = never seen).

PORTABLE IDEA #4: CONTEXT-AWARE PERMISSION PERSISTENCE
  - Per-tool + per-context-hash (not just tool name)
  - LLM-based read-only detection (PermissionJudge)
  - Time-limited grants (expiry)
  - Atomic file writes (transaction-safe)
  - Blake3 context hashing (resilient vs minor arg changes)
  - [File evidence: permission/permission_store.rs:11-80 (storage schema, atomic write),
    permission/permission_judge.rs:13-100 (read-only classifier),
    permission/permission_confirmation.rs (user flow, referenced but not fully read)]

=== SECTION 5: SECURITY (7 files)

THREAT PATTERNS AND CLASSIFICATION

SecurityManager: coordinates all security checks. Holds OnceLock<PromptInjectionScanner>.

ThreatPattern: name (identifier), pattern (regex), description, risk_level (Low|Medium|High|Critical),
  category (FileSystemDestruction|RemoteCodeExecution|DataExfiltration|SystemModification|NetworkAccess|
    ProcessManipulation|PrivilegeEscalation|CommandInjection).

THREAT_PATTERNS const array (security/patterns.rs:48+): ~40-60 regexes covering:
  - rm -rf /, /home, /root (root filesystem destruction)
  - dd if=/dev/zero of=/dev/sda (disk wiping)
  - format, mkfs commands (drive formatting)
  - curl | bash, wget | bash (remote code execution)
  - eval, exec, $(command substitution) (code execution)
  - ssh -i, rsync, scp (privilege/credential theft)
  - chmod 777, sudo, su (privilege escalation)
  - nc, socat, ncat, xxd (network exfiltration)

Risk score: RiskLevel::confidence_score() returns 0.95 (Critical), 0.75 (High), 0.60 (Medium), 0.45 (Low).

SCANNING PIPELINE

PromptInjectionScanner: two modes:
  1. Pattern-only (new()): regex matching against THREAT_PATTERNS
  2. ML-enhanced (with_ml_detection()): additionally loads a classifier model

analyze_tool_call_with_context(tool_call, messages) - scans tool_call.name + serde_json serialized inputs.
  Returns AnalysisResult: is_malicious (bool), confidence (f32), explanation (string), scanned (bool).

Two config params:
  - SECURITY_PROMPT_ENABLED (default false): enable scanning at all
  - SECURITY_COMMAND_CLASSIFIER_ENABLED (default false): enable ML classifier (fallback if init fails)
  - SECURITY_PROMPT_CLASSIFIER_ENABLED: enable prompt-side ML classification

Override env vars (uppercase _OVERRIDE suffix) bypass config files.

THREAT CLASSIFICATION

ClassificationClient (classification_client.rs): wraps external ML classifier (deferred, not fully read).
  Used if SECURITY_COMMAND_CLASSIFIER_ENABLED=true.

EgressInspector (egress_inspector.rs): checks if tool call attempts to exfiltrate data outward.
  (Not fully read; inferred from name and module structure.)

SecurityResult: is_malicious, confidence, explanation, should_ask_user (always true if above threshold),
  finding_id (UUID), tool_request_id.

THRESHOLD AND DECISION

Threshold: Config param "SECURITY_THRESHOLD" (default ~0.70 inferred).
  above_threshold = confidence > config_threshold. If true, user is asked. If false, logged but not blocking.

filter_malicious_tool_calls(messages, permission_check_result, system_prompt) - post-permission security gate.
  Filters across approved + needs_approval tool requests.

EXECUTION HOOKS

analyze_tool_requests() is called during conversation processing (likely at message->tool-request boundary).
  Emits tracing spans: prompt_injection_finding (malicious), prompt_injection_tool_call_passed (clean),
    prompt_injection_analysis_performed (count).

PORTABLE IDEA #5: PATTERN + ML HYBRID THREAT DETECTION
  - ~40+ regex patterns for common shell/exfiltration attacks
  - RiskLevel enum (Low/Medium/High/Critical) with confidence_score()
  - ML fallback (optional, graceful degrade to pattern-only)
  - Per-threshold decision (confidence > SECURITY_THRESHOLD = ask user)
  - Structured SecurityResult with finding_id + explanation
  - Async scanning integrated into tool-request pipeline
  - [File evidence: security/patterns.rs:48-80 (ThreatPattern, THREAT_PATTERNS array),
    security/scanner.rs (PromptInjectionScanner, not fully read but inferred from SecurityManager),
    security/mod.rs:77-140 (analyze_tool_requests, threshold logic, ML init)]

=== SECTION 6: HINTS (3 files)

DESCRIPTION: Load contextual .goosehints and AGENTS.md files from project directories.

SubdirectoryHintTracker: records which directories have been processed.
  record_tool_arguments(args, working_dir) - extracts path and command tokens from tool args,
    resolves to parent directories, queues for hint loading.
  load_new_hints(working_dir) - scans pending directories for .goosehints and AGENTS.md,
    returns (filename, content) pairs.

Context file names: Config param CONTEXT_FILE_NAMES, defaults to [".goosehints", "AGENTS.md"].

Hints are implicitly included in the agent's context window (mechanism deferred, inferred from hints-as-context pattern).
  [File evidence: hints/mod.rs (module exports), hints/load_hints.rs:1-80 (SubdirectoryHintTracker,
    get_context_filenames, load_new_hints)]

=== SECTION 7: GOOSE_APPS (4 files)

DESCRIPTION: MCP app registry and metadata extraction.

GooseApp: resource (McpAppResource), mcp_servers (list of server names), window_props (optional width/height/resizable),
  prd (product requirements document, optional text).

from_html(html) parses JSON-LD metadata embedded in <script type="application/ld+json"> + <script type="application/x-goose-prd">.
  Extracts name, description, window dimensions, mcp_servers list, prd text.

McpAppResource: name, description, uri, permissions/CSP/UI metadata (defined in resource.rs, not fully read).

McpAppCache: caches fetched apps (cache.rs, not fully read).

Usage: discover MCP apps from HTML pages (likely served by remote repos or local registries).
  [File evidence: goose_apps/app.rs:37-100 (from_html parser), goose_apps/mod.rs (module exports)]

=== SECTION 8: DICTATION (3 files)

DESCRIPTION: Audio-to-text transcription.

Providers: enum-based provider selection (openai, local_whisper, others) - details deferred.
Whisper: local inference via whisper.rs (conditional on feature="local-inference").
  (Not fully read; integration details deferred.)

=== SECTION 9: BIN (3 files)

DESCRIPTION: Standalone CLI tools.

analyze_cli.rs: ad-hoc code analysis via tree-sitter.
  --focus <symbol>: call graph for a symbol (shows callers/callees)
  --depth <n>: directory recursion limit (default 3)
  --follow <n>: call graph traversal depth (default 2)
  Outputs formatted analysis or symbol call graph.

build_canonical_models.rs: model cache construction (deferred).
generate_acp_schema.rs: ACP (Agent Capability Protocol?) schema generation (deferred).
  [File evidence: bin/analyze_cli.rs:1-50]

=== SECTION 10: CROSS-CUTTING DESIGN PATTERNS

1. DISCOVERY AND REGISTRATION
   - Recipes: walk dirs, parse YAML/JSON, return Recipe struct
   - Skills: walk dirs, parse SKILL.md frontmatter, return SourceEntry
   - Commands: merge Builtin + Recipe + Skill with precedence
   - All three support project + global hierarchies

2. FRONTMATTER / METADATA
   - Recipe: YAML/JSON with version/title/description
   - Skill: YAML frontmatter with name/description/metadata object
   - Both support optional parameters/extensions/configuration

3. PARAMETERIZATION
   - Recipe: minijinja template substitution before load
   - Skill: $ARGUMENTS placeholder replacement + apply_skill_arguments()
   - Both allow per-invocation context injection

4. PERSISTENCE AND TRANSACTIONS
   - Permissions: atomic JSON writes (write-to-tmp, rename)
   - Recipes/Skills: immutable discovery (no write-back)

5. SECURITY
   - Recipe: pre-scan for unicode tags (injection detection)
   - Security: pattern + ML hybrid threat scanning of tool calls
   - Permission: LLM-based read-only classification

=== PORTABLE IDEAS FOR REIMPLEMENTATION IN OTHER RUST AGENTS

1. **Recipe framework**: Declarative YAML/JSON with minijinja substitution + multi-tier discovery.
   [evidence: recipe/mod.rs:42-87 (Recipe struct), recipe/template_recipe.rs:15-60 (minijinja),
    recipe/local_recipes.rs:21-46 (discovery dirs)]

2. **Skill frontmatter + markdown body**: Agentskills.io-compliant YAML frontmatter + argument substitution.
   [evidence: skills/mod.rs:24-36 (SkillFrontmatter), skills/mod.rs:319-356 (parse_skill_content)]

3. **Three-tier command merge**: Builtin > Recipe > Skill with collision detection via normalize_command_name.
   [evidence: slash_commands/slash_command.rs:29-52 (merge_command_sources), tests 85-101]

4. **Context-aware permission persistence**: Per-tool + per-context-hash with expiry + LLM read-only detection.
   [evidence: permission/permission_store.rs:11-80 (ToolPermissionRecord, blake3 hashing),
    permission/permission_judge.rs:19-100 (read-only classifier)]

5. **Pattern + ML hybrid threat detection**: ~40+ regex patterns (Critical/High/Medium/Low) + optional ML fallback.
   [evidence: security/patterns.rs:48-100 (ThreatPattern array, RiskLevel enum),
    security/mod.rs:77-140 (PromptInjectionScanner init, threshold logic)]

6. **Atomic transactional writes**: Write to .tmp, atomic rename (prevents partial-write corruption).
   [evidence: permission/permission_store.rs:63-77 (save method)]

7. **Lazy static regex compilation**: Use std::sync::LazyLock for THREAT_PATTERNS regex compilation (not shown but standard).
   [evidence: security/patterns.rs:1-3 (lazy imports implied)]

8. **Builder pattern for complex structs**: RecipeBuilder for Recipe construction.
   [evidence: recipe/mod.rs:209-434 (RecipeBuilder)]

=== ARCHITECTURE SUMMARY

Goose is structured as a **recipe-driven multi-capability agent** with strong defaults:

- **Core execution model**: Recipe (declarative session template) → Extensions (tools/capabilities) → Conversational loop
- **Customization layers**: Skills (markdown fragments), Commands (slash-prefixed actions), Permissions (per-tool gates)
- **Safety gates**: Security (pattern + ML threat detection), Permission (LLM read-only detection + user approval), Recipe (unicode tag scan)
- **Discovery**: Multi-tier (project + global + env-var + plugins) filesystem walk with deduplication
- **Parameterization**: Template substitution (Recipe: minijinja, Skill: $ARGUMENTS placeholder)

Key insight: Recipes are **values** (data), Skills are **code fragments** (markdown), Extensions are **capabilities** (tools).
The recipe *declares* which extensions it needs and which parameters to gather; skills are optional educational content.
Permissions and Security run orthogonally to prevent misuse, operating at the tool-call boundary.
