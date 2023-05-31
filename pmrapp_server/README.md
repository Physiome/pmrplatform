# pmrapp\_server

This package provides the PMR application server.

## Features

This package provides a binary that will contain everything necessary
to serve the Physiome Model Repository web application.

## Build

In order for this to build, please ensure the Wasm binary from the
accompanied `pmrapp_client` package with the Cargo workspace was built.
Once that's done, the following build command may be issued from the
root of the Cargo workspace:

```console
cargo build --release --bin pmrapp_server --package pmrapp_server
```

Note that `--package` must be specified as this is not one of the default
members of the PMR3 Cargo workspace.

## Usage

Simply run the binary and connect to the application end point.
