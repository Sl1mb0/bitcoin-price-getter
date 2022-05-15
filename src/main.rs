use tokio::sync::mpsc;
use std::thread;
use std::time;
use std::str;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let (price_tx, mut price_rx) = mpsc::channel(1);

    tokio::spawn(async move {
        loop {
            let bytes = reqwest::get("https://api.cryptowat.ch/markets/kraken/btcusd/price")
                .await.unwrap()
                .bytes()
                .await.unwrap();

            let doc = str::from_utf8(&bytes)
                .expect("`std::str::from_utf8()`: Failed to parse string from request");
            let data_value: serde_json::Value = serde_json::from_str(&doc)
                .expect("`serde_json::from_str()`: Failed to build `serde_json::Value` from string");
            let updated_price = data_value["result"]["price"].as_f64().expect(
                "`serde_json::Value::as_f64()`: JSON value did not contain expected price information",
            );

            if let Err(_) = price_tx.send(updated_price).await {
                panic!("Receiving thread dropped!");
            }

            thread::sleep(time::Duration::new(5, 0));
        }
    });

    while let Some(received_price) = price_rx.recv().await {
        println!("BTC/USD = {}", received_price);
    }

    Ok(())
}
