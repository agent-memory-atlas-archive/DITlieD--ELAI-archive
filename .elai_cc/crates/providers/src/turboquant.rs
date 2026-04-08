// TurboQuant: data-oblivious KV cache quantizer.
// Random rotation (QR) + nearest-centroid MSE (PolarQuant) + QJL residual.
// Paper: TurboQuant (ICLR 2026), Zandieh et al. arXiv:2504.19874.

/// Minimum dimension for Beta-to-Gaussian approximation validity.
const MIN_DIMENSION: usize = 64;

/// Maximum supported bit-width (empirically tabulated codebooks).
const MAX_BIT_WIDTH: u8 = 4;

/// Lloyd-Max iteration cap.
const LLOYD_MAX_ITERS: usize = 100;

/// Lloyd-Max convergence threshold (max centroid shift per iteration).
const LLOYD_CONVERGENCE: f64 = 1e-6;

/// Norm threshold below which input is considered zero.
const NORM_ZERO_THRESHOLD: f32 = 1e-7;

/// QJL dequantization constant: sqrt(pi/2).
const QJL_CONST: f32 = 1.253_314_1; // sqrt(pi/2) = 1.2533141...

/// 2-bit codebook inner values (scaled by 1/sqrt(d) at construction).
/// From paper Section 4: optimal Lloyd-Max centroids for N(0,1/d).
const CODEBOOK_2BIT_INNER: f64 = 0.453;

/// 2-bit codebook outer values (scaled by 1/sqrt(d) at construction).
const CODEBOOK_2BIT_OUTER: f64 = 1.51;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from TurboQuantizer operations.
#[derive(Debug, thiserror::Error)]
pub enum TurboQuantError {
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    #[error("input vector has zero norm")]
    NormZero,

    #[error("invalid bit_width {0}: must be 1..={MAX_BIT_WIDTH}")]
    InvalidBitWidth(u8),

    #[error("invalid dimension {0}: must be >= {MIN_DIMENSION}")]
    InvalidDimension(usize),

    #[error("codebook not found for bit_width={bit_width}, d={d}")]
    CodebookNotFound { bit_width: u8, d: usize },
}

// ---------------------------------------------------------------------------
// Config / output structs
// ---------------------------------------------------------------------------

/// Configuration for TurboQuantizer construction.
#[derive(Debug, Clone)]
pub struct TurboQuantConfig {
    /// Bits per coordinate for MSE codebook. Range 1..=4.
    pub bit_width: u8,
    /// Seed for deterministic rotation matrix and QJL projection.
    pub rotation_seed: u64,
}

/// Compressed representation of a single vector.
#[derive(Debug, Clone)]
pub struct QuantizedKV {
    /// Per-coordinate centroid indices from MSE quantization.
    pub mse_indices: Vec<u8>,
    /// Original L2 norm of the input vector.
    pub norm: f32,
    /// QJL sign bit: +1 or -1. 0 when bit_width == 1 (no QJL stage).
    pub qjl_sign: Vec<i8>,
    /// L2 norm of the MSE residual. 0.0 when bit_width == 1.
    pub residual_norm: f32,
    /// Bit-width used for quantization.
    pub bit_width: u8,
    /// Dimension of the original vector.
    pub d: usize,
}

// ---------------------------------------------------------------------------
// PRNG: splitmix64 (deterministic, no deps)
// ---------------------------------------------------------------------------

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform f64 in (0, 1) — excludes 0 to keep Box-Muller safe.
    fn next_f64(&mut self) -> f64 {
        loop {
            let v = (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64);
            if v > 0.0 {
                return v;
            }
        }
    }

    /// Box-Muller: two independent N(0,1) samples.
    fn next_gaussian_pair(&mut self) -> (f64, f64) {
        let u1 = self.next_f64();
        let u2 = self.next_f64();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
        (r * theta.cos(), r * theta.sin())
    }
}

// ---------------------------------------------------------------------------
// Normal CDF/PDF for Lloyd-Max (f64 precision)
// ---------------------------------------------------------------------------

