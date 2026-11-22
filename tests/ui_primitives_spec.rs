use std::sync::Arc;

use karigor::{
    prelude::*,
    ui::{
        Alignment, Color, ContentArrangement, MockPromptBackend, PromptBackend, PromptHandler,
        TablePreset, TerminalPromptBackend, UiWriter, presets, reset_color_override,
        set_color_enabled, strip_ansi, visible_width,
    },
};

static COLOR_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn ansi_color_and_style_generation() {
    let guard = COLOR_TEST_LOCK.lock().unwrap();
    set_color_enabled(true);

    let red_text = "error".red().bold().to_string();
    assert!(red_text.contains("\x1b[1;31merror\x1b[0m"));

    let on_blue = "badge".black().on_blue().to_string();
    assert!(on_blue.contains("\x1b[30;44mbadge\x1b[0m"));

    let dim_italic = "subtext".dimmed().italic().underline().to_string();
    assert!(dim_italic.contains("\x1b[2;3;4msubtext\x1b[0m"));

    let rgb = "truecolor"
        .color(Color::Rgb(10, 20, 30))
        .on_color(Color::Rgb(40, 50, 60))
        .to_string();
    assert!(rgb.contains("38;2;10;20;30"));
    assert!(rgb.contains("48;2;40;50;60"));

    let ansi256 = "indexed"
        .color(Color::Ansi256(123))
        .on_color(Color::Ansi256(234))
        .to_string();
    assert!(ansi256.contains("38;5;123"));
    assert!(ansi256.contains("48;5;234"));

    let default_col = "def"
        .color(Color::Default)
        .on_color(Color::Default)
        .to_string();
    assert!(default_col.contains("39;49"));

    let brights = "bright"
        .bright_red()
        .bright_green()
        .bright_yellow()
        .bright_blue()
        .bright_magenta()
        .bright_cyan()
        .bright_white()
        .on_bright_red()
        .on_bright_green()
        .on_bright_yellow()
        .on_bright_blue()
        .on_bright_magenta()
        .on_bright_cyan()
        .on_bright_white();
    assert!(!brights.to_string().is_empty());

    let styled = "full"
        .green()
        .yellow()
        .magenta()
        .cyan()
        .white()
        .bright_black()
        .on_black()
        .on_red()
        .on_green()
        .on_yellow()
        .on_magenta()
        .on_cyan()
        .on_white()
        .on_bright_black()
        .reverse()
        .hidden()
        .strikethrough();
    assert_eq!(styled.as_str(), "full");
    assert!(!styled.to_string().is_empty());

    set_color_enabled(false);
    let plain = "error".red().bold().to_string();
    assert_eq!(plain, "error");

    reset_color_override();
    drop(guard);
}

#[test]
fn strip_ansi_and_visible_width_calculation() {
    let styled = "\x1b[1;31mHello\x1b[0m \x1b[32mWorld\x1b[0m";
    assert_eq!(strip_ansi(styled), "Hello World");
    assert_eq!(visible_width(styled), 11);

    let cjk = "করিগর";
    assert_eq!(strip_ansi(cjk), "করিগর");
    assert_eq!(visible_width(cjk), 5);

    let ctrl = "\tHello\r\n";
    assert_eq!(visible_width(ctrl), 5);
}

#[test]
fn table_alignments_and_multiline_cells() {
    let mut table = Table::new();
    table.load_style(TablePreset::Rounded);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header([
        Cell::new("Name")
            .align(Alignment::Left)
            .add_attribute(Attribute::Dim),
        Cell::new("Count")
            .align(Alignment::Right)
            .add_attribute(Attribute::Italic),
        Cell::new("Status")
            .align(Alignment::Center)
            .add_attribute(Attribute::Underline),
    ]);

    table.add_row([
        Cell::new("First\nItem").bg(Color::BrightBlack),
        Cell::new("42").align(Alignment::Right),
        Cell::new("OK").align(Alignment::Center),
    ]);

    let rendered = table.to_string();
    assert!(rendered.contains("╭"));
    assert!(rendered.contains("╰"));
    assert!(rendered.contains("Name"));
    assert!(rendered.contains("First"));
    assert!(rendered.contains("Item"));
    assert!(rendered.contains("42"));
    assert!(rendered.contains("OK"));

    let empty_table = Table::new();
    assert_eq!(empty_table.to_string(), "");

    let preset = presets::UTF8_FULL.with_rounded_corners();
    assert_eq!(preset, TablePreset::Rounded);
}

