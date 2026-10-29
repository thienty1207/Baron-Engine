use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::Read;
use std::path::{Component, Path};
use std::sync::{Arc, Mutex};

use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

pub const REQUIRED_LANGUAGES: [&str; 7] = [
    "python",
    "cpp",
    "java",
    "rust",
    "go",
    "typescript",
    "csharp",
];

pub const SHARED_ATTACK_CLASSES: [&str; 8] = [
    "trust_boundary_and_input_validation",
    "authentication_authorization_and_ownership",
    "secrets_sessions_and_cryptography",
    "query_serialization_and_parsing",
    "files_uploads_and_outbound_network",
    "process_execution_and_information_disclosure",
    "concurrency_resources_and_supply_chain",
    "deployment_and_security_configuration",
];

const MAX_REPOSITORY_ENTRIES: usize = 20_000;
const MAX_PATHS_PER_LANGUAGE: usize = 96;
const MAX_EVIDENCE_PER_LANGUAGE: usize = 160;
const MAX_SCOPE_EXCLUSIONS: usize = 256;
const MAX_UNSUPPORTED_EVIDENCE: usize = 256;
const MAX_STACK_SIGNALS: usize = 128;
const MAX_HEADERS_PER_LANGUAGE: usize = 96;
const MAX_GAPS: usize = 128;
const MAX_REPOSITORY_PATH_BYTES: usize = 1024;
const MAX_MANIFEST_BYTES: u64 = 128 * 1024;
const MAX_TOTAL_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SOURCE_EVIDENCE_BYTES: u64 = 64 * 1024;
const GENERIC_SCOPE: &str = "planned_application_or_security_boundary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundedReadError {
    LimitExceeded,
    Unreadable,
}

