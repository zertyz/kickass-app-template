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

use rocket;

/// launches and rides the Rocket until the end
pub async fn launch_rocket() -> Result<(), rocket::Error> {
    rocket::build()
        .mount(files::BASE_PATH,   files::routes())
        .mount(api::BASE_PATH,     api::routes())
        .mount(backend::BASE_PATH, backend::routes())
        .launch().await
}