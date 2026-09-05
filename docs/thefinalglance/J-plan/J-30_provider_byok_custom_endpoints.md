# J-30: provider byok — per-provider key store, declarative custom endpoints, the configure flow with a live test call, and the turn-scoped backend lock

STATUS: PLANNED (WAVE GSW-2; .elai_cc runtime + CLI + small panel extension; EXTENSION of B-37/I-50 — keys and endpoints ONLY: zero OAuth (I-50's), zero new wire formats (I-49's), zero dynamic catalogs (ledger))

- Track J (repo-steal extension) | lever REACH + STEERING | grounding STRONG (B-37 WIRED live;
  goose source via goose-main-INDEX.md, MEDIUM, ideas only; reference digest WEAK) | nature
  EXTENSION | effort M | WAVE GSW-2

- Premise: ELAI's provider abstraction is SHIPPED (B-37 RuntimeAdapter trait, 11 backends,
  PerRoleRuntime, factory — feature map [WIRED]) but BYOK ergonomics are absent: keys are env
  vars only, there is no per-backend base-url generalization beyond the sidecar's
  ANTHROPIC_BASE_URL, no way to add an anthropic-compat/openai-compat endpoint without Rust
  code, no configure flow that proves a key works before saving it, and no explicit
  turn-scoped backend lock. Goose's config-first secrets + declarative provider overlay + test
  -call wizard are the pattern source; the digest confirms the product shape (per-provider key,
  base-url override, active-provider lock). Everything here EXTENDS B-37/I-50 — the ledger
  rulings bind: device-code OAuth NOT WORTH PORTING (OAuth stays I-50's AuthProvider), dynamic
  model/provider catalogs NOT WORTH PORTING (I-51 static models.toml canonical), wire-format
  shims are I-49's scope. Steal file: workflow/reasoning/steal-goose-factory-2026-06-12.md
  (S-17..S-29, S-33).

## Steal/build
T1 KEY STORE (S-19). Two-tier per-provider key storage with the goose lookup order, ELAI-native:
  env var (exact name, untouched precedence) -> key store file. Default store = a 0600-mode file
  under the ELAI config dir (Unix perms; Windows = restrictive ACL best-effort + a documented
  caveat — this repo's primary host is Windows); atomic write (tmp + rename + fsync — goose
  config/base.rs:572-608 hygiene). OS-keyring backend = an OPTIONAL upgrade behind explicit user
  dep confirmation (keyring crate; anti-slopsquat name/version/maintenance verification AT EDIT,
  never assumed) — the file tier ships first with zero new deps. Key material NEVER lands in
  SSB/journal/logs (I-50 doctrine, G-9).
T2 DECLARATIVE CUSTOM ENDPOINTS (S-18). custom_providers.toml: {name, engine =
  anthropic-compat | openai-compat, base_url, api_key_ref (env name or store key — never the key
  itself), models[], headers?, timeout_s?}. Loaded at startup, registered as dynamic backends
  through the EXISTING B-37 factory/resolve path — covers vLLM/LiteLLM-proxy/self-hosted
  endpoints with zero new Rust per endpoint. The openai-compat engine carries the
  field-probe note (S-33): unsupported request fields (e.g. stream options) are detected/dropped
  per endpoint, never silently timing out. TOML not YAML (house style; sits beside models.toml).
T3 BASE-URL OVERRIDE GENERALIZATION (S-20). Per-backend base_url via config + env override for
  the BUILT-IN backends too; ANTHROPIC_BASE_URL keeps working unchanged (back-compat with the
  sidecar tunnel setup).
T4 PER-BACKEND CONFIG DESCRIPTOR (S-21). A declarative descriptor per backend (required keys,
  optional keys, base-url support, secret flags) consumed by T5's CLI flow and the panel —
  masked entry for secret-flagged values (goose DefaultProviderSetupForm masking idea). NO
  oauth_flow / device_code_flow arms (ledger; I-50).
T5 CONFIGURE FLOW WITH A LIVE TEST CALL (S-22). CLI verb (extend the existing configure surface
  if one exists at edit, else `elai providers configure`): select backend -> masked key entry ->
  optional base-url -> LIVE minimal test call against the endpoint -> atomic save ONLY on
  success. The dashboard panel mirrors the same descriptor-driven flow (thin: form + test
  button + status). On 4xx model-availability errors, suggest models from the I-51 catalog
  (S-29 rider — optional leg).
  UI/UX INSPIRATION (build-time, panel + first-run onboarding): OpenCode's provider-connect
  surface is the closest existing match to THIS exact shape — a Providers list, a per-provider
  Connect action, an env-detected "Environment" badge (= T1 env-var precedence), and a
  live-verify status pill (= T5 test call / T6 verified_at). When the LLM builds this panel +
  onboarding it inspires itself on OpenCode's source (github.com/sst/opencode ->
  anomalyco/opencode, MIT, TypeScript) for LAYOUT + INTERACTION ONLY: clean-room ELAI-native
  code + names (same posture as the goose steal), index-before-EXECUTING per the INDEX protocol,
  NO OAuth / device-code / credential-replay code copied (that is I-50's lane; ledger
  DO-NOT-COPY on OAuth-replay stands).
