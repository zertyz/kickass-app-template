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

use crate::config::config::{Config, WebConfig, RocketConfigOptions, RocketProfiles};
use std::{
    sync::Arc,
    net::Ipv4Addr,
};
use owning_ref::OwningRef;
use futures::future::BoxFuture;
use rocket;


/// Returned by this module when the Rocket server starts -- see [runner()].\
/// Used to, programmatically, interact with the Rocket server:
///  * inquire if the server is running
///  * request the server to shutdown
pub struct WebServer {
    /// runtime configs for this server
    web_config: OwningRef<Arc<Config>, WebConfig>,
    /// tells if the service is fully started & working
    started: bool,
    /// contains the builder for Rocket -- which exists between [new()] and [runner()] calls
    rocket_builder: Option<rocket::Rocket<rocket::Build>>,
    /// if present, exposes the Rocket's `shutdown_token`, through which one may request the service to cease running
    pub shutdown_token: Option<rocket::Shutdown>,
}

impl WebServer {

    pub fn new(web_config: OwningRef<Arc<Config>, WebConfig>) -> WebServer {
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
        Self {
            web_config,
            started: false,
            rocket_builder: Some(rocket_builder),
            shutdown_token: None,
        }
    }

    /// returns a runner, which you may call to run Rocket and that will only return when
    /// the service is over -- this special semantics allows holding the mutable reference to `self`
    /// as little as possible.\
    /// Example:
    /// ```no_compile
    ///     self.runner()().await;
    pub async fn runner<'r>(&mut self) -> Result<impl FnOnce() -> BoxFuture<'r, Result<(),
                                                                                       Box<dyn std::error::Error + Send + Sync>>> + Send + 'r,
                                                 Box<dyn std::error::Error + Send + Sync>> {

        let ignited_rocket = self.rocket_builder.take().expect("BUG: web.rs: rocket_builder is empty")
            .mount(api::BASE_PATH, api::routes())
            .ignite().await
            .map_err(|err| format!("Error 'Ignite'ing rocket: {:?}", err))?;

        self.shutdown_token = Some(ignited_rocket.shutdown());

        let runner = move || -> BoxFuture<'_, Result<(), Box<dyn std::error::Error + Send + Sync>>> {
            Box::pin(async move {
                let _rocket_ignite = ignited_rocket
                    .launch().await
                    .map_err(|err| format!("Error 'Launch'ing rocket: {:?}", err))?;
                Ok(())
            })
        };

        Ok(runner)
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