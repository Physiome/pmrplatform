# Physiome Model Repository

This is the Cargo workspace for the PMR project, for building the next
generation of Physiome Model Repository.

## Build

To build, simply clone this repository, change to that directory, and:

```console
$ cargo build
```

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

## Usage

If `cargo build` is issued at the root of this workspace, all available
CLI utilities within PMR will be built.  Please refer to the README in
the relevant packages for additional details.
