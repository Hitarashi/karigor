# Karigor (`karigor`)

Framework-agnostic, developer-friendly CLI application builder and terminal UI toolkit for Rust.

## Features

- **Declarative Kernel Dispatcher**: Register commands and groups with support for nested/colon subcommands (`make:route` ↔ `make route`).
- **Fuzzy Command Matching**: Automatically suggests closest matching commands on typo inputs with Levenshtein distance.
- **Beautiful Terminal UI**: Pre-styled rounded tables, colored banners, section badges, and status logs.
- **Interactive Prompts**: Text input with defaults, yes/no confirmation prompts, and single-selection lists powered by `dialoguer`.
- **Steady Progress Indicators**: Elegant terminal spinners and progress bars powered by `indicatif`.
- **Template Code Generator**: Built-in template rendering and file ejection powered by `minijinja`.

## Quick Start

```rust
use karigor::prelude::*;

struct BuildCommand;

impl Command for BuildCommand {
    fn name(&self) -> &'static str {
        "build"
    }

    fn description(&self) -> &'static str {
        "Compile and build project artifacts"
    }

    fn flags(&self) -> Vec<Flag> {
        vec![Flag::boolean("release", Some('r'), "Build in release mode")]
    }

    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        Box::pin(async move {
            let release = ctx.flag_bool("release");
            ctx.ui.section("Build Pipeline");
            if release {
                ctx.ui.info("Compiling with release optimizations...");
            } else {
                ctx.ui.info("Compiling in debug mode...");
            }
            ctx.ui.success("Artifacts generated successfully!");
            Ok(())
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kernel = Kernel::new("my-cli", "Developer Tool Suite")
        .version("0.1.0")
        .register(BuildCommand);

    kernel.handle_env().await?;
    Ok(())
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
