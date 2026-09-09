//! Runtime identity for one Baron operation.
//!
//! Project configuration may retain historical adapter values for migration,
//! but correctness-sensitive work must carry an explicit supported adapter.
//! This type is intentionally in-memory and is not part of any persisted
//! Baron schema.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

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
}

impl fmt::Display for OperationIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAdapter => write!(formatter, "explicit adapter identity is required"),
            Self::UnsupportedAdapter(value) => write!(
                formatter,
                "unsupported runtime adapter `{value}`; supported adapters are codex and claude"
            ),
        }
    }
}

impl std::error::Error for OperationIdentityError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationContext {
    pub adapter: SupportedAdapter,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl OperationContext {
    pub const fn new(adapter: SupportedAdapter) -> Self {
        Self {
            adapter,
            session_id: None,
            request_id: None,
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

    pub fn adapter_kind(&self) -> AdapterKind {
        self.adapter.as_adapter_kind()
    }
}
