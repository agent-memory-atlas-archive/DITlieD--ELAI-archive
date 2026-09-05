Algorithm: TurboQuant (MSE variant: TurboQuant_mse; inner-product variant: TurboQuant_prod)
ArXiv: 2504.19874
Paper: TurboQuant: Online Vector Quantization with Near-optimal Distortion Rate (ICLR 2026)
Authors: Amir Zandieh (Google Research), Majid Daliri (NYU), Majid Hadian (Google DeepMind), Vahab Mirrokni (Google Research)

CORE TECHNIQUE SUMMARY
Random rotation + Beta distribution scalar quantization.
Step 1: multiply input x (unit norm, d-dim) by random rotation matrix Pi (QR-decomposed random Gaussian matrix).
Step 2: rotated coordinates follow Beta distribution f_X(x) = Gamma(d/2) / (sqrt(pi) * Gamma((d-1)/2)) * (1 - x^2)^((d-3)/2) for x in [-1,1].
Step 3: in high dimensions this converges to N(0, 1/d) and coordinates become nearly independent.
Step 4: apply precomputed Lloyd-Max (Max-Lloyd) optimal scalar quantizer codebook per coordinate independently.
Step 5 (prod variant only): compute residual r = x - DeQuant_mse(Quant_mse(x)), apply QJL (sign(S * r)) on residual for unbiased inner product.

Inputs:
x | float32 tensor | shape (d,) | must have ||x||_2 = 1 (unit norm); non-unit inputs: store L2 norm separately, rescale after dequant
d | int | scalar | dimension of input vector; higher d -> better Beta-to-Gaussian approximation; tested at d=200, 1536, 3072
b | int | scalar | bit-width per coordinate; range [1, 4] for empirically tabulated values; b > 4 uses Panter-Dite formula
Pi | float32 matrix | shape (d, d) | random rotation (QR decomposition of iid Gaussian matrix); fixed per quantizer instance (data-oblivious)
codebook | float32 array | shape (2^b,) | precomputed Lloyd-Max centroids for Beta(d) distribution; stored per bit-width b
S | float32 matrix | shape (d, d) | iid N(0,1) matrix for QJL; only needed for TurboQuant_prod; fixed per instance

Outputs (TurboQuant_mse):
idx | int array | shape (d,) | b-bit centroid indices per coordinate; stored as b-bit integers
x_tilde | float32 tensor | shape (d,) | dequantized reconstruction = Pi^T * [c_{idx_j} for j in 1..d]

Outputs (TurboQuant_prod):
(idx, qjl, gamma) | tuple | idx: int array (d,); qjl: sign vector {-1,+1}^d; gamma: float scalar = ||r||_2
x_tilde_prod | float32 tensor | shape (d,) | = DeQuant_mse(idx) + (sqrt(pi/2)/d) * gamma * S^T * qjl

Hyperparameters:
b (bit-width) | no universal default; paper evaluates b=1,2,3,4 | range: b >= 1 integer | low sensitivity once >= 2; b=2 gives good recall
outlier_fraction | paper uses 32/128 = 25% at 3-bit for 2.5-bit effective | range: 0-0.5 | used for outlier channel split; optional
entropy_coding | disabled in paper | reduces avg bit-width by ~5% at b=4 | not recommended (minimal gain, added complexity)

Codebook values (precomputed, from paper, high-dim Gaussian approximation N(0,1/d)):
b=1: centroids = {+sqrt(2/pi)/sqrt(d), -sqrt(2/pi)/sqrt(d)}  [2 centroids]
b=2: centroids = {+-0.453/sqrt(d), +-1.51/sqrt(d)}  [4 centroids]
b=3,4: solve Lloyd-Max numerically; paper reports D_mse approx 0.03/d and 0.009/d respectively

MSE distortion (unit-norm input, TurboQuant_mse):
b=1: D_mse approx 0.36
b=2: D_mse approx 0.117
b=3: D_mse approx 0.030
b=4: D_mse approx 0.009
general: D_mse <= (sqrt(3)*pi/2) * (1/4^b)   [Theorem 1, eq. in Section 1.3]

Inner-product distortion (TurboQuant_prod, unit-norm x):
b=1: D_prod approx 1.57/d
b=2: D_prod approx 0.56/d
b=3: D_prod approx 0.18/d
b=4: D_prod approx 0.047/d
general: D_prod <= (sqrt(3)*pi^2 * ||y||^2) / d * (1/4^b)   [Theorem 2]
unbiasedness: E[<y, Q_prod^{-1}(Q_prod(x))>] = <y, x>   [Theorem 2, exact]

Information-theoretic lower bounds (Theorem 3):
D_mse(any Q) >= 1/4^b   [tight to TurboQuant by factor sqrt(3)*pi/2 approx 2.7, or 1.45 at b=1]
D_prod(any Q) >= (||y||^2 / d) * (1/4^b)

