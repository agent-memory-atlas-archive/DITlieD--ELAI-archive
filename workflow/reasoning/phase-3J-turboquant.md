P3J.4 TurboQuant on E2B -- decision doc

What: TurboQuantizer struct in crates/providers/src/turboquant.rs.
Data-oblivious KV cache quantizer: random rotation (QR) + nearest-centroid MSE + QJL residual.
Feature-gated behind cfg(feature = "turboquant"). Standalone evaluation, not wired into VllmProvider.

Key decisions:
1. PRNG: splitmix64 seeded from rotation_seed (u64). Box-Muller transform for Gaussian samples. No rand crate.
2. QR: modified Gram-Schmidt with single re-orthogonalization pass for numerical stability at d=256+.
3. Codebook b=1: {-sqrt(2/pi)/sqrt(d), +sqrt(2/pi)/sqrt(d)}. b=2: {-1.51/sqrt(d), -0.453/sqrt(d), +0.453/sqrt(d), +1.51/sqrt(d)}.
4. Codebook b=3,4: Lloyd-Max on N(0, 1/sqrt(d)). 100 iteration cap, delta < 1e-6 convergence.
5. QJL sign vector: 1xd {+1,-1} seeded from rotation_seed+1000. Only need scalar dot product S*r.
6. For b=1: pure 1-bit MSE quantization (2 centroids). No QJL residual (bit_width-1=0 MSE bits would be meaningless).
   For b>=2: (b-1) bits MSE + 1 bit QJL residual correction.
7. Pi stored as Vec<f32> row-major d*d. matmul is O(d^2) per vector.

Expected test outcomes: 10 new tests + 113 baseline = 123 green, 0 failures.
Test command: cargo test -p elai-providers --features elai-providers/turboquant

Unknowns:
- MGS precision at d=512+ may need double re-orthogonalization. Code uses 2-pass MGS (resolved).
- Lloyd-Max convergence for b=4 on N(0,1/sqrt(d)) -- 100 iters should suffice but untested at extremes.
- MSE bounds in tests use generous margins (0.50 for b=2, 0.20 for b=3) due to TurboQuant split (b-1 MSE + 1 QJL).

FEASIBLE/DEFER decision:
- FEASIBLE: TurboQuantizer compiles, 10 synthetic invariant tests pass (Pi orthogonality, codebook ordering,
  roundtrip MSE bounds, error paths, deterministic seeding). No new deps. Feature-gated, zero blast radius.
- DEFER to compose step: real-model KV capture from Gemma4 E2B (HF safetensors cached in WSL2),
  compression ratio measurement, perplexity/quality evaluation, wiring into unified DoLa+TurboQuant+steering pipeline.
  Paper reference: b=2 gives ~8x KV compression, b=3.5 matches full-precision on NIAH (0.997).

Audit results (1 cycle): 2 BLOCKs fixed (dequant_kv validation for bit_width mismatch and mse_indices length).
4 WARNs: EntropySource trait bypass accepted (standalone eval scope), lloyd_max return type fixed,
comment accuracy fixed, decision doc updated. 116/116 green, 1 ignored.
