//! I-32: schema-first role contracts for factual/code agent roles.
//!
//! Replaces narrative persona blocks ("You are an interactive agent who…") in the
//! system prompts for factual/code roles with typed I/O constraint lists. Alignment-
//! dependent roles (Auditor, IdeaCritic, InjectionClassifier, Auditor* variants) are
//! LEFT UNCHANGED — their hostile-critic/adversarial framing depends on the persona.
//!
//! GROUNDING (IC-17): persona prompting measurably harms factual/code accuracy (MMLU
//! -3.6..-5.3pp; MT-Bench coding -0.65) while providing no mechanical compliance
//! guarantee. The flip is evidence-gated per ID-5 via an A/B harness.
//!
//! COMPOSE-NOT-RESTATE (GD-1): apply_to_system_prompt mutates the `sections`
//! Vec produced by `report_fns::load_system_prompt`. It does not re-implement
//! the system-prompt assembly pipeline.

use crate::agent_pool::AgentRole;

/// Schema-first role contract for a factual/code agent role.
///
/// Replaces the narrative persona block with:
/// 1. A one-line role header.
/// 2. Typed I/O constraints (what the role accepts and must produce).
/// 3. An explicit capability scope (which tool categories are in-scope).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleContract {
    /// The role this contract governs.
    pub role: AgentRole,
    /// Typed constraint list (e.g. "output: valid Rust that compiles").
    /// One entry per constraint; shown as a bullet list in the assembled prompt.
    pub typed_constraints: Vec<String>,
    /// I/O schema description injected as the opening context block.
    pub io_schema: String,
    /// Tool categories that are IN-SCOPE for this role.
    /// Used by the supervisor role-scope gate to block out-of-scope tool calls.
    pub allowed_tool_categories: Vec<String>,
}

impl RoleContract {
    /// Build the system-prompt replacement text for this contract.
    /// The returned string replaces the narrative intro section in the assembled
    /// `Vec<String>` sections. Format is header + io_schema + constraints bullet list.
    pub fn prompt_text(&self) -> String {
        let mut out = format!(
            "# Role: {}\n\n{}\n\nConstraints:\n",
            role_display_name(self.role),
            self.io_schema
        );
        for c in &self.typed_constraints {
            out.push_str(&format!("- {c}\n"));
        }
        out
    }

    /// Returns `true` if the given tool name is in-scope for this role.
    /// An empty `allowed_tool_categories` list is treated as "all allowed"
    /// (the contract does not restrict tools for this role).
    pub fn tool_allowed(&self, tool_name: &str) -> bool {
        if self.allowed_tool_categories.is_empty() {
            return true;
        }
        self.allowed_tool_categories
            .iter()
            .any(|cat| tool_name.starts_with(cat.as_str()))
    }
}

fn role_display_name(role: AgentRole) -> &'static str {
    match role {
        AgentRole::Executor => "Executor",
        AgentRole::Researcher => "Researcher",
        AgentRole::ContextCompiler => "ContextCompiler",
        AgentRole::Designer => "Designer",
        AgentRole::MemoryWriter => "MemoryWriter",
        AgentRole::PromptEngineer => "PromptEngineer",
        AgentRole::Auditor => "Auditor",
        AgentRole::IdeaFramer => "IdeaFramer",
        AgentRole::IdeaCritic => "IdeaCritic",
        AgentRole::MediaValidator => "MediaValidator",
        AgentRole::AuditorConcepts => "AuditorConcepts",
        AgentRole::AuditorPhases => "AuditorPhases",
        AgentRole::AuditorSteps => "AuditorSteps",
        AgentRole::AuditorBabySteps => "AuditorBabySteps",
        AgentRole::AuditorBabyStepDetail => "AuditorBabyStepDetail",
        AgentRole::AuditorProductionGrade => "AuditorProductionGrade",
        AgentRole::InjectionClassifier => "InjectionClassifier",
    }
}

/// Registry of role contracts keyed by `AgentRole`.
///
/// Immutable after construction. The production store is built once at process
/// start via [`RoleContractStore::default`] and shared read-only.
#[derive(Debug, Clone)]
pub struct RoleContractStore {
    contracts: Vec<RoleContract>,
}

impl RoleContractStore {
    /// Look up the contract for `role`. Returns `None` for alignment-dependent
    /// roles that intentionally retain their persona (Auditor*, IdeaCritic,
    /// InjectionClassifier).
    pub fn get(&self, role: AgentRole) -> Option<&RoleContract> {
        self.contracts.iter().find(|c| c.role == role)
    }
}

impl Default for RoleContractStore {
    fn default() -> Self {
        Self {
            contracts: default_contracts(),
        }
    }
}

