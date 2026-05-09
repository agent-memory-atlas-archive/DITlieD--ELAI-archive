// Interrogation tree storage in rules.db (spec: reasoning-architecture-design.txt LAYER 3).
//
// Trees stored as JSON blobs in the `interrogation_trees` table, indexed by
// (language, domain). Migration handled in schema.rs (v3 -> v4).

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use elai_core::interrogation_tree::{InterrogationTree, TREE_SCHEMA_VERSION};

use crate::error::RulesDbError;

// ---------------------------------------------------------------------------
// InterrogationStore trait — consumer-facing handle for the orchestrator
// (Session A-19.REDO)
// ---------------------------------------------------------------------------

/// Send+Sync handle used by the orchestrator to resolve interrogation trees at
/// runtime without taking a direct rusqlite `Connection` dependency.
///
/// Implementations own their own synchronization (the seed tables are read-
/// mostly at runtime, so a `Mutex<Connection>` is adequate). Methods are
/// infallible from the caller's POV: a DB error or a missing tree both
/// collapse to `None` / `0` so callers can treat the store as an optional
/// best-effort source. Implementations SHOULD log the underlying error via
/// `tracing` before returning the fallback so operators still see a signal.
pub trait InterrogationStore: Send + Sync {
    /// Return the tree whose `language` matches and whose `domain` is either
    /// exactly `domain` OR NULL (universal). The first matching row by
    /// `query_trees` ordering is returned. `None` when no match (or on error).
    fn lookup_tree(&self, language: &str, domain: Option<&str>) -> Option<InterrogationTree>;

    /// Return the total number of trees currently stored. `0` on error.
    /// Used by the task_runner to emit the "seed-tree count" metric Finding
    /// (acceptance criterion 3 of Session A-19.REDO).
    fn tree_count(&self) -> usize;
}

/// Default SQLite-backed implementation of [`InterrogationStore`] that wraps
/// a shared `Connection` under a `Mutex`. Constructed by the CLI/dashboard
/// wiring once the rules DB is open.
pub struct SqliteInterrogationStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteInterrogationStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl InterrogationStore for SqliteInterrogationStore {
    fn lookup_tree(&self, language: &str, domain: Option<&str>) -> Option<InterrogationTree> {
        let guard = match self.conn.lock() {
            Ok(g) => g,
            Err(e) => {
                tracing::warn!(error = %e, "interrogation_store: mutex poisoned");
                return None;
            }
        };
        match query_trees(&guard, language, domain) {
            Ok(mut v) => {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            }
            Err(e) => {
                tracing::warn!(
                    language,
                    domain = domain.unwrap_or(""),
                    error = %e,
                    "interrogation_store: query_trees failed",
                );
                None
            }
        }
    }

