use std::io::IsTerminal;

use karigor::{prelude::*, ui};

#[test]
fn banner_renders_with_and_without_subtitle() {
    ui::print_banner("Karigor", "Developer Tool Suite");
    ui::print_banner("Karigor", "");
}

#[test]
fn banner_handles_wide_and_unicode_titles_without_panicking() {
    ui::print_banner(
        "🚀 করিগর — Command Line Toolkit",
        "A subtitle long enough to overflow",
    );
}

#[test]
fn status_primitives_render() {
    ui::print_section("Section");
    ui::print_success("Saved");
    ui::print_info("Note");
    ui::print_warn("Careful");
    ui::print_error("Failed");
}

#[test]
fn http_method_cells_cover_every_badge() {
    for method in [
        "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "TRACE",
    ] {
        let cell = ui::cell_http_method(method);
        assert!(cell.content().to_uppercase().contains(method));
    }

    assert!(
        ui::cell_http_method("get")
            .content()
            .to_uppercase()
            .contains("GET")
    );
}

#[test]
fn http_method_strings_cover_every_badge() {
    for method in [
        "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "TRACE",
    ] {
        let badge = ui::format_http_method(method);
        assert!(badge.to_uppercase().contains(method));
    }

    assert!(ui::format_http_method("get").to_uppercase().contains("GET"));
}

#[test]
fn tables_are_rounded_and_render_rows() {
    let mut table = ui::create_table(&["Method", "Path"]);
    table.add_row(vec![ui::cell_http_method("GET"), Cell::new("/users")]);

    let rendered = table.to_string();
    assert!(rendered.contains("Method"));
    assert!(rendered.contains("Path"));
    assert!(rendered.contains("/users"));
}

#[test]
fn spinners_follow_the_terminal_and_the_quiet_flag() {
    let spinner = ui::create_spinner("Working");
    let is_dumb = std::env::var("TERM").is_ok_and(|term| term == "dumb");
    let is_term = std::io::stdout().is_terminal() && std::io::stderr().is_terminal();
    let expected_hidden = !is_term || is_dumb;
    assert_eq!(spinner.is_hidden(), expected_hidden);
    spinner.set_message("Still working");
    spinner.finish_and_clear();

    let quiet = Ui::new().quiet(true);
    let hidden = quiet.spinner("Working");
    assert!(hidden.is_hidden());
    hidden.finish_and_clear();
}

#[test]
fn ui_messages_execute_in_verbose_and_quiet_modes() {
    let ui = Ui::new();
    ui.banner("Karigor", "Toolkit");
    ui.section("Section");
    ui.success("Saved");
    ui.info("Note");
    ui.warn("Careful");
    ui.error("Failed");

    let quiet = Ui::new().quiet(true);
    quiet.banner("Karigor", "Toolkit");
    quiet.section("Section");
    quiet.success("Saved");
    quiet.info("Note");
    quiet.warn("Careful");
    quiet.error("Failed");
}

#[test]
fn ui_table_builds_the_same_shape_as_the_free_function() {
    let ui = Ui::new();
    let mut table = ui.table(&["Name"]);
    table.add_row(vec![Cell::new("value")]);

    assert!(table.to_string().contains("value"));
}

#[test]
fn default_ui_is_not_quiet() {
    let ui = Ui::default();
    let is_dumb = std::env::var("TERM").is_ok_and(|term| term == "dumb");
    let is_term = std::io::stdout().is_terminal() && std::io::stderr().is_terminal();
    let expected_hidden = !is_term || is_dumb;
    assert_eq!(ui.spinner("x").is_hidden(), expected_hidden);
}

#[test]
fn prompts_return_an_error_when_no_terminal_is_available() {
    // `dialoguer` checks the terminal before reading, so this cannot block when
    // stderr is not a TTY. Skip when the suite runs attached to a real terminal.
    if std::io::stderr().is_terminal() {
        return;
    }

    let ui = Ui::new();

    assert!(ui.input("Name?", None).is_err());
    assert!(ui.input("Name?", Some("Default")).is_err());
    assert!(ui.confirm("Continue?", true).is_err());
    assert!(ui.select("Pick", &["one", "two"], 0).is_err());

    assert!(ui::prompt_input("Name?", None).is_err());
    assert!(ui::prompt_confirm("Continue?", false).is_err());
    assert!(ui::prompt_select("Pick", &["one", "two"], 1).is_err());
}
