E-14 browser-e2e journeys
=========================

This directory holds journey fixtures consumed by the browser_e2e runner
in `crates/orchestrator/src/browser_e2e/`. One TOML file per critical
user journey (signup, first-value loop, checkout, etc.).

Loading
-------

Parsed by `crates/orchestrator/src/browser_e2e/journey.rs::load_journey`.
The function returns `Result<Journey, E2eError>`; malformed TOML surfaces
as `E2eError::MalformedOutput`, missing file surfaces as `E2eError::Io`.

Schema
------

Each journey is a single TOML document with three top-level keys:

    name         (string, required)  Snake-case identifier. Stable
                                     across runs — the dashboard
                                     attributes pass/fail counts to
                                     this name.

    url          (string, required)  Entry URL for the journey.

    [[steps]]    (array, optional)   Sequence of actions the runner
                                     walks. Each step is a table with
                                     fields:
                                         kind   = "navigate" | "click" |
                                                  "fill" | "wait_for"
                                         target = "<url or selector>"
                                         value  = "<fill value>"
                                                  (optional, only used
                                                  by `fill`)

    [[assertions]] (array, optional) Pass/fail checks against the page
                                     snapshot. Each assertion table
                                     carries:
                                         kind  = "contains_text" |
                                                 "has_selector" |
                                                 "status_ok"
                                         label = "<snake_case_id>"
                                       payload (varies by kind):
                                         contains_text  -> needle
                                         has_selector   -> selector

Reference
---------

See `example_journey.toml` in this directory for a fully-populated
example covering all step kinds and assertion kinds.

CLI surface
-----------

* `elai browser list-journeys`  — print every TOML file in this dir.
* `elai browser run <name>`     — run the named journey via Playwright.
* `elai browser status`         — print recent SSB rows from the
                                  `browser_e2e_*` action_type family.

RUNPOD-PENDING
--------------

Live Playwright execution requires `npx` + `playwright` + (for the
axe-core gate) `@axe-core/playwright` on PATH. The runner posture is
wire-dark: when any of these is missing it returns
`E2eError::ToolchainMissing` rather than panicking.

VLM-tier validation (Mode A Qwen3.5-VL-30B-A3B / Mode B InternVL3.5-38B)
is deferred to RUNPOD-PENDING — neither model fits the 12 GB primary
VRAM box. The scope-lean cut runs Tier 1 only (HTML / AX tree / axe-core).
