use anyhow::Result;
use btc_wallet::{Balance, BtcWallet, Transaction};
use std::io::{self, Write};
use tracing::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("anyonecanpay example");

    let config1 =
        btc_wallet::load_config("./config1.toml").inspect_err(|e| error!("load_config1: {e}"))?;
    let config2 =
        btc_wallet::load_config("./config2.toml").inspect_err(|e| error!("load_config2: {e}"))?;

    let mut wallet1 = BtcWallet::create_or_load(config1).inspect_err(|e| error!("create1: {e}"))?;
    info!("wallet1 created: {}", wallet1.config.network);
    let mut wallet2 = BtcWallet::create_or_load(config2).inspect_err(|e| error!("create2: {e}"))?;
    info!("wallet2 created: {}", wallet2.config.network);

    let addr1 = wallet1.new_address();
    let addr2 = wallet2.new_address();
    println!("Send 1 BTC to {} (wallet1)", addr1);
    println!("Send 1 BTC to {} (wallet2)", addr2);

    let _ = update_balances(&mut wallet1, &mut wallet2);

    let addr3 = wallet1.new_address();
    let addr4 = wallet2.new_address();
    let tx1 = wallet1.create_tx_single_anypay(&addr4.to_string(), 100_000_000 - 160, 1.0)?;
    let tx2 = wallet2.create_tx_single_anypay(&addr3.to_string(), 100_000_000 - 300, 1.0)?; // fee少しup

    println!("tx1:");
    println!("{:#?}", tx1);
    println!("tx2:");
    println!("{:#?}", tx2);

    println!("\nSend tx1:");
    let txid = wallet1.send_tx(&tx1)?;
    println!("txid1={}", txid);
    std::thread::sleep(std::time::Duration::from_secs(5));

    // tx1 + tx2
    let tx = Transaction {
        version: tx1.version,
        lock_time: tx1.lock_time,
        input: vec![tx1.input[0].clone(), tx2.input[0].clone()],
        output: vec![tx1.output[0].clone(), tx2.output[0].clone()],
    };

    println!("tx:");
    println!("{:#?}", tx);

    println!("\nSend tx1+tx2:");
    let txid = wallet1.send_tx(&tx)?;
    println!("txid={}", txid);

    Ok(())
}

fn update_balances(wallet1: &mut BtcWallet, wallet2: &mut BtcWallet) -> (Balance, Balance) {
    let mut balance1 = wallet1.balance();
    let mut balance2 = wallet2.balance();
    println!("balance1: {}", balance1);
    println!("balance2: {}", balance2);

    loop {
        print!(".");
        io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        wallet1.sync().unwrap();
        wallet2.sync().unwrap();
        let new_balance1 = wallet1.balance();
        let new_balance2 = wallet2.balance();
        if new_balance1.confirmed != balance1.confirmed
            && new_balance2.confirmed != balance2.confirmed
        {
            balance1 = new_balance1;
            balance2 = new_balance2;
            println!();
            println!("balance1: {}", balance1);
            println!("balance2: {}", balance2);
            break;
        }
    }

    (balance1, balance2)
}
