# Comments

Comment the why, and only when the code cannot state it: a constraint from
outside the file, a deliberate omission, or a decision whose alternative a
reader would otherwise pick.

If a competent reader could infer it from the code, the code already says it.
Rustdoc (`///`, `//!`) is always fine — it documents the public surface.

# Code Style and Lints

- Follow [Google's Rust Style Guide](https://google.github.io/styleguide/rustguide.md).
- Do not suppress Clippy warnings inline (`#[allow(...)]` or `#[expect(...)]`) at all. Resolve the warning or
  restructure the code instead.
- Avoid using leading underscores (`_`) on identifiers, variables, parameters, or struct fields.
- Code formatting must be verified via `cargo +nightly fmt --check`.
- Always check local repo `.md` files whether ignored or not.

# Framework-Agnostic Documentation & Codebase

- Do not mention any specific web frameworks in rustdocs, codebase comments, identifiers, or error messages.
- Reference only official standards, terminal conventions, and Rust types.

# Scope & Responsibilities

`karigor` is the framework-agnostic CLI toolkit and terminal UI engine under `Hitarashi/karigor`.
Its responsibilities include:
1. Command registry, lifecycle management, and execution kernel.
2. Argument parsing, flag validation, and colon-delimited subcommand normalization.
3. Terminal UI presentation (banners, rounded tables, status bullets, badges, spinners).
4. Interactive prompts (text input, confirmations, selection menus).
5. Template rendering and file publication utilities.
