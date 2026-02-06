use alloy::primitives::{Address, TxHash, U256};
use bigdecimal::BigDecimal;

#[derive(Debug, Clone)]
pub struct PaymentEvent {
    pub network: String,
    pub tx_hash: TxHash,
    pub from: Address,
    pub to: Address,
    pub token: String,
    pub amount: BigDecimal,
    pub amount_raw: U256,
}