mod types;

mod socket_server;
pub use socket_server::*;

mod protocol;

mod serial_processor;
mod parallel_processor;
mod futures_processor;
/////////////////////////////////////////////////////////////
// uncomment one of the processors bellow to activate them //
/////////////////////////////////////////////////////////////
pub use serial_processor::{sync_processors, spawn_stream_executor};
//pub use futures_processor::{sync_processors, spawn_stream_executor};
//pub use parallel_processor::{sync_processors, spawn_stream_executor};

mod executor;