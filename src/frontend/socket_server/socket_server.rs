//! The socket server, using `message-io`
//! TODO 20220910: `message-io` should be, eventually, replaced by my own Tokio version of this nice event's library (which is uncapable of processing more than 1 client when flooded)


use crate::config::config::{Config, SocketServerConfig};
use super::{
    types::*,
    protocol::{self, ServerMessages, ClientMessages},
};
use std::{
    sync::Arc,
    net::{ToSocketAddrs,SocketAddr},
};
use std::collections::HashSet;
use owning_ref::OwningRef;
use futures::future::BoxFuture;
use futures::{Stream, stream, StreamExt};
use message_io::{
    network::{NetEvent, Transport, Endpoint, SendStatus},
    node::{self, NodeHandler, NodeListener},
};
use message_io::node::NodeEvent;
use log::{trace, debug, info, warn, error};


type DeserializerFn = fn(&[u8]) -> Result<ClientMessages, Box<dyn std::error::Error>>;
type SerializerFn   = fn(ServerMessages) -> String;

// RON serde
const DESERIALIZER: DeserializerFn = protocol::ron_deserializer;
const SERIALIZER:   SerializerFn   = protocol::ron_serializer;
const TRANSPORT:    Transport      = Transport::Tcp;   // Tcp allows plain text messages and seems to work fine for small messages (provided length < MTU size?)

// BinCode serde
// const TRANSPORT:    Transport = Transport::Tcp;   // FramedTcp puts the message length at the beginning of each message, so this is suitable for binary formats
// ...


/// The internal events this server shares with the protocol processors
/// -- think of this as an Adapter between `message-io` and our protocol processor,
/// which will, eventually, be the basis for our Tokio implementation of that to-be-dropped crate.
#[derive(Debug)]
pub enum SocketEvent<ClientMessages> {
    Connected    {endpoint: Endpoint},
    Incoming     {endpoint: Endpoint, client_message: ClientMessages},
    Disconnected {endpoint: Endpoint},
}

/// The handle to define, start and shutdown a Socket Server
pub struct SocketServer<'a> {
    config:                            OwningRef<Arc<Config>, SocketServerConfig>,
    handler:                           NodeHandler<()>,
    listener:                          Option<NodeListener<()>>,
    request_processor_stream_producer: Option<Box<dyn FnMut(SocketEvent<ClientMessages>) -> bool + Send + Sync + 'a>>,
    request_processor_stream_closer:   Option<Box<dyn FnMut() + Send + Sync + 'a>>,
}

impl SocketServer<'static> {

    pub fn new(server_config: OwningRef<Arc<Config>, SocketServerConfig>) -> Self {
        let (handler, listener) = node::split::<()>();
        Self {
            config:                            server_config,
            handler,
            listener:                          Some(listener),
            request_processor_stream_producer: None,
            request_processor_stream_closer:   None,
        }
    }

    /// Attaches a request processor to this Socket Server, comprising of:
    ///   - `request_processor_stream`: this is a stream yielding [ServerMessages] -- most likely mapping [ClientMessages] to it. See [processor::processor()] for an implementation
    ///   - `request_processor_stream_producer`: a `sync` function to feed in [ClientMessages] to the `request_stream_processor`
    ///   - `request_processor_stream_closer`: this closes the stream and is called when the server is shutdown
    pub fn set_processor(&mut self,
                         request_processor_stream:          impl Stream<Item = Result<(Endpoint, ServerMessages), (Endpoint, Box<dyn std::error::Error + Sync + Send>)>> + Send + Sync + 'static,
                         request_processor_stream_producer: impl FnMut(SocketEvent<ClientMessages>) -> bool + Send + Sync + 'static,
                         request_processor_stream_closer:   impl FnMut() + Send + Sync + 'static) -> impl Stream<Item = (Endpoint, SendStatus)> + Send + Sync + 'static {
        self.request_processor_stream_producer = Some(Box::new(request_processor_stream_producer));
        self.request_processor_stream_closer   = Some(Box::new(request_processor_stream_closer));
        to_sender_stream(self.handler.clone(), request_processor_stream)
    }

    /// returns a runner, which you may call to run `Server` and that will only return when
    /// the service is over -- this special semantics allows holding the mutable reference to `self`
    /// as little as possible.\
    /// Example:
    /// ```no_compile
    ///     self.runner()().await;
    pub async fn runner(&mut self) -> Result<impl FnOnce() -> BoxFuture<'static, Result<(),
                                                                                        Box<dyn std::error::Error + Sync + Send>>> + Sync + Send + 'static,
                                             Box<dyn std::error::Error + Sync + Send>> {

        let handler = self.handler.clone();
        let listener = self.listener.take();
        let interface = self.config.interface.clone();
        let port        = self.config.port;
        let request_processor_stream_producer = self.request_processor_stream_producer.take();
        let request_processor_stream_closer = self.request_processor_stream_closer.take();

        if listener.is_none() {
            return Err(Box::from(format!("`listener` is not present. Was this server already executed?")))
        }
        if request_processor_stream_producer.is_none() || request_processor_stream_closer.is_none() {
            return Err(Box::from(format!("Request processor fields are not present. Was `set_processor()` called? ... or was this server already executed?")));
        }

        let request_processor_stream_producer = request_processor_stream_producer.unwrap();
        let request_processor_stream_closer = request_processor_stream_closer.unwrap();

        let runner = move || -> BoxFuture<'_, Result<(), Box<dyn std::error::Error + Send + Sync>>> {
            Box::pin(async move {
                let addr = (interface, port).to_socket_addrs()?.next().expect("Addr Iterator ended prematurely");
                tokio::task::spawn_blocking(move || {
                    run(handler, listener.unwrap(), addr, request_processor_stream_producer, request_processor_stream_closer)
                }).await?;

                Ok(())
            })
        };

        Ok(runner)
    }

    pub fn shutdown(&self) {
        warn!("Socket Server: Shutdown asked & initiated");
        self.handler
            .signals()
            .send(());
    }

}

