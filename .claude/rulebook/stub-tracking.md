Stub tracking discipline (HARD RULE)

Loaded by: ALWAYS during baby step execution. Strengthens the existing "Zero lazy code (HARD RULE)" in token-discipline.md by allowing INTENTIONAL stubs only when registered with a resolver.


THE PROBLEM

LLMs commonly create stubs (TODO comments, hardcoded mocks, placeholder returns, fatalError calls) as scaffolding while implementing a feature. Without tracking, those stubs survive into production. The existing "zero lazy code" rule blanket-bans them, which is correct for unintended stubs but too strict for legitimate scaffolding (e.g. baby step 1 declares a protocol, baby step 2 wires a consumer, baby step 3 implements the producer — steps 1 and 2 will look "stubby" until step 3 lands).

This rule is the bridge: stubs ARE allowed if and only if they are registered with a resolver baby step. Unregistered stubs remain banned.


VALID STUB MARKERS (the only allowed scaffolding tokens)

- TODO       — explicit `// TODO` or `# TODO` comment with context
- PLACEHOLDER — a return value or function body that exists for compile but is fake
- MOCK       — hardcoded fake data that will be replaced
- UNIMPLEMENTED — function that throws / panics / fatalErrors with intent
- FORWARD_REF — interface declared without consumer, consumer expected later

BANNED scaffolding tokens (always lazy code, regardless of registration):
- `.unwrap()` / `try!` / `as!` (force-unwrap, force-try, force-cast — never registerable, always BLOCK)
- empty `catch {}` / `except: pass` / `_ = result` silent error swallows (never registerable)
- `fatalError("unimplemented")` is allowed ONLY if registered as UNIMPLEMENTED with resolver
- `// ...` ellipsis comment-out (never registerable — finish or delete the code)


REGISTRATION REQUIREMENT

When a baby step intentionally creates a stub:
1. Declare it under BLOCK 5 (STUBS) of the BABYSTEP-TEMPLATE.md.
2. Append a row to `workflow/sessions/{id}/stubs.md` (created lazily from STUBS-TEMPLATE.md on first use).
3. The row MUST include a `will_resolve_in: babystep_id` pointer.
4. The marker in source code MUST embed the babystep id, e.g.:
   - Swift: `// TODO[C2.P1.S3.B2]: replace mock with KeychainStore.fetch()`
   - TypeScript: `// PLACEHOLDER[C1.P2.S1.B1]: hardcoded user`
   - Python: `# UNIMPLEMENTED[C1.P1.S2.B3]: backend not wired`

The babystep id in the comment is what the auditor greps for to verify the stub is registered.


PRE_GATE CHECK (every baby step)

Before EXECUTING starts, the planning-workflow skill runs:

```
python .claude/tools/check-stubs.py --baby-step {current_id} --pre-gate
```

This:
1. greps stubs.md for any open stub with will_resolve_in <= current_id and status=open.
2. if any matches: BLOCK. The current baby step must close those stubs OR the user must explicitly defer them.
3. if no matches: pass.

The check tool is a small python script. If it does not exist yet, the rule still applies — the main LLM does the grep manually:

```
grep "^stub-" workflow/sessions/{id}/stubs.md | awk -F' \\| ' '$10=="open" && $6<=current_id'
```


DETERMINISTIC_CHECK (every baby step touching code)

Before AUDITING is allowed:
1. for each file touched by this baby step, grep for stub markers:
   ```
   grep -E "(TODO|PLACEHOLDER|MOCK|UNIMPLEMENTED|FORWARD_REF)\[[A-Z][0-9]+\.[A-Z][0-9]+\.[A-Z][0-9]+\.[A-Z][0-9]+\]" {file}
   ```
2. count the matches. count must equal the number of NEW stubs declared in BLOCK 5 of this baby step.
3. for each match, verify the embedded babystep id is in stubs.md status=open.
4. for each NEW stub declared in BLOCK 5, verify the source file has the corresponding marker.
5. count mismatch = BLOCK with "stub-registration-mismatch: declared {N}, found {M} in {file}".


AUDIT CHECK (auditor agent)

When the auditor enters AUDITING/RUN_TESTS, in addition to the standard build/test pipeline:
1. read workflow/sessions/{id}/stubs.md.
2. for every stub with will_resolve_in <= current_baby_step_id and status=open: BLOCK with "stub-not-resolved: stub-{id} should have been resolved by {baby_step_id} but is still open."
3. for every newly-introduced stub marker in touched files that does NOT match a registered stub-{id} in stubs.md: BLOCK with "unregistered-stub: {file}:{line} marker not in stubs.md."


PLAN COMPLETION CHECK (final MEMORIZING)

The last baby step in the decomposition tree triggers a plan-level stub check:
1. read workflow/sessions/{id}/stubs.md.
2. fail if any entry has status=open.
3. fail if any entry has status=closed without a populated justification field.
4. if all pass: stubs.md is finalized, the plan can transition to COMPLETE.

A plan may not COMPLETE while stubs are open. This is the strongest guarantee the rule provides.


CLOSED STUBS

A stub can be closed (status=closed) instead of resolved when:
1. it represents an intentional permanent gap (e.g. debug-only path, never-runs-in-production)
2. the user explicitly approved the closure with reason recorded in the justification field
3. a related concept was ReplanConcept'd and the stub no longer has a target

Closed stubs are kept in stubs.md as historical record, not deleted.


INTERACTION WITH ZERO LAZY CODE RULE

The existing rule in token-discipline.md (no empty function bodies, no TODO without context, no placeholder returns) remains in force. This rule REFINES that one:
- registered stubs (with babystep id and entry in stubs.md) are allowed
- unregistered stubs are still BLOCK at audit time
- banned tokens (force-unwrap, silent swallow, ellipsis comment-out) remain BLOCK regardless of registration

Both rules should be cited when stubs come up in audit findings.


CHECK-STUBS TOOL SPEC (for future implementation)

`python .claude/tools/check-stubs.py [options]`

Options:
- `--baby-step {id}`     check stubs relevant to a single baby step
- `--pre-gate`           run as a PRE_GATE (fail if any open stub <= current baby step)
- `--strict`             run as DETERMINISTIC_CHECK (registration consistency)
- `--plan-complete`      run final plan check (zero open stubs)
- `--session {id}`       override session id (default: read from FSM state)

Exit codes:
- 0  pass
- 1  registration mismatch or open stub past resolver
- 2  banned scaffolding token detected
- 3  stubs.md missing or malformed

Until the tool is implemented, the main LLM and auditor perform these checks via grep + manual count, and the rule still applies.
