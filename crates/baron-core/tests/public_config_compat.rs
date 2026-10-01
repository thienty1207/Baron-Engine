use std::path::PathBuf;

use baron_core::config::{AutomationConfig, LocalConfig, ProjectConfig};

#[test]
fn public_config_struct_literals_remain_source_compatible() {
    let automation = AutomationConfig {
        context: true,
        plan: true,
        harness: true,
        proof: true,
        trace: true,
    };
    let project = ProjectConfig {
        schema_version: 4,
        project_id: "project".into(),
        identity_binding: "binding".into(),
        project_slug: "project".into(),
        platform: None,
        platform_extensions: Vec::new(),
        adapters: Vec::new(),
        active_adapter: None,
        automation,
        legacy_adapters: Vec::new(),
        legacy_active_adapter: None,
    };
    let local = LocalConfig {
        vault_path: PathBuf::from("vault"),
    };

    assert_eq!(project.project_id, "project");
    assert_eq!(local.vault_path, PathBuf::from("vault"));
}
