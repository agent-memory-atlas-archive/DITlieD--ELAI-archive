# L-14 decisions — the one-shot verified builder front door

Plan: docs/thefinalglance/L-plan/L-14_one_shot_verified_builder_front_door.md
Task 7 deliverable. Written by the walk controller after the three build slices landed.

NOTE ON THIS FILE: the ELAI repo `.gitignore` is a whitelist whose first line is `*`, so
`workflow/reasoning/` is untracked. This file cannot be a commit. Its durable copy lives at
`conductor/opus_walk/evidence/L-14/L-14-decisions.md`.

## What shipped

| Task | State | Where |
| --- | --- | --- |
| 1 NN-5a `elai build "<idea>"` | DONE | a7e1595 (verb, parser, dispatch, `--plan-only`) + 93dcc140 (generation) |
| 2 NN-5b the RRA receipt, FS-3 | DONE | 93dcc140 |
| 3 L-11 output-quality gate over generated code | DARK, no designed seam | see D-3 below |
| 4 `elai artifact run` via the J-63 broker | RESIDUE, upstream unshipped | see D-4 below |
| 5 dashboard one-field flow | UI HOLD, never built by this walk | owner-ui-rework-hold-2026-08-06 |
| 6 expose as a J-2 MCP verb | DONE, read-only half | cee0d3f |
| 7 this file | DONE | on disk, untracked |

## D-1 The receipt and generation land together, never apart

The plan's stub_ban says no one-shot output may exist without an RRA receipt. So slice 1 shipped
the front door with the generating arm FAILING CLOSED, and slice 2 replaced that refusal with
generation plus the receipt in the same command. FS-3 also holds under failure: if the receipt
cannot be written the generated directory is removed and the command exits non-zero, and with
the existing `ELAI_WRAP_RRA_DISABLE` kill switch set nothing is generated at all.

## D-2 The scaffold is reached through a re-export, not a new dependency

`crates/cli` gains no dependency on the scaffold crate. The orchestrator, whose Cargo.toml
already depends on it, re-exports `apply_template`, `default_build_vars` and `ScaffoldError`.
This keeps GD-1: the shipped E-12 scaffold is driven, not reimplemented.

## D-3 Task 3 is DARK because no directory-level quality gate exists

Recon (conductor/opus_walk/evidence/L-14/S2-tasks346-recon.md) found no production function that
takes a directory or a file set and returns a quality verdict. What exists is:
- `MiniCritic::check(blank_id, body: &str, lang)` at
  crates/orchestrator/src/mini_critic.rs:171, which judges ONE fill-body string inside the
  skeleton generator (caller skeleton_generator.rs:295, top caller task_runner.rs:10801).
- `evaluate_vibegate(source, file_path)` at vibegate.rs:141, which judges ONE tool-use string.
- `solid_proxy_check.py` and `maintainability_check.py`, which are CLI scripts over elai.map
  rows, not Rust-callable APIs.

Gating a generated app therefore needs a new production seam that does not exist yet. K-27
forbids executing an undesigned seam, so task 3 goes to the escalation queue for a design turn
rather than being invented here. The uncovered mutation is named honestly: nothing today would
fail if the generated app were garbage.

## D-4 Task 4 takes the plan's own escape

The plan says: "if J-63 unshipped, emit the app + receipt and record residue". There is no
`elai artifact run` subcommand in crates/cli/src/args.rs today, confirmed by grep. So the build
emits the app and its receipt without an in-broker run, and this is the residue record.

## D-5 Only the read-only half of `elai build` is exposed over MCP

The Tier-1 MCP server runs as one hardcoded role, `planner` (mcp_server.rs:12), which
`.elai_cc/authz-policy.toml` defines as read-only under `default = "deny"`. The shipped J-2
contract test asserts every listed tool carries `readOnlyHint == true`. The generating arm
writes an app directory and prints to stdout, and a `println!` inside the MCP server corrupts
the stdio frame stream.

So slice 3 exposed `build_plan`: idea in, resolved class and E-12 profile out, nothing written.
`build_cmd` gained one extracted function, `resolve_build_plan`, so the CLI printer and the MCP
handler compute the same plan from the same code. Whether the generating arm is ever reachable
over MCP is an owner decision about roles and authz, not a code detail, and it stays unbuilt.

## D-6 Every slice was falsified by the controller, off agent

Slice 2: deleting the `write_rra` call site left the tree compiling and turned
`build_generates_app_and_receipt` RED with the other 8 tests green; build_cmd.rs restored byte
identical at sha256 e1ab3593c04aaf8d2cb9965a248dda8b515394ff67b04710f2a139cc2893baae.
Slice 3: deleting the `BUILD_PLAN => build_plan(&arguments),` dispatch arm left the tree
compiling and turned `j2_mcp_serve_exposes_build_plan_read_only` RED while the shipped
deny-default contract test stayed green; mcp_server.rs restored byte identical at sha256
7c470e9f7134b27bfc629827cdd2726eb1e031528b7543ea971ef322c7362c35.

Both mutations targeted the PRODUCTION CALL SITE, not the new function, so they falsify the
wiring rather than the unit.

## D-7 The gate is two commands, always

Per D-267, `cargo test -p elai-cli --test <name>` can rebuild only the test binary and reuse a
stale `elai` from the shared warm target dir, reporting a green that describes the previous
commit. Every gate in this plan ran `cargo build -p elai-cli --bin elai` FIRST, then the test.
This caught a real failure: slice 3's first attempt reported exit 0 through the worker's Depot
shim and then failed the caged build with `error: type BuildPlan is private`.