#[test]
fn table_presets_plain_and_compact() {
    let mut plain_table = Table::new();
    plain_table.load_style(TablePreset::Plain);
    plain_table.set_header(["Col1", "Col2"]);
    plain_table.add_row(["Val1", "Val2"]);
    let plain_out = plain_table.to_string();
    assert!(!plain_out.contains('╭'));
    assert!(plain_out.contains("Val1"));

    let mut compact_table = Table::new();
    compact_table.load_style(TablePreset::Compact);
    compact_table.set_header(["Col1", "Col2"]);
    compact_table.add_row(["Val1", "Val2"]);
    let compact_out = compact_table.to_string();
    assert!(compact_out.contains('|'));
    assert!(compact_out.contains("Val1"));
}

#[test]
fn buffered_ui_captures_all_primitives() {
    let ui = Ui::buffered();
    ui.banner("Test Suite", "Automated Runner");
    ui.section("Initialization");
    ui.info("Loading configuration");
    ui.success("Configuration parsed");
    ui.warn("Deprecation warning");
    ui.error("Fatal error test");

    let captured = strip_ansi(&ui.output_string().expect("buffer must exist"));
    assert!(captured.contains("Test Suite"));
    assert!(captured.contains("Automated Runner"));
    assert!(captured.contains("Initialization"));
    assert!(captured.contains("Loading configuration"));
    assert!(captured.contains("Configuration parsed"));
    assert!(captured.contains("Deprecation warning"));
    assert!(captured.contains("Fatal error test"));
}

#[test]
fn action_logging_lifecycle() {
    let ui = Ui::buffered();

    ui.action("Database schema migration").succeeded();
    ui.action("Cache directory warmup")
        .skipped("no cache configured");
    ui.action("External webhook ping")
        .failed("connection refused");

    let captured = strip_ansi(&ui.output_string().expect("buffer must exist"));
    assert!(captured.contains("DONE"));
    assert!(captured.contains("Database schema migration"));
    assert!(captured.contains("SKIPPED"));
    assert!(captured.contains("Cache directory warmup"));
    assert!(captured.contains("FAILED"));
    assert!(captured.contains("External webhook ping"));
}

#[test]
fn steps_and_instruction_formatting() {
    let ui = Ui::buffered();

    let mut steps = ui.steps();
    steps.add(
        "Clone repository",
        Some("git clone https://example.com/repo"),
    );
    steps.step("Start server", "cargo dev");
    steps.add("Run setup", None::<String>);
    ui.render_steps(&steps);
    ui.print("Direct print line");

    let mut instruction = ui.instruction("Deployment Checklist");
    instruction.add("Verify production credentials");
    instruction.add("Run database backup");
    instruction.render(&ui);

    let empty_inst = Instruction::new("Empty");
    assert!(empty_inst.prepare().contains("Empty"));

    let mut dl = ui.definition_list();
    dl.add("Host", "127.0.0.1");
    dl.add("Port", "5555");
    dl.render(&ui);

    let captured = strip_ansi(&ui.output_string().expect("buffer must exist"));
    assert!(captured.contains("1. Clone repository"));
    assert!(captured.contains("2. Start server"));
    assert!(captured.contains("3. Run setup"));
    assert!(captured.contains("Direct print line"));
    assert!(captured.contains("Deployment Checklist"));
    assert!(captured.contains("Verify production credentials"));
    assert!(captured.contains("Host"));
    assert!(captured.contains("127.0.0.1"));
    assert!(captured.contains("Port"));
    assert!(captured.contains("5555"));
}

#[test]
fn mock_prompt_backend_interaction() {
    let mock = Arc::new(MockPromptBackend::new());
    mock.push_input("production_db");
    mock.push_confirm(true);
    mock.push_select(1);

    let ui = Ui::buffered().with_prompts(PromptHandler::mock(Arc::clone(&mock)));

    let input_res = ui.input("Enter database name", Some("dev_db")).unwrap();
    assert_eq!(input_res, "production_db");

    let confirm_res = ui.confirm("Proceed with migration?", false).unwrap();
    assert!(confirm_res);

    let select_res = ui
        .select("Choose environment", &["dev", "staging", "prod"], 0)
        .unwrap();
    assert_eq!(select_res, 1);

    // Test fallback when mock queues are exhausted
    let fallback_input = ui
        .input("Enter another name", Some("fallback_val"))
        .unwrap();
    assert_eq!(fallback_input, "fallback_val");

    let fallback_confirm = ui.confirm("Continue?", true).unwrap();
    assert!(fallback_confirm);

    let fallback_select = ui.select("Select", &["a", "b"], 0).unwrap();
    assert_eq!(fallback_select, 0);

    let err_input = ui.input("No default", None);
    assert!(err_input.is_err());

    // Debug representation
    let debug_str = format!("{ui:?}");
    assert!(debug_str.contains("Ui"));

    // Free prompt helper functions (using global non-interactive override)
    karigor::ui::set_non_interactive(true);
    assert!(karigor::ui::is_non_interactive());
    assert_eq!(
        karigor::ui::prompt_input("Free", Some("free_val")).unwrap(),
        "free_val"
    );
    assert!(karigor::ui::prompt_confirm("FreeConfirm", true).unwrap());
    assert_eq!(
        karigor::ui::prompt_select("FreeSelect", &["opt1", "opt2"], 0).unwrap(),
        0
    );
    karigor::ui::set_non_interactive(false);
}