/// Opens a validated regular file once and enforces the read ceiling on that
/// handle, so a path swap or concurrent file growth cannot bypass the bound.
fn read_bounded_text_file(
    path: &Path,
    max_bytes: u64,
) -> std::result::Result<Option<(String, u64)>, BoundedReadError> {
    let Some(file) = crate::safe_io::open_regular_file_for_read(path)
        .map_err(|_| BoundedReadError::Unreadable)?
    else {
        return Ok(None);
    };
    let metadata = file.metadata().map_err(|_| BoundedReadError::Unreadable)?;
    if metadata.len() > max_bytes {
        return Err(BoundedReadError::LimitExceeded);
    }

    let read_limit = max_bytes.saturating_add(1);
    let mut bytes = Vec::with_capacity(metadata.len().min(max_bytes) as usize);
    file.take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|_| BoundedReadError::Unreadable)?;
    if bytes.len() as u64 > max_bytes {
        return Err(BoundedReadError::LimitExceeded);
    }
    let byte_count = bytes.len() as u64;
    String::from_utf8(bytes)
        .map(|content| Some((content, byte_count)))
        .map_err(|_| BoundedReadError::Unreadable)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditEvidence {
    pub path: String,
    pub signal: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditHeaderCoverage {
    pub path: String,
    pub interpretations: Vec<String>,
    pub consuming_translation_units: Vec<String>,
    pub ambiguous: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditLanguageCoverage {
    pub language: String,
    pub confidence: String,
    pub in_scope: bool,
    pub evidence: Vec<SecurityAuditEvidence>,
    pub source_paths: Vec<String>,
    #[serde(default)]
    pub in_scope_paths: Vec<String>,
    #[serde(default)]
    pub lifecycle_surfaces: Vec<String>,
    pub headers: Vec<SecurityAuditHeaderCoverage>,
    pub rule_pack: String,
    pub planned_attack_classes: Vec<String>,
    pub coverage_gaps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditStackSignal {
    pub kind: String,
    pub language: String,
    pub name: String,
    pub evidence_paths: Vec<String>,
    pub rule_pack: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditScopeExclusion {
    pub path: String,
    pub rule_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditUnsupportedEvidence {
    pub language: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditDiscovery {
    pub languages: Vec<SecurityAuditLanguageCoverage>,
    pub stack_signals: Vec<SecurityAuditStackSignal>,
    pub scope_exclusions: Vec<SecurityAuditScopeExclusion>,
    pub unsupported_evidence: Vec<SecurityAuditUnsupportedEvidence>,
    pub coverage_gaps: Vec<String>,
    pub scan_truncated: bool,
}

#[derive(Debug, Clone, Copy)]
struct ScanLimits {
    max_entries: usize,
    max_paths_per_language: usize,
    max_evidence_per_language: usize,
    max_scope_exclusions: usize,
    max_manifest_bytes: u64,
    max_total_manifest_bytes: u64,
    max_source_evidence_bytes: u64,
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            max_entries: MAX_REPOSITORY_ENTRIES,
            max_paths_per_language: MAX_PATHS_PER_LANGUAGE,
            max_evidence_per_language: MAX_EVIDENCE_PER_LANGUAGE,
            max_scope_exclusions: MAX_SCOPE_EXCLUSIONS,
            max_manifest_bytes: MAX_MANIFEST_BYTES,
            max_total_manifest_bytes: MAX_TOTAL_MANIFEST_BYTES,
            max_source_evidence_bytes: MAX_SOURCE_EVIDENCE_BYTES,
        }
    }
}

#[derive(Default)]
struct LanguageDraft {
    source_paths: BTreeSet<String>,
    evidence: BTreeSet<(String, String)>,
    confidence: Confidence,
    headers: BTreeMap<String, HeaderDraft>,
    gaps: BTreeSet<String>,
}

#[derive(Default)]
struct HeaderDraft {
    interpretations: BTreeSet<String>,
    consumers: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Confidence {
    #[default]
    Unknown,
    Low,
    Medium,
    High,
    Ambiguous,
}

impl Confidence {
    fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Ambiguous => "ambiguous",
        }
    }
}

#[derive(Debug, Clone)]
struct ManifestRecord {
    path: String,
    language: &'static str,
    name: String,
    content: Option<String>,
}

#[derive(Debug, Clone)]
struct TranslationUnit {
    path: String,
    language: &'static str,
}

/// Inspect repository evidence without executing project-controlled code.
/// Ignore files and VCS defaults are disabled so every exclusion is explicit
/// and returned to the caller as a scope rule.
pub fn discover_security_audit_coverage(
    repository_root: &Path,
    requested_scope: &[String],
) -> SecurityAuditDiscovery {
    discover_with_limits(repository_root, requested_scope, ScanLimits::default())
}

fn discover_with_limits(
    repository_root: &Path,
    requested_scope: &[String],
    limits: ScanLimits,
) -> SecurityAuditDiscovery {
    let mut drafts: BTreeMap<&'static str, LanguageDraft> = BTreeMap::new();
    let mut manifests = Vec::new();
    let mut translation_units = Vec::new();
    let mut headers = BTreeSet::new();
    let mut unsupported_evidence = Vec::new();
    let mut coverage_gaps = BTreeSet::new();
    let mut scan_truncated = false;
    let mut total_manifest_bytes = 0u64;
    let mut total_source_bytes = 0u64;
    let excluded = Arc::new(Mutex::new((Vec::new(), false)));

    if !repository_root.is_dir() {
        return SecurityAuditDiscovery {
            languages: languages_from_requested_scope(requested_scope),
            stack_signals: Vec::new(),
            scope_exclusions: Vec::new(),
            unsupported_evidence: Vec::new(),
            coverage_gaps: vec!["repository_root_unavailable".to_string()],
            scan_truncated: false,
        };
    }

    let root_for_filter = repository_root.to_path_buf();
    let exclusions_for_filter = Arc::clone(&excluded);
    let mut walker = WalkBuilder::new(repository_root);
    walker
        .hidden(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .parents(false)
        .follow_links(false)
        .threads(1)
        .sort_by_file_name(|left, right| left.cmp(right))
        .filter_entry(move |entry| {
            if entry.depth() == 0
                || !entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_dir())
            {
                return true;
            }
            let Some((rule_id, reason)) = exclusion_rule(entry.path()) else {
                return true;
            };
            let Ok(relative) = entry.path().strip_prefix(&root_for_filter) else {
                return false;
            };
            let (path, path_truncated) = display_path(relative);
            if let Ok(mut state) = exclusions_for_filter.lock() {
                if path_truncated {
                    state.1 = true;
                }
                if state.0.len() < limits.max_scope_exclusions {
                    state.0.push(SecurityAuditScopeExclusion {
                        path,
                        rule_id: rule_id.to_string(),
                        reason: reason.to_string(),
                    });
                } else {
                    state.1 = true;
                }
            }
            false
        });

    for (visited, result) in walker.build().enumerate() {
        if visited >= limits.max_entries {
            scan_truncated = true;
            coverage_gaps.insert("repository_scan_entry_limit_reached".to_string());
            break;
        }
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => {
                coverage_gaps.insert("repository_scan_encountered_unreadable_entry".to_string());
                continue;
            }
        };
        let Some(file_type) = entry.file_type() else {
            coverage_gaps.insert("repository_scan_entry_type_unknown".to_string());
            continue;
        };
        if file_type.is_symlink() {
            coverage_gaps.insert("symlink_source_not_followed".to_string());
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let relative = match entry.path().strip_prefix(repository_root) {
            Ok(path) => path,
            Err(_) => {
                coverage_gaps.insert("repository_scan_path_outside_root".to_string());
                continue;
            }
        };
        let Some(relative) = normalized_relative_path(relative) else {
            coverage_gaps.insert("repository_contains_non_utf8_or_unsafe_path".to_string());
            continue;
        };
        let path = entry.path();

        if let Some(language) = source_language(&relative) {
            if language == "c" {
                add_unsupported_evidence(
                    &mut unsupported_evidence,
                    language,
                    &relative,
                    &mut scan_truncated,
                    &mut coverage_gaps,
                );
                translation_units.push(TranslationUnit {
                    path: relative.clone(),
                    language,
                });
            } else if language == "php" || language == "javascript" {
                add_unsupported_evidence(
                    &mut unsupported_evidence,
                    language,
                    &relative,
                    &mut scan_truncated,
                    &mut coverage_gaps,
                );
            } else if language == "header" {
                headers.insert(relative.clone());
            } else {
                let draft = drafts.entry(language).or_default();
                draft.confidence = draft.confidence.max(Confidence::High);
                add_evidence(draft, &relative, "source_extension", limits);
                add_source_path(draft, &relative, limits, &mut scan_truncated);
                if matches!(language, "c" | "cpp") {
                    translation_units.push(TranslationUnit {
                        path: relative.clone(),
                        language,
                    });
                }
            }
        }

        if let Some((language, _manifest_kind)) = manifest_language(&relative) {
            let manifest_name = Path::new(&relative)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let remaining_bytes = limits
                .max_total_manifest_bytes
                .saturating_sub(total_manifest_bytes);
            let read_limit = limits.max_manifest_bytes.min(remaining_bytes);
            let content = match read_bounded_text_file(path, read_limit) {
                Ok(Some((content, bytes_read))) => {
                    total_manifest_bytes = total_manifest_bytes.saturating_add(bytes_read);
                    Some(content)
                }
                Ok(None) => {
                    coverage_gaps.insert(format!("manifest_metadata_unavailable:{relative}"));
                    None
                }
                Err(BoundedReadError::LimitExceeded) => {
                    coverage_gaps.insert(format!("manifest_read_limit_reached:{relative}"));
                    None
                }
                Err(BoundedReadError::Unreadable) => {
                    coverage_gaps.insert(format!("manifest_unreadable:{relative}"));
                    None
                }
            };
            manifests.push(ManifestRecord {
                path: relative,
                language,
                name: manifest_name,
                content,
            });
        }
    }

    let excluded_build_directories = excluded
        .lock()
        .map(|state| {
            state
                .0
                .iter()
                .filter(|rule| rule.rule_id == "build_output")
                .map(|rule| rule.path.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for directory in excluded_build_directories {
        let relative = format!("{directory}/compile_commands.json");
        let path = repository_root.join(&relative);
        let remaining_bytes = limits
            .max_total_manifest_bytes
            .saturating_sub(total_manifest_bytes);
        let read_limit = limits.max_manifest_bytes.min(remaining_bytes);
        let content = match read_bounded_text_file(&path, read_limit) {
            Ok(Some((content, bytes_read))) => {
                total_manifest_bytes = total_manifest_bytes.saturating_add(bytes_read);
                Some(content)
            }
            Ok(None) => continue,
            Err(BoundedReadError::LimitExceeded) => {
                coverage_gaps.insert(format!("manifest_read_limit_reached:{relative}"));
                None
            }
            Err(BoundedReadError::Unreadable) => {
                coverage_gaps.insert(format!("manifest_unreadable:{relative}"));
                None
            }
        };
        manifests.push(ManifestRecord {
            path: relative,
            language: "cpp",
            name: "compile_commands.json".to_string(),
            content,
        });
    }

    let mut compilation_header_evidence = Vec::new();
    for manifest in manifests
        .iter()
        .filter(|manifest| manifest.name == "compile_commands.json")
    {
        let Some(content) = manifest.content.as_deref() else {
            continue;
        };
        let entries = match serde_json::from_str::<serde_json::Value>(content) {
            Ok(serde_json::Value::Array(entries)) => entries,
            _ => {
                coverage_gaps.insert(format!("compilation_database_invalid:{}", manifest.path));
                continue;
            }
        };
        for entry in entries {
            let Some(file) = entry.get("file").and_then(serde_json::Value::as_str) else {
                coverage_gaps.insert(format!(
                    "compilation_database_entry_missing_file:{}",
                    manifest.path
                ));
                continue;
            };
            let Some(file_path) = compilation_database_file_path(
                repository_root,
                &manifest.path,
                entry.get("directory").and_then(serde_json::Value::as_str),
                file,
            ) else {
                coverage_gaps.insert(format!(
                    "compilation_database_path_outside_or_invalid:{}",
                    manifest.path
                ));
                continue;
            };
            let command_language = compilation_command_language(&entry, &file_path);
            if headers.contains(&file_path) {
                if let Some(language) = command_language {
                    compilation_header_evidence.push((file_path, language, manifest.path.clone()));
                }
                continue;
            }
            if let Some(unit) = translation_units
                .iter_mut()
                .find(|unit| unit.path == file_path)
            {
                if let Some(language) = command_language {
                    unit.language = language;
                }
            }
        }
    }
    for (header, language, database) in compilation_header_evidence {
        let header_draft = drafts
            .entry("cpp")
            .or_default()
            .headers
            .entry(header.clone())
            .or_default();
        header_draft.interpretations.insert(language.to_string());
        header_draft.consumers.insert(database);
    }

    let mut source_contents = BTreeMap::<String, String>::new();
    let mut source_candidates = translation_units
        .iter()
        .map(|unit| unit.path.clone())
        .collect::<BTreeSet<_>>();
    for (language, draft) in &drafts {
        for path in &draft.source_paths {
            if source_language(path).is_some_and(|detected| detected == *language) {
                source_candidates.insert(path.clone());
            }
        }
    }
    for source_path in source_candidates {
        if total_source_bytes >= limits.max_source_evidence_bytes {
            scan_truncated = true;
            coverage_gaps.insert("source_evidence_read_limit_reached".to_string());
            break;
        }
        let absolute_path = repository_root.join(&source_path);
        let remaining_bytes = limits
            .max_source_evidence_bytes
            .saturating_sub(total_source_bytes);
        let read_limit = limits.max_manifest_bytes.min(remaining_bytes);
        let content = match read_bounded_text_file(&absolute_path, read_limit) {
            Ok(Some((content, bytes_read))) => {
                total_source_bytes = total_source_bytes.saturating_add(bytes_read);
                content
            }
            Ok(None) => continue,
            Err(BoundedReadError::LimitExceeded) => {
                coverage_gaps.insert(format!("source_evidence_too_large:{source_path}"));
                continue;
            }
            Err(BoundedReadError::Unreadable) => {
                coverage_gaps.insert(format!("source_evidence_unreadable:{source_path}"));
                continue;
            }
        };
        source_contents.insert(source_path, content);
    }

    let mut stack_signals = Vec::new();
    for (path, content) in &source_contents {
        let Some(language) = source_language(path) else {
            continue;
        };
        if language == "c" || language == "header" || !REQUIRED_LANGUAGES.contains(&language) {
            continue;
        }
        for (framework, marker) in source_framework_catalog(language) {
            if source_mentions_framework(language, content, marker) {
                add_stack_signal(
                    &mut stack_signals,
                    SecurityAuditStackSignal {
                        kind: "framework".to_string(),
                        language: language.to_string(),
                        name: framework.to_string(),
                        evidence_paths: vec![path.clone()],
                        rule_pack: Some(format!("rules/{language}/FRAMEWORKS.md")),
                    },
                    &mut scan_truncated,
                    &mut coverage_gaps,
                );
            }
        }
    }

    for translation_unit in &translation_units {
        let Some(content) = source_contents.get(&translation_unit.path) else {
            continue;
        };
        for include in quoted_includes(content) {
            if let Some(header) =
                resolve_header(repository_root, &translation_unit.path, &include, &headers)
            {
                let header_draft = drafts
                    .entry("cpp")
                    .or_default()
                    .headers
                    .entry(header.clone())
                    .or_default();
                header_draft
                    .interpretations
                    .insert(translation_unit.language.to_string());
                header_draft.consumers.insert(translation_unit.path.clone());
            }
        }
    }

    for manifest in &manifests {
        if let Some((name, rule_pack)) = build_system_signal(manifest) {
            add_stack_signal(
                &mut stack_signals,
                SecurityAuditStackSignal {
                    kind: "build_system".to_string(),
                    language: manifest.language.to_string(),
                    name: name.to_string(),
                    evidence_paths: vec![manifest.path.clone()],
                    rule_pack,
                },
                &mut scan_truncated,
                &mut coverage_gaps,
            );
        }
        for (framework, markers) in framework_catalog(manifest.language) {
            if markers
                .iter()
                .any(|marker| manifest_confirms_framework(manifest, marker))
            {
                add_stack_signal(
                    &mut stack_signals,
                    SecurityAuditStackSignal {
                        kind: "framework".to_string(),
                        language: manifest.language.to_string(),
                        name: framework.to_string(),
                        evidence_paths: vec![manifest.path.clone()],
                        rule_pack: Some(format!("rules/{}/FRAMEWORKS.md", manifest.language)),
                    },
                    &mut scan_truncated,
                    &mut coverage_gaps,
                );
            }
        }
        let draft = drafts.entry(manifest.language).or_default();
        draft.confidence = draft.confidence.max(Confidence::Medium);
        add_evidence(
            draft,
            &manifest.path,
            "build_or_dependency_manifest",
            limits,
        );
    }

    for header in &headers {
        let draft = drafts.entry("cpp").or_default();
        let header_draft = draft.headers.entry(header.clone()).or_default();
        if header_draft.interpretations.is_empty() {
            if let Some(language) = cmake_language_for_header(header, &manifests) {
                header_draft.interpretations.insert(language.to_string());
                header_draft.consumers.insert(
                    cmake_path_for_header(header, &manifests)
                        .unwrap_or_default()
                        .to_string(),
                );
            } else {
                header_draft.interpretations.insert("c".to_string());
                header_draft.interpretations.insert("cpp".to_string());
            }
        }
        if header_draft.interpretations.contains("c") {
            draft
                .gaps
                .insert(format!("header_language_ambiguous_or_c_only:{header}"));
        }
        draft.confidence = if header_draft.interpretations.len() > 1 {
            Confidence::Ambiguous
        } else {
            draft.confidence.max(Confidence::Medium)
        };
        add_evidence(draft, header, "c_or_cpp_header", limits);
        add_source_path(draft, header, limits, &mut scan_truncated);
    }

    if manifests
        .iter()
        .any(|manifest| manifest.language == "typescript")
    {
        let has_ts_source = drafts
            .get("typescript")
            .is_some_and(|draft| !draft.source_paths.is_empty());
        let has_ts_configuration = manifests.iter().any(manifest_confirms_typescript);
        if !has_ts_source {
            let draft = drafts.entry("typescript").or_default();
            draft.confidence = if has_ts_configuration {
                Confidence::Medium
            } else {
                Confidence::Low
            };
            if !has_ts_configuration {
                draft
                    .gaps
                    .insert("node_manifest_without_typescript_evidence".to_string());
            }
            if let Some(manifest) = manifests.iter().find(|manifest| {
                manifest.language == "typescript"
                    && matches!(
                        manifest.name.as_str(),
                        "package.json" | "package-lock.json" | "pnpm-lock.yaml" | "yarn.lock"
                    )
            }) {
                add_evidence(
                    draft,
                    &manifest.path,
                    "node_ecosystem_manifest_only",
                    limits,
                );
            }
        }
    }

    for unsupported in &unsupported_evidence {
        coverage_gaps.insert(format!(
            "unsupported_language_evidence:{}",
            unsupported.language
        ));
    }

    let mut scope_exclusions = excluded
        .lock()
        .map(|state| {
            if state.1 {
                scan_truncated = true;
                coverage_gaps.insert("scope_exclusion_evidence_truncated".to_string());
            }
            state.0.clone()
        })
        .unwrap_or_else(|_| {
            coverage_gaps.insert("scope_exclusion_record_lock_poisoned".to_string());
            Vec::new()
        });
    scope_exclusions.sort_by(|left, right| left.path.cmp(&right.path));
    let stack_signals = dedupe_stack_signals(stack_signals);
    unsupported_evidence
        .sort_by(|left, right| (&left.language, &left.path).cmp(&(&right.language, &right.path)));
    unsupported_evidence.dedup();
    let mut languages = Vec::new();
    for (language, draft) in drafts {
        if draft.evidence.is_empty() && draft.source_paths.is_empty() {
            continue;
        }
        let source_paths = draft.source_paths.into_iter().collect::<Vec<_>>();
        let in_scope_paths = source_paths
            .iter()
            .filter(|path| path_is_in_scope(path, requested_scope))
            .cloned()
            .collect::<Vec<_>>();
        let in_scope = !in_scope_paths.is_empty()
            || draft
                .evidence
                .iter()
                .any(|(path, _)| path_is_in_scope(path, requested_scope));
        let header_count = draft.headers.len();
        let mut language_gaps = draft.gaps.into_iter().collect::<Vec<_>>();
        if header_count > MAX_HEADERS_PER_LANGUAGE {
            language_gaps.push("cpp_header_coverage_truncated".to_string());
        }
        if !in_scope {
            language_gaps.push("language_detected_outside_requested_scope".to_string());
        } else {
            language_gaps
                .push("framework_detection_catalog_non_exhaustive_requires_validation".to_string());
        }
        let headers = draft
            .headers
            .into_iter()
            .take(MAX_HEADERS_PER_LANGUAGE)
            .map(|(path, header)| {
                let interpretations = header.interpretations.into_iter().collect::<Vec<_>>();
                SecurityAuditHeaderCoverage {
                    path,
                    ambiguous: interpretations.len() > 1,
                    interpretations,
                    consuming_translation_units: header.consumers.into_iter().collect(),
                }
            })
            .collect::<Vec<_>>();
        let evidence = draft
            .evidence
            .into_iter()
            .map(|(path, signal)| SecurityAuditEvidence { path, signal })
            .collect::<Vec<_>>();
        languages.push(SecurityAuditLanguageCoverage {
            language: language.to_string(),
            confidence: draft.confidence.as_str().to_string(),
            in_scope,
            evidence,
            source_paths,
            in_scope_paths,
            lifecycle_surfaces: Vec::new(),
            headers,
            rule_pack: format!("rules/{language}/RULES.md"),
            planned_attack_classes: if in_scope {
                SHARED_ATTACK_CLASSES
                    .iter()
                    .map(|class| class.to_string())
                    .collect()
            } else {
                Vec::new()
            },
            coverage_gaps: language_gaps,
        });
    }

    for language in languages_from_requested_scope(requested_scope) {
        if let Some(coverage) = languages
            .iter_mut()
            .find(|coverage| coverage.language == language.language)
        {
            for evidence in language.evidence {
                if !coverage.evidence.contains(&evidence) {
                    coverage.evidence.push(evidence);
                }
            }
            for path in language.source_paths {
                if !coverage.source_paths.contains(&path) {
                    coverage.source_paths.push(path.clone());
                }
                if !coverage.in_scope_paths.contains(&path) {
                    coverage.in_scope_paths.push(path);
                }
            }
            coverage.source_paths.sort();
            coverage.in_scope_paths.sort();
            coverage.in_scope = true;
            coverage.confidence = if coverage.confidence == "unknown" {
                "planned".to_string()
            } else {
                coverage.confidence.clone()
            };
        } else {
            languages.push(language);
        }
    }
    languages.sort_by(|left, right| left.language.cmp(&right.language));
    if languages.iter().all(|language| !language.in_scope) {
        coverage_gaps.insert("no_supported_language_evidence_in_scope".to_string());
    }
    if scan_truncated {
        coverage_gaps.insert("coverage_inventory_truncated".to_string());
    }

    SecurityAuditDiscovery {
        languages,
        stack_signals,
        scope_exclusions,
        unsupported_evidence,
        coverage_gaps: coverage_gaps.into_iter().take(MAX_GAPS).collect(),
        scan_truncated,
    }
}

fn languages_from_requested_scope(
    requested_scope: &[String],
) -> Vec<SecurityAuditLanguageCoverage> {
    let mut languages = BTreeMap::<&'static str, BTreeSet<String>>::new();
    for path in requested_scope {
        if !safe_relative_text(path) {
            continue;
        }
        if let Some(language) = language_for_path(path) {
            if REQUIRED_LANGUAGES.contains(&language) {
                languages
                    .entry(language)
                    .or_default()
                    .insert(path.replace('\\', "/"));
            }
        }
    }
    languages
        .into_iter()
        .map(|(language, paths)| SecurityAuditLanguageCoverage {
            language: language.to_string(),
            confidence: "planned".to_string(),
            in_scope: true,
            evidence: paths
                .iter()
                .map(|path| SecurityAuditEvidence {
                    path: path.clone(),
                    signal: "requested_scope_path".to_string(),
                })
                .collect(),
            source_paths: paths.iter().cloned().collect(),
            in_scope_paths: paths.into_iter().collect(),
            lifecycle_surfaces: Vec::new(),
            headers: Vec::new(),
            rule_pack: format!("rules/{language}/RULES.md"),
            planned_attack_classes: SHARED_ATTACK_CLASSES
                .iter()
                .map(|class| class.to_string())
                .collect(),
            coverage_gaps: Vec::new(),
        })
        .collect()
}

fn source_language(path: &str) -> Option<&'static str> {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("py" | "pyi") => Some("python"),
        Some("cc" | "cpp" | "cxx") => Some("cpp"),
        Some("h" | "hh" | "hpp" | "hxx") => Some("header"),
        Some("c") => Some("c"),
        Some("java") => Some("java"),
        Some("rs") => Some("rust"),
        Some("go") => Some("go"),
        Some("ts" | "tsx" | "mts" | "cts") => Some("typescript"),
        Some("cs" | "csx") => Some("csharp"),
        Some("php") => Some("php"),
        Some("js" | "jsx" | "mjs" | "cjs") => Some("javascript"),
        _ => None,
    }
}

fn language_for_path(path: &str) -> Option<&'static str> {
    if let Some(language) = source_language(path) {
        if language == "header" {
            return Some("cpp");
        }
        if REQUIRED_LANGUAGES.contains(&language) {
            return Some(language);
        }
        return None;
    }
    let name = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)?;
    manifest_language_by_name(&name).map(|(language, _)| language)
}

fn manifest_language(path: &str) -> Option<(&'static str, &'static str)> {
    let name = Path::new(path).file_name()?.to_str()?.to_ascii_lowercase();
    manifest_language_by_name(&name)
}

fn manifest_language_by_name(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "pyproject.toml" | "setup.py" | "setup.cfg" | "pipfile" | "pipfile.lock"
        | "poetry.lock" | "uv.lock" => Some(("python", "python_project_manifest")),
        "cargo.toml" | "cargo.lock" => Some(("rust", "cargo_manifest")),
        "go.mod" | "go.sum" | "go.work" | "go.work.sum" => Some(("go", "go_module_manifest")),
        "pom.xml" => Some(("java", "maven_manifest")),
        "build.gradle"
        | "build.gradle.kts"
        | "settings.gradle"
        | "settings.gradle.kts"
        | "gradle.properties" => Some(("java", "gradle_manifest")),
        "cmakelists.txt" | "meson.build" | "compile_commands.json" => {
            Some(("cpp", "cpp_build_manifest"))
        }
        "global.json"
        | "directory.build.props"
        | "directory.build.targets"
        | "nuget.config"
        | "packages.config"
        | "directory.packages.props" => Some(("csharp", "dotnet_build_manifest")),
        "package.json" | "package-lock.json" | "pnpm-lock.yaml" | "yarn.lock" | "bun.lock"
        | "bun.lockb" | "tsconfig.json" => Some(("typescript", "node_typescript_manifest")),
        _ if name.starts_with("requirements") && name.ends_with(".txt") => {
            Some(("python", "python_requirements_manifest"))
        }
        _ if name.ends_with(".csproj") || name.ends_with(".sln") || name.ends_with(".slnx") => {
            Some(("csharp", "dotnet_project_manifest"))
        }
        _ => None,
    }
}

fn compilation_database_file_path(
    repository_root: &Path,
    database_path: &str,
    directory: Option<&str>,
    file: &str,
) -> Option<String> {
    let database_parent = Path::new(database_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let base = if let Some(directory) = directory {
        let directory = Path::new(directory);
        if directory.is_absolute() {
            directory.to_path_buf()
        } else {
            repository_root.join(database_parent).join(directory)
        }
    } else {
        repository_root.join(database_parent)
    };
    let file = Path::new(file);
    let candidate = if file.is_absolute() {
        file.to_path_buf()
    } else {
        base.join(file)
    };
    let candidate = if candidate.is_absolute() {
        candidate
    } else {
        repository_root.join(candidate)
    };
    let relative = candidate.strip_prefix(repository_root).ok()?;
    let mut parts = Vec::<String>::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_str()?.to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if parts.is_empty() {
        return None;
    }
    let path = parts.join("/");
    safe_relative_text(&path).then_some(path)
}

fn compilation_command_language(
    entry: &serde_json::Value,
    file_path: &str,
) -> Option<&'static str> {
    let arguments = entry
        .get("arguments")
        .and_then(serde_json::Value::as_array)
        .map(|arguments| {
            arguments
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            entry
                .get("command")
                .and_then(serde_json::Value::as_str)
                .map(|command| command.split_whitespace().collect())
                .unwrap_or_default()
        });
    for (index, argument) in arguments.iter().enumerate() {
        let explicit = if *argument == "-x" {
            arguments.get(index + 1).copied()
        } else {
            argument.strip_prefix("-x")
        };
        match explicit.map(str::to_ascii_lowercase).as_deref() {
            Some("c" | "objective-c") => return Some("c"),
            Some("c++" | "cpp-output" | "c++-cpp-output" | "objective-c++") => return Some("cpp"),
            _ => {}
        }
    }
    match source_language(file_path) {
        Some("c") => Some("c"),
        Some("cpp") => Some("cpp"),
        _ => None,
    }
}

fn build_system_signal(manifest: &ManifestRecord) -> Option<(&'static str, Option<String>)> {
    let name = manifest.name.as_str();
    let result = match (manifest.language, name) {
        ("python", "pyproject.toml") => ("pyproject", None),
        ("python", "setup.py" | "setup.cfg") => ("setuptools", None),
        ("python", "pipfile" | "pipfile.lock") => ("pipenv", None),
        ("python", "poetry.lock") => ("poetry", None),
        ("python", "uv.lock") => ("uv", None),
        ("python", _) if name.starts_with("requirements") => ("pip-requirements", None),
        ("cpp", "cmakelists.txt") => ("cmake", None),
        ("cpp", "meson.build") => ("meson", None),
        ("cpp", "compile_commands.json") => ("compilation-database", None),
        ("java", "pom.xml") => ("maven", None),
        ("java", _) => ("gradle", None),
        ("rust", "cargo.toml" | "cargo.lock") => ("cargo", None),
        ("go", "go.mod" | "go.sum" | "go.work" | "go.work.sum") => ("go-modules", None),
        ("typescript", "tsconfig.json") => ("typescript-config", None),
        ("typescript", "package.json" | "package-lock.json") => ("npm", None),
        ("typescript", "pnpm-lock.yaml") => ("pnpm", None),
        ("typescript", "yarn.lock") => ("yarn", None),
        ("typescript", "bun.lock" | "bun.lockb") => ("bun", None),
        ("csharp", "global.json") => ("dotnet-sdk", None),
        ("csharp", "nuget.config" | "packages.config" | "directory.packages.props") => {
            ("nuget", None)
        }
        ("csharp", _) => ("msbuild", None),
        _ => return None,
    };
    Some(result)
}

fn framework_catalog(language: &str) -> &'static [(&'static str, &'static [&'static str])] {
    match language {
        "python" => &[
            ("django", &["django"]),
            ("fastapi", &["fastapi"]),
            ("flask", &["flask"]),
        ],
        "cpp" => &[
            ("drogon", &["drogon"]),
            ("crow", &["crow::crow", "crow"]),
            ("grpc-cpp", &["grpc::grpc++", "grpc++"]),
        ],
        "java" => &[
            ("spring", &["spring-boot", "spring-web"]),
            ("jakarta", &["jakarta.servlet", "jakarta.ws.rs"]),
        ],
        "rust" => &[
            ("axum", &["axum"]),
            ("actix-web", &["actix-web"]),
            ("rocket", &["rocket"]),
            ("sqlx", &["sqlx"]),
            ("diesel", &["diesel"]),
        ],
        "go" => &[
            ("net-http", &["net/http"]),
            ("gin", &["github.com/gin-gonic/gin"]),
            ("echo", &["github.com/labstack/echo"]),
            ("fiber", &["github.com/gofiber/fiber"]),
            ("chi", &["github.com/go-chi/chi"]),
        ],
        "typescript" => &[
            ("express", &["express"]),
            ("fastify", &["fastify"]),
            ("koa", &["koa"]),
            ("nestjs", &["@nestjs/core"]),
            ("nextjs", &["next"]),
        ],
        "csharp" => &[(
            "aspnet-core",
            &["microsoft.aspnetcore", "microsoft.net.sdk.web"],
        )],
        _ => &[],
    }
}

