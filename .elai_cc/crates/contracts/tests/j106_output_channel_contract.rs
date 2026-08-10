//! J-106 / HD-3 output-channel classification and guarded postprocessor seam.

use elai_orchestrator::{
    output_channel_class, postprocess_output, AgentRole, OutputChannelClass, OutputConfig,
};

fn config_for(role: AgentRole) -> OutputConfig {
    OutputConfig {
        strip_emoji: false,
        strip_sycophancy: false,
        max_response_tokens: 0,
        restore_formatting: false,
        restore_formatting_language: String::new(),
        channel_class: output_channel_class(role),
    }
}

#[test]
fn reasoning_roles_passthrough_and_classifier_is_exhaustive() {
    std::env::set_var("ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE", "1");
    let sample = "STATUS: ceremony acknowledged.
FOUND: work-log prose remains unchanged.";
    let reasoning_roles = [
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
        AgentRole::IdeaFramer,
        AgentRole::IdeaCritic,
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
        AgentRole::AuditorProductionGrade,
        AgentRole::InjectionClassifier,
    ];
    for role in reasoning_roles {
        assert_eq!(output_channel_class(role), OutputChannelClass::Reasoning);
        assert_eq!(postprocess_output(sample, &config_for(role)), sample);
    }

    for role in [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
        AgentRole::IdeaFramer,
        AgentRole::IdeaCritic,
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
        AgentRole::AuditorProductionGrade,
        AgentRole::InjectionClassifier,
    ] {
        let mirrored = match role {
            AgentRole::Executor => OutputChannelClass::OpEmission,
            AgentRole::Auditor
            | AgentRole::Researcher
            | AgentRole::Designer
            | AgentRole::PromptEngineer
            | AgentRole::ContextCompiler
            | AgentRole::MemoryWriter
            | AgentRole::MediaValidator
            | AgentRole::IdeaFramer
            | AgentRole::IdeaCritic
            | AgentRole::AuditorConcepts
            | AgentRole::AuditorPhases
            | AgentRole::AuditorSteps
            | AgentRole::AuditorBabySteps
            | AgentRole::AuditorBabyStepDetail
            | AgentRole::AuditorProductionGrade
            | AgentRole::InjectionClassifier => OutputChannelClass::Reasoning,
        };
        assert_eq!(output_channel_class(role), mirrored);
    }

    let executor_sample = "STATUS: ceremony acknowledged. FOUND: unchanged text";
    std::env::set_var("ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE", "1");
    assert_eq!(
        postprocess_output(executor_sample, &config_for(AgentRole::Executor)),
        executor_sample
    );
    std::env::remove_var("ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE");
    assert_eq!(postprocess_output(executor_sample, &config_for(AgentRole::Executor)), executor_sample);
}

#[test]
fn output_channel_defaults_fail_closed() {
    assert_eq!(OutputConfig::default().channel_class, OutputChannelClass::Reasoning);
    let parsed: OutputConfig = serde_json::from_str(
        r#"{"strip_emoji":false,"strip_sycophancy":false,"max_response_tokens":0,"restore_formatting":false,"restore_formatting_language":""}"#,
    )
    .expect("deserialize config without channel_class");
    assert_eq!(parsed.channel_class, OutputChannelClass::Reasoning);
}
