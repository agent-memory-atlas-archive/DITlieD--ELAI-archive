# L-2 decisions

- The shipped tier function uses two distinct fired predictors for High or Critical, one fired predictor capped at Medium, none is Low, `causal_supported` only from a fired counterfactual, and logprob is optional.
- The journal records one row per `ClaimSet`, keyed on the maximum tier, rather than one row per claim, so the aggregate observation preserves the set-level risk signal without multiplying journal events.
- A new `SsbActionType` was required instead of riding `ClaimConfidenceScored`: that payload is flat, has no Kind enum, and uses `deny_unknown_fields`; extending it would break deserialization of existing rows.
- The tier boundary constants are authored and pending L-10 measurement (ID-5).
- This slice is observe-only: it records tiers and acts on nothing; the action gate is L-7.
