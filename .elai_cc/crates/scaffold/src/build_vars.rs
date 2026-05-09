//! Default substitution variables shared by every consumer of
//! `scaffold::apply_template`.
//!
//! Lives in the `scaffold` crate (not in any dashboard-side module) so
//! contract tests can verify the shape without linking against the
//! dashboard. Templates expect exactly seven keys:
//!     project_name, description, author, primary_hsl, bg_hsl, fg_hsl, accent_hsl
//! Missing any of them = `ScaffoldError::VariableMissing` at apply-time.

use std::collections::BTreeMap;

use crate::{DEFAULT_ACCENT_HSL, DEFAULT_BG_HSL, DEFAULT_FG_HSL, DEFAULT_PRIMARY_HSL};

/// Fallback `project_name` injected when the caller does not supply one.
/// Same value the .bkp_old DashboardScaffoldApplier shipped.
pub const SCAFFOLD_DEFAULT_PROJECT_NAME: &str = "New Project";

/// Fallback `author` injected when the caller does not supply one.
pub const SCAFFOLD_DEFAULT_AUTHOR: &str = "elai";

/// Cap on auto-derived description length (taken from the trimmed task
/// text). 200 chars covers a reasonable one-sentence summary without
/// risking a description that breaks `<title>` rendering or README
/// frontmatter.
pub const SCAFFOLD_DESCRIPTION_MAX_CHARS: usize = 200;

/// Build the default substitution variable map.
///
/// Resolution rules per field:
/// - `project_name`: caller override → `SCAFFOLD_DEFAULT_PROJECT_NAME`.
/// - `description`: caller override → first `SCAFFOLD_DESCRIPTION_MAX_CHARS`
///   chars of trimmed `task_text` → `SCAFFOLD_DEFAULT_PROJECT_NAME` when both
///   are blank.
/// - `author`: caller override → `SCAFFOLD_DEFAULT_AUTHOR`.
/// - HSL palette: always the four `DEFAULT_*_HSL` constants (override is
///   not yet exposed; templates will pick these up via `{{primary_hsl}}` etc.).
pub fn default_build_vars(
    task_text: &str,
    project_name: Option<&str>,
    description: Option<&str>,
    author: Option<&str>,
) -> BTreeMap<String, String> {
    let mut vars: BTreeMap<String, String> = BTreeMap::new();

    let resolved_name = project_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(SCAFFOLD_DEFAULT_PROJECT_NAME)
        .to_owned();

    let resolved_description = description
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            let truncated: String = task_text
                .trim()
                .chars()
                .take(SCAFFOLD_DESCRIPTION_MAX_CHARS)
                .collect();
            if truncated.is_empty() {
                SCAFFOLD_DEFAULT_PROJECT_NAME.to_owned()
            } else {
                truncated
            }
        });

    let resolved_author = author
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(SCAFFOLD_DEFAULT_AUTHOR)
        .to_owned();

    vars.insert("project_name".to_owned(), resolved_name);
    vars.insert("description".to_owned(), resolved_description);
    vars.insert("author".to_owned(), resolved_author);
    vars.insert("primary_hsl".to_owned(), DEFAULT_PRIMARY_HSL.to_owned());
    vars.insert("bg_hsl".to_owned(), DEFAULT_BG_HSL.to_owned());
    vars.insert("fg_hsl".to_owned(), DEFAULT_FG_HSL.to_owned());
    vars.insert("accent_hsl".to_owned(), DEFAULT_ACCENT_HSL.to_owned());

    vars
}
