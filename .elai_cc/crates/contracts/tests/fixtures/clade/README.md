Frozen archive fixture for h46_clade_metaproductivity_contract.rs

Node A: high immediate score (0.9), direct child A1 that regresses by -0.3.
Node B: mediocre immediate score (0.4), direct child B1 that improves by +0.3.

A1 and B2 are leaves (no fitness delta) to also exercise the ImmediateScoreFallback arm.

The test asserts:
  cmp(B) > cmp(A) despite immediate(A) > immediate(B)
  select_parent_with_cmp returns B over the real path (R4/R7)
  force_immediate_score=true (R2 broken arm) returns A
  a node with no descendants uses ImmediateScoreFallback
  same inputs -> byte-identical cmp (GD-2)
