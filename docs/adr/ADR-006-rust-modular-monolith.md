# ADR-006: Use a Rust modular monolith for the initial application

- **Status:** Accepted
- **Date:** 2026-07-25

## Context

The design documents describe clean layers and optional modules but leave three implementation questions unresolved:

- whether application use cases live in Rust or TypeScript;
- whether the Rust backend begins as several crates or one crate;
- whether optional capabilities run in-process or behind a separate host.

Starting with many crates, processes, and plugin contracts would add packaging and coordination cost before the core library and reader workflows exist. Splitting business behavior between React and Rust would also make filesystem, database, cancellation, and transaction rules difficult to test consistently.

## Decision

Use a **Rust modular monolith** for the first implementation.

Domain rules, application use cases, persistence orchestration, filesystem operations, reader adapters, jobs, and Tauri command handlers live in Rust. React and TypeScript own presentation, interaction state, routing, and typed calls to the Tauri boundary.

Begin with one Tauri Rust crate organized into explicit modules:

```text
src-tauri/src/
  domain/
  application/
  infrastructure/
  desktop/
```

Feature-oriented submodules may exist inside those layers, for example `library`, `reader`, `notes`, and `search`. Dependency direction is enforced by ownership and tests before introducing multiple workspace crates.

Optional capabilities such as OCR, dictionary, and AI providers run in-process initially behind application-owned ports. They remain disabled by default and cannot be required by core use cases.

## Dependency rules

- `domain` depends only on the Rust standard library and narrowly justified domain-safe libraries.
- `application` depends on `domain` and defines ports required from infrastructure.
- `infrastructure` depends on `application` and `domain` to implement ports.
- `desktop` depends on `application` and infrastructure composition; Tauri commands contain no business rules.
- React depends only on typed command/event contracts, never on SQLite or direct source-folder access.
- Infrastructure modules must not call presentation code.

Call flow may travel from UI to infrastructure through a use case, but source-code dependencies still point toward domain/application contracts. Diagrams must distinguish call flow from compile-time dependency direction.

## Considered options

### Put some application logic in TypeScript

Rejected for the core because filesystem, transactions, jobs, and error recovery would be split across the WebView and native process. TypeScript may contain view models and UI-only validation but not authoritative business rules.

### Start with multiple Rust workspace crates

Deferred. Multiple crates can enforce boundaries, but they increase setup, build, dependency, and publishing complexity before the module seams are proven.

### Start with one unstructured Tauri crate

Rejected. A single crate is acceptable only with explicit module boundaries and dependency rules.

### Run optional modules in separate processes

Deferred until there is a real need for untrusted plugins, crash isolation, independent upgrades, or resource isolation.

## Consequences

Positive consequences:

- one authoritative place for domain and application behavior;
- simpler transactions, cancellation, recovery, and integration testing;
- faster Sprint 01 scaffold with fewer premature packaging decisions;
- clear path to extract stable modules into crates later.

Trade-offs:

- Rust module boundaries are partly enforced by convention and visibility at first;
- optional providers are trusted in-process code;
- careless shared utilities could still create coupling.

## Implementation constraints

- Use `pub(crate)` or narrower visibility by default.
- Do not create a generic `utils` dumping ground; helpers belong to the module that owns the concept.
- Define infrastructure ports in `application` or `domain`, never in adapter modules.
- Keep Tauri command payloads separate from domain entities when serialization concerns differ.
- Add architecture tests or lint rules when practical.
- Extract a module into a crate only when it has a stable public contract, independent tests, or a demonstrated need for compilation isolation.

## Revisit when

Revisit the crate/process structure after the Library and Reader MVPs reveal stable module seams, or when untrusted plugins require a sandboxed host.