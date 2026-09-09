use serde::{Deserialize, Serialize};

mod install;
mod managed;
mod update;

pub use install::{
    core_managed_payloads, ensure_core_runtime, install_adapter, managed_payloads_for_adapter,
    CoreInstallReport, InstallReport,
};
pub use update::{
    core_reconcile_managed_assets, ensure_managed_baseline, load_managed_baseline,
    managed_baseline_content, managed_content_for_kind, managed_state_dir, managed_target_path,
    migrate_managed_ownership, plan_managed_update, reconcile_installed_managed_assets,
    record_managed_baseline, replace_managed_baseline, LocalReconcileReport, ManagedAssetPayload,
    ManagedAssetRecord, ManagedBaseline, ManagedMergeKind, ManagedOwner, ManagedProvenance,
    ManagedUpdateAction, ManagedUpdatePlan, OwnershipMigrationReport, SupportedManagedAdapter,
    UpdateDisposition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentAdapter {
    Codex,
    Claude,
}

impl AgentAdapter {
    pub fn flag(self) -> &'static str {
        match self {
            AgentAdapter::Codex => "--codex",
            AgentAdapter::Claude => "--claude",
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
            files: vec![
                "AGENTS.md".to_string(),
                ".agents/skills/baron-engine/SKILL.md".to_string(),
                ".agents/skills/baron-engine/agents/openai.yaml".to_string(),
                ".codex/INDEX.md".to_string(),
                ".codex/hooks.json".to_string(),
            ],
            directories: vec![
                ".baron/core".to_string(),
                ".agents/skills/baron-engine".to_string(),
                ".codex/agents".to_string(),
            ],
            message: "Codex adapter would install a thin native bridge over Baron Core, three quality-agent wrappers, and merged hooks.".to_string(),
        },
        AgentAdapter::Claude => ShadowPreview {
            adapter: "claude".to_string(),
            files: vec![
                "CLAUDE.md".to_string(),
                ".claude/skills/baron-engine/SKILL.md".to_string(),
                ".claude/agents/code-reviewer.md".to_string(),
                ".claude/agents/security-auditor.md".to_string(),
                ".claude/agents/test-engineer.md".to_string(),
                ".claude/settings.json".to_string(),
            ],
            directories: vec![
                ".baron/core".to_string(),
                ".claude/skills/baron-engine".to_string(),
                ".claude/agents".to_string(),
                ".claude/commands".to_string(),
            ],
            message: "Claude adapter would install a thin native bridge, three Core-provenance subagent wrappers, diagnostic commands, and merged settings hooks over Baron Core.".to_string(),
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
    }

    #[test]
    fn shadow_preview_is_explicitly_read_only() {
        let preview = shadow_preview(AgentAdapter::Codex).to_markdown();
        assert!(preview.contains("No files were written"));
        assert!(preview.contains("AGENTS.md"));
    }
}
