I-46 bash interceptor + A-26 consolidation -- executor decisions

WHAT
  Deliver crates/tools/src/bash_interceptor.rs (classify/redirect/fixup/prefix),
  insert into lib.rs run_bash before routing_rules, emit BashRedirected SSB rows
  from InProcessDispatch::dispatch_tool, add Pattern-2 3-arm BashRedirected SSB
  variant, add contract test i46_bash_interceptor_contract.rs, consolidate A-26
  routing_rules per CONTROLLER RULING D-028 (OPTION A).

KEY DECISIONS

D-1: A-26 CONSOLIDATION = OPTION A (binding from controller ruling).
  Remove routing_rules rules 2 (cat/head/tail), 4 (find+glob), 5 (grep/rg) and
  their 7 tests. Retain rule 1 (echo-redirect/heredoc->write_file) and rule 3
  (sed/awk->edit_file). bash_interceptor.rs becomes the sole redirect path for
  the removed verbs. BUILTIN_RULE_COUNT 5->2.

D-2: LIVE SSB BASELINE (critical correction from exploration doc).
  Context package stated 240->241 / TV 549->552. Exploration doc stated 241->242 / TV 550->553.
  Live ground truth (re-read ssb.rs:3169, action_type_lock.rs:712, all_test_payloads assert):
    SCHEMA_VERSION = 243 (I-41 CritiqueLensFinding was the last bump)
    TOTAL_VARIANTS = 552
    all_test_payloads len = 453 (one representative per variant I-41)
  I-46 targets: SCHEMA_VERSION 243->244, TOTAL_VARIANTS 552->553 (+1 variant),
    all_test_payloads 453->456 (+3 arms representative).
  e45 pin update: 552->553.

D-3: BashRedirected = ONE action_type variant (+1 TV).
  Controller correction binding: ONE action_type "bash_redirected" with 3-arm
  BashRedirectedKind (Redirected/FixedUp/PrefixBlocked). NOT 3 separate action
  types (+3 TV). Final TV = 553.

D-4: SSB EMIT SITE = InProcessDispatch::dispatch_tool.
  run_bash lives in crates/tools (no Ssb handle). The structured_error propagates
  as Err(String) up to execute_tool_with_enforcer_scoped (lib.rs:1478) then to
  the InProcessToolRunner closure (in_process.rs:119). At line 124 in dispatch_tool,
  parse the Err string for error_code="bash_redirected" / "bash_fixedup" /
  "bash_prefix_blocked" and emit SsbPayload::BashRedirected before returning Err.
  serde_json already in the orchestrator crate (no new dep).

D-5: STRUCTURED ERROR CODES.
  Use error_code values that match the 3-arm discriminator:
    "bash_redirected"     -> BashRedirectedKind::Redirected
    "bash_fixedup"        -> BashRedirectedKind::FixedUp
    "bash_prefix_blocked" -> BashRedirectedKind::PrefixBlocked
  Propagated via the existing structured_error() JSON envelope.

D-6: FIXUP TABLE (2 patterns, deterministic, no brush-parser dep).
  Port gajae bash-command-fixup.ts description literally:
    1. Strip trailing "| head [args]" / "|& head [args]" (output-limiter, ELAI already truncates)
    2. Strip trailing "2>&1" on segment with no remaining pipe.
  Implementation: simple string suffix scan on the last segment of a split on
  ";", "&&", "||". No full AST tokenizer needed.

D-7: ALLOWED-PREFIXES TABLE.
  ELAI production prefix table: cargo, git, pnpm, npm, python, pytest (the
  commands an agent legitimately needs without tool routing). Ban logic from gajae:
  shell control chars (;|&<>()), command substitution ($() or backtick),
  unquoted expansion chars ($*?[]{}~), newlines, backslash escapes.
  split_whitespace is sufficient for unquoted commands (quoted-arg handling deferred
  per exploration doc LOW risk item).

D-8: REDIRECT INSERTION POINT.
  bash_interceptor::check_intercept() fires BEFORE routing_rules::check_routing
  in run_bash (OPTION A: overlapping routing_rules entries removed, so there is
  no double-fire risk; the interceptor is the only path for grep/cat/find verbs).

COVERAGE MAPPING (D-028 condition: 7 deleted tests -> new contract arms)

Old test (routing_rules.rs) -> New contract arm (i46_bash_interceptor_contract.rs)
  cat_file_is_rejected       -> i46_redirect_cat_to_read_file
  head_file_is_rejected      -> i46_redirect_head_to_read_file
  tail_file_is_rejected      -> i46_redirect_tail_to_read_file
  find_with_glob_is_rejected -> i46_redirect_find_glob_to_glob_search
  ls_with_glob_is_rejected   -> i46_redirect_ls_glob_to_glob_search (note: ls added to interceptor)
  grep_is_rejected           -> i46_redirect_grep_to_grep_search
  rg_is_rejected             -> i46_redirect_rg_to_grep_search

EXPECTED TEST OUTCOME
  cargo check --workspace --tests: exit 0
  cargo test -p elai-tools: routing_rules tests (no grep/cat/find/head/tail/rg/ls arms) PASS
  cargo test -p elai-orchestrator: action_type_lock (553), e45 (pin 553) PASS
  cargo test -p elai-contracts --test i46_bash_interceptor_contract: all arms PASS

UNKNOWNS
  None blocking. Golden file regen is handled by the regen_golden_file test (--ignored).
