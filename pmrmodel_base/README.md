# pmrmodel\_base

This library provides the data structures for the base model of the next
generation of Physiome Model Repository.

## Usage

Given that this package is just a library, it's typically built as part
of the other packages, such as the companions within the workspace this
package resides in.

### Cargo Feature Flags

While the goal of this library is to provide data structures, there are
implementations of traits here to avoid the orphan rule.  As this
typically require external dependencies that will make these simple data
structures from being used, all of the non-core ones can be disabled.

The following are the flags that may be enabled to allow the usage of
relevant implementation of traits

- `sqlx`: Provide implementations of `sqlx::FromRow` for various types
