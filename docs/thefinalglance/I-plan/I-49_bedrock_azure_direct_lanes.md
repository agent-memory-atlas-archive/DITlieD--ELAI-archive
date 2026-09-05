# I-49: bedrock + azure direct lanes + cross-protocol shim — the three direct clients no CLI bridge can provide, ported as protocol logic

- Track 20 | lever REACH | grounding STRONG (gap: SL-8) / MEDIUM (gajae semantics, SL-10) | nature BUILD
  (reference port, TS source read first) | effort L | WAVE 4 | network-gated live legs (W-10)
- EXECUTOR NOTE: effort L — this file MUST be expanded to the FULL FORM (template section 3) before
  EXECUTING, re-grepping every anchor (the I-0 rule for L/XL sessions).

- Premise: the STEAL-LIST's headline decision is DO NOT port gajae's provider layer (ELAI's B-37 +
  CLI-bridge + OAuth + local-serving is broader and philosophically better) — steal ONLY the three
  protocol shapes ELAI cannot reach any other way: (1) AWS Bedrock direct (SigV4 signing + binary
  EventStream framing — enterprise accounts have Bedrock, not Anthropic API keys), (2) Azure OpenAI
  Responses API shape (same story for Azure shops), (3) the OpenAI-as-Anthropic cross-protocol shim as
  the PATTERN for ELAI's normalization seam. Grep-verified zero Bedrock/Azure in crates/ (SL-8, HIGH).

## Steal/build
- Reference ports of the protocol logic, never code copies (TS->Rust). SL-10 binding read: read
  amazon-bedrock.ts, aws-sigv4.ts, aws-eventstream.ts, aws-credentials.ts, azure-openai-responses.ts,
  openai-anthropic-shim.ts under docs/.source/gajae-code/packages/ai/src/providers/ before EXECUTING.
- Build: three RuntimeAdapter implementations in crates/providers/src/runtime/ mirroring the existing
  adapter shape (anthropic_api.rs is the pattern file): bedrock.rs (SigV4 request signing + EventStream
  decode), azure_openai.rs (Responses API), and the shim as a shared normalization module (present an
  OpenAI-shape backend through the Anthropic-shape internal interface — the seam genai_adapter already
  gestures at; extend it rather than adding a parallel one if it fits).
- PRE-TASK (anti-slopsquatting + anti-stale-training, MANDATORY): WebSearch current Bedrock
  InvokeModelWithResponseStream / Converse API docs + Azure Responses API docs; verify any candidate
  crate (aws-sigv4, aws-credential-types, aws-smithy-eventstream) via registry + repo + last-publish +
  license; hand-roll if the deps are heavier than the protocol logic. ANY new dep = user approval first.
- EXCLUDED: no GitLab Duo / Kimi / Vertex (A4 — on user demand only); no replacement of any existing
  adapter; no API-key storage changes (I-50's job); no removal of the CLI-bridge philosophy anywhere.

## Existing ground truth
- crates/providers/src/runtime/ holds anthropic_api, claude_code, codex_cli, copilot_cli, gemini_cli,
  kiro_cli, opencode, composer2, genai_adapter, hermes + local serving (STEAL-LIST inventory, HIGH;
  B-37 shipped, memory: project_b37_shipped). oauth.rs is the Claude-plan auth to KEEP (SL-3).
- Zero Bedrock/Azure symbols in crates/ (SL-8).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier 1): the three adapters' protocol logic + a deterministic replay-fixture harness
  (recorded EventStream frames / Responses payloads decoded byte-exactly — the real code path, no
  network) + registry wiring into B-37's adapter enumeration.
- DEFER (Tier 2, separately-gated checkboxes): live Bedrock call (needs AWS creds), live Azure call
  (needs an Azure deployment) — each a one-line checkbox with its env documented.

## Integration
- crates/providers/src/runtime/{bedrock.rs, azure_openai.rs} + the normalization shim module; B-37
  PerRoleRuntime registry (the enumeration + `elai runtime list` surface). frontend_pair: the two new
  backends appear in the existing RuntimePanel list (B-37 surface — rows, no new component).

## WIRING_CONTRACT
- produces_symbols: providers::runtime::bedrock (SigV4 + EventStream decode), providers::runtime::
  azure_openai (Responses shape), the openai->anthropic normalization shim module, replay fixtures.
