Algorithm: ONC (Optimal Number of Clusters) for Effective Trial Count Ne
Paper: Detection of false investment strategies using unsupervised learning methods
Authors: Marcos Lopez de Prado, Michael J. Lewis
Published: Quantitative Finance 19(9):1555-1565, 2019
DOI: 10.1080/14697688.2019.1622311
SSRN preprint: 3167017
Source: https://www.tandfonline.com/doi/abs/10.1080/14697688.2019.1622311
Reference implementation: hudson-and-thames/mlfinlab clustering/onc.py
Reliability: HIGH (peer-reviewed journal); implementation notes from mlfinlab 0.12.0 readthedocs mirror
  (random-docs.readthedocs.io/en/latest/implementations/onc.html; confirmed formula alignment)
Note: known bug in book ML for Asset Managers Chapter 4 ONC implementation; use mlfinlab reference, not book code

Purpose (H-17 grounding):
ONC takes the N x N correlation matrix of N backtest strategies and outputs K* = number of clusters.
K* is the effective number of uncorrelated trial groups Ne. Ne <= N always.
Ne replaces N in DSR/SR_0 to avoid inflating selection-bias penalty when strategies are correlated.
When strategies are all independent: K* = N and Ne = N.
When strategies are highly correlated: K* << N, meaning only K* truly independent bets were made.

Inputs:
corr_mat     | pd.DataFrame | shape (N, N) | correlation matrix of N strategy returns; diagonal must be 1.0
             |              | symmetric, values in [-1, 1]; mandatory
repeat       | int          | scalar | number of K-means random restarts per cluster count k; default 10
             |              | range [1, 50]; higher = more stable but slower; optional
max_num_clusters | int | scalar | upper bound on k to search; default 10 or N-1 (whichever smaller); optional

Outputs:
corr_mat_reordered | pd.DataFrame | shape (N, N) | correlation matrix with rows/cols sorted by cluster membership
clusters           | dict         | {cluster_id: [strategy_indices]} | optimal cluster assignment
silhouette_scores  | pd.Series    | shape (N,)  | per-strategy silhouette score; in [-1, 1]
K_star             | int          | scalar      | optimal number of clusters = Ne (effective independent trials)
  NOTE: K_star is derived as len(clusters); it is NOT directly returned as a named output by get_onc_clusters
  but equals the number of keys in the clusters dict

Ne derivation:
  Ne = K_star = len(clusters)
  This is the count to substitute for N in DSR/SR_0 formulas (see deflated-sharpe-ratio interface map)
  Ne is an integer in [1, N]; Ne=1 means all strategies are essentially one correlated bet

Critical invariants (testable assertions):
assert corr_mat.shape[0] == corr_mat.shape[1], "correlation matrix must be square"
assert abs(corr_mat.values.diagonal() - 1.0).max() < 1e-9, "diagonal of corr_mat must be 1.0"
assert abs(corr_mat.values - corr_mat.values.T).max() < 1e-9, "corr_mat must be symmetric"
assert 1 <= len(clusters) <= corr_mat.shape[0], "number of clusters must be in [1, N]"
assert silhouette_scores.between(-1.0, 1.0).all(), "silhouette scores must be in [-1, 1]"
assert set(itertools.chain(*clusters.values())) == set(range(corr_mat.shape[0])), "all strategies assigned"

Core formulas:

Angular distance (correlation to distance; used as distance metric for K-means):
  D_ij = sqrt(0.5 * (1 - rho_ij))
  where rho_ij is the Pearson correlation between strategy i and strategy j returns
  D_ij is in [0, 1]; D=0 means perfectly correlated (rho=1); D=1 means perfectly anti-correlated (rho=-1)
  SOURCE: mlfinlab readthedocs 0.12.0: "D_{i,j} = sqrt(0.5*(1-rho_{i,j}))"

Distance-of-distances (used in higher-level stage to measure inter-cluster distance):
  D_hat_ij = sqrt(sum_k (D_ik - D_jk)^2)
  This is the L2 norm between rows i and j of the angular distance matrix D
  SOURCE: mlfinlab readthedocs 0.12.0: "D_hat_{i,j} = sqrt(sum_k(D_{i,k} - D_{j,k})^2)"

Silhouette score per strategy i:
  a_i = mean distance from i to all other strategies in same cluster
  b_i = min over other clusters c': mean distance from i to all strategies in c'
  S_i = (b_i - a_i) / max(a_i, b_i)
  S_i in [-1, 1]; S_i close to 1 means well-clustered; S_i < 0 means misclassified
  SOURCE: mlfinlab readthedocs 0.12.0: "S_i = (b_i - a_i) / max(a_i, b_i)"

