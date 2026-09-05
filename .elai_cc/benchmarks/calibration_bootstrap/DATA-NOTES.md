# Calibration bootstrap data notes

The public Git snapshot intentionally omits these four local QA inputs:

- `halueval_qa_500.jsonl`
- `fixtures/halueval_qa_500_full.jsonl`
- `mmlu_verified_500.jsonl`
- `fixtures/mmlu_verified_500_full.jsonl`

The local originals are preserved for existing local workflows but are not redistributed in the public archive. The older benchmark checks that consumed HaluEval/MMLU inputs are therefore not reproducible from the archive alone; rerunning them requires restoring the omitted inputs and matching source snapshots. The archive does not claim those checks are self-contained.

The retained TruthfulQA files are covered by the root `THIRD_PARTY_NOTICES.md` and the Apache 2.0 license record in `third-party-licenses/`.
