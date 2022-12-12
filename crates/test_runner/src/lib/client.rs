use borsh::BorshDeserialize;
use eyre::{eyre, Context, Result};
use namada_core::types::storage::Key;
use regex::Regex;

use crate::exec::{execute, execute_or_die};
use std::process::Command;

const NATIVE_TOKEN: &str = "NAM";

pub struct Client {
    ledger_address: String,
}

impl Client {
    pub fn new(ledger_address: &str) -> Client {
        Client {
            ledger_address: ledger_address.to_owned(),
        }
    }

    pub fn init_account(&self, source: &str, alias: &str, code_path: Option<&str>) {
        let mut cmd = Command::new("namadac");
        let mut args = vec![
            "init-account",
            "--ledger-address",
            &self.ledger_address,
            "--source",
            source,
            "--public-key",
            source,
            "--alias",
            alias,
        ];
        if code_path.is_some() {
            args.append(&mut vec!["--code-path", code_path.unwrap()]);
        };
        let cmd = cmd.args(args);
        execute_or_die(cmd);
    }

    pub fn get_native_tokens_from_faucet(&self, target: &str) {
        let mut cmd = Command::new("namadac");
        let cmd = cmd.args([
            "transfer",
            "--ledger-address",
            &self.ledger_address,
            "--token",
            NATIVE_TOKEN,
            "--amount",
            "1000",
            "--source",
            "faucet",
            "--target",
            target,
            "--signer",
            target,
        ]);
        execute_or_die(cmd);
    }

    pub fn tx(&self, code_path: &str, signer: &str, data_path: Option<&str>) {
        let mut cmd = Command::new("namadac");
        let mut args = vec![
            "tx",
            "--ledger-address",
            &self.ledger_address,
            "--code-path",
            code_path,
            "--signer",
            signer,
        ];
        if data_path.is_some() {
            args.append(&mut vec!["--data-path", data_path.unwrap()]);
        };
        let cmd = cmd.args(args);
        execute_or_die(cmd);
    }

    pub fn query_bytes<T: BorshDeserialize>(&self, storage_key: &Key) -> Result<T> {
        let mut cmd = Command::new("namadac");
        let cmd = cmd.env("ANOMA_LOG", "none").args([
            "query-bytes",
            "--ledger-address",
            &self.ledger_address,
            "--storage-key",
            &storage_key.to_string(),
        ]);
        let output = execute(cmd)?;
        let stdout = output.stdout;

        let stdout_str = String::from_utf8(stdout)?;
        let stdout_parsed = parse_query_bytes_output(&stdout_str)?;
        let borsh_serialized = hex::decode(stdout_parsed)?;
        T::try_from_slice(&borsh_serialized).wrap_err("couldn't parse stored value".to_string())
    }
}

fn parse_query_bytes_output(stdout: &str) -> Result<String> {
    let re = Regex::new(r"Found data: 0x(.*)").expect("Can always construct regex");
    let caps = match re.captures(stdout) {
        Some(caps) => caps,
        None => return Err(eyre!("No match found when querying bytes")),
    };
    Ok(caps
        .get(1)
        .expect("We can always find at least one match if a `Captures` was returned")
        .as_str()
        .to_string())
}

/// NB: requires ANOMA_NETWORK_CONFIGS_SERVER in env
pub fn join_network(chain_id: &str) -> Result<std::process::Output, std::io::Error> {
    let mut cmd = Command::new("namadac");
    let cmd = cmd.args(["utils", "join-network", "--chain-id", chain_id]);
    execute(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_bytes_output() {
        const OUTPUT: &str = "IMMA VEC!!!!!!\nFound data: 0x80f0fa0200000000\n";
        assert_eq!(
            parse_query_bytes_output(OUTPUT).unwrap(),
            "80f0fa0200000000"
        );
    }
}