    fn tree_count(&self) -> usize {
        let guard = match self.conn.lock() {
            Ok(g) => g,
            Err(e) => {
                tracing::warn!(error = %e, "interrogation_store: mutex poisoned on count");
                return 0;
            }
        };
        let result: Result<i64, _> = guard.query_row(
            "SELECT COUNT(*) FROM interrogation_trees",
            [],
            |row| row.get(0),
        );
        match result {
            Ok(n) if n >= 0 => n as usize,
            Ok(_) => 0,
            Err(e) => {
                tracing::warn!(error = %e, "interrogation_store: count query failed");
                0
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Query: retrieve trees by language + optional domain
// ---------------------------------------------------------------------------

/// Retrieve all interrogation trees for a given language.
/// If `domain` is Some, also filters by domain (exact match or NULL domain).
/// Trees with domain=NULL are "universal" for that language and always included.
pub fn query_trees(
    conn: &Connection,
    language: &str,
    domain: Option<&str>,
) -> Result<Vec<InterrogationTree>, RulesDbError> {
    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match domain {
        Some(d) => (
            "SELECT tree_json FROM interrogation_trees \
             WHERE language = ?1 AND (domain = ?2 OR domain IS NULL) \
             ORDER BY domain IS NULL ASC, tree_id ASC",
            vec![
                Box::new(language.to_string()),
                Box::new(d.to_string()),
            ],
        ),
        None => (
            "SELECT tree_json FROM interrogation_trees \
             WHERE language = ?1 \
             ORDER BY domain IS NULL ASC, tree_id ASC",
            vec![Box::new(language.to_string())],
        ),
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        let json_str: String = row.get(0)?;
        Ok(json_str)
    })?;

    let mut trees = Vec::new();
    for row in rows {
        let json_str = row?;
        let tree: InterrogationTree = serde_json::from_str(&json_str)
            .map_err(|e| RulesDbError::TreeDeserialize(e.to_string()))?;
        trees.push(tree);
    }
    Ok(trees)
}

/// Retrieve a single interrogation tree by tree_id.
pub fn query_tree_by_id(
    conn: &Connection,
    tree_id: &str,
) -> Result<Option<InterrogationTree>, RulesDbError> {
    let mut stmt = conn.prepare(
        "SELECT tree_json FROM interrogation_trees WHERE tree_id = ?1",
    )?;
    let mut rows = stmt.query(rusqlite::params![tree_id])?;
    match rows.next()? {
        Some(row) => {
            let json_str: String = row.get(0)?;
            let tree: InterrogationTree = serde_json::from_str(&json_str)
                .map_err(|e| RulesDbError::TreeDeserialize(e.to_string()))?;
            Ok(Some(tree))
        }
        None => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Insert / upsert
// ---------------------------------------------------------------------------

/// Insert or replace an interrogation tree in the database.
/// Validates the tree before storing. Uses INSERT OR REPLACE for idempotency.
pub fn upsert_tree(
    conn: &Connection,
    tree: &InterrogationTree,
) -> Result<(), RulesDbError> {
    tree.validate().map_err(|errs| {
        RulesDbError::TreeValidation(errs.join("; "))
    })?;

    let json_str = serde_json::to_string(tree)
        .map_err(|e| RulesDbError::TreeDeserialize(e.to_string()))?;

    conn.execute(
        "INSERT OR REPLACE INTO interrogation_trees \
         (tree_id, language, domain, description, version, tree_json) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            tree.tree_id,
            tree.language,
            tree.domain,
            tree.description,
            tree.version as i64,
            json_str,
        ],
    )?;
    Ok(())
}

/// Delete an interrogation tree by tree_id. Returns true if a row was deleted.
pub fn delete_tree(conn: &Connection, tree_id: &str) -> Result<bool, RulesDbError> {
    let affected = conn.execute(
        "DELETE FROM interrogation_trees WHERE tree_id = ?1",
        rusqlite::params![tree_id],
    )?;
    Ok(affected > 0)
}

// ---------------------------------------------------------------------------
// Seed data: initial decision trees per language
// ---------------------------------------------------------------------------

/// Seed the database with initial interrogation trees for 4 languages.
/// Uses upsert_tree internally so it is idempotent.
pub fn seed_interrogation_trees(conn: &Connection) -> Result<(), RulesDbError> {
    let trees = build_seed_trees();
    for tree in &trees {
        upsert_tree(conn, tree)?;
    }
    Ok(())
}

/// Build the 4 seed trees (Rust, Python, Go, TypeScript).
pub fn build_seed_trees() -> Vec<InterrogationTree> {
    vec![
        build_rust_lifetime_tree(),
        build_python_import_tree(),
        build_go_error_tree(),
        build_typescript_type_tree(),
    ]
}

// ---------------------------------------------------------------------------
// Rust: lifetime / borrow / trait errors
// ---------------------------------------------------------------------------

fn build_rust_lifetime_tree() -> InterrogationTree {
    use elai_core::interrogation_tree::{Choice, ChoiceId, FixTemplate, QuestionNode};
    use std::collections::BTreeMap;

    // Q3 leaves for borrow path
    let q3_borrow = QuestionNode {
        question_template: "To fix the borrow issue in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "clone the value".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("B"), label: "restructure to avoid simultaneous borrows".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("C"), label: "use a reference instead of owned value".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "restructure borrows in {function} at line {line}".to_string(),
            category: "borrow_fix".to_string(),
        }),
    };

    // Q3 leaves for lifetime path
    let q3_lifetime = QuestionNode {
        question_template: "To fix the lifetime in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "add explicit lifetime annotation".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "change to owned type (String, Vec, etc.)".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("C"), label: "use 'static or leak the value".to_string(), confidence_hint: 0.1 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "add lifetime annotation or change to owned type in {function}".to_string(),
            category: "lifetime_fix".to_string(),
        }),
    };

