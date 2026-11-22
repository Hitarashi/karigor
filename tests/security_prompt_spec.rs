use std::{io::Cursor, sync::Arc};

use karigor::{
    prelude::*,
    ui::{
        MockPromptBackend, PromptBackend, PromptHandler, TerminalPromptBackend, is_non_interactive,
        prompt_multiselect, prompt_multiselect_stream, prompt_password, prompt_password_stream,
        set_non_interactive,
    },
};

static PROMPT_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn password_prompt_stream_and_mock_execution() {
    let test_guard = PROMPT_TEST_MUTEX.lock().unwrap();

    // Test stream execution
    let mut input = Cursor::new(b"super_secret_token\n");
    let mut output = Vec::new();
    let result = prompt_password_stream(&mut input, &mut output, "Enter Secret Key")
        .expect("password stream read");
    assert_eq!(result, "super_secret_token");
    let out_str = String::from_utf8(output).unwrap();
    assert!(out_str.contains("Enter Secret Key"));

    // Test MockPromptBackend integration
    let mock = Arc::new(MockPromptBackend::new());
    mock.push_password("mocked_database_password");

    let handler = PromptHandler::mock(Arc::clone(&mock));
    let ui = Ui::new().with_prompts(handler);

    let entered = ui
        .password("Database Password:")
        .expect("mocked password returned");
    assert_eq!(entered, "mocked_database_password");

    // When mock queue is exhausted, it should error
    let missing = ui.password("Another Password:");
    assert!(missing.is_err());
    drop(test_guard);
}

#[test]
fn multiselect_prompt_stream_and_mock_execution() {
    let test_guard = PROMPT_TEST_MUTEX.lock().unwrap();

    let items = vec![
        "Authentication".to_string(),
        "Rate Limiter".to_string(),
        "Session Storage".to_string(),
        "CORS Protection".to_string(),
    ];
    let defaults = vec![0, 3];

    // Case 1: Empty input accepts defaults
    let mut input_empty = Cursor::new(b"\n");
    let mut out_empty = Vec::new();
    let selected = prompt_multiselect_stream(
        &mut input_empty,
        &mut out_empty,
        "Select Features",
        &items,
        &defaults,
    )
    .expect("multiselect with defaults");
    assert_eq!(selected, vec![0, 3]);

    // Case 2: Comma-separated user input with extra whitespace, invalid tokens, and out-of-range numbers
    let mut input_custom = Cursor::new(b"2, invalid, 3, 99\n");
    let mut out_custom = Vec::new();
    let selected_custom = prompt_multiselect_stream(
        &mut input_custom,
        &mut out_custom,
        "Select Features",
        &items,
        &defaults,
    )
    .expect("multiselect with custom choices");
    assert_eq!(selected_custom, vec![1, 2]);

    // Case 3: Empty defaults hint stream formatting
    let empty_defs: Vec<usize> = Vec::new();
    let mut input_none = Cursor::new(b"\n");
    let mut out_none = Vec::new();
    let selected_none = prompt_multiselect_stream(
        &mut input_none,
        &mut out_none,
        "Select Features",
        &items,
        &empty_defs,
    )
    .expect("multiselect empty defaults");
    assert!(selected_none.is_empty());

    // Case 4: Empty items list should error
    let no_items: Vec<String> = Vec::new();
    let mut in_err = Cursor::new(b"1\n");
    let mut out_err = Vec::new();
    assert!(
        prompt_multiselect_stream(&mut in_err, &mut out_err, "Select", &no_items, &[]).is_err()
    );

    // Case 5: MockPromptBackend integration
    let mock = Arc::new(MockPromptBackend::new());
    mock.push_multiselect(vec![1, 3]);

    let handler = PromptHandler::mock(Arc::clone(&mock));
    let ui = Ui::new().with_prompts(handler);

    let chosen = ui
        .multiselect("Pick Middleware:", &items, &defaults)
        .expect("mocked multiselect returned");
    assert_eq!(chosen, vec![1, 3]);

    // When mock queue is exhausted, it falls back to defaults
    let fallback = ui
        .multiselect("Pick Again:", &items, &defaults)
        .expect("fallback to defaults");
    assert_eq!(fallback, vec![0, 3]);
    drop(test_guard);
}

#[test]
fn non_interactive_backend_and_global_flag() {
    let test_guard = PROMPT_TEST_MUTEX.lock().unwrap();

    set_non_interactive(true);
    assert!(is_non_interactive());

    let backend = TerminalPromptBackend::new();
    // In non-interactive mode without terminal, password prompts must fail fast with Unsupported
    let res = backend.prompt_password("API Key");
    assert!(res.is_err());

    // In non-interactive mode, multiselect returns defaults
    let items = vec!["A".to_string(), "B".to_string()];
    let defaults = vec![1];
    let chosen = backend
        .prompt_multiselect("Select", &items, &defaults)
        .expect("defaults returned in headless mode");
    assert_eq!(chosen, vec![1]);

    // Empty items error
    let empty_items: Vec<String> = Vec::new();
    assert!(
        backend
            .prompt_multiselect("Empty", &empty_items, &[])
            .is_err()
    );

    // Free functions in non-interactive environment
    assert!(prompt_password("Secret").is_err());
    assert_eq!(
        prompt_multiselect("Select Items", &items, &defaults).unwrap(),
        vec![1]
    );

    set_non_interactive(false);
    assert!(!is_non_interactive());
    drop(test_guard);
}

#[test]
fn kernel_handles_global_no_interaction_flag_and_context_prompts() {
    let test_guard = PROMPT_TEST_MUTEX.lock().unwrap();

    struct PromptingCommand;
    impl Command for PromptingCommand {
        fn name(&self) -> &str {
            "test:prompt"
        }
        fn description(&self) -> &str {
            "Triggers a prompt in run"
        }
        fn run(&self, ctx: &mut Context) -> Result<(), Error> {
            assert!(is_non_interactive());
            let confirmed = ctx.confirm("Proceed?", true)?;
            assert!(confirmed);
            let selected = ctx.multiselect("Features", &["A", "B"], &[0])?;
            assert_eq!(selected, vec![0]);
            assert!(ctx.password("Key").is_err());
            Ok(())
        }
    }

    let kernel = Kernel::new("test-cli", "Testing CLI").register(PromptingCommand);

    // Test with -n
    set_non_interactive(false);
    let result = kernel.handle(vec!["test-cli".into(), "-n".into(), "test:prompt".into()]);
    assert!(result.is_ok());

    // Test with --no-interaction
    set_non_interactive(false);
    let result = kernel.handle(vec![
        "test-cli".into(),
        "--no-interaction".into(),
        "test:prompt".into(),
    ]);
    assert!(result.is_ok());

    set_non_interactive(false);
    drop(test_guard);
}
