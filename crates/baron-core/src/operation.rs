//! Runtime identity for one Baron operation.
//!
//! Project configuration may retain historical adapter values for migration,
//! but correctness-sensitive work must carry an explicit supported adapter.
//! This type is intentionally in-memory and is not part of any persisted
//! Baron schema.

use std::fmt;
use std::str::FromStr;

use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::AdapterKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupportedAdapter {
    Codex,
    Claude,
}

impl SupportedAdapter {
    pub fn parse(value: &str) -> Result<Self, OperationIdentityError> {
        value.parse()
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    pub const fn as_adapter_kind(self) -> AdapterKind {
        match self {
            Self::Codex => AdapterKind::Codex,
            Self::Claude => AdapterKind::Claude,
        }
    }
}

impl FromStr for SupportedAdapter {
    type Err = OperationIdentityError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "codex" => Ok(Self::Codex),
            "claude" => Ok(Self::Claude),
            value => Err(OperationIdentityError::UnsupportedAdapter(
                value.to_string(),
            )),
        }
    }
}

impl TryFrom<AdapterKind> for SupportedAdapter {
    type Error = OperationIdentityError;

    fn try_from(value: AdapterKind) -> Result<Self, Self::Error> {
        match value {
            AdapterKind::Codex => Ok(Self::Codex),
            AdapterKind::Claude => Ok(Self::Claude),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationIdentityError {
    MissingAdapter,
    UnsupportedAdapter(String),
    InvalidField { field: String, reason: String },
    RandomIdentifier(String),
}

impl fmt::Display for OperationIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAdapter => write!(formatter, "explicit adapter identity is required"),
            Self::UnsupportedAdapter(value) => write!(
                formatter,
                "unsupported runtime adapter `{value}`; supported adapters are codex and claude"
            ),
            Self::InvalidField { field, reason } => write!(formatter, "invalid {field}: {reason}"),
            Self::RandomIdentifier(error) => {
                write!(
                    formatter,
                    "could not synthesize a lifecycle identifier: {error}"
                )
            }
        }
    }
}

impl std::error::Error for OperationIdentityError {}

pub const MAX_IDENTIFIER_CHARS: usize = 256;
const RANDOM_IDENTIFIER_BYTES: usize = 16;

/// Complete identity for one correctness-sensitive Baron lifecycle operation.
///
/// Fields are private so callers can only create this value through the
/// validating constructors below. `OperationContext` remains as a compatible
/// optional diagnostic view for older read-only paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LifecycleIdentity {
    project_id: String,
    task_id: String,
    operation_id: String,
    adapter: SupportedAdapter,
    session_id: String,
    request_id: String,
}

