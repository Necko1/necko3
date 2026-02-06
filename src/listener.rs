use crate::config::ChainConfig;
use crate::model::PaymentEvent;
use alloy::consensus::Transaction;
use alloy::network::TransactionResponse;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::Block;
use alloy::rpc::types::Transaction as RpcTransaction;
use bigdecimal::{BigDecimal, FromPrimitive};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use url::Url;

fn format_units(amount: U256, decimals: u8) -> BigDecimal {
    let amount_str = amount.to_string();
    let amount_big: BigDecimal = amount_str.parse().unwrap_or_default();

    let scale = BigDecimal::from_u64(10u64.pow(decimals as u32)).unwrap();
    amount_big / scale
}

pub async fn listen_on(
    config: Arc<ChainConfig>,
    sender: mpsc::Sender<PaymentEvent>
) -> anyhow::Result<()> {
    let rpc_url = Url::parse(&config.rpc_url)?;
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let mut last_block_num = match provider.get_block_number().await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("failed to get latest block number: {}. retrying in 5s...", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
            provider.get_block_number().await?
        }
    };

    loop {
        let current_block_num = match provider.get_block_number().await {
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to get latest block number: {}. sleep 2s...", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue
            }
        };

        if current_block_num <= last_block_num {
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        }

        for block_num in (last_block_num + 1)..=current_block_num {
            println!("processing block {}...", block_num);

            if let Some(block) = provider
                .get_block_by_number(block_num.into())
                .full()
                .await
                .ok()
                .flatten()
            {
                let addresses = {
                    let guard = config.watch_addresses.read().await;
                    guard.clone()
                };

                let transactions = handle_new_block(&addresses, block).unwrap_or_default();
                for tx in transactions {
                    let amount_human = format_units(tx.value(), config.decimals);

                    let event = PaymentEvent {
                        network: config.name.clone(),
                        tx_hash: tx.tx_hash(),
                        from: tx.from(),
                        to: tx.to().unwrap_or_default(), // default is unreachable, but it's better to keep this instead of ::unwrap()
                        token: config.native_symbol.clone(), // todo: usdc/usdt/other tokens
                        amount: amount_human,
                        amount_raw: tx.value(),
                    };

                    let _ = sender.send(event).await;
                }
            }
        }

        last_block_num = current_block_num;
    }
}

fn handle_new_block(
    addresses: &[Address],
    block: Block,
) -> anyhow::Result<Vec<RpcTransaction>> {
    let txs = block.into_transactions_vec();

    let transactions = txs
        .into_iter()
        .filter(|tx| {
            tx.to().map_or(false, |to|
                addresses.contains(&to))
        })
        .collect();

    Ok(transactions)
}