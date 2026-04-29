---
name: sdd-define-spec
description: Use when the user wants to define a new feature, component, or system spec. Triggers on "define spec", "create spec", "new spec", or when the user provides a high-level idea to formalize into a design document.
---

# SDD Define Spec

## Overview

Transform a feature idea into a formal spec in `docs/specs/` by asking focused clarifying questions first, then writing once confirmed.

## Process

**Step 1 — Read project context**
Read these files before asking any question:
- `docs/constitution.md`
- `docs/tech-stack.md`
- `docs/rules.md`
- All existing specs in `docs/specs/` (to understand conventions and determine next number)

**Step 2 — Brainstorm with the user**
Ask clarifying questions until all of the following are answered:
- What problem does this solve and for whom?
- What are the key components / bounded contexts?
- What data flows through the system (inputs → outputs)?
- What are the public interfaces, exit codes, or API contracts?
- What error cases need explicit handling?
- What are the testing requirements per `rules.md`?
- What is explicitly **out of scope**?

Ask in batches — do not ask one question per turn. Group related questions.

**Step 3 — Confirm before writing**
Summarize the spec structure and ask the user to approve before creating the file.

**Step 4 — Write spec**
Create `docs/specs/NNN-<kebab-name>-design.md` where `NNN` is the next sequential number.

## Spec Format

```markdown
# <Title> — Design Spec

**Spec:** NNN
**Status:** Draft
**References:** [constitution](../constitution.md) · [tech-stack](../tech-stack.md) · [rules](../rules.md)

---

## Overview
...

## Architecture
...

## Bounded Contexts / Components
(one section per bounded context with domain models, ports/traits, and responsibilities)

## Error Handling
...

## Testing Strategy
(per rules.md: unit + integration, TDD required)

## CLI Interface (if applicable)
...

## Out of Scope
...
```

## Rules

- Do NOT write the file until brainstorm is complete and the user confirms
- Status is always `Draft` — only the user promotes it to `Approved`
- Spec number must not conflict with existing files in `docs/specs/`
- Every spec must have an "Out of Scope" section
