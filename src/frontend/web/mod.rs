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

mod planetpedia_api;

use rocket::{self, routes};

/// the route for our logic api. Maybe you could rename it just to '/api' if you'll expose just one
const API_BASE: &str = "/planetpedia_api/";

/// launches and rides the Rocket until the end
pub async fn launch_rocket() -> Result<(), rocket::Error> {
    rocket::build()
        .mount(API_BASE, routes![
            planetpedia_api::index
        ])
        .launch().await
}