use std::sync::atomic::{AtomicUsize, Ordering};

use karigor::prelude::*;
use serde_json::json;

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let id = NEXT_DIR.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("karigor_{label}_{}_{id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn render_applies_registered_inflection_filters() {
    let template = concat!(
        "{{ entity | snake_case }}|",
        "{{ entity | pascal_case }}|",
        "{{ entity | kebab_case }}|",
        "{{ entity | plural }}|",
        "{{ entity | singular }}"
    );
    let context = json!({ "entity": "userProfile" });

    let rendered = karigor::template::render(template, &context).unwrap();

    assert!(rendered.contains("user_profile"));
    assert!(rendered.contains("UserProfile"));
    assert!(rendered.contains("user-profile"));
    assert!(rendered.contains("userProfiles"));
    assert!(rendered.contains("userProfile"));
}

#[test]
fn render_reports_template_syntax_errors() {
    let err = karigor::template::render("{{ name ", &json!({ "name": "x" })).unwrap_err();

    assert!(matches!(err, KarigorError::Template(_)));
}

#[test]
fn render_handles_undefined_values_leniently() {
    let rendered = karigor::template::render("Hello {{ missing }}!", &json!({})).unwrap();

    assert_eq!(rendered, "Hello !");
}

#[test]
fn create_environment_registers_the_template_set() {
    let mut env = karigor::template::create_environment();
    env.add_template("probe", "{{ value | plural }}").unwrap();

    let template = env.get_template("probe").unwrap();
    let rendered = template.render(minijinja::value::Value::from_pairs([("value", "category")]));

    assert!(rendered.is_ok());
}

#[test]
fn publish_files_writes_new_files_and_creates_target_directory() {
    let target = unique_temp_dir("publish_new");
    let files = &[("one.txt", "first"), ("two.txt", "second")];

    let outcomes = karigor::template::publish_files(&target, files, false).unwrap();

    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(|(_, written)| *written));
    assert_eq!(
        std::fs::read_to_string(target.join("one.txt")).unwrap(),
        "first"
    );
    assert_eq!(
        std::fs::read_to_string(target.join("two.txt")).unwrap(),
        "second"
    );

    drop(std::fs::remove_dir_all(target));
}

#[test]
fn publish_files_skips_existing_targets_without_force() {
    let target = unique_temp_dir("publish_skip");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("one.txt"), "original").unwrap();

    let files = &[("one.txt", "replacement"), ("two.txt", "added")];
    let outcomes = karigor::template::publish_files(&target, files, false).unwrap();

    assert_eq!(outcomes[0], ("one.txt".to_string(), false));
    assert_eq!(outcomes[1], ("two.txt".to_string(), true));
    assert_eq!(
        std::fs::read_to_string(target.join("one.txt")).unwrap(),
        "original"
    );
    assert_eq!(
        std::fs::read_to_string(target.join("two.txt")).unwrap(),
        "added"
    );

    drop(std::fs::remove_dir_all(target));
}

#[test]
fn publish_files_overwrites_existing_targets_with_force() {
    let target = unique_temp_dir("publish_force");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("one.txt"), "original").unwrap();

    let files = &[("one.txt", "replacement")];
    let outcomes = karigor::template::publish_files(&target, files, true).unwrap();

    assert_eq!(outcomes[0], ("one.txt".to_string(), true));
    assert_eq!(
        std::fs::read_to_string(target.join("one.txt")).unwrap(),
        "replacement"
    );

    drop(std::fs::remove_dir_all(target));
}

#[test]
fn publish_files_accepts_an_empty_manifest() {
    let target = unique_temp_dir("publish_empty");

    let outcomes = karigor::template::publish_files(&target, &[], false).unwrap();

    assert!(outcomes.is_empty());
    assert!(target.is_dir());

    drop(std::fs::remove_dir_all(target));
}
