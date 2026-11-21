//! Specification test suite for Karigor CLI.

use druto_karigor::{
    commands::{
        doctor::execute_doctor,
        make::{
            execute_make_command, execute_make_controller, execute_make_middleware,
            execute_make_route, execute_stub_publish,
        },
        middleware::execute_middleware,
        routes::execute_routes,
    },
    injector::{inject_controller_module, inject_route_module},
    normalize_args,
    stubs::{get_stub, render_stub},
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
        druto_karigor::commands::dev::execute_dev(Some("127.0.0.1".to_string()), Some(8080), true)
            .await
            .is_ok()
    );
    assert!(
        druto_karigor::commands::dev::execute_dev(None, None, true)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn test_scaffolding_execution() {
    let temp_name = format!(
        "{}/test_druto_app_{}",
        std::env::temp_dir().to_str().unwrap(),
        std::process::id()
    );

    let result = druto_karigor::commands::new::execute_new(Some(temp_name.clone())).await;
    assert!(result.is_ok());

    let path = std::path::Path::new(&temp_name);
    assert!(path.join("Cargo.toml").exists());
    assert!(path.join("src/main.rs").exists());
    assert!(path.join("src/routes/mod.rs").exists());
    assert!(!path.join("stubs").exists());

    let env_text = std::fs::read_to_string(path.join(".env")).unwrap();
    assert!(env_text.contains("PORT=5555"));
    let main_text = std::fs::read_to_string(path.join("src/main.rs")).unwrap();
    assert!(main_text.contains("druto::ignitor::start"));

    drop(std::fs::remove_dir_all(path));
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
    assert!(execute_make_route("Users", false, true).is_ok());
    assert!(execute_make_route("Orders", true, true).is_ok());
    assert!(execute_make_controller("Users", true, true).is_ok());
    assert!(execute_make_controller("Categories", false, true).is_ok());
    assert!(execute_make_middleware("Audit", true).is_ok());
    assert!(execute_make_command("SyncDb", true).is_ok());
    assert!(execute_stub_publish(true).is_ok());
    drop(std::fs::remove_dir_all("stubs"));
}

#[test]
fn test_stubs_rendering_and_borno_inflections() {
    let stub = get_stub("route.stub");
    assert!(!stub.is_empty());

    let context = serde_json::json!({
        "name_snake": "category",
        "has_controller": true,
    });
    let rendered = render_stub(&stub, &context).expect("failed to render stub");
    assert!(rendered.contains("use crate::controllers::category;"));
    assert!(rendered.contains("router.get(\"/\", category::index);"));
}

#[test]
fn test_stub_publish_explicit() {
    let temp_dir = std::env::temp_dir().join(format!("test_stubs_{}", std::process::id()));
    let outcomes = druto_karigor::stubs::publish_stubs(&temp_dir, true).unwrap();
    assert_eq!(outcomes.len(), 4);
    assert!(temp_dir.join("route.stub").exists());
    assert!(temp_dir.join("controller.stub").exists());
    assert!(temp_dir.join("middleware.stub").exists());
    assert!(temp_dir.join("command.stub").exists());
    drop(std::fs::remove_dir_all(&temp_dir));
}

#[test]
fn test_smart_injector_idempotence() {
    let temp_dir = std::env::temp_dir().join(format!("karigor_test_{}", std::process::id()));
    let routes_mod = temp_dir.join("routes").join("mod.rs");
    let controllers_mod = temp_dir.join("controllers").join("mod.rs");

    let outcome1 = inject_route_module(&routes_mod, "users").expect("first route injection");
    assert!(outcome1.module_injected);
    assert!(outcome1.route_mounted);

    // Second injection must be idempotent (no duplicates)
    let outcome2 = inject_route_module(&routes_mod, "users").expect("second route injection");
    assert!(!outcome2.module_injected);
    assert!(!outcome2.route_mounted);

    // Controllers injection
    let ctrl1 = inject_controller_module(&controllers_mod, "users").expect("first ctrl injection");
    assert!(ctrl1);
    let ctrl2 = inject_controller_module(&controllers_mod, "users").expect("second ctrl injection");
    assert!(!ctrl2);

    drop(std::fs::remove_dir_all(temp_dir));
}

#[test]
fn test_doctor_execution() {
    assert!(execute_doctor().is_ok());
}
