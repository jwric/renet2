use crate::channel::ChannelConfig;
use crate::client::{LocalClient, LocalClientConnected};
use crate::error::{DisconnectionReason, MessageError};
use crate::packet::Payload;
use crate::remote_connection::{ConnectionConfig, NetworkInfo, RemoteConnection};
use crate::{ClientId, ConnectionControl, Transport};

use log::{error, info};

use std::collections::HashMap;
use std::collections::VecDeque;
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionPermission {
    /// All connection are allowed, excepet clients that are denied.
    All,
    /// Only clients in the allow list can connect.
    OnlyAllowed,
    /// No connection can be stablished.
    None,
}

impl Default for ConnectionPermission {
    fn default() -> Self {
        ConnectionPermission::All
    }
}

pub struct ServerConfig {
    pub max_clients: usize,
    pub max_payload_size: usize,
}

impl ServerConfig {
    pub fn new(max_clients: usize, max_payload_size: usize) -> Self {
        Self {
            max_clients,
            max_payload_size,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_clients: 16,
            max_payload_size: 8 * 1024,
        }
    }
}

#[derive(Debug)]
pub enum ServerEvent<C> {
    ClientConnected(C),
    ClientDisconnected(C),
}

pub struct Server<C, T> {
    config: ServerConfig,
    transport: T,
    remote_clients: HashMap<C, RemoteConnection<C>>,
    local_clients: HashMap<C, LocalClient<C>>,
    // connecting: HashMap<C, RemoteConnection<P, C>>,
    channels_config: HashMap<u8, Box<dyn ChannelConfig>>,
    events: VecDeque<ServerEvent<C>>,
    connection_config: ConnectionConfig,
    connection_control: ConnectionControl<C>,
}

