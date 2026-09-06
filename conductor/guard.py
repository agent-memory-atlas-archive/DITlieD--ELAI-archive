"""Safety guard for prompts and Codex output."""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Iterable


# Patterns that suggest prompt injection in the input we send to the model
PROMPT_INJECTION_PATTERNS = (
    re.compile(r"ignore\s+(?:all\s+)?(?:previous|prior)\s+(?:instructions|prompts)", re.I),
    re.compile(r"disregard\s+(?:all\s+)?(?:previous|prior)\s+(?:instructions|prompts)", re.I),
    re.compile(r"forget\s+(?:all\s+)?(?:previous|prior)\s+(?:instructions|prompts)", re.I),
    re.compile(r"you\s+(?:are|were)\s+now", re.I),
    re.compile(r"new\s+(?:system|instruction|prompt|role)", re.I),
    re.compile(r"system\s+prompt", re.I),
    re.compile(r"override\s+(?:instructions|constraints|rules)", re.I),
    re.compile(r"ignore\s+(?:the\s+)?(?:constraints|rules)", re.I),
    re.compile(r"ignore\s+(?:previous|above|above\s+instructions)", re.I),
    re.compile(r"do\s+not\s+(?:tell|inform|let)\s+(?:the\s+)?user", re.I),
    re.compile(r"this\s+is\s+(?:a\s+)?test\s+(?:of|for)\s+(?:your|the)\s+instructions", re.I),
    re.compile(r"do\s+anything\s+now", re.I),
    re.compile(r"\bDAN\b", re.I),
    re.compile(r"developer\s+mode", re.I),
    re.compile(r"jailbreak", re.I),
    re.compile(r"simulate\s+(?:being|acting\s+as)", re.I),
    re.compile(r"pretend\s+(?:to\s+be|you\s+are)", re.I),
)

# Command-position anchor for destructive shell commands. It allows a line start or shell separator
# and excludes prose, quoted mentions, markdown list items, and diff hunks.
_CMD = r"(?:^[ \t]*(?:\$[ \t]+)?|[;&|]{1,2}[ \t]*)"
_JSON_CMD = r'[{,]\s*"(?:command|cmd|shell[_-]?command|command[_-]?line)"\s*:\s*"[ \t]*(?:\$[ \t]+)?'

# Patterns that suggest dangerous instructions in the model output we receive
DANGEROUS_OUTPUT_PATTERNS = (
    re.compile(_CMD + r"rm\s+-rf\s+/", re.I | re.M),
    re.compile(_JSON_CMD + r"rm\s+-rf\s+/", re.I),
    re.compile(r"\brm\s+.*--no-preserve-root", re.I),
    re.compile(r"\bmkfs\.?", re.I),
    re.compile(r"\bdd\s+if=", re.I),
    re.compile(r"\bformat\s+c:", re.I),
    re.compile(r"\bdel\s+/[fqs]", re.I),
    re.compile(_CMD + r"git\s+push\b", re.I | re.M),
    re.compile(_CMD + r"git\s+reset\s+--hard\b", re.I | re.M),
    re.compile(_CMD + r"git\s+clean\s+-[fd]", re.I | re.M),
    re.compile(_CMD + r"git\s+checkout\s+--\s*\.", re.I | re.M),
    re.compile(_CMD + r"git\s+restore\s+--staged\s+\.", re.I | re.M),
    re.compile(_CMD + r"git\s+revert\s+--no-edit", re.I | re.M),
    re.compile(_CMD + r"git\s+branch\s+-D\b", re.I | re.M),
    re.compile(_CMD + r"git\s+tag\s+-d\b", re.I | re.M),
    re.compile(_CMD + r"git\s+reflog\s+expire\b", re.I | re.M),
    re.compile(_CMD + r"git\s+filter-branch\b", re.I | re.M),
    re.compile(_CMD + r"git\s+filter-repo\b", re.I | re.M),
    re.compile(_CMD + r"git\s+gc\b[^\n]*--prune=now", re.I | re.M),
    re.compile(r"\bchmod\s+-R\s+777\s+/", re.I),
    re.compile(r"\bchown\s+-R\s+root\s+/", re.I),
    re.compile(r"\bsudo\s+rm\b", re.I),
    re.compile(r"(?:\b|^|(?<=[\s;&|]))(?:\d+|&|:)?>>?\s*/(?:etc|boot|sys|proc|usr|bin|sbin|lib|lib64|var|root)(?:/|\b)", re.I),
    re.compile(_CMD + r"shutdown\s+-h\s+now\b", re.I | re.M),
    re.compile(_CMD + r"reboot\b", re.I | re.M),
    re.compile(_CMD + r"kill\s+-9\s+\b", re.I | re.M),
    re.compile(_CMD + r"pkill\s+-9\b", re.I | re.M),
    re.compile(r"(?:\b|^|(?<=[\s;&|]))(?:\d+|&|:)?>>?\s*/dev/[sh]da", re.I),
    re.compile(r"\bwget\s+.*\|\s*bash", re.I),
    re.compile(r"\bcurl\s+.*\|\s*bash", re.I),
    re.compile(r"\bcurl\s+.*\|\s*sh", re.I),
    re.compile(r"\bnc\s+.*-e\s+\b", re.I),
    re.compile(r"\bpython\s+-c\s+.*import\s+socket.*subprocess", re.I),
    re.compile(r"\bbase64\s+.*\|\s*bash", re.I),
    re.compile(r"\becho\s+.*\|\s*base64\s+-d", re.I),
    re.compile(r"\bchmod\s+\+x\s+.*\|\s*bash", re.I),
)

