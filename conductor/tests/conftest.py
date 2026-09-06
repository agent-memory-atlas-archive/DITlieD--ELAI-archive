"""Shared pytest fixtures for the conductor suite.

BRAIN-RAG safety net (U1): index-on-write now embeds every brain write into brain_vec, whose
default path is the REAL state/brain_vec.db. Any test that does a brain write (upsert_entry /
rank_item / queue_item / write_page / upsert_page -- directly or via the research suites) under a
semantic-live interpreter (.memory-venv) would otherwise pollute the owner's production vector db
with test fixtures. This autouse fixture points TEIKOKU_BRAIN_VEC_DB at a per-test temp file so NO
test can ever touch state/brain_vec.db, regardless of which suite it lives in. A test that sets its
own TEIKOKU_BRAIN_VEC_DB in setUp still wins (setUp runs after this fixture); this is the floor.
"""
from __future__ import annotations

import os
import tempfile

import pytest


@pytest.fixture(autouse=True)
def _isolate_brain_vec_db():
    old = os.environ.get("TEIKOKU_BRAIN_VEC_DB")
    with tempfile.TemporaryDirectory(prefix="brain-vec-guard-") as d:
        os.environ["TEIKOKU_BRAIN_VEC_DB"] = os.path.join(d, "brain_vec.db")
        try:
            yield
        finally:
            if old is None:
                os.environ.pop("TEIKOKU_BRAIN_VEC_DB", None)
            else:
                os.environ["TEIKOKU_BRAIN_VEC_DB"] = old
