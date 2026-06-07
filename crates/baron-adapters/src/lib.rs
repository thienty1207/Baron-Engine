use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentAdapter {
    Codex,
    Claude,
    Generic,
}

impl AgentAdapter {
    pub fn flag(self) -> &'static str {
        match self {
            AgentAdapter::Codex => "--codex",
            AgentAdapter::Claude => "--claude",
            AgentAdapter::Generic => "--agent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowPreview {
    pub adapter: String,
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub message: String,
}

impl ShadowPreview {
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        output.push_str("# Shadow Init Preview\n\n");
        output.push_str(&format!("- Adapter: `{}`\n", self.adapter));
        output.push_str("- Mode: read-only shadow preview\n");
        output.push_str("- No files were written.\n\n");
        output.push_str("## Files\n\n");
        for file in &self.files {
            output.push_str(&format!("- `{}`\n", file));
        }
        output.push_str("\n## Directories\n\n");
        for directory in &self.directories {
            output.push_str(&format!("- `{}`\n", directory));
        }
        output.push_str("\n## Message\n\n");
        output.push_str(&format!("- {}\n", self.message));
        output
    }
}

pub fn shadow_preview(adapter: AgentAdapter) -> ShadowPreview {
    match adapter {
        AgentAdapter::Codex => ShadowPreview {
            adapter: "codex".to_string(),
            files: vec!["AGENTS.md".to_string()],
            directories: vec![
                ".codex/skills".to_string(),
                ".codex/agents".to_string(),
                ".codex/commands".to_string(),
            ],
            message: "Codex adapter would install Baron startup guidance, skill routing, and core quality agents.".to_string(),
        },
        AgentAdapter::Claude => ShadowPreview {
            adapter: "claude".to_string(),
            files: vec!["CLAUDE.md".to_string()],
            directories: vec![".claude/commands".to_string(), ".claude/hooks".to_string()],
            message: "Claude adapter would install Baron startup guidance and Claude command/hook surfaces.".to_string(),
        },
        AgentAdapter::Generic => ShadowPreview {
            adapter: "agent".to_string(),
            files: vec![
                "AGENT.md".to_string(),
                "baron-context.md".to_string(),
                "baron-context.json".to_string(),
            ],
            directories: vec![".baron".to_string()],
            message: "Generic adapter would install portable Markdown and JSON context contracts.".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_initial_adapter_flags() {
        assert_eq!(AgentAdapter::Codex.flag(), "--codex");
        assert_eq!(AgentAdapter::Claude.flag(), "--claude");
        assert_eq!(AgentAdapter::Generic.flag(), "--agent");
    }

    #[test]
    fn shadow_preview_is_explicitly_read_only() {
        let preview = shadow_preview(AgentAdapter::Codex).to_markdown();
        assert!(preview.contains("No files were written"));
        assert!(preview.contains("AGENTS.md"));
    }
}
