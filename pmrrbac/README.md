# pmrrbac

Role-Based Access Control for PMR

## Features

This package uses Casbin to provide the framework to enforce role-based
access control for resources within PMR when accessed through the web
application.

## Build

Simply run `cargo build`.

## Status

Currently, the default model and basic set of policies should cover the
main use cases for managing access to the underlying resources at every
workflow state of a resource's lifecycle.  While the underlying API may
be adapted to other use cases, the API calls are simply wrappers around
Casbin with the expectations of the default model which may impede this
idea.
