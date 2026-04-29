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
            id: RuleId::parse("rul_018f1234-abcd-7000-8001-000000000006").unwrap(),
            name: "Typosquatting risk".to_owned(),
            description: String::new(),
            severity: Severity::Medium,
            language: Language::NodeJs,
            pattern: Pattern::TypoSquat {
                known_packages: vec!["express".to_owned(), "lodash".to_owned()],
            },
        }],
    })
}

#[test]
fn detects_express_typo_in_dependencies() {
    let pkg = r#"{"dependencies": {"expres": "^4.0.0"}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("expres"));
}

#[test]
fn detects_lodash_typo_in_dev_dependencies() {
    let pkg = r#"{"devDependencies": {"lodahs": "^4.0.0"}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_exact_package_names() {
    let pkg = r#"{"dependencies": {"express": "^4.0.0", "lodash": "^4.0.0"}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_clearly_unrelated_names() {
    let pkg = r#"{"dependencies": {"completely-different-name": "^1.0.0"}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert!(findings.is_empty());
}
