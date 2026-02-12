use anyhow::{Context, Result};

use crate::receipt::deserialize_receipt;

pub fn run(image_id: impl Into<risc0_zkvm::sha::Digest>, receipt_str: &str) -> Result<()> {
    let receipt = deserialize_receipt(receipt_str)?;
    receipt.verify(image_id).context("Receipt verification failed")?;
    let (block_inclusion_root, block_month, identity): ([u8; 32], String, String) =
        receipt.journal.decode().context("Failed to decode receipt journal")?;
    println!();
    println!(
        "Block inclusion root: {:?}",
        hex::encode(block_inclusion_root)
    );
    println!("Block month: {block_month:?}");
    println!("Identity: {identity:?}");
    println!();
    Ok(())
}
