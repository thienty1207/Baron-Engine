use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIdentity {
    pub project_id: String,
    pub project_slug: String,
    pub capsule_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapsuleMetadata {
    pub schema_version: u32,
    pub project_id: String,
    pub project_slug: String,
}

pub fn project_id_for_path(repo_root: &Path) -> Result<String> {
    let canonical = repo_root
        .canonicalize()
        .with_context(|| format!("Could not resolve repo path: {}", repo_root.display()))?;
    let normalized = canonical
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase();
    let mut digest = Sha256::new();
    digest.update(b"baron-project-identity-v2\0");
    digest.update(normalized.as_bytes());
    Ok(format!("{:x}", digest.finalize()))
}

pub fn capsule_key(project_slug: &str, project_id: &str) -> String {
    let short_id: String = project_id.chars().take(12).collect();
    format!("{project_slug}--{short_id}")
}

pub fn identity(project_slug: String, project_id: String) -> ProjectIdentity {
    let capsule_key = capsule_key(&project_slug, &project_id);
    ProjectIdentity {
        project_id,
        project_slug,
        capsule_key,
    }
}
