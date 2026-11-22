use std::io;

use karigor::{
    Error, Kernel,
    command::{Argument, Flag},
};

#[test]
fn error_exit_codes_mapping() {
    let not_found = Error::CommandNotFound {
        name: "unknown".into(),
        suggestion: Some("known".into()),
    };
    assert_eq!(not_found.exit_code(), 2);

    let invalid_arg = Error::InvalidArgument("missing target".into());
    assert_eq!(invalid_arg.exit_code(), 2);

    let invalid_flag = Error::InvalidFlag("unknown option --xyz".into());
    assert_eq!(invalid_flag.exit_code(), 2);

    let exec_err = Error::Execution("runtime failure".into());
    assert_eq!(exec_err.exit_code(), 1);

    let io_err = Error::Io(io::Error::new(io::ErrorKind::NotFound, "file missing"));
    assert_eq!(io_err.exit_code(), 1);
}

#[test]
fn kernel_execute_returns_zero_on_success() {
    let kernel = Kernel::new("test-app", "Testing App").command("health", "Healthcheck", |cmd| {
        cmd.handle(|ctx| {
            ctx.ui.success("OK");
            Ok(())
        })
    });

    let exit_code = kernel.execute(vec!["test-app".into(), "health".into()]);
    assert_eq!(exit_code, 0);
}

#[test]
fn kernel_execute_returns_two_on_syntax_and_missing_args() {
    let kernel = Kernel::new("test-app", "Testing App").command("deploy", "Deploy app", |cmd| {
        cmd.argument(Argument::required("target", "Deployment target"))
            .flag(Flag::string("env", Some('e'), "Environment", None).required())
            .handle(|ctx| {
                let _ = ctx;
                Ok(())
            })
    });

    // Missing required argument
    let code_missing_arg = kernel.execute(vec!["test-app".into(), "deploy".into()]);
    assert_eq!(code_missing_arg, 2);

    // Missing required flag
    let code_missing_flag =
        kernel.execute(vec!["test-app".into(), "deploy".into(), "staging".into()]);
    assert_eq!(code_missing_flag, 2);

    // Unknown command
    let code_unknown_cmd = kernel.execute(vec!["test-app".into(), "nonexistent".into()]);
    assert_eq!(code_unknown_cmd, 2);
}

#[test]
fn kernel_execute_returns_one_on_runtime_failure() {
    let kernel = Kernel::new("test-app", "Testing App").command("fail", "Fails always", |cmd| {
        cmd.handle(|ctx| {
            let _ = ctx;
            Err(Error::Execution("Database connection refused".into()))
        })
    });

    let exit_code = kernel.execute(vec!["test-app".into(), "fail".into()]);
    assert_eq!(exit_code, 1);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn kernel_execute_async_exit_codes() {
    let kernel =
        Kernel::new("test-app", "Testing App").command("async:fail", "Fails async", |cmd| {
            cmd.handle_async(|ctx| {
                let _ = ctx;
                Box::pin(async move { Err(Error::Execution("Remote connection timeout".into())) })
            })
        });

    let exit_code = kernel
        .execute_async(vec!["test-app".into(), "async:fail".into()])
        .await;
    assert_eq!(exit_code, 1);
}

#[test]
fn kernel_env_dispatch_methods_exist_and_callable() {
    let kernel = Kernel::new("test-app", "Testing App");
    let _ = kernel.handle_env();
    let code = kernel.execute_env();
    let _ = code;
}

#[cfg(feature = "async")]
#[tokio::test]
async fn kernel_async_env_and_execute_methods() {
    let kernel = Kernel::new("test-app", "Testing App");
    let _ = kernel.handle_env_async().await;
    let code = kernel.execute_env_async().await;
    let _ = code;
}