fn source_framework_catalog(language: &str) -> &'static [(&'static str, &'static str)] {
    match language {
        "python" => &[
            ("django", "django"),
            ("fastapi", "fastapi"),
            ("flask", "flask"),
        ],
        "cpp" => &[
            ("drogon", "drogon"),
            ("crow", "crow"),
            ("grpc-cpp", "grpcpp"),
        ],
        "java" => &[
            ("spring", "org.springframework"),
            ("jakarta", "jakarta.servlet"),
        ],
        "rust" => &[
            ("axum", "axum"),
            ("actix-web", "actix-web"),
            ("rocket", "rocket"),
            ("sqlx", "sqlx"),
            ("diesel", "diesel"),
        ],
        "go" => &[
            ("net-http", "net/http"),
            ("gin", "github.com/gin-gonic/gin"),
            ("echo", "github.com/labstack/echo"),
            ("fiber", "github.com/gofiber/fiber"),
            ("chi", "github.com/go-chi/chi"),
        ],
        "typescript" => &[
            ("express", "express"),
            ("fastify", "fastify"),
            ("koa", "koa"),
            ("nestjs", "@nestjs/core"),
            ("nextjs", "next"),
        ],
        "csharp" => &[("aspnet-core", "microsoft.aspnetcore")],
        _ => &[],
    }
}

