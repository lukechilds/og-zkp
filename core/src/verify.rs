use anyhow::{ensure, Context, Result};
use bitcoin::hashes::{sha256, Hash};

use crate::receipt::deserialize_receipt;

const EXPECTED_BLOCK_INCLUSION_ROOT: &str =
    "645193f7e45302f503f14d6bdc593a12ee954b5ca844d38affaae51febb77a3e";
const ATTESTATION_URL_BASE: &str = "https://og-zkp.com/proof/";

pub struct VerifyResult {
    pub block_inclusion_root: String,
    pub block_month: String,
    pub identity: String,
    pub attestation_url: String,
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
    let block_inclusion_root = verify_block_inclusion_root(block_inclusion_root)?;
    Ok(VerifyResult {
        block_inclusion_root,
        block_month,
        identity,
        attestation_url: attestation_url(receipt_str),
    })
}

pub fn run(
    image_id: impl Into<risc0_zkvm::sha::Digest>,
    receipt_str: &str,
    json: bool,
) -> Result<()> {
    let result = verify_and_extract(receipt_str, image_id)?;

    if json {
        let output = serde_json::json!({
            "block_inclusion_root": result.block_inclusion_root,
            "block_month": result.block_month,
            "identity": result.identity,
            "attestation_url": result.attestation_url,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        let month_display = result
            .block_month
            .parse::<i64>()
            .ok()
            .and_then(|ts| time::OffsetDateTime::from_unix_timestamp(ts).ok())
            .map(|dt| format!("{} {}", dt.month(), dt.year()))
            .unwrap_or(result.block_month);
        println!("OG Status:   {month_display}");
        println!("Identity:    {}", result.identity);
        println!("Attestation: {}", result.attestation_url);
        println!();
        println!("\x1b[32m✓\x1b[0m Proof is valid");
    }
    Ok(())
}

fn verify_block_inclusion_root(root: [u8; 32]) -> Result<String> {
    let root = hex::encode(root);
    ensure!(
        root == EXPECTED_BLOCK_INCLUSION_ROOT,
        "Unexpected block inclusion root: expected {EXPECTED_BLOCK_INCLUSION_ROOT}, got {root}"
    );
    Ok(root)
}

fn proof_id(receipt_str: &str) -> String {
    let mut id = hex::encode(sha256::Hash::hash(receipt_str.as_bytes()).to_byte_array());
    id.truncate(32);
    id
}

fn attestation_url(receipt_str: &str) -> String {
    format!("{ATTESTATION_URL_BASE}{}", proof_id(receipt_str))
}

#[cfg(test)]
mod tests {
    use super::attestation_url;

    #[test]
    fn attestation_url_matches_web_short_hash() {
        assert_eq!(
            attestation_url("og-zkp1test"),
            "https://og-zkp.com/proof/e0dc66a0f0217c19b662a141e86ff308"
        );
    }
}
