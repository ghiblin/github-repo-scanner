use scanner_analysis::{vscode::VsCodeAnalyzer, Analyzer};
use scanner_repository::{FileContent, RepoFile, RepoSnapshot};
use scanner_rules::{Language, Pattern, Rule, RuleId, RuleSet, Severity};
use std::{path::PathBuf, sync::Arc};

fn snapshot(filename: &str, content: &str) -> RepoSnapshot {
    RepoSnapshot {
        owner: "t".to_owned(),
        name: "r".to_owned(),
        files: vec![RepoFile {
            path: PathBuf::from(filename),
            content: FileContent::Text(content.to_owned()),
            size_bytes: content.len() as u64,
        }],
    }
}

fn ruleset() -> Arc<RuleSet> {
    Arc::new(RuleSet {
        rules: vec![Rule {
            id: RuleId::parse("VSCODE-004").unwrap(),
            name: "Suspicious extension recommendation".to_owned(),
            description: String::new(),
            severity: Severity::Medium,
            language: Language::VsCode,
            pattern: Pattern::ExtensionIdCheck,
        }],
    })
}

#[test]
fn detects_extension_id_without_publisher_dot() {
    let content = r#"{ "recommendations": ["malicious-ext"] }"#;
    let findings =
        VsCodeAnalyzer.analyze(&snapshot(".vscode/extensions.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("malicious-ext"));
}

#[test]
fn flags_each_invalid_id_separately() {
    let content = r#"{ "recommendations": ["hacktools", "ms-python.python", "noPublisher"] }"#;
    let findings =
        VsCodeAnalyzer.analyze(&snapshot(".vscode/extensions.json", content), &ruleset());
    assert_eq!(findings.len(), 2);
}

#[test]
fn ignores_valid_publisher_dot_name_ids() {
    let content = r#"{ "recommendations": ["ms-python.python", "vscodevim.vim", "rust-lang.rust-analyzer"] }"#;
    let findings =
        VsCodeAnalyzer.analyze(&snapshot(".vscode/extensions.json", content), &ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_non_extensions_json_file() {
    let content = r#"{ "recommendations": ["hacktools"] }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert!(findings.is_empty());
}
