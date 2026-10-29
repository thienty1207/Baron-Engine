use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const LANGUAGES: [&str; 7] = [
    "python",
    "cpp",
    "java",
    "rust",
    "go",
    "typescript",
    "csharp",
];

const FIXTURE_FAMILIES: [&str; 7] = [
    "input-injection-output",
    "authentication-authorization-ownership",
    "secrets-session-cryptography",
    "files-paths-upload-outbound-network",
    "serialization-process-execution",
    "concurrency-resource-exhaustion",
    "dependency-build-deployment",
];

const SHARED_ATTACK_CLASSES: [&str; 8] = [
    "trust_boundary_and_input_validation",
    "authentication_authorization_and_ownership",
    "secrets_sessions_and_cryptography",
    "query_serialization_and_parsing",
    "files_uploads_and_outbound_network",
    "process_execution_and_information_disclosure",
    "concurrency_resources_and_supply_chain",
    "deployment_and_security_configuration",
];

fn expected_frameworks(language: &str) -> &'static [&'static str] {
    match language {
        "python" => &["django", "fastapi", "flask"],
        "cpp" => &["drogon"],
        "java" => &["spring", "jakarta"],
        "rust" => &["axum", "sqlx"],
        "go" => &["net-http", "fiber"],
        "typescript" => &["express", "nestjs", "nextjs"],
        "csharp" => &["aspnet-core"],
        _ => &[],
    }
}

fn identifier_key(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn assets_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/core/skills/security-audit")
}

#[test]
fn local_security_audit_corpus_covers_all_seven_languages_and_fixture_families() {
    let assets = assets_root();
    let taxonomy = fs::read_to_string(assets.join("rules/shared/ATTACK-TAXONOMY.md"))
        .expect("shared taxonomy must be packaged as a local Baron asset");
    for attack_class in SHARED_ATTACK_CLASSES {
        assert!(
            taxonomy.contains(attack_class),
            "missing taxonomy class {attack_class}"
        );
    }
    for severity in ["Critical", "High", "Medium", "Low", "Info"] {
        assert!(
            taxonomy.contains(severity),
            "missing canonical severity {severity}"
        );
    }

    for language in LANGUAGES {
        let rule_pack = fs::read_to_string(assets.join(format!("rules/{language}/RULES.md")))
            .unwrap_or_else(|_| panic!("missing {language} rule pack"));
        for field in [
            "Risky source or operation",
            "Protected boundary",
            "Safe pattern",
            "Common false positives",
            "Confirmation evidence",
        ] {
            assert!(
                rule_pack.contains(field),
                "{language} pack misses field {field}"
            );
        }
        for family in FIXTURE_FAMILIES {
            assert!(
                rule_pack.contains(family),
                "{language} pack misses {family}"
            );
        }
        for attack_class in SHARED_ATTACK_CLASSES {
            assert!(
                rule_pack.contains(attack_class),
                "{language} pack misses {attack_class}"
            );
        }

        let framework_pack =
            fs::read_to_string(assets.join(format!("rules/{language}/FRAMEWORKS.md")))
                .unwrap_or_else(|_| panic!("missing {language} framework supplement"));
        assert!(framework_pack.contains("evidence"));

        let fixture_path = assets.join(format!("fixtures/{language}.json"));
        let fixture_text = fs::read_to_string(&fixture_path)
            .unwrap_or_else(|_| panic!("missing {language} static fixtures"));
        let fixtures: Value = serde_json::from_str(&fixture_text)
            .unwrap_or_else(|_| panic!("invalid JSON in {}", fixture_path.display()));
        let cases = fixtures["cases"].as_array().expect("cases array required");
        let framework_cases = fixtures["framework_cases"]
            .as_array()
            .expect("framework cases array required");
        let expected_special = match language {
            "cpp" => Some("memory-lifetime-bounds"),
            "rust" => Some("unsafe-ffi-boundary"),
            _ => None,
        };
        assert_eq!(
            cases.len(),
            FIXTURE_FAMILIES.len() + usize::from(expected_special.is_some())
        );
        let mut seen = Vec::new();
        for case in cases {
            let family = case["family"].as_str().expect("family id required");
            seen.push(family);
            let overlap = case["overlap_token"]
                .as_str()
                .expect("positive/safe overlap token required")
                .to_ascii_lowercase();
            let vulnerable = case["vulnerable"]["source"]
                .as_str()
                .expect("positive source required")
                .to_ascii_lowercase();
            let safe = case["safe"]["source"]
                .as_str()
                .expect("safe source required")
                .to_ascii_lowercase();
            let safe_reason = case["safe"]["why_safe"]
                .as_str()
                .expect("safe example needs a rationale");
            assert_eq!(
                case["vulnerable"]["expected_outcome"].as_str(),
                Some("candidate_requires_source_validation"),
                "{language}/{family} positive needs an explicit candidate outcome"
            );
            assert_eq!(
                case["safe"]["expected_outcome"].as_str(),
                Some("must_not_be_confirmed_from_keyword_alone"),
                "{language}/{family} safe case needs an explicit bounded outcome"
            );
            assert!(!overlap.is_empty());
            assert!(
                vulnerable.contains(&overlap),
                "{language}/{family} positive misses overlap token"
            );
            assert!(
                safe.contains(&overlap),
                "{language}/{family} safe example misses overlap token"
            );
            assert!(
                safe_reason.len() >= 20,
                "{language}/{family} needs an evidence-based rationale"
            );
            assert_ne!(vulnerable, safe, "{language}/{family} examples must differ");
        }
        for family in FIXTURE_FAMILIES {
            assert!(
                seen.contains(&family),
                "missing {language}/{family} fixture pair"
            );
        }
        if let Some(special) = expected_special {
            assert!(seen.contains(&special), "missing {language}/{special} pair");
        }
        let framework_names = framework_cases
            .iter()
            .map(|case| case["framework"].as_str().expect("framework id required"))
            .collect::<Vec<_>>();
        assert_eq!(framework_names, expected_frameworks(language).to_vec());
        for case in framework_cases {
            let framework = case["framework"].as_str().unwrap();
            assert!(
                identifier_key(&framework_pack).contains(&identifier_key(framework)),
                "{language} framework pack misses {framework}"
            );
            assert!(!case["evidence_signal"]
                .as_str()
                .unwrap_or_default()
                .is_empty());
            let overlap = case["overlap_token"].as_str().unwrap().to_ascii_lowercase();
            let positive = case["vulnerable"]["source"]
                .as_str()
                .unwrap()
                .to_ascii_lowercase();
            let safe = case["safe"]["source"]
                .as_str()
                .unwrap()
                .to_ascii_lowercase();
            assert!(
                positive.contains(&overlap),
                "{language}/{framework} positive misses its marker"
            );
            assert!(
                safe.contains(&overlap),
                "{language}/{framework} safe example misses its marker"
            );
            assert!(case["safe"]["why_safe"].as_str().unwrap().len() >= 20);
            assert_eq!(
                case["vulnerable"]["expected_outcome"].as_str(),
                Some("candidate_requires_source_validation"),
                "{language}/{framework} positive needs an explicit candidate outcome"
            );
            assert_eq!(
                case["safe"]["expected_outcome"].as_str(),
                Some("must_not_be_confirmed_from_keyword_alone"),
                "{language}/{framework} safe case needs an explicit bounded outcome"
            );
        }
    }
}
