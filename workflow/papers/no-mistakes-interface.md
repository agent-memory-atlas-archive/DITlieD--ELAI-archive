Source: no-mistakes
Reliability: MEDIUM
Files examined:
  docs/.source/no-mistakes/go.mod
  docs/.source/no-mistakes/docs/src/content/docs/reference/pipeline-steps.md
  docs/.source/no-mistakes/docs/src/content/docs/reference/repo-config.md
  docs/.source/no-mistakes/internal/agent/agent.go
  docs/.source/no-mistakes/docs/src/content/docs/concepts/pipeline.md (not read; pipeline-steps.md covers all steps)

Language: Go 1.25.0. License: MIT (CONTRIBUTING.md + LICENSE). Tag: v1.30.2.
Module: github.com/kunchenguid/no-mistakes.

Dependencies (from go.mod, exact versions verified):
  github.com/charmbracelet/bubbles v1.0.0            (TUI component library)
  github.com/charmbracelet/bubbletea v1.3.10         (Elm-arch TUI framework)
  github.com/charmbracelet/lipgloss v1.1.0           (TUI styling)
  github.com/mattn/go-isatty v0.0.20
  github.com/muesli/termenv v0.16.0
  github.com/oklog/ulid/v2 v2.1.1                    (run IDs)
  github.com/spf13/cobra v1.10.2                      (CLI)
  github.com/toon-format/toon-go v0.0.0-20251202084852-7ca0e27c4e8c  (TOON output format)
  golang.org/x/sys v0.42.0
  gopkg.in/yaml.v3 v3.0.1                             (config parsing)
  modernc.org/sqlite v1.48.1                          (run/session DB; pure Go, no CGO)

Entry points:
  cmd/no-mistakes/main.go           — main CLI binary
  cmd/fakeagent/main.go             — test harness: fake Claude/Codex/OpenCode agent for testing
  cmd/genskill/main.go              — generates SKILL.md from CLI home output (--check for CI guard)
  cmd/recordfixture/main.go         — records agent SSE streams to fixture files for replay tests
  internal/agent/agent.go           — Agent interface + Result/RunOpts/TokenUsage types
  internal/agent/claude.go          — Claude Code driver
  internal/agent/codex.go           — Codex driver
  internal/agent/opencode.go        — OpenCode driver (HTTP stream)
  internal/agent/acpx.go            — ACP protocol driver

Public API (internal/agent/agent.go):

Agent interface:
  Name() string
  Run(ctx context.Context, opts RunOpts) (*Result, error)
  Close() error

RunOpts struct:
  Prompt     string
  CWD        string
  JSONSchema json.RawMessage   // structured output schema, optional; triggers text->JSON parse if set
  OnChunk    func(text string) // streaming text callback, optional

Result struct:
  Output json.RawMessage   // nil if no JSONSchema supplied
  Text   string
  Usage  TokenUsage

TokenUsage struct:
  InputTokens         int
  OutputTokens        int
  CacheReadTokens     int
  CacheCreationTokens int

PIPELINE STAGE MACHINE (verbatim order from pipeline-steps.md):

intent -> rebase -> review -> test -> document -> lint -> push -> pr -> ci

Each step: produces findings, requests approval, triggers auto-fix, or applies safe fixes during its own pass.
Steps that encounter fatal errors stop the pipeline.

STEP DEFINITIONS:

1. INTENT
   Purpose: Infer the author's intent from agent transcripts for use by all downstream steps.
   Sources: agent-supplied intent (verbatim, skips inference) | transcript-based inference from
     Claude Code, Codex, OpenCode, Rovo Dev, or Pi local transcripts (when intent.enabled=true).
   Matching: matches transcripts against non-deleted changed files; agent-disambiguated on plausible matches.
   Output: derived {summary, source, session_id, match_score} stored on run.
   Failure modes: does NOT block pipeline for missing transcripts, timeouts (5-min cap), or failures.
     Can fail only if cleanup after disambiguation agent leaves worktree side effects.
   Config: intent.enabled (bool), intent.threshold (float, default 0.2), intent.slack_days (int, default 3).

