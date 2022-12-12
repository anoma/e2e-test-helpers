use eyre::{eyre, Result};
use namada_core::types::address::Address;
use regex::Regex;

use crate::{NamadaError, Output};

pub mod raw {
    use std::io;

    use crate::namadaw::namadaw;

    pub async fn gen(unsafe_dont_encrypt: bool) -> io::Result<std::process::Output> {
        let mut cmd = namadaw();
        let mut args = vec!["address", "gen"];
        if unsafe_dont_encrypt {
            args.push("--unsafe-dont-encrypt");
        }
        let cmd = cmd.args(args);
        cmd.output().await
    }

    pub async fn find(alias: &str) -> io::Result<std::process::Output> {
        let mut cmd = namadaw();
        let cmd = cmd.args(["address", "find", "--alias", alias]);
        cmd.output().await
    }
}

type Alias = String;

const GEN_ALIAS_REGEX: &str = r#"Successfully added a key and an address with alias: "(\w+)""#;

pub async fn gen(unsafe_dont_encrypt: bool) -> Result<Output<Alias>, NamadaError<()>> {
    let output = raw::gen(unsafe_dont_encrypt)
        .await
        .map_err(|source| NamadaError::Io { source })?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(parsed) = parse_generated_alias(&stdout) {
            return Ok(Output {
                raw: output,
                parsed,
            });
        } else {
            return Err(NamadaError::Unrecognized { output });
        }
    }
    Err(NamadaError::Unrecognized { output })
}

fn parse_generated_alias(stdout: &str) -> Option<String> {
    let re = Regex::new(GEN_ALIAS_REGEX).unwrap();
    if let Some(captured) = re.captures(stdout) {
        let n = captured.get(1).unwrap();
        Some(n.as_str().to_owned())
    } else {
        None
    }
}

const FIND_ADDRESS_REGEX: &str = r#"Found address \w+: (\w{84})"#;

pub async fn find(alias: &str) -> Result<Output<Address>, NamadaError<()>> {
    let output = raw::find(alias)
        .await
        .map_err(|source| NamadaError::Io { source })?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(parsed) = parse_found_address(&stdout) {
            return Ok(Output {
                raw: output,
                parsed,
            });
        }
    }
    Err(NamadaError::Unrecognized { output })
}

fn parse_found_address(stdout: &str) -> Result<Address> {
    let re = Regex::new(FIND_ADDRESS_REGEX).unwrap();
    if let Some(captured) = re.captures(stdout) {
        let n = captured.get(1).unwrap();
        Ok(Address::decode(n.as_str())?)
    } else {
        Err(eyre!("Could not parse an address"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_generated_alias() {
        const OUTPUT: &str = r#"
        Warning: The keypair will NOT be encrypted.
        Successfully added a key and an address with alias: "ee16622f97756135aaea17a4df5dffd6064f1b42""#;
        assert_eq!(
            parse_generated_alias(OUTPUT).unwrap(),
            "ee16622f97756135aaea17a4df5dffd6064f1b42"
        );
    }
    #[test]
    fn test_parse_found_address() {
        const OUTPUT: &str = r#"ee16622f97756135aaea17a4df5dffd6064f1b42










        Found address Implicit: atest1d9khqw36g4znzd3kxgeyvwfhxu6nvvfnx4q5z32pxym5zdzygc65g3jxgsmrqd35gcc5ydpjtff6td
        "#;
        assert_eq!(
            parse_found_address(OUTPUT).unwrap(),
            Address::decode("atest1d9khqw36g4znzd3kxgeyvwfhxu6nvvfnx4q5z32pxym5zdzygc65g3jxgsmrqd35gcc5ydpjtff6td").unwrap()
        );
    }
}