fn manifest_confirms_framework(manifest: &ManifestRecord, marker: &str) -> bool {
    let Some(content) = manifest.content.as_deref() else {
        return false;
    };
    match manifest.language {
        "typescript" => match manifest.name.as_str() {
            "package.json" => node_json_declares_package(content, marker, false),
            "package-lock.json" => node_json_declares_package(content, marker, true),
            "pnpm-lock.yaml" | "yarn.lock" => node_text_lock_declares_package(content, marker),
            // Bun's binary lockfile is intentionally not interpreted as text.
            "bun.lock" => node_json_declares_package(content, marker, true),
            _ => false,
        },
        "python" => match manifest.name.as_str() {
            "pyproject.toml" => toml_declares_dependency(content, marker),
            "pipfile" => toml_declares_dependency(content, marker),
            "pipfile.lock" => python_lock_declares_package(content, marker),
            name if name.starts_with("requirements") => {
                requirements_declares_package(content, marker)
            }
            "setup.cfg" | "setup.py" => python_setup_declares_package(content, marker),
            _ => false,
        },
        "cpp" => match manifest.name.as_str() {
            "cmakelists.txt" => build_manifest_declares_marker(
                content,
                marker,
                &["find_package", "target_link_libraries", "pkg_check_modules"],
            ),
            "meson.build" => build_manifest_declares_marker(content, marker, &["dependency("]),
            _ => false,
        },
        "java" if manifest.name == "pom.xml" => xml_dependency_declares_marker(content, marker),
        "java" => gradle_declares_framework(content, marker),
        "rust" if manifest.name == "cargo.toml" => toml_declares_dependency(content, marker),
        "rust" if manifest.name == "cargo.lock" => cargo_lock_declares_package(content, marker),
        "go" if manifest.name == "go.mod" => go_mod_declares_package(content, marker),
        "csharp" => dotnet_manifest_declares_framework(content, marker),
        _ => false,
    }
}

fn manifest_confirms_typescript(manifest: &ManifestRecord) -> bool {
    if manifest.language != "typescript" {
        return false;
    }
    if manifest.name == "tsconfig.json" {
        return true;
    }
    let Some(content) = manifest.content.as_deref() else {
        return false;
    };
    match manifest.name.as_str() {
        "package.json" => node_json_declares_package(content, "typescript", false),
        "package-lock.json" => node_json_declares_package(content, "typescript", true),
        "pnpm-lock.yaml" | "yarn.lock" => node_text_lock_declares_package(content, "typescript"),
        "bun.lock" => node_json_declares_package(content, "typescript", true),
        // bun.lockb is binary, so an extension alone is not enough to distinguish JS.
        _ => false,
    }
}

