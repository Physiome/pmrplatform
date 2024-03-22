# pmrrepo

A demo of the next generation of the PMR core repository, written in
Rust.

## Features

This project provides the support for PMR Workspaces.  In this demo, the
data storage for models and data is Git, while the metadata for the
management and bookkeeping of workspace and related data is currently
stored in SQLite, though the goal is to provide this backend in both
SQLite and PostgreSQL for production usage.

## Build

To build, simply clone the cargo workspace this package resides in,
change to the directory where the repository was cloned to, and:

```console
$ cargo build --release
```

## Configuration

The database source is defined in the `.env` file at the workspace root,
and the binary will make use of the `.env` file found in the current
working directory for the values required.  If modifications to these
values are needed, it's recommend that a copy be made to a different
directory, and run the `pmrrepo` binary while making that the current
working directory.

The following usage examples below assumes no modifications being done
to that file, and that the current working directory remains in the
root of the local clone of this project.

## Usage

Once the build process is completed, starting the binary may produce the
following output:

```console
$ ./target/release/pmrrepo
2023-02-23T00:00:00+00:00 - WARN - database sqlite:workspace.db does not exist; creating...
Printing list of all workspaces
id - url - description
```

To make some workspaces present, a quick way is to import some existing
workspaces from the existing repository.  First, define a few workspaces
from there as the source by using the register subcommand:

```console
$ ./target/release/pmrrepo register https://models.physiomeproject.org/workspace/beeler_reuter_1977 'Beeler, Reuter, 1977'
Registering workspace with url 'https://models.physiomeproject.org/workspace/beeler_reuter_1977'...
Registered workspace with id 1
$ ./target/release/pmrrepo register https://models.physiomeproject.org/workspace/hodgkin_huxley_1952 'Hodgkin, Huxley, 1952'
Registering workspace with url 'https://models.physiomeproject.org/workspace/hodgkin_huxley_1952'...
Registered workspace with id 2
$ ./target/release/pmrrepo register https://models.physiomeproject.org/workspace/noble_1962 'Noble, 1962'
Registering workspace with url 'https://models.physiomeproject.org/workspace/noble_1962'...
Registered workspace with id 3
```

To bring the data in, synchronize the remote data to local:

```console
$ ./target/release/pmrrepo sync 1
Syncing commits for workspace with id 1...
$ ./target/release/pmrrepo sync 2
Syncing commits for workspace with id 2...
$ ./target/release/pmrrepo sync 3
Syncing commits for workspace with id 3...
```

This would create a directory (defined by the `PMR_REPO_ROOT` environment
variable), and a clone of the workspaces (as a bare git repo) will be
created for each of the ids.

```console
$ ls ./repos
1  2  3
```

## Issues

Currently, the schema will change and database migration steps may
abruptly get changed, as this package is currently undergoing rapid
iteration.
