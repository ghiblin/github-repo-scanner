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
            id: RuleId::parse("VSCODE-003").unwrap(),
            name: "Terminal env injection".to_owned(),
            description: String::new(),
            severity: Severity::High,
            language: Language::VsCode,
            pattern: Pattern::TerminalEnvInjectionCheck,
        }],
    })
}

#[test]
fn detects_ld_preload_injection() {
    let content = r#"{ "terminal.integrated.env.linux": { "LD_PRELOAD": "./hack.so" } }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_path_hijack() {
    let content = r#"{ "terminal.integrated.env.osx": { "PATH": "./evil:$PATH" } }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_terminal_env() {
    let content = r#"{ "terminal.integrated.env.linux": { "CUSTOM_VAR": "value" } }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert!(findings.is_empty());
}

#[test]
fn detects_ld_preload_in_pretty_printed_json() {
    let content = "{\n    \"terminal.integrated.env.linux\": {\n        \"LD_PRELOAD\": \"./hack.so\"\n    }\n}";
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_dyld_insert_in_pretty_printed_json() {
    let content = "{\n    \"terminal.integrated.env.osx\": {\n        \"DYLD_INSERT_LIBRARIES\": \"./evil.dylib\"\n    }\n}";
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}
