use crate::{error::RulesError, models::RuleSet, rule_id::RuleId};
use std::{collections::HashSet, path::Path};

#[derive(serde::Deserialize)]
struct RuleSetFile {
    rules: Vec<crate::models::Rule>,
}

/// # Errors
/// Returns an error if the file cannot be read, the TOML is malformed, a rule ID has an invalid
/// format, a rule ID prefix does not match its language, or duplicate rule IDs are found.
pub fn load(path: &Path) -> Result<RuleSet, RulesError> {
    let contents = std::fs::read_to_string(path)?;
    let file: RuleSetFile = toml::from_str(&contents)?;
    let mut seen_ids = HashSet::new();
    for rule in &file.rules {
        RuleId::parse(rule.id.as_str())?;
        if !seen_ids.insert(rule.id.as_str().to_owned()) {
            return Err(RulesError::DuplicateId(rule.id.to_string()));
        }
        let actual_prefix = rule.id.as_str().split('-').next().unwrap_or("");
        let expected_prefix = rule.language.prefix();
        if actual_prefix != expected_prefix {
            return Err(RulesError::LanguageMismatch {
                id: rule.id.to_string(),
                actual_prefix: actual_prefix.to_owned(),
                expected_prefix: expected_prefix.to_owned(),
            });
        }
    }
    Ok(RuleSet { rules: file.rules })
}
