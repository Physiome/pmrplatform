# pmrmodel

This library provides the core model for the next generation of PMR -
Physiome Model Repository.

## Features

This package contains all the interactions of the various models that
make up PMR, such as storage and retrieval from the underlying database,
processing of model data, and more.

## Build

As this package make use of the `sqlx::query!` family of macros using
offline mode, the `sqlx-cli` package must be installed, and be used to
update the query metadata so that the package may be built.

On systems with `sh` compatible shell, at the package root, run the
following commands instead of `cargo sqlx prepare`

```console
$ cargo install sqlx-cli
$ ./sqlx_prepare.sh
```

Otherwise systems that use batch files (i.e. Windows) will need to
replicate those steps manually, or have the database file available.

## Usage

Given that this package is just a library, it's typically built as part
of the other packages, such as the companions within the workspace this
package resides in.