/// Build the canonical set of role contracts for factual/code roles.
/// Alignment-dependent roles are intentionally ABSENT: they keep their persona.
fn default_contracts() -> Vec<RoleContract> {
    vec![
        RoleContract {
            role: AgentRole::Executor,
            io_schema: "Input: a task description and context. \
                        Output: Rust/Python/TypeScript code or shell commands \
                        that satisfy the task. All output must compile or run clean."
                .into(),
            typed_constraints: vec![
                "Produce only code or commands — no conversational padding.".into(),
                "Every code block must be syntactically complete; no ellipsis stubs.".into(),
                "Use ? / Result propagation; no .unwrap() outside test scope.".into(),
                "Cite the grounding source for any external API or library claim.".into(),
            ],
            allowed_tool_categories: vec![
                "file_write".into(),
                "file_read".into(),
                "bash_tool_use".into(),
                "edit".into(),
                "read".into(),
                "grep".into(),
                "glob".into(),
            ],
        },
        RoleContract {
            role: AgentRole::Researcher,
            io_schema: "Input: a research question or topic. \
                        Output: a structured report with citations. \
                        Every factual claim carries a source reference."
                .into(),
            typed_constraints: vec![
                "Label unverified claims explicitly (Ungrounded conjecture:).".into(),
                "Use WebSearch / WebFetch to ground external API or version claims.".into(),
                "Produce structured output (sections + citations), not stream-of-thought.".into(),
            ],
            allowed_tool_categories: vec![
                "web_search".into(),
                "web_fetch".into(),
                "file_read".into(),
                "read".into(),
                "grep".into(),
                "glob".into(),
            ],
        },
        RoleContract {
            role: AgentRole::ContextCompiler,
            io_schema: "Input: a set of file paths and a task description. \
                        Output: a structured context package (context-package-*.md) \
                        listing symbols, entry points, integration surface, and gaps."
                .into(),
            typed_constraints: vec![
                "Read source files; do not invent symbol names.".into(),
                "Emit only grounded facts (code just read = High confidence).".into(),
                "Output follows the context-package template; no freeform prose.".into(),
            ],
            allowed_tool_categories: vec![
                "file_read".into(),
                "file_write".into(),
                "read".into(),
                "grep".into(),
                "glob".into(),
                "bash_tool_use".into(),
            ],
        },
        RoleContract {
            role: AgentRole::Designer,
            io_schema: "Input: a product or architecture brief. \
                        Output: a structured design document with \
                        tradeoffs, acceptance criteria, and implementation notes."
                .into(),
            typed_constraints: vec![
                "Two approaches minimum before any recommendation.".into(),
                "Every design claim cites prior art or measured data.".into(),
                "No single-option recommendations without stated alternatives.".into(),
            ],
            allowed_tool_categories: vec![
                "file_write".into(),
                "file_read".into(),
                "read".into(),
                "web_search".into(),
                "web_fetch".into(),
            ],
        },
        RoleContract {
            role: AgentRole::MemoryWriter,
            io_schema: "Input: a session summary and changed-file list. \
                        Output: RAG-indexed memory-bank entries via mcp_memorize / mcp_log_change. \
                        Never writes memory-bank files directly."
                .into(),
            typed_constraints: vec![
                "All writes go through mcp_memorize or mcp_log_change (never direct file edit).".into(),
                "One entry per changed symbol; max 5 lines per changelog entry.".into(),
                "Do not delete or truncate existing entries without explicit instruction.".into(),
            ],
            allowed_tool_categories: vec![
                "file_read".into(),
                "read".into(),
                "grep".into(),
            ],
        },
    ]
}

/// Parse a snake_case role string (as stored in `Directive::initiator`) back to
/// an [`AgentRole`]. Returns `None` for unrecognised strings (the gate treats
/// unknown roles as if they were alignment-dependent: no blocking).
///
/// Mirrors the forward map in `agent_pool::agent_role_to_snake_case`.
pub fn agent_role_from_snake_case(s: &str) -> Option<AgentRole> {
    Some(match s {
        "executor" => AgentRole::Executor,
        "auditor" => AgentRole::Auditor,
        "researcher" => AgentRole::Researcher,
        "designer" => AgentRole::Designer,
        "prompt_engineer" => AgentRole::PromptEngineer,
        "context_compiler" => AgentRole::ContextCompiler,
        "memory_writer" => AgentRole::MemoryWriter,
        "media_validator" => AgentRole::MediaValidator,
        "idea_framer" => AgentRole::IdeaFramer,
        "idea_critic" => AgentRole::IdeaCritic,
        "auditor_concepts" => AgentRole::AuditorConcepts,
        "auditor_phases" => AgentRole::AuditorPhases,
        "auditor_steps" => AgentRole::AuditorSteps,
        "auditor_babysteps" => AgentRole::AuditorBabySteps,
        "auditor_babystep_detail" => AgentRole::AuditorBabyStepDetail,
        "auditor_production_grade" => AgentRole::AuditorProductionGrade,
        "injection_classifier" => AgentRole::InjectionClassifier,
        _ => return None,
    })
}

