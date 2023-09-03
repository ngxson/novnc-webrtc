# noVNC with WebRTC as transport layer

```
⚠️ This project is in very alpha version, bugs are expected
```

## What is that

noVNC, but it uses WebRTC instead of Websocket

```
Browser     <== WebRTC ==>     This app     <== TCP ==>     VNC server
```

- The server app is built on rust, see `/server`
- The UI is taken from [official noVNC repo](https://github.com/novnc/noVNC), with minor adaptations and bundled using vite

## How to use

### Binary

```shell
./novnc-webrtc -h  # show help
./novnc-webrtc --listen "0.0.0.0:6901" --upstream "127.0.0.1:5901"

# then, open browser: http://localhost:6901
```

TODO: prebuilt binary via github actions

### Docker

TODO

## Changelog

v0.1.0: Initial release

## How to build

Firstly, build the frontend. Please note that `vite-plugin-singlefile` is used to bundle the frontend into a single `index.html`

```bash
cd webui_vanilla
npm i
npm run build
```

Then, build the server.

```bash
cd ../server
cargo build -r

# output file: server/target/release/novnc-webrtc
```

Frontend `index.html` will be embedded into the final binary, making it portable and easy to deploy (no need to mess with `dpkg` or `apt-get install`, yay!!)

## TODO

- [ ] Support HTTPS with custom key / certificate
- [ ] Close TCP connection as soon as WebRTC connection is closed
- [ ] Built-in TURN server
- [ ] Auto reconnect
- [ ] Maybe remake frontend using react-ts
