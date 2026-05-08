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
            id: RuleId::parse("NODE-007").unwrap(),
            name: "Credential file access".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r#"readFileSync\s*\(['"].*(?:\.ssh|\.aws|\.gnupg|\.npmrc|\.netrc)|readFile\s*\(['"].*(?:\.ssh|\.aws|\.gnupg|\.npmrc|\.netrc)"#.to_owned(),
            },
        }],
    })
}

#[test]
fn detects_ssh_key_read() {
    let src = r"const key = fs.readFileSync('/home/user/.ssh/id_rsa', 'utf8');";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_aws_credentials_read() {
    let src = r"readFile('/root/.aws/credentials', callback);";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_file_read() {
    let src = r"const data = fs.readFileSync('./config.json', 'utf8');";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_non_js_files() {
    let src = r"readFileSync('/root/.ssh/id_rsa')";
    let findings = NodeJsAnalyzer.analyze(&snapshot("README.md", src), &ruleset());
    assert!(findings.is_empty());
}
