use std::error::Error as _;

use karigor::prelude::*;
use serde_json::json;

#[test]
fn command_not_found_display_includes_suggestion() {
    let err = KarigorError::CommandNotFound {
        name: "gret".to_string(),
        suggestion: Some("greet".to_string()),
    };

    let message = err.to_string();
    assert!(message.contains("Unknown command `gret`"));
    assert!(message.contains("Did you mean `greet`?"));
    assert!(err.source().is_none());
}

#[test]
fn command_not_found_display_without_suggestion() {
    let err = KarigorError::CommandNotFound {
        name: "nope".to_string(),
        suggestion: None,
    };

    let message = err.to_string();
    assert!(message.contains("Unknown command `nope`"));
    assert!(!message.contains("Did you mean"));
}

#[test]
fn validation_error_displays_are_prefixed() {
    let argument = KarigorError::InvalidArgument("missing name".to_string());
    assert_eq!(argument.to_string(), "Invalid argument: missing name");

    let flag = KarigorError::InvalidFlag("missing value".to_string());
    assert_eq!(flag.to_string(), "Invalid flag: missing value");
}

#[test]
fn execution_error_exposes_inner_error_as_source() {
    let err = KarigorError::Execution(Box::new(std::io::Error::other("disk full")));

    assert!(err.to_string().contains("disk full"));
    let source = err
        .source()
        .expect("execution error must expose its source");
    assert!(source.to_string().contains("disk full"));
}

#[test]
fn template_error_exposes_minijinja_source() {
    let err = karigor::template::render("{{ unclosed ", &json!({})).unwrap_err();

    assert!(matches!(err, KarigorError::Template(_)));
    assert!(err.to_string().contains("Template error"));
    assert!(err.source().is_some());
}

#[test]
fn io_error_converts_and_exposes_source() {
    let err = KarigorError::from(std::io::Error::other("permission denied"));

    assert!(matches!(err, KarigorError::Io(_)));
    assert!(err.to_string().contains("permission denied"));
    assert!(err.source().is_some());
}

#[tokio::test]
async fn execution_source_survives_kernel_dispatch() {
    struct FailingCommand;

    impl Command for FailingCommand {
        fn name(&self) -> &'static str {
            "fail"
        }

        fn description(&self) -> &'static str {
            "Always fails"
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            let _ = ctx;
            Box::pin(async {
                Err(KarigorError::Execution(Box::new(std::io::Error::other(
                    "boom",
                ))))
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(FailingCommand);

    let err = kernel
        .handle(vec!["test-cli".to_string(), "fail".to_string()])
        .await
        .unwrap_err();

    assert!(err.to_string().contains("boom"));
    assert!(err.source().is_some());
}

#[test]
fn command_execution_accepts_boxed_errors() {
    let err = KarigorError::Execution(Box::new(std::io::Error::other("shared boom")));

    assert!(err.to_string().contains("shared boom"));
}