Critical invariants (testable assertions):
assert abs(np.linalg.norm(x) - 1.0) < 1e-5, "input must be unit norm before quantization"
assert Pi.shape == (d, d) and abs(np.linalg.norm(Pi @ Pi.T - np.eye(d))) < 1e-4, "Pi must be orthogonal rotation matrix"
assert len(codebook) == 2**b, "codebook size must equal 2^b"
assert np.all(codebook[:-1] <= codebook[1:]), "codebook centroids must be in ascending sorted order"
assert abs(np.dot(y, dequant_prod(quant_prod(x))) - np.dot(y, x)) < threshold, "prod variant must be unbiased in expectation"
# For TurboQuant_prod output tuple:
assert qjl.shape == (d,) and set(np.unique(qjl)) <= {-1, 1}, "QJL output must be sign vector"
assert gamma >= 0, "residual norm must be non-negative"

Complexity:
Quant_mse: O(d^2) for rotation (Pi * x), O(d * 2^b) for nearest centroid lookup | space O(d^2) for Pi + O(2^b) for codebook
Quant_prod: O(d^2) for rotation + O(d * 2^b) for MSE quant + O(d^2) for QJL (S * r) | total O(d^2) dominated by matrix multiply
DeQuant_mse: O(d) centroid lookup + O(d^2) for Pi^T multiply | O(d^2)
DeQuant_prod: O(d^2) for Pi^T + O(d^2) for S^T * qjl | O(d^2)
Indexing time vs PQ: near-zero (TurboQuant 0.0007-0.0021 sec vs PQ 37-494 sec at d=200-3072, Table 2)
Parallelism: fully vectorizable; no sequential dependency between coordinates after rotation; GPU-friendly

Failure modes:
Non-unit-norm input: distortion bounds only proven for ||x||=1; store and restore norms externally
Low-dimension (d small): Beta distribution does not approximate Gaussian; near-independence of coordinates fails; minimum safe d approx 64+
MSE quantizer used for inner product estimation: introduces bias (grows with average inner product value); use prod variant instead
Using TurboQuant_prod at b=1: only 0 bits for MSE stage (b-1=0), QJL carries all bits; verified to still be unbiased but higher variance
Entropy coding applied without care: can change codebook index distributions if not perfectly matched to actual Beta distribution for given d

Implementation pitfalls (do NOT do this):
Do NOT reuse Pi or S across different quantizer instances without resetting; they must be fixed per-instance (data-oblivious but instance-specific)
Do NOT skip the norm rescaling step for non-unit inputs; the codebook centroids are calibrated for ||x||=1
Do NOT approximate the rotation with a random diagonal sign flip (Hadamard-based); paper uses full QR rotation for theoretical guarantees
Do NOT use the MSE variant for nearest-neighbor search inner product queries; it is biased and bias grows with inner product magnitude
Do NOT apply Lloyd-Max codebook for one dimension to another without recomputing; codebook depends on dimension d through the Beta distribution
Do NOT use integer rounding instead of nearest-centroid lookup; the centroids are not uniformly spaced
Do NOT apply entropy coding in production if streaming latency matters; paper explicitly omits it for simplicity and speed

Experimental results (nearest-neighbor search recall, from Section 4.4 / Figure 5):
Datasets: GloVe d=200, OpenAI3-large d=1536, OpenAI3-large d=3072; 100k training, 1k-10k queries
TurboQuant consistently outperforms PQ and RabitQ on recall@k across all dimensions and bit-widths
Indexing time: TurboQuant ~0.001 sec vs PQ 37-494 sec vs RabitQ 597-3957 sec (Table 2)
KV cache: 3.5-bit TurboQuant matches full-precision (score 0.997) on Needle-In-A-Haystack; 2.5-bit gives marginal degradation
LongBench: 3.5-bit TurboQuant average 50.06 vs full-cache 50.06 (Llama-3.1-8B); 2.5-bit gives 49.44 (Table 1)
Outlier handling: split channels into outlier set (higher b) and regular set (lower b); effective b = weighted average

Streaming / online suitability:
Algorithm is data-oblivious: Pi and S are random matrices independent of data; no training or calibration phase needed
Codebook is precomputed once offline for each (d, b) pair and reused for all vectors
Quantization is per-vector and independent; suitable for streaming KV cache token-by-token
Dequantization is also per-vector (no inter-vector dependencies)

API sketch for streaming vector quantization (derived from Algorithm 1 and 2):
Setup: Pi = qr(randn(d,d))[0]; codebook = solve_lloyd_max(beta_pdf(d), b); S = randn(d,d)  [prod only]
Quant_mse(x): y = Pi @ x; idx = argmin_k |y_j - c_k| for each j; return idx
DeQuant_mse(idx): y_tilde = codebook[idx]; return Pi.T @ y_tilde
Quant_prod(x): idx = Quant_mse(x); r = x - DeQuant_mse(idx); qjl = sign(S @ r); gamma = norm(r); return (idx, qjl, gamma)
DeQuant_prod(idx, qjl, gamma): x_mse = DeQuant_mse(idx); x_qjl = (sqrt(pi/2)/d) * gamma * S.T @ qjl; return x_mse + x_qjl
InnerProduct_prod(y, idx, qjl, gamma): return dot(y, DeQuant_prod(idx, qjl, gamma))  [unbiased estimator]
