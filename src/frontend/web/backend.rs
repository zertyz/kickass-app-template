//! Place here any methods your web UI uses

use rocket::get;


pub const BASE_PATH: &str = "/backend";

/// all methods exported by this module
pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        info,
    ]
}


#[get("/info")]
fn info() -> &'static str {
    "Backend written in Rust!"
}