## Example cross-platform server using native UDP sockets and WebTransport

This example is set up to use self-signed certificates for WebTransport clients. WebTransport clients will use an HTTP request to get the current cert automatically.

In practice you'd pass the server's certificate hash through a secure channel to the client (e.g. via an authenticated channel set up when a user connects to your frontend).

### Certificate generation

If you don't want to use a self-signed certificate workflow/security model, then you'll need to generate actual certificates for testing.

- Install [mkcert](https://github.com/FiloSottile/mkcert)
- Run `generate_cert.sh`
    - The script will generate a certificate and a key file in the current directory and install the CA certificate in the local certificate store and also in supported browsers.
- You should now see in the console the path to the CA certificate.
- Install the generated CA certificate in your browser, if the browser is not supported by `mkcert`.
- Load the certificate to `renet2` with `get_certificate_and_key_from_files()` from the WebTransport cert utils provided by `renet2`. Use the cert/key obtained when constructing `WebTransportServerConfig`, instead of `WebTransportServerConfig::new_selfsigned()`.

The certificate from that script is only valid for `localhost`, `127.0.0.1` (Ipv4), and `::1` (Ipv6). If you want to use a different hostname, you need to change the script.

- Alternative way for Linux/MacOS: see https://github.com/hyperium/h3/tree/master/examples
