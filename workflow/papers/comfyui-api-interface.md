Source: ComfyUI REST and WebSocket API
Reliability: HIGH (official repository: github.com/comfyanonymous/ComfyUI, server.py verified)
Files examined:
  https://raw.githubusercontent.com/comfyanonymous/ComfyUI/master/server.py (route definitions)
  https://raw.githubusercontent.com/comfyanonymous/ComfyUI/master/execution.py (WebSocket messages, history format)
  https://raw.githubusercontent.com/comfyanonymous/ComfyUI/master/script_examples/websockets_api_example.py (official usage example)
  https://docs.comfy.org/development/comfyui-server/comms_routes (official route docs)

Public API:

POST /prompt | submit workflow for execution
  Request body (JSON):
    {
      "prompt":    object,   REQUIRED — workflow node graph (see Workflow JSON Schema)
      "client_id": string,   OPTIONAL — UUID hex string; links prompt to WebSocket session
      "prompt_id": string,   OPTIONAL — caller-supplied UUID; server generates one if absent
      "number":    float,    OPTIONAL — queue priority position
      "front":     boolean,  OPTIONAL — true = jump to front of queue
      "partial_execution_targets": array,  OPTIONAL — subset of node IDs to execute
      "extra_data": object,  OPTIONAL — passthrough metadata
    }
  Response 200 (success):
    {
      "prompt_id": "string",   — unique ID to poll history with
      "number":    float,      — actual queue position assigned
      "node_errors": {}        — empty object on success
    }
  Response 400 (validation failure):
    {
      "error": {
        "type":    "string",   — e.g. "no_prompt", "validation_error"
        "message": "string",
        "details": "string",
        "extra_info": {}
      },
      "node_errors": {
        "<node_id>": {
          "errors": [
            {
              "type":    "string",  — one of: required_input_missing, bad_linked_input,
                                      return_type_mismatch, invalid_input_type,
                                      value_smaller_than_min, value_bigger_than_max,
                                      value_not_in_list, custom_validation_failed,
                                      exception_during_validation
              "message": "string",
              "details": "string",
              "extra_info": {}
            }
          ],
          "dependent_outputs": ["node_id", ...],
          "class_type": "string"
        }
      }
    }
  Response 400 (no prompt field):
    {
      "error": { "type": "no_prompt", "message": "No prompt provided", "details": "No prompt provided", "extra_info": {} },
      "node_errors": {}
    }

GET /history | list recent history
  Query params: max_items (int, optional), offset (int, optional)
  Response 200: object keyed by prompt_id, each value is a history entry (see below)

GET /history/{prompt_id} | poll completion status
  Path param: prompt_id — UUID string returned from POST /prompt
  Response 200 (prompt known):
    {
      "<prompt_id>": {
        "prompt":  [...],       — internal prompt data array (not for external use)
        "outputs": {
          "<node_id>": {
            "images": [
              {
                "filename":  "string",  — e.g. "ComfyUI_00001_.png"
                "subfolder": "string",  — "" for root output directory
                "type":      "output"   — "output" | "temp" | "input"
              }
            ]
            — other node-type-specific output keys may appear
          }
        },
        "status": {
          "status_str":  "success" | "error",
          "completed":   boolean,
          "messages":    [["string", {}], ...]  — array of [event_type, data] pairs
        }
      }
    }
  Response 200 (prompt NOT yet known / still running): {} (empty object — callers MUST treat empty as "not done yet", not "error")
  Note: there is NO 404 for unknown prompt_id; the server returns {} for both "not yet executed" and "never seen" prompts.

GET /view | download image file
  Query params:
    filename:  string, REQUIRED — e.g. "ComfyUI_00001_.png"
    subfolder: string, OPTIONAL — subdirectory under output root; default ""
    type:      string, OPTIONAL — "output" (default) | "temp" | "input"
    preview:   string, OPTIONAL — "webp" or "jpeg;quality=N" for server-side transcode
    channel:   string, OPTIONAL — "rgba" | "rgb" | "a" for channel extraction
  Response 200: binary image bytes with Content-Disposition header
  Response 400: invalid filename path (path traversal rejected)
  Response 403: path traversal attempt detected
  Response 404: file not found

GET /queue | inspect queue state
  Response 200:
    {
      "queue_running": [[ number, prompt_id, prompt_object, {extra_data}, [client_ids] ]],
      "queue_pending": [[ number, prompt_id, prompt_object, {extra_data}, [client_ids] ]]
    }
  Note: queue_running has at most 1 entry (single-threaded executor). No built-in max queue size limit in default config.

