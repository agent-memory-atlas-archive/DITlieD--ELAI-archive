"""C-37 HotpotQA distractor dataset loader.

Paper Section 4.1 (verbatim):
  Dataset: HotpotQA distractor setting (Yang et al. 2018). Each question
  has 2 gold paragraphs + 8 distractors. The paper's pilot sampled 1000
  questions from HotpotQA, observed Qwen3-8B solving 631 first-try,
  filtered the remaining 369 via LLM-as-judge semantic equivalence + a
  difficulty heuristic, then split the resulting ~100 hard samples into:
    Pool A: 55 samples — baseline Reflexion solved between trials 2 and 5
                         primary metric: Average Trials (AT)
    Pool B: 45 samples — baseline completely failed within 5 trials
                         primary metric: Success Rate (SR)

Repo absence: the paper's published Pool A / Pool B IDs are NOT recoverable
from the original repo (HTTP 404 per workflow/papers/
structure-snowballing-source-interface.md). This loader implements a
deterministic-seeded split over the dev-distractor set so reruns are
reproducible — but the exact 100 IDs cannot match the paper without the
author's manifest.

Set env var HOTPOTQA_DISTRACTOR_PATH to the path of the
`hotpot_dev_distractor_v1.json` file (HuggingFace mirror or
hotpotqa.github.io download). If unset, defaults to
`~/datasets/hotpotqa/hotpot_dev_distractor_v1.json`.

The download is NOT performed automatically (no network calls in this
module). When the file is missing, `print_setup_instructions()` is
called and the function raises FileNotFoundError so the user makes the
explicit choice.
"""

from __future__ import annotations

import json
import os
import random
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


# Paper-grounded pool sizes (Section 4.1).
POOL_A_SIZE: int = 55  # baseline-solved-between-trial-2-and-5
POOL_B_SIZE: int = 45  # baseline-completely-failed

DEFAULT_DATASET_PATH_ENV: str = "HOTPOTQA_DISTRACTOR_PATH"
DEFAULT_DATASET_PATH_FALLBACK: str = "~/datasets/hotpotqa/hotpot_dev_distractor_v1.json"

# Deterministic seed for the pool split. Repeated runs against the same
# dataset file produce identical Pool A / Pool B ids.
SPLIT_SEED: int = 20260524  # YYYYMMDD of session 2026-05-24


@dataclass(frozen=True)
class HotpotQAItem:
    """One question with gold + distractors.

    Field names match the HotpotQA distractor JSON schema exactly so
    callers can read the file shape from the dataclass.
    """

    question_id: str
    question: str
    gold_answer: str
    gold_paragraphs: list[str]  # 2 paragraphs containing the answer
    distractors: list[str]  # 8 distracting paragraphs


def default_dataset_path() -> Path:
    """Resolve the dataset path from env var, falling back to the documented
    default location.

    No I/O is performed; the returned path is NOT validated for existence.
    """
    raw = os.environ.get(DEFAULT_DATASET_PATH_ENV) or DEFAULT_DATASET_PATH_FALLBACK
    return Path(os.path.expanduser(raw))


def print_setup_instructions(path: Path) -> None:
    """Emit a self-contained setup message when the dataset is missing.

    Print only — never raises, never auto-downloads. The user must choose.
    """
    msg = (
        f"\n[C-37 setup] HotpotQA distractor JSON not found at: {path}\n"
        "  Download URL (Yang et al. 2018, paper Section 4.1):\n"
        "    https://hotpotqa.github.io/  (dev_distractor: ~46 MB)\n"
        "    direct: http://curtis.ml.cmu.edu/datasets/hotpot/"
        "hotpot_dev_distractor_v1.json\n"
        f"  Either: (a) place the file at the path above, or\n"
        f"          (b) set HOTPOTQA_DISTRACTOR_PATH=/your/path/hotpot_dev_distractor_v1.json\n"
        "  No automatic download is performed.\n"
    )
    print(msg)