    // Q3 leaves for trait path
    let q3_trait = QuestionNode {
        question_template: "To fix the trait issue in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "implement the missing trait".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "add a trait bound to the generic parameter".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("C"), label: "use a concrete type instead of generic".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "resolve trait bound in {function}: impl or constrain {type}".to_string(),
            category: "trait_fix".to_string(),
        }),
    };

    // Q2: root cause by category
    let mut q2_children = BTreeMap::new();
    q2_children.insert(ChoiceId::new("A"), q3_borrow);
    q2_children.insert(ChoiceId::new("B"), q3_lifetime);
    q2_children.insert(ChoiceId::new("C"), q3_trait);

    let q2 = QuestionNode {
        question_template: "The Rust error in {function} is related to:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "borrow checker (multiple/mutable borrows)".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "lifetime annotations (missing or wrong)".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("C"), label: "trait bounds (missing impl or constraint)".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: None,
    };

    // Reassemble q2 with children
    let q2_with_children = QuestionNode {
        children: q2_children,
        ..q2
    };

    // Q1: error location
    let q1e_rust = QuestionNode {
        question_template: "To fix the test expectation in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "fix the assertion value or type".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "fix the test setup / fixture".to_string(), confidence_hint: 0.35 },
            Choice { id: ChoiceId::new("C"), label: "update expected output to match new behaviour".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "fix test assertion or setup in {function} at line {line}".to_string(),
            category: "test_fix".to_string(),
        }),
    };

    let mut root_children = BTreeMap::new();
    root_children.insert(ChoiceId::new("B"), q2_with_children);
    root_children.insert(ChoiceId::new("E"), q1e_rust);

    let root = QuestionNode {
        question_template: "The error in {function} is in:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "function signature (params, return type)".to_string(), confidence_hint: 0.2 },
            Choice { id: ChoiceId::new("B"), label: "function body logic".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("C"), label: "return value or type conversion".to_string(), confidence_hint: 0.2 },
            Choice { id: ChoiceId::new("D"), label: "an imported dependency or module".to_string(), confidence_hint: 0.1 },
            Choice { id: ChoiceId::new("E"), label: "test expectations (wrong assertion or test setup)".to_string(), confidence_hint: 0.1 },
        ],
        children: root_children,
        terminal: Some(FixTemplate {
            instruction: "check {function} signature and return type at line {line}".to_string(),
            category: "general_fix".to_string(),
        }),
    };

    InterrogationTree {
        tree_id: "rust-ownership".to_string(),
        root,
        language: "rust".to_string(),
        domain: None,
        description: "Rust lifetime, borrow checker, and trait bound errors".to_string(),
        version: TREE_SCHEMA_VERSION,
    }
}

// ---------------------------------------------------------------------------
// Python: import / type errors
// ---------------------------------------------------------------------------