2. REBASE
   Purpose: Fetch upstream + push target, rebase branch.
   Behavior (from pipeline-steps.md verbatim):
     "Fetches origin/<default_branch> into the worktree, and also fetches the pushed branch for
     non-default branches unless the push rewrote branch history."
     "If the branch is not the default branch, tries rebasing onto the pushed-branch target first,
     then origin/<default_branch>."
     "If a fast-forward is possible, does a hard-reset instead of a rebase."
     "If the diff against the default branch is empty after rebase, completes rebase and skips all
     remaining pipeline steps."
     "On conflict: records conflicting files, aborts the rebase, and reports findings."
   Fork routing: pushed-branch target is refs/remotes/no-mistakes-push/<branch> for forks.
   Auto-fix: agent resolves conflict markers, stages, runs git rebase --continue in non-interactive env.
     Prompt includes user intent. Fix commits: "no-mistakes(rebase): <summary>". Limit: 3.
   Manual fix rounds: include per-conflict user notes + selected user-authored findings + sanitized prior-round history.

3. REVIEW
   Purpose: AI code review of the diff.
   Behavior: diffs base..head; filters ignore_patterns; sends to agent with structured review instructions
     + JSON output schema; includes user intent when available.
   Agent returns: findings[]{severity, file_location, description, action} + risk_level + risk_rationale
     severity: "error" | "warning" | "info"
     action: "no-op" | "auto-fix" | "ask-user"
     risk_level: "low" | "medium" | "high"
   Approval: required if any finding has severity "error" or "warning".
     action=ask-user: pause for approval (challenges author intent, not routine fixes).
     action=auto-fix: eligible for fix loop.
     action=no-op: informational.
   Auto-fix: agent receives selected prev findings + per-finding user notes + selected user-authored findings
     from TUI/AXI + sanitized history (prior round summaries + skipped findings).
     Follow-up passes avoid re-reporting user-ignored findings unless materially different problem.
     Fix commits: "no-mistakes(review): <summary>". Default limit: 0.

4. TEST
   Purpose: Run baseline tests + gather evidence for the intended behavior.
   Behavior:
     If commands.test set: runs via sh -c (POSIX) or cmd.exe /c (Windows); non-zero = error findings.
     If commands.test empty OR intent available after baseline: agent validates with evidence-oriented
       tests or manual checks.
     Returns: findings + tested[] + testing_summary (optional) + artifacts[{path|url|content}].
     artifacts.path: repo-relative or absolute under no-mistakes-evidence/<runID>/.
     artifacts.url: externally visible.
     artifacts.content: short inline logs.
     Evidence storage: default temp (no-mistakes-evidence/<runID>); store_in_repo=true -> <dir>/<branch-slug>.
     Branch-slug rules: slashes become nested dirs; unsafe chars replaced; empty falls back to run ID.
     Agent removes transient artifacts (models, caches, build outputs) but preserves evidence + source changes.
     If agent creates new test files (git status --porcelain): approval required even if tests pass.
   Approval: action=ask-user pause (includes missing-evidence warnings). action=auto-fix: fix loop.
   Auto-fix: same prompt structure as rebase/review. Fix commits: "no-mistakes(test): <summary>". Limit: 3.

5. DOCUMENT
   Purpose: Update matching docs for code changes; report only unresolved gaps.
   Behavior: diffs base..head; skips if no non-ignored changed files.
     Agent: find every doc gap, update docs/doc comments, verify edits, commit any changes.
     Returns findings only for UNRESOLVED gaps or human judgment calls.
   Approval: required whenever any unresolved doc finding returned, INCLUDING info findings.
   Auto-fix: fixes happen during initial pass; unresolved pause for approval (no auto follow-up loop).
     Manual fix: standard prompt + prior history. Fix commits: "no-mistakes(document): <summary>".

6. LINT
   Purpose: Linters + static analysis.
   Behavior:
     If commands.lint set: runs via shell; non-zero = warning findings.
     If commands.lint empty: agent detects linters/formatters, applies safe fixes, reruns, commits,
       returns findings only for unresolved issues. Unresolved findings pause for approval (no auto loop).
   Auto-fix: when commands.lint configured, same pattern as test. Fix commits: "no-mistakes(lint): <summary>". Limit: 3.

7. PUSH
   Purpose: Push validated branch to configured push target.
   Behavior:
     If commands.format set, runs it first.
     Stages in-repo test evidence when store_in_repo enabled.
     Commits uncommitted agent changes: "no-mistakes: apply agent fixes".
     Without fork routing: pushes to repos.upstream_url (from origin).
     With fork routing: pushes to repos.fork_url.
     ls-remote to get current SHA before push.
     --force-with-lease for existing branches; regular push for new.
     Updates run's head SHA in DB after push.
   Never requires approval. Auto-triggers after review + test + document + lint pass.