T6 INVENTORY STATUS (S-23). Per-backend inventory: configured? (descriptor-satisfied),
  verified_at (last successful test call), models (from models.toml / custom_providers.toml),
  stale flag (verified_at older than a threshold). Surfaced as a RuntimePanel extension (B-37's
  panel), not a new window.
T7 TURN-SCOPED BACKEND LOCK (S-24). Once RuntimeOrchestrator resolves a backend for a
  turn/sequence, it is LOCKED until the sequence completes — a mid-sequence swap attempt is
  refused + logged (the digest's lock insight: never split one conversation across wire shapes
  without an explicit boundary). Existing role routing (PerRoleRuntime, H-7/E-31) decides
  BETWEEN turns; the EscalateToCloud promotion path remains the sanctioned mid-task escalation
  and is NOT blocked by the lock (it is a sequence boundary by definition).
RIDERS (recorded, not tasks): S-25 Gemini wire shape = optional I-49 rider (user-demand gated);
  S-26 retry/backoff + auth-refresh = I-10/I-50; S-27 dynamic catalog = ALREADY-REJECTED
  (ledger); S-28 mTLS = REJECT (no user story).

## Existing ground truth (grep/audit 2026-06-12; re-grep at edit, W-7)
- B-37 MULTI-RUNTIME BACKEND [WIRED]: RuntimeAdapter trait, 11 backends, PerRoleRuntime,
  resolve_backend, RuntimePanel (feature map; phase-4M). The factory/registration seam is THE
  integration point — J-30 adds entries, never a parallel registry.
- I-50 SUBSCRIPTION AUTH BROKER [PLANNED]: AuthProvider trait wrapping oauth.rs, token material
  never journaled, AuthBrokerEvent. J-30's key store is the plain-API-key sibling; OAuth stays
  I-50's. I-51 MODEL CAPABILITY CATALOG: models.toml over pricing.rs/vram_tier.rs (audit
  confirms pricing.rs is already a view over it).
- I-49 BEDROCK + AZURE LANES [PLANNED]: SigV4 + EventStream, Azure Responses, OpenAI-as-
  Anthropic shim — the wire-format work J-30 must NOT duplicate (anthropic-compat /
  openai-compat engines reuse shapes ELAI already speaks).
- ANTHROPIC_BASE_URL is the live sidecar override (CLAUDE.md remote-sidecar block; phase-3J).
- ssb.rs:3096 SCHEMA_VERSION 234 / action_type_lock.rs:676 TOTAL_VARIANTS 543 (grep this
  session); J-30 = +1 AFTER J-28's bump (sequence the golden-lock edits, re-grep first).
- Goose ground truth (W-3, via INDEX): config/base.rs:369-386/627-658/730-735 two-tier secrets +
  lookup order; declarative_providers.rs:34-100 overlay; base.rs:194-370 metadata/ConfigKey;
  configure.rs:675-812 wizard; custom_requests.rs:1315-1379 inventory; api_client.rs:28-38
  AuthMethod enum (the no-OAuth subset is what T4 mirrors).

## REVISED scope (W-10 tiers)
SHIP NOW Tier-1 (in-process / local-network): T1 store + lookup, T2 toml + registration, T3
  override, T4 descriptor, T6 inventory (fixture clock), T7 lock; T5's test call exercised
  against the LOCAL sidecar endpoint (anthropic-compat — no cloud key, no cost) and a fixture
  HTTP server for openai-compat.
