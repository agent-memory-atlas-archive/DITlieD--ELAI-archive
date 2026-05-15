// B-30 Pass C: multi-language import resolution submodule.
//
// Each resolver implements ImportResolver and is constructed lazily inside
// DkgEngine::resolve_multi_module_imports. All internal maps use BTreeMap
// for deterministic iteration order (CLAUDE.md anti-stale + DECISION_4).

pub mod jvm;
pub mod csharp;
pub mod go_module;
pub mod tsconfig;

pub use jvm::JvmResolver;
pub use csharp::CsNamespaceResolver;
pub use go_module::GoModuleResolver;
pub use tsconfig::TsPathAliasResolver;

use std::path::PathBuf;

/// Resolve a user-visible import string to a candidate file path.
///
/// The `import_str` passed in is already cleaned by the caller — language-
/// specific prefixes ("import ", "using ", etc.) and suffixes (";", wildcard
/// ".*") have been stripped. Resolvers do only structural mapping.
pub trait ImportResolver {
    /// Return the best candidate file path for `import_str`, or `None` if
    /// this resolver cannot map the import.
    fn resolve(&self, import_str: &str) -> Option<PathBuf>;
}
