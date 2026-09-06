# Conductor archive verification

The selected snapshot was checked on 2026-09-06 using Python 3.14.4 and pytest.
**The retained suite is not green: 491 tests passed and 15 failed**, with no skips,
30 passing subtests and 10 deprecation warnings. There are 506 test cases across
37 test modules. [verification.json](verification.json) lists the files and every
failing test identifier.

## What was tested

The run covered selected walk-state, selection, supervisor, provider-adapter,
worktree/worker, lease, database/event, gate, verifier, landing/recovery and
build-offload tests. Provider responses and external operations are largely
mocked. Some tests create real temporary Git repositories or child processes;
those ran entirely inside the isolated test environment.

Bubblewrap provided separate user, PID and network namespaces. The environment
was cleared, system files and installed Python packages were read-only, and a
disposable archive copy and temporary home were writable. The host's home,
credentials, live state and working repositories were not mounted. No provider
service, real work dispatch, canonical repository landing or remote push was run.

The equivalent inner pytest command was:

```sh
PYTHONPATH=conductor:conductor/opus_walk:conductor/tools:conductor/glm_walk:conductor/fallback_walk \
PYTEST_DISABLE_PLUGIN_AUTOLOAD=1 PYTHONDONTWRITEBYTECODE=1 \
python3 -m pytest --import-mode=importlib -p no:cacheprovider -q --tb=short \
  conductor/opus_walk conductor/tests conductor/fallback_walk/tests conductor/glm_walk/tests
```

This is the command **inside** the disposable, network-disabled environment.
Running it in an ordinary configured workspace does not reproduce that isolation.
The tests contain historical machine paths and process/file operations. Supply
pytest separately; package source and a Python environment are not vendored here.

## Retained failures

| Area | Failures | Observed mismatch |
| --- | ---: | --- |
| Build-offload routing/provenance | 6 | Old synthetic Git states do not meet the newer provenance requirements; another case lacks the excluded project-profile configuration. |
| Cursor provider adapter | 2 | Tests expect a numeric return code where the retained implementation returns `None`. |
| Fresh-verifier sandbox contracts | 3 | Tests expect a strict sandbox argument; the retained adapter produces `off`. The corresponding probe rejects that mismatch. This is a concrete reason not to assume secure isolation from the design claims. |
| Event handling/atomicity | 3 | Corrupt event JSON raises instead of soft-parsing; another test expects an `_insert_event` seam absent from the retained DB implementation. |
| Worker heartbeat | 1 | A test passes `lease_heartbeat_interval_seconds`, which the retained dispatch signature does not accept. |

For comparison, the same selected source files **before publication edits** were
copied into a second isolated environment: 485 tests passed and 21 failed. Every
public-snapshot failure also occurs in that source baseline; there are no new
failing test identifiers after the privacy and relocation edits. That comparison
does not establish behavior in the original live installation or prove equivalence
for untested paths. Raw baseline logs remain private.

No assertions, thresholds or failure expectations were relaxed to get these
numbers. Runtime defects and historical test drift were retained rather than
turning this archival import into a repair of the abandoned system.

## Packaging checks and limits

All 105 selected Python files compiled. Static import inspection found no missing
local Python modules in the selected dependency set. Apart from the two documented
helper-location changes, copied Python syntax trees differ only in string values
used for privacy edits. Original source-file hashes remained unchanged after testing.
The Cargo shim's helper location was also adapted. Dispatch, landing, supervisor
and worker CLI help entrypoints passed without adding the source directories to
PYTHONPATH; both included shell scripts passed syntax checks.

The published file set, privacy scan and repository checksum manifest are separate
archive checks. Passing those says nothing about model quality, secure sandboxing
or end-to-end autonomous success. External runtimes, account configuration, live
status/ledgers and remote-build profiles are omitted. Interactive/dashboard and
other unselected services are outside this test run.

[Conductor guide](README.md) · [Exact snapshot](SNAPSHOT.json) · [ELAI benchmark guide](../BENCHMARKS.md)
