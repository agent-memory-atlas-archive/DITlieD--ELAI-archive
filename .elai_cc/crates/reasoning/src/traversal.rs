// InterrogationTree traversal logic (spec: reasoning-architecture-design.txt LAYER 3).
//
// Given an InterrogationTree and a sequence of answers (ChoiceIds), traverse
// the tree to produce a FixTemplate or report exhaustion/mismatch.

use elai_core::interrogation_tree::{
    ChoiceId, InterrogationTree, QuestionNode, TraversalOutcome, TraversalState,
    TOKENS_PER_QUESTION,
};

// ---------------------------------------------------------------------------
// TreeTraverser: stateful traversal of an interrogation tree
// ---------------------------------------------------------------------------

/// Traverses an InterrogationTree step by step.
///
/// Usage:
///   1. Create with `TreeTraverser::new(&tree)`.
///   2. Call `current_question()` to get the current QuestionNode.
///   3. Call `answer(choice_id)` to advance.
///   4. Repeat until `answer()` returns a TraversalOutcome.
pub struct TreeTraverser<'a> {
    tree: &'a InterrogationTree,
    state: TraversalState,
    /// Pointer to the current node in the tree. We store the path of nodes
    /// rather than a reference to avoid lifetime complexity with mutable state.
    current_path: Vec<ChoiceId>,
}

impl<'a> TreeTraverser<'a> {
    pub fn new(tree: &'a InterrogationTree) -> Self {
        Self {
            tree,
            state: TraversalState::new(),
            current_path: Vec::new(),
        }
    }

