# claims_policy_lint.py negative fixture (K-1 Task 5, seeded to FAIL)

This file exists ONLY to be scanned by `claims_policy_lint.py`; it is not
positioning copy. It seeds the two violation classes the lint must catch:

ELAI is the safest coding agent on the market today.

Independent researchers found hallucinated packages in 21.7% of open-source
LLM code generations.

Both lines above ship a claim (a banned superlative, then a bare untagged
external number) with no [EXTERNAL]/[DESIGN]/[MEASURED] tag and no recompute
pointer on the same line. `claims_policy_lint.py --files <this file>` MUST
exit nonzero.
