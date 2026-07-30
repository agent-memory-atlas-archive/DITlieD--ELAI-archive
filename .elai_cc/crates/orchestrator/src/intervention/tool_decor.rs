// J-12 H3: tool description embedding. Constraint text injected into
// TOOL DESCRIPTIONS (never into the model system block), provider-agnostic,
// before MessageRequest.tools is sent. Pure, LLM-free, byte-stable.

use elai_providers::ToolDefinition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolConstraint {
    pub id: String,
    pub tool_name: Option<String>,
    pub text: String,
}

/// Byte-stable decoration pass. Empty constraints => identity (noop).
/// Append format (stable): description becomes
///   "{original}\n\n[j12-protocol:{id}] {text}"
/// When original was None the description is just the tag block.
/// Constraints are sorted by `id` ascending before application (HD-2
/// cache prefix stability). Never touches name, input_schema, or
/// cache_control.
pub fn decorate_tool_descriptions(
    mut tools: Vec<ToolDefinition>,
    constraints: &[ToolConstraint],
) -> Vec<ToolDefinition> {
    if constraints.is_empty() {
        return tools;
    }
    let mut sorted: Vec<&ToolConstraint> = constraints.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    for tool in &mut tools {
        let mut appended = String::new();
        for c in &sorted {
            let matches = match &c.tool_name {
                None => true,
                Some(name) => name == &tool.name,
            };
            if !matches {
                continue;
            }
            if !appended.is_empty() {
                appended.push('\n');
            }
            appended.push_str("[j12-protocol:");
            appended.push_str(&c.id);
            appended.push_str("] ");
            appended.push_str(&c.text);
        }
        if appended.is_empty() {
            continue;
        }
        let new_desc = match &tool.description {
            Some(orig) if !orig.is_empty() => format!("{orig}\n\n{appended}"),
            _ => appended,
        };
        tool.description = Some(new_desc);
    }
    tools
}

pub struct ToolDecor {
    constraints: Vec<ToolConstraint>,
}

impl ToolDecor {
    pub fn empty() -> Self {
        Self { constraints: Vec::new() }
    }

    pub fn new(constraints: Vec<ToolConstraint>) -> Self {
        Self { constraints }
    }

    pub fn apply(&self, tools: Vec<ToolDefinition>) -> Vec<ToolDefinition> {
        decorate_tool_descriptions(tools, &self.constraints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tool(name: &str, desc: Option<&str>) -> ToolDefinition {
        ToolDefinition {
            name: name.to_string(),
            description: desc.map(|s| s.to_string()),
            input_schema: json!({"type": "object"}),
            cache_control: None,
        }
    }

    fn constraint(id: &str, tool_name: Option<&str>, text: &str) -> ToolConstraint {
        ToolConstraint {
            id: id.to_string(),
            tool_name: tool_name.map(|s| s.to_string()),
            text: text.to_string(),
        }
    }

    #[test]
    fn empty_constraints_identity() {
        let tools = vec![tool("a", Some("desc a")), tool("b", None)];
        let out = decorate_tool_descriptions(tools, &[]);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].description, Some("desc a".to_string()));
        assert_eq!(out[1].description, None);
    }

    #[test]
    fn all_tools_constraint_appends_to_every_description() {
        let tools = vec![tool("a", Some("desc a")), tool("b", None)];
        let cs = [constraint("c1", None, "be safe")];
        let out = decorate_tool_descriptions(tools, &cs);
        assert_eq!(out[0].description, Some("desc a\n\n[j12-protocol:c1] be safe".to_string()));
        assert_eq!(out[1].description, Some("[j12-protocol:c1] be safe".to_string()));
    }

    #[test]
    fn tool_name_filter_only_mutates_matching() {
        let tools = vec![tool("a", Some("desc a")), tool("b", Some("desc b"))];
        let cs = [constraint("c1", Some("a"), "only a")];
        let out = decorate_tool_descriptions(tools, &cs);
        assert_eq!(out[0].description, Some("desc a\n\n[j12-protocol:c1] only a".to_string()));
        assert_eq!(out[1].description, Some("desc b".to_string()));
    }

    #[test]
    fn two_constraints_sort_by_id_deterministic() {
        let tools = vec![tool("a", Some("desc a"))];
        let cs = [
            constraint("z1", None, "second"),
            constraint("a1", None, "first"),
        ];
        let out = decorate_tool_descriptions(tools, &cs);
        assert_eq!(
            out[0].description,
            Some("desc a\n\n[j12-protocol:a1] first\n[j12-protocol:z1] second".to_string())
        );
    }
}
