#![cfg_attr(docsrs, feature(doc_auto_cfg))]
/*!
Provides integration for [`bevy_replicon`](https://docs.rs/bevy_replicon) for `bevy_renet2`.

# Getting started

This guide assumes that you have already read [quick start guide](https://docs.rs/bevy_replicon#quick-start) from `bevy_replicon`.

All Renet API is re-exported from this plugin, you don't need to include `bevy_renet` or `renet` to your `Cargo.toml`.

Renet by default uses the netcode transport which is re-exported by the `transport` feature. If you want to use other transports, you can disable it.

## Initialization

Add [`RepliconRenetPlugins`] along with [`RepliconPlugins`](bevy_replicon::prelude::RepliconPlugins):

```
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::RepliconRenetPlugins;

let mut app = App::new();
app.add_plugins((MinimalPlugins, RepliconPlugins, RepliconRenetPlugins));
```

Plugins in [`RepliconRenetPlugins`] automatically add `renet2` plugins, you don't need to add them.

If the `transport` feature is enabled, netcode plugins will also be automatically added.

## Server and client creation

To connect to the server or create it, you need to initialize the
[`RenetClient`](renet2::RenetClient) and [`NetcodeClientTransport`](renet2::transport::NetcodeClientTransport) **or**
[`RenetServer`](renet2::RenetServer) and [`NetcodeServerTransport`](renet2::transport::NetcodeServerTransport) resources from Renet.

Never insert client and server resources in the same app for single-player, it will cause a replication loop.

This crate provides the [`RenetChannelsExt`] extension trait to conveniently convert channels
from the [`RepliconChannels`] resource into renet2 channels.
When creating a server or client you need to use a [`ConnectionConfig`](renet2::ConnectionConfig)
from [`renet2`], which can be initialized like this:

```
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::{renet2::ConnectionConfig, RenetChannelsExt, RepliconRenetPlugins};

# let mut app = App::new();
# app.add_plugins(RepliconPlugins);
let channels = app.world().resource::<RepliconChannels>();
let connection_config = ConnectionConfig::from_channels(
    channels.get_server_configs(),
    channels.get_client_configs(),
);
```

For a full example of how to initialize a server or client see the example in the
repository.
*/

pub use bevy_renet2::renet2;
#[cfg(feature = "transport")]
pub use bevy_renet2::transport;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub use client::*;
#[cfg(feature = "server")]
pub use server::*;

use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_replicon::{
    core::ClientId,
    prelude::{ChannelKind, RepliconChannel, RepliconChannels},
};
use renet2::{ChannelConfig, SendType};

pub struct RepliconRenetPlugins;

impl PluginGroup for RepliconRenetPlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut builder = PluginGroupBuilder::start::<Self>();

        #[cfg(feature = "client")]
        {
            builder = builder.add(RepliconRenetClientPlugin);
        }

        #[cfg(feature = "server")]
        {
            builder = builder.add(RepliconRenetServerPlugin);
        }

        builder
    }
}

/// External trait for [`RepliconChannels`] to provide convenient conversion into renet channel configs.
pub trait RenetChannelsExt {
    /// Returns server channel configs that can be used to create [`ConnectionConfig`](renet2::ConnectionConfig).
    fn get_server_configs(&self) -> Vec<ChannelConfig>;

    /// Same as [`RenetChannelsExt::get_server_configs`], but for clients.
    fn get_client_configs(&self) -> Vec<ChannelConfig>;
}

impl RenetChannelsExt for RepliconChannels {
    fn get_server_configs(&self) -> Vec<ChannelConfig> {
        create_configs(self.server_channels(), self.default_max_bytes)
    }

    fn get_client_configs(&self) -> Vec<ChannelConfig> {
        create_configs(self.client_channels(), self.default_max_bytes)
    }
}

/// Converts replicon channels into renet channel configs.
fn create_configs(channels: &[RepliconChannel], default_max_bytes: usize) -> Vec<ChannelConfig> {
    let mut channel_configs = Vec::with_capacity(channels.len());
    for (index, channel) in channels.iter().enumerate() {
        let send_type = match channel.kind {
            ChannelKind::Unreliable => SendType::Unreliable,
            ChannelKind::Unordered => SendType::ReliableUnordered {
                resend_time: channel.resend_time,
            },
            ChannelKind::Ordered => SendType::ReliableOrdered {
                resend_time: channel.resend_time,
            },
        };
        channel_configs.push(ChannelConfig {
            channel_id: index as u8,
            max_memory_usage_bytes: channel.max_bytes.unwrap_or(default_max_bytes),
            send_type,
        });
    }
    channel_configs
}

/// External trait for [`ClientId`] to provide convenient conversion into [`renet2::ClientId`].
pub trait Renet2ClientIdExt {
    /// Returns renet's Client ID.
    fn to_renet2(&self) -> renet2::ClientId;
}
impl Renet2ClientIdExt for ClientId {
    fn to_renet2(&self) -> renet2::ClientId {
        renet2::ClientId::from_raw(self.get())
    }
}
/// External trait for [`renet2::ClientId`] to provide convenient conversion into [`ClientId`].
pub trait ClientIdExt {
    /// Returns replicon's Client ID.
    fn to_replicon(&self) -> ClientId;
}
impl ClientIdExt for renet2::ClientId {
    fn to_replicon(&self) -> ClientId {
        ClientId::new(self.raw())
    }
}