fn build_python_import_tree() -> InterrogationTree {
    use elai_core::interrogation_tree::{Choice, ChoiceId, FixTemplate, QuestionNode};
    use std::collections::BTreeMap;

    let q3_import = QuestionNode {
        question_template: "To fix the import in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "fix the import path".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "install the missing package".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("C"), label: "use a different module/function".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "fix import: {import_path} in {file}".to_string(),
            category: "import_fix".to_string(),
        }),
    };

    let q3_type = QuestionNode {
        question_template: "To fix the type error in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "add or fix type annotation".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("B"), label: "convert the value to expected type".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("C"), label: "change the function signature".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "fix type mismatch in {function}: convert or annotate at line {line}".to_string(),
            category: "type_fix".to_string(),
        }),
    };

    let mut q2_children = BTreeMap::new();
    q2_children.insert(ChoiceId::new("A"), q3_import);
    q2_children.insert(ChoiceId::new("B"), q3_type);

    let q2 = QuestionNode {
        question_template: "The Python error is related to:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "import / module not found".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "type mismatch / AttributeError".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("C"), label: "incorrect function arguments".to_string(), confidence_hint: 0.2 },
        ],
        children: q2_children,
        terminal: Some(FixTemplate {
            instruction: "fix argument mismatch in call to {function}".to_string(),
            category: "arg_fix".to_string(),
        }),
    };

    let q1e_python = QuestionNode {
        question_template: "To fix the test expectation in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "fix the mock return value or side effect".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "fix the assertion (assertEqual, assertTrue, etc.)".to_string(), confidence_hint: 0.35 },
            Choice { id: ChoiceId::new("C"), label: "fix the test fixture or setUp method".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "fix mock or assertion in {function} at line {line}".to_string(),
            category: "test_fix".to_string(),
        }),
    };

    let mut root_children = BTreeMap::new();
    root_children.insert(ChoiceId::new("B"), q2);
    root_children.insert(ChoiceId::new("E"), q1e_python);

    let root = QuestionNode {
        question_template: "The error in {function} is in:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "function signature / decorator".to_string(), confidence_hint: 0.2 },
            Choice { id: ChoiceId::new("B"), label: "function body logic".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("C"), label: "return value".to_string(), confidence_hint: 0.2 },
            Choice { id: ChoiceId::new("D"), label: "an imported dependency".to_string(), confidence_hint: 0.15 },
            Choice { id: ChoiceId::new("E"), label: "test expectations (mock or assertion error)".to_string(), confidence_hint: 0.05 },
        ],
        children: root_children,
        terminal: Some(FixTemplate {
            instruction: "check {function} definition and imports in {file}".to_string(),
            category: "general_fix".to_string(),
        }),
    };

    InterrogationTree {
        tree_id: "python-import-type".to_string(),
        root,
        language: "python".to_string(),
        domain: None,
        description: "Python import resolution and type errors".to_string(),
        version: TREE_SCHEMA_VERSION,
    }
}

// ---------------------------------------------------------------------------
// Go: goroutine / error wrapping
// ---------------------------------------------------------------------------

fn build_go_error_tree() -> InterrogationTree {
    use elai_core::interrogation_tree::{Choice, ChoiceId, FixTemplate, QuestionNode};
    use std::collections::BTreeMap;

    let q3_goroutine = QuestionNode {
        question_template: "The goroutine issue in {function} is:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "data race (shared state without sync)".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "deadlock (channel or mutex)".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("C"), label: "goroutine leak (no cleanup)".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "add synchronization or fix channel usage in {function}".to_string(),
            category: "concurrency_fix".to_string(),
        }),
    };

    let q3_error = QuestionNode {
        question_template: "To fix error handling in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "wrap error with fmt.Errorf(\"%w\", err)".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "add missing error check (if err != nil)".to_string(), confidence_hint: 0.35 },
            Choice { id: ChoiceId::new("C"), label: "define custom error type".to_string(), confidence_hint: 0.15 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "fix error handling in {function}: wrap or check at line {line}".to_string(),
            category: "error_handling_fix".to_string(),
        }),
    };

    let mut q2_children = BTreeMap::new();
    q2_children.insert(ChoiceId::new("A"), q3_goroutine);
    q2_children.insert(ChoiceId::new("B"), q3_error);

    let q2 = QuestionNode {
        question_template: "The Go error is related to:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "goroutine / channel / concurrency".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("B"), label: "error wrapping / handling".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("C"), label: "interface satisfaction".to_string(), confidence_hint: 0.2 },
        ],
        children: q2_children,
        terminal: Some(FixTemplate {
            instruction: "check interface implementation for {type} in {file}".to_string(),
            category: "interface_fix".to_string(),
        }),
    };

    let q1e_go = QuestionNode {
        question_template: "To fix the test expectation in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "fix the assert/require call arguments".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "fix the test setup or TestMain".to_string(), confidence_hint: 0.35 },
            Choice { id: ChoiceId::new("C"), label: "update expected value to match new behaviour".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "fix assert or test setup in {function} at line {line}".to_string(),
            category: "test_fix".to_string(),
        }),
    };

    let mut root_children = BTreeMap::new();
    root_children.insert(ChoiceId::new("B"), q2);
    root_children.insert(ChoiceId::new("E"), q1e_go);

    let root = QuestionNode {
        question_template: "The error in {function} is in:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "function signature / receiver".to_string(), confidence_hint: 0.2 },
            Choice { id: ChoiceId::new("B"), label: "function body logic".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("C"), label: "return values".to_string(), confidence_hint: 0.2 },
            Choice { id: ChoiceId::new("D"), label: "an imported package".to_string(), confidence_hint: 0.15 },
            Choice { id: ChoiceId::new("E"), label: "test expectations (wrong assert or test setup)".to_string(), confidence_hint: 0.05 },
        ],
        children: root_children,
        terminal: Some(FixTemplate {
            instruction: "check {function} signature and receiver in {file}".to_string(),
            category: "general_fix".to_string(),
        }),
    };

    InterrogationTree {
        tree_id: "go-concurrency-error".to_string(),
        root,
        language: "go".to_string(),
        domain: None,
        description: "Go goroutine/channel concurrency and error wrapping".to_string(),
        version: TREE_SCHEMA_VERSION,
    }
}

