"""C-36 MuCoCo MutationResult dataclass.

Grounded by paper interface map sec "Per-operator apply contract":
  - applied=True, mutated_source=<rewritten>, skip_reason=None
  - applied=False, mutated_source=<original>, skip_reason="<why>"

The paper (sec S3.p3.1) MANDATES applicability check first; non-applicable
operators MUST skip with a populated `skip_reason`, NEVER synthesize a fake
mutation. This module enforces that contract via the dataclass constructor.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class MutationResult:
    """Single mutation operator output.

    Fields (paper interface contract):
      applied        -- True iff a semantic-preserving rewrite was applied
      mutated_source -- the rewritten source (== original when applied=False)
      skip_reason    -- non-None and non-empty iff applied=False
      operator       -- operator name (snake_case from the registry)
    """

    operator: str
    applied: bool
    mutated_source: str
    skip_reason: Optional[str]

    def __post_init__(self) -> None:
        # Stub-ban: a skipped mutation MUST carry a non-empty reason so the
        # consumer surface (Rust SsbActionType::MucocoMutationApplied) gets
        # a debuggable trail. Mirrors crates/orchestrator/src/metamorphic_testing.rs
        # DRIFT-C29-11 "counterexample MUST be non-empty" convention.
        if not self.applied and (self.skip_reason is None or not self.skip_reason):
            # Cannot mutate frozen dataclass; raise so the bug surfaces at
            # call site rather than producing a silently-corrupt SSB row.
            raise ValueError(
                f"MutationResult(operator={self.operator!r}, applied=False) "
                "requires a non-empty skip_reason (stub_ban contract)"
            )
        if self.applied and self.skip_reason is not None:
            raise ValueError(
                f"MutationResult(operator={self.operator!r}, applied=True) "
                "MUST have skip_reason=None (mutually exclusive)"
            )


def applied(operator: str, mutated_source: str) -> MutationResult:
    """Builder for the applied=True path."""
    return MutationResult(
        operator=operator,
        applied=True,
        mutated_source=mutated_source,
        skip_reason=None,
    )


def skipped(operator: str, source: str, reason: str) -> MutationResult:
    """Builder for the applied=False path; preserves original source."""
    return MutationResult(
        operator=operator,
        applied=False,
        mutated_source=source,
        skip_reason=reason,
    )
