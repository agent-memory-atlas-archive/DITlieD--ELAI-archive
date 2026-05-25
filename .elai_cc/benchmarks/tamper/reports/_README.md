D-25 tamper-suite report sink

`elai bench tamper --output <path>` writes a JSON
[`TamperBenchmarkReport`] here (default: stdout). The schema mirrors
the orchestrator-side `TamperBenchmarkReport` struct:

  {
    "per_scenario": [
      {
        "id": "scenario_01",
        "scenario_kind": "PositiveControl",
        "expected_error_kind": null,
        "observed_outcome": "CleanPass",
        "wall_ms": 12,
        "passed": true
      },
      ...
    ],
    "gating_verdict": "Pass",
    "max_wall_ms": 47,
    "total_scenarios": 10,
    "passed_count": 10,
    "failed_count": 0,
    "scenarios_blake3": "<64-hex>",
    "scenario_kind_counts": { "PositiveControl": 1, "BrokenParent": 1, ... }
  }

Files in this directory are NOT checked into git (per
.elai_cc/.gitignore or workflow convention). Treat them as
ephemeral build artifacts.
