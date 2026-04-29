use scanner_rules::{RuleId, Severity};
use std::path::PathBuf;

#[must_use]
#[derive(Debug, Clone)]
pub struct Finding {
    pub rule_id: RuleId,
    pub severity: Severity,
    pub file: PathBuf,
    pub line: Option<u32>,
    pub message: String,
    pub snippet: Option<String>,
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Safe,
    Suspicious,
    Dangerous,
}

impl Verdict {
    #[must_use]
    pub fn from_findings(findings: &[Finding]) -> Self {
        if findings.is_empty() {
            return Self::Safe;
        }
        let max = findings.iter().map(|f| &f.severity).max();
        match max {
            Some(Severity::Critical | Severity::High) => Self::Dangerous,
            _ => Self::Suspicious,
        }
    }
}
