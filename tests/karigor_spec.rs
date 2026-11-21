//! Specification test suite for Karigor CLI.

use druto_karigor::{
    commands::{
        doctor::execute_doctor,
        make::{execute_make_command, execute_make_controller, execute_make_middleware},
        middleware::execute_middleware,
        routes::execute_routes,
    },
    normalize_args,
    ui::{
        create_table, format_http_method, print_banner, print_error, print_info, print_section,
        print_success, print_warn,
    },
};

#[test]
fn test_normalize_args_direct_invocation() {
    let args = vec!["karigor", "routes"];
    let normalized = normalize_args(args);
    assert_eq!(normalized, vec!["karigor", "routes"]);
}

#[test]
fn test_normalize_args_cargo_subcommand() {
    let args = vec!["cargo-karigor", "karigor", "routes"];
    let normalized = normalize_args(args);
    assert_eq!(normalized, vec!["cargo-karigor", "routes"]);
}

#[test]
fn test_normalize_args_colon_subcommands() {
    let args = vec!["karigor", "make:controller", "User", "--api"];
    let normalized = normalize_args(args);
    assert_eq!(
        normalized,
        vec!["karigor", "make", "controller", "User", "--api"]
    );

    let args_mw = vec!["cargo-karigor", "karigor", "make:middleware", "Auth"];
    let normalized_mw = normalize_args(args_mw);
    assert_eq!(
        normalized_mw,
        vec!["cargo-karigor", "make", "middleware", "Auth"]
    );
}

#[test]
fn test_ui_http_method_formatting() {
    assert!(format_http_method("GET").contains("GET"));
    assert!(format_http_method("POST").contains("POST"));
    assert!(format_http_method("PUT").contains("PUT"));
    assert!(format_http_method("PATCH").contains("PATCH"));
    assert!(format_http_method("DELETE").contains("DELETE"));
    assert!(format_http_method("HEAD").contains("HEAD"));
    assert!(format_http_method("OPTIONS").contains("OPTIONS"));
    assert!(format_http_method("CUSTOM").contains("CUSTOM"));
}

#[test]
fn test_ui_table_creation() {
    let mut table = create_table(&["Col1", "Col2"]);
    assert_eq!(table.column_count(), 2);
}

#[test]
fn test_ui_primitives() {
    print_banner("Test Title", "Test Subtitle");
    print_section("Test Section");
    print_success("Operation succeeded");
    print_info("Information note");
    print_warn("Warning notification");
    print_error("Error notification");
}

#[tokio::test]
async fn test_dev_execution() {
    assert!(
        druto_karigor::commands::dev::execute_dev(Some("127.0.0.1".to_string()), Some(8080))
            .await
            .is_ok()
    );
}

#[test]
fn test_routes_execution() {
    assert!(execute_routes(None, false).is_ok());
    assert!(execute_routes(Some("auth".to_string()), false).is_ok());
    assert!(execute_routes(None, true).is_ok());
}

#[test]
fn test_middleware_execution() {
    assert!(execute_middleware().is_ok());
}

#[test]
fn test_make_commands_execution() {
    assert!(execute_make_controller("Users", true).is_ok());
    assert!(execute_make_middleware("Audit").is_ok());
    assert!(execute_make_command("SyncDb").is_ok());
}

#[test]
fn test_doctor_execution() {
    assert!(execute_doctor().is_ok());
}