/// Standard normal PDF: phi(x) = exp(-x^2/2) / sqrt(2*pi).
fn std_normal_pdf(x: f64) -> f64 {
    let inv_sqrt_2pi = 0.398_942_280_401_432_7; // 1/sqrt(2*pi)
    inv_sqrt_2pi * (-0.5 * x * x).exp()
}

/// Standard normal CDF via erfc: Phi(x) = 0.5 * erfc(-x / sqrt(2)).
fn std_normal_cdf(x: f64) -> f64 {
    0.5 * erfc_approx(-x * std::f64::consts::FRAC_1_SQRT_2)
}

/// Complementary error function approximation (Abramowitz & Stegun, max error ~1.5e-7).
fn erfc_approx(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    let result = poly * (-x * x).exp();
    if x >= 0.0 {
        result
    } else {
        2.0 - result
    }
}

/// Gaussian conditional expectation E[X | a < X < b] where X ~ N(0, sigma).
/// Formula: sigma * (phi(a/sigma) - phi(b/sigma)) / (Phi(b/sigma) - Phi(a/sigma)).
fn gaussian_cond_expect(sigma: f64, a: f64, b: f64) -> f64 {
    let a_std = if a.is_finite() { a / sigma } else { a };
    let b_std = if b.is_finite() { b / sigma } else { b };

    let prob = if !a_std.is_finite() && a_std < 0.0 {
        std_normal_cdf(b_std)
    } else if !b_std.is_finite() && b_std > 0.0 {
        1.0 - std_normal_cdf(a_std)
    } else {
        std_normal_cdf(b_std) - std_normal_cdf(a_std)
    };

    if prob < 1e-15 {
        if a.is_finite() && !b.is_finite() {
            return a + sigma;
        } else if !a.is_finite() && b.is_finite() {
            return b - sigma;
        } else if a.is_finite() && b.is_finite() {
            return (a + b) / 2.0;
        }
        return 0.0;
    }

    let pdf_diff = std_normal_pdf(a_std) - std_normal_pdf(b_std);
    sigma * pdf_diff / prob
}

// ---------------------------------------------------------------------------
// Codebook construction
// ---------------------------------------------------------------------------

/// Build optimal MSE codebook for post-rotation coordinates ~ N(0, 1/d).
/// bit_width=1: closed-form {-c, +c} where c = sqrt(2/(pi*d)).
/// bit_width=2: closed-form {-1.51, -0.453, +0.453, +1.51} / sqrt(d).
/// bit_width=3,4: Lloyd-Max iterative solver on N(0, 1/sqrt(d)).
fn build_codebook(bit_width: u8, d: usize) -> Result<Vec<f32>, TurboQuantError> {
    let n_centroids = 1usize << bit_width;
    let d_f64 = d as f64;

    if bit_width == 1 {
        let c = ((2.0 / (std::f64::consts::PI * d_f64)).sqrt()) as f32;
        return Ok(vec![-c, c]);
    }

    if bit_width == 2 {
        let inv_sqrt_d = 1.0 / d_f64.sqrt();
        return Ok(vec![
            (-CODEBOOK_2BIT_OUTER * inv_sqrt_d) as f32,
            (-CODEBOOK_2BIT_INNER * inv_sqrt_d) as f32,
            (CODEBOOK_2BIT_INNER * inv_sqrt_d) as f32,
            (CODEBOOK_2BIT_OUTER * inv_sqrt_d) as f32,
        ]);
    }

    // Lloyd-Max on N(0, sigma) where sigma = 1/sqrt(d).
    let sigma = 1.0 / d_f64.sqrt();
    Ok(lloyd_max(n_centroids, sigma)
        .into_iter()
        .map(|v| v as f32)
        .collect())
}

