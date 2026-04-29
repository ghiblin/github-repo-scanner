# Rules

These rules are enforced across the entire codebase. They are not guidelines — they are requirements. Any code that violates them must be fixed before merging.

---

## Test-Driven Development (TDD)

- **Write the test first.** No production code is written without a failing test that justifies it.
- **Red → Green → Refactor.** Tests fail, then minimal code makes them pass, then code is cleaned up. Never skip the red phase.
- **One test file per rule.** Each scanner rule (e.g. `NODE-001`) has its own test file with at least one true-positive and one false-positive test case.
- **No test skips without a comment.** `#[ignore]` requires a comment explaining why and a linked issue.
- **Tests are first-class code.** Tests are readable, well-named, and maintained to the same standard as production code.
- **100% of public interfaces must be covered.** Every public function, trait impl, and error variant must have at least one test.

---

## Domain-Driven Design (DDD)

- **Bounded contexts are enforced by crate boundaries.** One crate per context. Cross-context communication happens only through public domain types — never through internal structs.
- **Domain types carry no framework dependencies.** `RepoSnapshot`, `Finding`, `RuleSet`, and `Verdict` must not import `reqwest`, `clap`, `tokio`, or any other infrastructure crate.
- **Ports and adapters.** Each context that touches I/O exposes a trait (port). Concrete implementations (adapters) live in the same crate but behind the trait. Tests use mock adapters.
- **Ubiquitous language.** The names in code must match the names used in design documents. If a design doc says `Finding`, the struct is `Finding` — not `Issue`, `Result`, or `Warning`.
- **No leaking of infrastructure concerns.** Error types, serialization details, and HTTP concepts must not appear in domain structs.

---

## Rust Best Practices

### Safety
- `#![deny(unsafe_code)]` in every library crate. No exceptions.
- No `unwrap()` or `expect()` in production code. Use `?` or handle errors explicitly.
- No `panic!` in library crates. Only `cli` may panic (and only at startup for unrecoverable configuration errors).

### Error Handling
- Library crates use `thiserror` for typed error enums.
- `anyhow` is allowed only in `cli`.
- Every error variant must have a human-readable message via `#[error("...")]`.
- Errors must be propagated with `?`, not swallowed with `let _ =`.

### Types
- Prefer `&str` over `String` in function parameters when ownership is not required.
- Use newtypes to distinguish semantically different strings (e.g. `RuleId(String)` vs a plain `String`).
- `#[must_use]` on all types where ignoring the value is almost certainly a bug (`Finding`, `Verdict`).
- Avoid `clone()` in hot paths. Use `Arc` for shared ownership of large read-only data (e.g. `Arc<RuleSet>`).

### Async
- Only `cli` and `repository` crates are async. All other crates are synchronous.
- No `block_on` or `spawn_blocking` inside library crates.
- Use `tokio::test` for async tests in `repository`.

### Code Style
- `clippy::pedantic` is enabled for all crates. All warnings are errors in CI.
- `rustfmt` is enforced. All code must be formatted before committing.
- No dead code (`#[allow(dead_code)]` is forbidden without a comment and linked issue).
- Maximum function length: 30 lines. Functions longer than this should be decomposed.
- No nested closures deeper than two levels.

### Dependencies
- Every new dependency requires a justification comment in `Cargo.toml`.
- No dependencies with known CVEs (enforced by `cargo audit` in CI).
- Prefer the standard library over external crates for simple tasks.

---

## Git & Collaboration

- **Conventional commits.** Commit messages follow the format: `type(scope): description` (e.g. `feat(analysis): add NODE-004 shell execution rule`).
- **One logical change per commit.** Do not mix refactoring with feature work in the same commit.
- **No commented-out code.** Dead code is deleted, not commented out.
- **No `TODO` without an issue.** Any `TODO` comment must reference a GitHub issue number.
