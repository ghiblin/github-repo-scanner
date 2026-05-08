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
            id: RuleId::parse("NODE-009").unwrap(),
            name: "Prototype pollution".to_owned(),
            description: String::new(),
            severity: Severity::High,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r"Object\.prototype\.|__proto__\s*\[".to_owned(),
            },
        }],
    })
}

#[test]
fn detects_object_prototype_mutation() {
    let src = "Object.prototype.toString = function() { return 'evil'; };";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_proto_bracket_access() {
    let src = r#"obj.__proto__["admin"] = true;"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_object_usage() {
    let src = "const copy = Object.assign({}, source);";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert!(findings.is_empty());
}
