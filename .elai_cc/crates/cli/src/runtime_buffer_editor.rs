use std::sync::Arc;

use crate::runtime_builder::RuntimeWiring;

pub(crate) fn install_buffer_editor(wiring: &mut RuntimeWiring) {
    let editor: Arc<dyn elai_orchestrator::focus_tunnel::BufferEditor> = Arc::new(
        elai_tools::dispatcher::InProcessToolDispatcher::new(Arc::new(
            elai_tools::dispatcher::default_validator(
                Arc::new(crate::runtime_builder::runtime_dispatcher::NoOpIntegrationRunner),
                "rust",
            ),
        )),
    );
    wiring.buffer_editor = Some(editor);
    tracing::info!(
        "B-28: production InProcessToolDispatcher buffer_editor installed; \
         focus-tunnel DIFF branch will apply hunks through the 3-tier matcher"
    );
}
