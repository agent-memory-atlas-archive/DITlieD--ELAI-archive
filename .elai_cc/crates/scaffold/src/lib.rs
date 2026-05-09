pub mod apply;
pub mod build_vars;
pub mod error;
pub mod fim;
pub mod manifest;
pub mod substitute;

pub use apply::{apply_template, apply_template_from_root, list_templates, MANIFEST_FILENAME, TEMPLATES_WEB_SUBDIR};
pub use build_vars::{
    default_build_vars, SCAFFOLD_DEFAULT_AUTHOR, SCAFFOLD_DEFAULT_PROJECT_NAME,
    SCAFFOLD_DESCRIPTION_MAX_CHARS,
};
pub use error::ScaffoldError;
pub use fim::{
    build_instruction_fim_prompt, build_qwen_fim_prompt, find_fim_section, splice_fim_section,
    strip_fim_artifacts, FimError, FimSplice, FIM_TOKEN_FILL_END, FIM_TOKEN_FILL_START,
    QWEN_END_OF_TEXT, QWEN_FIM_MIDDLE, QWEN_FIM_PAD, QWEN_FIM_PREFIX, QWEN_FIM_SUFFIX,
};
pub use manifest::{load_manifest, TemplateEntry, TemplateManifest};
pub use substitute::{substitute_variables, VAR_CLOSE, VAR_OPEN};

pub const DEFAULT_PRIMARY_HSL: &str = "162 68% 51%";
pub const DEFAULT_BG_HSL: &str = "220 13% 7%";
pub const DEFAULT_FG_HSL: &str = "0 0% 95%";
pub const DEFAULT_ACCENT_HSL: &str = "265 89% 78%";
