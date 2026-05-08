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
            id: RuleId::parse("NODE-008").unwrap(),
            name: "Dynamic eval alternative".to_owned(),
            description: String::new(),
            severity: Severity::High,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r#"new\s+Function\s*\(|setTimeout\s*\(\s*['"]|setInterval\s*\(\s*['"]"#.to_owned(),
            },
        }],
    })
}

#[test]
fn detects_new_function_constructor() {
    let src = "const fn = new Function('return process.env.SECRET')();";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_settimeout_with_string() {
    let src = r#"setTimeout("malicious()", 0);"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_settimeout_with_callback() {
    let src = "setTimeout(callback, 1000);";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert!(findings.is_empty());
}
