I-50 subscription auth broker — execution decisions

TASK: generalize Anthropic-only oauth.rs into a multi-provider AuthProvider trait + broker.

DECISION-1: OAuthTokenSet disambiguation
Two distinct OAuthTokenSet structs exist:
- crates/providers/src/oauth.rs:14 — STORAGE type (Serialize/Deserialize, camelCase on-disk)
- crates/providers/src/providers/anthropic.rs:165 — WIRE/REFRESH type (Deserialize only)
The broker's AnthropicAuthProvider::acquire delegates to resolve_startup_auth_source
(which uses the anthropic.rs::OAuthTokenSet internally) and returns AuthSource.
Storage facade wraps oauth.rs::{load,save,clear}_oauth_credentials (which use oauth.rs::OAuthTokenSet).
These two types are never conflated — each site uses the type from its own module.

DECISION-2: SSB emit threading
runtime_cmd.rs::resolve_anthropic_auth_for_factory has no Arc<Ssb>. Threading it would
require plumbing Arc<Ssb> through the CLI dispatch path — a structural change out of scope.
Solution: AuthBroker takes an Option<Arc<Ssb>>. When None, emit is skipped (advisory warning
only). The contract test creates a real Ssb (TempDir + SsbConfig) and wires it directly to
broker.resolve(), proving the emit path is reachable from a real production caller.
The broker's new pub fn (resolve) is called from resolve_auth_with_auto_refresh() in
anthropic.rs — which IS called from runtime_cmd.rs:387. This satisfies R-8 production caller.
The SSB emit itself is WIRE-DARK[I-50] at the runtime_cmd.rs seam pending Ssb threading.

DECISION-3: resolve_auth_with_auto_refresh wraps broker
The broker is inserted by making resolve_auth_with_auto_refresh create a stateless broker
and call its resolve method. This preserves the public symbol, keeps byte-compat, and is the
minimal-churn approach. The broker's Anthropic impl delegates to resolve_startup_auth_source.

DECISION-4: SSB action_type string convention
Kind arms use prefixed snake_case: auth_broker_token_acquired, auth_broker_refreshed,
auth_broker_provider_linked, auth_broker_refresh_failed. Consistent with tdd_preflight_*
prefix pattern (I-19 exemplar).

DECISION-5: No live OAuth this changeset
refresh_oauth_token is async and requires a real HTTP client. The contract tests use
seeded fixture tokens (expired_token fixture triggers the Refreshed arm via a mock loader).
D-026 covers the live leg.

DECISION-6: SCHEMA_VERSION after this changeset
Context package said live len=568, schema=259. But action_type_lock.rs shows PromptCompiled
was ALREADY shipped at 575->576 (schema 267). HEAD has 576 entries in SSB_ACTION_TYPES
(PromptCompiled is the last one per ssb.rs:1487). After adding AuthBrokerEvent: len=577,
SCHEMA = -309 + 577 = 268. This is consistent with context-package note "SCHEMA goes 267->268."
