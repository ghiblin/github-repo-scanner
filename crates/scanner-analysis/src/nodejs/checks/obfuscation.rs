use crate::models::Finding;
use scanner_repository::{FileContent, RepoFile};
use scanner_rules::Rule;

const JS_EXTENSIONS: &[&str] = &["js", "ts", "mjs", "cjs", "jsx", "tsx"];

#[must_use]
pub fn check(file: &RepoFile, rule: &Rule, pattern: &str) -> Vec<Finding> {
    let ext = file.path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !JS_EXTENSIONS.contains(&ext) {
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
