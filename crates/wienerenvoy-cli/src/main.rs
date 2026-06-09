//! `wenvoy`: the WienerEnvoy control CLI.

use std::path::Path;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use wienerenvoy_core::Config;

mod client;
mod commands;
mod ui;

const DEFAULT_CONFIG_PATH: &str = "/Library/Application Support/WienerEnvoy/config.toml";

#[derive(Parser)]
#[command(
    name = "wenvoy",
    version,
    about = "WienerEnvoy control CLI: manage your Mac mini home server over the tailnet."
)]
struct Cli {
    /// Suppress informational output.
    #[arg(long, global = true)]
    quiet: bool,
    /// Disable colored output.
    #[arg(long, global = true)]
    no_color: bool,
    /// Path to config.toml.
    #[arg(long, global = true, env = "WIENERENVOY_CONFIG")]
    config: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show daemon and server status.
    Status,
    /// Print a one-shot telemetry snapshot.
    Metrics,
    /// Print one-shot machine info.
    Info,
    /// Control machine power.
    Power {
        #[command(subcommand)]
        action: PowerCmd,
    },
    /// Toggle the keep-awake assertion.
    KeepAwake {
        /// on or off
        #[arg(value_enum)]
        state: Toggle,
    },
    /// Turn the server function on or off (daemon stays reachable).
    Server {
        /// on or off
        #[arg(value_enum)]
        state: Toggle,
    },
    /// List Docker services (stacks and containers).
    Services,
    /// Control a Docker stack or container.
    Service {
        #[command(subcommand)]
        action: ServiceCmd,
    },
    /// Manage the bearer token.
    Token {
        #[command(subcommand)]
        action: TokenAction,
    },
    /// Print instructions to install the daemon.
    Install,
    /// Print instructions to uninstall the daemon.
    Uninstall,
    /// Generate shell completions.
    Completions {
        /// Target shell.
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand)]
enum PowerCmd {
    /// Sleep the machine (recoverable over the tailnet).
    Sleep,
    /// Restart the machine (returns automatically after boot).
    Restart {
        /// Confirm the restart.
        #[arg(long)]
        yes: bool,
    },
    /// Shut down the machine (recover via Wake-on-LAN only).
    Shutdown {
        /// Confirm the shutdown.
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Clone, ValueEnum)]
enum Toggle {
    On,
    Off,
}

#[derive(Subcommand)]
enum ServiceCmd {
    /// Start a container (or a whole stack with --stack).
    Start {
        id: String,
        #[arg(long)]
        stack: bool,
    },
    /// Stop a container (or a whole stack with --stack).
    Stop {
        id: String,
        #[arg(long)]
        stack: bool,
    },
    /// Restart a container (or a whole stack with --stack).
    Restart {
        id: String,
        #[arg(long)]
        stack: bool,
    },
    /// Tail a container's logs.
    Logs {
        id: String,
        #[arg(long, default_value_t = 200)]
        tail: usize,
    },
}

#[derive(Subcommand)]
enum TokenAction {
    /// Print the bearer token.
    Show,
    /// Rotate the bearer token.
    Rotate,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    if cli.quiet {
        ui::set_quiet(true);
    }
    if cli.no_color {
        ui::set_color_override(-1);
    }

    // Completions need no config.
    if let Command::Completions { shell } = cli.command {
        clap_complete::generate(shell, &mut Cli::command(), "wenvoy", &mut std::io::stdout());
        return ExitCode::SUCCESS;
    }

    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
    let config = match Config::load(Path::new(&config_path)) {
        Ok(cfg) => cfg,
        Err(err) => {
            ui::error(format!("config: {err}"));
            return ExitCode::FAILURE;
        }
    };

    let result = match cli.command {
        Command::Status => commands::status(&config).await,
        Command::Metrics => commands::metrics(&config).await,
        Command::Info => commands::info(&config).await,
        Command::Power { action } => match action {
            PowerCmd::Sleep => commands::power(&config, "sleep", true).await,
            PowerCmd::Restart { yes } => commands::power(&config, "restart", yes).await,
            PowerCmd::Shutdown { yes } => commands::power(&config, "shutdown", yes).await,
        },
        Command::KeepAwake { state } => {
            commands::keep_awake(&config, matches!(state, Toggle::On)).await
        }
        Command::Server { state } => commands::server(&config, matches!(state, Toggle::On)).await,
        Command::Services => commands::services(&config).await,
        Command::Service { action } => match action {
            ServiceCmd::Start { id, stack } => {
                commands::service_control(&config, target(stack), &id, "start").await
            }
            ServiceCmd::Stop { id, stack } => {
                commands::service_control(&config, target(stack), &id, "stop").await
            }
            ServiceCmd::Restart { id, stack } => {
                commands::service_control(&config, target(stack), &id, "restart").await
            }
            ServiceCmd::Logs { id, tail } => commands::service_logs(&config, &id, tail).await,
        },
        Command::Token { action } => match action {
            TokenAction::Show => commands::token_show(&config),
            TokenAction::Rotate => commands::token_rotate(),
        },
        Command::Install => commands::install(),
        Command::Uninstall => commands::uninstall(),
        Command::Completions { .. } => unreachable!("handled above"),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            ui::error(err);
            ExitCode::FAILURE
        }
    }
}

fn target(stack: bool) -> &'static str {
    if stack { "stack" } else { "container" }
}
