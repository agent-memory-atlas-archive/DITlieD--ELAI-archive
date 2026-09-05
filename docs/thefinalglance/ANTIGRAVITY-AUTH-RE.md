# Antigravity — Auth System (reverse-engineered, 2026-07-06)

Scope: how Google Antigravity (v2.2.1) authenticates a user and attaches that identity to model
calls. Grounded from the shipped binary `resources/bin/language_server.exe` (Go, stripped) and the
on-disk credential store under `~/.gemini/`. Everything below is either read from those artifacts or
labelled INFERRED. Written to sit next to ELAI-SYSTEMS-OVERVIEW.md because the one genuinely
transferable piece for ELAI is this layer (a `CloudCodeAdapter` for the existing multi-runtime
backend set), not the agent orchestration.

Anti-slopsquat / secret note: `~/.gemini/oauth_creds.json` is a LIVE bearer credential (access +
refresh + id token) in plaintext. This doc records structure and constants only, never token values.

---

## 0. TL;DR

- Identity = personal Google account via OAuth 2.0 (Authorization Code + PKCE, loopback redirect;
  device-code as fallback). `settings.json` -> `security.auth.selectedType: "oauth-personal"`.
- Credentials live in `~/.gemini/` — the SAME store Google's `gemini-cli` uses. Antigravity is the
  Codeium/Windsurf "Cascade" language server re-skinned onto Google's Cloud Code identity + backend.
- Access token is short-lived (expires ~1h). The durable secret is the refresh_token; the Go server
  refreshes on demand against `oauth2.googleapis.com/token`.
- Model calls go to `cloudcode-pa.googleapis.com` (Cloud Code Assist private API), authenticated with
  `Authorization: Bearer <access_token>` and a quota-project header. Entitlement/tier is resolved at
  session start via `loadCodeAssist` / `onboardUser` (free-tier vs standard-tier).
- An enterprise/API-key + Vertex path exists in the binary but is UNUSED on this install (no key file).

---

## 1. Identity model

- Auth type: `oauth-personal` (read from `~/.gemini/settings.json` -> `security.auth.selectedType`).
  This is "log in with your Google account", not an API key and not a GCP service account.
- Shared credential root: `~/.gemini/`. Antigravity, the Antigravity-installed IDE, and Google's
  standalone `gemini-cli` all read/write the same directory. Logging in via one logs in all.
- Multi-account aware: `~/.gemini/google_accounts.json` holds `{ active: <email>, old: [<email>...] }`.
  A single active account at a time; account switch rewrites this file + re-runs the OAuth flow.
- Per-machine identity: `~/.gemini/antigravity/installation_id` (36-byte UUID) + a telemetry
  machineId/devDeviceId in the IDE's globalStorage. INFERRED: installation_id is sent as a stable
  client fingerprint on `onboardUser`.

## 2. Where credentials are stored (on-disk)

All under `~/.gemini/`:
- `oauth_creds.json` — the OAuth token bundle. Keys (values NOT recorded here):
  - `access_token`  (string, Bearer, short-lived)
  - `refresh_token` (string, durable — the real secret)
  - `id_token`      (string, OIDC identity JWT: email + sub)
  - `scope`         (string, space-delimited granted scopes)
  - `token_type`    ("Bearer")
  - `expiry_date`   (number, epoch ms — access-token expiry; observed already-past, i.e. refreshed on
                     demand rather than pre-emptively)
- `google_accounts.json` — active + previously-seen account emails.
- `config/config.json` — non-secret user settings (e.g. `remoteControlHostname`). Plaintext.
- `settings.json` — feature/auth/model prefs, incl. `security.auth.selectedType` and `model.name`.
- `projects.json` — maps local workspace folder paths -> an Antigravity project slug (per-folder
  project registration; NOT GCP project IDs).
- `state.json` — trivial UI state (banner counts, tips shown).
- No API-key file present on this install (`ls ~/.gemini | grep -i key` = empty) -> confirms pure OAuth.

Credentials are NOT in the OS keychain here; they are flat JSON in the home dir (same as gemini-cli).

## 3. The OAuth flow (grounded from binary string constants)

Primary: OAuth 2.0 Authorization Code grant with PKCE and a loopback redirect.
- The server starts a local HTTP listener on `127.0.0.1:<ephemeral-port>` as the redirect target
  (binary shows `127.0.0.1:<port>` + `redirect_uri` / `redirect_uris`).
