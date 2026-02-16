use anyhow::{Context, Result};

use crate::receipt::deserialize_receipt;

pub struct VerifyResult {
    pub block_inclusion_root: String,
    pub block_month: String,
    pub identity: String,
}

pub fn verify_and_extract(
    receipt_str: &str,
    image_id: impl Into<risc0_zkvm::sha::Digest>,
) -> Result<VerifyResult> {
    let receipt = deserialize_receipt(receipt_str)?;
    receipt
        .verify(image_id)
        .context("Receipt verification failed")?;
    let (block_inclusion_root, block_month, identity): ([u8; 32], String, String) = receipt
        .journal
        .decode()
        .context("Failed to decode receipt journal")?;
    Ok(VerifyResult {
        block_inclusion_root: hex::encode(block_inclusion_root),
        block_month,
        identity,
    })
}

pub fn run(
    image_id: impl Into<risc0_zkvm::sha::Digest>,
    receipt_str: &str,
    json: bool,
) -> Result<()> {
    let receipt = deserialize_receipt(receipt_str)?;
    receipt
        .verify(image_id)
        .context("Receipt verification failed")?;
    let (block_inclusion_root, block_month, identity): ([u8; 32], String, String) = receipt
        .journal
        .decode()
        .context("Failed to decode receipt journal")?;

    if json {
        let output = serde_json::json!({
            "block_inclusion_root": hex::encode(block_inclusion_root),
            "block_month": block_month,
            "identity": identity,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        let month_display = block_month
            .parse::<i64>()
            .ok()
            .and_then(|ts| time::OffsetDateTime::from_unix_timestamp(ts).ok())
            .map(|dt| format!("{} {}", dt.month(), dt.year()))
            .unwrap_or(block_month.clone());
        println!("OG Status: {month_display}");
        println!("Identity:  {identity}");
        println!();
        println!("\x1b[32m✓\x1b[0m Proof is valid");
    }
    Ok(())
}
