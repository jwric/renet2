use std::net::{SocketAddr, UdpSocket};

use super::{ClientSocket, NetcodeError, NetcodeTransportError, ServerSocket};

/// Implementation of [`ServerSocket`] for `UdpSockets`.
#[derive(Debug)]
pub struct NativeSocket {
    socket: UdpSocket,
}

impl NativeSocket {
    /// Makes a new native socket.
    pub fn new(socket: UdpSocket) -> Result<Self, NetcodeError> {
        socket.set_nonblocking(true)?;
        Ok(Self { socket })
    }
}

impl ServerSocket for NativeSocket {
    fn is_encrypted(&self) -> bool {
        false
    }
    fn is_reliable(&self) -> bool {
        false
    }

    fn addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    fn is_closed(&mut self) -> bool {
        false
    }

    fn close(&mut self) {}
    fn connection_denied(&mut self, _: SocketAddr) {}
    fn connection_accepted(&mut self, _: u64, _: SocketAddr) {}
    fn disconnect(&mut self, _: SocketAddr) {}
    fn preupdate(&mut self) {}

    fn try_recv(&mut self, buffer: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buffer)
    }

    fn postupdate(&mut self) {}

    fn send(&mut self, addr: SocketAddr, packet: &[u8]) -> Result<(), NetcodeTransportError> {
        self.socket.send_to(packet, addr)?;
        Ok(())
    }
}

impl ClientSocket for NativeSocket {
    fn is_encrypted(&self) -> bool {
        false
    }
    fn is_reliable(&self) -> bool {
        false
    }

    fn addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    fn is_closed(&mut self) -> bool {
        false
    }

    fn close(&mut self) {}
    fn preupdate(&mut self) {}

    fn try_recv(&mut self, buffer: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buffer)
    }

    fn postupdate(&mut self) {}

    fn send(&mut self, addr: SocketAddr, packet: &[u8]) -> Result<(), NetcodeTransportError> {
        self.socket.send_to(packet, addr)?;
        Ok(())
    }
}
