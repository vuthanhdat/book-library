# Contributing to Book Library

Book Library is designed for human and AI-assisted development. Consistency, data safety, and truthful documentation matter more than individual style.

Read [AGENTS.md](AGENTS.md) before contributing. Its architecture and data-safety rules are mandatory.

## Before starting

1. Confirm the product behavior in `docs/01-product/`.
2. Locate the feature or backlog identifier.
3. Read the owning module specification.
4. Check [accepted ADRs](docs/adr/README.md).
5. Confirm the work belongs to the active milestone or sprint.
6. Resolve any blocking design question with an ADR before coding.

The [documentation guide](docs/README.md) defines which document wins when content conflicts.

## Branches

Use one focused branch per change. Suggested prefixes:

- `feature/` — user-visible capability;
- `fix/` — defect correction;
- `refactor/` — behavior-preserving code improvement;
- `docs/` — documentation-only change;
- `chore/` — tooling, dependencies, or repository maintenance;
- `agent/` — AI-agent working branch.

Do not mix unrelated cleanup into a feature pull request.

## Commit messages

Use concise Conventional Commit messages:

```text
feat(library): add relative path validation
fix(reader): preserve page after reopen
docs: record database location decision
refactor(search): isolate FTS query builder
test(library): cover natural page ordering
chore(ci): add Rust and frontend checks
```

Each commit should leave the branch understandable. Separate mechanical restructuring from behavior changes when that makes review safer.

## Development workflow

Use this sequence:

```text
requirement -> decision -> implementation -> tests -> documentation -> review
```

Implementation rules:

- enter business behavior through an application use case;
- keep Tauri commands thin;
- keep React free of database and source-filesystem access;
- implement infrastructure behind application/domain ports;
- preserve user-owned books and Markdown notes;
- avoid persisted absolute content paths;
- make long-running work observable and recoverable according to the milestone.

## Testing

Run all checks available for the changed area. Sprint 01 is expected to establish:

- Rust formatting, linting, and tests;
- TypeScript type checking and frontend build;
- frontend tests where meaningful;
- Markdown linting or link validation;
- integration tests with temporary SQLite databases and filesystem fixtures.

A missing test command is not evidence that a change is safe. Add focused tests for critical rules before marking a feature complete.

## Documentation updates

Update only authoritative documents affected by the change:

| Change | Documentation |
|---|---|
| Product behavior or scope | requirements and feature catalog |
| Technical decision | new ADR, or supersede an existing ADR |
| Layer/module ownership | architecture documents |
| Database schema or migration | persistence and recovery notes |
| Milestone/sprint scope | implementation plan, backlog, or sprint |
| Feature completion | feature catalog status |

Do not mark planned functionality as implemented until the source and tests exist.

## Pull request expectations

A pull request should explain:

- what changed;
- why the change is needed;
- which feature/backlog item it serves;
- important architecture or data-safety implications;
- checks that were run;
- remaining limitations or follow-up work.

Keep the PR draft while significant checks or acceptance criteria are incomplete.

## Review checklist

Before requesting review, verify:

- scope is focused;
- accepted ADRs are followed;
- dependency direction is preserved;
- no duplicate domain/use-case logic was added;
- source books and note files are not modified unexpectedly;
- errors and cancellation are handled;
- critical tests pass;
- documentation matches the branch;
- no secrets, logs, caches, build outputs, or debug code are committed.

## Definition of done

A task is done when its acceptance criteria pass, tests and builds for affected areas pass, error/recovery behavior is implemented, and the authoritative documentation reflects the actual code.