#[test]
fn terminal_prompt_backend_non_interactive_fallbacks() {
    let backend = TerminalPromptBackend::non_interactive();

    // prompt_input with default returns default
    assert_eq!(
        backend.prompt_input("Test", Some("default_value")).unwrap(),
        "default_value"
    );

    // prompt_input without default returns error
    assert!(backend.prompt_input("Required", None).is_err());

    // prompt_confirm returns default
    assert!(backend.prompt_confirm("Confirm", true).unwrap());
    assert!(!backend.prompt_confirm("Confirm", false).unwrap());

    // prompt_select with items returns default
    assert_eq!(
        backend
            .prompt_select("Select", &["opt1".into(), "opt2".into()], 1)
            .unwrap(),
        1
    );

    // prompt_select out of bounds returns error
    assert!(
        backend
            .prompt_select("Select", &["opt1".into()], 10)
            .is_err()
    );

    // prompt_select on empty list returns error
    assert!(backend.prompt_select("Select", &[], 0).is_err());
}

#[test]
fn spinner_api_methods() {
    let style = ProgressStyle::default_spinner()
        .tick_chars("+-*")
        .template("{spinner} {msg}")
        .expect("template must be valid");
    let mut spinner = ProgressBar::new_spinner();
    spinner.set_style(style);
    spinner.set_message("Processing items");
    spinner.tick();
    assert!(spinner.elapsed().as_nanos() > 0);
    spinner.finish_with_message("Done");
    spinner.finish_and_clear();

    // Active (non-hidden) execution coverage
    let active_spinner = ProgressBar::new_spinner().with_hidden(false);
    assert!(!active_spinner.is_hidden());
    active_spinner.tick();
    active_spinner.enable_steady_tick(std::time::Duration::from_millis(5));
    std::thread::sleep(std::time::Duration::from_millis(15));
    active_spinner.finish_with_message("Completed active");
    active_spinner.finish_and_clear();
}

#[test]
fn stream_prompts_comprehensive_coverage() {
    use std::io::Cursor;

    use karigor::ui::{prompt_confirm_stream, prompt_input_stream, prompt_select_stream};

    // Text input
    let mut input = Cursor::new(b"User typed\n\n\n");
    let mut output = Vec::new();

    let val1 = prompt_input_stream(&mut input, &mut output, "Name", None).unwrap();
    assert_eq!(val1, "User typed");

    let val2 =
        prompt_input_stream(&mut input, &mut output, "Fallback", Some("default_val")).unwrap();
    assert_eq!(val2, "default_val");

    let val3 = prompt_input_stream(&mut input, &mut output, "Empty", None).unwrap();
    assert_eq!(val3, "");

    // Confirmations
    let mut confirm_input = Cursor::new(b"y\nYES\n1\nn\nfalse\n\nother\n");
    let mut confirm_output = Vec::new();

    assert!(prompt_confirm_stream(&mut confirm_input, &mut confirm_output, "Q1", false).unwrap());
    assert!(prompt_confirm_stream(&mut confirm_input, &mut confirm_output, "Q2", false).unwrap());
    assert!(prompt_confirm_stream(&mut confirm_input, &mut confirm_output, "Q3", false).unwrap());
    assert!(!prompt_confirm_stream(&mut confirm_input, &mut confirm_output, "Q4", true).unwrap());
    assert!(!prompt_confirm_stream(&mut confirm_input, &mut confirm_output, "Q5", true).unwrap());
    assert!(prompt_confirm_stream(&mut confirm_input, &mut confirm_output, "Q6", true).unwrap());
    assert!(prompt_confirm_stream(&mut confirm_input, &mut confirm_output, "Q7", true).unwrap());

    // Selections
    let items = vec![
        "First".to_string(),
        "Second".to_string(),
        "Third".to_string(),
    ];
    let mut select_input = Cursor::new(b"2\n\n99\ninvalid\n");
    let mut select_output = Vec::new();

    let sel1 =
        prompt_select_stream(&mut select_input, &mut select_output, "Pick", &items, 0).unwrap();
    assert_eq!(sel1, 1);

    let sel2 = prompt_select_stream(
        &mut select_input,
        &mut select_output,
        "Pick default",
        &items,
        2,
    )
    .unwrap();
    assert_eq!(sel2, 2);

    let sel3 = prompt_select_stream(
        &mut select_input,
        &mut select_output,
        "Pick out of range",
        &items,
        0,
    )
    .unwrap();
    assert_eq!(sel3, 0);

    let sel4 = prompt_select_stream(
        &mut select_input,
        &mut select_output,
        "Pick parse err",
        &items,
        1,
    )
    .unwrap();
    assert_eq!(sel4, 1);

    assert!(prompt_select_stream(&mut select_input, &mut select_output, "Empty", &[], 0).is_err());
}

