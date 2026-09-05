Source: vercel-labs/skills (npx skills CLI)
Reliability: MEDIUM
Files examined:
  F:\.ELAI_workflow\docs\.source\vercel-skills\package.json
  F:\.ELAI_workflow\docs\.source\vercel-skills\src\cli.ts
  F:\.ELAI_workflow\docs\.source\vercel-skills\src\agents.ts
  F:\.ELAI_workflow\docs\.source\vercel-skills\src\add.ts (lines 1-1597)
  F:\.ELAI_workflow\docs\.source\vercel-skills\src\installer.ts (lines 1-80)

Package:
  name: skills
  version: 1.5.12
  license: MIT
  bin: { "skills": "./bin/cli.mjs", "add-skill": "./bin/cli.mjs" }
  runtime: Node >=18 (note: project uses pnpm@10.17.1 and vitest -- NOT bun; bun-not-node rule means we'd use bunx skills)
  dependencies: yaml ^2.8.3
  devDependencies (load-bearing): @clack/prompts ^0.11.0, picocolors ^1.1.1, simple-git ^3.27.0, xdg-basedir ^5.1.0, obuild ^0.4.22

Public API (CLI command surface):
  skills add <source> [opts]   | installs skill(s) from github owner/repo, URL, local path, or well-known endpoint
  skills use <pkg>@<skill>     | generates a one-shot prompt without installing (stdout pipe to agent)
  skills remove [skills...]    | removes installed skills (interactive or named)
  skills list / ls             | lists installed skills
  skills find [query]          | searches skills.sh registry interactively
  skills update [skills...]    | updates installed skills to latest versions
  skills init [name]           | scaffolds a new SKILL.md in cwd/<name>/ or cwd/
  skills experimental_install  | restores from skills-lock.json
  skills experimental_sync     | syncs skills from node_modules into agent dirs

Key flags for add:
  -g / --global           install to home dir instead of project dir
  -a / --agent <agents>   target specific agents (or '*' for all)
  -s / --skill <skills>   select specific skills from a package (or '*' for all)
  -y / --yes              non-interactive (skip all prompts)
  --copy                  copy files instead of symlinking (default: symlink)
  --all                   shorthand for --skill '*' --agent '*' -y
  --full-depth            search all subdirectories even when root SKILL.md exists
  --subagent <names>      Eve-only: install to named subagent dirs
  --dangerously-accept-openclaw-risks  bypass openclaw block

SKILL manifest format (from runInit, src/cli.ts:247-265):
  A SKILL.md file with YAML frontmatter:
    ---
    name: <skill-name>
    description: <one-line description loaded into every system prompt>
    ---
    [body: full instructions read only when the skill is activated]

  Frontmatter fields recognized: name (required), description (required).
  Body is the full instructions; only description is in the system prompt (progressive disclosure).
  Skills discovered by discoverSkills(): searches for SKILL.md files recursively.
  Internal skills (not shown by default) require --skill <name> or --full-depth to expose.

Multi-agent harness support (src/agents.ts):
  54 named AgentType entries. Canonical directory per agent:
    claude-code:    skillsDir=.claude/skills          globalSkillsDir=~/.claude/skills
    codex:          skillsDir=.agents/skills           globalSkillsDir=~/.codex/skills
    opencode:       skillsDir=.agents/skills           globalSkillsDir=~/.config/opencode/skills
    cursor:         skillsDir=.agents/skills           globalSkillsDir=~/.cursor/skills
    windsurf:       skillsDir=.windsurf/skills         globalSkillsDir=~/.codeium/windsurf/skills
    roo:            skillsDir=.roo/skills              globalSkillsDir=~/.roo/skills
    goose:          skillsDir=.goose/skills            globalSkillsDir=~/.config/goose/skills
    gemini-cli:     skillsDir=.agents/skills           globalSkillsDir=~/.gemini/skills
    github-copilot: skillsDir=.agents/skills           globalSkillsDir=~/.copilot/skills
    eve:            skillsDir=agent/skills             globalSkillsDir=undefined (project-only)
    universal:      skillsDir=.agents/skills           globalSkillsDir=~/.config/agents/skills

  Universal agents (.agents/skills): amp, antigravity, antigravity-cli, cline, codex, cursor, deepagents,
    dexto, firebender, gemini-cli, github-copilot, kimi-code-cli, loaf, opencode, promptscript,
    replit, universal, warp, zed. These share ONE directory; symlink from canonical to per-agent is skipped.
  Non-universal agents get symlinks FROM .agents/skills/<skill>/ TO their own skillsDir/<skill>/.
  detectInstalled(): probes filesystem for each agent's config dir (e.g. ~/.claude exists -> claude-code installed).
  Agent auto-detection: when running inside an AI agent (CLAUDE env var or similar), -y is auto-set
    and the running agent is auto-selected. Detected via isRunningInAgent().

Source resolution (src/source-parser.ts, referenced):
  Accepts: github:owner/repo, owner/repo shorthand, full HTTPS URL, SSH URL, local path, well-known URL
  well-known: fetches /.well-known/agent-skills/index.json or /.well-known/skills/index.json (RFC 8615)
  Blob fast-path: vercel-labs, vercel, heygen-com owners + BLOB_ALLOWED_REPOS get a CDN download instead of clone
  ref support: owner/repo@branch/tag/sha
  subpath support: owner/repo/subdir
  skillFilter via @: owner/repo@skill-name

Install mechanics (src/installer.ts):
  sanitizeName(): kebab-case, no path traversal, max 255 chars, fallback "unnamed-skill"
  isPathSafe(): validates target is within basePath (prevents traversal)
  InstallMode: 'symlink' (default, single source of truth) | 'copy' (independent per agent)
  Canonical path: .agents/skills/<skill-name>/ (project) or ~/agent.globalSkillsDir/<skill> (global)
  Symlinks for non-universal: canonical -> agent skillsDir
  Windows note: symlinks fail without Developer Mode; falls back to copy with warning

Security posture:
  Telemetry: track() called on install with source, skill names, agent list (skipped for private repos)
  OpenClaw block: sources from openclaw/ owner blocked unless --dangerously-accept-openclaw-risks
  Security audit display: fetchAuditData() fetches ATH + Socket + Snyk ratings from skills.sh for GitHub sources
  Outro text VERBATIM: "Review skills before use; they run with full agent permissions."
  NO sandbox, NO signature verification of SKILL.md content. Skills are arbitrary markdown that runs as agent instructions.
  A SKILL.md can instruct the agent to run any shell command the agent has permission to run.

Lock files:
  skills-lock.json (global, ~/.config/agents/ or XDG): tracks global installs for update + experimental_install
  .agents/skills-lock.json (project): tracks project installs
  Both record: source, sourceType, skillFolderHash (SHA-256 of skill folder contents -- NOT blake3)

Tests:
  vitest-based, in tests/ directory
  tests/blob-fetch-tree-auth.test.ts, tests/blob-root-skill.test.ts, tests/cross-platform-paths.test.ts,
  tests/dist.test.ts, tests/eve-agent.test.ts, tests/full-depth-discovery.test.ts,
  tests/installer-copy.test.ts
  src/*.test.ts: add.test.ts, cli.test.ts, git.test.ts, init.test.ts, list.test.ts,
    remove.test.ts, source-parser.test.ts, use.test.ts, update-source.test.ts

Behavioral contracts:
  skills add -> parse source -> (blob fast-path or git clone to temp) -> discoverSkills() -> interactive prompts -> installSkillForAgent() per target -> track() -> addSkillToLock() -> cleanup tempDir
  skills add inside an AI agent: auto-yes, auto-target detected agent, no logo/banner shown
  skills use: generates a temporary prompt with skill content piped to stdout (no filesystem writes)
  skills experimental_install: reads skills-lock.json, re-installs each entry; restore workflow for team onboarding

Platform constraints:
  Node >=18 required
  Symlinks on Windows require Developer Mode (falls back to copy)
  xdg-basedir used for config paths on Linux/Mac; APPDATA env used for Zed on Windows
  eve agent has no globalSkillsDir (project-only by design)
  Agent detection: CLAUDE_CONFIG_DIR, CODEX_HOME, VIBE_HOME, HERMES_HOME, AUTOHAND_HOME env vars respected

Gotchas:
  skill-lock.json uses SHA-256 for hashing, not blake3 (not our code but important for integration)
  Installing to --agent '*' hits ALL 54 agents, including ones not installed on the machine (writes dirs anyway)
  --full-depth disables blob fast-path, always clones
  Running `npx skills` (the video recommendation) is equivalent to `bunx skills` per bun-not-node rule
  Telemetry is on by default; no opt-out flag visible in source (tracks installs to skills.sh)
  OpenClaw community skills are unverified user submissions, blocked by default with clear warning
  No code signing or content hashing of SKILL.md at install time -- content trust is zero

Critical assertions (testable):
  assert sanitizeName("../evil/../path") == "evil-path"  # path traversal neutralized
  assert isPathSafe("/home/user", "/home/user/skills/foo") == True
  assert isPathSafe("/home/user", "/home/other") == False
  # Skills installed to .claude/skills/<name>/SKILL.md for claude-code agent
  # Skills installed to .agents/skills/<name>/SKILL.md for universal agents
