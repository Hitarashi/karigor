use karigor::{
    prelude::*,
    ui::{
        Alert, AlertKind, Color, Sticker, print_alert, print_sticker, reset_color_override,
        set_color_enabled,
    },
};

static COLOR_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn sticker_fluent_and_closure_rendering() {
    let test_guard = COLOR_TEST_MUTEX.lock().unwrap();
    set_color_enabled(false);

    let mut sticker = Sticker::default();
    sticker
        .title("Service Supervisor")
        .border_color(Color::Cyan)
        .min_width(40)
        .line("Status: Active")
        .line("Port: 8080")
        .blank()
        .note("Press [Ctrl+C] to exit");

    let output = sticker.prepare();
    assert!(output.contains("Service Supervisor"));
    assert!(output.contains("Status: Active"));
    assert!(output.contains("Port: 8080"));
    assert!(output.contains("Press [Ctrl+C] to exit"));
    assert!(output.starts_with("╭"));
    assert!(output.ends_with("╯"));

    // Verify closure rendering into buffered UI
    let ui = Ui::buffered();
    ui.render_sticker("Daemon Worker", |s| {
        s.line("Workers: 4").line("Queue: Default");
    });

    let captured = ui.output_string().expect("captured output");
    assert!(captured.contains("Daemon Worker"));
    assert!(captured.contains("Workers: 4"));
    assert!(captured.contains("Queue: Default"));

    // Render directly to a UI instance
    let direct_ui = Ui::buffered();
    sticker.render(&direct_ui);
    assert!(
        direct_ui
            .output_string()
            .unwrap()
            .contains("Service Supervisor")
    );

    // Free function print_sticker
    print_sticker("Free Sticker", &["Item 1", "Item 2"]);

    // Test ui.sticker() builder accessor
    let sticker_from_ui = ui.sticker().title("Ui Sticker").prepare();
    assert!(sticker_from_ui.contains("Ui Sticker"));

    reset_color_override();
    drop(test_guard);
}

#[test]
fn sticker_auto_width_expands_to_fit_longest_line() {
    let test_guard = COLOR_TEST_MUTEX.lock().unwrap();
    set_color_enabled(false);

    let long_line = "A".repeat(70);
    let mut sticker = Sticker::new();
    sticker.title("Auto Sizing").line(&long_line);

    let rendered = sticker.prepare();
    for line in rendered.lines() {
        assert!(
            line.chars().count() >= 72,
            "line width was {}",
            line.chars().count()
        );
    }

    // Sticker without title
    let mut no_title = Sticker::new();
    no_title.line("Standalone notice");
    let no_title_out = no_title.prepare();
    assert!(no_title_out.contains("Standalone notice"));
    assert!(!no_title_out.contains("├"));

    reset_color_override();
    drop(test_guard);
}

#[test]
fn alert_kinds_and_fields_rendering() {
    let test_guard = COLOR_TEST_MUTEX.lock().unwrap();
    set_color_enabled(false);

    let mut alert = Alert::new(AlertKind::Warning, "Database Schema Outdated");
    alert
        .message("Found 3 unapplied migrations in target environment.")
        .suggestion("Execute migration runner before launching server.")
        .consequence("Queries depending on new columns will fail at runtime.")
        .bullet("Migration 001_users.sql (pending)")
        .bullet("Migration 002_posts.sql (pending)");

    let rendered = alert.prepare();
    assert!(rendered.contains("WARNING"));
    assert!(rendered.contains("Database Schema Outdated"));
    assert!(rendered.contains("Found 3 unapplied migrations"));
    assert!(rendered.contains("Suggestion:"));
    assert!(rendered.contains("Consequence:"));
    assert!(rendered.contains("• Migration 001_users.sql"));

    // Verify all alert kinds
    for kind in [
        AlertKind::Success,
        AlertKind::Info,
        AlertKind::Warning,
        AlertKind::Danger,
    ] {
        let instance = Alert::new(kind, "Status Notification");
        let output = instance.prepare();
        assert!(output.contains("Status Notification"));
    }

    // Verify closure helper on Ui
    let ui = Ui::buffered();
    ui.render_alert(AlertKind::Danger, "Data Purge Warning", |a| {
        a.message("This will permanently delete database records.")
            .consequence("Action is irreversible.");
    });

    let captured = ui.output_string().expect("captured alert");
    assert!(captured.contains("DANGER"));
    assert!(captured.contains("Data Purge Warning"));
    assert!(captured.contains("Action is irreversible."));

    // Test shortcut methods
    ui.alert_info("System Info", |a| {
        a.message("Engine initialized.");
    });
    ui.alert_success("Build Complete", |a| {
        a.message("All artifacts compiled.");
    });
    ui.alert_warning("Resource Warning", |a| {
        a.message("High memory consumption.");
    });
    ui.alert_danger("Fatal Exception", |a| {
        a.message("Disk I/O failure.");
    });

    let full_capture = ui.output_string().expect("captured alerts");
    assert!(full_capture.contains("Engine initialized."));
    assert!(full_capture.contains("All artifacts compiled."));
    assert!(full_capture.contains("High memory consumption."));
    assert!(full_capture.contains("Disk I/O failure."));

    // Free function print_alert
    print_alert(AlertKind::Info, "Global Notice", "Notice payload");

    // Test ui.alert() builder accessor
    let alert_from_ui = ui.alert(AlertKind::Info, "Ui Alert").prepare();
    assert!(alert_from_ui.contains("Ui Alert"));

    reset_color_override();
    drop(test_guard);
}

#[test]
fn quiet_mode_suppresses_sticker_and_alert() {
    let ui = Ui::buffered().quiet(true);
    ui.render_sticker("Suppressed Sticker", |s| {
        s.line("Hidden");
    });
    ui.render_alert(AlertKind::Warning, "Suppressed Alert", |a| {
        a.message("Hidden");
    });

    assert_eq!(ui.output_string().unwrap(), "");
}

#[test]
fn no_color_and_dumb_terminal_environment() {
    let test_guard = COLOR_TEST_MUTEX.lock().unwrap();

    reset_color_override();
    unsafe {
        std::env::set_var("NO_COLOR", "1");
    }
    assert!(!karigor::ui::is_color_enabled());
    unsafe {
        std::env::remove_var("NO_COLOR");
    }

    reset_color_override();
    unsafe {
        std::env::set_var("TERM", "dumb");
    }
    assert!(!karigor::ui::is_color_enabled());
    unsafe {
        std::env::remove_var("TERM");
    }

    reset_color_override();
    drop(test_guard);
}
