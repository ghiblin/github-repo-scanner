use scanner_analysis::{Finding, Verdict};
use scanner_rules::Severity;
use std::path::PathBuf;

fn finding(severity: Severity) -> Finding {
    Finding {
        rule_id: scanner_rules::RuleId::parse("NODE-001").unwrap(),
        severity,
        file: PathBuf::from("test.js"),
        line: None,
        message: "test".to_owned(),
        snippet: None,
    }
}

#[test]
fn no_findings_is_safe() {
    assert_eq!(Verdict::from_findings(&[]), Verdict::Safe);
}

#[test]
fn only_low_medium_is_suspicious() {
    let findings = vec![finding(Severity::Low), finding(Severity::Medium)];
    assert_eq!(Verdict::from_findings(&findings), Verdict::Suspicious);
}

#[test]
fn high_severity_is_dangerous() {
    let findings = vec![finding(Severity::High)];
    assert_eq!(Verdict::from_findings(&findings), Verdict::Dangerous);
}

#[test]
fn critical_severity_is_dangerous() {
    let findings = vec![finding(Severity::Critical)];
    assert_eq!(Verdict::from_findings(&findings), Verdict::Dangerous);
}

#[test]
fn mixed_severities_with_high_is_dangerous() {
    let findings = vec![finding(Severity::Low), finding(Severity::High)];
    assert_eq!(Verdict::from_findings(&findings), Verdict::Dangerous);
}
