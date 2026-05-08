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
    VsCode,
}

impl Language {
    #[must_use]
    pub fn prefix(&self) -> &'static str {
        match self {
            Language::NodeJs => "NODE",
            Language::Any => "ANY",
            Language::VsCode => "VSCODE",
        }
    }

    #[must_use]
    pub fn from_prefix(s: &str) -> Option<Self> {
        match s {
            "NODE" => Some(Language::NodeJs),
            "ANY" => Some(Language::Any),
            "VSCODE" => Some(Language::VsCode),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Pattern {
    Regex { value: String },
    ScriptKey { keys: Vec<String> },
    FileMatch { glob: String },
    TypoSquat { known_packages: Vec<String> },
    ExtensionIdCheck,
    TerminalEnvInjectionCheck,
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
    fn nodejs_prefix_is_node() {
        assert_eq!(Language::NodeJs.prefix(), "NODE");
    }

    #[test]
    fn any_prefix_is_any() {
        assert_eq!(Language::Any.prefix(), "ANY");
    }

    #[test]
    fn from_prefix_node_returns_nodejs() {
        assert_eq!(Language::from_prefix("NODE"), Some(Language::NodeJs));
    }

    #[test]
    fn from_prefix_any_returns_any() {
        assert_eq!(Language::from_prefix("ANY"), Some(Language::Any));
    }

    #[test]
    fn from_prefix_unknown_returns_none() {
        assert_eq!(Language::from_prefix("FOO"), None);
    }

    #[test]
    fn from_prefix_empty_returns_none() {
        assert_eq!(Language::from_prefix(""), None);
    }

    #[test]
    fn vscode_prefix_is_vscode() {
        assert_eq!(Language::VsCode.prefix(), "VSCODE");
    }

    #[test]
    fn from_prefix_vscode_returns_vscode() {
        assert_eq!(Language::from_prefix("VSCODE"), Some(Language::VsCode));
    }

    #[test]
    fn ruleset_holds_rules() {
        let id = crate::RuleId::parse("NODE-001").unwrap();
        let rule = Rule {
            id,
            name: "Test rule".to_owned(),
            description: "desc".to_owned(),
            severity: Severity::High,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: "foo".to_owned(),
            },
        };
        let ruleset = RuleSet { rules: vec![rule] };
        assert_eq!(ruleset.rules.len(), 1);
    }
}
