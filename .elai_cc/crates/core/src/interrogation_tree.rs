// InterrogationTree types for Layer 3 reasoning (spec: reasoning-architecture-design.txt LAYER 3).
//
// Language-parameterized decision trees stored in rules.db.
// Traversal: reasoning crate. Storage: rules crate.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum depth of an interrogation tree (spec: 3-7 questions).
pub const MAX_TREE_DEPTH: usize = 7;

/// Minimum choices per question node.
pub const MIN_CHOICES: usize = 3;

/// Maximum choices per question node (spec: 3-6 options).
pub const MAX_CHOICES: usize = 6;

/// Approximate token cost per question (spec: ~310 tokens input + ~10 output).
pub const TOKENS_PER_QUESTION: usize = 310;

/// Maximum total token budget for a single tree traversal (spec: 930-2170).
pub const MAX_TRAVERSAL_TOKENS: usize = 2170;

// ---------------------------------------------------------------------------
// ChoiceId: unique identifier for a choice within a question node
// ---------------------------------------------------------------------------

/// Identifier for a choice (A, B, C, D, E, F).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ChoiceId(pub String);

impl ChoiceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// Choice: a single option within a question node
// ---------------------------------------------------------------------------

/// A single selectable option within a QuestionNode.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Choice {
    pub id: ChoiceId,
    /// Human-readable label (e.g., "wrong algorithm", "missing edge case").
    pub label: String,
    /// System's prior estimate of this choice being correct (from DKG analysis).
    /// Range: 0.0-1.0. Used for sorting/highlighting, not hard filtering.
    pub confidence_hint: f32,
}

// ---------------------------------------------------------------------------
// FixTemplate: leaf node output — assembled into a fix instruction
// ---------------------------------------------------------------------------

/// A fix instruction template with {placeholders} for code context.
/// Produced at leaf nodes of the interrogation tree.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FixTemplate {
    /// Template string with {placeholders} (e.g., "modify {function} at line {line}").
    pub instruction: String,
    /// Category tag for the fix (e.g., "signature_change", "add_branch", "type_fix").
    pub category: String,
}

// ---------------------------------------------------------------------------
// QuestionNode: a single question in the decision tree
// ---------------------------------------------------------------------------

/// A single question in the interrogation tree.
/// Contains a question template, choices, conditional children, and optional terminal.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestionNode {
    /// Question text with {placeholders} for code context
    /// (e.g., "The error in {function} is in: ...").
    pub question_template: String,
    /// Available choices (3-6 options per spec).
    pub choices: Vec<Choice>,
    /// Conditional follow-up questions keyed by ChoiceId.
    /// A choice not present here AND without a terminal is a dead end (should not happen
    /// in well-formed trees).
    pub children: HashMap<ChoiceId, QuestionNode>,
    /// If Some, this node is a leaf — the traversal produces this fix instruction.
    /// Mutually exclusive with children for a given choice path (children take priority).
    pub terminal: Option<FixTemplate>,
}

// ---------------------------------------------------------------------------
// InterrogationTree: top-level tree with language + domain tagging
// ---------------------------------------------------------------------------

/// A complete interrogation tree for a specific language and optional domain.
/// Stored in rules.db as JSON, retrieved by (language, domain) key.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterrogationTree {
    /// Unique identifier for this tree (e.g., "rust-lifetime", "python-import").
    pub tree_id: String,
    /// Root question node — entry point for traversal.
    pub root: QuestionNode,
    /// Language this tree applies to (from ProjectProfile).
    pub language: String,
    /// Optional domain constraint (e.g., "web_backend", "ml_pipeline").
    /// None = applies to all domains for this language.
    pub domain: Option<String>,
    /// Human-readable description of what this tree diagnoses.
    pub description: String,
    /// Schema version for forward compatibility.
    pub version: u32,
}

/// Current schema version for InterrogationTree serialization.
pub const TREE_SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// TraversalState: tracks progress through an interrogation tree
// ---------------------------------------------------------------------------

/// Tracks the current state of a tree traversal (for reasoning crate).
#[derive(Debug, Clone)]
pub struct TraversalState {
    /// Sequence of (question_template, chosen ChoiceId) pairs.
    pub answers: Vec<(String, ChoiceId)>,
    /// Current depth in the tree.
    pub depth: usize,
    /// Accumulated token cost estimate.
    pub token_cost: usize,
}

impl TraversalState {
    pub fn new() -> Self {
        Self {
            answers: Vec::new(),
            depth: 0,
            token_cost: 0,
        }
    }

    /// Whether the traversal has exceeded the maximum depth.
    pub fn is_max_depth(&self) -> bool {
        self.depth >= MAX_TREE_DEPTH
    }

    /// Whether the traversal has exceeded the token budget.
    pub fn over_budget(&self) -> bool {
        self.token_cost > MAX_TRAVERSAL_TOKENS
    }
}

impl Default for TraversalState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// TraversalOutcome: result of completing a tree traversal
// ---------------------------------------------------------------------------

/// Result of completing (or aborting) a tree traversal.
#[derive(Debug, Clone)]
pub enum TraversalOutcome {
    /// Reached a leaf node with a fix template.
    Fix(FixTemplate),
    /// Reached max depth or budget without a terminal node.
    Exhausted {
        partial_path: Vec<(String, ChoiceId)>,
        reason: String,
    },
    /// No matching choice for the given answer (malformed tree or bad input).
    NoMatch {
        question: String,
        attempted_choice: ChoiceId,
    },
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

impl InterrogationTree {
    /// Validate that the tree is well-formed:
    /// - At least MIN_CHOICES choices per node
    /// - At most MAX_CHOICES choices per node
    /// - Every child key references a valid choice id
    /// - Depth does not exceed MAX_TREE_DEPTH
    /// - version matches current schema
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.tree_id.is_empty() {
            errors.push("tree_id is empty".to_string());
        }
        if self.language.is_empty() {
            errors.push("language is empty".to_string());
        }
        if self.version != TREE_SCHEMA_VERSION {
            errors.push(format!(
                "version mismatch: tree has {}, expected {}",
                self.version, TREE_SCHEMA_VERSION
            ));
        }

