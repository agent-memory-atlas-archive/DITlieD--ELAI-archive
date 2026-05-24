use elai_plugins::mcp::gate::{GatedCatalog, ToolGate, ToolGateConfig, ToolGateStateConfig};
use elai_plugins::mcp::lazy::LazySchemaCache;
use elai_plugins::McpToolSpec;
use rustc_hash::FxHashMap;
use serde_json::json;
// HashMap retained for `planning_gate` (ToolGateConfig.states uses HashMap;
// gate.rs is out of B-15 17-file scope, scheduled for B-15-followup).
use std::collections::HashMap;

fn synthetic_catalog() -> Vec<McpToolSpec> {
    vec![
        McpToolSpec { name: "knowledge.search".into(), description: "Search the knowledge graph".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "knowledge.symbol_impact".into(), description: "Compute impact radius for a symbol".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "fs.read".into(), description: "Read a file from the filesystem".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "fs.edit".into(), description: "Edit a file in place".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "fs.write".into(), description: "Write a new file".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "fs.grep".into(), description: "Search files for a pattern".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "git.diff".into(), description: "Show git diff".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "git.commit".into(), description: "Create a git commit".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "runtime.invoke".into(), description: "Invoke a runtime backend".into(), input_schema: json!({"type":"object"}) },
        McpToolSpec { name: "runtime.health".into(), description: "Check health of a runtime backend".into(), input_schema: json!({"type":"object"}) },
    ]
}
fn planning_gate() -> ToolGate {
    let mut states = HashMap::new();
    states.insert("planning".to_string(), ToolGateStateConfig {
        eager: vec!["knowledge.search".into(), "knowledge.symbol_impact".into()],
        gated: vec!["fs.read".into(), "fs.grep".into(), "git.diff".into()],
        denied: vec!["git.commit".into(), "runtime.invoke".into()],
    });
    ToolGate::from_config(ToolGateConfig { states, max_eager_schemas: 4 })
}
#[test]
fn tool_gate_filter_state() {
    let gate = planning_gate();
    let catalog = synthetic_catalog();
    let result: GatedCatalog = gate.filter_for_state("planning", &catalog);
    let eager_names: Vec<&str> = result.eager_schemas.iter().map(|s| s.name.as_str()).collect();
    assert!(eager_names.contains(&"knowledge.search"), "knowledge.search must be eager");
    assert!(eager_names.contains(&"knowledge.symbol_impact"), "knowledge.symbol_impact must be eager");
    assert_eq!(result.eager_schemas.len(), 2);
    let gated_names: Vec<&str> = result.gated_summaries.iter().map(|s| s.name.as_str()).collect();
    assert!(gated_names.contains(&"fs.read"), "fs.read must be gated");
    assert!(gated_names.contains(&"fs.grep"), "fs.grep must be gated");
    assert!(gated_names.contains(&"git.diff"), "git.diff must be gated");
    assert!(result.denied_names.contains("git.commit"), "git.commit must be denied");
    assert!(result.denied_names.contains("runtime.invoke"), "runtime.invoke must be denied");
    assert_eq!(result.denied_names.len(), 2);
}
#[tokio::test]
async fn tool_gate_lru_eager() {
    let cache = LazySchemaCache::new(3);
    let catalog = synthetic_catalog();
    // B-15 HashMap ban cascade: LazySchemaCache::resolve now takes FxHashMap.
    let registry: FxHashMap<String, McpToolSpec> = catalog.into_iter().map(|s| (s.name.clone(), s)).collect();
    cache.resolve("knowledge.search", &registry).await.expect("resolve 1");
    cache.resolve("fs.read", &registry).await.expect("resolve 2");
    cache.resolve("fs.edit", &registry).await.expect("resolve 3");
    cache.resolve("git.diff", &registry).await.expect("resolve 4");
    assert_eq!(cache.len(), 3, "cache must not exceed max_eager_schemas");
    assert_eq!(cache.cache_misses(), 4);
    assert_eq!(cache.cache_hits(), 0);
    cache.resolve("fs.read", &registry).await.expect("re-resolve");
    assert_eq!(cache.cache_hits(), 1);
}
#[tokio::test]
async fn tool_gate_resolve_schema() {
    let cache = LazySchemaCache::new(16);
    let catalog = synthetic_catalog();
    let registry: FxHashMap<String, McpToolSpec> = catalog.iter().map(|s| (s.name.clone(), s.clone())).collect();
    for original in &catalog {
        let resolved = cache.resolve(&original.name, &registry).await
            .unwrap_or_else(|_| panic!("resolve failed for {}", original.name));
        assert_eq!(resolved.name, original.name);
        assert_eq!(resolved.description, original.description);
        assert_eq!(resolved.input_schema.to_string(), original.input_schema.to_string());
    }
}
#[test]
fn tool_gate_denied_structured_error() {
    let gate = planning_gate();
    let denied = gate.check_tool_allowed("planning", "git.commit");
    assert!(denied.is_err(), "denied tool must return Err");
    let err = denied.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("denied") || msg.contains("git.commit"), "error message: {msg}");
    assert!(gate.check_tool_allowed("planning", "fs.read").is_ok());
    assert!(gate.check_tool_allowed("planning", "knowledge.search").is_ok());
    let absent = ToolGate::safe_default();
    assert!(absent.check_tool_allowed("any_state", "git.commit").is_ok(), "safe default allows all");
    assert_eq!(absent.reason(), "manifest_absent");
}
// C-35 N3 wiring tests: verify DefaultMcpBridge::assemble_prompt + the
// ToolGateSsbEmitter trait + the SafeDefault emission-skip rule. These
// exercise the wiring half of the plan, not the library shape.

