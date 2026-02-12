use crate::chain::evm::EvmBlockchain;
use crate::chain::Blockchain::Evm;
use crate::model::PaymentEvent;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use utoipa::ToSchema;

pub mod evm;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum ChainType {
    EVM
}

impl Display for ChainType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainType::EVM => write!(f, "EVM"),
        }
    }
}

pub trait BlockchainAdapter: Sync + Send {
    fn new(state: Arc<AppState>, chain_type: ChainType, chain_name: &str, sender: Option<Sender<PaymentEvent>>) -> Self;
    async fn derive_address(&self, index: u32) -> anyhow::Result<String>;
    async fn listen(&self) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub enum Blockchain {
    Evm(EvmBlockchain),
}

impl BlockchainAdapter for Blockchain {
    fn new(state: Arc<AppState>, chain_type: ChainType, chain_name: &str, sender: Option<Sender<PaymentEvent>>) -> Self {
        match chain_type {
            ChainType::EVM => Evm(EvmBlockchain::new(state, chain_type, chain_name, sender))
        }
    }

    async fn derive_address(&self, index: u32) -> anyhow::Result<String> {
        match self {
            Evm(bc) => bc.derive_address(index).await,
        }
    }

    async fn listen(&self) -> anyhow::Result<()> {
        match self {
            Evm(bc) => bc.listen().await,
        }
    }
}