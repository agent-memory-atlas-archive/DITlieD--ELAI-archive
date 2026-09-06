# Conductor: the tooling that worked on ELAI

Conductor was the external automation used to coordinate development of ELAI:
select a plan, dispatch workers into isolated Git worktrees, collect evidence,
run gates, request a fresh review, and integrate accepted changes.

It is separate from the agent harness implemented inside `.elai_cc/`.
The [selected source snapshot](conductor/) includes the ELAI walk and its local
Python dependencies, database migrations, related tests and supporting gate tools.
It also contains later changes; it is not an exact reconstruction of every
configuration used during ELAI's development.

**Start with [how Conductor worked, what is included, and what is missing](conductor/README.md).**
That guide links the execution path, historical documentation and verification record.

The recovered [earlier implementation plan](conductor/IMPLEMENTATION_PLAN.md) and
[ACP comparison notes](conductor/ACP-IMPORT-NOTES.md) add design history. They may
describe components outside the selected code snapshot.

This is another abandoned research artifact. It is not a ready-to-run service.
Accounts, live state, worktrees, provider runtimes and personal services are omitted.
Old instructions and passing fixture tests do not certify autonomous operation.

[Back to ELAI](README.md) · [Reusable pieces](SALVAGE-MAP.md)
