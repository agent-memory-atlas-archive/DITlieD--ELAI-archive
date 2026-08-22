use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

pub(crate) fn build_buffer_tool_dispatcher(
    ssb: Arc<elai_orchestrator::Ssb>,
) -> Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> {
    let validator = elai_tools::dispatcher::default_validator(
        Arc::new(NoOpIntegrationRunner),
        "rust",
    );
    let inner: Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> = Arc::new(
        elai_tools::dispatcher::InProcessToolDispatcher::new(Arc::new(validator)),
    );
    Arc::new(elai_tools::adaptive_tool::IpcToolDispatcherAdapter::new(inner, ssb))
}

pub(crate) struct NoOpIntegrationRunner;

impl elai_tools::tiered_validation::IntegrationCommandRunner for NoOpIntegrationRunner {
    fn run_integration<'a>(
        &'a self,
        _level: &'a str,
        _dir: &'a Path,
    ) -> Pin<
        Box<
            dyn Future<Output = elai_tools::tiered_validation::IntegrationRunOutcome>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async {
            elai_tools::tiered_validation::IntegrationRunOutcome {
                passed: false,
                rules_fired: 0,
                stderr_snippet: Some(
                    "integration tier not wired (NoOpIntegrationRunner)".to_string(),
                ),
            }
        })
    }
}
