# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                          # build all crates
cargo test                           # run all tests
cargo test -p scanner-analysis       # run tests for a single crate
cargo test node_001                  # run a single test file by name
cargo nextest run                    # faster test runner (optional)
cargo clippy -- -D warnings          # lint (pedantic, all warnings are errors)
cargo fmt --check                    # verify formatting
```

## Architecture

Cargo workspace with five crates. Data flows strictly left to right; no crate reaches into another's internals:

```
GitHub URL
  → scanner-repository::fetch()   → RepoSnapshot
  → scanner-rules::load()         → RuleSet
  → scanner-analysis::run()       → Vec<Finding>
  → scanner-report::render()      → terminal output + exit code
```

- **`cli`** — only binary; async; wires all other crates; `anyhow` allowed here only
- **`scanner-repository`** — fetches repo content via GitHub API (`GithubApiClient`) or local clone (`LocalCloneClient`); both implement `RepositoryPort` trait
- **`scanner-rules`** — loads/validates TOML rule files; `RuleId` format is `<LANG>-<NNN>` (e.g. `NODE-001`, `VSCODE-003`); fails loudly at load time on invalid rules
- **`scanner-analysis`** — pure sync; `Analyzer` trait with `NodeJsAnalyzer` and `VsCodeAnalyzer` impls; derives `Verdict` (Safe/Suspicious/Dangerous) from findings
- **`scanner-report`** — pure formatting; no business logic; snapshot-tested with `insta`

Exit codes are a stable API contract: `0` = Safe, `1` = Suspicious, `2` = Dangerous.

## Rule Files

TOML rule files live in `rules/` (`nodejs.toml`, `vscode.toml`). Each rule entry must match a known `Language` prefix (`NODE`, `VSCODE`, `ANY`). Adding a new language requires: one new `Language` variant in `scanner-rules`, one new `<LANG>-NNN` ID range, and one new `Analyzer` impl in `scanner-analysis`.

Pattern types: `Regex`, `ScriptKey`, `FileMatch`, `TypoSquat`, `ExtensionIdCheck`, `TerminalEnvInjectionCheck`.

## Testing Conventions

- **TDD is mandatory.** Write a failing test before any production code.
- Each scanner rule has its own test file: `crates/scanner-analysis/tests/node_NNN_test.rs` or `vscode_NNN_test.rs`.
- Every test file must contain at least one true-positive and one false-positive case.
- Tests construct `RepoSnapshot` directly (no I/O) and pass an `Arc<RuleSet>` built from `Rule` structs inline — see [node_001_test.rs](crates/scanner-analysis/tests/node_001_test.rs) as the canonical pattern.
- `scanner-repository` tests use `mockito` for HTTP mocking; async tests use `#[tokio::test]`.

## Non-Negotiable Rules

- No `unwrap()` or `expect()` in production code; no `panic!` in library crates.
- No `unsafe` anywhere (`#![deny(unsafe_code)]` in every crate).
- Library crates use `thiserror`; `anyhow` is `cli`-only.
- Domain types (`RepoSnapshot`, `Finding`, `RuleSet`, `Verdict`) must not import infrastructure crates.
- `clippy::pedantic` enabled; all warnings are errors. Run `cargo fmt` before committing.
- Conventional commits: `type(scope): description` (e.g. `feat(analysis): add NODE-007 credential file rule`).
- Specs and plans live in `docs/specs/NNN-name.md` and `docs/plans/NNN-name.md` with a `NNN-` numeric prefix.