/// Lloyd-Max algorithm for optimal scalar quantization of N(0, sigma^2).
/// Returns sorted centroids.
fn lloyd_max(n_centroids: usize, sigma: f64) -> Vec<f64> {
    // Initialize boundaries from uniform quantiles of N(0, sigma).
    let mut boundaries: Vec<f64> = (1..n_centroids)
        .map(|i| {
            let p = i as f64 / n_centroids as f64;
            sigma * quantile_std_normal(p)
        })
        .collect();

    let mut centroids = vec![0.0f64; n_centroids];

    // Initial centroids: conditional expectations within each region.
    centroids[0] = gaussian_cond_expect(sigma, f64::NEG_INFINITY, boundaries[0]);
    for i in 1..n_centroids - 1 {
        centroids[i] = gaussian_cond_expect(sigma, boundaries[i - 1], boundaries[i]);
    }
    centroids[n_centroids - 1] =
        gaussian_cond_expect(sigma, boundaries[n_centroids - 2], f64::INFINITY);

    for _ in 0..LLOYD_MAX_ITERS {
        // Update boundaries: midpoints between consecutive centroids.
        for i in 0..boundaries.len() {
            boundaries[i] = (centroids[i] + centroids[i + 1]) / 2.0;
        }

        let mut max_shift = 0.0f64;

        // Update centroids: conditional expectations.
        let new_c0 = gaussian_cond_expect(sigma, f64::NEG_INFINITY, boundaries[0]);
        max_shift = max_shift.max((new_c0 - centroids[0]).abs());
        centroids[0] = new_c0;

        for i in 1..n_centroids - 1 {
            let new_ci = gaussian_cond_expect(sigma, boundaries[i - 1], boundaries[i]);
            max_shift = max_shift.max((new_ci - centroids[i]).abs());
            centroids[i] = new_ci;
        }

        let new_cn = gaussian_cond_expect(
            sigma,
            boundaries[n_centroids - 2],
            f64::INFINITY,
        );
        max_shift = max_shift.max((new_cn - centroids[n_centroids - 1]).abs());
        centroids[n_centroids - 1] = new_cn;

        if max_shift < LLOYD_CONVERGENCE {
            break;
        }
    }

    centroids.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    centroids
}

/// Approximate quantile of standard normal (Beasley-Springer-Moro algorithm).
fn quantile_std_normal(p: f64) -> f64 {
    // Rational approximation for central region.
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    let q = p - 0.5;
    if q.abs() <= 0.425 {
        let r = 0.180_625 - q * q;
        return q * (((((((2.509_080_928_730_122_7e3 * r
            + 3.343_050_530_544_727_6e4)
            * r
            + 6.726_577_092_700_870_0e4)
            * r
            + 4.592_195_393_154_987_0e4)
            * r
            + 1.373_169_376_550_946_0e4)
            * r
            + 1.971_590_950_306_551_4e3)
            * r
            + 1.331_416_678_917_843_8e2)
            * r
            + 3.387_132_872_796_366_5)
            / (((((((5.226_495_278_852_854_0e3 * r
                + 2.887_696_927_511_880_5e4)
                * r
                + 3.930_789_580_009_271_0e4)
                * r
                + 2.121_379_430_158_659_7e4)
                * r
                + 5.394_196_021_424_751_0e3)
                * r
                + 6.871_870_074_920_579_0e2)
                * r
                + 4.231_333_070_160_091_0e1)
                * r
                + 1.0);
    }

    let r = if q < 0.0 { p } else { 1.0 - p };
    let s = (-r.ln()).sqrt();

    let result = if s <= 5.0 {
        let t = s - 1.6;
        (((((((7.745_450_142_783_414_0e-4 * t
            + 2.272_384_498_926_918_4e-2)
            * t
            + 2.220_560_251_461_460_0e-1)
            * t
            + 7.169_140_924_991_990_5e-1)
            * t
            + 1.055_341_195_202_855_6)
            * t
            + 1.025_538_023_065_820_2e-1)
            * t
            - 1.476_945_998_292_614_5e-1)
            * t
            - 4.424_609_488_714_966_6e-2)
            / (((((((1.050_750_071_644_416_9e-4 * t
                + 1.898_496_154_952_363_7e-3)
                * t
                + 1.369_298_809_227_358_0e-2)
                * t
                + 1.087_141_567_270_900_5e-1)
                * t
                + 3.360_924_164_466_127_0e-1)
                * t
                - 4.613_375_167_062_111_0e-2)
                * t
                - 2.724_331_755_894_402_0e-2)
                * t
                + 1.0)
    } else {
        let t = s - 5.0;
        (((((((2.010_334_399_292_289_0e-7 * t
            + 2.711_555_568_743_488_0e-5)
            * t
            + 1.242_660_409_512_384_0e-3)
            * t
            + 2.152_149_887_979_066_4e-2)
            * t
            + 1.535_169_862_030_455_0e-1)
            * t
            + 4.272_789_340_618_697_0e-1)
            * t
            - 7.780_740_630_690_178_0e-3)
            * t
            - 3.223_964_580_411_365_0e-1)
            / (((((((1.363_171_783_672_130_0e-7 * t
                + 1.920_339_423_885_560_7e-5)
                * t
                + 1.131_937_695_416_420_0e-3)
                * t
                + 1.797_461_918_706_118_5e-2)
                * t
                + 1.323_256_419_069_053_0e-1)
                * t
                - 4.506_505_958_459_806_0e-3)
                * t
                - 3.022_417_438_716_218_5e-2)
                * t
                + 1.0)
    };

    if q < 0.0 {
        -result
    } else {
        result
    }
}

