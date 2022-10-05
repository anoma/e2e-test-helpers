use std::io;

use thiserror::Error;

pub mod namadac;
pub mod namadaw;

#[derive(Error, Debug)]
pub enum NamadaError<Reason> {
    #[error("The command failed: {reason}")]
    Recognized { reason: Reason },
    #[error("The command failed for some unrecognized reason")]
    Unrecognized { output: std::process::Output },
    #[error("The command failed due to an I/O error")]
    Io { source: io::Error },
}

/// Represents the output of some `namada` command, both raw and parsed into some Rust struct
pub struct Output<T> {
    pub raw: std::process::Output,
    pub parsed: T,
}