#[test]
fn table_jagged_rows_and_dividers() {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Col 1"),
        Cell::new("Col 2"),
        Cell::new("Col 3"),
    ]);
    table.add_row(vec![Cell::new("Single cell row")]);
    table.add_row(vec![Cell::new("A"), Cell::new("B"), Cell::new("C")]);

    let rendered = table.to_string();
    assert!(rendered.contains("Single cell row"));
    assert!(rendered.contains("Col 3"));
}

#[test]
fn writer_console_and_buffer_error_streams() {
    let (buf_writer, buffer) = UiWriter::buffer();
    buf_writer.print_err("buffered error message");
    let content = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    assert!(content.contains("buffered error message"));

    UiWriter::Console.print_err("console error test");
}

#[test]
fn colorize_all_ansi_methods() {
    let guard = COLOR_TEST_LOCK.lock().unwrap();
    set_color_enabled(true);

    let text = "sample";
    let _ = text.black();
    let _ = text.red();
    let _ = text.green();
    let _ = text.yellow();
    let _ = text.blue();
    let _ = text.magenta();
    let _ = text.cyan();
    let _ = text.white();
    let _ = text.bright_black();
    let _ = text.bright_red();
    let _ = text.bright_green();
    let _ = text.bright_yellow();
    let _ = text.bright_blue();
    let _ = text.bright_magenta();
    let _ = text.bright_cyan();
    let _ = text.bright_white();

    let _ = text.on_black();
    let _ = text.on_red();
    let _ = text.on_green();
    let _ = text.on_yellow();
    let _ = text.on_blue();
    let _ = text.on_magenta();
    let _ = text.on_cyan();
    let _ = text.on_white();
    let _ = text.on_bright_black();
    let _ = text.on_bright_red();
    let _ = text.on_bright_green();
    let _ = text.on_bright_yellow();
    let _ = text.on_bright_blue();
    let _ = text.on_bright_magenta();
    let _ = text.on_bright_cyan();
    let _ = text.on_bright_white();

    let styled = text
        .bold()
        .dimmed()
        .italic()
        .underline()
        .strikethrough()
        .reverse();
    let output = styled.to_string();
    assert!(output.contains("\x1b["));
    reset_color_override();
    drop(guard);
}

#[test]
fn context_prompt_methods_and_ui_http_helpers() {
    karigor::ui::set_non_interactive(true);

    let ui = Ui::buffered();
    let ctx = Context {
        args: std::collections::HashMap::new(),
        arg_lists: std::collections::HashMap::new(),
        flags: std::collections::HashMap::new(),
        raw_args: Vec::new(),
        ui: ui.clone(),
    };

    let input_val = ctx.input("Project name", Some("my-app")).unwrap();
    assert_eq!(input_val, "my-app");

    let confirm_val = ctx.confirm("Initialize repo?", true).unwrap();
    assert!(confirm_val);

    let presets = ["REST API", "Full-Stack"];
    let select_idx = ctx.select("Preset", &presets, 1).unwrap();
    assert_eq!(select_idx, 1);

    // UI HTTP helpers
    let cell = ui.cell_http_method("GET");
    assert_eq!(cell.content(), " GET ");
    let formatted = ui.format_http_method("POST");
    assert!(formatted.contains("POST"));

    karigor::ui::set_non_interactive(false);
}
