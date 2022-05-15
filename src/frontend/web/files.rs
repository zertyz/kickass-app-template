//! Exposes methods that allow serving files

use super::embedded_files;
use std::{
    io::Cursor,
    path::PathBuf,
};
use rocket::{
    get,
    Request,
    Response,
    response::{self, Responder},
    http::{
        ContentType,
        Status,
    },
};


pub const BASE_PATH: &str = "/";

/// all methods exported by this module
pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        get_embedded_file,
    ]
}

/// serves statically linked files (to the executable) for blazing-fast speeds
/// (no context switches nor cache additions/evictions)
/// -- for more details, see `build.rs`
#[get("/<file..>")]
fn get_embedded_file(file: PathBuf) -> EmbeddedFile {
    let internal_file_name = format!("/{}", file.to_string_lossy().to_string());
    EmbeddedFile {file_name: internal_file_name}
}

struct EmbeddedFile {
    file_name: String,
}

impl<'r> Responder<'r, 'r> for EmbeddedFile {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'r> {
        let file_name = self.file_name;
        let (compressed, file_contents) = match embedded_files::STATIC_FILES.get(file_name.as_str()) {
            Some(tuple) => tuple,
            None => return Result::Err(Status{code:404}),
        };
        let file_extension = match file_name.rsplit_once(".") {
            Some((_file_name_before_last_dot, file_extension)) => file_extension,
            None => "html",
        };
        let mut response_builder = Response::build();
        response_builder.header(ContentType::from_extension(file_extension).unwrap());
        if *compressed {
            // informs the client the content is compressed
            response_builder.raw_header("Content-Encoding", embedded_files::CONTENT_ENCODING);
        }
        response_builder
            // enforce caching on the client
            .raw_header("Cache-Control", embedded_files::CACHE_CONTROL)
            .raw_header("expires",       embedded_files::EXPIRATION_DATE)
            .raw_header("last-modified", embedded_files::GENERATION_DATE)
            .sized_body(file_contents.len(), Cursor::new(file_contents))
            .ok()
    }
}