- Opens the system browser to `https://accounts.google.com/o/oauth2/auth` with:
  `response_type=code`, `client_id=<one of the two below>`, `redirect_uri=http://127.0.0.1:<port>`,
  `scope=<see 5>`, `code_challenge=<S256>`, `code_challenge_method=S256` (PKCE — constants
  `code_challenge`, `code_challenge_method`, `code_verifier` all present).
- Google redirects back to the loopback with `?code=...`; the server exchanges it at
  `https://oauth2.googleapis.com/token` with `grant_type=authorization_code`, the `code_verifier`,
  and a client_secret (installed-app secret — non-confidential by design; PKCE is the real guard;
  binary shows both `client_secret_post` and `client_secret_basic` auth styles).
- Response is persisted to `oauth_creds.json`.

Fallback: device-code flow (constants `device_code`, `/device/code`, device `grant_type`). Used for
headless/remote/no-browser contexts (e.g. the remote-control host). Poll-based.

Refresh: when `access_token` is expired, the server POSTs `grant_type=refresh_token` +
`refresh_token` to `oauth2.googleapis.com/token`, replaces the access token + expiry in place. There
is also an mTLS token variant (`oauth2.mtls.googleapis.com/token`) and a `refresh_token_expires_at`
field in the token schema (refresh tokens can themselves expire / be revoked).

## 4. OAuth client IDs (embedded in the binary)

Two Google installed-app OAuth client IDs are compiled in:
- `1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com`
- `884354919052-36trc1jjb3tguiac32ov6cod268c5blh.apps.googleusercontent.com`

INFERRED (not fully confirmed): one is the browser/loopback client, the other the device-flow client
(or prod vs daily-sandbox). Neither matches gemini-cli's well-known public client id
(`681255809395-...`), so these are Antigravity/CloudCode-specific desktop clients. As Google
"installed application" clients, the accompanying client_secret is not a true secret — it ships in the
binary and Google's threat model relies on PKCE + the loopback redirect, not secret confidentiality.

## 5. Scopes

Granted scopes actually present in `oauth_creds.json` on this install:
- `https://www.googleapis.com/auth/cloud-platform`
- `https://www.googleapis.com/auth/userinfo.profile`
- `https://www.googleapis.com/auth/userinfo.email`
- `openid`

Additional scopes REQUESTABLE per binary strings (not all granted here) — the app can escalate to:
- a custom `.../auth/aicode` scope (the Code Assist / AI-code capability)
- `.../auth/experimentsandconfigs`, `.../auth/cclog` (config + client-logging)
- several Google Drive scopes (`drive.appdata`, `drive.file`, `drive.metadata.readonly`, ...) — used
  to sync agent state / knowledge into Drive app-data. INFERRED: opt-in, tied to the Drive-sync
  feature, not required for basic model calls.

`cloud-platform` is the load-bearing scope: it authorizes the Cloud Code Assist backend calls.

## 6. Endpoints (cleaned — the binary concatenates adjacent strings)

Auth / identity:
- Authorization:  `https://accounts.google.com/o/oauth2/auth`
- Token exchange: `https://oauth2.googleapis.com/token`
- Device code:    `https://oauth2.googleapis.com/device/code`
- mTLS token:     `https://oauth2.mtls.googleapis.com/token`
- IAM credentials:`https://iamcredentials.googleapis.com/v1/` (service-account impersonation path;
                   INFERRED enterprise-only)

Model / product backend:
- Prod:    `https://cloudcode-pa.googleapis.com`
- Sandbox: `https://daily-cloudcode-pa.googleapis.com` (the "daily" release channel)
- Telemetry: `https://play.googleapis.com/log`
- Exports:   `https://alkalimakersuiteapplets.pa.googleapis.com/v1/exports/`

The prod-vs-sandbox host is selected inside the binary (release channel), not in user config.

## 7. Token -> model-call binding (how auth reaches inference)

Every Cloud Code Assist request is `POST https://cloudcode-pa.googleapis.com/v1internal:<action>` with:
- `Authorization: Bearer <access_token>` (from `oauth_creds.json`, refreshed as needed)
- a quota/user project header (binary shows `x-goog-user-project` + `GOOGLE_CLOUD_PROJECT`) — the GCP
  project the usage is billed/quota'd against. For personal free-tier this is a Google-managed
  project resolved during onboarding.

Session bootstrap / entitlement (runs before inference):
- `v1internal:loadCodeAssist` — loads the account's Code Assist state (cached in `loadCodeAssistResponseCache`).
- `v1internal:onboardUser` (+ `onboardUserBackgroundTasks`) — first-run onboarding; establishes tier.
- `v1internal:fetchUserInfo`, `v1internal:getCodeAssistGlobalUserSetting` — user profile + global settings.
- Tier is `free-tier` vs `standard-tier` (both are literal constants; there is also an internal
  `standard-tier` "googler" branch). Tier gates which models `fetchAvailableModels` returns.

