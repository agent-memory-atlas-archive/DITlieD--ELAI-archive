---
name: grill
description: Landable rule-text mirror for the grill hard constraint (J-79). Full harness skill is gitignored under .claude/skills/grill/SKILL.md.
license: MIT
metadata:
  version: "1.0.0"
  author: "elai"
---

# Grill hard constraint (landable mirror, J-79)

This file is the product-landable rule-text for grill. The full harness skill
body lives at `.claude/skills/grill/SKILL.md` (gitignored). Production code
in `grill_interlace.rs` binds to the rules below.

## HARD CONSTRAINT — never auto-RUN grill

NEVER auto-start the grill interrogation without an explicit user accept.
Auto-starting the one-question-at-a-time loop unprompted is always banned.
An interrogation the user did not consent to is friction and undercuts
ELAI's anti-sycophancy posture.

## When an OFFER is allowed (J-79 consent-gated)

A signal-triggered OFFER is allowed and is consent-gated. It is NOT an
auto-run. When the B-40 idea-critic BLOCK (or later B-38/C-29) fires, the
runtime MAY surface grill in one line
("this looks ambiguous across N branches — grill it before i plan ?")
and WAIT for the user to accept.

- OFFER surfaced + WAIT = allowed (J-79).
- Auto-starting the interrogation without user accept = banned.

## Consent

Consent is an explicit user accept of the offer. Silence or vague assent
is not consent. The grill interrogation starts only after accept.
