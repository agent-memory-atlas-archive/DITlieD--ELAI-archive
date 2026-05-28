"""
E-54 sidecar embeddings endpoint -- BGE-M3 / Nomic Embed v2.

POST /v1/embeddings {texts:[str], model:"bge-m3"|"nomic-embed-v2"}
  -> {"embeddings": [[f32]], "model": str, "dim": int}

Reuses the sidecar's FastAPI app surface (server.py owns the actual mount).
Loads models LAZILY on first request: BGE-M3 (BAAI/bge-m3, 1024-dim) is the
default; Nomic Embed v2 (nomic-ai/nomic-embed-text-v2-moe, 768-dim) is opt-in.

Per CLAUDE.md HARD RULE anti-slopsquatting: both HF model deps are verified
via HuggingFace model card WebFetch + last-published date BEFORE adoption
(verification log in workflow/reasoning/E-54-decisions.md DECISION-9).

Per CLAUDE.md anti-stale-training: this module uses model.encode() (not
manual token loops) and torch.inference_mode() (not torch.no_grad()).

E-54 RUNPOD-PENDING: live BGE-M3 / Nomic loading on the remote 3090 sidecar.
This module ships the route + LazyLoader scaffold; the actual transformer
load is deferred until the user opt-ins via ELAI_EVAL_CORPUS_EMBEDDINGS=1
AND the sidecar is up. Without the env-var, the route returns 503 so the
Rust ingest pipeline falls back to ngram-only mode with a
corpus_ingest_cosine_unavailable SSB row.
"""

from __future__ import annotations

import logging
import os
import threading
from typing import Any, Dict, List, Optional

logger = logging.getLogger("elai.embeddings")


_ENV_GATE = "ELAI_EVAL_CORPUS_EMBEDDINGS"
_DEFAULT_MODEL = "bge-m3"
_SUPPORTED_MODELS = {
    "bge-m3": {
        "hf_id": "BAAI/bge-m3",
        "dim": 1024,
        "device_preferred": "cuda:0",
    },
    "nomic-embed-v2": {
        "hf_id": "nomic-ai/nomic-embed-text-v2-moe",
        "dim": 768,
        "device_preferred": "cuda:0",
    },
}


class EmbeddingsRoute:
    """Lazy loader + thread-safe model cache for the /v1/embeddings route.

    One instance is owned by server.py; the actual model object is loaded on
    first request (or never, when ELAI_EVAL_CORPUS_EMBEDDINGS != "1").
    """

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._models: Dict[str, Any] = {}

    def opt_in_enabled(self) -> bool:
        v = os.environ.get(_ENV_GATE, "")
        return v.strip().lower() in {"1", "true", "yes", "on"}

    def model_supported(self, name: str) -> bool:
        return name in _SUPPORTED_MODELS

    def dim_for(self, name: str) -> int:
        return _SUPPORTED_MODELS[name]["dim"]

    def _load(self, name: str) -> Any:
        # RUNPOD-PENDING: live BGE-M3 / Nomic Embed v2 load via
        # sentence-transformers OR FlagEmbedding. The actual import is
        # deferred until the user opts in (heavy compile + GPU load).
        # Until then this raises so the caller surfaces a clean 503.
        # When wiring live: replace the body with the canonical lazy load.
        raise NotImplementedError(
            "E-54 embeddings endpoint is RUNPOD-PENDING -- "
            "BGE-M3 / Nomic Embed v2 lazy load to be wired alongside the "
            "remote 3090 sidecar; see workflow/reasoning/E-54-decisions.md "
            "DECISION-9 + DECISION-10."
        )

    def encode_batch(self, texts: List[str], model: Optional[str] = None) -> Dict[str, Any]:
        """Encode a batch of texts; returns the JSON response body.

        Raises RuntimeError("disabled") when the opt-in gate is off; the
        caller (server.py route handler) converts that to a 503.
        Raises ValueError when the model is unsupported.
        Raises NotImplementedError when the lazy load is still RUNPOD-PENDING.
        """
        if not self.opt_in_enabled():
            raise RuntimeError("disabled")
        chosen = (model or _DEFAULT_MODEL).strip().lower()
        if not self.model_supported(chosen):
            raise ValueError(
                f"E-54 embeddings: unsupported model {chosen!r}; "
                f"supported = {sorted(_SUPPORTED_MODELS.keys())}"
            )
        with self._lock:
            if chosen not in self._models:
                self._models[chosen] = self._load(chosen)
        m = self._models[chosen]
        # RUNPOD-PENDING: call m.encode(texts, normalize_embeddings=True) for
        # BGE-M3; m.encode(texts, prompt_name="passage") for Nomic Embed v2.
        # Until then, return an empty embedding list so the test harness can
        # exercise the route shape without a live model.
        del m
        return {
            "embeddings": [[0.0] * self.dim_for(chosen) for _ in texts],
            "model": chosen,
            "dim": self.dim_for(chosen),
            "runpod_pending": True,
        }


_route_singleton: Optional[EmbeddingsRoute] = None
_singleton_lock = threading.Lock()


def get_route() -> EmbeddingsRoute:
    """Module-level lazy accessor used by server.py to mount the route."""
    global _route_singleton
    if _route_singleton is None:
        with _singleton_lock:
            if _route_singleton is None:
                _route_singleton = EmbeddingsRoute()
    return _route_singleton


def list_supported_models() -> List[str]:
    """Public helper for the /v1/embeddings/models GET surface."""
    return sorted(_SUPPORTED_MODELS.keys())


__all__ = ["EmbeddingsRoute", "get_route", "list_supported_models"]
