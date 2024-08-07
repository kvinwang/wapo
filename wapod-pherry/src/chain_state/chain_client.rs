use std::ops::Deref;

use anyhow::{Context, Result};
use phaxt::{signer::PhalaSigner, subxt::tx::Payload, ChainApi};
use sp_core::{sr25519, Pair};
use tokio::time::timeout;
use tracing::info;

use super::NET_TIMEOUT;

pub struct ChainClient {
    client: ChainApi,
    signer: PhalaSigner,
}

impl Deref for ChainClient {
    type Target = ChainApi;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl ChainClient {
    pub fn new(client: ChainApi, signer: PhalaSigner) -> Self {
        Self { client, signer }
    }

    pub fn signer(&self) -> &PhalaSigner {
        &self.signer
    }

    pub async fn connect(url: &str, signer: &str) -> Result<Self> {
        let client = timeout(NET_TIMEOUT, phaxt::connect(url))
            .await
            .context("connect to chain timeout")?
            .context("connect to chain failed")?;
        let pair = sr25519::Pair::from_string(signer, None).context("invalid signer")?;
        let signer = PhalaSigner::new(pair);
        Ok(Self::new(client, signer))
    }

    pub async fn submit_tx<Call>(&self, tx: &Call, wait_finalized: bool) -> Result<()>
    where
        Call: Payload,
    {
        let todo = "support tx lifetime and tip";
        let signed_tx = self
            .client
            .tx()
            .create_signed(tx, self.signer(), Default::default())
            .await
            .context("sign tx failed")?;
        let progress = signed_tx
            .submit_and_watch()
            .await
            .context("submit tx failed")?;
        if wait_finalized {
            let _events = progress
                .wait_for_finalized_success()
                .await
                .context("tx failed")?;
        }
        Ok(())
    }
}
