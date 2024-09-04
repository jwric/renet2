# CHANGELOG

## WIP

- Remove `bevy_renet2` dependency on `bevy_window`.
- Properly clean up WebTransport client's reader stream.
- Update to `bevy_replicon` v0.28.1.

## 0.0.5 - 07/04/2024

- Update to Bevy v0.14.

## 0.0.4 - 06/26/2024

- Update `WebTransportClientConfig` to use `WebServerDestination`, which allows connecting to a WebTransport server via URL (useful when your server has certs for a domain name).
- Update `h3` dependencies for the WebTransport server in the `renet2` crate to depend on the `h3-v0.0.4` tag.
- Fix `disconnect_on_exit`. See [renet #158](https://github.com/lucaspoffo/renet/pull/158).
- Bump `bevy_replicon_renet2` to v0.0.4 for `bevy_replicon` v0.26.
- Loosen `cfg` on `webtransport_socket` module.

## 0.0.3 - 05/24/2024

- Add `bevy_replicon_renet2` sub-crate.
- Add `client_should_update` run condition to `bevy_renet2` to fix disconnect bug.

## 0.0.2 - 05/07/2024

- Fix WebTransport server panicking on construction when not inside a tokio runtime.

## 0.0.1 - 03/29/2024

- Forked from `renet`.
    - Implement `Reflect` on `ClientId`. See [renet #130](https://github.com/lucaspoffo/renet/pull/130).
    - Optimize `bevy_renet2` builds. See [renet #104](https://github.com/lucaspoffo/renet/pull/104).
    - Refactor RenetClient so channels are accessed more efficiently. See [renet #154](https://github.com/lucaspoffo/renet/pull/154).
    - Update `bevy_renet2` so client systems don't run when the client is disconnected. See [renet #134](https://github.com/lucaspoffo/renet/pull/134).
    - Add `TransportSocket` trait for injecting the source of unreliable packets to netcode transports. See [renet #145](https://github.com/lucaspoffo/renet/pull/145).
    - Add optional encryption to `renetcode2` to support sockets that handle encryption internally. See [renet #149](https://github.com/lucaspoffo/renet/pull/149).
    - Refactor `NetcodeServer` to allow multiple underlying sockets. See [renet #150](https://github.com/lucaspoffo/renet/pull/150).
    - Add memory-channels transport socket. See [renet #117](https://github.com/lucaspoffo/renet/pull/117).
    - Add WebTransport server and client implementations of TransportSocket. See [renet #107](https://github.com/lucaspoffo/renet/pull/107).