/// Apply the role contract to `sections` by replacing the narrative intro block.
///
/// For RETAINED roles (Auditor, IdeaCritic, InjectionClassifier, Auditor* variants)
/// the sections slice is LEFT UNCHANGED and this function returns `PersonaRetained`.
/// For STRIPPED roles the first section starting with "You are" is replaced with
/// the contract's `prompt_text()`.
///
/// Returns a flag indicating whether the persona was stripped or retained,
/// so the caller can emit the correct `RoleContractApplied` SSB Kind.
pub fn apply_to_system_prompt(
    sections: &mut Vec<String>,
    role: AgentRole,
    store: &RoleContractStore,
) -> ApplyOutcome {
    let Some(contract) = store.get(role) else {
        // Alignment-dependent role: leave sections untouched.
        return ApplyOutcome::PersonaRetained;
    };

    // Locate the narrative intro section (the "You are…" opener from
    // simple_intro_default.md). If not found (already stripped or missing),
    // append the contract text as a new first section.
    let replacement = contract.prompt_text();
    if let Some(idx) = sections.iter().position(|s| s.contains("You are")) {
        sections[idx] = replacement;
    } else {
        sections.insert(0, replacement);
    }
    ApplyOutcome::PersonaStripped
}

/// Outcome of a [`apply_to_system_prompt`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// Narrative persona block was replaced with the schema contract.
    PersonaStripped,
    /// Role is alignment-dependent; sections were left unchanged.
    PersonaRetained,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executor_contract_is_present() {
        let store = RoleContractStore::default();
        let c = store.get(AgentRole::Executor).expect("Executor contract must exist");
        assert!(!c.io_schema.is_empty(), "io_schema must not be empty");
        assert!(
            !c.typed_constraints.is_empty(),
            "typed_constraints must not be empty"
        );
    }

    #[test]
    fn auditor_is_not_in_store() {
        let store = RoleContractStore::default();
        assert!(
            store.get(AgentRole::Auditor).is_none(),
            "Auditor must be absent (alignment-dependent, persona retained)"
        );
        assert!(
            store.get(AgentRole::IdeaCritic).is_none(),
            "IdeaCritic must be absent (alignment-dependent)"
        );
        assert!(
            store.get(AgentRole::InjectionClassifier).is_none(),
            "InjectionClassifier must be absent (alignment-dependent)"
        );
    }

    #[test]
    fn apply_strips_persona_for_executor() {
        let store = RoleContractStore::default();
        let mut sections = vec![
            "You are an interactive agent who helps with code tasks.".to_string(),
            "# Claude instructions\nsome rules".to_string(),
        ];
        let outcome = apply_to_system_prompt(&mut sections, AgentRole::Executor, &store);
        assert_eq!(outcome, ApplyOutcome::PersonaStripped);
        assert!(
            !sections[0].contains("You are an interactive agent"),
            "persona text must be replaced"
        );
        assert!(
            sections[0].contains("# Role: Executor"),
            "contract header must be present in sections[0]; got: {:?}",
            sections[0]
        );
    }

    #[test]
    fn apply_retains_persona_for_auditor() {
        let store = RoleContractStore::default();
        let original = "You are a hostile critic who must find flaws.".to_string();
        let mut sections = vec![original.clone()];
        let outcome = apply_to_system_prompt(&mut sections, AgentRole::Auditor, &store);
        assert_eq!(outcome, ApplyOutcome::PersonaRetained);
        assert_eq!(sections[0], original, "auditor persona must not be modified");
    }

    #[test]
    fn tool_allowed_respects_role_scope() {
        let store = RoleContractStore::default();
        let researcher = store.get(AgentRole::Researcher).expect("researcher contract");
        assert!(researcher.tool_allowed("web_search"), "web_search in scope for researcher");
        assert!(
            !researcher.tool_allowed("file_write"),
            "file_write out of scope for researcher"
        );
        // Executor: file_write allowed.
        let executor = store.get(AgentRole::Executor).expect("executor contract");
        assert!(executor.tool_allowed("file_write"), "file_write in scope for executor");
    }
}
