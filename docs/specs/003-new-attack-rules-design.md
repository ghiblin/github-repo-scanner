# New Attack Rules Design

**Date:** 2026-05-08
**Status:** Approved

## Overview

Extends the scanner with nine new detection rules across two namespaces:

- **NODE-007 to NODE-011** — five new Node.js supply chain attack rules
- **VSCODE-001 to VSCODE-004** — four VS Code workspace poisoning rules under a new `VSCODE` namespace

---

## Architecture Changes

### `scanner-rules` crate

Add `Language::VsCode` variant:

```rust
Language::VsCode => "VSCODE"
```

`Language::from_prefix("VSCODE")` returns `Some(Language::VsCode)`.

One new `Pattern` variant is required:

```rust
Pattern::ExtensionIdCheck
```

No fields — it is a marker that triggers a JSON-aware check of the `recommendations` array in `extensions.json`, analogous to how `ScriptKey` triggers a JSON-aware check of `package.json`. All other new rules use the existing `Regex` or `ScriptKey` pattern types. The TOML rule format gains one new pattern type name: `ExtensionIdCheck`.

### `scanner-analysis` crate

Add a `vscode` module at `crates/scanner-analysis/src/vscode/` containing:

- `mod.rs` — `VsCodeAnalyzer` struct implementing the `Analyzer` trait
- `checks/mod.rs` — reuses the same line-by-line `check` helper as `nodejs/checks/mod.rs`, but scopes file selection to paths starting with `.vscode/`

`VsCodeAnalyzer` is registered in `analyzer.rs` alongside `NodeJsAnalyzer`. No changes to `nodejs/mod.rs` or any other crate.

### No changes to

`scanner-repository`, `scanner-report`, `cli`

---

## Node.js New Rules (NODE-007 to NODE-011)

### NODE-007 — Credential file access

| Field | Value |
|---|---|
| Severity | Critical |
| Pattern | `Regex` |
| Regex | `readFileSync\s*\(['"].*(?:\.ssh\|\.aws\|\.gnupg\|\.npmrc\|\.netrc)\|readFile\s*\(['"].*(?:\.ssh\|\.aws\|\.gnupg\|\.npmrc\|\.netrc)` |

Catches the real-world pattern of malicious npm packages reading `~/.ssh/id_rsa`, `~/.aws/credentials`, or `~/.npmrc` to exfiltrate credentials. Applies to all JS/TS file extensions.

### NODE-008 — Dynamic eval alternative

| Field | Value |
|---|---|
| Severity | High |
| Pattern | `Regex` |
| Regex | `new\s+Function\s*\(\|setTimeout\s*\(\s*['"]\|setInterval\s*\(\s*['"]` |

Closes the eval bypass gap not covered by NODE-002. `new Function('return process.env')()` and `setTimeout("malicious()", 0)` are common obfuscation techniques used to avoid static `eval` detection.

### NODE-009 — Prototype pollution

| Field | Value |
|---|---|
| Severity | High |
| Pattern | `Regex` |
| Regex | `Object\.prototype\.\|__proto__\s*\[` |

Targets supply chain attacks that silently mutate shared objects for all downstream consumers. A polluted prototype can override security checks or inject properties into every object in the process.

### NODE-010 — Env variable exfiltration

| Field | Value |
|---|---|
| Severity | Critical |
| Pattern | `Regex` |
| Regex | `process\.env.*(?:fetch\|http\|axios\|request)\|(?:fetch\|http\|axios\|request).*process\.env` |

Requires both `process.env` AND a network primitive on the same line to minimise false positives on files that legitimately read env vars for configuration without sending them anywhere.

### NODE-011 — Expanded lifecycle hooks

| Field | Value |
|---|---|
| Severity | Critical |
| Pattern | `ScriptKey` |
| Keys | `["postinstall", "preinstall", "prepare", "install", "prepack", "postpack"]` |

Extends NODE-001 with additional lifecycle hooks that run automatically during `npm install` or `npm publish`. Attackers use `prepare` and `install` to bypass defences that only watch `postinstall`.

---

## VS Code Workspace Poisoning Rules (VSCODE-001 to VSCODE-004)

All four rules use `Language::VsCode`. The `VsCodeAnalyzer` only processes files whose path starts with `.vscode/`.

### VSCODE-001 — Auto-run task on open

| Field | Value |
|---|---|
| Severity | Critical |
| File | `.vscode/tasks.json` |
| Pattern | `Regex` |
| Regex | `"runOn"\s*:\s*"folderOpen"` |

A `folderOpen` task executes silently the moment a developer opens the repository — no prompt, no confirmation. Attackers commit a `tasks.json` with a `runOn: folderOpen` shell task to achieve immediate code execution on any developer machine that clones the repo.

### VSCODE-002 — Interpreter path hijacking

| Field | Value |
|---|---|
| Severity | Critical |
| File | `.vscode/settings.json` |
| Pattern | `Regex` |
| Regex | `python\.defaultInterpreterPath\|eslint\.nodePath\|typescript\.tsdk` |

Catches the "poisoned interpreter" pattern: attacker commits a malicious wrapper binary at a local path (e.g. `./scripts/python`) and sets `python.defaultInterpreterPath` to point at it. Every Python command VS Code runs — including test execution, linting, and formatting — invokes the attacker's binary instead.

### VSCODE-003 — Terminal env injection

| Field | Value |
|---|---|
| Severity | High |
| File | `.vscode/settings.json` |
| Pattern | `Regex` |
| Regex | `terminal\.integrated\.env.*(?:PATH\|LD_PRELOAD\|DYLD_INSERT)` |

`terminal.integrated.env` overrides environment variables for all integrated terminal sessions. Injecting `LD_PRELOAD` or prepending a malicious directory to `PATH` hijacks every subprocess spawned from the terminal for the lifetime of the VS Code session.

### VSCODE-004 — Suspicious extension recommendation

| Field | Value |
|---|---|
| Severity | Medium |
| File | `.vscode/extensions.json` |
| Pattern | `ExtensionIdCheck` |

Extension IDs in `extensions.json` must follow the `publisher.extension-name` format. IDs without a `.` separator do not belong to any registered publisher namespace and are a marker of a fabricated or placeholder extension ID used in social engineering attacks. Medium severity because the user must still manually confirm installation.

---

## Test Coverage

Following project rules, each new rule gets its own test file:

- `tests/node_007_test.rs` through `tests/node_011_test.rs`
- `tests/vscode_001_test.rs` through `tests/vscode_004_test.rs`

Each test file contains at least one true-positive and one false-positive case.

---

## Out of Scope

- Python / pip ecosystem rules (future work)
- Rust / crates.io rules (future work)
- Implementation of `FileMatch` glob pattern (not needed for this batch)
