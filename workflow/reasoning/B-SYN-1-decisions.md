# B-SYN-1 decisions

What I will implement: an archive-specific, zero-call replay CLI that reconstructs P-25 and b0v3
paired deltas at K=30/15/7 and appends the locked power block. A separate checker will independently
rederive source digests, intervals, effective power states, grammar fixtures, and the family verdict.

Key technical decision: preserve native archive verdicts separately from minimum-useful-effect
resolution, with raw prefix states reported and a conservative monotone effective envelope.

Expected test outcome: unchanged full-K results, at least one smaller-prefix
`NULL-UNDERPOWERED`, exact quantified-tail/legacy round-trips, and producer/checker agreement.

Unknowns to verify empirically:

- which comparisons become underpowered at K=15 or K=7;
- whether the current opaque KILL-EDGE fold preserves the canonical tail exactly;
- whether any raw smaller-prefix interval is non-monotone before the conservative envelope.
