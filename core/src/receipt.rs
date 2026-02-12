use anyhow::{Context, Result};
use bitcoin::bech32;
use bitcoin::bech32::{Bech32m, Hrp};
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use risc0_zkvm::Receipt;
use std::io::{Read, Write};

// Raw bytes methods for easier testing
fn serialize_bytes(data: &[u8]) -> Result<String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    let hrp = Hrp::parse("og-zkp").unwrap(); // static string, can't fail
    Ok(bech32::encode::<Bech32m>(hrp, &compressed)?)
}

fn deserialize_bytes(encoded: &str) -> Result<Vec<u8>> {
    let (_hrp, compressed) = bech32::decode(encoded).context("Invalid bech32m receipt")?;
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .context("Failed to decompress receipt")?;
    Ok(decompressed)
}

// Wrapped typed methods exported to be used by the codebase
pub fn serialize_receipt(receipt: &Receipt) -> Result<String> {
    let receipt_bytes = bincode::serialize(receipt).context("Failed to serialize receipt")?;
    serialize_bytes(&receipt_bytes)
}

pub fn deserialize_receipt(receipt: &str) -> Result<Receipt> {
    let bytes = deserialize_bytes(receipt)?;
    bincode::deserialize(&bytes).context("Failed to deserialize receipt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let data = b"hello this is a test payload for round-trip encoding";
        let encoded = serialize_bytes(data).unwrap();
        let decoded = deserialize_bytes(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn encoded_starts_with_og_zkp() {
        let encoded = serialize_bytes(b"test").unwrap();
        assert!(encoded.starts_with("og-zkp1"));
    }

    #[test]
    fn decode_invalid_bech32m_fails() {
        let result = deserialize_bytes("not-a-valid-receipt");
        assert!(result.is_err());
    }
}
