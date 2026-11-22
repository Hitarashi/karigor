use karigor::prelude::*;

struct AdvancedCommand;

impl Command for AdvancedCommand {
    fn name(&self) -> &'static str {
        "process"
    }

    fn description(&self) -> &'static str {
        "Test advanced argument and flag features"
    }

    fn arguments(&self) -> Vec<Argument> {
        vec![
            Argument::required("target", "Target entity"),
            Argument::variadic("files", "Trailing source files to process"),
        ]
    }

    fn flags(&self) -> Vec<Flag> {
        vec![
            Flag::array("include", Some('i'), "Include patterns"),
            Flag::string(
                "format",
                Some('f'),
                "Output format",
                Some("table".to_string()),
            )
            .choices(["table", "json", "csv"]),
            Flag::boolean("cache", None, "Enable caching"),
            Flag::string("token", None, "Auth token", None).required(),
        ]
    }

    fn run(&self, ctx: &mut Context) -> Result<(), Error> {
        let target = ctx.arg("target").unwrap();
        let files = ctx.arg_list("files");
        let includes = ctx.flag_array("include");
        let format = ctx.flag_str("format").unwrap();
        let cache = ctx.flag_bool("cache");
        let token = ctx.flag_str("token").unwrap();

        // Also test missing keys
        assert_eq!(ctx.arg_list("missing_args"), Vec::<&str>::new());
        assert_eq!(ctx.flag_array("missing_flags"), Vec::<&str>::new());

        ctx.ui.info(&format!("Target: {target}"));
        ctx.ui.info(&format!("Files: {}", files.join(",")));
        ctx.ui.info(&format!("Includes: {}", includes.join(",")));
        ctx.ui.info(&format!("Format: {format}"));
        ctx.ui.info(&format!("Cache: {cache}"));
        ctx.ui.info(&format!("Token: {token}"));
        ctx.ui.success("Process completed");
        Ok(())
    }
}

#[test]
fn advanced_flags_and_variadic_arguments_success() {
    let ui = Ui::buffered();
    let kernel = Kernel::new("test-app", "Testing Suite")
        .with_ui(ui.clone())
        .register(AdvancedCommand);

    assert_eq!(kernel.ui().output_string(), Some(String::new()));

    let args = vec![
        "test-app".to_string(),
        "process".to_string(),
        "build".to_string(),
        "src/a.rs".to_string(),
        "src/b.rs".to_string(),
        "--token".to_string(),
        "secret123".to_string(),
        "--include".to_string(),
        "tests/*".to_string(),
        "-i".to_string(),
        "benches/*".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--cache".to_string(),
    ];

    kernel.handle(args).unwrap();

    let output = ui.output_string().expect("buffer must be present");
    assert!(output.contains("Target: build"));
    assert!(output.contains("Files: src/a.rs,src/b.rs"));
    assert!(output.contains("Includes: tests/*,benches/*"));
    assert!(output.contains("Format: json"));
    assert!(output.contains("Cache: true"));
    assert!(output.contains("Token: secret123"));
    assert!(output.contains("Process completed"));
}

#[test]
fn array_flags_with_equals_syntax() {
    let ui = Ui::buffered();
    let kernel = Kernel::new("test-app", "Testing Suite")
        .with_ui(ui.clone())
        .register(AdvancedCommand);

    let args = vec![
        "test-app".to_string(),
        "process".to_string(),
        "build".to_string(),
        "--token".to_string(),
        "secret".to_string(),
        "--include=one".to_string(),
        "--include=two".to_string(),
    ];

    kernel.handle(args).unwrap();

    let output = ui.output_string().expect("buffer must be present");
    assert!(output.contains("Includes: one,two"));
}

#[test]
fn negatable_boolean_flags() {
    let ui = Ui::buffered();
    let kernel = Kernel::new("test-app", "Testing Suite")
        .with_ui(ui.clone())
        .register(AdvancedCommand);

    let args = vec![
        "test-app".to_string(),
        "process".to_string(),
        "build".to_string(),
        "--token".to_string(),
        "abc".to_string(),
        "--no-cache".to_string(),
    ];

    kernel.handle(args).unwrap();

    let output = ui.output_string().expect("buffer must be present");
    assert!(output.contains("Cache: false"));
}

#[test]
fn invalid_choice_flag_is_rejected() {
    let kernel = Kernel::new("test-app", "Testing Suite").register(AdvancedCommand);

    let args = vec![
        "test-app".to_string(),
        "process".to_string(),
        "build".to_string(),
        "--token".to_string(),
        "abc".to_string(),
        "--format".to_string(),
        "xml".to_string(),
    ];

    let err = kernel.handle(args).unwrap_err();
    assert!(
        err.to_string()
            .contains("Invalid value `xml` for option `--format`")
    );
    assert!(
        err.to_string()
            .contains("Allowed values: [table, json, csv]")
    );
}