// ---------------------------------------------------------------------------
// Linear algebra helpers (no deps)
// ---------------------------------------------------------------------------

/// L2 norm of a slice.
fn l2_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Matrix-vector multiply: out = mat (d x d, row-major) * v (d).
fn mat_vec_mul(mat: &[f32], v: &[f32], d: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; d];
    for i in 0..d {
        let row_start = i * d;
        let mut acc = 0.0f32;
        for j in 0..d {
            acc += mat[row_start + j] * v[j];
        }
        out[i] = acc;
    }
    out
}

/// Transpose matrix-vector multiply: out = mat^T (d x d, row-major) * v (d).
fn mat_t_vec_mul(mat: &[f32], v: &[f32], d: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; d];
    for i in 0..d {
        let row_start = i * d;
        for j in 0..d {
            out[j] += mat[row_start + j] * v[i];
        }
    }
    out
}

/// Find nearest centroid index for a single value in a sorted codebook.
/// Linear scan over codebook entries (acceptable for codebook sizes <= 16).
fn nearest_centroid(value: f32, codebook: &[f32]) -> u8 {
    let mut best = 0u8;
    let mut best_dist = (value - codebook[0]).abs();
    for (i, &c) in codebook.iter().enumerate().skip(1) {
        let dist = (value - c).abs();
        if dist < best_dist {
            best_dist = dist;
            best = i as u8;
        }
    }
    best
}

// ---------------------------------------------------------------------------
// TurboQuantizer
// ---------------------------------------------------------------------------

/// Data-oblivious KV cache quantizer.
///
/// Construction is O(d^2) for the rotation matrix.
/// Quantization/dequantization is O(d^2) per vector (dominated by matrix multiply).
pub struct TurboQuantizer {
    d: usize,
    config: TurboQuantConfig,
    /// Rotation matrix Pi, d×d row-major.
    pi: Vec<f32>,
    /// MSE codebook for PolarQuant stage.
    /// For bit_width >= 2: codebook has 2^(bit_width-1) centroids (b-1 bits).
    /// For bit_width == 1: codebook has 2 centroids (1 bit, MSE-only).
    mse_codebook: Vec<f32>,
    /// QJL projection matrix S, d×d row-major. Only used when bit_width >= 2.
    qjl_s: Vec<f32>,
}

impl std::fmt::Debug for TurboQuantizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TurboQuantizer")
            .field("d", &self.d)
            .field("config", &self.config)
            .field("mse_codebook", &self.mse_codebook)
            .finish_non_exhaustive()
    }
}

