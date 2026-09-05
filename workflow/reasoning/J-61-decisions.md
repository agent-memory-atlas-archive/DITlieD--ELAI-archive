J-61 decisions — canvas redirect loop (grill-resolved 2026-06-20)

what: the steer half of the layered canvas. a closed verb set over EXISTING nodes; every user edit is a journaled intent command to the single-writer engine; the LLM enacts across altitudes; the checker turns broken edges red deterministically; the user preempts.

key decisions (from the grill):
- redirect verbs (closed set): CONNECT (LLM proposes wiring, J-31 checker derives verdict — the user's original "connect and it wires itself"), CUT-EDGE, REORDER, REBIND, + AGENT-CONTROL (pause/redirect/kill on the per-agent tab). NEW nodes are born in CHAT (J-32), never hand-authored on the canvas. no node-delete verb (append-only journal; removal is lifecycle). because: manual node-wiring loses to typing — the documented visual-builder retention plateau; keep authoring in chat, keep the canvas for what's spatial.
- edit = journaled CanvasRedirectIntent COMMAND to the single writer, NOT a direct canvas mutation. canvas is read-only on SSB (SsbViewer WAL poll). preserves "no parallel state" with live editing.
- conflict model = the SSB total order. single-writer append-only WAL + sequence (phase-3B) gives a total order for free; no lock, no CRDT (two writers, not N). CORRECTION: I-6 PrivilegedAnchor was mis-cited earlier as a last-writer concurrency model — I-6 is a context-pane ordering slot (feature-map L314-316); the real model is the SSB sequence.
- collision policy = USER PREEMPTS. a pending user intent supersedes a not-yet-committed LLM action on the same node; rides H-49 RV-1 snapshot-before-restore + TaskStop/J-12 for agents. because: "user redirects" is the whole point — grab the wheel, get it now.
- build stays FSM (the J-31 category line): a redirect changes the plan/order/binding the FSM executes; the canvas never executes anything.

scope: 1-2 net-new J-family units extending J-31 (system) + H-13 (emit) + H-15 (deixis); J-60 = observability, J-61 = redirect. J-31 stays system-focused + a rider. NOT a new plan family.

expected test outcome: a redirect lands a CanvasRedirectIntent row and the graph re-derives from it (no direct mutation); a cycle-creating REORDER flips the edge broken via Kahn not the LLM; a same-node collision resolves to the user intent + an H-49 rollback row; queue-semantics arm rejected; ADD-NODE canvas attempt refused; dispatch reaches canvas_redirect.rs via the production namespace.

unknowns: mid-token preempt needs H-49 turn-grain rollback (Tier-2); agent-control pause/redirect depends on PLANNED J-12 (kill uses the shipped stop path); REORDER with no SYSTEM consequence must be a no-op verdict, not a false breakage.
