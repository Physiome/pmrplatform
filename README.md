# Physiome Model Repository Platform

This is the Cargo workspace for the PMR project, for building the next
generation of Physiome Model Repository.

[![build](https://github.com/Physiome/pmrplatform/actions/workflows/build.yml/badge.svg?branch=main
)](https://github.com/Physiome/pmrplatform/actions/workflows/build.yml?query=branch:main
)

## Build

To build, simply clone this repository, change to that directory, and:

```console
$ cargo build --release
```

### Usage Overview

Once the above command finish successfully, the default binaries will
become available.

Currently, only the `pmrrepo` is of interest, so please run it like so:

```console
$ ./target/release/pmrrepo
```

Which should automatically create the database file in the Cargo
workspace root, using the relative file name defined in the default
`.env` file that also reside there.

To populate that default database file with some useful data for the web
application, the `sync-pmr2-core.sh` helper script may be run to
register locally with a selection of real models from the main [Physiome
Model Repository](https://models.physiomeproject.org/).  Once that
completes successfully, build and run the web application using helper
script `pmrapp.sh`, which should replicate [this demo instance](
https://pmr3.demo.physiomeproject.org/).

For example:

```console
$ ./sync-pmr2-core.sh
Registering workspace with url ...
...
Syncing commits for workspace with id 17...
$ ./pmrapp.sh
[INFO]: Checking for the Wasm target...
[INFO]: Compiling to Wasm...
...
    Finished release [optimized] target(s) in 46.16s
...
[INFO]: Installing wasm-bindgen...
...
[INFO]: :-) Done in 52.74s
...
    Finished release [optimized] target(s) in 2m 31s
     Running `target/release/pmrapp_server`
serving at: http://0.0.0.0:9380
```

At this point, open a browser and point it to http://localhost:9380/ and
hopefully see the front page of the new prototype demo of the Physiome
Model Repository.  There is not much to see yet, but hopefully more
features will be added in the near future.

## Develop

Each member in this workspace are packages that together form the
Physiome Model Repository.  There will be additional README files within
each of them, documenting their use.

One thing of note, the `pmrmodel` packagemakes  use of the
`sqlx::query!` family of macros using offline mode to allow the project
to build without requiring a connection to a database.  The project will
build as normal unless modifications to SQL are required, in which case
the `sqlx-cli` package must be installed, and be used to update the
metadata file so that the package may be built with the updated queries.

On systems with `sh` compatible shell, at the project root, run the
following commands instead of `cargo sqlx prepare`

```console
$ cargo install sqlx-cli
$ ./pmrmodel/sqlx_prepare.sh
```

Otherwise systems that use batch files (i.e. Windows) will need to
replicate those steps manually, or have the database file available.