8. PR
   Purpose: Create or update pull request.
   Platforms: GitHub (gh CLI), GitLab (glab CLI), Bitbucket Cloud (email+token env).
   Skips when: default branch | unsupported host | missing CLI/credentials | legacy fork on GitLab/Bitbucket.
   Behavior:
     Checks for existing PR; updates if exists, creates if not.
     Title: agent-generated, conventional commit format (type(scope): desc | type: desc).
       user-facing scope -> feat or fix for release automation.
       scope = primary affected real module/package from changed paths, broad not file-level.
     Body sections: ## Intent (if available) | ## What Changed | ## Risk Assessment | ## Testing | ## Pipeline.
       ## Pipeline: issue -> fix -> verification narrative using fix summaries + re-check success + still-open findings.
       ## Testing: prefers testing_summary prose; fallback to check count; evidence artifacts rendered compactly.
         path artifacts -> Evidence links (GitHub: blob URL); content -> collapsible details blocks.
         Readable UTF-8 from temp evidence dir embedded inline with truncation; binary/visual -> local file refs.
     Stores PR URL in DB; streams to TUI.
   Fork routing (GitHub): --repo pointed at parent, head = <fork-owner>:<branch>.

9. CI
   Purpose: Monitor PR health; auto-fix CI failures and merge conflicts.
   Platforms: GitHub (gh), GitLab (glab), Bitbucket Cloud (API).
   Poll schedule: every 30s for 5min, every 60s for 5-15min, every 120s after.
   Continues until PR merged/closed/declined/timeout, even after checks healthy.
   Merge conflict handling: GitHub + GitLab only (via mergeability API).
   Ready signal: TUI + terminal title show "Checks passed" when checks green + mergeability clear.
     AXI returns outcome:checks-passed with reporting instructions for agents.
     Ready signal clears if checks restart, new failures appear, or PR merges/closes.
   CI failure fix: fetches failed job logs, sends to agent with intent, commits + force-pushes if changes.
   Merge conflict fix: agent rebases onto latest default-branch tip; smallest correct root-cause fix.
   Combined: fixes both CI failure + merge conflict in one attempt.
   Deduplication: dedups only after fix actually committed and pushed.
   Timeout: pauses for user approval (even if CI green; findings for failures/conflict/unresolved mergeability).
   Auto-fix limit: 3 total CI auto-fix attempts (covers both CI failure + merge conflict attempts).

STEP STATUSES:
  pending | running | fixing | awaiting_approval | fix_review | completed | skipped | failed

