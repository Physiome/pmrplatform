#!/bin/sh

# Grant all permissions to manager role
./target/release/pmrac policy assign private manager '*'
./target/release/pmrac policy assign pending manager '*'
./target/release/pmrac policy assign published manager '*'
./target/release/pmrac policy assign expired manager '*'

# Grant standard permission to reader role
./target/release/pmrac policy assign published reader ''
./target/release/pmrac policy assign expired reader ''

# Create admin user with manager role
./target/release/pmrac user create admin
./target/release/pmrac user password admin set "admin"
./target/release/pmrac role grant admin manager

# Publish default items
./target/release/pmrac resource /workspace/ state published
./target/release/pmrac resource /exposure/ state published

# Policy: owners should have the default ability on their own resources for all workflow states
./target/release/pmrac policy assign private owner ''
./target/release/pmrac policy assign pending owner ''
./target/release/pmrac policy assign published owner ''
./target/release/pmrac policy assign expired owner ''

# TODO this alternative policy will need work, as this requires granting editor to more users than
# necessary.  For now, use the next policy instead.
# Policy: editors can create additional content on published resources
# ./target/release/pmrac policy assign published editor create
# Naturally, standard users should have the following if they are to be permitted to create workspace/exposures
# ./target/release/pmrac resource /workspace/ role grant ${USER} editor
# ./target/release/pmrac resource /exposure/ role grant ${USER} editor

# Policy: readers can 'create' additional items under published resources.  Generally speaking, only the
# `/workspace/` and `/exposure/` endpoint should have this enforcement done.
./target/release/pmrac policy assign published reader 'create'

# Policy: owners can protocol_write under all standard states but when expired.
./target/release/pmrac policy assign private owner 'protocol_write'
./target/release/pmrac policy assign pending owner 'protocol_write'
./target/release/pmrac policy assign published owner 'protocol_write'

# Policy: owners can edit their own data when the state is private
./target/release/pmrac policy assign private owner 'edit'
