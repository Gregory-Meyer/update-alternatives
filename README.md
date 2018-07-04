## Synopsis

`update-alternatives` is a utility to manage executable symlinks in
`/usr/local/bin`. It implements a subset of the functionality in Debian's tool
of the same name. Information about alternatives is stored in
`/etc/alternatives` for recovery between invocations of `update-alternatives`.

## Usage

The first invocation of `update-alternatives` will require read-write access to
the directory `/etc/alternatives` should the directory not exist already.

`update-alternatives list NAME` will list all currently installed alternatives
for the link `NAME` and their priority.

`update-alternatives add TARGET NAME PRIORITY` will add or modify the list of
alternatives. There will be an alternative for `NAME` that points to `TARGET`
with numeric priority `PRIORITY` after invocation of this subcommand. You will
require read-write access to `/usr/local/bin` and `/etc/alternatives` to run
this subcommand.

`update-alternatives remove TARGET NAME` will remove the alternative for `NAME`
that points to `TARGET` should there be one. If such an alternative is not
found, this is a no-op. You will require read-write access to
`/usr/local/bin` and `/etc/alternatives` to run this subcommand.

## Installation

Clone this repository, then run `cargo build --release` in the root of the
repository. Copy the executable located in `target/release/update-alternatives`
to your installation directory, such as `/usr/local/bin`.

## Contributors

`update-alternatives` is authored and maintained by Gregory Meyer, and is
licensed under the BSD 3-Clause license.
