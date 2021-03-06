use std::{io, net::SocketAddr};
extern crate clap;
use crate::node::Node;
use clap::{value_t, App, Arg};

pub mod helper_fns;
pub mod node;
pub mod quinn_ext;

#[macro_use]
extern crate log;

const LISTEN_ON_IP: &str = "127.0.0.1";
const LISTEN_ON_PORT: u16 = 7878;

//Message size restrictions are 2048 bytes, as mentioned in issue https://github.com/libp2p/rust-libp2p/issues/991
const TX_BYTES: usize = 1000;
const TX_INTERVAL_SEC: usize = 5;

//Node lives this much seconds, then it saves the stats to a file and exits
const NODE_TTL: f64 = 1000.0;

//window size of requests to store and use for statistics
pub const STATS_WINDOW_SIZE: usize = 100;

const TCP_PROTOCOL: &str = "tcp";
const QUIC_PROTOCOL: &str = "quic";

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    //configure_client_untrusted();
    let matches = App::new("TCP p2p example node")
        .version("0.1")
        .author("Egor Ivkov e.o.ivkov@gmail.com")
        .about("-")
        .arg(
            Arg::with_name("connect")
                .short("c")
                .long("connect")
                .value_name("SocketAddr")
                .help("Establishes connection with a swarm through the specified node.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("u16")
                .help("Port to listen on.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("tx_bytes")
                .long("tx_bytes")
                .value_name("usize")
                .help("Number of pending tx data bytes to generate and forward between nodes. Upper limit is 2048 bytes.")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("tx_interval_sec")
                .long("tx_interval_sec")
                .value_name("usize")
                .help("Interval between sending pending transactions. Simulates the process of getting transactions from clients.")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("node_ttl")
                .long("node_ttl")
                .value_name("f64")
                .help("Number of seconds before this node exits and saves stats.")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("stats_window_size")
                .long("stats_window_size")
                .value_name("usize")
                .help("Number of requests/responses that the stats struct stores to calculate mean.")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("transport")
                .short("t")
                .long("transport")
                .value_name("transport_protocol")
                .help("Choose transport layer protocol. Either TCP or QUIC.")
                .default_value("tcp")
                .possible_values([TCP_PROTOCOL, QUIC_PROTOCOL].as_ref())
                .takes_value(true)
        )
        .get_matches();

    let tx_bytes = value_t!(matches, "tx_bytes", usize).unwrap_or(TX_BYTES);
    let tx_interval_sec = value_t!(matches, "tx_interval_sec", usize).unwrap_or(TX_INTERVAL_SEC);
    let node_ttl = value_t!(matches, "node_ttl", f64).unwrap_or(NODE_TTL);
    let stats_window_size =
        value_t!(matches, "stats_window_size", usize).unwrap_or(STATS_WINDOW_SIZE);

    let port = value_t!(matches, "port", u16).unwrap_or(LISTEN_ON_PORT);
    let listen_address = SocketAddr::new(LISTEN_ON_IP.parse().expect("Failed to parse ip."), port);
    let use_quic = match matches.value_of("transport") {
        Some(TCP_PROTOCOL) => false,
        Some(QUIC_PROTOCOL) => true,
        _ => false,
    };
    let node = Node::new(
        listen_address,
        tx_bytes,
        tx_interval_sec,
        node_ttl,
        stats_window_size,
        use_quic,
    );
    match matches.value_of("connect") {
        Some(address) => node.start_and_connect(address).await?,
        None => node.start().await?,
    }
    Ok(())
}