fn node_json_declares_package(content: &str, package: &str, lockfile: bool) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return false;
    };
    if lockfile {
        let packages = value
            .get("packages")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|packages| {
                packages.keys().any(|path| {
                    path.strip_prefix("node_modules/") == Some(package)
                        || path.ends_with(&format!("/node_modules/{package}"))
                })
            });
        let dependencies = value
            .get("dependencies")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|dependencies| dependencies.contains_key(package));
        packages || dependencies
    } else {
        [
            "dependencies",
            "devDependencies",
            "peerDependencies",
            "optionalDependencies",
        ]
        .iter()
        .any(|section| {
            value
                .get(*section)
                .and_then(serde_json::Value::as_object)
                .is_some_and(|dependencies| dependencies.contains_key(package))
        })
    }
}

fn node_text_lock_declares_package(content: &str, package: &str) -> bool {
    content.lines().any(|raw_line| {
        let line = raw_line.trim().trim_start_matches('-').trim();
        let key = line
            .split_once(':')
            .map_or(line, |(key, _)| key)
            .trim_matches(['"', '\'']);
        let key = key.strip_prefix('/').unwrap_or(key);
        key.strip_prefix(package).is_some_and(|suffix| {
            suffix.starts_with('@') || suffix.starts_with(':') || suffix.is_empty()
        })
    })
}

fn toml_declares_dependency(content: &str, package: &str) -> bool {
    let Ok(value) = toml::from_str::<toml::Value>(content) else {
        return false;
    };
    fn visit(value: &toml::Value, package: &str) -> bool {
        match value {
            toml::Value::Table(table) => table.iter().any(|(key, child)| {
                (matches!(
                    key.as_str(),
                    "dependencies"
                        | "dev-dependencies"
                        | "build-dependencies"
                        | "optional-dependencies"
                        | "packages"
                        | "dev-packages"
                ) && toml_dependency_value_matches(child, package))
                    || visit(child, package)
            }),
            toml::Value::Array(values) => values.iter().any(|value| visit(value, package)),
            _ => false,
        }
    }
    visit(&value, package)
}

fn toml_dependency_value_matches(value: &toml::Value, package: &str) -> bool {
    match value {
        toml::Value::Table(table) => table.iter().any(|(name, detail)| {
            dependency_name_matches(name, package)
                || detail
                    .get("package")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|actual| dependency_name_matches(actual, package))
        }),
        toml::Value::Array(values) => values.iter().any(|value| {
            value
                .as_str()
                .is_some_and(|value| dependency_name_matches(&requirement_name(value), package))
        }),
        _ => false,
    }
}

fn dependency_name_matches(name: &str, marker: &str) -> bool {
    let normalize = |value: &str| {
        value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    normalize(name) == normalize(marker)
}

fn requirement_name(line: &str) -> String {
    let line = line
        .split_once('#')
        .map_or(line, |(requirement, _)| requirement)
        .trim();
    if line.is_empty() || line.starts_with('-') {
        return String::new();
    }
    let line = line.trim_matches(['"', '\'']);
    let end = line
        .find(|character: char| {
            matches!(
                character,
                '[' | '>' | '<' | '=' | '!' | '~' | ';' | ' ' | '\t'
            )
        })
        .unwrap_or(line.len());
    line[..end].to_string()
}

fn requirements_declares_package(content: &str, package: &str) -> bool {
    content
        .lines()
        .any(|line| dependency_name_matches(&requirement_name(line), package))
}

fn python_lock_declares_package(content: &str, package: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return false;
    };
    ["default", "develop"].iter().any(|section| {
        value
            .get(*section)
            .and_then(serde_json::Value::as_object)
            .is_some_and(|dependencies| {
                dependencies
                    .keys()
                    .any(|name| dependency_name_matches(name, package))
            })
    })
}

fn python_setup_declares_package(content: &str, package: &str) -> bool {
    content.lines().any(|line| {
        let line = line.trim();
        !line.starts_with('#')
            && (line.contains("install_requires")
                || line.starts_with("django")
                || line.starts_with("fastapi")
                || line.starts_with("flask"))
            && contains_manifest_token(line, package)
    })
}

fn cargo_lock_declares_package(content: &str, package: &str) -> bool {
    let Ok(value) = toml::from_str::<toml::Value>(content) else {
        return false;
    };
    value
        .get("package")
        .and_then(toml::Value::as_array)
        .is_some_and(|packages| {
            packages.iter().any(|record| {
                record
                    .get("name")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|name| dependency_name_matches(name, package))
            })
        })
}

fn xml_elements<'a>(content: &'a str, element: &str) -> Vec<&'a str> {
    let lower = content.to_ascii_lowercase();
    let start_tag = format!("<{element}");
    let end_tag = format!("</{element}>");
    let mut blocks = Vec::new();
    let mut cursor = 0usize;
    while let Some(offset) = lower[cursor..].find(&start_tag) {
        let start = cursor + offset;
        let after_name = start + start_tag.len();
        if lower.as_bytes().get(after_name).is_some_and(|character| {
            !character.is_ascii_whitespace() && *character != b'>' && *character != b'/'
        }) {
            cursor = after_name;
            continue;
        }
        let Some(open_end) = lower[after_name..].find('>') else {
            break;
        };
        let open_end = after_name + open_end + 1;
        if lower[start..open_end].ends_with("/>") {
            blocks.push(&content[start..open_end]);
            cursor = open_end;
            continue;
        }
        let Some(close_offset) = lower[open_end..].find(&end_tag) else {
            break;
        };
        let close_end = open_end + close_offset + end_tag.len();
        blocks.push(&content[start..close_end]);
        cursor = close_end;
    }
    blocks
}

fn xml_dependency_declares_marker(content: &str, marker: &str) -> bool {
    let content = strip_xml_comments(content);
    ["dependency", "parent"].iter().any(|element| {
        xml_elements(&content, element)
            .iter()
            .any(|block| contains_manifest_token(block, marker))
    })
}

fn gradle_declares_framework(content: &str, marker: &str) -> bool {
    content.lines().any(|raw_line| {
        let line = raw_line.trim();
        !line.is_empty()
            && !line.starts_with("//")
            && [
                "implementation",
                "api",
                "compileOnly",
                "runtimeOnly",
                "testImplementation",
                "annotationProcessor",
                "platform(",
            ]
            .iter()
            .any(|configuration| line.starts_with(configuration))
            && contains_manifest_token(line, marker)
    })
}

fn build_manifest_declares_marker(content: &str, marker: &str, directives: &[&str]) -> bool {
    content.lines().any(|raw_line| {
        let line = raw_line.trim();
        let lower_line = line.to_ascii_lowercase();
        !line.is_empty()
            && !line.starts_with('#')
            && directives
                .iter()
                .any(|directive| lower_line.starts_with(directive))
            && contains_manifest_token(line, marker)
    })
}

fn go_mod_declares_package(content: &str, package: &str) -> bool {
    let mut in_require_block = false;
    for raw_line in content.lines() {
        let line = raw_line
            .split_once("//")
            .map_or(raw_line, |(code, _)| code)
            .trim();
        if line == "require (" {
            in_require_block = true;
            continue;
        }
        if in_require_block && line == ")" {
            in_require_block = false;
            continue;
        }
        let dependency = if in_require_block {
            line.split_whitespace().next()
        } else {
            line.strip_prefix("require ")
                .and_then(|required| required.split_whitespace().next())
        };
        if dependency.is_some_and(|dependency| go_module_matches(dependency, package)) {
            return true;
        }
    }
    false
}

fn go_module_matches(module: &str, marker: &str) -> bool {
    let module = module.to_ascii_lowercase();
    let marker = marker.to_ascii_lowercase();
    module == marker
        || module.strip_prefix(&marker).is_some_and(|suffix| {
            suffix.strip_prefix("/v").is_some_and(|version| {
                !version.is_empty() && version.chars().all(|c| c.is_ascii_digit())
            })
        })
}

fn dotnet_manifest_declares_framework(content: &str, marker: &str) -> bool {
    let content = strip_xml_comments(content);
    if xml_elements(&content, "Project")
        .iter()
        .any(|tag| contains_manifest_token(tag, marker))
    {
        return true;
    }
    ["PackageReference", "PackageVersion", "package"]
        .iter()
        .any(|element| {
            xml_elements(&content, element)
                .iter()
                .any(|tag| contains_manifest_token(tag, marker))
        })
}

fn strip_xml_comments(source: &str) -> String {
    let mut cleaned = String::with_capacity(source.len());
    let mut cursor = 0usize;
    let mut in_comment = false;
    while cursor < source.len() {
        if in_comment {
            if let Some(end) = source[cursor..].find("-->") {
                cursor += end + 3;
                in_comment = false;
            } else {
                break;
            }
        } else if let Some(start) = source[cursor..].find("<!--") {
            let start = cursor + start;
            cleaned.push_str(&source[cursor..start]);
            cursor = start + 4;
            in_comment = true;
        } else {
            cleaned.push_str(&source[cursor..]);
            break;
        }
    }
    cleaned
}

fn contains_manifest_token(content: &str, marker: &str) -> bool {
    let content = content.to_ascii_lowercase();
    let marker = marker.to_ascii_lowercase();
    content.match_indices(&marker).any(|(start, _)| {
        let end = start + marker.len();
        let before_is_identifier = content[..start]
            .chars()
            .next_back()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
        let after_is_identifier = content[end..]
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
        !before_is_identifier && !after_is_identifier
    })
}

fn source_mentions_framework(language: &str, source: &str, marker: &str) -> bool {
    source_import_targets(language, source)
        .iter()
        .any(|target| import_target_matches(target, marker))
}

