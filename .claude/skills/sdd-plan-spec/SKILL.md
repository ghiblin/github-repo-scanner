---
name: sdd-plan-spec
description: Use when the user has a spec and wants to create an implementation plan. Triggers on "plan spec", "create plan", "write the plan", or when the user references a spec number and wants tasks to implement it.
---

# SDD Plan Spec

## Overview

Transform a spec into a detailed TDD implementation plan saved to `docs/plans/`.

## Process

**Step 1 — Load spec and context**
Read in parallel:
- The latest spec from `docs/specs/` (or the one the user specifies)
- `docs/constitution.md`, `docs/tech-stack.md`, `docs/rules.md`

**Step 2 — Draft task list**
Break the spec into ordered implementation tasks. Each task must:
- Have a clear name and scope (one bounded context or one logical unit)
- List exact files to create or modify
- Follow TDD: failing tests → implement → verify → commit

**Step 3 — Confirm task list with user**
Present the task breakdown before writing any file. Ask if the order or scope needs adjustment.

**Step 4 — Write plan**
Create `docs/plans/NNN-<kebab-name>.md` using the **same NNN** as the spec it implements.

## Plan Format

```markdown
# <Title> Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ...
**Architecture:** ...
**Tech Stack:** ...

---

## File Map

\`\`\`
path/to/file    description
\`\`\`

---

## Task N: <Name>

**Files:**
- Create: `path/to/file`
- Modify: `path/to/file`

- [ ] **Step 1: Write failing tests**
\`\`\`rust
// test code
\`\`\`

- [ ] **Step 2: Run to verify failure**
\`\`\`bash
cargo test -p <crate>
\`\`\`
Expected: compile error / test failure.

- [ ] **Step 3: Implement**
\`\`\`rust
// implementation
\`\`\`

- [ ] **Step 4: Run tests to verify they pass**
\`\`\`bash
cargo test -p <crate>
\`\`\`
Expected: N tests pass.

- [ ] **Step 5: Commit**
\`\`\`bash
git add <files>
git commit -m "feat(<crate>): ..."
\`\`\`
```

## Rules

- Every task must follow TDD: tests first, implementation second
- Tasks must reference exact file paths (no "add a file for X")
- A commit step is required at the end of every task
- Plan NNN must match the spec NNN it implements
- Always include the subagent-driven-development note at the top
- Include implementation code in the plan — the plan must be self-contained enough to execute without re-reading the spec
