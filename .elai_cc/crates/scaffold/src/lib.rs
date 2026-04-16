pub mod apply;
pub mod error;
pub mod manifest;
pub mod substitute;

pub use apply::{apply_template, apply_template_from_root, list_templates, MANIFEST_FILENAME, TEMPLATES_WEB_SUBDIR};
pub use error::ScaffoldError;
pub use manifest::{load_manifest, TemplateEntry, TemplateManifest};
pub use substitute::{substitute_variables, VAR_CLOSE, VAR_OPEN};

pub const DEFAULT_PRIMARY_HSL: &str = "162 68% 51%";
pub const DEFAULT_BG_HSL: &str = "220 13% 7%";
pub const DEFAULT_FG_HSL: &str = "0 0% 95%";
pub const DEFAULT_ACCENT_HSL: &str = "265 89% 78%";
