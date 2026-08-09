use serde_json::Value;

/// One normalized SARIF result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SarifFinding {
    /// SARIF `result.ruleId`. Optional in the spec, so optional here.
    pub rule_id: Option<String>,
    /// SARIF `result.level`. Absent in the document means absent here: the SARIF
    /// default comes from the rule's configuration, which this parser cannot see,
    /// so inventing "warning" would be a fabricated severity.
    pub level: Option<String>,
    /// SARIF `result.message.text`. Required.
    pub message: String,
    /// First location's `physicalLocation.artifactLocation.uri`, when present.
    pub file: Option<String>,
    /// First location's `physicalLocation.region.startLine`, when present.
    pub start_line: Option<u64>,
}

/// A parsed SARIF document, flattened across runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SarifReport {
    /// `runs[0].tool.driver.name`, when present.
    pub tool_name: Option<String>,
    /// Every `result` from every run, in document order.
    pub findings: Vec<SarifFinding>,
}

#[derive(Debug, thiserror::Error)]
pub enum SarifIngestError {
    #[error("invalid SARIF JSON: {0}")]
    InvalidJson(String),
    #[error("unsupported SARIF version {found:?}: only {} is supported", SUPPORTED_SARIF_VERSION)]
    UnsupportedVersion { found: Option<String> },
    #[error("SARIF document has no `runs` array")]
    MissingRuns,
    #[error("malformed SARIF result at runs[{run}].results[{result}]: {reason}")]
    MalformedResult { run: usize, result: usize, reason: String },
}

/// The only SARIF version this parser accepts.
pub const SUPPORTED_SARIF_VERSION: &str = "2.1.0";