        validate_node(&self.root, 0, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn validate_node(node: &QuestionNode, depth: usize, errors: &mut Vec<String>) {
    if depth > MAX_TREE_DEPTH {
        errors.push(format!("depth {} exceeds MAX_TREE_DEPTH {}", depth, MAX_TREE_DEPTH));
        return;
    }

    if node.choices.len() < MIN_CHOICES {
        errors.push(format!(
            "node '{}' has {} choices, minimum is {}",
            truncate_question(&node.question_template),
            node.choices.len(),
            MIN_CHOICES
        ));
    }

    if node.choices.len() > MAX_CHOICES {
        errors.push(format!(
            "node '{}' has {} choices, maximum is {}",
            truncate_question(&node.question_template),
            node.choices.len(),
            MAX_CHOICES
        ));
    }

    let choice_ids: Vec<&str> = node.choices.iter().map(|c| c.id.as_str()).collect();
    for (child_id, child_node) in &node.children {
        if !choice_ids.contains(&child_id.as_str()) {
            errors.push(format!(
                "child key '{}' not found in choices of '{}'",
                child_id.as_str(),
                truncate_question(&node.question_template)
            ));
        }
        validate_node(child_node, depth + 1, errors);
    }

    for choice in &node.choices {
        if choice.confidence_hint < 0.0 || choice.confidence_hint > 1.0 {
            errors.push(format!(
                "choice '{}' confidence_hint {} out of [0.0, 1.0]",
                choice.label, choice.confidence_hint
            ));
        }
    }
}

fn truncate_question(q: &str) -> String {
    if q.len() <= 40 {
        q.to_string()
    } else {
        format!("{}...", &q[..37])
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_choices(labels: &[&str]) -> Vec<Choice> {
        labels
            .iter()
            .enumerate()
            .map(|(i, label)| Choice {
                id: ChoiceId::new(format!("{}", (b'A' + i as u8) as char)),
                label: label.to_string(),
                confidence_hint: 0.2,
            })
            .collect()
    }

    fn leaf_node(question: &str, labels: &[&str], fix: &str) -> QuestionNode {
        QuestionNode {
            question_template: question.to_string(),
            choices: make_choices(labels),
            children: HashMap::new(),
            terminal: Some(FixTemplate {
                instruction: fix.to_string(),
                category: "test".to_string(),
            }),
        }
    }

    fn simple_tree() -> InterrogationTree {
        let child_b = leaf_node(
            "The body logic error is:",
            &["wrong algorithm", "missing edge case", "wrong variable"],
            "fix body logic at {line}",
        );

        let mut root_children = HashMap::new();
        root_children.insert(ChoiceId::new("B"), child_b);

        let root = QuestionNode {
            question_template: "The error is in:".to_string(),
            choices: make_choices(&[
                "function signature",
                "function body logic",
                "return value",
            ]),
            children: root_children,
            terminal: Some(FixTemplate {
                instruction: "generic fix".to_string(),
                category: "fallback".to_string(),
            }),
        };

        InterrogationTree {
            tree_id: "test-tree".to_string(),
            root,
            language: "rust".to_string(),
            domain: None,
            description: "Test tree".to_string(),
            version: TREE_SCHEMA_VERSION,
        }
    }

    #[test]
    fn validate_well_formed_tree() {
        let tree = simple_tree();
        assert!(tree.validate().is_ok());
    }

    #[test]
    fn validate_empty_tree_id() {
        let mut tree = simple_tree();
        tree.tree_id = String::new();
        let errs = tree.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("tree_id is empty")));
    }

    #[test]
    fn validate_too_few_choices() {
        let mut tree = simple_tree();
        tree.root.choices = vec![
            Choice { id: ChoiceId::new("A"), label: "one".into(), confidence_hint: 0.5 },
        ];
        let errs = tree.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("minimum is")));
    }

    #[test]
    fn validate_bad_confidence() {
        let mut tree = simple_tree();
        tree.root.choices[0].confidence_hint = 1.5;
        let errs = tree.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("confidence_hint")));
    }

    #[test]
    fn validate_orphan_child_key() {
        let mut tree = simple_tree();
        let orphan = leaf_node("orphan", &["a", "b", "c"], "fix");
        tree.root.children.insert(ChoiceId::new("Z"), orphan);
        let errs = tree.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("not found in choices")));
    }

    #[test]
    fn traversal_state_defaults() {
        let state = TraversalState::new();
        assert_eq!(state.depth, 0);
        assert_eq!(state.token_cost, 0);
        assert!(!state.is_max_depth());
        assert!(!state.over_budget());
    }

    #[test]
    fn choice_id_round_trip() {
        let id = ChoiceId::new("B");
        assert_eq!(id.as_str(), "B");
    }

    #[test]
    fn tree_serde_round_trip() {
        let tree = simple_tree();
        let json = serde_json::to_string(&tree).expect("serialize");
        let back: InterrogationTree = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.tree_id, tree.tree_id);
        assert_eq!(back.language, tree.language);
        assert_eq!(back.root.choices.len(), tree.root.choices.len());
    }
}
