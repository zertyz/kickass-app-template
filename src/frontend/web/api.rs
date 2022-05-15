//! Place here any APIs your program shares with external services

use rocket::{
    get, post,
    response::Responder,
    FromFormField,
    FromForm,
    serde::{json::Json, Serialize, Deserialize},
};


pub const BASE_PATH: &str = "/api";

/// all methods exported by this module
pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        rest_service,
        get_service,
        post_service,
    ]
}


/// A simple rest service demo, returning a JSON built out of a string
#[get("/rest-service/<world>")]
fn rest_service(world: &str) -> RawJson {
    RawJson { json: format!(r#"{{"msg":"Hello, world of {}!"}}"#, world) }
}

/// A simple get service demo using native types and a custom enum, returning a JSON built out of a string
#[get("/get-service?<from_temperature>&<from_length>&<conversion>")]
fn get_service(from_temperature: f64, from_length: f64, conversion: Conversions) -> RawJson {
    let (from_temperature_unit, from_length_unit,
        to_temperature, to_length,
        to_temperature_unit, to_length_unit) = match conversion {
        Conversions::MetricToImperial => ("°C", "m",  (from_temperature * 9.0/5.0) + 32.0, from_length * 3.2808398950132, "°F", "ft"),
        Conversions::ImperialToMetric => ("°F", "ft", (from_temperature - 32.0) * 5.0/9.0, from_length / 3.2808398950132, "°C", "m")
    };
    RawJson { json: format!("{{\
                                \"from_temperature\": \"{:.2}{}\",
                                \"from_length\":      \"{:.2}{}\",
                                \"to_temperature\":   \"{:.2}{}\",
                                \"to_length\":        \"{:.2}{}\"
                            }}",
                            from_temperature, from_temperature_unit,
                            from_length,      from_length_unit,
                            to_temperature,   to_temperature_unit,
                            to_length,        to_length_unit) }
}
#[derive(Debug, PartialEq, FromFormField)]
enum Conversions {
    MetricToImperial,
    ImperialToMetric,
}

/// A simple post service demo receiving & sending a JSON made out of a struct
#[post("/post-service", format = "json", data = "<shipping_info_json>")]
fn post_service(shipping_info_json: Json<ShippingInfo>) -> Json<ShippingInfo> {
    let shipping_info = shipping_info_json.into_inner();
    Json(shipping_info)
}
#[derive(Debug, PartialEq, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ShippingInfo {
    company:          Option<String>,
    first_name:       String,
    last_name:        String,
    address:          String,
    city:             String,
    state:            String,
    postal_code:      u32,
    shipping:         String,
    refuse_housemate: bool,
}

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
struct RawJson {
    json: String,
}