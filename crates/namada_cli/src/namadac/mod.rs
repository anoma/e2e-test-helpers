pub mod utils;
use eyre::{eyre, Result};
use namada::types::token::Amount;
use regex::Regex;

use tokio::process::Command;

use crate::{NamadaError, Output};

fn namadac() -> Command {
    Command::new("namadac")
}

pub mod raw {
    use crate::namadac::namadac;

    use std::io;

    pub async fn balance(
        ledger_address: Option<&str>,
        owner: Option<&str>,
        token: Option<&str>,
    ) -> io::Result<std::process::Output> {
        let mut cmd = namadac();
        let mut args = vec!["balance"];
        if let Some(ledger_address) = ledger_address {
            args.push("--ledger-address");
            args.push(ledger_address);
        }
        if let Some(owner) = owner {
            args.push("--owner");
            args.push(owner);
        }
        if let Some(token) = token {
            args.push("--token");
            args.push(token);
        }
        let cmd = cmd.args(args);
        cmd.output().await
    }
}

#[derive(Debug)]
pub enum BalanceErrorReason {
    UnknownTokenAddress(std::process::Output),
    NoBalanceFound(std::process::Output),
}

const NO_BALANCE_FOUND_REGEX: &str = r"No a.* balance found for a.*";
const SINGLE_TOKEN_BALANCE_REGEX: &str = r"\w+: (\d+)";

pub async fn balance(
    ledger_address: Option<&str>,
    owner: Option<&str>,
    token: Option<&str>,
) -> Result<Output<()>, NamadaError<BalanceErrorReason>> {
    let output = raw::balance(ledger_address, owner, token)
        .await
        .map_err(|source| NamadaError::Io { source })?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(NO_BALANCE_FOUND_REGEX).unwrap();
        if re.is_match(&stdout) {
            return Err(NamadaError::Recognized {
                reason: BalanceErrorReason::NoBalanceFound(output),
            });
        }

        return Ok(Output {
            raw: output,
            parsed: (),
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let re = Regex::new(r"Unknown address").unwrap();
    if re.is_match(&stderr) {
        return Err(NamadaError::Recognized {
            reason: BalanceErrorReason::UnknownTokenAddress(output),
        });
    }
    Err(NamadaError::Unrecognized { output })
}

pub async fn balance_of_token_for_owner(
    ledger_address: Option<&str>,
    owner: &str,
    token: &str,
) -> Result<Output<Amount>, NamadaError<BalanceErrorReason>> {
    let output = balance(ledger_address, Some(owner), Some(token)).await?;
    let stdout = String::from_utf8_lossy(&output.raw.stdout);
    if let Ok(parsed) = parse_balance_of_token_for_owner(&stdout) {
        Ok(Output {
            raw: output.raw,
            parsed,
        })
    } else {
        Err(NamadaError::Unrecognized { output: output.raw })
    }
}

fn parse_balance_of_token_for_owner(stdout: &str) -> Result<Amount> {
    let re = Regex::new(SINGLE_TOKEN_BALANCE_REGEX).unwrap();
    if let Some(captured) = re.captures(stdout) {
        let n = captured.get(1).unwrap();
        let n: i128 = n.as_str().parse()?;
        Ok(Amount::from_change(n))
    } else {
        Err(eyre!("Could not parse a balance"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_balance_found_regex() {
        const OUTPUT: &str = "\u{1b}[2m2022-10-04T22:31:59.707006Z\u{1b}[0m \u{1b}[32m INFO\u{1b}[0m \u{1b}[2mnamada_apps::cli::context\u{1b}[0m\u{1b}[2m:\u{1b}[0m Chain ID: dev.70637df1cdce6f442a8f501274\nNo atest1v9hx7w36g42ysgzzwf5kgem9ypqkgerjv4ehxgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpq8f99ew balance found for atest1v4ehgw36x5myzdfcg5myg3feg4p5vvzyxfrrgdpnxccnvvp5gdrrqse3gdz5zv2yxsenqs3ntgy89n\n";
        let re = Regex::new(NO_BALANCE_FOUND_REGEX).unwrap();
        assert!(re.is_match(OUTPUT))
    }

    #[test]
    fn test_parse_balance_of_token_for_owner() {
        const OUTPUT: &str = "XAN: 923892839\n";
        assert_eq!(
            parse_balance_of_token_for_owner(OUTPUT).unwrap(),
            Amount::from(923892839)
        );
    }
}
