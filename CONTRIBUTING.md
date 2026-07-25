# CONTRIBUTING.md

# Contributing to Book Library

Thank you for contributing to Book Library.

This project is designed to be understandable by both humans and AI coding agents.
Consistency is more important than individual coding style.

---

# Project Philosophy

Before writing code, understand these principles.

- Desktop First
- Offline First
- Filesystem First
- Relative Path Only
- Markdown First
- AI Optional

Read `AGENTS.md` before contributing.

---

# Development Workflow

Every change should follow this order.

Requirement

↓

Design

↓

Implementation

↓

Testing

↓

Documentation

↓

Commit

Do not skip documentation.

---

# Before Starting

Before implementing a feature, verify:

- Is there already a Feature Spec?
- Does an ADR already define this decision?
- Does the architecture document already describe this module?
- Is another module already responsible for this behavior?

Avoid duplicate implementations.

---

# Branch Strategy

Recommended branch names

feature/library-scanner

feature/pdf-reader

feature/notes

feature/search

fix/thumbnail-cache

refactor/domain-model

docs/database-schema

Do not develop unrelated features in the same branch.

---

# Commit Messages

Use Conventional Commits.

Examples

feat: add library scanner

feat(reader): implement PDF page navigation

fix(notes): preserve markdown formatting

docs: update architecture

refactor(search): simplify FTS indexing

test(reader): add PDF loading tests

Avoid messages like

update

fix

change

misc

---

# Pull Request Checklist

Before opening a Pull Request, verify:

- Feature works
- Tests pass
- Documentation updated
- No duplicate logic
- No architecture violation
- No unused code
- No debug code
- No unnecessary dependencies

---

# Code Organization

Business logic belongs in

Application

or

Domain

Never inside

React Components

Tauri Commands

SQLite Repositories

---

# Preferred Development Style

Prefer

Small functions

Explicit names

Composition over inheritance

Immutable objects where practical

Dependency Injection

Strong typing

Pure functions when possible

Avoid

God Services

Utility classes with unrelated methods

Long functions

Deep nesting

Shared mutable state

Copy-paste implementations

---

# Error Handling

Never silently ignore errors.

Prefer

Return typed errors.

Recover when possible.

Log useful context.

Show meaningful messages to users.

Do not crash unless data integrity is at risk.

---

# Database Rules

SQLite stores

- metadata
- indexes
- reading state
- settings
- jobs

SQLite never stores

- PDFs
- images
- Markdown contents

Never persist absolute paths.

---

# Filesystem Rules

The user's library belongs to the user.

Never

Rename books

Move books

Delete books

Modify PDFs

Modify image folders

unless explicitly requested.

Generated artifacts such as

- thumbnails
- cache
- indexes

must be stored separately.

---

# Testing

Each feature should include appropriate tests.

Recommended order

Unit Tests

↓

Application Tests

↓

Integration Tests

UI tests are optional during early development.

---

# Documentation

Whenever changing

Database

API

Architecture

Events

Jobs

Settings

also update the corresponding documentation.

Documentation is considered part of the feature.

---

# Definition of Done

A task is complete only when

- Code is finished
- Tests pass
- Documentation updated
- No architecture violations
- No duplicated logic
- Feature satisfies its specification

---

# Project Structure

```

docs/
architecture/
database/
api/
product/

planning/

specs/

src/

```

Keep documentation synchronized with implementation.

---

# Final Principle

Optimize for long-term maintainability rather than short-term speed.

The codebase should remain understandable after several years by contributors who have never seen the project before.