- consumes_symbols: B-37 RuntimeAdapter trait + registry (the shape mirrored from anthropic_api.rs),
  genai_adapter (extended if the shim fits there), pricing/catalog read (I-51 when it ships).
- reachable_from: `elai runtime list` + the PerRoleRuntime dispatch path (PRODUCTION — selecting the
  backend routes a real request build through the adapter). Contracts-only callers do not satisfy (R7).
- stub_ban: NO mock SigV4 (the fixture harness signs real canonical requests against recorded keys and
  asserts the known-good signature); NO panic on malformed EventStream frames (typed error, fail-closed);
  NO unwrap/expect on prod paths; NO unverified dep (anti-slopsquat protocol first); NO silent fallback
  to another backend on adapter error (surface the error).
- observable_effect:
  trigger: "elai runtime list (shows bedrock + azure-openai); cargo test -p providers --test
  i49_replay_fixtures (the deterministic leg); [Tier-2 checkbox] a live call per backend"
  expect_ssb: "NO new action_type — RouteDecisionEvent / existing runtime rows name the new backend ids
  (GD-1)"
  expect_artifact: "replay fixtures decode byte-exactly (EventStream frames -> the normalized internal
  events; a corrupted frame fixture FAILS with the typed error — R2); the SigV4 fixture reproduces the
  documented known-good signature; live legs each land one real completion when their checkbox runs."

## Grounding
- STEAL-LIST DECISION block + provider steals 1-3 (Tier 2 priority); SL-8 (gap, HIGH), SL-10 (read rule),
  grounding-map WEB VERIFICATION note (protocol docs verified at authoring-time, not from training).
- CLAUDE.md anti-slopsquatting + anti-stale-training (the pre-task), no-new-deps-without-confirmation.

## COMPOSES-WITH
- B-37 RuntimeAdapter/PerRoleRuntime (the frame — mirrored, never forked), genai_adapter (shim seam
  candidate), oauth.rs + CLI bridges (untouched — the KEEP list), I-50 auth broker (credential supply
  when it ships), I-51 catalog (model metadata). Restates none. SSB: NO new action_type (GD-1).

## J-15 AMENDMENT — replay-fixture contract line + DO-NOT-COPY positional matching (source-cited)
Grounding: determinism triage ledger items 6 + 8
(workflow/reasoning/steal-determinism-repos-2026-06-11.md); beyhangl/evalcraft fixture-cassette
shape; dgenio/AnshKanyadi/culpa positional stub-matching finding (agent det-repos-H2/H3
verdicts); J-14 repro-pack fixture format (the `rra export-fixture` spec).

### Fixture contract: J-14 format, content-hash key, NEVER positional
I-49's replay-fixture harness MUST source its fixtures from J-14's `rra export-fixture` output
format (the shared repro-pack fixture contract, GD-1 compose-not-restate). The matching key for
fixture replay is: `(model_id, blake3(message_content), tool_name)` — a content-hash triple,
NEVER a positional index. A fixture matches when the model identity, the blake3 of the request
message content, and the tool name all agree. Positional matching (fixture[0] matches
request[0]) is BRITTLE BY CONSTRUCTION: any reordering, insertion, or deletion in the request
stream silently misaligns fixtures and produces false passes.

### DO-NOT-COPY — positional stub-matching (evalcraft + culpa)
Both evalcraft and culpa use positional fixture matching (fixture cassette index = request
index in the recorded sequence). This is a KNOWN-BRITTLE design (agent verdicts det-repos-H2/H3):
- A new tool call inserted mid-sequence shifts all subsequent indices, silently matching the
  wrong fixture.
- A retried call (same position, different content) matches the original fixture, masking the
  retry.
- A skipped call leaves a dangling fixture that either errors or silently matches the next
  request.
I-49's fixture harness MUST use the content-hash triple above. The positional approach is
recorded as a DO-NOT-COPY exhibit, never a pattern to replicate.

### DO-NOT-COPY — verbatim auth-header storage (culpa)
culpa stores auth headers verbatim in session JSON files. I-49's fixture harness MUST NOT
include auth headers in fixtures. J-14's `rra export-fixture` already scrubs auth before
archive (the counter-design); I-49 fixtures inherit that scrub. A fixture that carries an auth
header is a credential leak, not a replay artifact.

End of I-49.
