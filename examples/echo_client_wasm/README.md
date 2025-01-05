## Example WASM App

To run:

- Install `wasm-pack`
- Install `npm`

```bash
wasm-pack build \
&& cd wasm-app \
&& npm install \
&& npm run start
```

The app will fall back to websockets for browsers that don't [support](https://caniuse.com/mdn-api_webtransport_webtransport_options_servercertificatehashes_parameter) self-signed webtransport certificates. Note that Firefox `v133.0.*` will not work due to a Firefox webtransport bug (which is undetectable, so we can't fall back to websockets properly).
