# P-10: ingress adapter family + credential vault — normalize any external shape to a taint-tagged SsbEvent, and give the connector surface an auth plane (GAP 2)

STATUS: PLANNED (product surface; executes AFTER the proof spine per A-7 / GATE M-0 / PSR-1 — authored now, not pulled forward). Authored 2026-07-08 from workflow/reasoning/elai-command-center-eval-2026-07-08.md §3-GAP2/§4.2/§5.

PROOF_SPINE: none (distribution) — spine-exempt product-surface (O/P class). User PSR-2 ack recorded (2026-07-09). Supplies the ingress + credential substrate P-9's `http_call` and the P-plan command-center vision need; does not feed a spine number.

Objective: GAP 2 — O-2's trigger enum is Cron | SsbEvent | Manual (no inbound event ingress) and credentials exist only as scatter (G-3 mentions a credential broker in passing; M-1 files tokens under persona data). The design point (§5): DO NOT grow the trigger enum per protocol. Add an INGRESS ADAPTER family whose only job is to convert any external shape into a taint-tagged (G-1), journaled SsbEvent row — then every downstream mechanism (O-2 SsbEvent trigger, G-2 replay, provenance, privacy lattice) works UNCHANGED. Plus a first-class CredentialVault on O-2's contract so the connector surface (P-9) has an auth plane instead of scattered tokens.

## Context
External connections reduce to FOUR shapes; the invariant is NORMALIZE AT THE BOUNDARY into one contract {external bytes -> taint Untrusted (G-1) -> SsbEvent row}:
  a. OUTBOUND PULL (call an API on schedule/demand): already covered by O-2 Cron + a fetch stage (O-1) — needs ONLY the credential vault for auth.
  b. INBOUND PUSH (webhook/callback): one WebhookIngress adapter (a local HTTP listener on the I-1 supervisor plane; bytes Untrusted at ingest; emits SsbEvent). K-14 TZ rules keep the wall-clock/external event JOURNALED-AS-INPUT so G-2 replay stays faithful (the replayable-async differentiator; G3-G6 async primitives are the substrate).
  c. STREAMING/PERSISTENT (websocket, MQ, IMAP idle, file watch): a long-lived listener daemon per adapter, same normalization contract. Precedent in-corpus: C-17 FileWatcherService (watch -> event) is exactly this shape for the filesystem.
  d. GENUINELY OTHER (a device, a local DB, a CLI tool): the B-37 RuntimeAdapter precedent — a closed adapter trait; new adapters ship as packs through P-9's admission format.

## Existing state (substrate P-10 composes, does NOT fork)
- O-2 trigger contract (Cron | SsbEvent | Manual) — the SsbEvent trigger is the join point; the adapter family feeds it, the enum does not grow.
- G-1 taint tiers (Untrusted at ingest) + G-2 RRA replay + K-14 TZ/wall-clock-as-input + G3-G6 async primitives — the faithful-replay substrate for async ingress.
- I-1 supervisor plane — where a WebhookIngress HTTP listener lives (not in the pipeline hot path).
- C-17 FileWatcherService — the persistent-listener precedent (shape c).
- B-37 RuntimeAdapter + P-9 pack admission — the closed-adapter-trait + ship-as-pack path (shape d).
- G-3 (credential broker in passing) + M-1 (tokens under persona data) — the scatter P-10 consolidates into CredentialVault (persona-excluded per P-3 export rules).

## Reads (mandatory before coding)
1. workflow/reasoning/elai-command-center-eval-2026-07-08.md §3-GAP2/§4.2/§5.
2. O-2 trigger contract (the SsbEvent join point) + O-1 fetch pipeline (shape a).
3. G-1 taint tiers + K-14 TZ rules + G-2 RRA replay (the async-faithful-replay substrate).
4. I-1 supervisor plane (the listener host) + C-17 FileWatcherService (the persistent-listener precedent).
5. G-3 credential broker + M-1 persona-token handling + P-3 export exclusion rules (the CredentialVault predecessors).

