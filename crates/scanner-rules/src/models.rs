// Domain models for scanner rules
use crate::RuleId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Language {
    NodeJs,
    Any,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Pattern {
    Regex { value: String },
    ScriptKey { keys: Vec<String> },
    FileMatch { glob: String },
    TypoSquat { known_packages: Vec<String> },
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub language: Language,
    pub pattern: Pattern,
}

#[derive(Debug)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering_is_critical_highest() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn ruleset_holds_rules() {
        let id = crate::RuleId::parse("rul_018f1234-abcd-7000-8000-000000000001").unwrap();
        let rule = Rule {
            id,
            name: "Test rule".to_owned(),
            description: "desc".to_owned(),
            severity: Severity::High,
            language: Language::NodeJs,
            pattern: Pattern::Regex { value: "foo".to_owned() },
        };
        let ruleset = RuleSet { rules: vec![rule] };
        assert_eq!(ruleset.rules.len(), 1);
    }
}