Inference actions (all under `/v1internal:`):
- `generateContent`, `generateChat`, `generateCode` — the model calls.
- `internalAtomicAgenticChat` — the agentic (tool-using) turn.
- `completeCode` — inline autocomplete.
- `countTokens` — context budgeting.
- `fetchAvailableModels` / `listModelConfigs` — SERVER decides the model menu per account/tier.
- `listAgents` — the multi-agent surface.

Models reachable through this backend (grounded strings), server-gated:
- Gemini: `gemini-2.5-pro` (the settings default), `gemini-2.5-flash`, `-flash-lite`,
  `-flash-image-preview`.
- Claude: `claude-opus-4`, `-4-5`, `-4-6`, `claude-sonnet-4`, `-4-5`, `-4-6` (there is a
  `gemini_default -> claude-opus-4-6` alias string — INFERRED: the default agent model may route to
  Claude Opus behind the scenes).
- House coder model family: `gemini_coder` (the Codeium-trained model, for cheap/fast paths).

## 8. Enterprise / alternative auth (present, UNUSED here)

The binary carries a full API-key + Vertex path: `api_key`, `apiKeyConfig`, `apiKeySecret`,
`apiKeySecretVersion`, `GOOGLE_CLOUD_PROJECT`, IAM-credentials impersonation. INFERRED: this is the
Gemini Code Assist Standard/Enterprise route (Workspace/Vertex, API key or service account + a real
GCP project). Not active on this install (no key file; auth type is oauth-personal). Relevant only if
ELAI wanted a paid/enterprise Cloud Code lane later.

## 9. What transfers to ELAI (the actual payload)

The reusable artifact is a new runtime backend, NOT any agent logic. ELAI's multi-runtime abstraction
(B-37, ELAI-SYSTEMS-OVERVIEW.md §11) already lists `GeminiCli` as a `RuntimeAdapter`. This RE turns
that into a real `CloudCodeAdapter`:

- Reuse the user's existing `~/.gemini/oauth_creds.json` (Bearer + refresh) — zero new login if the
  user already runs gemini-cli or Antigravity.
- Implement the token lifecycle: read creds, refresh at `oauth2.googleapis.com/token` on 401/expiry,
  write back atomically.
- Call `cloudcode-pa.googleapis.com/v1internal:generateContent` (or `:generateChat`) with the Bearer
  + `x-goog-user-project`. Map ELAI's Anthropic-protocol request shape to the Cloud Code request
  shape (INFERRED: protobuf/JSON; the exact message schema is the next RE step — dump proto
  descriptors from the binary or proxy one live call).
- Payoff: a near-free flagship lane (Gemini 2.5 Pro AND Claude 4.x via Google's free/standard Code
  Assist tier) behind ELAI's trust gates. Directly serves the "$/resolved-task, verified" thesis
  (M-plan) rather than paying Anthropic list rates.
- Determinism caveat (ELAI-specific): Cloud Code is a cloud lane — it goes in the CLOUD/action-faithful
  RRA tier, never token-faithful. Entitlement + model availability are exogenous and server-driven, so
  ELAI's model-drift regression canary (§16) MUST watch `fetchAvailableModels` output like any other
  exogenous model version. ToS: personal Code Assist free tier has usage terms; a product shipping this
  lane needs to check Google's ToS on programmatic reuse — this is a personal-use RE note, not a
  cleared distribution plan.

## 10. Open items / how to verify further (next RE steps)

- Exact request/response SCHEMA for `generateContent` / `internalAtomicAgenticChat`: dump the embedded
  protobuf FileDescriptorSet from the Go binary (the `exa.*` + cloudcode protos), or MITM-proxy one
  live call (set an HTTPS proxy + trust a local CA, watch the Bearer + body). Highest-value next step.
- Which of the two client IDs is browser vs device vs sandbox: diff the two flows live.
- Whether refresh tokens are per-client-id or shared: check after an account switch.
- The `x-goog-user-project` value source for free-tier (Google-managed vs derived): read it off a live
  request.
- Confirm the `gemini_default -> claude-opus-4-6` alias behavior with a real chat turn + network
  capture.

Grounding: `resources/bin/language_server.exe` string extraction (Go 1.26/1.27, `exa.*`/`codeium`
symbols, `gemini_coder` model family), `~/.gemini/{oauth_creds,google_accounts,settings,config,
projects,state}.json`, and the launcher `dist/{paths,languageServer}.js`. All read 2026-07-06.
