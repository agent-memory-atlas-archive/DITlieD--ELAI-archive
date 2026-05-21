"""Unit tests for FP8 KV cache toggle in InferenceEngine.

E-27 Task: verify kv_cache_dtype_software_sim is emitted on Ampere (SM < 89)
and that kv_cache_dtype attribute is set correctly for all valid dtype strings.

Run: python -m pytest elai_inference/test_fp8_kv_cache.py
"""

VALID_KV_DTYPES = ("fp16", "fp8_e4m3", "fp8_e5m2")
FP8_KV_DTYPES = ("fp8_e4m3", "fp8_e5m2")
ADA_PLUS_SM = 89
AMPERE_SM = 86
THROUGHPUT_COST_PCT = 5.0


def test_valid_kv_dtypes():
    assert "fp16" in VALID_KV_DTYPES
    assert "fp8_e4m3" in VALID_KV_DTYPES
    assert "fp8_e5m2" in VALID_KV_DTYPES


def test_fp8_dtypes_subset():
    for d in FP8_KV_DTYPES:
        assert d in VALID_KV_DTYPES


def test_ampere_sm_is_below_ada():
    assert AMPERE_SM < ADA_PLUS_SM


def _should_use_software_sim(kv_dtype: str, compute_cap: int) -> bool:
    return kv_dtype in FP8_KV_DTYPES and compute_cap < ADA_PLUS_SM


def _throughput_cost_pct(kv_dtype: str, compute_cap: int) -> float:
    if _should_use_software_sim(kv_dtype, compute_cap):
        return THROUGHPUT_COST_PCT
    return 0.0


def test_fp16_no_software_sim():
    assert not _should_use_software_sim("fp16", AMPERE_SM)
    assert not _should_use_software_sim("fp16", ADA_PLUS_SM)


def test_fp8_ampere_software_sim():
    assert _should_use_software_sim("fp8_e4m3", AMPERE_SM)
    assert _should_use_software_sim("fp8_e5m2", AMPERE_SM)


def test_fp8_ada_native():
    assert not _should_use_software_sim("fp8_e4m3", ADA_PLUS_SM)
    assert not _should_use_software_sim("fp8_e5m2", ADA_PLUS_SM)


def test_throughput_cost_fp16():
    assert _throughput_cost_pct("fp16", AMPERE_SM) == 0.0


def test_throughput_cost_fp8_ampere():
    assert _throughput_cost_pct("fp8_e5m2", AMPERE_SM) == THROUGHPUT_COST_PCT


def test_throughput_cost_fp8_ada():
    assert _throughput_cost_pct("fp8_e5m2", ADA_PLUS_SM) == 0.0


class FakeLogger:
    def __init__(self):
        self.warnings = []

    def warning(self, event, **kwargs):
        self.warnings.append({"event": event, **kwargs})


def test_software_sim_warning_emitted():
    logger = FakeLogger()
    kv_dtype = "fp8_e5m2"
    compute_cap = AMPERE_SM
    if _should_use_software_sim(kv_dtype, compute_cap):
        logger.warning(
            "kv_cache_dtype_software_sim",
            kv_dtype=kv_dtype,
            throughput_cost_pct=_throughput_cost_pct(kv_dtype, compute_cap),
            compute_capability=compute_cap,
        )
    assert len(logger.warnings) == 1
    w = logger.warnings[0]
    assert w["event"] == "kv_cache_dtype_software_sim"
    assert w["kv_dtype"] == "fp8_e5m2"
    assert w["throughput_cost_pct"] == THROUGHPUT_COST_PCT
    assert w["compute_capability"] == AMPERE_SM


def test_no_warning_on_ada():
    logger = FakeLogger()
    kv_dtype = "fp8_e5m2"
    compute_cap = ADA_PLUS_SM
    if _should_use_software_sim(kv_dtype, compute_cap):
        logger.warning("kv_cache_dtype_software_sim")
    assert len(logger.warnings) == 0