impl<C, T> Server<C, T>
where
    T: Transport + Transport<ClientId = C>,
    C: ClientId,
{
    pub fn new(
        transport: T,
        config: ServerConfig,
        connection_config: ConnectionConfig,
        connection_permission: ConnectionPermission,
        channels_config: HashMap<u8, Box<dyn ChannelConfig>>,
    ) -> Self {
        let connection_control = ConnectionControl::new(connection_permission);

        Self {
            transport,
            remote_clients: HashMap::new(),
            local_clients: HashMap::new(),
            // connecting: HashMap::new(),
            config,
            channels_config,
            connection_config,
            connection_control,
            events: VecDeque::new(),
        }
    }

    //pub fn addr(&self) -> Option<SocketAddr> {
    //    self.socket.local_addr().ok()
    //}

    pub fn has_clients(&self) -> bool {
        !self.remote_clients.is_empty() || !self.local_clients.is_empty()
    }

    pub fn get_event(&mut self) -> Option<ServerEvent<C>> {
        self.events.pop_front()
    }
    /*
        fn find_client_by_addr(&mut self, addr: &SocketAddr) -> Option<&mut RemoteConnection<P, C>> {
            self.remote_clients
                .values_mut()
                .find(|c| *c.addr() == *addr)
        }

        fn find_connecting_by_addr(
            &mut self,
            addr: &SocketAddr,
        ) -> Option<&mut RemoteConnection<P, C>> {
            self.connecting.values_mut().find(|c| c.addr() == addr)
        }
    */
    pub fn allow_client(&mut self, client_id: &C) {
        self.connection_control.allow_client(client_id);
    }

    pub fn allow_connected_clients(&mut self) {
        for client_id in self.get_clients_id().iter() {
            self.allow_client(client_id);
        }
    }

    pub fn deny_client(&mut self, client_id: &C) {
        self.connection_control.deny_client(client_id);
    }

    pub fn set_connection_permission(&mut self, connection_permission: ConnectionPermission) {
        self.connection_control
            .set_connection_permission(connection_permission);
    }

    pub fn connection_permission(&self) -> &ConnectionPermission {
        self.connection_control.connection_permission()
    }

    pub fn allowed_clients(&self) -> Vec<C> {
        self.connection_control.allowed_clients()
    }

    pub fn denied_clients(&self) -> Vec<C> {
        self.connection_control.denied_clients()
    }

    pub fn network_info(&self, client_id: C) -> Option<&NetworkInfo> {
        if let Some(connection) = self.remote_clients.get(&client_id) {
            return Some(connection.network_info());
        }
        None
    }

    pub fn create_local_client(&mut self, client_id: C) -> LocalClientConnected<C> {
        let channels = self.channels_config.keys().copied().collect();
        self.events
            .push_back(ServerEvent::ClientConnected(client_id));
        let (local_client_connected, local_client) = LocalClientConnected::new(client_id, channels);
        self.local_clients.insert(client_id, local_client);
        local_client_connected
    }

    pub fn disconnect(&mut self, client_id: &C) {
        if let Some(remote_client) = self.remote_clients.remove(&client_id) {
            self.events
                .push_back(ServerEvent::ClientDisconnected(*client_id));
            self.transport.disconnect(
                remote_client.client_id(),
                DisconnectionReason::DisconnectedByServer,
            );
        } else if let Some(mut local_client) = self.local_clients.remove(&client_id) {
            local_client.disconnect();
            self.events
                .push_back(ServerEvent::ClientDisconnected(*client_id));
        }
    }

    pub fn disconnect_clients(&mut self) {
        for client_id in self.get_clients_id().iter() {
            self.disconnect(client_id);
        }
    }

    pub fn send_message<ChannelId: Into<u8>>(
        &mut self,
        client_id: C,
        channel_id: ChannelId,
        message: Vec<u8>,
    ) -> Result<(), MessageError> {
        let channel_id = channel_id.into();
        if let Some(remote_connection) = self.remote_clients.get_mut(&client_id) {
            remote_connection.send_message(channel_id, message)
        } else if let Some(local_connection) = self.local_clients.get_mut(&client_id) {
            local_connection.send_message(channel_id, message)
        } else {
            Err(MessageError::ClientNotFound)
        }
    }

    pub fn broadcast_message<ChannelId: Into<u8>>(
        &mut self,
        channel_id: ChannelId,
        message: Payload,
    ) {
        let channel_id = channel_id.into();
        for remote_client in self.remote_clients.values_mut() {
            if let Err(e) = remote_client.send_message(channel_id, message.clone()) {
                error!("{}", e);
            }
        }

        for local_client in self.local_clients.values_mut() {
            if let Err(e) = local_client.send_message(channel_id, message.clone()) {
                error!("{}", e);
            }
        }
    }

    pub fn broadcast_message_remote<ChannelId: Into<u8>>(
        &mut self,
        channel_id: ChannelId,
        message: Payload,
    ) {
        let channel_id = channel_id.into();
        for remote_client in self.remote_clients.values_mut() {
            if let Err(e) = remote_client.send_message(channel_id, message.clone()) {
                error!("{}", e);
            }
        }
    }

    pub fn broadcast_message_local<ChannelId: Into<u8>>(
        &mut self,
        channel_id: ChannelId,
        message: Payload,
    ) {
        let channel_id = channel_id.into();
        for local_client in self.local_clients.values_mut() {
            if let Err(e) = local_client.send_message(channel_id, message.clone()) {
                error!("{}", e);
            }
        }
    }

    pub fn receive_message<ChannelId: Into<u8>>(
        &mut self,
        client_id: C,
        channel_id: ChannelId,
    ) -> Result<Option<Payload>, MessageError> {
        let channel_id = channel_id.into();
        if let Some(remote_client) = self.remote_clients.get_mut(&client_id) {
            remote_client.receive_message(channel_id)
        } else if let Some(local_client) = self.local_clients.get_mut(&client_id) {
            local_client.receive_message(channel_id)
        } else {
            Err(MessageError::ClientNotFound)
        }
    }

    pub fn get_clients_id(&self) -> Vec<C> {
        self.remote_clients
            .keys()
            .copied()
            .chain(self.local_clients.keys().copied())
            .collect()
    }

    pub fn is_client_connected(&self, client_id: &C) -> bool {
        self.remote_clients.contains_key(client_id) || self.local_clients.contains_key(client_id)
    }

    pub fn update(&mut self) -> Result<(), io::Error> {
        while let Ok(Some((client_id, payload))) = self.transport.recv(&self.connection_control) {
            if let Some(client) = self.remote_clients.get_mut(&client_id) {
                if let Err(e) = client.process_payload(&payload) {
                    error!("Error processing client payload: {}", e);
                }
            }
        }

        // TODO(transport): Check if there is disconnect on the transport layer

        let mut disconnected_remote_clients: Vec<C> = vec![];
        for (&client_id, connection) in self.remote_clients.iter_mut() {
            if connection.update().is_err() {
                disconnected_remote_clients.push(client_id);
            }
        }

        for &client_id in disconnected_remote_clients.iter() {
            self.remote_clients.remove(&client_id);
            self.events
                .push_back(ServerEvent::ClientDisconnected(client_id));
            info!("Remote Client {} disconnected.", client_id);
        }

        let mut disconnected_local_clients: Vec<C> = vec![];
        for (&client_id, connection) in self.local_clients.iter() {
            if !connection.is_connected() {
                disconnected_local_clients.push(client_id);
            }
        }

        for &client_id in disconnected_local_clients.iter() {
            self.local_clients.remove(&client_id);
            self.events
                .push_back(ServerEvent::ClientDisconnected(client_id));
            info!("Local Client {} disconnected.", client_id);
        }

        // TODO(transport): review the connection state
        for client_id in self.transport.update() {
            if self.remote_clients.len() + self.local_clients.len() >= self.config.max_clients {
                self.transport
                    .disconnect(client_id, DisconnectionReason::MaxPlayer);
                info!(
                    "Connection from {} successfuly stablished but server was full.",
                    client_id
                );

                continue;
            } else {
                self.transport.confirm_connect(client_id);
            }
        }

        /*
        match self.update_pending_connections() {
            Err(RenetError::IOError(e)) => Err(e),
            Err(e) => {
                error!("Error updating pending connections: {}", e);
                Ok(())
            }
            Ok(()) => Ok(()),
        }
        */
        Ok(())
    }

    /*
        fn update_pending_connections(&mut self) -> Result<(), RenetError> {
            let mut connected_clients = vec![];
            let mut disconnected_clients = vec![];
            for connection in self.connecting.values_mut() {
                if connection.update().is_err() {
                    disconnected_clients.push(connection.client_id());
                    continue;
                }

                if connection.is_connected() {
                    connected_clients.push(connection.client_id());
                } else if let Ok(Some(payload)) = connection.create_protocol_payload() {
                    let packet = Packet::Unauthenticaded(Unauthenticaded::Protocol { payload });
                    try_send_packet(&self.socket, packet, connection.addr())?;
                }
            }


            for client_id in disconnected_clients {
                self.connecting
                    .remove(&client_id)
                    .expect("Disconnected Clients always exists");
                info!("Request connection {} failed.", client_id);
            }

            for client_id in connected_clients {
                let mut connection = self
                    .connecting
                    .remove(&client_id)
                    .expect("Connected Client always exist");
                if self.remote_clients.len() >= self.config.max_clients {
                    info!(
                        "Connection from {} successfuly stablished but server was full.",
                        connection.addr()
                    );
                    let packet = Unauthenticaded::ConnectionError(DisconnectionReason::MaxPlayer);
                    try_send_packet(&self.socket, packet, connection.addr())?;

                    continue;
                }

                info!(
                    "Connection stablished with client {} ({}).",
                    connection.client_id(),
                    connection.addr(),
                );

                for (channel_id, channel_config) in self.channels_config.iter() {
                    let channel = channel_config.new_channel();
                    connection.add_channel(*channel_id, channel);
                }

                self.events
                    .push_back(ServerEvent::ClientConnected(connection.client_id()));
                self.remote_clients
                    .insert(connection.client_id(), connection);
            }


            Ok(())
        }
    */

    pub fn send_packets(&mut self) {
        for (client_id, connection) in self.remote_clients.iter_mut() {
            if let Err(e) = connection.send_packets(&mut self.transport) {
                error!("Failed to send packet for Client {}: {:?}", client_id, e);
            }
        }
    }
    /*
        fn process_payload_from(
            &mut self,
            payload: &[u8],
            addr: &SocketAddr,
        ) -> Result<(), RenetError> {
            if let Some(client) = self.find_client_by_addr(addr) {
                return client.process_payload(payload);
            }

            if self.remote_clients.len() >= self.config.max_clients {
                let packet = Unauthenticaded::ConnectionError(DisconnectionReason::MaxPlayer);
                try_send_packet(&self.socket, packet, addr)?;
                debug!("Connection Denied to addr {}, server is full.", addr);
                return Ok(());
            }

            match self.find_connecting_by_addr(addr) {
                Some(connection) => {
                    return connection.process_payload(payload);
                }
                None => {
                    let packet = bincode::deserialize::<Packet>(payload)?;
                    if let Packet::Unauthenticaded(Unauthenticaded::Protocol { payload }) = packet {
                        let protocol =
                            P::from_payload(&payload).map_err(RenetError::AuthenticationError)?;
                        let id = protocol.id();
                        if !self.can_client_connect(id) {
                            info!(
                                "Client with id {} is not allowed to connect the server.",
                                id
                            );

                            let packet = Unauthenticaded::ConnectionError(DisconnectionReason::Denied);
                            try_send_packet(&self.socket, packet, addr)?;
                        }

                        if self.is_client_connected(&id) {
                            info!(
                                "Client with id {} already connected, discarded connection attempt.",
                                id
                            );
                            let packet = Unauthenticaded::ConnectionError(
                                DisconnectionReason::ClientIdAlreadyConnected,
                            );
                            try_send_packet(&self.socket, packet, addr)?;
                        } else {
                            info!("Created new protocol from payload with client id {}", id);
                            let new_connection = RemoteConnection::new(
                                protocol.id(),
                                self.connection_config.clone(),
                                protocol,
                            );
                            self.connecting.insert(id, new_connection);
                        }
                    }
                }
            };

            Ok(())
        }

    */
}
