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

use scanner_analysis::{Analyzer, NodeJsAnalyzer, Verdict, VsCodeAnalyzer};
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

#[test]
fn exits_with_error_when_no_token_provided() {
    let output = bin()
        .arg("https://github.com/owner/repo")
        .env_remove("GITHUB_TOKEN")
        .output()
        .expect("binary must exist");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("GitHub token is required"),
        "expected token-required error, got: {stderr}"
    );
}

#[test]
fn accepts_token_from_env_var() {
    let output = bin()
        .arg("https://github.com/owner/repo")
        .env("GITHUB_TOKEN", "env-token-value")
        .output()
        .expect("binary must exist");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("GitHub token is required"),
        "should not show token error when GITHUB_TOKEN is set, got: {stderr}"
    );
}

#[test]
fn rejects_empty_token_flag() {
    let output = bin()
        .arg("https://github.com/owner/repo")
        .arg("--token")
        .arg("")
        .env_remove("GITHUB_TOKEN")
        .output()
        .expect("binary must exist");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("GitHub token"),
        "expected token error for empty --token flag, got: {stderr}"
    );
}

#[test]
fn token_flag_takes_precedence_over_env_var() {
    let output = bin()
        .arg("https://github.com/owner/repo")
        .arg("--token")
        .arg("flag-token-value")
        .env("GITHUB_TOKEN", "env-token-value")
        .output()
        .expect("binary must exist");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("GitHub token is required"),
        "should not show token error when --token is set, got: {stderr}"
    );
}
