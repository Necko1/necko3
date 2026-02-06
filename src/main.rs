mod listener;
mod config;
mod model;

use std::str::FromStr;
use std::sync::Arc;
use alloy::consensus::Transaction;
use alloy::network::TransactionResponse;
use alloy::primitives::Address;
use coins_bip32::prelude::*;
use tokio::sync::{mpsc, RwLock};
use crate::config::{ChainConfig, ChainType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel(100);

    let xpub_str = "xpub6EeaXhbbgvtV6KF1fvBeEn7DZnd1Gd4xh36eMAAeBB4KA73ZV5pXmjyddjPziE5QqkcoHtRRpkce9UP5qxsd2Q9qi3zmeXtEz5sc7NFGcvN";

    let xpub = XPub::from_str(xpub_str)
        .expect("Invalid Xpub string");

    let polygon_conf = Arc::new(ChainConfig {
        name: "Polygon Mainnet".to_owned(),
        rpc_url: "https://polygon-bor-rpc.publicnode.com".to_owned(),
        chain_type: ChainType::EVM,
        native_symbol: "POL".to_owned(),
        decimals: 18,
        watch_addresses: RwLock::new(vec![]),
    });

    {
        let mut addresses = polygon_conf.watch_addresses.write().await;

        for i in 0..30 {
            let child_xpub = xpub.derive_child(i)?;
            let verifying_key = child_xpub.as_ref();

            let address = Address::from_public_key(&verifying_key);

            println!("address /{i}: {:#?}", address);
            addresses.push(address);
        }
    }

    let tx_polygon = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = listener::listen_on(polygon_conf, tx_polygon).await {
            eprintln!("Polygon listener died: {}", e);
        }
    });

    while let Some(event) = rx.recv().await {
        println!(r#"
        АЙЙЙ ЛЕВ АЙ ТИГР 🦁🐅🦁🐅🦁🐅🦁🐅
        НУ МОЛОДЕЦ! ГОРЖУСЬ, {to}!!!
        сколько тебе там перевели?? {value} {token}!??!?!
        А КТО ЭТО ТУТ РАСЩЕДРИЛСЯ? Ааааа, это {from}...
        который ещё из {network}...

        нууу эээ ты сохрани этот, как его... стринги: {hash}
        потом придём вернём...
        "#,
            to=event.to,
            value=event.amount,
            token=event.token,
            from=event.from,
            network=event.network,
            hash=event.tx_hash)
    }

    Ok(())
}