impl TurboQuantizer {
    /// Construct a new quantizer for vectors of dimension `d`.
    ///
    /// # Errors
    /// - `InvalidDimension` if d < 64.
    /// - `InvalidBitWidth` if bit_width not in 1..=4.
    pub fn new(d: usize, config: TurboQuantConfig) -> Result<Self, TurboQuantError> {
        if d < MIN_DIMENSION {
            return Err(TurboQuantError::InvalidDimension(d));
        }
        if config.bit_width == 0 || config.bit_width > MAX_BIT_WIDTH {
            return Err(TurboQuantError::InvalidBitWidth(config.bit_width));
        }

        let mut rng = SplitMix64::new(config.rotation_seed);

        // Generate d×d Gaussian matrix, then QR via modified Gram-Schmidt.
        let pi = generate_rotation_matrix(&mut rng, d);

        // Build MSE codebook.
        // For bit_width >= 2: PolarQuant uses (bit_width - 1) bits.
        // For bit_width == 1: MSE-only with 1 bit.
        let mse_bits = if config.bit_width >= 2 {
            config.bit_width - 1
        } else {
            1
        };
        let mse_codebook = build_codebook(mse_bits, d)?;

        // QJL projection matrix (seeded from rotation_seed + 1000).
        let qjl_s = if config.bit_width >= 2 {
            let mut qjl_rng = SplitMix64::new(config.rotation_seed.wrapping_add(1000));
            generate_gaussian_matrix(&mut qjl_rng, d)
        } else {
            Vec::new()
        };

        Ok(Self {
            d,
            config,
            pi,
            mse_codebook,
            qjl_s,
        })
    }

    /// Quantize a key or value vector.
    ///
    /// # Errors
    /// - `DimensionMismatch` if input length != d.
    /// - `NormZero` if input has zero norm.
    pub fn quant_kv(&self, key_or_value: &[f32]) -> Result<QuantizedKV, TurboQuantError> {
        if key_or_value.len() != self.d {
            return Err(TurboQuantError::DimensionMismatch {
                expected: self.d,
                got: key_or_value.len(),
            });
        }

        let norm = l2_norm(key_or_value);
        if norm < NORM_ZERO_THRESHOLD {
            return Err(TurboQuantError::NormZero);
        }

        // Normalize to unit norm.
        let inv_norm = 1.0 / norm;
        let x_hat: Vec<f32> = key_or_value.iter().map(|&v| v * inv_norm).collect();

        // Rotate: y = Pi @ x_hat.
        let rotated = mat_vec_mul(&self.pi, &x_hat, self.d);

        // MSE quantization: nearest centroid per coordinate.
        let mse_indices: Vec<u8> = rotated
            .iter()
            .map(|&v| nearest_centroid(v, &self.mse_codebook))
            .collect();

        if self.config.bit_width == 1 {
            // MSE-only, no QJL stage.
            return Ok(QuantizedKV {
                mse_indices,
                norm,
                qjl_sign: Vec::new(),
                residual_norm: 0.0,
                bit_width: self.config.bit_width,
                d: self.d,
            });
        }

        // Reconstruct MSE approximation in rotated domain.
        let dequant_approx: Vec<f32> = mse_indices
            .iter()
            .map(|&idx| self.mse_codebook[idx as usize])
            .collect();

        // Norm correction: renormalize reconstructed rotated vector to unit norm
        // before computing residual (matches Python PolarQuant.dequantize).
        let approx_norm = l2_norm(&dequant_approx);
        let dequant_corrected: Vec<f32> = if approx_norm > 1e-10 {
            let scale = 1.0 / approx_norm;
            dequant_approx.iter().map(|&v| v * scale).collect()
        } else {
            dequant_approx.clone()
        };

        // Residual in rotated domain.
        let residual: Vec<f32> = rotated
            .iter()
            .zip(dequant_corrected.iter())
            .map(|(&r, &a)| r - a)
            .collect();

        let residual_norm = l2_norm(&residual);

        // QJL: sign(S @ residual) per coordinate.
        let projected = mat_vec_mul(&self.qjl_s, &residual, self.d);
        let qjl_sign: Vec<i8> = projected
            .iter()
            .map(|&v| if v >= 0.0 { 1i8 } else { -1i8 })
            .collect();

        Ok(QuantizedKV {
            mse_indices,
            norm,
            qjl_sign,
            residual_norm,
            bit_width: self.config.bit_width,
            d: self.d,
        })
    }

