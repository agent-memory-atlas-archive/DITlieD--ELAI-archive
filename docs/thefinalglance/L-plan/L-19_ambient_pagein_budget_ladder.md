L-19 — AMBIENT PAGE-IN BUDGET LADDER (concrete H-6 read-back policy)
===================================================================
Track: L-T3 CONTEXT-CORE. Binds CC-2 (the recall cards ride the tail, never the prefix).
Nature: BUILD (concretizes H-6's "selective" read-back into a pressure-scaled, cyclic, append-only policy).
Lever: page folded content back in by relevance when it is re-touched, scaling the page-in budget to live
  window pressure, so recall never busts the cache prefix and self-evicts at the next epoch.
Wave: W3 | depends L-16, L-17.
Composes-with: H-6 deterministic observation masking + memory read-back, L-17 freeze tail (where the cards
  ride), I-5 pristine context injector (the contrastive-constraint re-inject), L-16 fold (recall is a tail
  strategy).
Authored 2026-06-30. Reference: CWD foldRecall.ts (the pressure ladder + residency TTL + tier-0/1/2 recall
  signals; cards ride the freeze tail and re-fold at the next epoch, fully cyclic).

BUILD
=====
A pressure ladder that scales the page-in budget by the L-17 window-pressure band:
  healthy   -> full recall cards
  warning   -> card budget halved
  critical  -> one card, quartered
  overflow  -> hints only (a pointer, not the content)
Recall signals (cheapest first): a tool re-touches a path/symbol present in the folded span (tier 0); a new
claim lands on a folded path (tier 1); a K-12 critical atom re-surfaces in the active window (tier 2). Matched
content is injected as APPEND-ONLY cards on the raw tail (CC-2: they ride the L-17 tail, never the frozen
prefix, so the cache stays hot), with a residency TTL in passes; at the next freeze epoch they re-fold away
(cyclic — no permanent growth). The re-injected content is the I-5 compressed contrastive form where one
exists (a <=30-token "avoid X" constraint), not a raw paste, when the trigger is a failed-path re-touch.

COMPOSES-WITH (do NOT re-author)
================================
H-6: this is the concrete policy for H-6's selective read-back; the masking (drop trailing observations) and
the read-back (page relevant content) are the two directions of the same tail strategy. I-5: the contrastive
constraint shape is reused for failed-path recalls. L-17: the freeze tail is where cards live and the pressure
band comes from evaluate_freeze.

INTEGRATION
===========
crates/orchestrator/src/context_projection.rs (the recall tail strategy + the pressure-band budget table) +
H-6 read-back. SSB: prefer-reuse the ContextReduction row (a recall is a negative reduction / a re-inject);
optional +1 only if the dashboard needs a distinct recall event, decide at edit. frontend_pair: the context
gauge shows "recalled N (TTL k)"; or justified N/A.

OBSERVABLE_EFFECT (done-gate)
=============================
Re-touching a folded file pages its content back as an append-only card on the tail; the frozen prefix hash is
unchanged (CC-2). Under critical pressure the card budget is quartered (the band drives it). After the TTL
passes with no re-touch the card re-folds away at the next epoch (cyclic — no permanent growth). NEGATIVE: a
recall NEVER rewrites the frozen prefix (proven by the unchanged prefix hash across a recall turn); a low-
relevance span is not paged in (the cost guard — recall is targeted, never blanket).
