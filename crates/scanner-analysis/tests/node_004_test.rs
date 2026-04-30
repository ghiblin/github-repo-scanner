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
            id: RuleId::parse("NODE-004").unwrap(),
            name: "Shell execution in source".to_owned(),
            description: String::new(),
            severity: Severity::High,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r#"require\(['"]child_process['"]\)|execSync|spawnSync"#.to_owned(),
            },
        }],
    })
}

#[test]
fn detects_child_process_require() {
    let findings = NodeJsAnalyzer.analyze(
        &snapshot("const cp = require('child_process');"),
        &ruleset(),
    );
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_execsync() {
    let findings = NodeJsAnalyzer.analyze(&snapshot("execSync('ls -la')"), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_clean_code() {
    let findings = NodeJsAnalyzer.analyze(&snapshot("console.log('hello')"), &ruleset());
    assert!(findings.is_empty());
}
