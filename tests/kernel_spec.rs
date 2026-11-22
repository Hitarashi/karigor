use std::collections::HashMap;

use karigor::prelude::*;

struct ParseCommand;

impl Command for ParseCommand {
    fn name(&self) -> &'static str {
        "parse"
    }

    fn description(&self) -> &'static str {
        "Captures parsed arguments and flags"
    }

    fn arguments(&self) -> Vec<Argument> {
        vec![Argument::required("name", "Value to echo")]
    }

    fn flags(&self) -> Vec<Flag> {
        vec![
            Flag::boolean("loud", Some('l'), "Uppercase output"),
            Flag::string("prefix", Some('p'), "Greeting prefix", None),
            Flag::boolean("controller", Some('c'), "Also generate controller"),
        ]
    }

    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        Box::pin(async move {
            assert_eq!(ctx.arg("name"), Some("value"));
            assert_eq!(ctx.raw_args, vec!["value".to_string()]);
            Ok(())
        })
    }
}

struct DefaultsCommand;

impl Command for DefaultsCommand {
    fn name(&self) -> &'static str {
        "defaults"
    }

    fn description(&self) -> &'static str {
        "Exercises argument and flag defaults"
    }

    fn arguments(&self) -> Vec<Argument> {
        vec![Argument::optional(
            "title",
            "Honorific title",
            Some("Developer".to_string()),
        )]
    }

    fn flags(&self) -> Vec<Flag> {
        vec![
            Flag::string("prefix", None, "Greeting prefix", Some("Hello".to_string())),
            Flag::boolean("loud", None, "Uppercase output"),
        ]
    }

    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        Box::pin(async move {
            assert_eq!(ctx.arg("title"), Some("Developer"));
            assert_eq!(ctx.flag_str("prefix"), Some("Hello"));
            assert!(!ctx.flag_bool("loud"));
            Ok(())
        })
    }
}

struct AliasedColonCommand;

impl Command for AliasedColonCommand {
    fn name(&self) -> &'static str {
        "make:thing"
    }

    fn description(&self) -> &'static str {
        "Namespaced command with an alias"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["mkt"]
    }

    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        let _ = ctx;
        Box::pin(async { Ok(()) })
    }
}

struct BareCommand;

impl Command for BareCommand {
    fn name(&self) -> &'static str {
        "bare"
    }

    fn description(&self) -> &'static str {
        "Relies on every trait default"
    }

    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        let _ = ctx;
        Box::pin(async { Ok(()) })
    }
}

struct SucceedsCommand;

impl Command for SucceedsCommand {
    fn name(&self) -> &'static str {
        "duplicate"
    }

    fn description(&self) -> &'static str {
        "First registration"
    }

    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        let _ = ctx;
        Box::pin(async { Ok(()) })
    }
}

struct FailsCommand;

impl Command for FailsCommand {
    fn name(&self) -> &'static str {
        "duplicate"
    }

    fn description(&self) -> &'static str {
        "Second registration"
    }

    fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
        let _ = ctx;
        Box::pin(async { Err(KarigorError::InvalidArgument("second won".to_string())) })
    }
}

fn parse_kernel() -> Kernel {
    Kernel::new("test-cli", "CLI Test Suite").register(ParseCommand)
}

