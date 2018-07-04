## Status

![Travis build status][travis]

[travis]: https://travis-ci.org/Gregory-Meyer/update-alternatives.svg?branch=master

## Synopsis

`update-alternatives` is a utility to manage executable symlinks in
`/usr/local/bin`. It implements a subset of the functionality in Debian's tool
of the same name. Information about alternatives is stored in
`/etc/alternatives` for recovery between invocations of `update-alternatives`.

## Example

```sh
$ sudo update-alternatives add $(which gcc) cc 10
using /usr/bin/gcc for cc with priority 10
$ sudo update-alternatives add $(which clang) cc 20
found link cc with 1 alternatives
added alternative /usr/bin/clang with priority 20
using /usr/bin/clang for cc with priority 20
$ update-alternatives list cc
found link cc with 2 alternatives
alternatives for cc:
    /usr/bin/gcc: 10
    /usr/bin/clang: 20
$ update-alternatives remove $(which gcc) cc
found link cc with 2 alternatives
removed alternative /usr/bin/gcc
using /usr/bin/clang for cc with priority 20
$ update-alternatives list cc
found link cc with 1 alternatives
alternatives for cc:
    /usr/bin/clang: 20
```

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

## Motivation

Arch Linux (naturally) doesn't have `update-alternatives`, nor was I able to
easily find one on AUR. This seemed fun and easy enough.

## Contributors

`update-alternatives` is authored and maintained by Gregory Meyer, and is
licensed under the BSD 3-Clause license.