POST /queue | clear queue or delete specific entries
  Request body (JSON):
    {
      "clear":  boolean,  OPTIONAL — true = wipe all pending
      "delete": ["prompt_id", ...]  OPTIONAL — remove specific pending IDs
    }
  Response 200: no body
  Note: cannot cancel the currently running prompt; use POST /interrupt for that.

POST /interrupt | cancel running execution
  Request body (JSON):
    {
      "prompt_id": "string"  OPTIONAL — if omitted, interrupts whatever is running
    }
  Response 200: no body (no confirmation of what was interrupted)
  Behavior: sends interrupt signal; server sends execution_interrupted WebSocket message; does NOT remove from history.

GET /ws | WebSocket connection (upgrade)
  Query param: clientId (string, optional) — if omitted, server generates uuid4().hex and sends it in initial status message
  Protocol: standard WebSocket (ws:// or wss://)
  Initial server message on connect:
    {
      "type": "status",
      "data": {
        "status": { "exec_info": { "queue_remaining": int } },
        "sid": "string"   — the clientId in use (echo of query param or generated value)
      }
    }

WebSocket Message Types (server -> client):

  status:
    { "type": "status", "data": { "status": { "exec_info": { "queue_remaining": int } } } }
    — sent on queue state changes; no sid field after initial connect message

  execution_start:
    { "type": "execution_start", "data": { "prompt_id": "string", "timestamp": int } }

  execution_cached:
    { "type": "execution_cached", "data": { "nodes": ["node_id", ...], "prompt_id": "string", "timestamp": int } }

  executing:
    { "type": "executing", "data": { "node": "node_id" | null, "display_node": "node_id", "prompt_id": "string", "timestamp": int } }
    — COMPLETION SIGNAL: when node == null AND prompt_id matches, execution is done. This is the canonical completion indicator when using WebSocket.

  progress:
    { "type": "progress", "data": { "value": int, "max": int, "prompt_id": "string", "node": "node_id" } }
    — emitted by KSampler during denoising steps; value goes from 1 to max

  executed:
    { "type": "executed", "data": { "node": "node_id", "display_node": "node_id", "output": {ui_output_object}, "prompt_id": "string", "timestamp": int } }

  execution_error:
    { "type": "execution_error", "data": {
        "prompt_id":        "string",
        "node_id":          "string",
        "node_type":        "string",
        "executed":         ["node_id", ...],
        "exception_message":"string",
        "exception_type":   "string",
        "traceback":        ["line1", "line2", ...],
        "current_inputs":   {},
        "current_outputs":  [],
        "timestamp":        int
      }
    }

  execution_interrupted:
    { "type": "execution_interrupted", "data": {
        "prompt_id": "string",
        "node_id":   "node_id",
        "node_type": "string",
        "executed":  ["node_id", ...]
      }
    }

  execution_success:
    { "type": "execution_success", "data": { "prompt_id": "string", "timestamp": int } }

  Binary WebSocket frames: first 8 bytes are a header (event type uint32 + image type uint32); remaining bytes are raw image data (live previews from KSampler). Strip first 8 bytes to get image. Only relevant if using SaveImageWebsocket node or latent previews.

Workflow JSON Schema (the "prompt" field in POST /prompt body):

  {
    "<node_id>": {               — string key, typically "1", "2", ... (any string)
      "class_type": "string",   — REQUIRED: node class name, e.g. "KSampler", "CheckpointLoaderSimple"
      "inputs": {
        "<input_name>": value_or_link
        — Direct value: any JSON primitive or array of primitives
        — Linked input: [source_node_id_string, output_index_int]
          e.g. "model": ["4", 0]  means output index 0 of node "4"
      }
    }
  }

  Minimal SD1.5 workflow example nodes:
    "4": { "class_type": "CheckpointLoaderSimple", "inputs": { "ckpt_name": "v1-5-pruned-emaonly.safetensors" } }
    "5": { "class_type": "EmptyLatentImage",       "inputs": { "batch_size": 1, "height": 512, "width": 512 } }
    "6": { "class_type": "CLIPTextEncode",         "inputs": { "clip": ["4", 1], "text": "positive prompt" } }
    "7": { "class_type": "CLIPTextEncode",         "inputs": { "clip": ["4", 1], "text": "negative prompt" } }
    "3": { "class_type": "KSampler",               "inputs": { "cfg": 8, "denoise": 1, "latent_image": ["5", 0],
                                                                "model": ["4", 0], "negative": ["7", 0],
                                                                "positive": ["6", 0], "sampler_name": "euler",
                                                                "scheduler": "normal", "seed": 42, "steps": 20 } }
    "8": { "class_type": "VAEDecode",              "inputs": { "samples": ["3", 0], "vae": ["4", 2] } }
    "9": { "class_type": "SaveImage",              "inputs": { "filename_prefix": "ComfyUI", "images": ["8", 0] } }

  Rules:
    - Seeds must be changed between runs; ComfyUI caches and returns cached output if seed unchanged and inputs match.
    - Node IDs are strings even if they look like integers.
    - Linked inputs are always [string, int] arrays (node_id as string, output_index as int).
    - Output nodes (SaveImage) must be present or no images will be saved and history outputs will be empty.

Behavioral contracts:

POST /prompt -> validates all node class_types exist, all inputs type-check, then enqueues; returns immediately without waiting for execution
GET /history/{prompt_id} -> returns {} if prompt not yet in history (still queued/running); returns full entry when done (success or error)
GET /view -> streams file bytes; server rejects path traversal with 403; unknown file returns 404
POST /queue clear=true -> removes all pending items; does NOT affect running item
POST /interrupt -> signals executor thread to stop after current node; execution_interrupted sent via WS; history entry NOT written on interrupt
WebSocket /ws -> single persistent connection per session; server broadcasts all execution events to all connected clients with matching clientId; events for OTHER prompts may arrive if clientId not filtered client-side
GET /queue -> queue_running contains at most 1 tuple (ComfyUI is single-execution-thread); queue_pending is FIFO

Platform constraints:

Default port: 8188 (configurable via --port CLI arg)
No built-in authentication: ComfyUI has no auth layer; security is entirely by network isolation (bind to 127.0.0.1 or use reverse proxy with auth)
No TLS by default: plain ws:// and http:// only; TLS requires --tls-keyfile/--tls-certfile flags
CORS: configurable via --enable-cors-header; defaults to no CORS headers
Single execution thread: one prompt runs at a time; concurrent POST /prompt calls succeed (enqueue) but execute serially
No queue size limit by default: queue can grow unboundedly; no 429 response for queue full
Windows path handling: output filenames use forward slashes in subfolder field even on Windows
Python 3.10+ required (async features, match statements in execution.py)
History is in-memory: restarting ComfyUI loses all history; GET /history returns {} after restart for previously-submitted prompt_ids
No idempotency: submitting same prompt_id twice enqueues twice (server ignores duplicate prompt_id in queue — does not deduplicate)

Gotchas:

GET /history/{prompt_id} returns {} (empty object) for both "still running" and "server restarted and lost history" — callers CANNOT distinguish these cases via HTTP alone; must use WebSocket execution_error or execution_success to know definitively
Completion detection via polling: must check response != {} AND response[prompt_id]["status"]["completed"] == true
execution_interrupted does NOT write to history; GET /history/{prompt_id} returns {} forever for interrupted prompts — caller must track interruption state separately via WebSocket or timeout
OOM during execution: CUDA OOM causes execution_error WebSocket message with exception_type="torch.cuda.OutOfMemoryError"; server process survives but current prompt fails; subsequent prompts may also fail if memory not released; no automatic recovery
Stuck prompt (no WebSocket messages): can happen if server process hangs (deadlock in custom node); GET /history returns {} indefinitely; no heartbeat or keepalive mechanism in WebSocket protocol; only detection is wall-clock timeout
Seed caching: ComfyUI returns CACHED results (execution_cached message, no actual execution) if seed and all inputs are identical to a previously executed prompt; always change seed for new generation
Node class_type must exist: if a required custom node is not installed, POST /prompt returns 400 with node_errors containing "required_input_missing" or similar; server does not auto-install nodes
Binary WebSocket frame 8-byte header: bytes 0-3 = event_type (uint32 big-endian: 1=preview_image), bytes 4-7 = image_type (uint32 big-endian: 1=JPEG, 2=PNG); payload follows; only relevant for SaveImageWebsocket node
POST /interrupt with prompt_id field: the field exists in the API body parse but may be ignored in some versions; interrupt always affects current running prompt regardless of prompt_id
History outputs only contain data from output nodes (SaveImage, PreviewImage, etc.); intermediate nodes do not appear in outputs

Timeout and retry patterns:

Poll pattern (no WebSocket):
  1. POST /prompt -> get prompt_id
  2. Poll GET /history/{prompt_id} every 2s
  3. On non-empty response: check response[prompt_id]["status"]["completed"]
  4. Timeout at 600s: call POST /interrupt then POST /queue with delete=[prompt_id]
  5. On timeout: treat as error; image will NOT be in history

WebSocket pattern (preferred for live progress):
  1. Connect ws://host:port/ws?clientId=<uuid> BEFORE submitting prompt
  2. POST /prompt with client_id=<same uuid>
  3. Filter incoming WS messages for matching prompt_id
  4. On executing{node: null}: call GET /history/{prompt_id} to retrieve outputs
  5. On execution_error: extract exception_message; do NOT retry same prompt without fixing inputs
  6. On execution_interrupted: do NOT poll history; it will never appear
  7. Wall-clock timeout if no WS message received for 300s: send POST /interrupt

Retry policy:
  - 400 validation error: do NOT retry; fix workflow JSON
  - 5xx or connection error on POST /prompt: retry with exponential backoff (1s, 2s, 4s, max 3 attempts)
  - Stuck execution (timeout): POST /interrupt once; wait 10s; POST /queue {clear: true} if needed
  - OOM: wait 30s before retry to allow CUDA memory release; reduce batch_size or resolution in workflow

Critical assertions (testable):

assert "prompt_id" in response and len(response["prompt_id"]) > 0, "POST /prompt success must return non-empty prompt_id"
assert isinstance(response.get("node_errors"), dict), "POST /prompt response always contains node_errors dict"
assert response.get("status_code") != 404, "GET /history/{id} never returns 404; returns {} for unknown id"
history = get_history(prompt_id); assert history == {} or (prompt_id in history and "outputs" in history[prompt_id] and "status" in history[prompt_id]), "history entry shape invariant"
assert history[prompt_id]["status"]["status_str"] in ("success", "error"), "status_str is always 'success' or 'error'"
assert isinstance(history[prompt_id]["outputs"], dict), "outputs is always a dict (may be empty if no SaveImage node)"
for img in images_list: assert all(k in img for k in ("filename", "subfolder", "type")), "image entry must have filename, subfolder, type"
assert ws_executing_null_message["data"]["node"] is None, "completion signal: node field must be None not absent"
linked_input = node["inputs"][key]; assert isinstance(linked_input, list) and len(linked_input) == 2 and isinstance(linked_input[0], str) and isinstance(linked_input[1], int), "linked input must be [str_node_id, int_output_index]"

Minimal Rust client signature example (reqwest async):

use reqwest::Client;
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::Message};