fn source_import_targets(language: &str, source: &str) -> Vec<String> {
    let source = strip_source_comments(language, source);
    let mut targets = Vec::new();
    let mut go_import_block = false;
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if language == "go" {
            if line.starts_with("import (") {
                go_import_block = true;
                continue;
            }
            if go_import_block && line == ")" {
                go_import_block = false;
                continue;
            }
            if line.starts_with("import ") || go_import_block {
                if let Some(target) = first_quoted_value(line) {
                    targets.push(target);
                }
            }
            continue;
        }
        let target = match language {
            "python" => python_import_targets(line),
            "cpp" => cpp_include_target(line),
            "java" => java_import_target(line),
            "rust" => rust_import_target(line),
            "typescript" => typescript_import_target(line),
            "csharp" => csharp_import_target(line),
            _ => None,
        };
        if let Some(target) = target {
            targets.extend(target);
        }
    }
    targets
}

fn strip_source_comments(language: &str, source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut characters = source.chars().peekable();
    let mut quote = None;
    let mut escaped = false;
    let mut python_triple_quote = None;
    let mut python_triple_escaped = false;
    let mut block_comment = false;
    while let Some(character) = characters.next() {
        if let Some(delimiter) = python_triple_quote {
            if python_triple_escaped {
                python_triple_escaped = false;
                output.push(if character == '\n' { '\n' } else { ' ' });
                continue;
            }
            if character == '\\' {
                python_triple_escaped = true;
                output.push(' ');
                continue;
            }
            if character == delimiter && characters.peek() == Some(&delimiter) {
                characters.next();
                if characters.peek() == Some(&delimiter) {
                    characters.next();
                    python_triple_quote = None;
                    output.push_str("   ");
                    continue;
                }
                output.push_str("  ");
                continue;
            }
            output.push(if character == '\n' { '\n' } else { ' ' });
            continue;
        }
        if block_comment {
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                block_comment = false;
            } else if character == '\n' {
                output.push('\n');
            }
            continue;
        }
        if let Some(delimiter) = quote {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if language == "python" && character == '#' {
            for comment_character in characters.by_ref() {
                if comment_character == '\n' {
                    output.push('\n');
                    break;
                }
            }
            continue;
        }
        if language == "python"
            && matches!(character, '\'' | '"')
            && characters.peek() == Some(&character)
        {
            characters.next();
            if characters.peek() == Some(&character) {
                characters.next();
                python_triple_quote = Some(character);
                output.push_str("   ");
                continue;
            }
            output.push(character);
            output.push(character);
            continue;
        }
        if language != "python" && character == '/' {
            match characters.peek() {
                Some('/') => {
                    characters.next();
                    for comment_character in characters.by_ref() {
                        if comment_character == '\n' {
                            output.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    characters.next();
                    block_comment = true;
                    continue;
                }
                _ => {}
            }
        }
        if matches!(character, '\'' | '"' | '`') {
            quote = Some(character);
        }
        output.push(character);
    }
    output
}

fn python_import_targets(line: &str) -> Option<Vec<String>> {
    if let Some(imported) = line.strip_prefix("from ") {
        let module = imported.split_once(" import ")?.0.trim();
        return Some(vec![module.to_string()]);
    }
    let imported = line.strip_prefix("import ")?;
    Some(
        imported
            .split(',')
            .filter_map(|item| {
                let module = item.split_once(" as ").map_or(item, |(module, _)| module);
                let module = module.trim();
                (!module.is_empty()).then(|| module.to_string())
            })
            .collect(),
    )
}

fn cpp_include_target(line: &str) -> Option<Vec<String>> {
    let include = line
        .strip_prefix("#include")
        .or_else(|| line.strip_prefix("# include"))?
        .trim();
    let target = include
        .strip_prefix('<')
        .and_then(|rest| rest.split_once('>').map(|(target, _)| target))
        .or_else(|| {
            include
                .strip_prefix('"')
                .and_then(|rest| rest.split_once('"').map(|(target, _)| target))
        })?;
    Some(vec![target.to_string()])
}

fn java_import_target(line: &str) -> Option<Vec<String>> {
    let imported = line.strip_prefix("import ")?.trim();
    let imported = imported.strip_prefix("static ").unwrap_or(imported);
    let imported = imported.trim_end_matches(';').trim();
    (!imported.is_empty()).then(|| vec![imported.to_string()])
}

fn rust_import_target(line: &str) -> Option<Vec<String>> {
    let imported = line
        .strip_prefix("use ")
        .or_else(|| line.strip_prefix("extern crate "))?
        .trim_end_matches(';')
        .trim();
    let imported = imported.split('{').next()?.trim_end_matches(':').trim();
    let imported = imported
        .split_once(" as ")
        .map_or(imported, |(path, _)| path);
    (!imported.is_empty()).then(|| vec![imported.to_string()])
}

fn typescript_import_target(line: &str) -> Option<Vec<String>> {
    if line.starts_with("import ") || line.starts_with("export ") {
        if let Some((_, target)) = line.rsplit_once(" from ") {
            return first_quoted_value(target).map(|target| vec![target]);
        }
        if line.starts_with("import ") {
            return first_quoted_value(line.strip_prefix("import ")?).map(|target| vec![target]);
        }
    }
    for call in ["require(", "require (", "import(", "import ("] {
        if let Some((_, target)) = line.split_once(call) {
            if let Some(target) = first_quoted_value(target) {
                return Some(vec![target]);
            }
        }
    }
    None
}

fn csharp_import_target(line: &str) -> Option<Vec<String>> {
    let imported = line
        .strip_prefix("using ")
        .or_else(|| line.strip_prefix("global using "))?
        .trim_end_matches(';')
        .trim();
    let imported = imported.strip_prefix("static ").unwrap_or(imported);
    let imported = imported
        .split_once('=')
        .map_or(imported, |(_, namespace)| namespace);
    (!imported.is_empty()).then(|| vec![imported.trim().to_string()])
}

fn first_quoted_value(value: &str) -> Option<String> {
    let mut characters = value.char_indices();
    let (start, quote) = characters.find(|(_, character)| matches!(character, '\'' | '"'))?;
    let value_start = start + quote.len_utf8();
    let value_end = value[value_start..].find(quote)? + value_start;
    Some(value[value_start..value_end].to_string())
}

fn import_target_matches(target: &str, marker: &str) -> bool {
    let target = target.to_ascii_lowercase();
    let marker = marker.to_ascii_lowercase();
    target == marker
        || target.strip_prefix(&marker).is_some_and(|suffix| {
            suffix
                .chars()
                .next()
                .is_some_and(|character| matches!(character, '.' | '/' | ':' | '+' | '_'))
        })
}

pub(crate) fn exclusion_rule(path: &Path) -> Option<(&'static str, &'static str)> {
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();
    match name.as_str() {
        ".git" => Some(("vcs_metadata", "Git object and metadata directory")),
        ".baron" => Some((
            "baron_runtime_state",
            "Baron-managed runtime and local state",
        )),
        ".codex" | ".claude" | ".agents" => {
            Some(("host_adapter_projection", "Host integration projection"))
        }
        ".superpowers" => Some(("workflow_state", "Generated workflow state and ledgers")),
        "vendor" | "vendors" | "third_party" | "third-party" | "node_modules" => Some((
            "vendored_dependency",
            "Vendored or installed dependency tree",
        )),
        "generated" | "generated-code" | "autogenerated" | "autogen" | "gen" => {
            Some(("generated_source", "Generated source directory"))
        }
        "target" | "build" | "dist" | "out" | "bin" | "obj" | "coverage" | ".next" | ".nuxt"
        | ".venv" | "venv" => Some(("build_output", "Build, packaging, or environment output")),
        "fixtures" | "__fixtures__" | "testdata" => {
            Some(("fixture_data", "Fixture or test-data directory"))
        }
        _ => None,
    }
}

fn add_evidence(draft: &mut LanguageDraft, path: &str, signal: &str, limits: ScanLimits) {
    if draft
        .evidence
        .contains(&(path.to_string(), signal.to_string()))
    {
        return;
    }
    if draft.evidence.len() < limits.max_evidence_per_language {
        draft
            .evidence
            .insert((path.to_string(), signal.to_string()));
    } else {
        draft
            .gaps
            .insert("language_evidence_limit_reached".to_string());
    }
}

fn add_unsupported_evidence(
    evidence: &mut Vec<SecurityAuditUnsupportedEvidence>,
    language: &str,
    path: &str,
    scan_truncated: &mut bool,
    coverage_gaps: &mut BTreeSet<String>,
) {
    if evidence.len() < MAX_UNSUPPORTED_EVIDENCE {
        evidence.push(SecurityAuditUnsupportedEvidence {
            language: language.to_string(),
            path: path.to_string(),
        });
    } else {
        *scan_truncated = true;
        coverage_gaps.insert("unsupported_language_evidence_truncated".to_string());
    }
}

fn add_stack_signal(
    signals: &mut Vec<SecurityAuditStackSignal>,
    signal: SecurityAuditStackSignal,
    scan_truncated: &mut bool,
    coverage_gaps: &mut BTreeSet<String>,
) {
    if signals.len() < MAX_STACK_SIGNALS {
        signals.push(signal);
    } else {
        *scan_truncated = true;
        coverage_gaps.insert("stack_signal_limit_reached".to_string());
    }
}

fn add_source_path(
    draft: &mut LanguageDraft,
    path: &str,
    limits: ScanLimits,
    scan_truncated: &mut bool,
) {
    if draft.source_paths.len() < limits.max_paths_per_language {
        draft.source_paths.insert(path.to_string());
    } else {
        *scan_truncated = true;
        draft
            .gaps
            .insert("language_source_path_limit_reached".to_string());
    }
}

fn path_is_in_scope(path: &str, requested_scope: &[String]) -> bool {
    if requested_scope.is_empty() || requested_scope.iter().any(|scope| scope == GENERIC_SCOPE) {
        return true;
    }
    requested_scope.iter().any(|scope| {
        if !safe_relative_text(scope) {
            return false;
        }
        let scope = scope.replace('\\', "/").trim_end_matches('/').to_string();
        path == scope
            || path.starts_with(&format!("{scope}/"))
            || scope.starts_with(&format!("{path}/"))
    })
}

fn safe_relative_text(path: &str) -> bool {
    if path.is_empty() || path.len() > 1024 || path.contains(':') || path.starts_with('\\') {
        return false;
    }
    let path = Path::new(path);
    !path.is_absolute()
        && path.components().all(|component| {
            !matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}

fn normalized_relative_path(path: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_str()?.to_string()),
            Component::CurDir => {}
            _ => return None,
        }
    }
    if parts.is_empty() {
        None
    } else {
        let path = parts.join("/");
        (path.len() <= MAX_REPOSITORY_PATH_BYTES).then_some(path)
    }
}

fn normalized_relative_path_allowing_parent(path: &Path) -> Option<String> {
    let mut parts = Vec::<String>::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_str()?.to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn display_path(path: &Path) -> (String, bool) {
    let path = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    if path.len() <= MAX_REPOSITORY_PATH_BYTES {
        return (path, false);
    }
    let mut end = MAX_REPOSITORY_PATH_BYTES;
    while !path.is_char_boundary(end) {
        end -= 1;
    }
    (format!("{}…", &path[..end]), true)
}

fn quoted_includes(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let rest = line.strip_prefix("#include")?.trim_start();
            let (opening, closing) = match rest.chars().next()? {
                '"' => ('"', '"'),
                '<' => ('<', '>'),
                _ => return None,
            };
            let rest = rest.strip_prefix(opening)?;
            let index = rest.find(closing)?;
            Some(rest[..index].to_string())
        })
        .collect()
}