def _load_all(path: Path) -> list[HotpotQAItem]:
    """Read the full distractor JSON file into HotpotQAItem records.

    The HotpotQA distractor schema: list of dicts with keys
      {_id, question, answer, supporting_facts, context, type, level}
    where `context: list[[title:str, sentences:list[str]]]` has 10 entries
    (2 gold + 8 distractors). Gold paragraphs are those whose title
    appears in any supporting_facts row.

    Raises FileNotFoundError with setup hint when the file is missing.
    """
    if not path.exists():
        print_setup_instructions(path)
        raise FileNotFoundError(
            f"HotpotQA distractor dataset not found at {path}. "
            f"Set {DEFAULT_DATASET_PATH_ENV} or place file at default."
        )

    with open(path, "r", encoding="utf-8") as fh:
        raw = json.load(fh)

    items: list[HotpotQAItem] = []
    for entry in raw:
        # Pull gold titles from supporting_facts (list of [title, sent_idx]).
        gold_titles: set[str] = set()
        for sf in entry.get("supporting_facts", []):
            if isinstance(sf, list) and len(sf) >= 1:
                gold_titles.add(sf[0])

        gold_paragraphs: list[str] = []
        distractors: list[str] = []
        for ctx_pair in entry.get("context", []):
            if not isinstance(ctx_pair, list) or len(ctx_pair) < 2:
                continue
            title, sentences = ctx_pair[0], ctx_pair[1]
            paragraph_text = "".join(sentences) if isinstance(sentences, list) else str(sentences)
            paragraph = f"{title}\n{paragraph_text}"
            if title in gold_titles:
                gold_paragraphs.append(paragraph)
            else:
                distractors.append(paragraph)

        items.append(
            HotpotQAItem(
                question_id=str(entry.get("_id", "")),
                question=str(entry.get("question", "")),
                gold_answer=str(entry.get("answer", "")),
                gold_paragraphs=gold_paragraphs,
                distractors=distractors,
            )
        )
    return items


def _deterministic_split(
    items: list[HotpotQAItem],
    pool_a_size: int,
    pool_b_size: int,
    seed: int,
) -> tuple[list[HotpotQAItem], list[HotpotQAItem]]:
    """Deterministic seeded split of items into Pool A + Pool B.

    Without the paper's exact question IDs (repo absent), the harness
    reproduces the paper's CARDINALITY but not its EXACT membership. A
    pinned seed keeps runs reproducible across machines.
    """
    rng = random.Random(seed)
    indices = list(range(len(items)))
    rng.shuffle(indices)
    needed = pool_a_size + pool_b_size
    if len(indices) < needed:
        raise ValueError(
            f"Dataset has {len(indices)} items but Pool A+B need {needed}."
        )
    a_indices = indices[:pool_a_size]
    b_indices = indices[pool_a_size : pool_a_size + pool_b_size]
    pool_a = [items[i] for i in a_indices]
    pool_b = [items[i] for i in b_indices]
    return pool_a, pool_b


def load_pool_a(path: Path | None = None) -> list[HotpotQAItem]:
    """Load Pool A (baseline-solved-between-trial-2-and-5) — 55 items.

    Deterministic seeded selection per SPLIT_SEED.
    """
    target = path if path is not None else default_dataset_path()
    items = _load_all(target)
    pool_a, _ = _deterministic_split(items, POOL_A_SIZE, POOL_B_SIZE, SPLIT_SEED)
    return pool_a


def load_pool_b(path: Path | None = None) -> list[HotpotQAItem]:
    """Load Pool B (baseline-completely-failed) — 45 items.

    Deterministic seeded selection per SPLIT_SEED.
    """
    target = path if path is not None else default_dataset_path()
    items = _load_all(target)
    _, pool_b = _deterministic_split(items, POOL_A_SIZE, POOL_B_SIZE, SPLIT_SEED)
    return pool_b


def iter_pool_items(items: Iterable[HotpotQAItem]) -> Iterable[HotpotQAItem]:
    """Pass-through iterator. Present so callers can plug a generator
    chain in front of the loaders without changing call sites.
    """
    for item in items:
        yield item
