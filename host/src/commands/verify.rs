use anyhow::{Context, Result};
use methods::OGZKP_ID;

use crate::receipt::deserialize_receipt;

pub fn run(receipt_str: &str) -> Result<()> {
    let receipt = deserialize_receipt(receipt_str)?;
    receipt.verify(OGZKP_ID).context("Receipt verification failed")?;
    let (block_inclusion_root, block_month, identity): ([u8; 32], String, String) =
        receipt.journal.decode().context("Failed to decode receipt journal")?;
    println!();
    println!(
        "Block inclusion root: {:?}",
        hex::encode(block_inclusion_root)
    );
    println!("Block month: {:?}", block_month);
    println!("Identity: {:?}", identity);
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify the guest program image ID hasn't changed unexpectedly.
    // If this test fails it means the guest binary has changed which will invalidate all existing proofs.
    // Update the expected value only after intentional guest changes.
    #[test]
    fn guest_image_id_unchanged() {
        let expected: [u32; 8] = [
            2411500237, 3681540482, 3726348527, 4044067061, 3987686267, 558276497, 3763698140,
            2629674215,
        ];
        assert_eq!(
            OGZKP_ID, expected,
            "Guest image ID has changed! This will break verification of existing proofs. \
             If this change is intentional, update the expected value in this test."
        );
    }
}
