# `karigor` Crate Context & Architecture

## Vision & Scope

`karigor` (কারিগর — the craftsman / artisan) is the central command-line interface, project scaffolding engine, code
generator, development supervisor, and diagnostic workbench for Druto web applications.

It provides dual invocation paradigms:
- As a Cargo subcommand: `cargo karigor <command>`
- As a standalone executable: `karigor <command>`

## Architecture & Responsibilities

1. **Terminal Presentation & Design System (`ui`)**:
   - Modern hybrid aesthetics combining rounded box boundaries (`╭───╮`), bold HTTP method badges (`GET`, `POST`, `PUT`, `DELETE`), and unicode status badges (`✔`, `➜`, `✖`, `⚠`).
   - Standardized table layouts using `comfy-table` with rounded UTF-8 border styling.
   - Tracing integration for structured telemetry and runtime diagnostic logging.

2. **Core Command Hub (`cli`)**:
   - `new`: Interactive project generator supporting multiple presets (Minimal API, Production API, Full-Stack).
   - `dev` / `serve`: Development HTTP server supervisor with live reload, request logging, and interactive key listeners.
   - `routes`: Inspection table displaying compiled routes, HTTP methods, route parameters, handler references, and attached onion middlewares.
   - `middleware`: Visualization of the onion middleware stack from outermost wrapper to the core route handler.
   - `make:*`: Generator commands (`make:controller`, `make:middleware`, `make:command`, `make:test`) with file injection previews.
   - `doctor`: Comprehensive diagnostic suite checking Rust toolchains, port availability, environment configurations, and external services.
