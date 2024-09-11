use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use renetcode2::NETCODE_MAX_PACKET_BYTES;

use crossbeam::channel::{Receiver, Sender};

mod client;
mod server;

pub use client::*;
pub use server::*;

struct InMemoryPacket {
    bytes: [u8; NETCODE_MAX_PACKET_BYTES],
    len: usize,
}

impl Default for InMemoryPacket {
    fn default() -> Self {
        Self {
            bytes: [0u8; NETCODE_MAX_PACKET_BYTES],
            len: 0,
        }
    }
}

const IN_MEMORY_SERVER_ID: u16 = u16::MAX;

/// Produces a [`SocketAddr`] for in-memory server sockets.
///
/// This should be used in the [`ConnectToken::server_addresses`] field for client connection requests, and the
/// [`ServerSocketConfig::public_addresses`] field for setting up servers.
pub fn in_memory_server_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), IN_MEMORY_SERVER_ID)
}

/// Converts a client index into a [`SocketAddr`] for in-memory client sockets.
pub fn in_memory_client_addr(client_id: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), client_id)
}

/// Holds the endpoints of bi-directional channels used by in-memory sockets.
#[derive(Debug, Clone)]
pub struct MemorySocketChannels {
    /// Sends packets to a partner.
    pub(self) sender: Sender<InMemoryPacket>,
    /// Receives packets from a partner.
    pub(self) receiver: Receiver<InMemoryPacket>,
}

impl MemorySocketChannels {
    /// Generates a pair of connected [`MemorySocketChannels`].
    pub fn channel_pair() -> (MemorySocketChannels, MemorySocketChannels) {
        let (sender_a, receiver_a) = crossbeam::channel::unbounded();
        let (sender_b, receiver_b) = crossbeam::channel::unbounded();

        (
            MemorySocketChannels {
                sender: sender_a,
                receiver: receiver_b,
            },
            MemorySocketChannels {
                sender: sender_b,
                receiver: receiver_a,
            },
        )
    }
}

/// Generates in-memory sockets.
///
/// Set the `encrypted` parameter to `true` if you want to pretend that channels are encrypted. If true, then
/// netcode **won't** encrypt packets.
///
/// Returns `(server socket, client sockets)`. Client addresses and client ids are derived from indices into the
/// client socket list.
///
/// ## Panics
///
/// Panics if `num_clients >= u16::MAX`.
pub fn new_memory_sockets(num_clients: usize, encrypted: bool) -> (MemorySocketServer, Vec<MemorySocketClient>) {
    if num_clients >= u16::MAX as usize {
        panic!("only up to u16::MAX - 1 clients are supported for in-memory sockets");
    }
    let mut server_channels = Vec::default();
    let mut client_sockets = Vec::default();
    server_channels.reserve(num_clients);
    client_sockets.reserve(num_clients);

    for client_id in 0..num_clients {
        let (server_chans, client_chans) = MemorySocketChannels::channel_pair();
        server_channels.push(server_chans);
        client_sockets.push(MemorySocketClient::new_with(client_id as u16, client_chans, encrypted));
    }

    let server_socket = MemorySocketServer::new_with(server_channels, encrypted);

    (server_socket, client_sockets)
}