    /// Get the current question node (following the path from root).
    pub fn current_question(&self) -> Option<&'a QuestionNode> {
        let mut node = &self.tree.root;
        for choice_id in &self.current_path {
            match node.children.get(choice_id) {
                Some(child) => node = child,
                None => return None,
            }
        }
        Some(node)
    }

    /// Get the current traversal state.
    pub fn state(&self) -> &TraversalState {
        &self.state
    }

    /// Answer the current question with a choice, advancing the traversal.
    ///
    /// Returns `None` if the traversal can continue (call `current_question()` next).
    /// Returns `Some(outcome)` if the traversal has terminated.
    pub fn answer(&mut self, choice_id: ChoiceId) -> Option<TraversalOutcome> {
        let node = match self.current_question() {
            Some(n) => n,
            None => {
                return Some(TraversalOutcome::Exhausted {
                    partial_path: self.state.answers.clone(),
                    reason: "no current node (tree may be malformed)".to_string(),
                });
            }
        };

        // Validate that the choice exists in the current node.
        if !node.choices.iter().any(|c| c.id == choice_id) {
            return Some(TraversalOutcome::NoMatch {
                question: node.question_template.clone(),
                attempted_choice: choice_id,
            });
        }

        // Record the answer.
        self.state
            .answers
            .push((node.question_template.clone(), choice_id.clone()));
        self.state.depth += 1;
        self.state.token_cost += TOKENS_PER_QUESTION;

        // Check if there's a child node for this choice.
        if node.children.contains_key(&choice_id) {
            // Check budget/depth before descending.
            if self.state.is_max_depth() {
                return Some(TraversalOutcome::Exhausted {
                    partial_path: self.state.answers.clone(),
                    reason: "max depth reached".to_string(),
                });
            }
            if self.state.over_budget() {
                return Some(TraversalOutcome::Exhausted {
                    partial_path: self.state.answers.clone(),
                    reason: "token budget exceeded".to_string(),
                });
            }

            self.current_path.push(choice_id);
            None // Continue traversal
        } else if let Some(fix) = &node.terminal {
            // No child for this choice, but node has a terminal — use it.
            Some(TraversalOutcome::Fix(fix.clone()))
        } else {
            // Dead end: no child and no terminal.
            Some(TraversalOutcome::Exhausted {
                partial_path: self.state.answers.clone(),
                reason: "no child node and no terminal for this choice".to_string(),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Batch traversal: given a full answer sequence, traverse to completion
// ---------------------------------------------------------------------------

/// Traverse a tree with a pre-determined sequence of answers.
/// Returns the outcome after processing all answers or hitting a terminal/error.
pub fn traverse_with_answers(
    tree: &InterrogationTree,
    answers: &[ChoiceId],
) -> TraversalOutcome {
    let mut traverser = TreeTraverser::new(tree);

    for choice in answers {
        match traverser.answer(choice.clone()) {
            Some(outcome) => return outcome,
            None => continue,
        }
    }

    // All answers consumed but no terminal reached — check current node.
    match traverser.current_question() {
        Some(node) => {
            if let Some(fix) = &node.terminal {
                TraversalOutcome::Fix(fix.clone())
            } else {
                TraversalOutcome::Exhausted {
                    partial_path: traverser.state.answers.clone(),
                    reason: "answers exhausted without reaching terminal".to_string(),
                }
            }
        }
        None => TraversalOutcome::Exhausted {
            partial_path: traverser.state.answers.clone(),
            reason: "lost position in tree".to_string(),
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::interrogation_tree::{Choice, FixTemplate, QuestionNode};
    use std::collections::HashMap;

    fn make_choices(labels: &[&str]) -> Vec<Choice> {
        labels
            .iter()
            .enumerate()
            .map(|(i, label)| Choice {
                id: ChoiceId::new(format!("{}", (b'A' + i as u8) as char)),
                label: label.to_string(),
                confidence_hint: 0.25,
            })
            .collect()
    }

    fn test_tree() -> InterrogationTree {
        let leaf = QuestionNode {
            question_template: "Fix approach:".to_string(),
            choices: make_choices(&["fix A", "fix B", "fix C"]),
            children: HashMap::new(),
            terminal: Some(FixTemplate {
                instruction: "apply fix at {line}".to_string(),
                category: "test_fix".to_string(),
            }),
        };

        let mut mid_children = HashMap::new();
        mid_children.insert(ChoiceId::new("A"), leaf);

        let mid = QuestionNode {
            question_template: "Root cause:".to_string(),
            choices: make_choices(&["cause A", "cause B", "cause C"]),
            children: mid_children,
            terminal: Some(FixTemplate {
                instruction: "generic mid fix".to_string(),
                category: "generic".to_string(),
            }),
        };

        let mut root_children = HashMap::new();
        root_children.insert(ChoiceId::new("B"), mid);

        let root = QuestionNode {
            question_template: "Error location:".to_string(),
            choices: make_choices(&["signature", "body", "return"]),
            children: root_children,
            terminal: Some(FixTemplate {
                instruction: "generic root fix".to_string(),
                category: "generic".to_string(),
            }),
        };

        InterrogationTree {
            tree_id: "test".to_string(),
            root,
            language: "rust".to_string(),
            domain: None,
            description: "test tree".to_string(),
            version: elai_core::interrogation_tree::TREE_SCHEMA_VERSION,
        }
    }

    #[test]
    fn traverse_to_leaf() {
        let tree = test_tree();
        let outcome = traverse_with_answers(
            &tree,
            &[ChoiceId::new("B"), ChoiceId::new("A"), ChoiceId::new("A")],
        );
        match outcome {
            TraversalOutcome::Fix(fix) => {
                assert_eq!(fix.category, "test_fix");
                assert!(fix.instruction.contains("apply fix"));
            }
            other => panic!("expected Fix, got {:?}", other),
        }
    }

    #[test]
    fn traverse_terminal_at_non_child_choice() {
        let tree = test_tree();
        // Choose A at root — no child for A, but root has terminal.
        let outcome = traverse_with_answers(&tree, &[ChoiceId::new("A")]);
        match outcome {
            TraversalOutcome::Fix(fix) => {
                assert_eq!(fix.category, "generic");
            }
            other => panic!("expected Fix from terminal, got {:?}", other),
        }
    }

    #[test]
    fn traverse_invalid_choice() {
        let tree = test_tree();
        let outcome = traverse_with_answers(&tree, &[ChoiceId::new("Z")]);
        match outcome {
            TraversalOutcome::NoMatch { attempted_choice, .. } => {
                assert_eq!(attempted_choice.as_str(), "Z");
            }
            other => panic!("expected NoMatch, got {:?}", other),
        }
    }

    #[test]
    fn traverse_answers_exhausted_at_mid_node() {
        let tree = test_tree();
        // B -> reaches mid node, but no more answers.
        // Mid node has a terminal, so should produce Fix.
        let outcome = traverse_with_answers(&tree, &[ChoiceId::new("B")]);
        match outcome {
            TraversalOutcome::Fix(fix) => {
                assert_eq!(fix.category, "generic");
            }
            other => panic!("expected Fix from mid terminal, got {:?}", other),
        }
    }

    #[test]
    fn step_by_step_traversal() {
        let tree = test_tree();
        let mut traverser = TreeTraverser::new(&tree);

        // First question
        let q1 = traverser.current_question().unwrap();
        assert!(q1.question_template.contains("Error location"));

        // Answer B -> advance to mid
        assert!(traverser.answer(ChoiceId::new("B")).is_none());

        // Second question
        let q2 = traverser.current_question().unwrap();
        assert!(q2.question_template.contains("Root cause"));

        // Answer A -> advance to leaf
        assert!(traverser.answer(ChoiceId::new("A")).is_none());

        // Third question (leaf)
        let q3 = traverser.current_question().unwrap();
        assert!(q3.question_template.contains("Fix approach"));

        // Answer A -> terminal
        let outcome = traverser.answer(ChoiceId::new("A"));
        assert!(matches!(outcome, Some(TraversalOutcome::Fix(_))));

        // State should reflect 3 answers
        assert_eq!(traverser.state().depth, 3);
        assert_eq!(traverser.state().answers.len(), 3);
    }

    #[test]
    fn token_cost_tracking() {
        let tree = test_tree();
        let mut traverser = TreeTraverser::new(&tree);
        traverser.answer(ChoiceId::new("B"));
        assert_eq!(traverser.state().token_cost, TOKENS_PER_QUESTION);
        traverser.answer(ChoiceId::new("A"));
        assert_eq!(traverser.state().token_cost, TOKENS_PER_QUESTION * 2);
    }
}
