# `karigor` Agent Instructions

## Scope

This crate owns the command-line interface, interactive project scaffolding, code generation,
development server supervisor, route and middleware inspection, and diagnostic tooling for the Druto ecosystem.

## Design Rules

1. **Terminal Aesthetics & UX**:
   - Adhere to the Hybrid Modern design language: clean rounded borders (`╭───╮`), bold colored badges for HTTP methods, and clean unicode status symbols (`✔`, `➜`, `✖`, `⚠`).
   - Terminal styling must degrade gracefully when colors or unicode are not supported (e.g. `NO_COLOR=1` or non-interactive TTY).
   - Use `tracing` for all internal operational logging, debug channels, and structured telemetry.
   - Command output must be responsive, clear, and informative.

2. **Command Dispatch & Ergonomics**:
   - Provide dual binary entrypoints: `cargo-karigor` (invoked via `cargo karigor <cmd>`) and `karigor` (invoked directly).
   - Handle both single and multi-word command syntax (e.g. `make:controller` and `make controller`).
   - Support global flags: `--verbose`, `--quiet`, `--json`, `--no-color`.

3. **Code Quality & Style**:
   - Follow Google's Rust Style Guide.
   - Zero inline Clippy suppressions (`#[allow(...)]` or `#[expect(...)]`). Resolve warnings or restructure code instead.
   - Avoid leading underscores (`_`) on identifiers, parameters, and fields.
   - Verify formatting with `cargo +nightly fmt --check`.

4. **Framework-Agnostic Documentation**:
   - Reference only RFCs, W3C standards, official specifications, and Druto types.
   - Do not mention other web frameworks in rustdocs, comments, identifiers, or error messages.
