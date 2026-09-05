# Why F-plan wiring beats the B/C/D/E plans
Written 2026-06-02 (session s-6344). Grounded in F-0_INDEX.md "Why F-plan exists" + the W-1..W-10 doctrine,
and CONFIRMED empirically by the four F-plan units shipped this session (the audit gates caught the exact
failure class in 3 of 4 units). This is the explainer; F-0_INDEX.md is the canonical spec.

================================================================================
THE PROBLEM F-PLAN EXISTS TO SOLVE
================================================================================
The night-run of 2026-05-30 cross-checked 187 B/C/D/E plans and found the spread:
  ~52% SHIPPED-WIRED, ~15% SHIPPED-WIRE-DARK (code lives, ZERO production callers),
  ~17% SHIPPED-ENV-GATED (behind ELAI_*_ENABLE=1, default OFF), ~10% PARTIAL, ~5% NOT-SHIPPED.
So ~40-45% of the inventory was wire-dark, env-gated, or partial. The handoff's own words:
"the production graph is sparse - many modules ship symbols + tests, but the canonical orchestrator/cli/
dashboard entry points never invoke them" + "~232 invisible backend signals (producer exists, no consumer)."

The cruel part: the B/C/D/E plans ALREADY carried a WIRING_CONTRACT block. It did not save them, because
the contract was a DOCUMENTED block, never an ENFORCED, EXECUTED done-gate. A plan could write
"produces X, consumes Y, reachable from Z" and then ship X with no caller, Y never read, Z never wired -
and still mark itself DONE on "cargo green + tests pass." Even the wires that DID land during the night-run
shipped default-OFF (E-17, B-40, D-16, E-49) - dark by another name.

THE ROOT CAUSE, in one line: the B/C/D/E done-gate was "does it COMPILE and do the UNIT TESTS PASS?"
That question is blind to wire-dark code. A feature with zero production callers compiles fine and its
unit tests pass fine - because the tests call it directly, exactly like the orphaned producer.

================================================================================
THE F-PLAN DIFFERENCE, IN ONE SENTENCE
================================================================================
B/C/D/E asked "does it compile + do the tests pass?" (cargo-green).
F-plan asks "did the feature actually FIRE end-to-end in production, reachable from a REAL entrypoint,
ON by default, leaving an OBSERVABLE trace a human can see in the dashboard?" (an EXECUTED contract).

cargo-green is NECESSARY but NOT SUFFICIENT. The F-plan done-gate is the wiring contract RUN, not read:
  python .claude/tools/wiring_contract_check.py --bn-file <F-N>.md --run-trigger  must return PASS.

================================================================================
THE 10 ENFORCEMENT CLAUSES (W-1..W-10) AND THE OLD-PLAN FAILURE EACH PREVENTS
================================================================================
W-1  NO-WIRE-DARK DONE-GATE. The contract is EXECUTED: every produces_symbol exists as a real DEF, every
     consumes_symbol has a real caller, every reachable_from anchor is reached by BFS from a production
     entrypoint, the stub_ban finds zero forbidden literals, AND the observable_effect actually RAN and
     emitted its expected SSB row + artifact. (Prevents: "DONE on cargo-green while wire-dark.")

W-2  NO DEFAULT-OFF ENV GATE AS THE TERMINAL STATE. A kill-switch to DISABLE is allowed, but the DEFAULT
     must be ON / in the production path. (Prevents: the ELAI_*_ENABLE=1-default-off escape hatch that the
     night-run wires fell through - dark by another name.)

W-3  PRODUCTION ENTRYPOINT REQUIRED. Reachable from a REAL entrypoint: a CLI subcommand, an FSM transition/
     evidence gate, a PreToolUse/PostToolUse hook, or a Tauri command. A symbol with only #[cfg(test)]
     callers is wire-dark = NOT DONE. (Prevents: the ~232 "producer exists, no consumer" signals.)

W-4  FRONTEND PAIR MANDATORY for any dashboard-surfaceable signal: the Svelte consumer + the Tauri command,
     with cross_lang_diff PASS (no Rust<->TS drift). (Prevents: backend-only invisible signals no human sees.)

W-5  OBSERVABLE_EFFECT IS A LIVE END-TO-END TRIGGER, not a unit test. A real command a human can run that
     produces a real SSB row + a real artifact. "cargo test passes" is NEVER the observable effect. Where a
     live model is unavailable, ship a DETERMINISTIC FIXTURE HARNESS that exercises the SAME production code
     path (not a mock), and gate the live run as a separate checkbox. (Prevents: tests-pass-but-never-runs.)

