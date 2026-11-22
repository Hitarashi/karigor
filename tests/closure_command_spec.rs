use karigor::{
    Command, CommandBuilder, Error, Kernel,
    command::{Argument, Flag},
};

#[test]
fn command_builder_fluent_sync_execution() {
    let builder = CommandBuilder::new("serve", "Starts application server")
        .argument(Argument::optional(
            "port",
            "Port number",
            Some("8080".into()),
        ))
        .flag(Flag::boolean("daemon", Some('d'), "Run in background"))
        .handle(|ctx| {
            let port = ctx.arg("port").unwrap_or("8080");
            let daemon = ctx.flag_bool("daemon");
            assert_eq!(port, "3000");
            assert!(daemon);
            ctx.ui.success("Server started");
            Ok(())
        });

    let kernel = Kernel::new("test-app", "Testing App").register(builder);
    let result = kernel.handle(vec![
        "test-app".into(),
        "serve".into(),
        "3000".into(),
        "-d".into(),
    ]);

    assert!(result.is_ok());
}

#[test]
fn kernel_closure_command_registration() {
    let kernel =
        Kernel::new("test-app", "Testing App").command("db:seed", "Seeds the database", |cmd| {
            cmd.aliases(&["seed"])
                .flag(Flag::string(
                    "class",
                    Some('c'),
                    "Database seeder class",
                    None,
                ))
                .handle(|ctx| {
                    let class_name = ctx.flag_str("class").unwrap_or("DatabaseSeeder");
                    assert_eq!(class_name, "UsersTableSeeder");
                    ctx.ui.info(&format!("Seeding {class_name}"));
                    Ok(())
                })
        });

    // Test primary name
    let result = kernel.handle(vec![
        "test-app".into(),
        "db:seed".into(),
        "--class".into(),
        "UsersTableSeeder".into(),
    ]);
    assert!(result.is_ok());

    // Test alias
    let alias_result = kernel.handle(vec![
        "test-app".into(),
        "seed".into(),
        "--class".into(),
        "UsersTableSeeder".into(),
    ]);
    assert!(alias_result.is_ok());
}

#[test]
fn closure_command_missing_handler_returns_error() {
    let empty_builder = CommandBuilder::new("noop", "No operation");
    let kernel = Kernel::new("test-app", "Testing App").register(empty_builder);

    let result = kernel.handle(vec!["test-app".into(), "noop".into()]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, Error::Execution(_)));
}

#[cfg(feature = "async")]
#[tokio::test]
async fn closure_command_async_execution() {
    let kernel = Kernel::new("test-app", "Testing App").command(
        "async:fetch",
        "Fetches remote data asynchronously",
        |cmd| {
            cmd.handle_async(|ctx| {
                Box::pin(async move {
                    ctx.ui.info("Starting async fetch");
                    tokio::task::yield_now().await;
                    ctx.ui.success("Fetch completed");
                    Ok(())
                })
            })
        },
    );

    let result = kernel
        .handle_async(vec!["test-app".into(), "async:fetch".into()])
        .await;
    assert!(result.is_ok());
}

#[test]
fn command_trait_defaults_and_builder_accessors() {
    struct OnlyAsyncCommand;
    impl Command for OnlyAsyncCommand {
        fn name(&self) -> &str {
            "only:async"
        }
        fn description(&self) -> &str {
            "Only async"
        }
    }

    let cmd = OnlyAsyncCommand;
    assert_eq!(cmd.name(), "only:async");
    assert_eq!(cmd.description(), "Only async");
    assert_eq!(cmd.aliases(), &[] as &[&str]);
    assert_eq!(cmd.arguments().len(), 0);
    assert_eq!(cmd.flags().len(), 0);

    let ui = karigor::Ui::buffered();
    let mut ctx = karigor::Context {
        args: std::collections::HashMap::new(),
        arg_lists: std::collections::HashMap::new(),
        flags: std::collections::HashMap::new(),
        raw_args: Vec::new(),
        ui: ui.clone(),
    };
    let run_res = cmd.run(&mut ctx);
    assert!(run_res.is_err());
    assert!(
        run_res
            .unwrap_err()
            .to_string()
            .contains("does not provide a synchronous execution handler")
    );

    let builder = CommandBuilder::new("inspect", "Inspect command")
        .aliases(&["insp"])
        .argument(Argument::required("pkg", "Package"))
        .flag(Flag::boolean("all", None, "All packages"));

    assert_eq!(builder.name(), "inspect");
    assert_eq!(builder.description(), "Inspect command");
    assert_eq!(Command::aliases(&builder), &["insp"]);
    assert_eq!(builder.arguments().len(), 1);
    assert_eq!(builder.flags().len(), 1);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn command_builder_async_fallback_and_command_trait_run_async() {
    struct SyncOnlyCommand;
    impl Command for SyncOnlyCommand {
        fn name(&self) -> &str {
            "sync:only"
        }
        fn description(&self) -> &str {
            "Sync only"
        }
        fn run(&self, ctx: &mut karigor::Context) -> Result<(), Error> {
            ctx.ui.success("Sync executed");
            Ok(())
        }
    }

    let sync_cmd = SyncOnlyCommand;
    let ui = karigor::Ui::buffered();
    let mut ctx = karigor::Context {
        args: std::collections::HashMap::new(),
        arg_lists: std::collections::HashMap::new(),
        flags: std::collections::HashMap::new(),
        raw_args: Vec::new(),
        ui: ui.clone(),
    };
    let res = sync_cmd.run_async(&mut ctx).await;
    assert!(res.is_ok());

    let sync_builder = CommandBuilder::new("fallback", "Fallback test").handle(|ctx| {
        ctx.ui.info("Handled via sync");
        Ok(())
    });
    let builder_res = sync_builder.run_async(&mut ctx).await;
    assert!(builder_res.is_ok());

    let empty_builder = CommandBuilder::new("empty", "Empty test");
    let empty_res = empty_builder.run_async(&mut ctx).await;
    assert!(empty_res.is_err());
}