/// upgrades the `request_processor_stream` to a `Stream` able to either process requests & send back answers to the clients
fn to_sender_stream(handler: NodeHandler<()>, request_processor_stream: impl Stream<Item = Result<(Endpoint, ServerMessages),
                                                                                                  (Endpoint, Box<dyn std::error::Error + Sync + Send>)>>)
                   -> impl Stream<Item = (Endpoint, SendStatus)> {

    request_processor_stream
        .map(move |processor_response| {
            let (endpoint, outgoing) = match processor_response {
                Ok((endpoint, outgoing)) => {
                    trace!("Sending `{:?}` to {}", outgoing, endpoint.addr());
                    (endpoint, outgoing)
                },
                Err((endpoint, err)) => {
                    let err_string = format!("{:?}", err);
                    error!("Socket Server's processor yielded an error: {}", err_string);
                    (endpoint, ServerMessages::ProcessorError(err_string))
                },
            };
            // send the message, skipping messages that are programmed not to generate any response
            if outgoing != ServerMessages::None {
                let output_data = SERIALIZER(outgoing);
                let result = handler.network().send(endpoint, &output_data.as_bytes());
                Some((endpoint, result))
            } else {
                None
            }
        })
        .flat_map(|into_iter| stream::iter(into_iter))
}

/// Runs the server until a shutdown is requested.\
/// Incoming requests are feed through `send_to_request_processor()` -- which was generated along with a stream that transforms [ClientMessages] into [ServerMessages];\
/// Once the server is shutdown, `close_request_processor_stream()` is called and waited on.
fn run(handler:                               NodeHandler<()>,
       listener:                              NodeListener<()>,
       addr:                                  SocketAddr,
       mut send_to_request_processor:         impl FnMut(SocketEvent<ClientMessages>) -> bool,
       mut close_request_processor_stream:    impl FnMut()) {

    let mut clients: HashSet<Endpoint> = HashSet::new();

    match handler.network().listen(TRANSPORT, addr) {
        Ok((_id, real_addr)) => info!("Socket Server running at {} by {}", real_addr, TRANSPORT),
        Err(_) => return error!("Cannot listening at {} by {}", addr, TRANSPORT),
    }

    listener.for_each(move |event| match event {
        NodeEvent::Network(net_event) => match net_event {
            NetEvent::Message(endpoint, input_data) => {
                for input_message in input_data.split(|c| *c == '\n' as u8).filter(|&msg| msg.len() > 0) {
                    match DESERIALIZER(input_message) {
                        Ok(incoming) => {
                            trace!("Received `{:?}` from {}", incoming, endpoint.addr());
                            let sent = send_to_request_processor(SocketEvent::Incoming { endpoint, client_message: incoming });
                            if !sent {
                                error!("Server was too busy to process message '{:?}' for {}", std::str::from_utf8(input_message), endpoint.addr());
                                let output_data = SERIALIZER(ServerMessages::TooBusy);
                                handler.network().send(endpoint, &output_data.as_bytes());
                            }
                        },
                        Err(err) => {
                            debug!("Unknown command received from {}: String: {:?}. Bytes: {:?}", endpoint.addr(), std::str::from_utf8(input_message), input_message);
                            let output_data = SERIALIZER(ServerMessages::UnknownMessage(err.to_string()));
                            handler.network().send(endpoint, &output_data.as_bytes());
                        },
                    }
                }
            },
            NetEvent::Connected(endpoint, handshake) => {
                debug!("Unknown connection attempted from '{endpoint}': handshake: {handshake} -- UDP?");
            },
            NetEvent::Accepted(endpoint, listener_id) => {
                clients.insert(endpoint);
                info!("Accepted TCP connection from '{}': listener_id: {} -- client count: {}", endpoint.addr(), listener_id, clients.len());
                send_to_request_processor(SocketEvent::Connected { endpoint });
            },
            NetEvent::Disconnected(endpoint) => {
                clients.remove(&endpoint);
                info!("TCP Disconnected from '{}': -- client count: {}", endpoint.addr(), clients.len());
                send_to_request_processor(SocketEvent::Disconnected { endpoint });
            },
        },
        // shutdown event
        NodeEvent::Signal(_) => {
            // send the shutdown notification to all clients
            warn!("Sending any pending messages");
            close_request_processor_stream();
            //drop(request_processor_stream_producer);
            warn!("Socket Server: Notifying {} client{}", clients.len(), if clients.len() != 1 {"s"} else {""});
            let output_data = SERIALIZER(ServerMessages::ShuttingDown);
            for endpoint in clients.drain() {
                handler.network().send(endpoint, &output_data.as_bytes());
            }
            warn!("Socket Server: telling `message-io` its services are no longer needed");
            handler.stop();
        },
    });
}
