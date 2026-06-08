//! WienerEnvoy daemon entry point. Thin shell over `wienerenvoy_daemon::run`,
//! which holds the wiring so it can be driven from integration tests.

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match wienerenvoy_daemon::run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // Errors here predate the tracing subscriber in some paths, so
            // print to stderr directly as a fallback.
            eprintln!("wienerenvoy-daemon: fatal: {err:#}");
            ExitCode::FAILURE
        }
    }
}