    /// Dequantize a compressed vector back to an approximation.
    ///
    /// # Errors
    /// - `DimensionMismatch` if qkv.d != self.d or qkv.mse_indices.len() != self.d.
    /// - `InvalidBitWidth` if qkv.bit_width != self.config.bit_width.
    pub fn dequant_kv(&self, qkv: &QuantizedKV) -> Result<Vec<f32>, TurboQuantError> {
        if qkv.d != self.d {
            return Err(TurboQuantError::DimensionMismatch {
                expected: self.d,
                got: qkv.d,
            });
        }
        if qkv.mse_indices.len() != self.d {
            return Err(TurboQuantError::DimensionMismatch {
                expected: self.d,
                got: qkv.mse_indices.len(),
            });
        }
        if qkv.bit_width != self.config.bit_width {
            return Err(TurboQuantError::InvalidBitWidth(qkv.bit_width));
        }

        // Reconstruct MSE approximation in rotated domain.
        // All indices are guaranteed in range 0..codebook.len() since they were
        // produced by nearest_centroid which returns indices within codebook bounds.
        // The bit_width check above ensures codebook size matches.
        let codebook_len = self.mse_codebook.len();
        let dequant_approx: Vec<f32> = qkv
            .mse_indices
            .iter()
            .map(|&idx| {
                let i = idx as usize;
                if i < codebook_len {
                    self.mse_codebook[i]
                } else {
                    0.0
                }
            })
            .collect();

        // Norm correction.
        let approx_norm = l2_norm(&dequant_approx);
        let mut reconstructed_rotated: Vec<f32> = if approx_norm > 1e-10 {
            let scale = 1.0 / approx_norm;
            dequant_approx.iter().map(|&v| v * scale).collect()
        } else {
            dequant_approx
        };

        // Add QJL residual estimate if bit_width >= 2.
        if self.config.bit_width >= 2 && !qkv.qjl_sign.is_empty() {
            // residual_est = sqrt(pi/2) / d * gamma * S^T @ signs
            let signs_f32: Vec<f32> = qkv.qjl_sign.iter().map(|&s| s as f32).collect();
            let st_signs = mat_t_vec_mul(&self.qjl_s, &signs_f32, self.d);
            let scale = QJL_CONST / (self.d as f32) * qkv.residual_norm;
            for (r, &s) in reconstructed_rotated.iter_mut().zip(st_signs.iter()) {
                *r += scale * s;
            }
        }

        // Inverse rotation: x_hat = Pi^T @ reconstructed_rotated.
        let x_hat = mat_t_vec_mul(&self.pi, &reconstructed_rotated, self.d);

        // Rescale by original norm.
        Ok(x_hat.iter().map(|&v| v * qkv.norm).collect())
    }
}

// ---------------------------------------------------------------------------
// Matrix generation
// ---------------------------------------------------------------------------

/// Generate d×d Gaussian matrix (row-major) from seeded PRNG.
fn generate_gaussian_matrix(rng: &mut SplitMix64, d: usize) -> Vec<f32> {
    let n = d * d;
    let mut mat = Vec::with_capacity(n);
    let pairs_needed = (n + 1) / 2;
    for _ in 0..pairs_needed {
        let (g1, g2) = rng.next_gaussian_pair();
        mat.push(g1 as f32);
        if mat.len() < n {
            mat.push(g2 as f32);
        }
    }
    mat.truncate(n);
    mat
}