// Submit prompt
async fn submit_prompt(
    client: &Client,
    base_url: &str,     // "http://127.0.0.1:8188"
    workflow: Value,    // serde_json::Value of the prompt object
    client_id: &str,    // uuid::Uuid::new_v4().to_simple().to_string()
) -> Result<String, reqwest::Error> {
    // Returns prompt_id string
    let body = json!({ "prompt": workflow, "client_id": client_id });
    let resp: Value = client.post(format!("{}/prompt", base_url))
        .json(&body).send().await?.json().await?;
    Ok(resp["prompt_id"].as_str().unwrap_or("").to_string())
}

// Poll history until complete or timeout
async fn poll_history(
    client: &Client,
    base_url: &str,
    prompt_id: &str,
    timeout_secs: u64,       // 600 per spec
    poll_interval_secs: u64, // 2
) -> Result<Value, anyhow::Error> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err(anyhow::anyhow!("timeout waiting for prompt_id={}", prompt_id));
        }
        let url = format!("{}/history/{}", base_url, prompt_id);
        let resp: Value = client.get(&url).send().await?.json().await?;
        if let Some(entry) = resp.get(prompt_id) {
            if entry["status"]["completed"].as_bool().unwrap_or(false) {
                return Ok(entry.clone());
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(poll_interval_secs)).await;
    }
}

