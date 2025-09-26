# g2dem-web

GNU v2 g++ demangler, running completely in your web browser.

This site is available at <https://decompollaborate.github.io/gnuv2_demangle>.

This application is meant to be built to a WASM target and be used as an static
website running completely on the client, without the need of any webserver.

## Running

g2dem-web can be built and hosted locally, mainly for local development.

### Dependencies

This site is built as a WASM application, meaning we need the WASM Rust target
installed. I recommend using [`trunk`](https://trunkrs.dev/) to serve the
application locally.

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

### Serving

In your terminal navigate into the `g2dem-web` folder (i.e.
`cd src/g2dem-web`) and run `trunk`:

```bash
trunk serve --release --open
```

This will get build the aplication, and open it in your default browser. `trunk`
will watch for changes in the code and rebuild if any is made.
