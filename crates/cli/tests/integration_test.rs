use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_repo-scanner"))
}

#[test]
fn shows_help_without_args() {
    let output = bin().arg("--help").output().expect("binary must exist");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GITHUB_URL"));
}

use scanner_analysis::{Analyzer, NodeJsAnalyzer, VsCodeAnalyzer, Verdict};
use scanner_repository::{FileContent, RepoFile, RepoSnapshot};
use scanner_rules::loader::load;
use std::{path::PathBuf, sync::Arc};

fn load_fixture(dir: &str) -> RepoSnapshot {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/repos")
        .join(dir);
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(&base).into_iter().flatten() {
        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(&base).unwrap().to_owned();
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            files.push(RepoFile {
                path: rel,
                content: FileContent::Text(content.clone()),
                size_bytes: content.len() as u64,
            });
        }
    }
    RepoSnapshot {
        owner: "test".to_owned(),
        name: dir.to_owned(),
        files,
    }
}

fn default_ruleset() -> Arc<scanner_rules::RuleSet> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules/nodejs.toml");
    Arc::new(load(&path).expect("default ruleset must load"))
}

fn vscode_ruleset() -> Arc<scanner_rules::RuleSet> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules/vscode.toml");
    Arc::new(load(&path).expect("vscode ruleset must load"))
}

#[test]
fn clean_repo_produces_safe_verdict() {
    let snapshot = load_fixture("clean-nodejs");
    let findings = NodeJsAnalyzer.analyze(&snapshot, &default_ruleset());
    assert_eq!(Verdict::from_findings(&findings), Verdict::Safe);
}

#[test]
fn malicious_repo_produces_dangerous_verdict() {
    let snapshot = load_fixture("malicious-nodejs");
    let findings = NodeJsAnalyzer.analyze(&snapshot, &default_ruleset());
    assert_eq!(Verdict::from_findings(&findings), Verdict::Dangerous);
    assert!(
        findings.len() >= 2,
        "expected at least 2 findings, got {}",
        findings.len()
    );
}

#[test]
fn malicious_vscode_repo_produces_findings() {
    let snapshot = load_fixture("malicious-vscode");
    let findings = VsCodeAnalyzer.analyze(&snapshot, &vscode_ruleset());
    assert!(
        !findings.is_empty(),
        "expected VSCODE findings from malicious-vscode fixture"
    );
}