#[derive(Default)]
struct CapturingEmitter {
    rows: std::sync::Mutex<Vec<(String, String, String, i64)>>,
}
impl elai_plugins::ToolGateSsbEmitter for CapturingEmitter {
    fn emit(
        &self,
        action_kind: &str,
        tool_name: &str,
        fsm_state: &str,
        prompt_tokens_saved: i64,
    ) -> Result<(), String> {
        self.rows.lock().expect("lock").push((
            action_kind.to_owned(),
            tool_name.to_owned(),
            fsm_state.to_owned(),
            prompt_tokens_saved,
        ));
        Ok(())
    }
}

#[test]
fn assemble_prompt_emits_tool_gated_and_tool_eager() {
    use std::sync::Arc;
    let gate = Arc::new(planning_gate());
    let cache = Arc::new(LazySchemaCache::new(4));
    let emitter = Arc::new(CapturingEmitter::default());
    let bridge = elai_plugins::DefaultMcpBridge::default()
        .with_tool_gate(gate, cache)
        .with_ssb_emitter(emitter.clone());
    let catalog = synthetic_catalog();
    let result = bridge.assemble_prompt("planning", &catalog, 4);
    assert!(!result.eager_schemas.is_empty(), "eager schemas non-empty in planning state");
    assert!(!result.gated_summaries.is_empty(), "gated summaries non-empty");
    assert!(result.prompt_tokens_saved > 0, "tokens_saved must be > 0");
    let rows = emitter.rows.lock().expect("lock");
    let eager_count = rows.iter().filter(|r| r.0 == "tool_eager").count();
    let gated_count = rows.iter().filter(|r| r.0 == "tool_gated").count();
    assert_eq!(eager_count, result.eager_schemas.len(), "one tool_eager row per eager schema");
    assert_eq!(gated_count, result.gated_summaries.len(), "one tool_gated row per gated summary");
    for (kind, _, fsm, _) in rows.iter() {
        assert_eq!(fsm, "planning", "row carries fsm_state from call");
        assert!(matches!(kind.as_str(), "tool_gated" | "tool_eager"));
    }
}

#[test]
fn assemble_prompt_safe_default_skips_emission() {
    use std::sync::Arc;
    let gate = Arc::new(elai_plugins::ToolGate::safe_default());
    let cache = Arc::new(LazySchemaCache::new(4));
    let emitter = Arc::new(CapturingEmitter::default());
    let bridge = elai_plugins::DefaultMcpBridge::default()
        .with_tool_gate(gate, cache)
        .with_ssb_emitter(emitter.clone());
    let catalog = synthetic_catalog();
    let result = bridge.assemble_prompt("planning", &catalog, 4);
    // Safe-default: ALL tools land in gated_summaries with empty eager.
    assert_eq!(result.eager_schemas.len(), 0);
    assert_eq!(result.gated_summaries.len(), catalog.len());
    // C-35 schema lock: NO emission on SafeDefault path.
    let rows = emitter.rows.lock().expect("lock");
    assert!(rows.is_empty(), "SafeDefault must skip SSB emission; got {} rows", rows.len());
}

#[test]
fn tool_gate_tokens_saved_metric() {
    let gate = planning_gate();
    let catalog = synthetic_catalog();
    let gated: GatedCatalog = gate.filter_for_state("planning", &catalog);
    let full_size: usize = catalog.iter()
        .map(|s| serde_json::to_string(s).unwrap_or_default().len()).sum();
    let eager_size: usize = gated.eager_schemas.iter()
        .map(|s| serde_json::to_string(s).unwrap_or_default().len()).sum();
    let summary_size: usize = gated.gated_summaries.iter()
        .map(|s| format!("{}: {}", s.name, s.summary).len()).sum();
    let gated_size = eager_size + summary_size;
    const CPT: usize = 4;
    let tokens_saved = (full_size.saturating_sub(gated_size)) / CPT;
    assert!(tokens_saved > 0, "tokens_saved={tokens_saved} full={full_size} gated={gated_size}");
    let reported = gated.prompt_tokens_saved(full_size, CPT);
    assert_eq!(reported, tokens_saved);
}
