use alloy::consensus::Transaction;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::Block;
use alloy::rpc::types::Transaction as RpcTransaction;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use url::Url;

pub async fn listen_on<T>(
    url: &str,
    addresses: Arc<RwLock<Vec<Address>>>,
    on_capture: T,
) -> anyhow::Result<()>
where
    T: Fn(RpcTransaction) + Send + 'static + Sync,
{
    let on_capture = Arc::new(on_capture);

    let rpc_url = Url::parse(url)?;
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
                    let guard = addresses.read().await;
                    guard.clone()
                };

                let transactions = handle_new_block(&addresses, block).unwrap_or_default();
                for tx in transactions {
                    on_capture(tx);
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