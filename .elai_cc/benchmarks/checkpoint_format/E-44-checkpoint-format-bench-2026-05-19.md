# E-44 checkpoint format benchmark

Date: 2026-08-15
Remote host class: Depot amd64 builder
OS image: rust:1-trixie on Debian trixie
Build URL: https://depot.dev/orgs/nxsf3003j6/projects/qzcg6cxtrj/builds/qlf887jpk9

Exact provenance from the benchmark log:

```text
DEPOT_PROVENANCE path=build head=2085bfcf12a51beadf806db9ecb326bbf6a72f3a dirty=1 worktree=/home/user/teikoku/conductor/elai-worktrees/opuswalk-E-44-EXECUTE-1786747442-c5fb0f_50459b1a
```

The benchmark measured 50,000 serialize-and-deserialize round-trips of `AgentCheckpointData`.

## Measured values

| Format | Payload size | Serialize total | Serialize per iteration | Deserialize total | Deserialize per iteration |
| --- | ---: | ---: | ---: | ---: | ---: |
| msgpack | 2,642 bytes | 28,831,974 ns | 576 ns | 62,072,288 ns | 1,241 ns |
| postcard | 2,231 bytes | 17,211,826 ns | 344 ns | 41,407,149 ns | 828 ns |

## Deltas

The benchmark reports msgpack minus postcard:

- Payload size: +411 bytes, so msgpack is larger.
- Serialize time: +232 ns per iteration, so msgpack is slower.
- Deserialize time: +413 ns per iteration, so msgpack is slower.

## Recommendation

Use postcard for this fixture. Postcard is 411 bytes smaller, 232 ns faster to serialize, and 413 ns faster to deserialize for this fixture.
