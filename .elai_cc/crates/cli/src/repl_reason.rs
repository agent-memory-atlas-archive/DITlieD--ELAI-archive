//! L-9: the `/reason` slash-command handler, split out of `repl_command.rs`
//! to keep that file under the maintainability line budget.

use crate::live_cli::LiveCli;

pub(crate) fn handle_reason(cli: &mut LiveCli, on_off: &str) -> bool {
    // B-39 / L-9: toggle the Chat REFLECT pipeline. Mirrors /mode: string-typed,
    // validation in handler, zero orchestrator dep.
    let trimmed = on_off.trim();
    if trimmed.eq_ignore_ascii_case("on") {
        cli.reflect_enabled = true;
        cli.reflect_profile = elai_reasoning::ReflectProfile::Default;
        tracing::info!("REFLECT enabled");
        eprintln!("[reason] Chat REFLECT pipeline enabled");
    } else if trimmed.eq_ignore_ascii_case("research") {
        cli.reflect_enabled = true;
        cli.reflect_profile = elai_reasoning::ReflectProfile::Research;
        tracing::info!("REFLECT enabled with research grounding profile");
        eprintln!("[reason] Chat REFLECT pipeline enabled with the research grounding profile");
    } else if trimmed.eq_ignore_ascii_case("off") {
        cli.reflect_enabled = false;
        cli.reflect_pipeline = None;
        cli.reflect_profile = elai_reasoning::ReflectProfile::Default;
        tracing::info!("REFLECT disabled");
        eprintln!("[reason] Chat REFLECT pipeline disabled");
    } else {
        tracing::warn!(
            on_off = %trimmed,
            "/reason: unrecognised value; expected `on`, `research`, or `off`"
        );
        eprintln!("/reason: unrecognised value `{trimmed}`; expected `on`, `research`, or `off`");
    }
    false
}