#[test]
fn missing_required_flag_is_rejected() {
    let kernel = Kernel::new("test-app", "Testing Suite").register(AdvancedCommand);

    let args = vec![
        "test-app".to_string(),
        "process".to_string(),
        "build".to_string(),
    ];

    let err = kernel.handle(args).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid flag: Missing required option `--token`"
    );
}

#[test]
fn missing_array_flag_value_is_rejected() {
    let kernel = Kernel::new("test-app", "Testing Suite").register(AdvancedCommand);

    let args = vec![
        "test-app".to_string(),
        "process".to_string(),
        "build".to_string(),
        "--token".to_string(),
        "abc".to_string(),
        "--include".to_string(),
    ];

    let err = kernel.handle(args).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid flag: Option `--include` requires a value"
    );

    let short_args = vec![
        "test-app".to_string(),
        "process".to_string(),
        "build".to_string(),
        "--token".to_string(),
        "abc".to_string(),
        "-i".to_string(),
    ];

    let short_err = kernel.handle(short_args).unwrap_err();
    assert_eq!(
        short_err.to_string(),
        "Invalid flag: Option `-i` requires a value"
    );
}

#[test]
fn short_array_flags_and_empty_variadic_args() {
    let ui = Ui::buffered();
    let kernel = Kernel::new("test-app", "Testing Suite")
        .with_ui(ui.clone())
        .register(AdvancedCommand);

    let args = vec![
        "test-app".to_string(),
        "process".to_string(),
        "build".to_string(),
        "--token".to_string(),
        "secret".to_string(),
        "-i".to_string(),
        "first".to_string(),
        "-i".to_string(),
        "second".to_string(),
    ];

    kernel.handle(args).unwrap();

    let output = ui.output_string().expect("buffer must be present");
    assert!(output.contains("Includes: first,second"));
    assert!(output.contains("Files: "));
}

#[test]
fn global_quiet_flag_suppresses_output() {
    let ui = Ui::buffered();
    let kernel = Kernel::new("test-app", "Testing Suite")
        .with_ui(ui.clone())
        .register(AdvancedCommand);

    let args = vec![
        "test-app".to_string(),
        "-q".to_string(),
        "process".to_string(),
        "build".to_string(),
        "--token".to_string(),
        "secret".to_string(),
    ];

    kernel.handle(args).unwrap();

    let output = ui.output_string().expect("buffer must be present");
    assert_eq!(output, "");
}

#[test]
fn global_no_ansi_flag_disables_color() {
    let ui = Ui::buffered();
    let kernel = Kernel::new("test-app", "Testing Suite")
        .with_ui(ui.clone())
        .register(AdvancedCommand);

    let args = vec![
        "test-app".to_string(),
        "--no-ansi".to_string(),
        "process".to_string(),
        "build".to_string(),
        "--token".to_string(),
        "secret".to_string(),
    ];

    kernel.handle(args).unwrap();
    let output = ui.output_string().expect("buffer must be present");
    assert!(output.contains("Target: build"));
}

#[test]
fn space_separated_repeated_array_flags() {
    let ui = Ui::buffered();
    let kernel = Kernel::new("test-app", "Testing Suite")
        .with_ui(ui.clone())
        .register(AdvancedCommand);

    let args = vec![
        "/usr/local/bin/test-app".to_string(),
        "process".to_string(),
        "build".to_string(),
        "--token".to_string(),
        "secret".to_string(),
        "--include".to_string(),
        "one".to_string(),
        "--include".to_string(),
        "two".to_string(),
        "--no-unknown-option".to_string(),
    ];

    kernel.handle(args).unwrap();
    let output = ui.output_string().expect("buffer must be present");
    assert!(output.contains("Includes: one,two"));
}

#[test]
fn variadic_help_and_flag_array_fallback() {
    let ui = Ui::buffered();
    let kernel = Kernel::new("test-app", "Testing Suite")
        .with_ui(ui.clone())
        .register(AdvancedCommand);

    let help_args = vec![
        "test-app".to_string(),
        "process".to_string(),
        "--help".to_string(),
    ];
    kernel.handle(help_args).unwrap();
    let output = ui.output_string().expect("buffer must be present");
    assert!(output.contains("[files...]"));
    assert!(output.contains("variadic"));

    // Context flag_array on a string flag
    let mut flags = std::collections::HashMap::new();
    flags.insert(
        "single".to_string(),
        FlagValue::Str("scalar_value".to_string()),
    );
    let ctx = Context {
        args: std::collections::HashMap::new(),
        arg_lists: std::collections::HashMap::new(),
        flags,
        raw_args: Vec::new(),
        ui: Ui::buffered(),
    };
    assert_eq!(ctx.flag_array("single"), vec!["scalar_value"]);
    assert_eq!(ctx.flag_array("missing"), Vec::<&str>::new());
}
