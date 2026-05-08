use scanner_analysis::{nodejs::NodeJsAnalyzer, Analyzer};
use scanner_repository::{FileContent, RepoFile, RepoSnapshot};
use scanner_rules::{Language, Pattern, Rule, RuleId, RuleSet, Severity};
use std::{path::PathBuf, sync::Arc};

fn snapshot(content: &str) -> RepoSnapshot {
    RepoSnapshot {
        owner: "t".to_owned(),
        name: "r".to_owned(),
        files: vec![RepoFile {
            path: PathBuf::from("index.js"),
            content: FileContent::Text(content.to_owned()),
            size_bytes: content.len() as u64,
        }],
    }
}

fn ruleset() -> Arc<RuleSet> {
    Arc::new(RuleSet {
        rules: vec![Rule {
            id: RuleId::parse("NODE-003").unwrap(),
            name: "Suspicious outbound network call".to_owned(),
            description: String::new(),
            severity: Severity::High,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r"https?://(?:10|172|192)\.[0-9]{1,3}".to_owned(),
            },
        }],
    })
}

#[test]
fn detects_hardcoded_ip_url() {
    let findings =
        NodeJsAnalyzer.analyze(&snapshot("fetch('http://192.168.1.1/data')"), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_localhost() {
    let findings = NodeJsAnalyzer.analyze(&snapshot("fetch('http://localhost/api')"), &ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_127_loopback() {
    let findings = NodeJsAnalyzer.analyze(&snapshot("fetch('http://127.0.0.1/api')"), &ruleset());
    assert!(findings.is_empty());
}
