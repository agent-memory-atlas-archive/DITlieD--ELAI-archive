# P-9: connector-pack admission format — own the FORMAT not the ecosystem, applied to CAPABILITY (GAP 1, the closed-vocabulary ceiling->moat unit)

STATUS: PLANNED (product surface; executes AFTER the proof spine per A-7 / GATE M-0 / PSR-1 — authored now, not pulled forward). Authored 2026-07-08 from workflow/reasoning/elai-command-center-eval-2026-07-08.md §2/§3-GAP1/§4.1/§5/§7-LANE-A/§8.1.

PROOF_SPINE: none (distribution) — spine-exempt product-surface, same class as O/P. User PSR-2 ack recorded (2026-07-09, "your go is the PSR-2 ack"). Does NOT feed a spine number; it is the breadth-moat that P's distribution phase rides once the spine is proven.

Objective: the O-plan closed stage vocabulary (A3: new kinds ship as code = maintainer ships them) is simultaneously the moat (every wire is provable) and the scaling ceiling (a solo author cannot out-write n8n's ~1000-integration long tail — training-unsearched, KD-1-tagged, verify before external use). The corpus already invented the correct escape ONCE — M-6, own the FORMAT not the ecosystem — but applied it only to verification. P-9 applies M-6 to CAPABILITY: a signed, versioned connector/stage-PACK format, declarative where possible + code where necessary, mounted through the EXISTING admission family (L-12 skill door + skillspector + J-102 config-ingress + N-6 scanner registry + taint tiers). The vocabulary stays CLOSED (every mounted pack is typed, gated, journaled, replayable) while the connector surface becomes UNBOUNDED. The load-bearing design bit (§2): open-node vocabulary buys breadth and forfeits receipts; a pack format buys breadth AND keeps receipts.

## Context
The market converged on hybrid (visual wiring + code escape hatch); ELAI already IS that hybrid (O-4 canvas + code stage kinds + A-5 data config). The missing leg is third-party/self breadth without forfeiting the trust core. Key mechanism (§5): ONE generic `http_call` stage kind parameterized by a DECLARATIVE ConnectorSpec (endpoint map, auth kind, pagination, response schema-map -> PortSchema) — the spec is DATA (admission-gated), never free-form code, so ~80% of connectors (HTTP+auth+pagination+schema-map) are a DATA DIFF of the first pack, not a build (§8.1, the biggest single second-node win). Genuinely-new behavior takes the code-pack lane (B-37 RuntimeAdapter closed-trait precedent).

## Existing state (corpus coverage — the substrate P-9 composes, does NOT fork)
- P-2 (M-5 conformance) makes a third-party tool a mountable node — but ONLY for tools that already emit ELAI-verifiable evidence. P-9 is the generalization to arbitrary connectors.
- M-6 own-the-format + versioned schema golden-lock — the pattern, proven on verification; P-9 reuses the golden-lock idiom.
- L-12 skill door + /skill-audit skillspector + J-102 config-ingress + N-6 scanner registry + G-1 taint tiers — the admission family (do NOT build a new gate; route packs through these).
- B-37 RuntimeAdapter — the closed-adapter-trait + new-adapters-ship-as-packs precedent (§5.d, LANE B).
- O A3 closed stage vocabulary + A-5 data-not-code two-masters doctrine — the invariants P-9 must not break.

## Reads (mandatory before coding)
1. workflow/reasoning/elai-command-center-eval-2026-07-08.md §3-GAP1/§4.1/§5/§7/§8.1 (the design).
2. The O-plan A3 closed-vocabulary rule + the O-1 fetch->classify pipeline (the http_call stage's home).
3. M-6 (own-the-format + golden-lock idiom) + P-2 M-5 conformance (the mountable-node precedent).
4. L-12 / .claude/tools/skill-audit.py (skillspector) / J-102 config-ingress / N-6 scanner registry (the admission seam).
5. B-37 RuntimeAdapter trait (the LANE-B code-pack precedent).

## RECON_NEEDED (settle at execution, exact check each)
- RECON_NEEDED-1 (ConnectorSpec shape): what does a declarative connector spec minimally need to cover the HTTP long tail? SETTLING CHECK: source-ground 2-3 real n8n declarative node definitions (WebFetch n8n node JSON) + the OpenAPI auth/pagination taxonomy; enumerate the auth-kind + pagination enums from real APIs, do NOT invent. Anti-slopsquat any referenced lib.
- RECON_NEEDED-2 (admission reuse vs extend): does the L-12/skillspector/J-102/N-6 family accept a non-skill pack as-is, or does it need a pack-type discriminator? SETTLING CHECK: read the skillspector wrapper + N-6 registry entry shape; if a connector-pack needs a new scanner category, that is a scoped extension, not a new gate.
- RECON_NEEDED-3 (PortSchema binding): how does a response schema-map bind to the existing PortSchema/threaded-contract system? SETTLING CHECK: grep the PortSchema / SCHEMA_REQUIRED rule predicate; the response map must produce a human-frozen PortSchema the generator satisfies, never invents.

## FORKS
- FORK-1 (http_call generality vs per-connector kinds): TRIGGER = does the declarative ConnectorSpec cover the target API's shape (auth+pagination+schema-map expressible as data)? Yes -> LANE A (data pack, no engine code). No (genuinely-new transport/behavior) -> LANE B (code pack via B-37 RuntimeAdapter trait). Default: LANE A; LANE B only on a RECON-proven gap.
- FORK-2 (pack signing key): TRIGGER = is there an existing ELAI signing key for RRA/.rra artifacts? Reuse it for pack signatures. If none applies to packs, a scoped key is a RECON item, not an invention.

## ABORT
- ABORT-1: if a pack format would require OPENING the stage vocabulary (a generic/custom runtime node, the n8n model O A3 bans) -> STOP + handoff. The whole point is closed-vocab + open-endpoint-surface; an open node kind forfeits receipts and is the vetoed design.
- ABORT-2: if the declarative ConnectorSpec would carry executable code (eval, a script field) -> STOP. The spec is DATA; code goes through LANE B's typed trait, admission-scanned. A code-in-data field is the injection surface the admission family exists to stop.
- ABORT-3: if admission would need a NEW ungated path (bypassing L-12/skillspector/N-6) -> STOP; reuse the frozen admission family or it is not P-9.

## Tasks (MOVES — each EXPECTED_OBSERVATION + ON_FAIL)
MOVE 1 — the ConnectorSpec schema (human-frozen, SCHEMA_REQUIRED): a JSON-Schema contract for {endpoints map, auth_kind enum, pagination enum, response schema-map -> PortSchema}, at the root of the pack's dependency DAG (human-authored schemas as a foundation / A-5).
  EXPECTED_OBSERVATION: a versioned connector-spec.schema.json exists; a fixture spec validates; a malformed spec (code-in-data field) is REJECTED by the schema (ABORT-2 guard is structural).
  ON_FAIL: the schema accepts a free-form/code field -> cause = under-constrained schema; counter-move = tighten to a closed field set, add the negative fixture.
MOVE 2 — the generic `http_call` stage kind: executes a ConnectorSpec (auth -> paginated fetch -> taint Untrusted at ingest (G-1) -> response schema-map -> PortSchema-validated emit). One kind, unbounded endpoints. EGRESS IS PINNED: `http_call` may only connect to hosts in the mounted spec's declared endpoints map (composed with the G-3 host-side allowlist/IP-pin); the declared-host set is part of the SIGNED pack so admission reviews it — a request to an undeclared host is REFUSED pre-connect. This closes the pure-data exfil hole: a spec whose URL templates interpolate context/journal bytes toward an attacker host is data (passes ABORT-2) but cannot connect.
  EXPECTED_OBSERVATION: `http_call` against a fixture ConnectorSpec + a local mock server returns a PortSchema-valid, taint-tagged row; the bytes carry Untrusted provenance through to the sink; a request templated at a host NOT in the declared endpoints map is REFUSED pre-connect with a journaled refusal row (the exfil negative arm).
  ON_FAIL: emitted bytes untainted -> cause = ingest not routed through G-1; counter-move = taint at the ingest boundary (§5 normalize-at-boundary), never downstream. Undeclared-host request connects -> cause = egress checked post-template instead of pre-connect; counter-move = resolve+pin the host set at mount time, enforce at the connector, never at the stage's discretion.
MOVE 3 — the pack format + admission: a signed, versioned pack (spec + PortSchema + N-6 tier + manifest) mounted through L-12/skillspector/J-102; M-6-style versioned schema golden-lock so a pack-format change is a reviewed golden bump.
  EXPECTED_OBSERVATION: a well-formed pack mounts (a MountedConnector row / event lands); a TAMPERED pack (bad signature / failed skillspector scan) REFUSES to mount (the negative arm).
  ON_FAIL: a tampered pack mounts -> cause = admission not fail-closed; counter-move = missing/failed scan = REFUSE (R1 fail-closed), never mount-with-warning.
MOVE 4 — LANE A scaffold + LANE B trait: `elai connector new` scaffolds the declarative pack template (ELAI fills endpoints/auth/schema-map with anti-slopsquat + source-grounding recon on the API docs, §7 LANE A); LANE B = a code pack via the B-37 RuntimeAdapter closed trait for genuinely-new behavior.
  EXPECTED_OBSERVATION: `elai connector new <name>` writes a valid scaffold that passes MOVE-1 schema validation; a LANE-B code pack implements the closed trait + mounts through the same gate.
  ON_FAIL: the scaffold emits an invalid spec -> cause = template drift from the schema; counter-move = generate the scaffold FROM the schema (single source).
MOVE 5 — decisions doc workflow/reasoning/P-9-decisions.md (the ConnectorSpec enum choices + their API grounding, the LANE-A/B boundary, the golden-lock bump procedure).
  EXPECTED_OBSERVATION: the doc exists and cites the real APIs the auth/pagination enums were grounded against.
  ON_FAIL: enums ungrounded (training-memory) -> cause = skipped RECON-1; counter-move = source-ground before locking (KD-1).

## PRODUCTION_HOOK (named at authoring; confirmed at execution pre-flight)
- Producer `http_call` stage kind -> consumer the O-1 fetch->classify pipeline / the FSM stage dispatcher (the real workflow-execution entrypoint), reached when a mounted ConnectorSpec runs in a TaskSpec. Producer the pack-mount path -> consumer the L-12/N-6 admission registry (a MountedConnector SSB row is the off-agent artifact). Exact file+fn seams are a RECON item resolved by the integration-explorer at execution pre-flight (this plan is pre-spine; seams are named then against live code).

## INTEGRATION_SMOKE (R10 — at execution)
An independent smoke drives the REAL entrypoint on a HARD input: mount a fixture connector pack, run a TaskSpec that fires `http_call` against a local mock server behind auth+pagination, and assert (off-agent) a taint-tagged PortSchema-valid SSB row lands AND a tampered pack REFUSES to mount. Toy happy-path does not satisfy it.

## WIRING_CONTRACT
produces: connector-spec.schema.json (frozen) + the `http_call` stage kind + the pack format/manifest + `elai connector new` + the LANE-B trait.
consumes: O-1 fetch pipeline + G-1 taint + PortSchema/SCHEMA_REQUIRED + L-12/skillspector/J-102/N-6 admission + M-6 golden-lock + B-37 RuntimeAdapter.
reachable_from: `elai connector new` [CLI] + a mounted ConnectorSpec executing in a TaskSpec through the FSM stage dispatcher [production workflow run] + the admission registry mount path. A test caller never satisfies R7.
stub_ban: NO open/custom runtime node (ABORT-1); NO code field in the declarative spec (ABORT-2); NO new ungated admission path (ABORT-3); NO invented auth/pagination enums (RECON-1); NO mount-with-warning on a failed scan (fail-closed); NO `http_call` egress to a host outside the mounted spec's declared endpoints map (G-3-composed pin, refused pre-connect).
observable_effect: trigger "elai connector new + mount + run a TaskSpec http_call"; expect a MountedConnector + a taint-tagged PortSchema-valid fetch row in SSB; a tampered pack refuses to mount (negative arm).

## DETERMINISTIC_CHECK (at execution)
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-9_connector_pack_admission_format.md --run-trigger
# schema rejects a code-in-data field; http_call taints at ingest; a tampered pack refuses to mount; MountedConnector row lands.
```

## VERIFY (at execution; off-agent pass artifacts)
```bash
# build + the connector-pack contract tests; then the real-entrypoint smoke:
elai connector new demo_api && <mount> && <run a TaskSpec that fires http_call against a local mock>   # asserts a taint-tagged PortSchema row in SSB (off-agent), not stdout
```
PASS = the MountedConnector + fetch rows present in the journal, the tampered-pack refusal observed, and the admission scan (skillspector) recorded — never the executor's stdout (R4).

## ACCEPTANCE (binary; off-agent proof)
- [ ] the stage vocabulary stays CLOSED (no generic/custom runtime node; ABORT-1 held) while the endpoint surface is unbounded via declarative specs.
- [ ] a declarative pack mounts through the EXISTING admission family (L-12/skillspector/N-6); a tampered pack refuses (fail-closed).
- [ ] `http_call` taints fetched bytes Untrusted at ingest (G-1) and validates the response to a human-frozen PortSchema.
- [ ] the pack format is versioned + golden-locked (M-6 idiom); auth/pagination enums are source-grounded (KD-1), not training-memory.
- [ ] egress pinned: a fetch templated at an undeclared host is REFUSED pre-connect (journaled refusal row — the exfil negative arm).
- [ ] OFF-AGENT: MountedConnector + PortSchema-valid fetch SSB rows from a real run; INTEGRATION_SMOKE green.

## REDTEAM (plan-standard point 7)
DEFERRED to the execution pre-flight. This plan is authored pre-spine (P executes after the proof spine, A-7/GATE M-0); per plan-standard the wargame red-team + the plan-gate 10-point verdict run when something first tries to EXECUTE it (refine-before-execute), at which point an adversarial-auditor (mode=plan, non-author) attacks it and its surviving attack + patches are appended here. Known attack surfaces to aim at then: the declarative-spec injection surface (code-in-data), the admission-bypass path, the closed-vocabulary invariant under a "just one custom node" pressure, and the pure-data exfil surface (a spec interpolating context/journal bytes into an undeclared-host URL — MOVE 2's pre-connect pin is the patch to attack).

## Open questions / risks
- The ~1000-integration n8n figure is training-unsearched (KD-1) — do not cite externally without verification; it motivates the moat, it is not a claim.
- The declarative-spec expressiveness ceiling (what fraction of real APIs LANE A actually covers) is a RECON-1 empirical, not an assumption.

## Next
P-9 is the breadth-moat leg. P-10 (ingress + credentials) supplies the auth substrate `http_call` needs; P-11 (workflow-from-goal) is the demo that converts an n8n user. All three execute after the proof spine.

End of P-9.
