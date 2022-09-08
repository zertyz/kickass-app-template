//! Home for all frontends & UIs

pub mod console;
pub mod terminal;
pub mod egui;
pub mod telegram;
pub mod web;

use crate::{runtime::Runtime, config::{Config}, ExtendedOption, UiOptions};
use tokio::sync::RwLock;
use crate::frontend::egui::Egui;
use log::{debug};


pub async fn async_run(runtime: &RwLock<Runtime>, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    match config.ui {
        ExtendedOption::Enabled(ui) => match ui {
            UiOptions::Console(job) => console::async_run(&job, runtime, &config).await,
            UiOptions::Terminal => Ok(()),//terminal::async_run(config, result).await,
            UiOptions::Egui => Ok(()),
        }
        _ => panic!("BUG! empty `config.ui`"),
    }
}

pub fn run(runtime: &RwLock<Runtime>, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    match config.ui {
        ExtendedOption::Enabled(ui) => match ui {
            UiOptions::Console(job) => console::run(&job, runtime, &config),
            UiOptions::Terminal => terminal::run(runtime, &config),
            UiOptions::Egui => Ok(Egui::run_egui_app(format!("We are!!"), 5.1)),
        }
        _ => panic!("BUG! empty `config.ui`"),
    }
}

/// signals background (async Tokio) tasks that a graceful shutdown was requested
pub async fn shutdown_tokio_services(runtime: &RwLock<Runtime>) -> Result<(), Box<dyn std::error::Error>> {

    debug!("Program logic is asking for a graceful shutdown...");

    // shutdown the web server
    Runtime::do_for_web_server(runtime, |web_server, _runtime| Box::pin(async move {
        if let Some(shutdown_token) = web_server.shutdown_token.clone() {
            shutdown_token.notify();
        }
    })).await;

    // shutdown telegram
    Runtime::do_for_telegram_ui(runtime, |telegram_ui, _runtime| Box::pin(async move {
        if let Some(shutdown_token) = telegram_ui.shutdown_token.clone() {
            shutdown_token.shutdown()
                .expect("Could not shutdown Telegram")
                .await;
        }
    })).await;

    Ok(())
}

pub fn sync_shutdown_tokio_services(runtime: &RwLock<Runtime>) -> Result<(), Box<dyn std::error::Error>> {
    futures::executor::block_on(shutdown_tokio_services(runtime))
}