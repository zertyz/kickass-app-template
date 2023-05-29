mod demo;

use crate::{
    config::{Config},
    runtime::Runtime,
    frontend
};
use tokio::sync::RwLock;


pub fn run(runtime: &RwLock<Runtime>, _config: &Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::thread::sleep(std::time::Duration::from_secs(5));
    demo::run_demo(demo::Config {
        enhanced_graphics: false,
        ..Default::default()
    }).map_err(|err| format!("Error running Terminal UI: {:?}", err))?;
    frontend::sync_shutdown_tokio_services(runtime)
}
