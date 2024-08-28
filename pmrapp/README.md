# pmrapp - Web application for serving Physiome Model Repository

This is the web application service for hosting a running instance of
the next generation of the Physiome Model Repository platform.  It is
built on top of the [Leptos](https://github.com/leptos-rs/leptos) web
framework for the UI and [Axum](https://github.com/tokio-rs/axum) web
application framework for the integration between various components.

## Building `pmrapp`

The front-end does depend on JavaScript sourced from `npm`, so Node.js
will also need to be available.  Install and build the JavaScript bundle
with the following commands:

```bash
npm install
npx webpack
```

Ensure the Rust compiler can produce the wasm target, which may be done
using `rustup target add wasm32-unknown-unknown`.

The `cargo-leptos` tool streamlines the whole development and build
process, install it with:

```bash
cargo install cargo-leptos --locked
```

Once ready, simply run:

```bash
cargo leptos watch
```

Which will start the application server, and once it started point your
browser to `http://127.0.0.1:3000`.

## Compiling for Release

```bash
cargo leptos build --release
```

Will generate your server binary in target/server/release and your site
package in target/site.

## Testing

TODO

Though once we have tests, the following may be done:

```bash
cargo leptos end-to-end
```

```bash
cargo leptos end-to-end --release
```

Cargo-leptos uses Playwright as the end-to-end test tool.  
Tests are located in end2end/tests directory.