## RECON_NEEDED
- RECON_NEEDED-1 (SsbEvent row shape for external ingress): what fields does an SsbEvent trigger row need so an adapter-produced event is indistinguishable-downstream from a native one AND carries its provenance? SETTLING CHECK: read the O-2 SsbEvent payload + the trigger match logic; the adapter output must satisfy it exactly (SCHEMA_REQUIRED), never a parallel shape.
- RECON_NEEDED-2 (replay faithfulness for async): does K-14 already journal an external wall-clock event as a replay input, or does the webhook path need a new "recorded external event" leg? SETTLING CHECK: read K-14 TZ rules + G-2 replay; confirm an inbound push at time T replays deterministically (the event is data-in-journal, not a live re-fetch).
- RECON_NEEDED-3 (CredentialVault at-rest + taint): how are secrets stored at rest, and do credential bytes get a taint tier (never journaled in cleartext, never exported P-3)? SETTLING CHECK: read G-3/M-1 + the privacy lattice (P-6); the vault must never write a secret into the SSB payload or an RRA/export.
- RECON_NEEDED-4 (credential ACQUISITION, not just storage): v1 = manual paste into the vault (`elai cred add`). OAuth flows (device-code vs localhost-redirect, refresh-token rotation) are a DECLARED follow-on, not silence: SETTLING CHECK at execution = source-ground the target providers' current OAuth docs (WebFetch, anti-stale — never train-memory a token endpoint); if any P-9 connector needs OAuth before the follow-on lands, that connector is BLOCKED on it, not hand-rolled inline.

## FORKS
- FORK-1 (per-shape adapter routing): TRIGGER = the external connection's shape. pull -> O-2 Cron + fetch + vault (no new adapter); push -> WebhookIngress on I-1; streaming -> a persistent listener daemon (C-17 shape); other -> a B-37/P-9 pack. Deterministic by shape, no judgment.
- FORK-2 (vault backend): TRIGGER = is there an OS keychain/secret store available, or file-at-rest only? Prefer OS keychain; file-at-rest is the fallback with an explicit at-rest-encryption RECON. Never plaintext.

## ABORT
- ABORT-1: if the design would GROW the O-2 trigger enum per protocol (a WebhookTrigger, WebsocketTrigger, ...) -> STOP. The whole move is: adapters normalize to the existing SsbEvent trigger; N new trigger kinds is the anti-pattern §5 rejects.
- ABORT-2: if an inbound push cannot be journaled-as-input for faithful replay (RECON-2 fails) -> STOP + handoff; a live-re-fetch-on-replay breaks G-2 determinism.
- ABORT-3: if a credential would land in cleartext in any SSB payload, RRA, or P-3 export -> STOP; the vault's non-leak invariant is load-bearing (privacy lattice P-6).

## Tasks (MOVES — each EXPECTED_OBSERVATION + ON_FAIL)
MOVE 1 — the IngressAdapter trait + the normalization contract: a closed trait whose one job is {external bytes -> taint Untrusted (G-1) -> a SCHEMA_REQUIRED SsbEvent row}. The contract is the same for all four shapes.
  EXPECTED_OBSERVATION: a fixture adapter converts a synthetic external payload into a taint-tagged SsbEvent that the O-2 trigger matcher accepts as a native trigger; the O-2 enum is unchanged.
  ON_FAIL: downstream treats the adapter event differently from a native SsbEvent -> cause = a parallel row shape; counter-move = satisfy the exact O-2 SsbEvent contract (RECON-1), one shape.