#[tokio::test]
async fn long_flag_accepts_equals_form() {
    struct EqualsCommand;

    impl Command for EqualsCommand {
        fn name(&self) -> &'static str {
            "equals"
        }

        fn description(&self) -> &'static str {
            "Checks equals form"
        }

        fn flags(&self) -> Vec<Flag> {
            vec![Flag::string("prefix", Some('p'), "Prefix", None)]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                assert_eq!(ctx.flag_str("prefix"), Some("Welcome"));
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(EqualsCommand);
    let args = vec![
        "test-cli".to_string(),
        "equals".to_string(),
        "--prefix=Welcome".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn long_flag_accepts_separate_value() {
    struct SeparateCommand;

    impl Command for SeparateCommand {
        fn name(&self) -> &'static str {
            "separate"
        }

        fn description(&self) -> &'static str {
            "Checks separate value"
        }

        fn flags(&self) -> Vec<Flag> {
            vec![Flag::string("prefix", Some('p'), "Prefix", None)]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                assert_eq!(ctx.flag_str("prefix"), Some("Hello"));
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(SeparateCommand);
    let args = vec![
        "test-cli".to_string(),
        "separate".to_string(),
        "--prefix".to_string(),
        "Hello".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn short_flag_accepts_separate_value() {
    struct ShortCommand;

    impl Command for ShortCommand {
        fn name(&self) -> &'static str {
            "short"
        }

        fn description(&self) -> &'static str {
            "Checks short value"
        }

        fn flags(&self) -> Vec<Flag> {
            vec![Flag::string("prefix", Some('p'), "Prefix", None)]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                assert_eq!(ctx.flag_str("prefix"), Some("Hi"));
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(ShortCommand);
    let args = vec![
        "test-cli".to_string(),
        "short".to_string(),
        "-p".to_string(),
        "Hi".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn clustered_short_flags_and_short_string_value() {
    struct ClusterCommand;

    impl Command for ClusterCommand {
        fn name(&self) -> &'static str {
            "cluster"
        }

        fn description(&self) -> &'static str {
            "Checks clustered shorts"
        }

        fn arguments(&self) -> Vec<Argument> {
            vec![Argument::required("name", "Value")]
        }

        fn flags(&self) -> Vec<Flag> {
            vec![
                Flag::boolean("loud", Some('l'), "Uppercase"),
                Flag::boolean("controller", Some('c'), "Controller"),
                Flag::string("prefix", Some('p'), "Prefix", None),
            ]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                assert_eq!(ctx.arg("name"), Some("value"));
                assert!(ctx.flag_bool("loud"));
                assert!(ctx.flag_bool("controller"));
                assert_eq!(ctx.flag_str("prefix"), Some("Hey"));
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(ClusterCommand);
    let args = vec![
        "test-cli".to_string(),
        "cluster".to_string(),
        "value".to_string(),
        "-lc".to_string(),
        "-p".to_string(),
        "Hey".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn boolean_flag_accepts_explicit_values() {
    struct ExpectFalseCommand;

    impl Command for ExpectFalseCommand {
        fn name(&self) -> &'static str {
            "expect-false"
        }

        fn description(&self) -> &'static str {
            "Expects loud to be false"
        }

        fn flags(&self) -> Vec<Flag> {
            vec![Flag::boolean("loud", None, "Uppercase")]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                if ctx.flag_bool("loud") {
                    return Err(KarigorError::InvalidArgument(
                        "expected loud to be false".to_string(),
                    ));
                }
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(ExpectFalseCommand);
    let args = vec![
        "test-cli".to_string(),
        "expect-false".to_string(),
        "--loud=false".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn malformed_boolean_value_defaults_to_true() {
    struct ExpectTrueCommand;

    impl Command for ExpectTrueCommand {
        fn name(&self) -> &'static str {
            "expect-true"
        }

        fn description(&self) -> &'static str {
            "Expects unparsable boolean to fall back to true"
        }

        fn flags(&self) -> Vec<Flag> {
            vec![Flag::boolean("loud", None, "Uppercase")]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                if !ctx.flag_bool("loud") {
                    return Err(KarigorError::InvalidArgument(
                        "expected loud to be true".to_string(),
                    ));
                }
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(ExpectTrueCommand);
    let args = vec![
        "test-cli".to_string(),
        "expect-true".to_string(),
        "--loud=garbage".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn unknown_long_flags_are_captured() {
    struct UnknownCommand;

    impl Command for UnknownCommand {
        fn name(&self) -> &'static str {
            "unknown"
        }

        fn description(&self) -> &'static str {
            "Accepts undeclared flags"
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                assert!(ctx.flag_bool("wat"));
                assert_eq!(ctx.flag_str("other"), Some("x"));
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(UnknownCommand);
    let args = vec![
        "test-cli".to_string(),
        "unknown".to_string(),
        "--wat".to_string(),
        "--other=x".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn unknown_short_flags_are_ignored() {
    struct IgnoreShortCommand;

    impl Command for IgnoreShortCommand {
        fn name(&self) -> &'static str {
            "ignoreshort"
        }

        fn description(&self) -> &'static str {
            "Ignores undeclared shorts"
        }

        fn flags(&self) -> Vec<Flag> {
            vec![Flag::boolean("loud", Some('l'), "Uppercase")]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                assert!(ctx.flag_bool("loud"));
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(IgnoreShortCommand);
    let args = vec![
        "test-cli".to_string(),
        "ignoreshort".to_string(),
        "-zl".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn missing_long_string_value_is_rejected() {
    let kernel = parse_kernel();
    let args = vec![
        "test-cli".to_string(),
        "parse".to_string(),
        "value".to_string(),
        "--prefix".to_string(),
    ];

    let err = kernel.handle(args).await.unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid flag: Option `--prefix` requires a value"
    );
}

#[tokio::test]
async fn missing_short_string_value_is_rejected() {
    let kernel = parse_kernel();
    let args = vec![
        "test-cli".to_string(),
        "parse".to_string(),
        "value".to_string(),
        "-p".to_string(),
    ];

    let err = kernel.handle(args).await.unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid flag: Option `-p` requires a value"
    );
}

#[tokio::test]
async fn string_flag_before_another_flag_falls_back_to_default() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(DefaultsCommand);
    let args = vec![
        "test-cli".to_string(),
        "defaults".to_string(),
        "--prefix".to_string(),
        "--loud".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn omitted_arguments_and_flags_receive_defaults() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(DefaultsCommand);
    let args = vec!["test-cli".to_string(), "defaults".to_string()];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn extra_positional_arguments_are_kept_in_raw_args() {
    struct OneArgCommand;

    impl Command for OneArgCommand {
        fn name(&self) -> &'static str {
            "one"
        }

        fn description(&self) -> &'static str {
            "Declares one argument"
        }

        fn arguments(&self) -> Vec<Argument> {
            vec![Argument::required("name", "Only argument")]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                assert_eq!(ctx.arg("name"), Some("first"));
                assert_eq!(ctx.raw_args.len(), 2);
                assert_eq!(ctx.raw_args[1], "second");
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(OneArgCommand);
    let args = vec![
        "test-cli".to_string(),
        "one".to_string(),
        "first".to_string(),
        "second".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn positional_arguments_may_follow_flags() {
    let kernel = parse_kernel();
    let args = vec![
        "test-cli".to_string(),
        "parse".to_string(),
        "--loud".to_string(),
        "value".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn colon_command_alias_resolves() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(AliasedColonCommand);
    let args = vec!["test-cli".to_string(), "mkt".to_string()];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn space_separated_subcommand_normalizes_to_colon_form() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite")
        .group("make", "Scaffolding commands")
        .register(AliasedColonCommand);

    let normalized = kernel.normalize_args(&[
        "test-cli".to_string(),
        "make".to_string(),
        "thing".to_string(),
    ]);
    assert_eq!(normalized, vec!["make:thing".to_string()]);

    let args = vec![
        "test-cli".to_string(),
        "make".to_string(),
        "thing".to_string(),
    ];
    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn unregistered_space_pair_is_not_normalized() {
    struct MakeOnlyCommand;

    impl Command for MakeOnlyCommand {
        fn name(&self) -> &'static str {
            "make"
        }

        fn description(&self) -> &'static str {
            "Root make command"
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            let _ = ctx;
            Box::pin(async { Ok(()) })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(MakeOnlyCommand);
    let normalized = kernel.normalize_args(&[
        "test-cli".to_string(),
        "make".to_string(),
        "route".to_string(),
    ]);

    assert_eq!(normalized, vec!["make".to_string(), "route".to_string()]);
}

#[tokio::test]
async fn normalize_args_strips_program_and_cargo_subcommand() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite");

    assert!(kernel.normalize_args(&[]).is_empty());
    assert_eq!(
        kernel.normalize_args(&["test-cli".to_string(), "greet".to_string()]),
        vec!["greet".to_string()]
    );
    assert_eq!(
        kernel.normalize_args(&[
            "cargo".to_string(),
            "test-cli".to_string(),
            "greet".to_string(),
        ]),
        vec!["greet".to_string()]
    );
    assert_eq!(
        kernel.normalize_args(&["/usr/local/bin/prog".to_string(), "greet".to_string()]),
        vec!["greet".to_string()]
    );
}

#[tokio::test]
async fn help_and_version_paths_return_ok() {
    let kernel = parse_kernel();

    for token in ["-h", "--help", "help", "-V", "--version", "version"] {
        let args = vec!["test-cli".to_string(), token.to_string()];
        assert!(kernel.handle(args).await.is_ok(), "token {token} failed");
    }

    assert!(kernel.handle(Vec::new()).await.is_ok());
}

#[tokio::test]
async fn command_help_takes_precedence_over_validation() {
    let kernel = parse_kernel();
    let args = vec![
        "test-cli".to_string(),
        "parse".to_string(),
        "--help".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[test]
fn print_help_lists_groups_and_root_commands() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite")
        .group("make", "Scaffolding commands")
        .register(BareCommand)
        .register(AliasedColonCommand);

    kernel.print_help();
}

#[test]
fn print_command_help_lists_arguments_and_options() {
    let kernel = parse_kernel();

    kernel.print_command_help(&ParseCommand);
    kernel.print_command_help(&BareCommand);
}

#[tokio::test]
async fn fuzzy_suggestion_respects_distance_threshold() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(BareCommand);

    let close = kernel
        .handle(vec!["test-cli".to_string(), "baer".to_string()])
        .await
        .unwrap_err();
    assert!(close.to_string().contains("Did you mean `bare`?"));

    let far = kernel
        .handle(vec!["test-cli".to_string(), "zzzzzz".to_string()])
        .await
        .unwrap_err();
    assert!(far.to_string().contains("Unknown command `zzzzzz`"));
    assert!(!far.to_string().contains("Did you mean"));
}

#[tokio::test]
async fn later_registration_wins_for_duplicate_names() {
    let kernel = Kernel::new("test-cli", "CLI Test Suite")
        .register(SucceedsCommand)
        .register(FailsCommand);

    let err = kernel
        .handle(vec!["test-cli".to_string(), "duplicate".to_string()])
        .await
        .unwrap_err();

    assert!(err.to_string().contains("second won"));
}

#[tokio::test]
async fn command_errors_propagate_out_of_handle() {
    struct ErrorCommand;

    impl Command for ErrorCommand {
        fn name(&self) -> &'static str {
            "error"
        }

        fn description(&self) -> &'static str {
            "Always errors"
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            let _ = ctx;
            Box::pin(async { Err(KarigorError::InvalidFlag("bad".to_string())) })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(ErrorCommand);
    let err = kernel
        .handle(vec!["test-cli".to_string(), "error".to_string()])
        .await
        .unwrap_err();

    assert_eq!(err.to_string(), "Invalid flag: bad");
}

#[test]
fn context_accessors_report_missing_and_mismatched_values() {
    let context = Context {
        args: HashMap::from([("name".to_string(), "Sayeed".to_string())]),
        arg_lists: HashMap::new(),
        flags: HashMap::from([
            ("loud".to_string(), FlagValue::Bool(true)),
            ("prefix".to_string(), FlagValue::Str("Hi".to_string())),
        ]),
        raw_args: vec!["Sayeed".to_string()],
        ui: Ui::new(),
    };

    assert_eq!(context.arg("name"), Some("Sayeed"));
    assert_eq!(context.arg("missing"), None);
    assert!(context.flag_bool("loud"));
    assert!(!context.flag_bool("prefix"));
    assert_eq!(context.flag_str("prefix"), Some("Hi"));
    assert_eq!(context.flag_str("loud"), None);
    assert_eq!(context.flag_str("missing"), None);
}
#[tokio::test]
async fn short_string_flag_before_another_flag_falls_back_to_default() {
    struct ShortDefaultCommand;

    impl Command for ShortDefaultCommand {
        fn name(&self) -> &'static str {
            "short-default"
        }

        fn description(&self) -> &'static str {
            "Declares a short string flag with a default"
        }

        fn flags(&self) -> Vec<Flag> {
            vec![
                Flag::string(
                    "prefix",
                    Some('p'),
                    "Greeting prefix",
                    Some("Hello".to_string()),
                ),
                Flag::boolean("loud", Some('l'), "Uppercase output"),
            ]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            Box::pin(async move {
                assert_eq!(ctx.flag_str("prefix"), Some("Hello"));
                assert!(!ctx.flag_bool("loud"));
                Ok(())
            })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(ShortDefaultCommand);
    let args = vec![
        "test-cli".to_string(),
        "short-default".to_string(),
        "-p".to_string(),
        "-l".to_string(),
    ];

    assert!(kernel.handle(args).await.is_ok());
}

#[tokio::test]
async fn missing_required_argument_is_rejected() {
    struct RequiredArgCommand;

    impl Command for RequiredArgCommand {
        fn name(&self) -> &'static str {
            "required"
        }

        fn description(&self) -> &'static str {
            "Declares one required argument"
        }

        fn arguments(&self) -> Vec<Argument> {
            vec![Argument::required("name", "Name to greet")]
        }

        fn run<'a>(&'a self, ctx: &'a mut Context) -> CommandFuture<'a> {
            let _ = ctx;
            Box::pin(async { Ok(()) })
        }
    }

    let kernel = Kernel::new("test-cli", "CLI Test Suite").register(RequiredArgCommand);
    let err = kernel
        .handle(vec!["test-cli".to_string(), "required".to_string()])
        .await
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "Invalid argument: Missing required argument `name`"
    );
}

#[tokio::test]
async fn handle_env_delegates_to_the_process_arguments() {
    // The test harness owns the real process arguments, so the only stable
    // expectation is that the delegation completes and returns a result.
    let kernel = parse_kernel();
    let _ = kernel.handle_env().await;
}
