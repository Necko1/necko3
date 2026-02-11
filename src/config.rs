use crate::chain::ChainType;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TokenConfig {
    pub symbol: String,
    pub contract: String,
    pub decimals: u8,
}

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_type: ChainType,
    pub xpub: String,
    pub native_symbol: String,
    pub decimals: u8,
    pub last_processed_block: u64,

    pub watch_addresses: Arc<RwLock<HashSet<String>>>,
    pub tokens: Arc<RwLock<HashSet<TokenConfig>>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinChainConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_type: ChainType,
    pub xpub: String,
    pub native_symbol: String,
    pub decimals: u8,
    pub last_processed_block: u64,
}

impl Into<ChainConfig> for MinChainConfig {
    fn into(self) -> ChainConfig {
        ChainConfig {
            name: self.name,
            rpc_url: self.rpc_url,
            chain_type: self.chain_type,
            xpub: self.xpub,
            native_symbol: self.native_symbol,
            decimals: self.decimals,
            last_processed_block: self.last_processed_block,
            
            watch_addresses: Arc::new(RwLock::new(HashSet::new())),
            tokens: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

impl Into<MinChainConfig> for ChainConfig {
    fn into(self) -> MinChainConfig {
        MinChainConfig {
            name: self.name,
            rpc_url: self.rpc_url,
            chain_type: self.chain_type,
            xpub: self.xpub,
            native_symbol: self.native_symbol,
            decimals: self.decimals,
            last_processed_block: self.last_processed_block,
        }
    }
}

impl Into<ChainConfig> for &MinChainConfig {
    fn into(self) -> ChainConfig {
        ChainConfig {
            name: self.name.clone(),
            rpc_url: self.rpc_url.clone(),
            chain_type: self.chain_type,
            xpub: self.xpub.clone(),
            native_symbol: self.native_symbol.clone(),
            decimals: self.decimals,
            last_processed_block: self.last_processed_block,

            watch_addresses: Arc::new(RwLock::new(HashSet::new())),
            tokens: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

impl Into<MinChainConfig> for &ChainConfig {
    fn into(self) -> MinChainConfig {
        MinChainConfig {
            name: self.name.clone(),
            rpc_url: self.rpc_url.clone(),
            chain_type: self.chain_type,
            xpub: self.xpub.clone(),
            native_symbol: self.native_symbol.clone(),
            decimals: self.decimals,
            last_processed_block: self.last_processed_block,
        }
    }
}