/// Parse a SARIF v2.1.0 document into normalized findings.
pub fn parse_sarif(bytes: &[u8]) -> Result<SarifReport, SarifIngestError> {
    let document: Value = serde_json::from_slice(bytes)
        .map_err(|error| SarifIngestError::InvalidJson(error.to_string()))?;
    let object = document.as_object();
    let found = object
        .and_then(|value| value.get("version"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    if found.as_deref() != Some(SUPPORTED_SARIF_VERSION) {
        return Err(SarifIngestError::UnsupportedVersion { found });
    }

    let runs = object
        .and_then(|value| value.get("runs"))
        .and_then(Value::as_array)
        .ok_or(SarifIngestError::MissingRuns)?;
    let tool_name = runs.first().and_then(tool_name);
    let mut findings = Vec::new();

    for (run_index, run) in runs.iter().enumerate() {
        let results = match run.get("results") {
            Some(value) => value
                .as_array()
                .map(Vec::as_slice)
                .ok_or_else(|| SarifIngestError::MalformedResult {
                    run: run_index,
                    result: 0,
                    reason: "`results` is not an array".to_owned(),
                })?,
            None => &[],
        };
        for (result_index, result) in results.iter().enumerate() {
            let result_object = result.as_object();
            let message = result_object
                .and_then(|value| value.get("message"))
                .and_then(Value::as_object)
                .and_then(|value| value.get("text"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| SarifIngestError::MalformedResult {
                    run: run_index,
                    result: result_index,
                    reason: "message.text is missing or not a string".to_owned(),
                })?;
            let rule_id = result_object
                .and_then(|value| value.get("ruleId"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            let level = result_object
                .and_then(|value| value.get("level"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            let (file, start_line) = first_location(result_object);
            findings.push(SarifFinding { rule_id, level, message, file, start_line });
        }
    }

    Ok(SarifReport { tool_name, findings })
}

fn tool_name(run: &Value) -> Option<String> {
    run.get("tool")
        .and_then(Value::as_object)
        .and_then(|tool| tool.get("driver"))
        .and_then(Value::as_object)
        .and_then(|driver| driver.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn first_location(result: Option<&serde_json::Map<String, Value>>) -> (Option<String>, Option<u64>) {
    let location = result
        .and_then(|value| value.get("locations"))
        .and_then(Value::as_array)
        .and_then(|locations| locations.first());
    let physical = location
        .and_then(Value::as_object)
        .and_then(|value| value.get("physicalLocation"))
        .and_then(Value::as_object);
    let file = physical
        .and_then(|value| value.get("artifactLocation"))
        .and_then(Value::as_object)
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let start_line = physical
        .and_then(|value| value.get("region"))
        .and_then(Value::as_object)
        .and_then(|value| value.get("startLine"))
        .and_then(Value::as_u64);
    (file, start_line)
}

impl SarifReport {
    /// The finding count this gate reports. Named rather than open-coded at the call site.
    #[must_use]
    pub fn finding_count(&self) -> u64 { self.findings.len() as u64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEMGREP_FIXTURE: &str = r#"{
        "version": "2.1.0",
        "runs": [{
            "tool": {"driver": {"name": "semgrep"}},
            "results": [
                {"ruleId": "demo.rule", "level": "warning", "message": {"text": "Avoid this"},
                 "locations": [{"physicalLocation": {"artifactLocation": {"uri": "src/main.py"}, "region": {"startLine": 12}}}]},
                {"message": {"text": "Another finding"}}
            ]
        }]
    }"#;

    #[test]
    fn parse_sarif_two_results_normalizes_both() {
        let report = parse_sarif(SEMGREP_FIXTURE.as_bytes()).expect("fixture is valid");
        assert_eq!(report.finding_count(), 2);
        assert_eq!(report.tool_name.as_deref(), Some("semgrep"));
        assert_eq!(report.findings[0], SarifFinding {
            rule_id: Some("demo.rule".to_owned()), level: Some("warning".to_owned()),
            message: "Avoid this".to_owned(), file: Some("src/main.py".to_owned()), start_line: Some(12),
        });
        assert_eq!(report.findings[1].rule_id, None);
        assert_eq!(report.findings[1].level, None, "absent level must not default to a severity");
        assert_eq!(report.findings[1].file, None);
        assert_eq!(report.findings[1].start_line, None);
    }

    #[test]
    fn parse_sarif_empty_runs_is_clean_not_error() {
        let report = parse_sarif(br#"{"version":"2.1.0","runs":[]}"#).expect("empty runs is valid");
        assert_eq!(report.finding_count(), 0);
    }

    #[test]
    fn parse_sarif_rejects_other_version() {
        let error = parse_sarif(br#"{"version":"2.2.0","runs":[]}"#).unwrap_err();
        assert!(matches!(error, SarifIngestError::UnsupportedVersion { found: Some(v) } if v == "2.2.0"));
    }

    #[test]
    fn parse_sarif_rejects_absent_version() {
        let error = parse_sarif(br#"{"runs":[]}"#).unwrap_err();
        assert!(matches!(error, SarifIngestError::UnsupportedVersion { found: None }));
    }

    #[test]
    fn parse_sarif_rejects_missing_runs() {
        let error = parse_sarif(br#"{"version":"2.1.0"}"#).unwrap_err();
        assert!(matches!(error, SarifIngestError::MissingRuns));
    }

    #[test]
    fn parse_sarif_rejects_invalid_json() {
        let error = parse_sarif(b"not sarif").unwrap_err();
        assert!(matches!(error, SarifIngestError::InvalidJson(_)));
    }

    #[test]
    fn parse_sarif_rejects_result_without_message_text() {
        let error = parse_sarif(br#"{"version":"2.1.0","runs":[{"results":[{"ruleId":"x"}]}]}"#).unwrap_err();
        assert!(matches!(error, SarifIngestError::MalformedResult { run: 0, result: 0, .. }));
    }

    #[test]
    fn parse_sarif_flattens_two_runs() {
        let input = br#"{"version":"2.1.0","runs":[{"results":[{"message":{"text":"one"}}]},{"results":[{"message":{"text":"two"}}]}]}"#;
        let report = parse_sarif(input).expect("both runs are valid");
        assert_eq!(report.finding_count(), 2);
    }
}
