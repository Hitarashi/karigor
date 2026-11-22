use std::sync::atomic::{AtomicBool, Ordering};

use karigor::prelude::*;
use serde_json::json;

struct GreetCommand;

impl Command for GreetCommand {
    fn name(&self) -> &'static str {
        "greet"
    }

    fn description(&self) -> &'static str {
        "Prints a personalized greeting"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["hello", "hi"]
    }

    fn arguments(&self) -> Vec<Argument> {
        vec![
            Argument::required("name", "The person to greet"),
            Argument::optional("title", "Honorific title", Some("Developer".to_string())),
        ]
    }

    fn flags(&self) -> Vec<Flag> {
        vec![
            Flag::boolean("loud", Some('l'), "Print in uppercase"),
            Flag::string(
                "prefix",
                Some('p'),
                "Greeting prefix",
                Some("Hello".to_string()),
            ),
        ]
    }

    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        Box::pin(async move {
            let name = ctx.arg("name").unwrap();
            let title = ctx.arg("title").unwrap();
            let loud = ctx.flag_bool("loud");
            let prefix = ctx.flag_str("prefix").unwrap();

            let message = format!("{prefix}, {title} {name}!");
            if loud {
                ctx.ui.success(&message.to_uppercase());
            } else {
                ctx.ui.success(&message);
            }

            Ok(())
        })
    }
}

static MAKE_ROUTE_EXECUTED: AtomicBool = AtomicBool::new(false);

struct MakeRouteCommand;

impl Command for MakeRouteCommand {
    fn name(&self) -> &'static str {
        "make:route"
    }

    fn description(&self) -> &'static str {
        "Generates a new route handler"
    }

    fn arguments(&self) -> Vec<Argument> {
        vec![Argument::required("name", "Route module name")]
    }

    fn flags(&self) -> Vec<Flag> {
        vec![Flag::boolean(
            "controller",
            Some('c'),
            "Generate paired controller",
        )]
    }

    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        Box::pin(async move {
            assert_eq!(ctx.arg("name"), Some("user"));
            assert!(ctx.flag_bool("controller"));
            MAKE_ROUTE_EXECUTED.store(true, Ordering::SeqCst);
            Ok(())
        })
    }
}

#[tokio::test]
async fn test_kernel_command_execution() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite")
        .version("1.0.0")
        .register(GreetCommand);

    let args = vec![
        "test-cli".to_string(),
        "greet".to_string(),
        "Sayeed".to_string(),
        "--loud".to_string(),
        "--prefix=Welcome".to_string(),
    ];

    let result = kernel.handle(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_kernel_alias_resolution() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(GreetCommand);

    let args = vec![
        "test-cli".to_string(),
        "hello".to_string(),
        "Sayeed".to_string(),
    ];

    let result = kernel.handle(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_kernel_colon_subcommand_normalization() {
    MAKE_ROUTE_EXECUTED.store(false, Ordering::SeqCst);

    let kernel = Kernel::new("test-cli", "CLI Test Suite")
        .group("make", "Scaffolding commands")
        .register(MakeRouteCommand);

    // Test colon syntax
    let args_colon = vec![
        "test-cli".to_string(),
        "make:route".to_string(),
        "user".to_string(),
        "--controller".to_string(),
    ];
    assert!(kernel.handle(args_colon).await.is_ok());
    assert!(MAKE_ROUTE_EXECUTED.load(Ordering::SeqCst));

    MAKE_ROUTE_EXECUTED.store(false, Ordering::SeqCst);

    // Test space-separated subcommands (e.g. `make route`)
    let args_space = vec![
        "test-cli".to_string(),
        "make".to_string(),
        "route".to_string(),
        "user".to_string(),
        "-c".to_string(),
    ];
    assert!(kernel.handle(args_space).await.is_ok());
    assert!(MAKE_ROUTE_EXECUTED.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_kernel_fuzzy_suggestion() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(GreetCommand);

    let args = vec!["test-cli".to_string(), "gret".to_string()];

    let result = kernel.handle(args).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("Unknown command `gret`"));
    assert!(err_msg.contains("Did you mean `greet`?"));
}

#[tokio::test]
async fn test_kernel_missing_required_argument() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(GreetCommand);

    let args = vec!["test-cli".to_string(), "greet".to_string()];

    let result = kernel.handle(args).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Missing required argument `name`")
    );
}

#[tokio::test]
async fn test_kernel_help_and_version() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite")
        .version("2.5.0")
        .register(GreetCommand);

    assert!(
        kernel
            .handle(vec!["test-cli".to_string(), "--help".to_string()])
            .await
            .is_ok()
    );
    assert!(
        kernel
            .handle(vec!["test-cli".to_string(), "--version".to_string()])
            .await
            .is_ok()
    );
    assert!(
        kernel
            .handle(vec![
                "test-cli".to_string(),
                "greet".to_string(),
                "--help".to_string()
            ])
            .await
            .is_ok()
    );
}

#[test]
fn test_template_rendering_with_inflections() {
    let template = "Hello {{ name | capitalize }}! Snake: {{ name_raw | snake_case }} Plural: {{ entity | plural }}";
    let context = json!({
        "name": "developer",
        "name_raw": "UserProfile",
        "entity": "category",
    });

    let rendered = karigor::template::render(template, &context).unwrap();
    assert!(rendered.contains("Hello Developer!"));
    assert!(rendered.contains("Snake: user_profile"));
    assert!(rendered.contains("Plural: categories"));
}

#[test]
fn test_ui_table_and_method_formatting() {
    let ui = Ui::new().quiet(true);
    let mut table = ui.table(&["Method", "Route"]);
    table.add_row(vec![
        karigor::ui::cell_http_method("GET"),
        Cell::new("/users"),
    ]);

    let output = table.to_string();
    assert!(output.contains("Method"));
    assert!(output.contains("/users"));

    assert!(karigor::ui::format_http_method("GET").contains("GET"));
}

#[test]
fn test_template_publish_files() {
    let temp_dir =
        std::env::temp_dir().join(format!("karigor_publish_test_{}", std::process::id()));
    let files = &[("test1.txt", "Content 1"), ("test2.txt", "Content 2")];

    let outcomes = karigor::template::publish_files(&temp_dir, files, true).unwrap();
    assert_eq!(outcomes.len(), 2);
    assert!(temp_dir.join("test1.txt").exists());
    assert!(temp_dir.join("test2.txt").exists());

    drop(std::fs::remove_dir_all(temp_dir));
}