// ---------------------------------------------------------------------------
// TypeScript: type / import errors
// ---------------------------------------------------------------------------

fn build_typescript_type_tree() -> InterrogationTree {
    use elai_core::interrogation_tree::{Choice, ChoiceId, FixTemplate, QuestionNode};
    use std::collections::BTreeMap;

    let q3_type = QuestionNode {
        question_template: "To fix the TypeScript type error in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "add or fix type assertion (as T)".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("B"), label: "update the interface/type definition".to_string(), confidence_hint: 0.35 },
            Choice { id: ChoiceId::new("C"), label: "add a type guard or narrowing check".to_string(), confidence_hint: 0.25 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "fix type in {function}: update type or add guard at line {line}".to_string(),
            category: "type_fix".to_string(),
        }),
    };

    let q3_import = QuestionNode {
        question_template: "To fix the import in {file}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "fix the import path or alias".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "install the missing @types package".to_string(), confidence_hint: 0.3 },
            Choice { id: ChoiceId::new("C"), label: "switch to a different export (named vs default)".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "fix import: update path or install types in {file}".to_string(),
            category: "import_fix".to_string(),
        }),
    };

    let mut q2_children = BTreeMap::new();
    q2_children.insert(ChoiceId::new("A"), q3_type);
    q2_children.insert(ChoiceId::new("B"), q3_import);

    let q2 = QuestionNode {
        question_template: "The TypeScript error is related to:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "type mismatch / inference failure".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "import / module resolution".to_string(), confidence_hint: 0.35 },
            Choice { id: ChoiceId::new("C"), label: "null / undefined handling".to_string(), confidence_hint: 0.2 },
        ],
        children: q2_children,
        terminal: Some(FixTemplate {
            instruction: "add null check or optional chaining in {function} at line {line}".to_string(),
            category: "null_fix".to_string(),
        }),
    };

    let q1e_ts = QuestionNode {
        question_template: "To fix the test expectation in {function}, should we:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "fix the assertion value or type mismatch in test".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("B"), label: "fix the test setup or mock type".to_string(), confidence_hint: 0.35 },
            Choice { id: ChoiceId::new("C"), label: "update expected output to match new behaviour".to_string(), confidence_hint: 0.2 },
        ],
        children: BTreeMap::new(),
        terminal: Some(FixTemplate {
            instruction: "fix test assertion or type in {function} at line {line}".to_string(),
            category: "test_fix".to_string(),
        }),
    };

    let mut root_children = BTreeMap::new();
    root_children.insert(ChoiceId::new("B"), q2);
    root_children.insert(ChoiceId::new("E"), q1e_ts);

    let root = QuestionNode {
        question_template: "The error in {function} is in:".to_string(),
        choices: vec![
            Choice { id: ChoiceId::new("A"), label: "function signature / generics".to_string(), confidence_hint: 0.2 },
            Choice { id: ChoiceId::new("B"), label: "function body logic".to_string(), confidence_hint: 0.4 },
            Choice { id: ChoiceId::new("C"), label: "return type / value".to_string(), confidence_hint: 0.2 },
            Choice { id: ChoiceId::new("D"), label: "an imported module / type".to_string(), confidence_hint: 0.15 },
            Choice { id: ChoiceId::new("E"), label: "test expectations (wrong assertion or type in test)".to_string(), confidence_hint: 0.05 },
        ],
        children: root_children,
        terminal: Some(FixTemplate {
            instruction: "check {function} generics and return type in {file}".to_string(),
            category: "general_fix".to_string(),
        }),
    };

    InterrogationTree {
        tree_id: "typescript-type-import".to_string(),
        root,
        language: "typescript".to_string(),
        domain: None,
        description: "TypeScript type errors and import resolution".to_string(),
        version: TREE_SCHEMA_VERSION,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;
    use crate::schema::open_rules_db;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    fn mem_db() -> Connection {
        open_rules_db(Path::new(":memory:"), &SeededClock::new()).expect("in-memory DB")
    }

    #[test]
    fn sqlite_store_lookup_and_count() {
        let conn = mem_db();
        let store = SqliteInterrogationStore::new(Arc::new(Mutex::new(conn)));
        assert_eq!(store.tree_count(), 4, "seed trees should be present");

        let tree = store
            .lookup_tree("rust", None)
            .expect("rust tree must resolve");
        assert_eq!(tree.language, "rust");

        let none = store.lookup_tree("nonexistent-lang", None);
        assert!(none.is_none(), "unknown language yields None");
    }

    #[test]
    fn seed_trees_all_valid() {
        let trees = build_seed_trees();
        assert_eq!(trees.len(), 4);
        for tree in &trees {
            tree.validate().unwrap_or_else(|errs| {
                panic!("tree '{}' invalid: {:?}", tree.tree_id, errs);
            });
        }
    }

    #[test]
    fn seed_trees_have_correct_languages() {
        let trees = build_seed_trees();
        let langs: Vec<&str> = trees.iter().map(|t| t.language.as_str()).collect();
        assert!(langs.contains(&"rust"));
        assert!(langs.contains(&"python"));
        assert!(langs.contains(&"go"));
        assert!(langs.contains(&"typescript"));
    }

    #[test]
    fn upsert_and_query_round_trip() {
        let conn = mem_db();
        let tree = build_rust_lifetime_tree();
        upsert_tree(&conn, &tree).unwrap();

        let results = query_trees(&conn, "rust", None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tree_id, "rust-ownership");
        assert_eq!(results[0].language, "rust");
    }

    #[test]
    fn query_by_id() {
        let conn = mem_db();
        seed_interrogation_trees(&conn).unwrap();

        let tree = query_tree_by_id(&conn, "go-concurrency-error").unwrap();
        assert!(tree.is_some());
        assert_eq!(tree.unwrap().language, "go");

        let missing = query_tree_by_id(&conn, "nonexistent").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn query_filters_by_language() {
        let conn = mem_db();
        seed_interrogation_trees(&conn).unwrap();

        let rust_trees = query_trees(&conn, "rust", None).unwrap();
        assert_eq!(rust_trees.len(), 1);

        let python_trees = query_trees(&conn, "python", None).unwrap();
        assert_eq!(python_trees.len(), 1);

        let empty = query_trees(&conn, "haskell", None).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn query_with_domain_includes_universal() {
        let conn = mem_db();
        seed_interrogation_trees(&conn).unwrap();

        // All seed trees have domain=None (universal), so they should appear
        // when querying with any domain.
        let results = query_trees(&conn, "rust", Some("web_backend")).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn seed_is_idempotent() {
        let conn = mem_db();
        seed_interrogation_trees(&conn).unwrap();
        seed_interrogation_trees(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM interrogation_trees", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn delete_tree_works() {
        let conn = mem_db();
        seed_interrogation_trees(&conn).unwrap();

        assert!(delete_tree(&conn, "rust-ownership").unwrap());
        assert!(!delete_tree(&conn, "rust-ownership").unwrap());

        let results = query_trees(&conn, "rust", None).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn upsert_rejects_invalid_tree() {
        let conn = mem_db();
        let mut tree = build_rust_lifetime_tree();
        tree.tree_id = String::new(); // invalid
        let result = upsert_tree(&conn, &tree);
        assert!(result.is_err());
    }
}
