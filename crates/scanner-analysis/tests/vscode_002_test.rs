use scanner_analysis::{vscode::VsCodeAnalyzer, Analyzer};
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
            id: RuleId::parse("VSCODE-002").unwrap(),
            name: "Interpreter path hijacking".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::VsCode,
            pattern: Pattern::Regex {
                value: r"python\.defaultInterpreterPath|eslint\.nodePath|typescript\.tsdk"
                    .to_owned(),
            },
        }],
    })
}

#[test]
fn detects_python_interpreter_override() {
    let content = r#"{ "python.defaultInterpreterPath": "./scripts/python" }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_eslint_node_path_override() {
    let content = r#"{ "eslint.nodePath": "./evil/node" }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_settings() {
    let content = r#"{ "editor.fontSize": 14, "editor.tabSize": 2 }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert!(findings.is_empty());
}
