# B-0 minimal discriminator benchmark

Run: `frontier-discriminator-b0-probe-smoke-2026-07-12`
K: 20 paired tasks
Model baseline: `glm-5-2`
Seed: 20260712

## Gate verdict

**REFUTED** — WIN iff paired success/normalized-cost delta vs direct >0, success-rate delta >=0, and bootstrap CI95 lower bound >0

## Arms

| arm | n | success | success/cost | mean probe cost | model tokens | failures |
|---|---:|---:|---:|---:|---:|---:|
| probe_utility | 20 | 1.000 | 0.3375 | 1.650 | 0 | 0 |

## Paired deltas

- probe_utility_vs_direct: 0.0000, CI95 [0.0000, 0.0000], success-rate delta 0.000
- probe_utility_vs_best_of_n: 0.0000, CI95 [0.0000, 0.0000], success-rate delta 0.000

## Integrity

- task corpus sha256: `bd05608b61fe8cb1d72f707ebb1c315517585d348990f4f9d6c7a0c0230e0928`
- discriminator sha256: `80ca153b05061c0480f0be3f812c24710b301b125e71013bf6c3e44a9b2d9145`
- harness sha256: `e77b74d7746cd0b3bd58009858b817dc91c82e910bc18a04035e35974c16d472`
- Truth labels were withheld from model prompts.
- Probe execution used a closed read-only vocabulary; arbitrary shell was not available.
- Normalized decision cost was charged equally to all arms; observed model tokens are reported separately.