DEFER Tier-2 (network/key-gated, W-10): live verification against real third-party cloud
  endpoints (requires the user's actual keys; run on demand, never in CI).

## Integration
- NEW crates/providers/src/runtime/key_store.rs (~180 LOC: lookup order, atomic write, perms).
- NEW crates/providers/src/runtime/custom_backends.rs (~200 LOC: toml load, validation,
  registration into the B-37 factory; field-probe handling on the openai-compat engine).
- crates/providers/src/runtime/ descriptor + inventory (~120 LOC).
- crates/orchestrator/src/runtime_orchestrator.rs (~40 LOC: turn-scoped lock at the resolve
  seam).
- crates/cli configure verb (~150 LOC, descriptor-driven, masked input).
- RuntimePanel extension + stub.ts/types.ts (+ the inventory/test Tauri commands, ~120 LOC).
- ssb.rs + action_type_lock.rs: +ProviderByokEvent (full 6-part discipline).
- frontend_pair: the RuntimePanel extension + its Tauri commands (cross_lang_diff applies).

## WIRING_CONTRACT
produces_symbols:
- key_store (env -> file lookup, atomic 0600/ACL write, optional keyring upgrade path)
- custom_providers.toml loader + dynamic backend registration on the B-37 factory
- per-backend config descriptor + inventory status (configured/verified_at/models/stale)
- configure flow (CLI + panel) with live test call gating the save
- turn-scoped backend lock in RuntimeOrchestrator
- ProviderByokEvent SSB action_type (Kind: KeyStored / EndpointVerified /
  CustomBackendRegistered; payload carries backend name + ok flags + ts — NEVER key material)
consumes_symbols:
- B-37 RuntimeAdapter trait + factory/resolve_backend + PerRoleRuntime (the registration and
  lock seams — never re-implemented)
- I-51 ModelCatalog/models.toml (inventory models + 4xx suggestions)
- existing config layering (args.rs surface) for env precedence
- EscalateToCloud path (the lock's sanctioned boundary — consumed, not modified)
reachable_from:
- `elai providers configure` (or the extended existing configure verb)  [PRODUCTION CLI]
- startup custom_providers.toml load -> B-37 factory registration -> any task routed to the
  custom backend  [PRODUCTION runtime]
- RuntimePanel inventory/test commands  [PRODUCTION dashboard]
- contracts/-only callers never satisfy this (R7)
stub_ban:
- NO key material in SSB rows, logs, error messages, or the toml (api_key_ref only)
- NO OAuth / device-code arms (I-50's scope; ledger NOT-WORTH-PORTING stands)
- NO new wire-format adapters (I-49's scope; engines reuse spoken shapes only)
- NO dynamic remote provider/model catalog fetch (ledger; static toml only)
- NO save-without-successful-test in the configure flow (fail-closed configure)
- NO silent vendor-proxy fallback (the digest's no-key proxy concept is NOT adopted)
- NO .unwrap()/.expect() on prod paths; NO blake2/sha2; NO new env gate
frontend_pair:
  RuntimePanel extension + inventory/test Tauri commands; stub.ts/types.ts in-unit;
  cross_lang_diff clean. Justified.
observable_effect:
  trigger: "`elai providers configure` pointed at the LOCAL sidecar endpoint (anthropic-compat),
    then a custom openai-compat fixture endpoint added via custom_providers.toml and one task
    routed to it"
  expect_ssb: "EndpointVerified row on the successful test call; KeyStored row on save (no key
    bytes anywhere in the payload); CustomBackendRegistered row at startup registration; the
    routed task's existing RuntimeBackendSelected/J-28 rows name the custom backend"
  expect_artifact: "config write is atomic + restrictive-permissioned; the custom backend is
    resolvable by name in PerRoleRuntime. NEGATIVE (R2): a dead endpoint FAILS the configure flow
    and nothing is saved; a contract test greps the journal/store artifacts for the fixture key
    bytes and FAILS on any hit; a mid-sequence backend swap attempt is refused + logged (T7);
    a custom_providers.toml entry with a missing api_key_ref fails loudly at load, never a
    silent skip."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-30_provider_byok_custom_endpoints.md --run-trigger
rg "ProviderByokEvent" .elai_cc/crates/orchestrator/src/ssb.rs                    # variant registered
rg "custom_providers" .elai_cc/crates/providers/src -l                            # loader exists
rg -i "oauth|device_code" .elai_cc/crates/providers/src/runtime/key_store.rs      # MUST be 0 (I-50 boundary)
python .claude/tools/cross_lang_diff.py                                           # panel pair clean
test -f workflow/reasoning/J-30-decisions.md
```

## Grounding
- ELAI: B-37/I-49/I-50/I-51 entries (feature map, 2026-06-12) + this session's live greps
  (ssb.rs/action_type_lock.rs baseline); integration anchors PLAN-asserted where map-only —
  executor re-greps the factory/resolve seam at edit (W-7).
- Goose: docs/.source/goose-main clone via goose-main-INDEX.md theme B (Apache-2.0, MEDIUM,
  ideas only; clean-room names: key_store / custom_providers.toml / ProviderByokEvent are
  ELAI-native). Reference digest section 3 (WEAK; confirms the product shape only).
- OpenCode (UI/UX reference for the T5/T6 panel + first-run onboarding ONLY): cloned to
  docs/.source/opencode (github.com/sst/opencode -> anomalyco/opencode, MIT, TypeScript). The
  screenshot-exact surface lives at packages/app/src/components/settings-providers.tsx +
  settings-v2/providers.tsx (the Providers list), dialog-connect-provider.tsx (per-provider
  Connect), dialog-custom-provider.tsx (BYOK / custom-endpoint form = T2), dialog-select-provider.tsx
  (picker), and hooks/use-providers.ts + provider-catalog.ts (env-detection = the "Environment"
  badge / T1 precedence). Backend auth logic for context: packages/opencode/src/{auth,provider} +
  packages/core/src/oauth (the OAuth flow is I-50's reference, NOT copied here). LAYOUT +
  INTERACTION ideas ONLY; clean-room ELAI-native code + names; MIT permits reuse but the steal
  posture stays ideas-not-code. INDEX-before-EXECUTING: the UI executor writes an opencode-INDEX.md
  (goose-INDEX protocol) over that packages/app subtree before building the panel.
- Ledger rulings honored: device-code OAuth NOT-WORTH-PORTING; models.dev-style dynamic
  catalogs NOT-WORTH-PORTING; gajae wholesale port REJECTED (I-49 carries the 3 shapes).
- SSB baseline: LIVE 234/543 (grep 2026-06-12); J-30 = +1 sequenced AFTER J-28 (W-9 re-grep at
  edit; golden-lock 9-tuple lockstep, one variant at a time).
- New dependency: NONE in Tier-1 (keyring crate ONLY behind explicit user confirmation +
  anti-slopsquat verification at edit).
- Doctrine: W-1/W-9/W-10, GD-1 (B-37/I-50/I-51 composed, never restated), GD-5 (Windows ACL
  caveat named), R7, G-9, J-22 (field-shape snapshot on the new payload if landed).

## COMPOSES-WITH
B-37 (the trait/factory/panel this extends), I-50 (key store = plain-key sibling; AuthProvider
untouched), I-51 (catalog feeds inventory + suggestions), I-49 (wire formats stay there; a
custom anthropic-compat endpoint is also I-49's fixture-harness consumer), J-19 (a vLLM
endpoint is the natural first custom_providers.toml interop test), H-7/E-31 (routing decides
between turns; the lock binds within one), J-28 (custom backends inherit the usage ledger for
free — emission sits at the orchestrator seams), E-33 config surfaces. Restates none.

## Open questions / risks
- Windows key-file protection is weaker than Unix 0600 — best-effort ACL + documented caveat
  (GD-5); the keyring upgrade is the real fix and stays user-gated.
- Whether a configure verb already exists on the CLI surface (E-33 refactor): check at edit;
  extend, never duplicate.
- Dynamic registration timing vs the factory's static registry shape: if the B-37 factory
  proves compile-time-only at edit, T2 lands as a startup-time registry layer ON the factory's
  resolve path (the seam stays B-37's; no parallel registry either way).
- The lock (T7) vs streaming retries within one turn: a provider retry is the SAME backend by
  definition; only cross-backend swaps are refused. State this in the lock's doc comment.

## Next
J-30 closes the BYOK cluster: keys, custom endpoints, verified configuration, and lock
semantics on the shipped B-37 spine. With J-28/J-29 it completes the goose+reference steal run:
the token window has its data, its surfaces, and its provider freedom. I-49/I-50 remain the
wire-format and OAuth lanes; J-19's vLLM backend becomes a one-stanza toml entry for users once
both land.

End of J-30.
