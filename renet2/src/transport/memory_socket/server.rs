use std::{io::ErrorKind, net::SocketAddr};

use crate::transport::{NetcodeTransportError, ServerSocket};

use super::*;

/// Implementation of [`ServerSocket`] for a server socket using in-memory channels.
///
/// In-memory sockets are treated as encrypted and reliable by default for efficiency. Use [`Self::new_with`] to use
/// a different policy (this is useful for performance tests).
#[derive(Debug)]
pub struct MemorySocketServer {
    clients: Vec<(u16, MemorySocketChannels)>,
    encrypted: bool,
    reliable: bool,
    drain_index: usize,
}

impl MemorySocketServer {
    /// Makes a new in-memory socket for a server.
    ///
    /// Takes a vector of `(client id, socket channels)`.
    pub fn new(clients: Vec<(u16, MemorySocketChannels)>) -> Self {
        Self::new_with(clients, true, true)
    }

    /// Makes a new in-memory socket for a server with a specific encryption policy.
    ///
    /// Takes a vector of `(client id, socket channels)`.
    ///
    /// If `encrypted` is set to `true` then the memory transport will be treated *as if* it were encrypted.
    /// If you want `renet2` to encrypt the channel, set it to `false`.
    ///
    /// If `reliable` is set to `true` then the memory transport will downgrade all channels to unreliable.
    /// If you don't want to downgrade channels (e.g. for performance testing), set it to false.
    pub fn new_with(clients: Vec<(u16, MemorySocketChannels)>, encrypted: bool, reliable: bool) -> Self {
        Self {
            clients,
            encrypted,
            reliable,
            drain_index: 0,
        }
    }
}

impl ServerSocket for MemorySocketServer {
    fn is_encrypted(&self) -> bool {
        self.encrypted
    }
    fn is_reliable(&self) -> bool {
        self.reliable
    }

    fn addr(&self) -> std::io::Result<SocketAddr> {
        Ok(in_memory_server_addr())
    }

    fn is_closed(&mut self) -> bool {
        false
    }

    fn close(&mut self) {}
    fn connection_denied(&mut self, _: SocketAddr) {}
    fn connection_accepted(&mut self, _: u64, _: SocketAddr) {}
    fn disconnect(&mut self, _: SocketAddr) {}
    fn preupdate(&mut self) {
        self.drain_index = 0;
    }

    fn try_recv(&mut self, buffer: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        loop {
            if self.drain_index >= self.clients.len() {
                return Err(std::io::Error::from(ErrorKind::WouldBlock));
            }

            let client = &mut self.clients[self.drain_index];
            if let Ok(packet) = client.1.receiver.try_recv() {
                buffer[..packet.len].copy_from_slice(&packet.bytes[..packet.len]);
                return Ok((packet.len, in_memory_client_addr(client.0)));
            };

            self.drain_index += 1;
        }
    }

    fn postupdate(&mut self) {}

    fn send(&mut self, addr: SocketAddr, packet: &[u8]) -> Result<(), NetcodeTransportError> {
        assert!(packet.len() <= NETCODE_MAX_PACKET_BYTES);

        let client_id = addr.port();

        let mut mem_packet = InMemoryPacket {
            len: packet.len(),
            ..Default::default()
        };
        mem_packet.bytes[..packet.len()].copy_from_slice(packet);
        let idx = self
            .clients
            .iter()
            .position(|(id, _)| *id == client_id)
            .ok_or_else(|| std::io::Error::from(ErrorKind::AddrNotAvailable))?;
        self.clients[idx]
            .1
            .sender
            .send(mem_packet)
            .map_err(|_| std::io::Error::from(ErrorKind::ConnectionAborted))?;

        Ok(())
    }
}
