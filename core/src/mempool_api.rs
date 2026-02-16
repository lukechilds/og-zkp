use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct TxShort {
    txid: String,
}

async fn fetch_txids(
    endpoint: &str,
    address: &str,
    after_txid: Option<&str>,
) -> Result<Vec<String>> {
    // Build the URL
    let mut url = format!("{endpoint}/address/{address}/txs");
    if let Some(cursor) = after_txid {
        url.push_str(&format!("?after_txid={cursor}"));
    }

    // Send the request
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("accept", "application/json")
        .send()
        .await?
        .error_for_status()?;

    // Parse the response
    let txs: Vec<TxShort> = resp.json().await?;

    // Return the txids
    Ok(txs.into_iter().map(|t| t.txid).collect())
}

pub async fn find_first_seen_txid(
    endpoint: &str,
    address: &str,
) -> Result<Option<String>> {
    // Walk pages using the second-to-last txid as the cursor until only one remains
    let mut after_txid = None;
    loop {
        let txids = fetch_txids(endpoint, address, after_txid.as_deref()).await?;

        // If no txids, return None
        if txids.is_empty() {
            return Ok(None);
        }

        // If there's only one txid, we have the first seen tx
        if txids.len() == 1 {
            return Ok(Some(txids[0].clone()));
        }

        // Set the cursor to the second-to-last txid
        let next_txid = Some(txids[txids.len() - 2].clone());
        if next_txid == after_txid {
            bail!("This instance of the mempool API does not support pagination");
        }
        after_txid = next_txid;
    }
}

pub async fn fetch_spv_proof(endpoint: &str, txid: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{endpoint}/tx/{txid}/merkleblock-proof");
    let resp = client.get(url).send().await?.error_for_status()?;
    let proof = resp.text().await?.trim().to_string();
    Ok(proof)
}

pub async fn fetch_raw_tx(endpoint: &str, txid: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{endpoint}/tx/{txid}/hex");
    let resp = client.get(url).send().await?.error_for_status()?;
    let tx = resp.text().await?.trim().to_string();
    Ok(tx)
}
