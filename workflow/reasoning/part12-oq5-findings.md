PART12 OQ5 — server.py /v1/settings key acceptance investigation

Scope: verify the 7 keys PART11 frontend writes via stub.set_sidecar_settings(...) round-trip through server.py /v1/settings.

Method: grep _engine class definition (server.py:1372 InferenceEngine) and __init__ (1379-1502) for attribute names; classify each key as RUNTIME_SETTABLE, RUNTIME_DISABLE_ONLY, BOOT_FLAG_ONLY, or NON_ENGINE.

Frontend write sites (verified):
- SettingsWindow.svelte:144  turboquant_bits
- SettingsWindow.svelte:186  vram_budget_gb
- SettingsWindow.svelte:191  cloud_api_key
- SettingsWindow.svelte:196  cloud_spend_cap_usd
- SettingsWindow.svelte:315  dola
- SettingsWindow.svelte:320  etc
- SettingsWindow.svelte:325  steering

Pre-PART12 server state (server.py:2887): updatable = {"max_new_tokens"}. All 7 frontend keys silently dropped.

Findings:

RUNTIME_SETTABLE
- max_new_tokens     attr=_engine.max_new_tokens (server.py:1450)             reason: pure scalar, read at request time (server.py:1656,1684)

RUNTIME_DISABLE_ONLY
- dola               attr=_engine.dola_enabled  (server.py:1448)             reason: setting False disables generation pathway (1718,1887). Setting True is INERT unless _dola_processor was constructed at boot (1465-1471). Re-enable requires restart.
- etc                attr=_engine.etc_enabled   (server.py:1475)             reason: setting False disables ETC LogitsProcessor application. Setting True is INERT unless _etc_processor was constructed at boot (1487).
- steering           attr=_engine.steering_enabled (server.py:1456)          reason: setting False short-circuits steering_engine.is_active() check (807,835). Setting True is INERT unless steering_engine was constructed at boot (1458-1461).

BOOT_FLAG_ONLY
- turboquant_bits    stored at _engine._turboquant_bits (server.py:1500)     reason: KVCompressor(bit_width=...) instantiated at __init__ (1453); init_prefix_kv_cache(...) called once at __init__ (1502); changing _turboquant_bits at runtime does not rebuild compressor or cache.

NON_ENGINE
- vram_budget_gb     no _engine attribute                                    reason: VRAM budgeting is host/orchestration concern, not inference-engine concern. Stored in module-level _cloud_settings dict.
- cloud_api_key      no _engine attribute                                    reason: cloud provider key is for the orchestrator's cloud-fallback path, not the local sidecar. Stored in module dict, redacted from GET response.
- cloud_spend_cap_usd no _engine attribute                                   reason: spend cap belongs to cloud-fallback budget tracking; not inference. Stored in module dict.

VERDICT — handle_set_settings categorization (server.py:2872):
  RUNTIME_SETTABLE     = {"max_new_tokens"}
  RUNTIME_DISABLE_ONLY = {"dola": "dola_enabled", "etc": "etc_enabled", "steering": "steering_enabled"}
  BOOT_FLAG_ONLY       = {"turboquant_bits"}
  NON_ENGINE           = {"vram_budget_gb", "cloud_api_key", "cloud_spend_cap_usd"}

Response contract (post-PART12):
  {
    "applied": { "<key>": <value>, ... },                  # actually written to engine or stored in _cloud_settings
    "boot_flags_require_restart": [ "<key>", ... ],        # acknowledged but inert until restart
    "silently_skipped": [ "<key>", ... ]                   # unknown keys (logged warning)
  }

Side-effect: handle_get_settings (server.py:2842) extended with cloud_api_key_present (bool, not raw value) plus vram_budget_gb and cloud_spend_cap_usd values, so the dashboard can re-seed its local state from the sidecar after a restart without leaking the key over the wire. cloud_api_key itself is never returned.

Out of scope this session:
- Hot-rebuild of _dola_processor / steering_engine / KVCompressor when toggling-on at runtime. Requires deeper engine surgery; tracked as future OQ.
- Persisting _cloud_settings across restarts. Requires a config file or env var fallback; out of scope.

Reliability: HIGH. _engine attribute names verified in server.py:1379-1502 by direct read; no reliance on training data.