impl LifecycleIdentity {
    pub fn new(
        project_id: impl Into<String>,
        task_id: impl Into<String>,
        operation_id: impl Into<String>,
        adapter: SupportedAdapter,
        session_id: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Result<Self, OperationIdentityError> {
        Ok(Self {
            project_id: validate_identifier("project_id", project_id.into())?,
            task_id: validate_identifier("task_id", task_id.into())?,
            operation_id: validate_identifier("operation_id", operation_id.into())?,
            adapter,
            session_id: validate_identifier("session_id", session_id.into())?,
            request_id: validate_identifier("request_id", request_id.into())?,
        })
    }

    /// Resolve external optional identity fragments exactly once at ingress.
    /// Supplied non-blank values are preserved after normalization; missing or
    /// blank values receive independent random identifiers.
    pub fn resolve(
        project_id: &str,
        task: &str,
        adapter: SupportedAdapter,
        session_id: Option<&str>,
        request_id: Option<&str>,
    ) -> Result<Self, OperationIdentityError> {
        let project_id = validate_identifier("project_id", project_id.trim().to_string())?;
        let task_id = task_id_for_task(&project_id, task)?;
        let session_id = resolve_optional_identifier("session_id", session_id)?;
        let request_id = resolve_optional_identifier("request_id", request_id)?;
        let operation_id =
            operation_id_for_parts(&project_id, &task_id, adapter, &session_id, &request_id);
        Self::new(
            project_id,
            task_id,
            operation_id,
            adapter,
            session_id,
            request_id,
        )
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    pub const fn adapter(&self) -> SupportedAdapter {
        self.adapter
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }
}

/// Derive the one task identity for a project and canonical task text.
pub fn task_id_for_task(project_id: &str, task: &str) -> Result<String, OperationIdentityError> {
    let project_id = validate_identifier("project_id", project_id.trim().to_string())?;
    let task = canonical_task_text(task)?;
    Ok(format!(
        "task-{}",
        digest_prefix(
            b"baron-task-v2",
            &[project_id.as_bytes(), task.as_bytes()],
            10,
        )
    ))
}

/// Derive an operation identity from the complete operation tuple.
pub fn operation_id_for_parts(
    project_id: &str,
    task_id: &str,
    adapter: SupportedAdapter,
    session_id: &str,
    request_id: &str,
) -> String {
    format!(
        "operation-{}",
        digest_prefix(
            b"baron-operation-v2",
            &[
                project_id.as_bytes(),
                task_id.as_bytes(),
                adapter.as_str().as_bytes(),
                session_id.as_bytes(),
                request_id.as_bytes(),
            ],
            12,
        )
    )
}

pub fn canonical_task_text(task: &str) -> Result<String, OperationIdentityError> {
    let canonical = task.trim().replace("\r\n", "\n").replace('\r', "\n");
    if canonical.is_empty() {
        return Err(invalid_field("task", "must not be empty"));
    }
    Ok(canonical)
}

fn validate_identifier(field: &str, value: String) -> Result<String, OperationIdentityError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(invalid_field(field, "must not be empty"));
    }
    if value.chars().count() > MAX_IDENTIFIER_CHARS {
        return Err(invalid_field(
            field,
            &format!("exceeds the {MAX_IDENTIFIER_CHARS} character limit"),
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(invalid_field(field, "must not contain control characters"));
    }
    Ok(value)
}

fn resolve_optional_identifier(
    field: &str,
    value: Option<&str>,
) -> Result<String, OperationIdentityError> {
    let value = value.map(str::trim).filter(|value| !value.is_empty());
    match value {
        Some(value) => validate_identifier(field, value.to_string()),
        None => synthesize_identifier(field),
    }
}

fn synthesize_identifier(field: &str) -> Result<String, OperationIdentityError> {
    let mut bytes = [0u8; RANDOM_IDENTIFIER_BYTES];
    getrandom(&mut bytes)
        .map_err(|error| OperationIdentityError::RandomIdentifier(error.to_string()))?;
    let suffix = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    validate_identifier(field, format!("baron-{field}-{suffix}"))
}

fn digest_prefix(domain: &[u8], values: &[&[u8]], bytes: usize) -> String {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update([0]);
    for value in values {
        digest.update(value);
        digest.update([0]);
    }
    digest
        .finalize()
        .iter()
        .take(bytes)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn invalid_field(field: &str, reason: &str) -> OperationIdentityError {
    OperationIdentityError::InvalidField {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationContext {
    pub adapter: SupportedAdapter,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
}

impl OperationContext {
    pub const fn new(adapter: SupportedAdapter) -> Self {
        Self {
            adapter,
            session_id: None,
            request_id: None,
            task_id: None,
            operation_id: None,
        }
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    pub fn with_operation_id(mut self, operation_id: impl Into<String>) -> Self {
        self.operation_id = Some(operation_id.into());
        self
    }

    pub fn adapter_kind(&self) -> AdapterKind {
        self.adapter.as_adapter_kind()
    }

    pub fn from_identity(identity: &LifecycleIdentity) -> Self {
        Self {
            adapter: identity.adapter,
            session_id: Some(identity.session_id.clone()),
            request_id: Some(identity.request_id.clone()),
            task_id: Some(identity.task_id.clone()),
            operation_id: Some(identity.operation_id.clone()),
        }
    }

    pub fn lifecycle_identity(
        &self,
        project_id: &str,
    ) -> Result<LifecycleIdentity, OperationIdentityError> {
        let session_id = self
            .session_id
            .as_deref()
            .ok_or_else(|| invalid_field("session_id", "is required"))?;
        let request_id = self
            .request_id
            .as_deref()
            .ok_or_else(|| invalid_field("request_id", "is required"))?;
        let task_id = self
            .task_id
            .as_deref()
            .ok_or_else(|| invalid_field("task_id", "is required"))?;
        let operation_id = self
            .operation_id
            .as_deref()
            .ok_or_else(|| invalid_field("operation_id", "is required"))?;
        LifecycleIdentity::new(
            project_id,
            task_id,
            operation_id,
            self.adapter,
            session_id,
            request_id,
        )
    }
}