fn resolve_header(
    repository_root: &Path,
    translation_unit: &str,
    include: &str,
    headers: &BTreeSet<String>,
) -> Option<String> {
    if include.is_empty() || include.contains(':') || Path::new(include).is_absolute() {
        return None;
    }
    let parent = Path::new(translation_unit)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let relative = parent.join(include);
    if let Some(relative) = normalized_relative_path_allowing_parent(&relative) {
        if headers.contains(&relative) {
            return Some(relative);
        }
    }
    let root_relative = Path::new(include);
    if let Some(relative) = normalized_relative_path_allowing_parent(root_relative) {
        if headers.contains(&relative) {
            return Some(relative);
        }
    }
    let filename = Path::new(include).file_name()?.to_str()?;
    let matches = headers
        .iter()
        .filter(|header| {
            Path::new(header)
                .file_name()
                .is_some_and(|name| name == filename)
        })
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        let candidate = repository_root.join(matches[0]);
        candidate
            .starts_with(repository_root)
            .then(|| matches[0].clone())
    } else {
        None
    }
}

fn cmake_language_for_header(header: &str, manifests: &[ManifestRecord]) -> Option<&'static str> {
    let manifest = manifests
        .iter()
        .filter(|manifest| manifest.name == "cmakelists.txt")
        .filter(|manifest| is_descendant_or_same(header, &manifest.path))
        .max_by_key(|manifest| Path::new(&manifest.path).components().count())?;
    let content = manifest.content.as_deref()?.to_ascii_lowercase();
    let languages = content
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .collect::<HashSet<_>>();
    match (
        languages.contains("cxx") || languages.contains("c++"),
        languages.contains("c"),
    ) {
        (true, false) => Some("cpp"),
        (false, true) => Some("c"),
        _ => None,
    }
}

fn cmake_path_for_header<'a>(header: &str, manifests: &'a [ManifestRecord]) -> Option<&'a str> {
    manifests
        .iter()
        .filter(|manifest| manifest.name == "cmakelists.txt")
        .filter(|manifest| is_descendant_or_same(header, &manifest.path))
        .max_by_key(|manifest| Path::new(&manifest.path).components().count())
        .map(|manifest| manifest.path.as_str())
}

fn is_descendant_or_same(path: &str, manifest_path: &str) -> bool {
    let directory = Path::new(manifest_path)
        .parent()
        .and_then(normalized_relative_path)
        .unwrap_or_default();
    directory.is_empty() || path == directory || path.starts_with(&format!("{directory}/"))
}

