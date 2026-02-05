mod listener;

use std::str::FromStr;
use std::sync::Arc;
use alloy::consensus::Transaction;
use alloy::network::TransactionResponse;
use alloy::primitives::Address;
use coins_bip32::prelude::*;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let xpub_str = "xpub6EeaXhbbgvtV6KF1fvBeEn7DZnd1Gd4xh36eMAAeBB4KA73ZV5pXmjyddjPziE5QqkcoHtRRpkce9UP5qxsd2Q9qi3zmeXtEz5sc7NFGcvN";

    let xpub = XPub::from_str(xpub_str)
        .expect("Invalid Xpub string");

    let addresses = Arc::new(RwLock::new(vec![]));

    {
        let mut addresses = addresses.write().await;

        for i in 0..30 {
            let child_xpub = xpub.derive_child(i)?;
            let verifying_key = child_xpub.as_ref();

            let address = Address::from_public_key(&verifying_key);

            println!("address /{i}: {:#?}", address);
            addresses.push(address);
        }
    }

    listener::listen_on("https://polygon-bor-rpc.publicnode.com", addresses, |tx| {
        println!(r#"
        АЙЙЙ ЛЕВ АЙ ТИГР 🦁🐅🦁🐅🦁🐅🦁🐅
        НУ МОЛОДЕЦ! ГОРЖУСЬ, {to}!!!
        сколько тебе там перевели?? {value}!??!?!
        А кто это тут расщедрился? Ааааа, это {from}...

        нууу эээ ты сохрани этот, как его... стринги: {hash}
        потом придём вернём...
        "#,
                 to=tx.to().unwrap_or_default(),
                 value=tx.value(),
                 from=tx.from(),
                 hash=tx.tx_hash())
    }).await?;

    Ok(())
}
