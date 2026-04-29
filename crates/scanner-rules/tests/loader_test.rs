use scanner_rules::loader::load;
use std::path::Path;

#[test]
fn loads_valid_ruleset() {
    let path = Path::new("tests/fixtures/valid.toml");
    let ruleset = load(path).expect("should load");
    assert_eq!(ruleset.rules.len(), 1);
    assert_eq!(ruleset.rules[0].name, "Test rule");
}

#[test]
fn rejects_invalid_pattern_type() {
    let path = Path::new("tests/fixtures/invalid_pattern_type.toml");
    assert!(load(path).is_err());
}

#[test]
fn rejects_duplicate_ids() {
    let path = Path::new("tests/fixtures/duplicate_ids.toml");
    let err = load(path).unwrap_err();
    assert!(err.to_string().contains("duplicate"));
}

#[test]
fn rejects_missing_required_field() {
    let path = Path::new("tests/fixtures/missing_field.toml");
    assert!(load(path).is_err());
}

#[test]
fn loads_default_nodejs_ruleset() {
    let path = std::path::Path::new("../../rules/nodejs.toml");
    let ruleset = load(path).expect("default ruleset must be valid");
    assert_eq!(ruleset.rules.len(), 6);
}
