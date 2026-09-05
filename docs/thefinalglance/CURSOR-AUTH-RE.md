CURSOR REVERSE-ENGINEERING — auth + protocol (for a TEIKOKU grok-4.5 provider)

Status: 2026-07-09. RE substantially complete + server-verified through the AvailableModels call and
the chat endpoint's own error-walk. One wall remains: the current chat transport encrypts its request
chunks (AES-256-GCM) and rides an encrypted BidiAppend/BidiPoll side-channel; the exact chunk framing
needs a Frida-unpinned live capture (cert pinning + proxy-bypass block a plain mitmproxy of the chat
stream). Sibling of ANTIGRAVITY-AUTH-RE.md; same personal-use posture.

================================================================================
0. TARGET
================================================================================
Cursor 3.10.20 (Anysphere fork of VS Code), commit 23b9fb205fe595ea2be29da7214e19762d037fc0,
date 2026-07-07. Install: C:\Program Files\cursor. AI code is in
out/vs/workbench/workbench.glass.main.js (45 MB, "glass" = Anysphere's fork codename).
Protobuf codegen is @bufbuild/protobuf (makeMessageType); the full aiserver.v1.* schema is baked in.

================================================================================
1. BACKEND + PROTOCOL
================================================================================
- Hosts: api2.cursor.sh (AI), api3.cursor.sh (telemetry /tev1), api4/api5 exist. marketplace.cursorapi.com (extensions).
- RPC: Connect-RPC (connect-es/1.6.1). Unary calls use content-type application/proto (no framing).
  Streaming uses application/connect+proto with 5-byte envelope frames: [flag:1][len:4 BE][payload];
  flag bit 0x01 = gzip, flag 0x02 = end-of-stream (JSON trailer with error/metadata).
- HTTP/2. Connect header connect-protocol-version: 1.
- Service for chat: aiserver.v1.ChatService. Model catalog: aiserver.v1.AiService/AvailableModels.
  Tab autocomplete: aiserver.v1.AiService/CppAppend + StreamCpp.

================================================================================
2. AUTH (proven)
================================================================================
- Login: WorkOS OAuth (signup=Google). Account here: Cursor Pro (full_stripe_profile membershipType=pro,
  subscriptionStatus=active).
- Tokens live in %APPDATA%\Cursor\User\globalStorage\state.vscdb (SQLite, table ItemTable):
    cursorAuth/accessToken   -> JWT (eyJ...), ~424 B. Sent as  Authorization: Bearer <accessToken>.
    cursorAuth/refreshToken  -> refresh JWT.
    cursorAuth/cachedEmail, cursorAuth/stripeMembershipType (=pro), cursorAuth/cachedSignUpType (=Google).
    BYOK slots present but empty: cursorAuth/{claudeKey,openAIKey,googleKey,azureApiKey,bedrock*}.
- machineid file: %APPDATA%\Cursor\machineid (36-byte UUID). The checksum uses the telemetryService
  machineId + macMachineId (sha256 hex, 64 chars each), NOT the raw uuid.
- Bearer accessToken alone is sufficient for AvailableModels / FileSync / Cpp (all verified 200).

================================================================================
3. x-cursor-checksum (proven, reproduced from source)
================================================================================
Two builders exist; the server accepts both (validates loosely).

AI-path builder DCv (workbench.glass.main.js), verbatim:
  function RCv(t){ let e=165; for(let n=0;n<t.length;n++){ t[n]=(t[n]^e)+n%256; e=t[n]; } return t; }
  // in DCv: k=Math.floor(Date.now()/1e6);
  //         E=new Uint8Array([k>>40&255,k>>32&255,k>>24&255,k>>16&255,k>>8&255,k&255]);
  //         A=base64Fn(RCv(E));            base64Fn = base64url, no padding
  //         header = n===undefined ? `${A}${machineId}` : `${A}${machineId}/${macMachineId}`
So: x-cursor-checksum = base64(RCv(6-byte ts)) + machineId + "/" + macMachineId  (total 137 chars:
  8 + 64 + 1 + 64). machineId/macMachineId are per-machine constants (read once).
This machine's values (captured live):
  machineId    = ce9481b087c0bcda6bb34d3f2a26cbb611d577b6f17a9d1f8a628c9a6dee173f
  macMachineId = 346bd093284d58cf00cad0220733ca2e0405b95f11e9622fdbc8a63728cedf81
  x-client-key = e2b01261a0cba20bef5f6301068a2d406d530bc8fd70c677f254f980d3ab7779
The connect-es transport (FileSync/Cpp) uses a different ts packing (reverses to bytes 36f2001b36f2 for
k=0x1B36F2) but the same machineId/macMachineId tail; the server accepted it too, so the ts byte-order
is not strictly validated. Reproduced Python (both variants) in scratch cursor_probe.py:RCv/checksum().

================================================================================
4. REQUEST HEADERS (from a real accepted AiService call, CppAppend)
================================================================================
authorization: Bearer <accessToken>
connect-protocol-version: 1
content-type: application/proto  (unary)  |  application/connect+proto (streaming)
user-agent: connect-es/1.6.1
x-client-key: <64 hex>
x-cursor-checksum: <section 3>
x-cursor-client-version: 3.10.20
x-cursor-client-type: glass         x-cursor-client-layout: glass
x-cursor-client-os: win32           x-cursor-client-arch: x64
x-cursor-client-device-type: desktop
x-cursor-config-version: <uuid, per session>
x-cursor-streaming: true            x-cursor-timezone: Asia/Tokyo
x-ghost-mode: false                 x-new-onboarding-completed: false
x-request-id: <uuid>                x-session-id: <uuid, stable per app run>
Headers are NOT the blocker — a hand-built request with these gets a normal structured Connect response.

================================================================================
5. MODEL CATALOG (fully working)
================================================================================
POST https://api2.cursor.sh/aiserver.v1.AiService/AvailableModels  (unary, empty body) -> 200, ~87 KB.
Decoded (blackboxprotobuf): top field 2 = repeated model entry (156 entries). Each entry:
  field 1  = canonical model_name (this is what model_details.model_name must be)
  field 17 / 24 = display name;  field 18 = name echo;  field 36 = base alias (e.g. "grok-4.5")
GROK 4.5 canonical ids (field 1): grok-4.5-medium ("Grok 4.5 Low"), grok-4.5-high ("Grok 4.5 Medium"),
  grok-4.5-xhigh ("Grok 4.5" <- the UI default), grok-4.5-fast-medium/-fast-high/-fast-xhigh.
  Bare "grok-4.5" is only field-36 alias -> ERROR_BAD_MODEL_NAME if sent as model_name.
  => USE model_name = "grok-4.5-xhigh" (or -fast-xhigh). Verified accepted by the chat model validator.
Roster also includes claude-opus-4-8/-4-7/-4-6, claude-sonnet-5, claude-fable-5, gpt-5.5/5.4/5.3-codex,
gemini-3.1-pro/3.5-flash, glm-5.2, kimi-k2.7-code, composer-2.5, cursor-small, gpt-5-mini, etc.

================================================================================
6. CHAT ENDPOINT (mapped; blocked on encrypted chunk framing)
================================================================================
Service aiserver.v1.ChatService methods (from binary):
  StreamUnifiedChat                     -> SERVER-DEPRECATED (ERROR_DEPRECATED).
  StreamUnifiedChatWithTools            (BiDi)         -> returns resource_exhausted / "Update Required"
                                                          catch-all for a hand-built client (semi-retired).
  StreamUnifiedChatWithToolsIdempotent  (BiDi)         -> THE LIVE PATH. Parses the request, then gates on
                                                          headers (below). This is what 3.10.20 uses.
  ...IdempotentSSE / ...IdempotentPoll  (ServerStreaming) -> resume/read side, keyed by BidiRequestId{request_id}.

Request nesting (all @bufbuild field numbers from the 3.10.20 schema):
  StreamUnifiedChatRequestWithToolsIdempotent {
    1 client_chunk : StreamUnifiedChatRequestWithTools {
        1 stream_unified_chat_request : StreamUnifiedChatRequest { ... }
        2 client_side_tool_v2_result  (subsequent frames only)
      }
    2 abort   3 close   4 idempotency_key(str)   5 seqno(uint)
  }
StreamUnifiedChatRequest key fields: 1 conversation(rep ConversationMessage), 3 explicit_context,
  5 model_details, 22 is_chat, 23 conversation_id, 26 environment_info, 27 is_agentic,
  30 full_conversation_headers_only(rep ConversationMessageHeader), 33 use_unified_chat_prompt,
  46 unified_mode(enum UNSPECIFIED/CHAT=1/AGENT=2/EDIT=3/CUSTOM=4), 47 tools_requiring_accepted_return.
ConversationMessage: 1 text, 2 type(enum), 13 bubble_id, 32 server_bubble_id, 18 tool_results, ...
ModelDetails: 1 model_name, 2 api_key, 5 enable_slow_pool, 8 max_mode.
EnvironmentInfo: 1 exthost_platform, 2 exthost_arch, 3 exthost_release, 5 local_timestamp,
  7 cursor_version, 9 local_os_type, 11 local_timezone.
ConversationMessageHeader: 1 bubble_id, 2 server_bubble_id, 3 type(enum).

Server error-walk on the Idempotent endpoint (each fixed in turn, proving structural correctness):
  (no idempotency header)          -> "X-Idempotency-Key header is required"
  + x-idempotency-key              -> "X-Idempotent-Encryption-Key header is required"
  + x-idempotent-encryption-key    :
       32-char key                 -> "X-Idempotent-Encryption-Key is invalid"     (wrong length)
       base64(32 bytes)=44 char    -> ACCEPTED (key format correct) but then HANGS (chunk not decryptable)
So: key = base64 of a 32-byte AES-256 key, sent in header x-idempotent-encryption-key; the client_chunk
(field 1) must be the AES-256-GCM ciphertext of the serialized WithTools message. Plaintext -> server
holds the stream open and returns nothing.

Encryption primitive (binary): AES-GCM, IV_LENGTH=12, key imported raw as 256-bit (class dkv/g4t is the
blob-store instance of the same primitive). The chat transport also uses a BidiAppend/BidiPoll channel:
  BidiAppendRequest { 2 request_id(BidiRequestId), 3 append_seqno, 4 data_binary(bytes) }
  BidiPollResponse  { 1 seqno, 2 data, 3 eof }
  StreamUnifiedChatResponseWithToolsIdempotent { 1 server_chunk, 3 welcome_message, 4 seqno_ack }
i.e. the live protocol is: open idempotent stream -> append encrypted data_binary chunks (seqno'd) ->
poll for encrypted response chunks (seqno/eof). Exact framing of the encrypted chunk (IV prepend vs
separate, AAD, tag placement) and the append/poll handshake are the remaining unknowns.

================================================================================
7. THE INTERCEPTION WALL (why a plain mitmproxy of the chat fails)
================================================================================
- Unary calls (AvailableModels/FileSync/Cpp) go through connect-es' fetch/undici transport and DO honor
  HTTPS_PROXY + NODE_TLS_REJECT_UNAUTHORIZED=0 -> captured fine via mitmproxy 12.2.2.
- The streaming chat uses node:http2 directly: it (a) ignores HTTPS_PROXY (connects direct to api2) and
  (b) rejects the mitm cert ("tlsv1 alert unknown ca") even with the CA in the Windows Root store =>
  certificate pinning on the AI transport. Grok works in-app while mitmproxy sees zero chat flows.
- => capturing the real chat request requires Frida SSL-unpin + forcing node:http2 through the proxy
  (or a JS-level hook that dumps the pre-encryption plaintext chunk + framing). This is the chosen next step.

================================================================================
8. WHAT A BRIDGE NEEDS (once the chunk framing is captured)
================================================================================
cursor_proxy.py (mirror of workflow/cloudcode-bridge/cloudcode_proxy.py): OpenAI /v1/chat/completions +
Anthropic /v1/messages in; on each turn: load accessToken from state.vscdb (auto-refresh via refreshToken
when 401), build DCv checksum + the header set (section 4), build the StreamUnifiedChatRequest, AES-256-GCM
encrypt the WithTools chunk with a fresh 32-byte key (header + framing per capture), drive the
Idempotent + Append/Poll channel, decrypt + protobuf-parse response chunks (field 2 stream_unified_chat_
response -> text), stream back. model_name = "grok-4.5-xhigh". Then TEIKOKU pool wiring mirrors the
cloud-code lane (worker_providers + slice_api._resolve_lane_env branch + FE provider button).

RISK (accepted, personal-use): Cursor actively detects/bans reverse-engineered clients, and this encrypted
polled transport is versioned + will drift. Treat as fragile; xAI api.x.ai is the robust paid fallback for grok-4.5.

================================================================================
9. SCRATCH ARTIFACTS (this session, ephemeral)
================================================================================
scratchpad/cursor_probe.py     - token load, DCv+connect-es checksum, protobuf writer, live probe (WORKING for
                                 AvailableModels + the endpoint error-walk).
scratchpad/cursor_capture.py   - mitmproxy addon (captures unary cursor.sh flows; blind to the pinned chat stream).
scratchpad/decode_chat.py      - connect de-framer + schema-labelled protobuf decoder.
scratchpad/cap2/               - captured unary flows (auth, FileSync, Cpp, stripe profile, model catalog bytes).

================================================================================
10. SESSION 2 CORRECTION (2026-07-09) — LIVE CAPTURE: THERE IS NO ENCRYPTION WALL
================================================================================
The "encrypted chunk / idempotency-key wall" in sections 6-8 was a MISDIAGNOSIS. It described the
LEGACY aiserver.v1.ChatService.StreamUnifiedChatWithToolsIdempotent path, which Cursor 3.10.20 no
longer uses for chat. A live capture of a real grok turn proved the current chat path is different,
unencrypted, and straightforward.

CAPTURE METHOD (no Frida, no symbols, no relaunch): BoringSSL is statically linked + stripped into
Cursor.exe, so the section-8 Frida-EVP/SSL symbol plan is dead. Cursor chat also has NO client-side
AES for the chat (the only WebCrypto AES-GCM in the bundles is the agent_kv blob store [dkv/g4t] and
the mcpEncryptionKey MCP-input crypto -- neither is chat). Instead: node V8 inspector.
  - The chat transport runs in the electron-utility process whose VSCODE_ESM_ENTRYPOINT is
    vs/code/electron-utility/alwaysLocalSingleton/alwaysLocalSingletonMain (a stock @connectrpc/
    connect-node http2 client). Activate its inspector with node's process._debugProcess(pid)
    (Windows cross-process inspector trigger), connect CDP, and Runtime.evaluate a hook that uses
    process.getBuiltinModule('node:http2') (node 22.3+, works with no require/import in the ESM eval
    context) to patch ClientHttp2Session.prototype.request. Dump headers + outgoing frames to a file
    via getBuiltinModule('node:fs'); the patch + capture survive after CDP detaches.
  - Tooling saved under workflow/cursor-bridge/frida/: cdp.ts (bun CDP client: scan|eval|hook),
    dbg_enable.js (_debugProcess trigger), http2_hook.js (the transport tap), run.py + enum.js
    (frida enum that proved BoringSSL is stripped). capture.jsonl + run_request.bin = the live grok turn.

THE REAL CHAT PATH (captured, live grok-4.5 turn):
  POST https://api2.cursor.sh/agent.v1.AgentService/Run     (NOT aiserver.v1.ChatService)
  content-type: application/connect+proto   connect-protocol-version: 1
  connect-content-encoding: gzip            connect-accept-encoding: gzip
  RPC shape (from bundle): agent.v1.AgentService.run = {I: AgentClientMessage, O: AgentServerMessage, kind:3 BIDI}.
    also runSSE (kind:1) + runPoll (kind:1) resume variants; getUsableModels/getAllowedModelIntents for the picker.
  NO x-idempotency-key, NO x-idempotent-encryption-key headers exist on this path. No app-layer encryption.

REQUEST FRAMING (connect streaming, unencrypted):
  body = [flag:1][len:4 BE][payload].  flag 0x01 = gzip-compressed payload (0x00 = identity).
  payload = gzip( serialized agent.v1.AgentClientMessage ).  gunzip -> plain protobuf, NO AES.
  Captured turn: one 69,657-byte frame -> gunzip -> 277,918 B protobuf (full conversation + codebase context).
  AgentClientMessage (uX) is a oneof: 1 run_request(Dz) 2 exec_client_message 5 exec_client_control_message
    3 kv_client_message 4 conversation_action 6 interaction_response 7 client_heartbeat 8 prewarm_request.
  The initial frame carries field 1 run_request; subsequent small client frames stream kv/control/heartbeat
    with an incrementing seqno (observed 1a0408NN1a00 = field-3 msgs, NN=01,02,03...).

RESPONSE FRAMING:
  AgentServerMessage (OS) oneof: 1 interaction_update(Qp = streamed model text) 2 exec_server_message
    5 exec_server_control_message 3 conversation_checkpoint_update(Tn) 4 kv_server_message 7 interaction_query.
  Frames are connect+proto, mostly uncompressed small frames; text tokens ride field 1 interaction_update.
  (Server->client body bytes were NOT tapped this session -- the hook taps outgoing only to avoid consuming
   the app's stream; a follow-on capture taps incoming to fully map Qp.)

MODEL SELECTION (new format, differs from section 5):
  In the run_request, the model is the BARE name "grok-4.5" (NOT grok-4.5-xhigh) plus a kv option block:
    {name:"grok-4.5"} {effort:"high"} {fast:"true"}   (effort in {low,medium,high}; fast bool).
  i.e. UI "Grok 4.5" = model "grok-4.5" + effort=high + fast=true. The -xhigh suffix belonged to the
  legacy aiserver path. The picker sends a repeated model list (default, grok-4.5, composer-2.5, ...).

LIVE HEADER SET (agent.v1.AgentService/Run, verbatim keys):
  authorization: Bearer <JWT from state.vscdb>
  connect-protocol-version: 1   content-type: application/connect+proto
  connect-content-encoding: gzip   connect-accept-encoding: gzip
  user-agent: CONNECT_ES_USER_AGENT   (literal placeholder string)
  x-cursor-checksum: <8char>+machineId+"/"+macMachineId  (macMachineId now 0fc5633727d6b5e57102dc064a69220165f6f75d10514ebdb3d50b2dd9b1666a)
  x-client-key / x-fs-client-key / x-blob-encryption-key: <32-byte hex each, per-turn/per-machine>
  x-cursor-streaming: true   x-cursor-retryinterceptor-enabled: true   x-cursor-remote-type: none
  x-cursor-client-version: 3.10.20  -type/-layout: glass  -os: win32  -arch: x64  -device-type: desktop
  x-cursor-config-version/-timezone, x-request-id/x-original-request-id, x-session-id, traceparent/backend-traceparent
  (x-blob-encryption-key is the agent_kv blob-store key, NOT a chat-chunk key -- there is no chat-chunk key.)

BRIDGE IMPACT: the bridge (section 8) is now much simpler than feared -- no AES, no key handshake.
  Steps: build agent.v1.AgentClientMessage{run_request: RunRequest(Dz) with the conversation + model
  "grok-4.5"+effort+fast}, gzip it, connect-frame it (flag 0x01), POST to /agent.v1.AgentService/Run with
  the header set above, then read connect+proto response frames and pull text from field-1 interaction_update.
  REMAINING for the bridge: fully map Dz (RunRequest) field layout (where conversation text/roles/context go)
  and Qp (interaction_update) -- both are in the bundle's makeMessageType defs + validatable against run_request.bin.

RESPONSE CAPTURED (2026-07-09, run id rckonj5s49, artifact run_response.bin, 17 connect frames / 28KB):
  AgentServerMessage frame types observed:
    field 1 interaction_update  -> the streamed model TEXT. Nested 0a<len>0a<len><utf8>; e.g. frame carried
      "Hi — what do you want to work on?" (the live grok reply). This is the field the bridge reads for output.
    field 3 conversation_checkpoint_update -> gzip-compressed; echoes the FULL conversation state as JSON.
    field 4 kv_server_message   -> gzip; blob/kv sync (hex hashes like 5bf87203..) -- irrelevant to text.
    final frame flag 0x02       -> connect end-of-stream trailer (len 2, empty {}).
  MESSAGE WIRE FORMAT (decoded from the checkpoint frames -- this is what the RunRequest also carries):
    messages are JSON strings embedded in the protobuf, OpenAI/Anthropic-shaped:
      {"role":"system","content":"You are an AI coding assistant, powered by Cursor Grok 4.5. You operate in Cursor...."}
      {"role":"user","content":[{"type":"text","text":"<timestamp>...</timestamp>\n<user_query>\nhi\n</user_query>"}], "providerOptions":...}
      also a ProseMirror input doc {"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"hi"}]}]}
      {"role":"assistant","content":[{"type":"redacted-reasoning","data":"..."},{"type":"text","text":"Hi — ..."}]}
    => the bridge maps OpenAI/Anthropic messages -> this role/content JSON, wraps the user turn in <user_query>, sets
       the system prompt (or lets Cursor's default stand), and selects model "grok-4.5" + effort + fast in the model block.
  Artifacts for the bridge builder: run_request.bin (277KB decoded RunRequest), run_response.bin (28KB response
  stream), capture.jsonl (raw framed flows w/ full header sets). Bundle schema source for exact field numbers:
  out/vs/code/electron-utility/alwaysLocalSingleton/alwaysLocalSingletonMain.js (grep makeMessageType("agent.v1.*)).
