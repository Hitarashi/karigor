# druto-karigor

[![Crates.io](https://img.shields.io/crates/v/druto-karigor.svg)](https://crates.io/crates/druto-karigor)
[![Documentation](https://docs.rs/druto-karigor/badge.svg)](https://docs.rs/druto-karigor)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Craftsman CLI, interactive scaffolding, dev server, and code generators for Druto.

`karigor` (কারিগর — the craftsman / artisan) is the central command-line engine for the Druto web framework. It manages the entire application lifecycle—from project creation and component scaffolding to dev server supervision, route introspection, and diagnostics.

---

## Installation

```bash
cargo install druto-karigor
```

This installs both `karigor` and `cargo-karigor`. You can invoke commands either as:

```bash
cargo karigor <command>
# or
karigor <command>
```

---

## Commands

- **`karigor new <name>`**: Launch the interactive project scaffolding wizard.
- **`karigor dev`**: Run the development server with live reload and interactive inspection shortcuts.
- **`karigor routes`**: Inspect all compiled application routes in a formatted table with methods and middlewares.
- **`karigor middleware`**: Visualize the middleware onion pipeline.
- **`karigor make:controller <name>`**: Scaffold a new route controller.
- **`karigor make:middleware <name>`**: Scaffold a custom onion middleware.
- **`karigor make:command <name>`**: Scaffold a custom task or command.
- **`karigor doctor`**: Run system, port, environment, and connectivity diagnostics.

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