# Patterns that suggest data exfiltration or callback attempts
EXFILTRATION_PATTERNS = (
    re.compile(r"\bcurl\s+.*?(?:https?://|ftp://)", re.I),
    re.compile(r"\bwget\s+.*?(?:https?://|ftp://)", re.I),
    re.compile(r"\bscp\s+.*@.*:", re.I),
    re.compile(r"\bnc\s+.*?(?:\d{1,3}\.){3}\d{1,3}", re.I),
    re.compile(r"\bnetcat\s+.*?(?:\d{1,3}\.){3}\d{1,3}", re.I),
    re.compile(r"\bpython\s+-m\s+http\.server", re.I),
    re.compile(r"\bphp\s+-S\s+", re.I),
    re.compile(r"\bssh\s+.*@.*", re.I),
    re.compile(r"\btelnet\s+.*?(?:\d{1,3}\.){3}\d{1,3}", re.I),
)

# Patterns that suggest the model is trying to give itself instructions
INSTRUCTION_OVERRIDE_PATTERNS = (
    re.compile(r"system\s*:?\s*you\s+are\s+now", re.I),
    re.compile(r"assistant\s*:?\s*ignore\s+previous", re.I),
    re.compile(r"user\s*:?\s*ignore\s+all", re.I),
    re.compile(r"important\s*:?\s*ignore\s+previous", re.I),
    re.compile(r"instructions\s*:?\s*ignore", re.I),
    re.compile(r"from\s+now\s+on\s+you\s+are", re.I),
)


@dataclass(frozen=True)
class GuardResult:
    safe: bool
    reasons: tuple[str, ...]
    score: int

    @property
    def blocked(self) -> bool:
        return not self.safe


def _scan(text: str, patterns: Iterable[tuple[re.Pattern, str, int]]) -> GuardResult:
    reasons: list[str] = []
    score = 0
    for pattern, label, weight in patterns:
        if pattern.search(text):
            reasons.append(f"{label}: matched {pattern.pattern[:60]}")
            score += weight
    return GuardResult(safe=not reasons, reasons=tuple(reasons), score=score)


class PromptGuard:
    """Stateful guard that can be configured with extra patterns."""

    def __init__(self, extra_patterns: Iterable[tuple[re.Pattern, str, int]] | None = None) -> None:
        self.patterns: list[tuple[re.Pattern, str, int]] = [
            (p, "prompt_injection", 10) for p in PROMPT_INJECTION_PATTERNS
        ]
        if extra_patterns:
            self.patterns.extend(extra_patterns)

    def check(self, text: str) -> GuardResult:
        return _scan(text, self.patterns)


class OutputGuard:
    """Stateful guard that can be configured with extra patterns."""

    def __init__(self, extra_patterns: Iterable[tuple[re.Pattern, str, int]] | None = None) -> None:
        self.patterns: list[tuple[re.Pattern, str, int]] = [
            (p, "dangerous_command", 10) for p in DANGEROUS_OUTPUT_PATTERNS
        ]
        self.patterns.extend((p, "exfiltration", 7) for p in EXFILTRATION_PATTERNS)
        self.patterns.extend((p, "instruction_override", 8) for p in INSTRUCTION_OVERRIDE_PATTERNS)
        if extra_patterns:
            self.patterns.extend(extra_patterns)

    def check(self, text: str) -> GuardResult:
        return _scan(text, self.patterns)
