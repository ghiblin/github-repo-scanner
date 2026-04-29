use scanner_analysis::{nodejs::NodeJsAnalyzer, Analyzer};
use scanner_repository::{FileContent, RepoFile, RepoSnapshot};
use scanner_rules::{Language, Pattern, Rule, RuleId, RuleSet, Severity};
use std::{path::PathBuf, sync::Arc};

fn snapshot(content: &str) -> RepoSnapshot {
    RepoSnapshot {
        owner: "t".to_owned(),
        name: "r".to_owned(),
        files: vec![RepoFile {
            path: PathBuf::from("config.js"),
            content: FileContent::Text(content.to_owned()),
            size_bytes: content.len() as u64,
        }],
    }
}

fn ruleset() -> Arc<RuleSet> {
    Arc::new(RuleSet {
        rules: vec![Rule {
            id: RuleId::parse("rul_018f1234-abcd-7000-8001-000000000005").unwrap(),
            name: "Hardcoded secret or token".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r#"(?i)(api_key|secret|token|private_key)\s*=\s*['"][A-Za-z0-9_\-]{16,}"#
                    .to_owned(),
            },
        }],
    })
}

#[test]
fn detects_hardcoded_api_key() {
    let findings = NodeJsAnalyzer.analyze(
        &snapshot(r#"const api_key = 'sk-abcdefghijklmnopqrstuvwx';"#),
        &ruleset(),
    );
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_hardcoded_token() {
    let findings = NodeJsAnalyzer.analyze(
        &snapshot(r#"const token = 'ghp_abcdefghijklmnopqrstuvwxyz12'"#),
        &ruleset(),
    );
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_short_values() {
    let findings = NodeJsAnalyzer.analyze(&snapshot(r#"const token = 'short'"#), &ruleset());
    assert!(findings.is_empty());
}
