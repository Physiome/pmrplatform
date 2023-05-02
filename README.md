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

```console
$ cargo install sqlx-cli
$ touch pmrmodel.db  # create the sqlite file.
$ cat migrations/*/*sql | sqlite3 pmrmodel.db
$ cargo sqlx prepare -- --tests  # ensure queries in tests are included
$ cargo test
$ cargo build
```

## Usage

Generally, this is a library meant as a base for which other components
of PMR are built upon.