MOVE 2 — WebhookIngress adapter (shape b) on the I-1 supervisor plane: a local HTTP listener; bytes Untrusted at ingest; K-14 journals the external event as a replay input. INBOUND AUTH IS MANDATORY: every registered hook carries a per-hook shared secret; inbound posts are signature-verified (HMAC over the body, the GitHub-webhook idiom — exact scheme is a RECON against the real providers' signing docs) BEFORE any byte is parsed; a missing/failed signature is REFUSED with a journaled refusal row and NO SsbEvent — taint protects downstream, but an unauthenticated listener still lets any local process forge trigger events, and that hole closes here, not downstream.
  EXPECTED_OBSERVATION: a correctly-signed POST to the local listener emits a taint-tagged SsbEvent; an unsigned/badly-signed POST is REFUSED (refusal row journaled, no trigger fires — the forged-ingress negative arm); a G-2 replay of that session reproduces the event from the journal WITHOUT a live re-fetch (faithful async replay).
  ON_FAIL: replay re-fetches / diverges -> cause = event not journaled-as-input; counter-move = record the inbound event as data (K-14), replay reads it (ABORT-2 guard). A forged post fires a trigger -> cause = verification after parse/emit; counter-move = verify-before-parse at the listener edge, secret drawn from the MOVE-3 vault.
MOVE 3 — CredentialVault: a first-class credential plane (OS-keychain preferred, encrypted-file fallback); secrets NEVER in an SSB payload / RRA / P-3 export; `http_call` (P-9) and the fetch stage draw auth from it.
  EXPECTED_OBSERVATION: a stored credential is usable by a fetch/`http_call` run; grepping the SSB journal + any RRA + a P-3 export for the secret value finds NOTHING (the non-leak negative arm).
  ON_FAIL: the secret appears in a journal/export -> cause = credential threaded through a journaled field; counter-move = the vault hands a handle, the fetch resolves it at call time, never persisting the value (ABORT-3).
MOVE 4 — the persistent-listener daemon shape (c) reusing the C-17 FileWatcherService pattern + the shape-d pack path (B-37/P-9); a registry of active adapters on the I-1 plane.
  EXPECTED_OBSERVATION: a file-watch adapter (C-17 shape) emits SsbEvents on change; a shape-d adapter mounts as a P-9 pack through admission.
  ON_FAIL: a listener leaks into the pipeline hot path / an unmounted adapter fires -> cause = listener not isolated to the supervisor plane; counter-move = adapters live on I-1, gated by the adapter registry.
MOVE 5 — decisions doc workflow/reasoning/P-10-decisions.md (the four-shape routing, the vault backend + at-rest choice, the replay-as-input proof).
  EXPECTED_OBSERVATION: the doc exists and records the vault at-rest mechanism + the K-14 replay-faithfulness confirmation.
  ON_FAIL: the replay/at-rest claims are asserted not grounded -> counter-move = ground against K-14 + the keychain API (KD-1) before locking.

## PRODUCTION_HOOK (named now; confirmed at execution pre-flight)
- Producer the IngressAdapter trait + WebhookIngress/FileWatch adapters -> consumer the O-2 SsbEvent trigger matcher (a trigger firing on an adapter-emitted SsbEvent is the real entrypoint). Producer CredentialVault -> consumer the fetch stage / P-9 `http_call` auth path. Exact file+fn seams (O-2 matcher, the I-1 listener host, the fetch auth site) are resolved by the integration-explorer at execution pre-flight against live code.

## INTEGRATION_SMOKE (R10 — at execution)
Independent, real entrypoint, hard input: POST to a live WebhookIngress listener -> assert a taint-tagged SsbEvent lands AND an O-2 TaskSpec triggered by it runs; then G-2-replay the session and assert the inbound event reproduces from the journal (no live re-fetch); then run a `http_call` drawing a credential from the vault and grep the journal/RRA for the secret (must be absent). Toy happy-path does not satisfy it.

## WIRING_CONTRACT
produces: the IngressAdapter closed trait + WebhookIngress + a file-watch adapter + the adapter registry + CredentialVault.
consumes: O-2 SsbEvent trigger + G-1 taint + K-14 TZ/replay + G-2 RRA + I-1 supervisor plane + C-17 FileWatcher + B-37/P-9 packs + G-3/M-1 (credential predecessors) + P-6 privacy lattice.
reachable_from: an inbound push to a live listener firing an O-2 SsbEvent trigger [production] + a fetch/`http_call` drawing vault auth [production] + the adapter registry on I-1. A test caller never satisfies R7.
stub_ban: NO new O-2 trigger kind (ABORT-1); NO live-re-fetch-on-replay (ABORT-2); NO secret in any journal/RRA/export (ABORT-3); NO listener in the pipeline hot path; NO plaintext at-rest; NO unauthenticated inbound ingress (per-hook signature verification before parse, MOVE 2); NO inline hand-rolled OAuth (RECON-4 — acquisition is manual-paste v1, OAuth a declared follow-on).
observable_effect: trigger "POST to WebhookIngress + O-2 trigger + replay + vault-auth fetch"; expect a taint-tagged SsbEvent + a faithful replay from journal + zero secret leakage (negative arm).

## DETERMINISTIC_CHECK (at execution)
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-10_ingress_adapters_credential_vault.md --run-trigger
# webhook POST -> taint-tagged SsbEvent; O-2 trigger fires; replay reproduces from journal; grep secret in journal/RRA/export == 0 hits.
```

## VERIFY (at execution; off-agent pass artifacts)
```bash
# build + ingress/vault contract tests; then the real-entrypoint smoke (webhook -> trigger -> replay -> vault-auth fetch)
# PASS = the SsbEvent + trigger-fire rows in the journal, a byte-faithful replay, and zero secret hits in journal/RRA/export.
```
PASS artifacts are the journal SsbEvent row + the replay equality + the zero-leak grep — never the executor's stdout (R4).

## ACCEPTANCE (binary; off-agent proof)
- [ ] the O-2 trigger enum is UNCHANGED; all four external shapes normalize to a taint-tagged SsbEvent (ABORT-1 held).
- [ ] an inbound push replays faithfully from the journal (K-14/G-2), no live re-fetch (ABORT-2 held).
- [ ] CredentialVault: a secret is usable by a fetch/`http_call` and appears in NO journal/RRA/P-3 export (ABORT-3 held; the negative arm passes).
- [ ] listeners live on the I-1 supervisor plane, gated by the adapter registry (not the hot path).
- [ ] inbound auth: an unsigned/badly-signed post is REFUSED before parse (journaled refusal, no trigger — the forged-ingress negative arm).
- [ ] OFF-AGENT: the SsbEvent + trigger-fire rows + the replay equality + the zero-leak grep from a real run; INTEGRATION_SMOKE green.

## REDTEAM (plan-standard point 7)
DEFERRED to the execution pre-flight (P executes post-spine, A-7/GATE M-0). The non-author adversarial-auditor attacks it when execution is first attempted; surviving attack + patches appended here. Aim then at: the credential-leak surface (does a secret reach ANY journaled/exported field), the replay-faithfulness of a webhook fired mid-run, a listener escaping the supervisor plane into the hot path, and the forged-ingress surface (an unsigned local post firing a TaskSpec — MOVE 2's verify-before-parse is the patch to attack, including replaying a captured signed payload = check for nonce/timestamp in the RECON'd scheme).

## Open questions / risks
- At-rest secret encryption (OS keychain vs encrypted file) is a RECON-3 empirical against the actual host, not an assumption.
- The persistent-listener daemon lifecycle (crash/restart, backpressure on a firehose) needs the same care as any I-1 supervised process; flagged for the C-17-pattern reuse review.

## Next
P-10 gives P-9's `http_call` its auth plane and the command center its inbound-event surface. P-11 (workflow-from-goal) is the front door that composes both into a user-facing "describe it, confirm the graph, run it" flow. All three execute after the proof spine.

End of P-10.