W-6  INTEGRATION CHECKPOINT THROUGH THE DASHBOARD. >=1 acceptance item verified THROUGH the dashboard.
     (Prevents: "works in a test, never surfaced to the user.")

W-7  NO HALLUCINATED INTEGRATION POINTS. Every existing symbol the new code attaches to is confirmed real
     in elai.map AT AUTHORING TIME. (Prevents: contracts that hook into a function that does not exist - the
     "I had to wire stuff later" root cause; the B-40/B-42 bad-cite failure mode applied to wiring.)

W-8  PRODUCES/CONSUMES SYMMETRY. Every produced symbol meant to be invoked must have an in-plan caller
     WITHIN THE SAME F-N. A produced fn with no in-plan caller is wire-dark BY CONSTRUCTION. (Prevents:
     "ship the producer now, wire the consumer later" = never.)

W-9  SSB VARIANT + GOLDEN LOCK. New SSB action_types flow through the action_type golden file; SCHEMA bump
     named; cross_lang_diff + action_type_lock green. (Prevents: silent journal-schema drift.)

W-10 TIERED-GATE RULE. Any axis needing NETWORK or SUBPROCESS at gate time is SPLIT: the in-process Tier-1
     ships genuinely fail-closed and IS the done-gate; the network/subprocess Tier-2 is separately gated and
     MUST NOT advertise fail-closed if it is fail-open. (Prevents: a fail-open gate implying safety it does
     not deliver - worse than no gate.)

DONE DEFINITION (the FSM may not reach COMPLETE without ALL of these):
  (1) cargo check --workspace GREEN, (2) auditor PASS, (3) wiring_contract_check --run-trigger PASS,
  (4) cross_lang_diff PASS if a frontend_pair exists, (5) the W-6 dashboard E2E item verified,
  (6) memory-writer ran. cargo-green-alone = NOT DONE.

================================================================================
EMPIRICAL PROOF FROM THIS SESSION (the doctrine caught what cargo-green would have shipped)
================================================================================
Four F-plan units were built this session. In 3 of the 4, the code compiled clean (cargo-green) and the
unit/contract tests passed - and the feature was STILL wire-dark or broken on the live path. The W-doctrine
gates caught every one. These are not hypotheticals; they are the exact B/C/D/E failure class, reproduced
live and then blocked:

  Unit A (F-1 VibeGate): the executors re-introduced the EXACT default-off env antipattern (W-2) under a
    RENAMED variable, AND left the SSB writer dormant with no production caller (W-3). cargo-green PASSED.
    The auditor + wiring-validator caught both -> fixed to default-ON + wired at the real dispatcher build.

  Unit C (F-3 Tool Filter): the enforcement POINTS were wired but NO production bootstrap instantiated them
    (W-3 wire-dark). cargo-green PASSED. THEN, after fixing that, a tokio runtime-ordering panic
    (Ssb::new constructed outside the runtime) that BOTH cargo-check AND the pure-function contract test
    masked - because the test called the pure scorer directly, never the production SSB-emit path. ONLY the
    W-5 LIVE --run-trigger caught it. After the fix the live run wrote real tool_filter_committed +
    untrusted_var_quarantined rows and blocked an out-of-set write (asr 1.0 -> 0.0).

  Unit D (F-9 dep-gate): the live gate path NEVER FIRED - the production caller passes file_path=None +
    task-summary text, and the gate's extractor routed on file extension, so it found nothing on every live
    write (W-3 dark). Separately, the live registry checker's JSON contract did not match the Python checker,
    so the network leg never blocked. cargo-green + the OFFLINE contract test PASSED through both bugs.
    W-3/W-5 caught them -> fixed; the live offline --run-trigger then wrote real write_time_slopsquat_blocked
    rows. (Unit B was clean precisely because the W-2/W-3 lessons from A were baked into its prompts up front.)

Every one of these would have shipped GREEN-AND-DARK under the old B/C/D/E "cargo-green + tests pass" gate.
That is the difference, demonstrated: the F-plan gate makes wire-dark UN-shippable.

================================================================================
THE BOTTOM LINE
================================================================================
Old plans proved the code EXISTS. F-plan proves the code RUNS - in production, by default, reachable from a
real entrypoint, leaving a trace a human can see. The WIRING_CONTRACT went from a paragraph you write to a
gate you must EXECUTE and PASS. That single change (documented -> executed) is why ~40-45% of B/C/D/E could
ship wire-dark and F-plan units cannot.
End of F-WIRING-DOCTRINE-WHY.md.
