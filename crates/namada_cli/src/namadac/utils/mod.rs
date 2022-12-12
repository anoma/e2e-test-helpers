use regex::Regex;

use crate::{NamadaError, Output};

pub mod raw {
    use std::io;

    use crate::namadac::namadac;

    const ENV_VAR_NETWORK_CONFIGS_SERVER: &str = "NAMADA_NETWORK_CONFIGS_SERVER";

    pub async fn join_network(
        chain_id: &str,
        network_configs_server: Option<&str>,
    ) -> io::Result<std::process::Output> {
        let mut cmd = namadac();
        let cmd = cmd.args([
            "utils",
            "join-network",
            "--chain-id",
            chain_id,
            "--dont-prefetch-wasm",
        ]);
        if let Some(network_configs_server) = network_configs_server {
            cmd.env(ENV_VAR_NETWORK_CONFIGS_SERVER, network_configs_server);
        }
        cmd.output().await
    }
}

#[derive(Debug)]
pub enum JoinNetworkErrorReason {
    ChainDirectoryAlreadyExists(std::process::Output),
    ConnectionRefused(std::process::Output),
}

const CONNECTION_REFUSED_REGEX: &str = r#"Connection refused"#;

pub async fn join_network(
    chain_id: &str,
    network_configs_server: Option<&str>,
) -> Result<Output<()>, NamadaError<JoinNetworkErrorReason>> {
    let output = raw::join_network(chain_id, network_configs_server)
        .await
        .map_err(|source| NamadaError::Io { source })?;
    if output.status.success() {
        return Ok(Output {
            raw: output,
            parsed: (),
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let re = Regex::new(r"already exists").unwrap();
    if re.is_match(&stderr) {
        return Err(NamadaError::Recognized {
            reason: JoinNetworkErrorReason::ChainDirectoryAlreadyExists(output),
        });
    }
    let re = Regex::new(CONNECTION_REFUSED_REGEX).unwrap();
    if re.is_match(&stderr) {
        return Err(NamadaError::Recognized {
            reason: JoinNetworkErrorReason::ConnectionRefused(output),
        });
    }
    Err(NamadaError::Unrecognized { output })
}
