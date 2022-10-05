use borsh::BorshSerialize;
use eyre::{Context, Result};
use hyper::{Body, Method, Request, Response};
use namada::types::{
    address::Address,
    ethereum_events::{EthAddress, EthereumEvent, TransferToNamada, Uint},
    token::Amount,
};
use rand::Rng;

pub const DEFAULT_ETHEREUM_EVENT_ENDPOINT: &str = "http://127.0.0.1:3030/eth_events";

pub const DAI_ERC20_ETH_ADDRESS_CHECKSUMMED: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";

pub struct Client {
    endpoint: String,
}

impl Client {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }

    async fn send(&self, event: &EthereumEvent) -> Result<Response<Body>> {
        let event = event.try_to_vec()?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(&self.endpoint)
            .header("content-type", "application/octet-stream")
            .body(Body::from(event))?;

        let client = hyper::Client::new();
        client.request(req).await.wrap_err_with(|| "sending event")
    }

    pub async fn send_fake_transfer_to_namada(
        &self,
        amount: Amount,
        asset: EthAddress,
        receiver: Address,
        nonce: Option<Uint>,
    ) -> Result<Response<Body>> {
        let transfer = TransferToNamada {
            amount,
            asset,
            receiver,
        };
        let transfers = vec![transfer];
        let nonce = match nonce {
            Some(nonce) => nonce,
            None => {
                let mut rng = rand::thread_rng();
                let rn: u64 = rng.gen();
                tracing::debug!("No nonce provided, generated random nonce {}", rn);
                rn.into()
            }
        };
        let event = EthereumEvent::TransfersToNamada { nonce, transfers };
        tracing::debug!("Posting event - {:#?}", event);
        let resp = self.send(&event).await?;
        tracing::debug!("Response: {:#?}", resp);
        Ok(resp)
    }
}
