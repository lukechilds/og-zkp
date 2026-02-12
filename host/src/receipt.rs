use anyhow::{Context, Result};
use bitcoin::bech32;
use bitcoin::bech32::{Bech32m, Hrp};
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use risc0_zkvm::Receipt;
use std::io::{Read, Write};

pub fn serialize_receipt(receipt: &Receipt) -> Result<String> {
    let receipt_bytes = bincode::serialize(receipt).context("Failed to serialize receipt")?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&receipt_bytes)?;
    let compressed = encoder.finish()?;
    let hrp = Hrp::parse("ogzkp").unwrap(); // static string, can't fail
    Ok(bech32::encode::<Bech32m>(hrp, &compressed)?)
}

pub fn deserialize_receipt(receipt: &str) -> Result<Receipt> {
    let (_hrp, compressed) = bech32::decode(receipt).context("Invalid bech32m receipt")?;
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .context("Failed to decompress receipt")?;
    bincode::deserialize(&decompressed).context("Failed to deserialize receipt")
}
