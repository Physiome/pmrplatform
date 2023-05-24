# pmrmodel

This library provides the core model for the next generation of PMR -
Physiome Model Repository.

## Build

To build, simply clone this repository, change to that directory, and:

```console
$ cargo build
```

## Develop

As this package make use of the `sqlx::query!` family of macros using
offline mode, the `sqlx-cli` package must be installed, and be used to
update the query metadata so that the package may be built.

On systems with `sh` compatible shell, at the project root, run the
following commands instead of `cargo sqlx prepare`

```console
$ cargo install sqlx-cli
$ ./pmrmodel/sqlx_prepare.sh
```

Otherwise systems that use batch files (i.e. Windows) will need to
replicate those steps manually, or have the database file available.

## Usage

Generally, this is a library meant as a base for which other components
of PMR are built upon.
