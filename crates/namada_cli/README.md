# namada_cli

This is a low level wrapper around `namada` CLI tools. It can be used to execute `namada` CLI commands and parse the output into Rust data types, that can be more easily asserted on in tests. It will only work with `namada` binaries built for the version of `namada` that this crate depends on (check the [Cargo.toml](./Cargo.toml) file).

This crate won't cover every `namada` command, it only needs to contain functionality for commands that are actually used in end-to-end tests. Parsing is hacky and done via regex as there is not (yet) machine readable output for most `namada` CLI commands. Ideally this crate could eventually be replaced by a proper SDK that exposed all the functionality of `namadac`/`namadaw` binaries as a Rust library, so that `namada` binaries would not need to be used in end-to-end tests at all.
