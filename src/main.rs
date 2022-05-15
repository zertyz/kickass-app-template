mod config;
mod frontend;
pub mod command_line;

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use log::{debug, warn};
use crate::config::Config;
use crate::config::config_model::UiOptions;
use crate::frontend::AvailableFrontends;

pub const APP_NAME: &str = "kickass-app-template";

#[cfg(debug_assertions)]
const DEBUG: bool = true;
#[cfg(not(debug_assertions))]
const DEBUG: bool = false;


/// TODO put here any extra code to start your sync libs / frameworks. Be careful not to block the main thread.
fn custom_sync_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // nothing here, for now...
    Ok(())
}

/// TODO put here any extra code to start your async libs/frameworks
async fn custom_async_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // nothing here, for now...
    Ok(())
}

fn main() {

    let config_file_options = load_configs();
    if DEBUG {
        println!("Config file options: {:#?}", config_file_options);
    }
    let command_line_options = command_line::parse_from_args();
    if DEBUG {
        println!("Command Line options: {:#?}", command_line_options);
    }
    let effective_config = command_line::merge_config_file_and_command_line_options(config_file_options, command_line_options);
    let effective_config = Arc::new(effective_config);
    let _logger_guard = setup_logging(&effective_config);

    warn!("{} application started!", APP_NAME);
    debug!("Running 'custom_sync_initialization()':");
    custom_sync_initialization().expect("Error in 'custom_sync_initialization()'");

    let _tokio_join_handle = start_tokio_runtime_and_apps(Arc::clone(&effective_config), 2);

    frontend::run(match effective_config.ui {
        UiOptions::Automatic => auto_select_ui(&effective_config),
        UiOptions::Console   => AvailableFrontends::Console,
        UiOptions::Terminal  => AvailableFrontends::Terminal,
        UiOptions::Egui      => AvailableFrontends::Egui,
    });
    debug!("App exit requested. Starting shutdown process!");

    debug!("(within 60 seconds...)");
    thread::sleep(std::time::Duration::from_secs(60));


    //tokio_join_handle.join().expect("Error while joining into the Tokio runtime");
}

/// Loads default configs from ${0}.config.ron file -- creating it with defaults if it doesn't exist
fn load_configs() -> Config {
    let program_name = std::env::args().next().expect("Program name couldn't be retrieve from args");
    let config_file = format!("{}.config.ron", program_name);
    config::load_or_create_default(&config_file)
        .expect("Could not load (or create) the configuration file")
}

/// starts the Tokio runtime and all related UIs,
fn start_tokio_runtime_and_apps(config: Arc<Config>, worker_threads: usize) -> JoinHandle<()> {
    thread::spawn(move || {
        debug!("  about to start the Tokio runtime with {} worker threads...", worker_threads);
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(worker_threads)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let config = Arc::clone(&config);
                let t1 = tokio::spawn(async {
                    debug!("    running 'custom_async_initialization()'...");
                    custom_async_initialization().await
                        .map_err(|err| format!("Couldn't run 'custom_async_initialization()' function from main: {}", err))
                        .unwrap();
                });
                let t2 = tokio::spawn(async move {
                    debug!("    starting Telegram UI service...");
                    let _shutdown_telegram = frontend::telegram::run(Arc::clone(&config)).await;
                });
                let t3 = tokio::spawn(async {
                    debug!("    starting Rocket Web service...");
                    frontend::web::launch_rocket().await
                        .map_err(|err| format!("Couldn't start Rocket: {}", err))
                        .unwrap();
                });
                let _result = tokio::join!(t1, t2, t3);
            })
    })
}

fn auto_select_ui(_config: &Config) -> AvailableFrontends {
    // if std::env("DISPLAY") {
    //     AvailableFrontends::Egui
    // } else if is_tty() && config.log != Console {
    //     AvailableFrontends::Terminal
    // } else {
    AvailableFrontends::Console
    // }
}


// LOGGING
//////////
// Facade for the `slog` crate to behave just like the `log` API
// (currently we use `slog-scope` & `slog-stdlog` crates for the heavy lifting)
use config::config_model::LoggingOptions;
use slog_scope::GlobalLoggerGuard;
use sloggers::{Build, types::{OverflowStrategy, Severity}};


/// Keep those levels in sync with Cargo.toml's `log` crate levels defined in features.
/// Example: features = ["max_level_debug", "release_max_level_info"]
const LOG_LEVEL: Severity = if DEBUG {
    Severity::Debug
} else {
    Severity::Info
};

/// starts a global logger according to `config` specifications
/// -- the returned value should not be dropped until the program ends
fn setup_logging(config: &Config) -> GlobalLoggerGuard {
    match &config.log {
        LoggingOptions::Quiet => build_quiet_logger(),
        LoggingOptions::ToConsole => build_console_logger(),
        LoggingOptions::ToFile {file_path, rotation_size, rotations_kept, compress_rotated} => build_file_logger(&file_path, *rotation_size, *rotations_kept, *compress_rotated)
    }
}

fn build_quiet_logger() -> GlobalLoggerGuard {
    let logger = sloggers::null::NullLoggerBuilder {}
        .build()
        .expect("Could not create a 'quiet' logger");
    let log_guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();
    log_guard
}

fn build_console_logger() -> GlobalLoggerGuard{
    let mut builder = sloggers::terminal::TerminalLoggerBuilder::new();
    builder.level(LOG_LEVEL);
    builder.destination(sloggers::terminal::Destination::Stdout);
    let logger = builder.build().expect("Could not create a 'console' logger");
    let log_guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();
    log_guard
}

fn build_file_logger(log_file: &str, rotate_size: usize, rotate_keep: usize, rotate_compress: bool) -> GlobalLoggerGuard {
    let mut builder = sloggers::file::FileLoggerBuilder::new(log_file);
    builder.overflow_strategy(OverflowStrategy::Block);
    builder.rotate_size(rotate_size as u64);
    builder.rotate_keep(rotate_keep);
    builder.rotate_compress(rotate_compress);
    builder.level(LOG_LEVEL);
    let logger = builder.build().expect("Could not create a file logger");
    let log_guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();
    log_guard
}
