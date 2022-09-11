//! Here you'll see a demonstration of how to create an async processor that needs a single thread to perform simple operations,
//! and, for this reason, it is way faster than [parallel_processor].\
//! On the example implemented here, it is able to perform:
//!   - 868k/s input messages speed -- with 180% CPU -- for the following input:
//!     (I used a variation of the following command, writing the input to a file and then passing the output through an 8k buffer dd writing to /dev/null)
//!     clear; (for i in {1..5654356}; do for m in "Ping" "Speechless" "Pang" "Help" "Ping" "Speechless" "Pang" "Help"; do echo "$m";done; done) | nc -vvvv localhost 9758 | dd status=progress | wc -l
//!   - 4M/s was attained (similar CPU usage) with an input file from this command:
//!     (for i in {1..5654356}; do echo -en "Speechless\nSpeechless\nSpeechless\nSpeechless\nSpeechless\nSpeechless\nSpeechless\nSpeechless\n"; done) >/tmp/kickass.input2
//!   - IMPORTANT: set `sync_processors()` to use a waiting producer, like [super::executor::sync_futures_processors()], or else you'll simply get `TooBusy` answers
//!
//! Analysis:
//!   - One thread is executing `message-io` and another, this processor
//!   - The last test don't use allocations and do not send back any messages -- and it was almost 6 times faster
//!   - In the future, those figures are to be improved when `message-io` is replaced with a Tokio implementation
//!     (so there is no async/sync overhead)
//!
//! `message-io`: it was a negative surprise that `message-io` wasn't able to process any other connections when these flood tests were being executed

use super::{
    types::*,
    socket_server::SocketEvent,
    protocol::{ClientMessages, ServerMessages},
};
use std::{
    sync::Arc,
    collections::HashMap
};
use futures::{Stream, StreamExt};
use message_io::network::{Endpoint, SendStatus};


/// customize this to hold the states you want for each client
#[derive(Debug)]
struct ClientStates {
    count: usize,
}

/// Here is where the main "protocol" processor logic lies: returns a Stream pipeline able to
/// transform client inputs ([ClientMessages] requests) into server outputs ([ServerMessages] answers)
fn processor(stream: impl Stream<Item = SocketEvent<ClientMessages>>)
            -> impl Stream<Item = Result<(Endpoint, ServerMessages),
                                         (Endpoint, Box<dyn std::error::Error + Sync + Send>)>> {

    let mut client_states: HashMap<Endpoint, ClientStates> = HashMap::new();

    stream
        .map(move |socket_event: SocketEvent<ClientMessages>| {
            match socket_event {

                SocketEvent::Incoming { endpoint, client_message } => {
                    let server_message = match client_message {

                        ClientMessages::Ping => {
                            let client_state = client_states.get_mut(&endpoint).expect("unknown client");
                            client_state.count += 1;
                            ServerMessages::Pong(client_state.count)
                        }

                        ClientMessages::Pang => {
                            let client_state = client_states.get_mut(&endpoint).expect("unknown client");
                            client_state.count += 1;
                            let param = format!("`Pang` from {}, {} times", endpoint.addr(), client_state.count);
                            ServerMessages::Pung(param)
                        }

                        ClientMessages::Speechless => {
                            ServerMessages::None
                        },

                        ClientMessages::Error => {
                            ServerMessages::ProcessorError("This processor handles all its errors internally...".to_string())
                        }
                    };
                    Ok((endpoint, server_message))
                },

                SocketEvent::Connected { endpoint } => {
                    client_states.insert(endpoint, ClientStates { count: 0 });
                    Ok((endpoint, ServerMessages::None))
                },

                SocketEvent::Disconnected { endpoint } => {
                    client_states.remove(&endpoint);
                    Ok((endpoint, ServerMessages::None))
                },

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

/// see [super::executor::spawn_concurrent_stream_executor()]
pub async fn spawn_stream_executor(stream: impl Stream<Item = (Endpoint, SendStatus)> + Send + Sync + 'static) -> tokio::task::JoinHandle<()> {
    super::executor::spawn_stream_executor(stream).await
}