fn dedupe_stack_signals(
    mut signals: Vec<SecurityAuditStackSignal>,
) -> Vec<SecurityAuditStackSignal> {
    signals.sort_by(|left, right| {
        (&left.kind, &left.language, &left.name, &left.evidence_paths).cmp(&(
            &right.kind,
            &right.language,
            &right.name,
            &right.evidence_paths,
        ))
    });
    signals.dedup();
    signals
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_file(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn language<'a>(
        inventory: &'a SecurityAuditDiscovery,
        name: &str,
    ) -> &'a SecurityAuditLanguageCoverage {
        inventory
            .languages
            .iter()
            .find(|language| language.language == name)
            .unwrap_or_else(|| panic!("missing language coverage for {name}"))
    }

    #[test]
    fn mixed_repository_keeps_all_seven_languages_and_records_exclusions() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        for (path, contents) in [
            ("py/app.py", "def app(): pass\n"),
            ("cpp/CMakeLists.txt", "project(sample LANGUAGES CXX)\n"),
            ("cpp/app.cpp", "int main() {}\n"),
            (
                "java/pom.xml",
                "<project><artifactId>sample</artifactId></project>\n",
            ),
            ("java/App.java", "class App {}\n"),
            (
                "rust/Cargo.toml",
                "[package]\nname=\"sample\"\nversion=\"0.1.0\"\n",
            ),
            ("rust/src/lib.rs", "pub fn run() {}\n"),
            ("go/go.mod", "module example.com/sample\n"),
            ("go/main.go", "package main\n"),
            (
                "ts/package.json",
                "{\"devDependencies\":{\"typescript\":\"^5\"}}\n",
            ),
            ("ts/app.ts", "export const app = true;\n"),
            (
                "cs/App.csproj",
                "<Project Sdk=\"Microsoft.NET.Sdk.Web\" />\n",
            ),
            ("cs/Program.cs", "class Program {}\n"),
            ("generated/ignored.py", "def generated(): pass\n"),
            ("vendor/ignored.go", "package vendor\n"),
            ("tests/fixtures/ignored.java", "class Fixture {}\n"),
            ("target/debug/ignored.rs", "fn ignored() {}\n"),
        ] {
            write_file(root, path, contents);
        }

        let inventory = discover_security_audit_coverage(root, &[]);
        assert_eq!(inventory, discover_security_audit_coverage(root, &[]));
        let detected = inventory
            .languages
            .iter()
            .map(|language| language.language.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            detected,
            [
                "cpp",
                "csharp",
                "go",
                "java",
                "python",
                "rust",
                "typescript"
            ]
        );
        for excluded in ["generated", "vendor", "tests/fixtures", "target"] {
            assert!(inventory
                .scope_exclusions
                .iter()
                .any(|rule| rule.path == excluded));
        }
        for ignored in [
            "generated/ignored.py",
            "vendor/ignored.go",
            "tests/fixtures/ignored.java",
            "target/debug/ignored.rs",
        ] {
            assert!(inventory
                .languages
                .iter()
                .all(|language| { !language.source_paths.iter().any(|path| path == ignored) }));
        }
    }

    #[test]
    fn node_manifest_alone_is_low_confidence_and_keeps_a_gap() {
        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "package.json",
            "{\"name\":\"plain-node-app\"}\n",
        );

        let inventory = discover_security_audit_coverage(temp.path(), &[]);
        let typescript = language(&inventory, "typescript");

        assert_eq!(typescript.confidence, "low");
        assert!(typescript
            .coverage_gaps
            .contains(&"node_manifest_without_typescript_evidence".to_string()));
        assert!(typescript
            .evidence
            .iter()
            .any(|evidence| evidence.signal == "node_ecosystem_manifest_only"));
    }

    #[test]
    fn in_scope_language_exposes_non_exhaustive_framework_catalog_gap() {
        let temp = tempdir().unwrap();
        write_file(temp.path(), "src/app.py", "def app(): pass\n");

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert!(language(&inventory, "python")
            .coverage_gaps
            .iter()
            .any(|gap| {
                gap == "framework_detection_catalog_non_exhaustive_requires_validation"
            }));
    }

    #[test]
    fn node_manifest_names_do_not_masquerade_as_typescript_or_framework_dependencies() {
        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "package.json",
            "{\"name\":\"typescript-express-app\",\"description\":\"Uses express and typescript\"}\n",
        );

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert_eq!(language(&inventory, "typescript").confidence, "low");
        assert!(inventory
            .stack_signals
            .iter()
            .all(|signal| { !(signal.language == "typescript" && signal.kind == "framework") }));
    }

    #[test]
    fn node_dependency_keys_confirm_typescript_and_frameworks() {
        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "package.json",
            "{\"name\":\"plain-app\",\"dependencies\":{\"typescript\":\"^5\",\"express\":\"^4\"}}\n",
        );

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert_eq!(language(&inventory, "typescript").confidence, "medium");
        assert!(inventory
            .stack_signals
            .iter()
            .any(|signal| { signal.language == "typescript" && signal.name == "express" }));
    }

    #[test]
    fn npm_lockfile_package_paths_confirm_exact_framework_and_typescript_evidence() {
        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "package-lock.json",
            "{\"packages\":{\"\":{\"name\":\"typescript-app-express\"},\"node_modules/typescript\":{},\"node_modules/express\":{}}}\n",
        );

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert_eq!(language(&inventory, "typescript").confidence, "medium");
        assert!(inventory
            .stack_signals
            .iter()
            .any(|signal| { signal.language == "typescript" && signal.name == "express" }));
    }

    #[test]
    fn shared_header_records_c_and_cpp_consumers_once_without_following_ignored_files() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        write_file(root, "include/shared.h", "int load_value(void);\n");
        write_file(root, "cpp/reader.cpp", "#include \"../include/shared.h\"\n");
        write_file(root, "c/reader.c", "#include \"../include/shared.h\"\n");

        let inventory = discover_security_audit_coverage(root, &[]);
        let cpp = language(&inventory, "cpp");
        let header = cpp
            .headers
            .iter()
            .find(|header| header.path == "include/shared.h")
            .unwrap();

        assert_eq!(header.interpretations, ["c", "cpp"]);
        assert!(header.ambiguous);
        assert_eq!(header.consuming_translation_units.len(), 2);
        assert_eq!(
            cpp.source_paths
                .iter()
                .filter(|path| *path == "include/shared.h")
                .count(),
            1
        );
        assert!(inventory
            .unsupported_evidence
            .iter()
            .any(|evidence| evidence.language == "c" && evidence.path == "c/reader.c"));
    }

    #[test]
    fn oversized_manifest_is_reported_without_reading_its_contents() {
        let temp = tempdir().unwrap();
        write_file(temp.path(), "requirements.txt", &"a".repeat(64));
        write_file(temp.path(), "src/app.py", "def app(): pass\n");
        let limits = ScanLimits {
            max_entries: 64,
            max_paths_per_language: 1,
            max_evidence_per_language: 2,
            max_scope_exclusions: 2,
            max_manifest_bytes: 16,
            max_total_manifest_bytes: 16,
            max_source_evidence_bytes: 32,
        };

        let inventory = discover_with_limits(temp.path(), &[], limits);

        assert!(inventory
            .coverage_gaps
            .iter()
            .any(|gap| gap == "manifest_read_limit_reached:requirements.txt"));
    }

    #[test]
    fn invalid_utf8_manifest_is_reported_as_unreadable_and_not_used_for_framework_guessing() {
        let temp = tempdir().unwrap();
        let manifest = temp.path().join("package.json");
        fs::write(&manifest, [0xff, 0xfe, 0xfd]).unwrap();

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert!(inventory
            .coverage_gaps
            .contains(&"manifest_unreadable:package.json".to_string()));
        assert!(inventory
            .stack_signals
            .iter()
            .all(|signal| signal.kind != "framework"));
        assert_eq!(language(&inventory, "typescript").confidence, "low");
    }

    #[test]
    fn build_output_compilation_database_classifies_a_header_without_scanning_the_build_tree() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        write_file(root, "include/api.h", "int call_api(void);\n");
        write_file(
            root,
            "build/compile_commands.json",
            &serde_json::json!([{
                "directory": root.join("build").display().to_string(),
                "file": "../include/api.h",
                "command": "clang++ -x c++ -c ../include/api.h"
            }])
            .to_string(),
        );
        write_file(
            root,
            "build/generated.cpp",
            "int generated() {{ return 0; }}\n",
        );

        let inventory = discover_security_audit_coverage(root, &[]);
        let cpp = language(&inventory, "cpp");
        let header = cpp
            .headers
            .iter()
            .find(|header| header.path == "include/api.h")
            .unwrap();

        assert_eq!(header.interpretations, ["cpp"]);
        assert!(!header.ambiguous);
        assert!(header
            .consuming_translation_units
            .iter()
            .any(|path| path == "build/compile_commands.json"));
        assert!(inventory
            .scope_exclusions
            .iter()
            .any(|rule| rule.path == "build"));
        assert!(cpp
            .source_paths
            .iter()
            .all(|path| path != "build/generated.cpp"));
        assert!(inventory.stack_signals.iter().any(|signal| {
            signal.name == "compilation-database"
                && signal.evidence_paths == ["build/compile_commands.json"]
        }));
    }

    #[test]
    fn source_imports_can_confirm_framework_but_similar_package_name_cannot() {
        let temp = tempdir().unwrap();
        write_file(temp.path(), "go.mod", "module example.com/service\n");
        write_file(
            temp.path(),
            "main.go",
            "package main\nimport \"net/http\"\nfunc main() {}\n",
        );
        write_file(
            temp.path(),
            "other.go",
            "package main\n// import \"net/http\"\nconst next = \"net/http\"\n",
        );
        write_file(
            temp.path(),
            "web/package.json",
            "{\"name\":\"expressive-client\"}\n",
        );

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert!(inventory.stack_signals.iter().any(|signal| {
            signal.language == "go"
                && signal.name == "net-http"
                && signal.evidence_paths == ["main.go"]
        }));
        assert!(inventory
            .stack_signals
            .iter()
            .all(|signal| signal.name != "express"));
    }

    #[test]
    fn framework_import_detection_ignores_inline_comments_and_unrelated_import_names() {
        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "app.py",
            "from app.helpers import fastapi  # framework is not used here\n\"\"\"Example:\nfrom fastapi import FastAPI\n\"\"\"\n",
        );
        write_file(
            temp.path(),
            "client.ts",
            "import { express } from \"./local-client\"; // framework is not used here\n",
        );

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert!(inventory.stack_signals.iter().all(|signal| {
            !matches!(
                (signal.language.as_str(), signal.name.as_str()),
                ("python", "fastapi") | ("typescript", "express")
            )
        }));
    }

    #[test]
    fn source_import_detection_recognizes_supported_frameworks_in_all_languages() {
        let temp = tempdir().unwrap();
        for (path, source) in [
            ("main.py", "from fastapi import FastAPI\n"),
            ("main.cpp", "#include <drogon/drogon.h>\n"),
            (
                "Main.java",
                "import org.springframework.web.bind.annotation.RestController;\n",
            ),
            ("main.rs", "use axum::{Router, routing::get};\n"),
            ("main.go", "import \"github.com/gin-gonic/gin\"\n"),
            ("main.ts", "import express from \"express\";\n"),
            ("Main.cs", "using Microsoft.AspNetCore.Builder;\n"),
        ] {
            write_file(temp.path(), path, source);
        }

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        for (language, framework) in [
            ("python", "fastapi"),
            ("cpp", "drogon"),
            ("java", "spring"),
            ("rust", "axum"),
            ("go", "gin"),
            ("typescript", "express"),
            ("csharp", "aspnet-core"),
        ] {
            assert!(
                inventory
                    .stack_signals
                    .iter()
                    .any(|signal| { signal.language == language && signal.name == framework }),
                "missing {language}/{framework} source import signal"
            );
        }
    }

    #[test]
    fn framework_names_in_xml_comments_are_not_repository_evidence() {
        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "pom.xml",
            "<project>\n  <!-- spring-boot is only mentioned in a comment -->\n</project>\n",
        );

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert!(inventory
            .stack_signals
            .iter()
            .all(|signal| signal.kind != "framework"));
        assert!(inventory
            .stack_signals
            .iter()
            .any(|signal| signal.kind == "build_system" && signal.name == "maven"));
    }

    #[test]
    fn project_names_in_framework_manifests_do_not_confirm_frameworks() {
        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "python/pyproject.toml",
            "[project]\nname = \"fastapi-service\"\ndependencies = []\n",
        );
        write_file(
            temp.path(),
            "rust/Cargo.toml",
            "[package]\nname = \"axum-service\"\nversion = \"0.1.0\"\n",
        );
        write_file(
            temp.path(),
            "java/pom.xml",
            "<project><groupId>org.example</groupId><artifactId>spring-web-demo</artifactId></project>\n",
        );

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert!(inventory.stack_signals.iter().all(|signal| {
            !matches!(
                (signal.language.as_str(), signal.name.as_str()),
                ("python", "fastapi") | ("rust", "axum") | ("java", "spring")
            )
        }));
    }

    #[test]
    fn supported_dependency_declarations_confirm_frameworks() {
        let temp = tempdir().unwrap();
        write_file(
            temp.path(),
            "python/pyproject.toml",
            "[project]\nname = \"api\"\ndependencies = [\"fastapi>=0.1\"]\n",
        );
        write_file(
            temp.path(),
            "rust/Cargo.toml",
            "[package]\nname = \"api\"\nversion = \"0.1.0\"\n[dependencies]\naxum = \"0.7\"\n",
        );
        write_file(
            temp.path(),
            "java/pom.xml",
            "<project><dependencies><dependency><groupId>org.springframework</groupId><artifactId>spring-web</artifactId></dependency></dependencies></project>\n",
        );

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        for (language, framework) in [("python", "fastapi"), ("rust", "axum"), ("java", "spring")] {
            assert!(
                inventory
                    .stack_signals
                    .iter()
                    .any(|signal| { signal.language == language && signal.name == framework }),
                "missing {language} framework signal {framework}"
            );
        }
    }

    #[test]
    fn scan_entry_limit_is_reported_and_marks_the_inventory_truncated() {
        let temp = tempdir().unwrap();
        for index in 0..8 {
            write_file(temp.path(), &format!("src/{index}.py"), "def app(): pass\n");
        }
        let limits = ScanLimits {
            max_entries: 3,
            max_paths_per_language: 1,
            max_evidence_per_language: 2,
            max_scope_exclusions: 2,
            max_manifest_bytes: 16,
            max_total_manifest_bytes: 16,
            max_source_evidence_bytes: 32,
        };

        let inventory = discover_with_limits(temp.path(), &[], limits);

        assert!(inventory.scan_truncated);
        assert!(inventory
            .coverage_gaps
            .contains(&"repository_scan_entry_limit_reached".to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn directory_symlinks_are_never_followed() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().unwrap();
        write_file(temp.path(), "real/app.py", "def app(): pass\n");
        symlink(temp.path().join("real"), temp.path().join("linked")).unwrap();

        let inventory = discover_security_audit_coverage(temp.path(), &[]);

        assert!(language(&inventory, "python")
            .source_paths
            .iter()
            .all(|path| !path.starts_with("linked/")));
        assert!(inventory
            .coverage_gaps
            .contains(&"symlink_source_not_followed".to_string()));
    }
}
