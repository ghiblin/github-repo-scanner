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

fn node_002_ruleset() -> Arc<RuleSet> {
    Arc::new(RuleSet {
        rules: vec![Rule {
            id: RuleId::parse("NODE-002").unwrap(),
            name: "Obfuscated JavaScript".to_owned(),
            description: String::new(),
            severity: Severity::High,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r"eval\s*\(\s*atob|\\x[0-9a-fA-F]{2}|_0x[0-9a-fA-F]+".to_owned(),
            },
        }],
    })
}

#[test]
fn detects_eval_atob_pattern() {
    let snapshot = make_snapshot("src/index.js", "eval(atob('aGVsbG8='))");
    let findings = NodeJsAnalyzer.analyze(&snapshot, &node_002_ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_hex_encoded_string() {
    let snapshot = make_snapshot("src/index.js", r"var x = '\x48\x65\x6c\x6c\x6f'");
    let findings = NodeJsAnalyzer.analyze(&snapshot, &node_002_ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_mangled_variable_names() {
    let snapshot = make_snapshot("src/index.js", "var _0x1a2b = function() {}");
    let findings = NodeJsAnalyzer.analyze(&snapshot, &node_002_ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_clean_javascript() {
    let snapshot = make_snapshot("src/index.js", "console.log('hello world')");
    let findings = NodeJsAnalyzer.analyze(&snapshot, &node_002_ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_non_js_files() {
    let snapshot = make_snapshot("README.md", "eval(atob('aGVsbG8='))");
    let findings = NodeJsAnalyzer.analyze(&snapshot, &node_002_ruleset());
    assert!(findings.is_empty());
}
