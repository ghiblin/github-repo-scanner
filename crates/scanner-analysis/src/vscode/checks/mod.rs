use crate::models::Finding;
use scanner_repository::{FileContent, RepoFile};
use scanner_rules::Rule;

#[must_use]
pub fn check(file: &RepoFile, rule: &Rule, pattern: &str) -> Vec<Finding> {
    if !file.path.starts_with(".vscode") {
        return vec![];
    }
    let FileContent::Text(ref content) = file.content else {
        return vec![];
    };
    let Ok(re) = regex::Regex::new(pattern) else {
        return vec![];
    };
    content
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            re.find(line).map(|_| Finding {
                rule_id: rule.id.clone(),
                severity: rule.severity.clone(),
                file: file.path.clone(),
                line: u32::try_from(i).ok().map(|n| n + 1),
                message: rule.name.clone(),
                snippet: Some(line.trim().to_owned()),
            })
        })
        .collect()
}

#[must_use]
pub fn extension_id_check(file: &RepoFile, rule: &Rule) -> Vec<Finding> {
    if !file.path.starts_with(".vscode") {
        return vec![];
    }
    if file.path.file_name().is_none_or(|n| n != "extensions.json") {
        return vec![];
    }
    let FileContent::Text(ref content) = file.content else {
        return vec![];
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(content) else {
        return vec![];
    };
    let Some(recs) = json.get("recommendations").and_then(|r| r.as_array()) else {
        return vec![];
    };
    recs.iter()
        .filter_map(|id| id.as_str())
        .filter(|id| !id.contains('.'))
        .map(|id| Finding {
            rule_id: rule.id.clone(),
            severity: rule.severity.clone(),
            file: file.path.clone(),
            line: None,
            message: format!("extension ID '{id}' has no publisher prefix"),
            snippet: Some((*id).to_owned()),
        })
        .collect()
}
