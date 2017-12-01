//! # Lycaon Registry
//!
//! The registry is aimed to fix the issues with the current registry
//! options that are currently available
//!
//! There are many features available:
//!

//! - Ability to delete images
//! - replication and masterless
//! - other stuff...

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate args;
extern crate config as cfg;
extern crate ctrlc;
extern crate failure;
extern crate fern;
extern crate futures;
extern crate getopts;
extern crate hostname;
extern crate orset;
extern crate ring;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate uuid;
extern crate protobuf;
extern crate grpcio;

extern crate lycaon_protobuf as grpc;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use(log, warn, info, debug)]
extern crate log;
extern crate env_logger;

#[cfg(test)]
extern crate quickcheck;

use std::thread;
use std::sync::mpsc;

pub mod controller;
pub mod config;
mod errors;
pub mod response;
mod routes;
mod state;
mod test;
mod types;
mod util;

// --- GRPC SERVER ---
use grpc::backend_grpc::Peer;
use futures::Future;

use std::cell::Cell;
#[derive(Clone)]
struct PeerService {
    counter: Cell<u64>,
}
impl PeerService {
    fn new() -> PeerService {
        PeerService { counter: Cell::new(0) }
    }
}

impl Peer for PeerService {
    fn delta_sync(
        &self,
        ctx: grpcio::RpcContext,
        req: grpc::backend::ORSetDelta,
        sink: grpcio::UnarySink<grpc::backend::ORSetDeltaReply>,
    ) {
        self.counter.set(self.counter.get() + 1);
        debug!("Counter: {:?}", self.counter);
        let mut resp = grpc::backend::ORSetDeltaReply::new();
        let deltatype = req.get_deltatype();
        resp.set_deltatype(deltatype);
        resp.set_element("server".to_owned());
        let f = sink.success(resp).map_err(move |e| {
            warn!("failed to reply! {:?}, {:?}", req, e)
        });
        ctx.spawn(f);
    }
}

fn server(config: config::LycaonConfig) {
    use futures::sync::oneshot;
    use std::sync::Arc;
    use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};
    use std::io::Read;

    let listen = config.grpc().listen();

    debug!("starting GRPC server");
    let env = Arc::new(Environment::new(1));
    let service = grpc::backend_grpc::create_peer(PeerService::new());
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind(listen.host(), listen.port())
        .build()
        .unwrap();
    server.start();
    for &(ref host, port) in server.bind_addrs() {
        info!("listening on {}:{}", host, port);
    }
    thread::park();
    let _ = server.shutdown().wait();
    warn!("GRPC Server shutdown!");
}

// --- END GRPC SERVER ---

fn grpc() -> std::thread::JoinHandle<()> {
    debug!("Setting up RPC Server");
    thread::spawn(|| {
        let _ = config::parse_args()
            .and_then(|args| {
                args.value_of("config")
                    .map_err(|e| e.into())
                    .and_then(|file| config::LycaonConfig::new(file))
                    .or_else(|e| config::LycaonConfig::default())
            })
            .map(|config| server(config));
    })
}

fn main() {
    // Init Logger
    let _ = env_logger::init()
        .and(config::main_logger().apply())
        .map_err(|e| {
            warn!("Error setting up logging: {:?}", e);
        });

    // Parse Args
    let args = match config::parse_args() {
        Ok(args) => args,
        Err(_) => {
            std::process::exit(0);
        }

    };

    // GRPC Backend thread
    let _grpc_thread = grpc();

    let (tx_a, rx_a) = mpsc::channel::<config::BackendMessage>();
    let (tx_b, rx_b) = mpsc::channel::<config::BackendMessage>();

    let backend_handler = config::SocketHandler::new(tx_a, rx_b);
    config::rocket(backend_handler, args).map(|rocket| rocket.launch());
}