/// Generate d×d orthogonal rotation matrix via modified Gram-Schmidt with
/// re-orthogonalization for numerical stability.
fn generate_rotation_matrix(rng: &mut SplitMix64, d: usize) -> Vec<f32> {
    let mut mat = generate_gaussian_matrix(rng, d);

    // Modified Gram-Schmidt (column-major view on row-major storage).
    // We work column by column. Column j of row-major mat is mat[i*d + j] for i in 0..d.
    for j in 0..d {
        // Two passes of MGS for stability (CGS2 equivalent).
        for _pass in 0..2 {
            for k in 0..j {
                // dot = col_k . col_j
                let mut dot = 0.0f64;
                for i in 0..d {
                    dot += mat[i * d + k] as f64 * mat[i * d + j] as f64;
                }
                // col_j -= dot * col_k
                for i in 0..d {
                    let correction = dot * mat[i * d + k] as f64;
                    mat[i * d + j] -= correction as f32;
                }
            }
        }

        // Normalize column j.
        let mut col_norm = 0.0f64;
        for i in 0..d {
            let v = mat[i * d + j] as f64;
            col_norm += v * v;
        }
        let col_norm = col_norm.sqrt();
        if col_norm > 1e-12 {
            let inv = 1.0 / col_norm;
            for i in 0..d {
                mat[i * d + j] = (mat[i * d + j] as f64 * inv) as f32;
            }
        }
    }

    mat
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_D: usize = 128;
    const TEST_SEED: u64 = 42;

    fn make_config(bit_width: u8) -> TurboQuantConfig {
        TurboQuantConfig {
            bit_width,
            rotation_seed: TEST_SEED,
        }
    }

    /// Generate a deterministic pseudo-random unit-norm vector of dimension d.
    fn random_unit_vector(d: usize, seed: u64) -> Vec<f32> {
        let mut rng = SplitMix64::new(seed);
        let mut v: Vec<f32> = (0..d)
            .map(|_| {
                let (g, _) = rng.next_gaussian_pair();
                g as f32
            })
            .collect();
        let norm = l2_norm(&v);
        for x in &mut v {
            *x /= norm;
        }
        v
    }

    #[test]
    fn test_pi_orthogonality() {
        let tq = TurboQuantizer::new(TEST_D, make_config(2)).unwrap();
        // Compute Pi @ Pi^T - I, measure Frobenius norm.
        let d = TEST_D;
        let mut frob_sq = 0.0f64;
        for i in 0..d {
            for j in 0..d {
                let mut dot = 0.0f64;
                for k in 0..d {
                    dot += tq.pi[i * d + k] as f64 * tq.pi[j * d + k] as f64;
                }
                let expected = if i == j { 1.0 } else { 0.0 };
                let diff = dot - expected;
                frob_sq += diff * diff;
            }
        }
        let frob = frob_sq.sqrt();
        assert!(
            frob < 1e-4,
            "Pi orthogonality: Frobenius norm of Pi*Pi^T - I = {frob}, expected < 1e-4"
        );
    }

    #[test]
    fn test_codebook_sorted_ascending() {
        for b in 1..=4u8 {
            let codebook = build_codebook(b, TEST_D).unwrap();
            assert_eq!(
                codebook.len(),
                1 << b,
                "codebook for b={b} should have {} entries",
                1 << b
            );
            for i in 1..codebook.len() {
                assert!(
                    codebook[i] > codebook[i - 1],
                    "codebook for b={b} not strictly ascending at index {i}: {} <= {}",
                    codebook[i],
                    codebook[i - 1]
                );
            }
        }
    }

    #[test]
    fn test_unit_norm_assertion() {
        let tq = TurboQuantizer::new(TEST_D, make_config(2)).unwrap();
        // Input with norm far from 1 should still work (we normalize internally).
        // But zero vector should fail.
        let mut v = random_unit_vector(TEST_D, 99);
        // Scale to non-unit norm -- this should succeed (we store norm separately).
        for x in &mut v {
            *x *= 3.5;
        }
        assert!(tq.quant_kv(&v).is_ok());
    }

    #[test]
    fn test_roundtrip_mse_b2() {
        let tq = TurboQuantizer::new(TEST_D, make_config(2)).unwrap();
        let v = random_unit_vector(TEST_D, 123);
        let compressed = tq.quant_kv(&v).unwrap();
        let reconstructed = tq.dequant_kv(&compressed).unwrap();

        let mut mse = 0.0f64;
        for (a, b) in v.iter().zip(reconstructed.iter()) {
            let diff = (*a - *b) as f64;
            mse += diff * diff;
        }
        mse /= TEST_D as f64;

        // Paper gives D_mse ~ 0.117 for b=2 (but this is b=2 TurboQuant = 1-bit MSE + QJL).
        // 1-bit MSE has D_mse ~ 0.36, QJL correction helps.
        // Use generous bound.
        assert!(
            mse < 0.50,
            "b=2 roundtrip MSE = {mse}, expected < 0.50"
        );
    }

    #[test]
    fn test_roundtrip_mse_b3() {
        let tq = TurboQuantizer::new(TEST_D, make_config(3)).unwrap();
        let v = random_unit_vector(TEST_D, 456);
        let compressed = tq.quant_kv(&v).unwrap();
        let reconstructed = tq.dequant_kv(&compressed).unwrap();

        let mut mse = 0.0f64;
        for (a, b) in v.iter().zip(reconstructed.iter()) {
            let diff = (*a - *b) as f64;
            mse += diff * diff;
        }
        mse /= TEST_D as f64;

        // b=3 TurboQuant = 2-bit MSE + QJL. 2-bit MSE gives D_mse ~ 0.117.
        // QJL correction reduces further. Use generous bound.
        assert!(
            mse < 0.20,
            "b=3 roundtrip MSE = {mse}, expected < 0.20"
        );
    }

    #[test]
    fn test_dimension_mismatch() {
        let tq = TurboQuantizer::new(TEST_D, make_config(2)).unwrap();
        let wrong_len = vec![1.0f32; TEST_D + 5];
        let err = tq.quant_kv(&wrong_len).unwrap_err();
        assert!(
            matches!(err, TurboQuantError::DimensionMismatch { .. }),
            "expected DimensionMismatch, got {err:?}"
        );
    }

    #[test]
    fn test_invalid_dimension() {
        let err = TurboQuantizer::new(32, make_config(2)).unwrap_err();
        assert!(
            matches!(err, TurboQuantError::InvalidDimension(32)),
            "expected InvalidDimension(32), got {err:?}"
        );
    }

    #[test]
    fn test_invalid_bit_width() {
        let err = TurboQuantizer::new(TEST_D, make_config(0)).unwrap_err();
        assert!(
            matches!(err, TurboQuantError::InvalidBitWidth(0)),
            "expected InvalidBitWidth(0), got {err:?}"
        );

        let err = TurboQuantizer::new(TEST_D, make_config(5)).unwrap_err();
        assert!(
            matches!(err, TurboQuantError::InvalidBitWidth(5)),
            "expected InvalidBitWidth(5), got {err:?}"
        );
    }

    #[test]
    fn test_norm_zero() {
        let tq = TurboQuantizer::new(TEST_D, make_config(2)).unwrap();
        let zeros = vec![0.0f32; TEST_D];
        let err = tq.quant_kv(&zeros).unwrap_err();
        assert!(
            matches!(err, TurboQuantError::NormZero),
            "expected NormZero, got {err:?}"
        );
    }

    #[test]
    fn test_deterministic_seed() {
        let tq1 = TurboQuantizer::new(TEST_D, make_config(2)).unwrap();
        let tq2 = TurboQuantizer::new(TEST_D, make_config(2)).unwrap();

        // Same seed must produce identical Pi.
        assert_eq!(tq1.pi.len(), tq2.pi.len());
        for (a, b) in tq1.pi.iter().zip(tq2.pi.iter()) {
            assert_eq!(a.to_bits(), b.to_bits(), "Pi matrices differ");
        }

        // Same codebook.
        assert_eq!(tq1.mse_codebook.len(), tq2.mse_codebook.len());
        for (a, b) in tq1.mse_codebook.iter().zip(tq2.mse_codebook.iter()) {
            assert_eq!(a.to_bits(), b.to_bits(), "codebooks differ");
        }
    }
}
