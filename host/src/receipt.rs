use bincode;
use bitcoin::bech32;
use bitcoin::bech32::{Bech32m, Hrp};
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use risc0_zkvm::Receipt;
use std::io::{Read, Write};

pub fn serialize_receipt(receipt: &Receipt) -> String {
    let receipt_bytes = bincode::serialize(receipt).unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&receipt_bytes).unwrap();
    let compressed = encoder.finish().unwrap();
    let hrp = Hrp::parse("ogzkp").unwrap();
    bech32::encode::<Bech32m>(hrp, &compressed).unwrap()
}

pub fn deserialize_receipt(receipt: &str) -> Receipt {
    let (_hrp, compressed) = bech32::decode(receipt).unwrap();
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    bincode::deserialize(&decompressed).unwrap()
}