T-score clustering quality metric q for a clustering with k clusters:
  q = E[{S_i}] / sqrt(V[{S_i}])
  = mean(silhouette_scores) / std(silhouette_scores)
  NOTE: this is the t-statistic of the silhouette score distribution (NOT a Student-t test)
  Higher q = better clustering; used to select optimal k and to identify below-average clusters
  SOURCE: mlfinlab readthedocs 0.12.0: "q = E[{S_i}] / sqrt(V[{S_i}])"

Algorithm (two-stage):

Stage 1 - Base clustering (function cluster_kmeans_base):
  Input: corr_mat, max_num_clusters (default min(10, N-1)), repeat (default 10)
  For k in range(2, max_num_clusters + 1):
    For _ in range(repeat):
      Run K-means on angular distance matrix D with k clusters
      Compute silhouette scores S_i for all i
      Compute q_k = mean(S) / std(S)
      Track best q_k and corresponding cluster assignment for this k
  Select k* = argmax_k(q_k) over k in [2, max_num_clusters]
  Return: corr_mat reordered by cluster membership, cluster dict, silhouette scores

Stage 2 - Higher-level clustering (function cluster_kmeans_top):
  Input: corr_mat, repeat (default 10)
  Run Stage 1 on full corr_mat -> get initial clusters, q values per cluster
  Compute mean_q = mean of q values across all clusters
  Identify "weak" clusters: those with q < mean_q (more than 2 such clusters triggers re-clustering)
  If more than 2 weak clusters:
    Extract the sub-correlation-matrix for strategies in weak clusters
    Re-run Stage 1 on that sub-matrix -> get new sub-clusters
    Merge new sub-clusters with the strong clusters from the initial pass
    Compare total q of new merged solution vs original; keep better
  Repeat until convergence (no more than 2 weak clusters remain or q does not improve)
  Return: final reordered corr_mat, final clusters dict, final silhouette scores
  Ne = K* = number of keys in final clusters dict

Stopping criterion:
  The higher-level loop terminates when fewer than 3 clusters have q < mean_q OR when re-clustering
  does not improve total q. No hard iteration limit documented; convergence is typically fast (2-3 rounds).

Hyperparameters:
repeat            | default 10 | range [5, 50] | K-means restarts; increase for stability on small N
max_num_clusters  | default 10 | range [2, N-1] | cap on k search; set to N-1 to allow all possible splits

Complexity:
Time: O(repeat * max_num_clusters * N^2 * T_kmeans) per stage; T_kmeans = O(N * k * iters)
  Practical: O(repeat * N^2 * max_k) for small N (strategy count typically <= 1000)
Space: O(N^2) for distance matrix D and D_hat
Parallelism: K-means restarts within each k are independent; parallelizable over k as well

Failure modes:
1. N < 3: K-means with k=2 is the only option; ONC degenerates; Ne = N if N <= 2
2. All strategies perfectly correlated (rank-1 correlation matrix): silhouette scores undefined/degenerate
3. Very small repeat (repeat=1): K-means init sensitivity can cause unstable K* across runs
4. max_num_clusters set too low (e.g. 2 when true Ne=8): underestimates Ne, under-corrects DSR
5. corr_mat computed from too-short return series (T < 30*N): correlation matrix unreliable (Ledoit-Wolf needed)
6. Using raw Euclidean distance instead of angular distance: violates paper's metric choice
7. Book Chapter 4 bug: the ML for Asset Managers book has a confirmed ONC bug; use mlfinlab not book code

Implementation pitfalls (do NOT do this):
Do NOT use corr_mat = returns.corr() without checking for NaNs; NaN propagates into angular distance silently
Do NOT use k=1 in base clustering; silhouette score is undefined for a single cluster
Do NOT substitute std correlation distance (sqrt(1 - rho)) for angular distance (sqrt(0.5*(1-rho))); different metric
Do NOT treat the cluster count from a single run as stable; variance is high with repeat=1
Do NOT use Ne < 1; if ONC returns K*=0 (degenerate input), clamp to 1 and flag error
Do NOT pass annualized returns correlation directly if strategies have different return frequencies
Do NOT skip Stage 2 (higher-level stage); Stage 1 alone is base clustering, not full ONC
