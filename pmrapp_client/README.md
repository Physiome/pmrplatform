# pmrapp\_client

The client library for the PMR application.

## Features

This client library can be used to produce HTML DOM representation of
the underlying models of PMR.

The library can be compiled to native code and WebAssembly.

## Build

In order for `pmrapp_server` be able to provide the instructions to
enable client side rendering for its clients, the Wasm binary must be
built first.  The pre-requsite for this is at the minimum `rustc 1.30.0`
and then following [its installation instructions](
https://rustwasm.github.io/docs/wasm-pack/)

When `wasm-pack` is available, build using the following command:

```console
$ wasm-pack build pmrapp_client --release --target web
```

## Usage

Typically, this is used from another application or package, such as
the `pmrapp_server` package.
