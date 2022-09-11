//! Here you'll see a demonstration of how to create an async processor that is able to use all available CPUs to perform
//! some CPU-bound work.\
//! When executed with the "CPU Intensive" section un-commented, it was able to use all 12 cores of my machine in
//! Release mode (although the `par-stream` crate seems to have a bad time with Debug mode).\
//! For performance comparisons with [serial_processor], the following figures were attained with the "CPU Intensive"
//! section commented out:
//!   - 470k/s input messages -- 425% CPU usage -- for the same input & method used when profiling [serial_processor] for multiple message types
//!   - 1,8M/s input messages for the same input & method used when profile [serial_processor] for the single message with no answer
//!   - IMPORTANT: set `sync_processors()` to use a waiting producer, like [super::executor::sync_futures_processors()], or else you'll simply get `TooBusy` answers
//!
//! Analysis:
//!   - Synchronizing between threads is worthless for such simple workload, as expected

use super::{
    types::*,
    socket_server::SocketEvent,
    protocol::{ClientMessages, ServerMessages},
};
use std::{
    sync::Arc,
    collections::HashMap
};
use futures::{Stream, FutureExt};
use par_stream::prelude::*;
use message_io::network::{Endpoint, SendStatus};
use tokio::sync::{RwLock};


/// customize this to hold the states you want for each client
#[derive(Debug)]
struct ClientStates {
    count: usize,
}

/// Here is where the main "protocol" processor logic lies: returns a Stream pipeline able to
/// transform client inputs ([ClientMessages] requests) into server outputs ([ServerMessages] answers)
fn processor(stream: impl Stream<Item = SocketEvent<ClientMessages>> + Send + 'static)
            -> impl Stream<Item = Result<(Endpoint, ServerMessages),
                                         (Endpoint, Box<dyn std::error::Error + Sync + Send>)>> {

    let client_states = Arc::new(RwLock::new(HashMap::<Endpoint, ClientStates>::new()));

    stream
        .par_then_unordered(super::executor::PAR_PARAMS, move |socket_event: SocketEvent<ClientMessages>| {
            let client_states = Arc::clone(&client_states);
            async move {
                let client_states = Arc::clone(&client_states);
                match socket_event {

                    SocketEvent::Incoming { endpoint, client_message } => {
                        let server_message = match client_message {

                            ClientMessages::Ping => {
                                let mut writeable_client_states = client_states.write().await;
                                let client_state = writeable_client_states.get_mut(&endpoint).expect("unknown client");
                                client_state.count += 1;
                                Ok(ServerMessages::Pong(client_state.count))
                            }

                            ClientMessages::Pang => {
                                let mut writeable_client_states = client_states.write().await;
                                let client_state = writeable_client_states.get_mut(&endpoint).expect("unknown client");
                                let msg_count = client_state.count + 1;
                                client_state.count = msg_count;
                                drop(client_state);
                                drop(writeable_client_states);
                                ///* CPU Intensive
                                let mut r = msg_count as u32;
                                for i in 1..(1<<24) {
                                    r ^= r % i;
                                }
                                let param = format!("`Pang` from {}, {} times -- r={r} -- THREAD {:?}", endpoint.addr(), msg_count, std::thread::current()); // */
                                /* Uncomment this if comparing the performance to [concurrent_processor]
                                let param = format!("`Pang` from {}, {} times", endpoint.addr(), msg_count); // */
                                Ok(ServerMessages::Pung(param))
                            }

                            ClientMessages::Speechless => {
                                Ok(ServerMessages::None)
                            },

                            ClientMessages::Error => {
                                // here there is a demonstration of how to handle errors from functions that fail
                                // (notice the wrapper the end of this match statement: there, the error will have the endpoint attached to it,
                                //  so the client will be notified their message wasn't processed correctly)
                                Err(Box::from(format!("This is an example of a fallible processor failing :)")))
                            },
                        };
                        // Ok / Err wrapper
                        match server_message {
                            Ok(server_message) => Ok((endpoint, server_message)),
                            Err(err) => Err((endpoint, err)),
                        }
                    },

                    SocketEvent::Connected { endpoint } => {
                        client_states.write().await
                            .insert(endpoint, ClientStates { count: 0 });
                        Ok((endpoint, ServerMessages::None))
                    },

                    SocketEvent::Disconnected { endpoint } => {
                        client_states.write().await
                            .remove(&endpoint);
                        Ok((endpoint, ServerMessages::None))
                    },

                }
            }
        })
}

/// Returns a tied-together `(stream, producer, closer)` tuple which [socket_server] uses to transform [ClientMessages] into [ServerMessages].\
/// The tuple consists of:
///   - The `Stream` of (`Endpoint`, [ServerMessages]) -- [socket_server] will, then, apply operations at the end of it to deliver the messages
///   - The producer to send `SocketEvent<ClientMessages>` to that stream
///   - The closer of the stream
pub fn sync_processors(tokio_runtime: Arc<tokio::runtime::Runtime>) -> (impl Stream<Item = Result<(Endpoint, ServerMessages), (Endpoint, Box<dyn std::error::Error + Sync + Send>)>>,
                                                                        impl FnMut(SocketEvent<ClientMessages>) -> bool,
                                                                        impl FnMut()) {
    let (stream, producer, closer) = super::executor::sync_tokio_stream(tokio_runtime);
    (processor(stream), producer, closer)
}

/// see [super::executor::spawn_parallel_stream_executor()]
pub async fn spawn_stream_executor(stream: impl Stream<Item = (Endpoint, SendStatus)> + Send + Sync + 'static) -> tokio::task::JoinHandle<()> {
    super::executor::spawn_parallel_stream_executor(stream).await
}