REPO CONFIG (.no-mistakes.yaml, VERBATIM schema):
  agent: auto|claude|codex|rovodev|opencode|pi|acp:<target>   (default: from global config)
  allow_repo_commands: bool (default false) — SECURITY: commands.* and agent read from DEFAULT BRANCH only unless this is true
  commands.lint: string (sh -c / cmd.exe /c) | empty = agent auto-detects
  commands.test: string | empty = agent auto-detects
  commands.format: string run before push step commits agent fixes
  ignore_patterns: string[] (basename match | /**  subtree | path/with/slash full glob)
  auto_fix.rebase: int (default 3) | auto_fix.review: int (default 0) | auto_fix.test: int (default 3)
  auto_fix.document: int | auto_fix.lint: int (default 3) | auto_fix.ci: int (default 3)
  intent.enabled: bool (default true) | intent.threshold: float (default 0.2) | intent.slack_days: int (default 3)
  intent.disabled_readers: string[] (adds to global disables; values: claude|codex|opencode|rovodev|pi)
  test.evidence.store_in_repo: bool (default false) | test.evidence.dir: string (default .no-mistakes/evidence)

SECURITY RULE (from repo-config.md verbatim):
  "commands.* execute arbitrary shell on the daemon host via sh -c / cmd.exe /c, and agent selects which process
  launches there... the daemon always reads commands and agent from your default branch (e.g. origin/main),
  never from the pushed SHA... If the fetch fails, both fields are forced empty."

INTENT INFERENCE:
  Matches local agent transcripts (Claude Code, Codex, OpenCode, Rovo Dev, Pi) against non-deleted changed files.
  5-minute extraction cap; skips (not fails) on timeout or missing transcripts.
  Stored: {summary, source, session_id, match_score} on the run.
  Used by: rebase fixes, review checks+fixes, test detection+evidence validation, document checks+fixes,
    lint detection+fixes, CI auto-fixes, PR drafting.

ADVERSARIAL REVIEW CONTRACT (from pipeline-steps.md):
  Structured output schema enforced: findings[]{severity, file_location, description, action} + risk_level + risk_rationale.
  Review prompt includes intent when available. Fresh context window per invocation (Agent.Run).
  Follow-up review passes receive history of prior rounds + ignored findings to prevent re-reporting user-dismissed issues.

EVIDENCE RECORDING FORMAT:
  Each test step returns artifacts array:
    {type:"path", path:"/abs/or/repo-relative"} -> rendered as Evidence link; GitHub converts to blob URL
    {type:"url", url:"https://..."} -> External Evidence link
    {type:"content", content:"short text"} -> Collapsible details block in PR
  Readable UTF-8 text from temp evidence dir: embedded inline with truncation (large files).
  Binary, visual, over-budget local: rendered as non-link local file reference.

RISK ASSESSMENT:
  Produced by review step: risk_level (low|medium|high) + risk_rationale (text).
  Included in PR body under ## Risk Assessment section.
  Not a numeric score; qualitative assessment by the reviewing agent.

PR BABYSIT LOOP:
  CI step polls provider at increasing intervals.
  AXI interface surface: when checks pass + mergeability clear, returns outcome:checks-passed
    with instructions for agent to summarize, ask user to review+merge, list pipeline fixes.
  Loop continues until merged/closed/declined/timeout; timeout -> pause for approval.
  Max 3 CI auto-fix attempts total (CI failures + merge conflicts combined).

AGENT ABSTRACTION (internal/agent/agent.go, verbatim):
  type Agent interface { Name() string; Run(ctx context.Context, opts RunOpts) (*Result, error); Close() error }
  JSONSchema in RunOpts triggers structured output parsing; empty schema -> text-only result.
  finalizeTextResult parses JSON from text output when schema set; fails if text empty.
  Supported backends: claude, codex, opencode (HTTP stream), acpx (ACP protocol), rovodev.
  fakeagent (cmd/fakeagent/): fixture-based test double for all backends; records via cmd/recordfixture/.

TESTS:
  cmd/fakeagent/*_test.go     -> agent fixture replay tests
  cmd/no-mistakes/main_test.go -> CLI integration
  cmd/recordfixture/*_test.go -> fixture scrubbing/recording
  internal/agent/*_test.go    -> agent driver unit tests
  install_script_test.go      -> install script tests
  .no-mistakes/evidence/      -> captured evidence from no-mistakes' own pipeline runs on itself (self-hosting)

DESIGN PHILOSOPHY (from docs verbatim):
  "Every pipeline agent invocation is prompt-steered to keep intentional writes inside the run worktree
  and avoid mutating system state outside it. This is a soft boundary, not OS-level sandbox enforcement."
  "The steering still allows requested test evidence under the managed temporary no-mistakes-evidence
  directory or the configured in-repo evidence directory, plus incidental temp or cache writes from
  normal development tools."

Gotchas:
  No OS-level sandbox: soft prompt steering only; a prompt-injection in a test could escape.
  commands.test / commands.lint read from DEFAULT BRANCH always unless allow_repo_commands=true.
  test step: if agent creates new test files (git status --porcelain) -> approval required even if tests pass.
  document step: unresolved findings pause for approval including info severity (stricter than other steps).
  pr step: scope in title should be "broad rather than file-level" per spec.
  CI auto-fix limit is TOTAL (CI failures + merge conflicts combined).
  go.mod says go 1.25.0 which is a future version as of 2026-06 (currently stable is 1.24.x) -- this is likely a dev branch target; UNVERIFIED against released binary.

Critical assertions (testable):
  assert pipeline order = ["intent","rebase","review","test","document","lint","push","pr","ci"]
  assert review default auto_fix limit = 0 (findings require manual approval by default)
  assert commands.* read from default branch (never pushed SHA) unless allow_repo_commands=true
  assert empty diff after rebase -> all remaining pipeline steps skipped
  assert test new-file creation -> approval required regardless of test outcome
  assert CI auto-fix limit covers combined CI-failure + merge-conflict attempts (not separate budgets)

RELIABILITY: MEDIUM (reviewed source, official public repo, no decompilation)
