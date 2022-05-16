//! Use this module's exported function to init the Rocket framework:
//! ```no_compile
//!     launch_rocket();
//! ```
//!
//! It provides the equivalent of the default, well documented way of launching Rocket apps:
//! ```no_compile
//!     #[launch]
//!     fn rocket() -> _ {
//!         rocket::build().mount("/", routes![...])
//!     }
//! ```
//! ... which cannot be used, since the official way implies that a `main()` function will be written for you,

mod files;
mod embedded_files;
mod api;
mod backend;

use crate::config::config_model::{self, RocketConfigOptions, RocketProfiles};
use std::{
    sync::Arc,
    net::Ipv4Addr,
};
use rocket;

/// launches and rides the Rocket until the end
pub async fn launch_rocket(config: Arc<config_model::Config>) -> Result<(), rocket::Error> {
    if let Some(web_config) = &config.services.web {
        let mut rocket_builder = match web_config.rocket_config {
            RocketConfigOptions::StandardRocketTomlFile => rocket::build(),
            RocketConfigOptions::Provided {http_port, workers} =>
                rocket::custom(build_rocket_config(&web_config.profile, http_port, workers))
        };
        if web_config.web_app {
            rocket_builder = rocket_builder
                .mount(files::BASE_PATH,   files::routes())
                .mount(backend::BASE_PATH, backend::routes());
        }
        rocket_builder
            .mount(api::BASE_PATH,     api::routes())
            .launch().await
    } else {
        Ok(())
    }
}

fn build_rocket_config(profile: &RocketProfiles, http_port: u16, workers: u16) -> rocket::Config {
    let address = Ipv4Addr::new(0, 0, 0, 0).into();
    match profile {
        RocketProfiles::Debug => rocket::Config {
            profile: rocket::Config::DEBUG_PROFILE,
            address,
            port: http_port,
            workers: workers as usize,
            ..rocket::Config::debug_default()
        },
        RocketProfiles::Production => rocket::Config {
            profile: rocket::Config::RELEASE_PROFILE,
            log_level: rocket::log::LogLevel::Critical,
            address,
            port: http_port,
            workers: workers as usize,
            ..rocket::Config::release_default()
        },
    }
}