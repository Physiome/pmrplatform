#!/bin/sh
set -e
cd "$(dirname "$0")"

# This script will build and run the pmrapp_server.

wasm-pack build pmrapp_client --release --target web
cargo run --release --bin pmrapp_server --package pmrapp_server
