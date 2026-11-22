# Karigor (কারিগর)

Framework-agnostic, developer-friendly CLI application builder and terminal UI toolkit for Rust.

## Features

- **Runtime-Agnostic & Synchronous by Default**: Zero async runtime dependencies in core; executes directly in `fn main() -> std::process::ExitCode`.
- **Declarative & Closure Commands**: Define commands with structured traits, fluent `CommandBuilder`, or ergonomic scoped closures (`kernel.command(...)`).
- **Standard Exit Codes & Safety**: Built-in POSIX/BSD exit code mappings (`0` for success, `2` for syntax/flag errors, `1` for execution failures) with formatted error alerts on `kernel.execute(...)`.
- **Terminal UI Components**: Pre-styled rounded tables, colored banners, information stickers, severity alerts (`AlertKind::{Info, Success, Warning, Danger}`), and section badges.
- **Interactive Security Prompts**: Text input with defaults, yes/no confirmations, numeric/token multiselect, and silent password keystroke suppression with non-interactive CI/headless detection.
- **Progress Indicators**: Terminal spinners and progress bars following terminal capabilities and `--quiet` flags.
- **Template Code Generator**: Built-in template rendering and file publication utilities powered by MiniJinja.

## Quick Start

### Synchronous Closure Commands (Default)

```rust
use karigor::prelude::*;

fn main() -> std::process::ExitCode {
    let kernel = Kernel::new("my-cli", "Developer Tool Suite")
        .version("0.1.0")
        .command("build", "Compile and build project artifacts", |cmd| {
            cmd.flag(Flag::boolean("release", Some('r'), "Build in release mode"))
                .argument(Argument::optional("target", "Target architecture", None))
                .handle(|ctx| {
                    let release = ctx.flag_bool("release");
                    let target = ctx.arg("target").unwrap_or("native");

                    ctx.ui.render_sticker("Build Pipeline", |s| {
                        s.line(&format!("Target: {target}"))
                            .line(&format!("Optimized: {release}"));
                    });

                    ctx.ui.success("Artifacts generated successfully!");
                    Ok(())
                })
        });

    let exit_code = kernel.execute_env();
    std::process::ExitCode::from(exit_code as u8)
}
```

### Struct-Based Command

```rust
use karigor::prelude::*;

struct GreetCommand;

impl Command for GreetCommand {
    fn name(&self) -> &str {
        "greet"
    }

    fn description(&self) -> &str {
        "Greets a user"
    }

    fn arguments(&self) -> Vec<Argument> {
        vec![Argument::required("name", "Name of user to greet")]
    }

    fn run(&self, ctx: &mut Context) -> Result<(), Error> {
        let name = ctx.arg("name").unwrap_or("developer");
        ctx.ui.success(&format!("Hello, {name}!"));
        Ok(())
    }
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