// Download image bytes
async fn download_image(
    client: &Client,
    base_url: &str,
    filename: &str,
    subfolder: &str,
    image_type: &str,  // "output" | "temp" | "input"
) -> Result<bytes::Bytes, reqwest::Error> {
    client.get(format!("{}/view", base_url))
        .query(&[("filename", filename), ("subfolder", subfolder), ("type", image_type)])
        .send().await?.bytes().await
}

// Interrupt running prompt
async fn interrupt(
    client: &Client,
    base_url: &str,
    prompt_id: Option<&str>,
) -> Result<(), reqwest::Error> {
    let body = match prompt_id {
        Some(id) => json!({ "prompt_id": id }),
        None => json!({}),
    };
    client.post(format!("{}/interrupt", base_url))
        .json(&body).send().await?.error_for_status()?;
    Ok(())
}

// Clear queue
async fn clear_queue(client: &Client, base_url: &str) -> Result<(), reqwest::Error> {
    client.post(format!("{}/queue", base_url))
        .json(&json!({ "clear": true })).send().await?.error_for_status()?;
    Ok(())
}

// WebSocket completion tracking (alternative to polling)
// ws_url: "ws://127.0.0.1:8188/ws?clientId=<uuid>"
// Completion: filter for type=="executing" AND data.node==null AND data.prompt_id==target_id
// Error: filter for type=="execution_error" AND data.prompt_id==target_id
// Interrupt: filter for type=="execution_interrupted" AND data.prompt_id==target_id
// After completion: call poll_history once to get outputs (not from WS directly)
