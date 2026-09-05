# I-50: subscription auth broker — generalize the Anthropic-only oauth.rs into a multi-provider subscription-auth abstraction

- Track 20 | lever REACH | grounding STRONG (SL-3) / MEDIUM (gajae semantics, SL-10) | nature EXTENSION |
  effort M | WAVE 4

- Premise: STEAL-LIST A2 ("medium effort, structural win"). ELAI's ride-the-subscription philosophy
  (oauth.rs Claude-plan auth + CLI bridges riding vendor logins) is the right bet, but oauth.rs is
  Anthropic-only and each CLI bridge handles auth its own way. gajae's auth-broker/auth-gateway shape
  generalizes it: one broker owning token acquisition / refresh / storage / provider linkage, with
  per-provider flows behind one trait. This is the structural seam I-49's direct lanes and any future
  subscription provider plug into.

## Steal/build
- Steal the broker/gateway/storage decomposition (SL-10 binding read: docs/.source/gajae-code/packages/
  ai/src/auth-broker/ + auth-gateway/ + auth-storage.ts before EXECUTING).
- Build: crates/providers/src/auth_broker.rs — an AuthProvider trait (acquire/refresh/revoke/status) +
  a broker that owns token lifecycle and storage; the EXISTING oauth.rs Anthropic flow becomes the first
  implementation BEHIND the trait with byte-compatible behavior (regression-guarded: same token file
  format, same refresh semantics — existing users must notice nothing); credential storage stays where
  oauth.rs keeps it today (no new storage backend without user confirmation).
- EXCLUDED: no new provider flows in this session (the broker ships with the Anthropic impl only; AWS/
  Azure credential supply for I-49 is a follow-on impl once the live-leg checkboxes are scheduled); no
  keychain/credential-manager dep; no change to CLI-bridge auth (the bridges keep riding vendor CLIs).

## Existing ground truth
- crates/providers/src/oauth.rs SHIPPED, Anthropic-only (SL-3). The KEEP list (STEAL-LIST): oauth.rs +
  claude_code runtime + CLI bridges + genai_adapter + local serving — none of these are replaced.

## REVISED scope (W-10 tiered)
- SHIP NOW: trait + broker + Anthropic impl migration with the byte-compat regression guard.
- DEFER: AWS/Azure credential impls (with I-49 Tier-2), any second subscription provider.

## Integration
- crates/providers/src/auth_broker.rs (new) + oauth.rs (refactored behind the trait, call sites
  migrated); the runtime adapters read credentials through the broker. frontend_pair: auth status per
  provider in the existing RuntimePanel/SettingsWindow rows (no new component).
  UI/UX INSPIRATION (build-time, the per-provider auth-status + Connect rows): when the LLM builds
  the provider auth surface it inspires itself on OpenCode's provider-connect UI
  (github.com/sst/opencode -> anomalyco/opencode, MIT, TypeScript; source cloned to
  docs/.source/opencode) — its Providers list + per-provider Connect + verified status is the
  closest existing match. LAYOUT + INTERACTION ideas ONLY, clean-room ELAI-native code + names;
  copy NO credential-replay (the sanctioned-OAuth lane only; ledger DO-NOT-COPY stands). J-30 owns
  the wider BYOK/custom-endpoint panel that shares this surface.

## WIRING_CONTRACT
- produces_symbols: providers::auth_broker (AuthProvider trait + broker + storage facade), the migrated
  Anthropic impl, AuthBrokerEvent SSB rows.
- consumes_symbols: providers::oauth (the flow being wrapped — logic preserved), B-37 runtime adapters
  (the credential consumers).
- reachable_from: the production request-build path of every adapter that authenticates (anthropic_api
  at minimum) — credentials flow through the broker, not around it. Contracts-only callers do not
  satisfy (R7).
- stub_ban: NO behavior change to the Anthropic flow (byte-compat guard is a test); NO plaintext
  credential logging (journal rows carry provider id + event kind, NEVER token material); NO new storage
  location; NO unwrap/expect on prod paths; NO silent refresh failure (typed error + row).
- observable_effect:
  trigger: "a real authenticated cloud request through the existing Anthropic plan auth, post-migration"
  expect_ssb: "AuthBrokerEvent(TokenAcquired or Refreshed) naming provider=anthropic; a seeded
  expired-token fixture lands Refreshed then the request succeeds; a refresh-failure fixture lands
  RefreshFailed + a typed error (never a hang)"
  expect_artifact: "the token storage file is byte-compatible with the pre-migration format (the
  regression test diffs a fixture store); the NEGATIVE arm — journal grep for token material — is EMPTY;
  a request that authenticated without a broker row is the broken arm and FAILS (R1)."

## Grounding
- STEAL-LIST A2 (Tier 2); SL-3, SL-10.
- CLAUDE.md no-removal-without-confirmation (oauth.rs behavior preserved), no-new-deps.

## COMPOSES-WITH
- oauth.rs (wrapped, preserved), B-37 adapters (consumers), I-49 direct lanes (future credential impls),
  I-18 (auth events join the chained record like every row). Restates none. SSB: +AuthBrokerEvent
  (Kind: TokenAcquired/Refreshed/ProviderLinked/RefreshFailed). baseline+1 from LIVE 221/532 (SL-1);
  re-read at edit (W-9).

End of I-50.
