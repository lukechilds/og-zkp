use risc0_zkvm::guest::env;

use rs_merkle::{algorithms::Sha256, MerkleProof};

use bitcoin::hashes::Hash as _;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::sign_message::{signed_msg_hash, MessageSignature, MessageSignatureError};
use bitcoin::{consensus, script::ScriptBuf, MerkleBlock, PublicKey, Transaction, Txid};

use time::{Date, OffsetDateTime};

pub const OGZKP_MESSAGE_PREFIX: &str = "og-zkp ";

// Verify the single-blob proof against an expected root.
// Inlined from host/src/block_inclusion_proof.rs to avoid pulling in shared crates that
// could accidentily break the guest program image id.
fn verify_header_merkle_proof(
    leaf_hash_be: [u8; 32],
    blob: &[u8],
    expected_root: [u8; 32],
) -> bool {
    let index = u32::from_le_bytes(blob[0..4].try_into().unwrap()) as usize;
    let total_leaves_count = u32::from_le_bytes(blob[4..8].try_into().unwrap()) as usize;
    let proof_bytes = &blob[8..];
    let proof = MerkleProof::<Sha256>::from_bytes(proof_bytes).unwrap();
    proof.verify(expected_root, &[index], &[leaf_hash_be], total_leaves_count)
}

fn pubkey_to_output_script(pubkey: PublicKey, address_type: i32) -> ScriptBuf {
    match address_type {
        // P2PKH
        0 => ScriptBuf::new_p2pkh(&pubkey.pubkey_hash()),
        // P2SH-P2WPKH
        1 => {
            let wpkh = pubkey.wpubkey_hash().unwrap();
            let redeem = ScriptBuf::new_p2wpkh(&wpkh);
            ScriptBuf::new_p2sh(&redeem.script_hash())
        }
        // P2WPKH
        2 => {
            let wpkh = pubkey.wpubkey_hash().unwrap();
            ScriptBuf::new_p2wpkh(&wpkh)
        }
        // P2TR
        3 => {
            let secp = Secp256k1::verification_only();
            let xonly = bitcoin::XOnlyPublicKey::from(pubkey);
            ScriptBuf::new_p2tr(&secp, xonly, None)
        }
        _ => panic!("Invalid address type"),
    }
}

pub fn recover_pubkey_from_bitcoin_signed_message(
    signature_bytes: &[u8],
    message: &str,
) -> Result<PublicKey, MessageSignatureError> {
    let signature = MessageSignature::from_slice(signature_bytes)?;
    let hash = signed_msg_hash(message);
    let secp = Secp256k1::verification_only();
    let pubkey = signature.recover_pubkey(&secp, hash)?;

    Ok(pubkey)
}

pub fn start_of_month(unix_ts: u32) -> u32 {
    let ts = OffsetDateTime::from_unix_timestamp(unix_ts.into()).unwrap();

    let date = Date::from_calendar_date(ts.year(), ts.month(), 1).unwrap();

    date.with_hms(0, 0, 0)
        .unwrap()
        .assume_utc()
        .unix_timestamp() as u32
}

fn main() {
    // Read the input
    let (
        message,
        signature_bytes,
        address_type,
        tx_hex,
        spv_proof_hex,
        header_proof,
        block_inclusion_root,
    ): (String, Vec<u8>, i32, String, String, Vec<u8>, [u8; 32]) = env::read();

    // Assert message starts with "og-zkp"
    assert!(
        message.starts_with(OGZKP_MESSAGE_PREFIX),
        "Message does not start with '{}'",
        OGZKP_MESSAGE_PREFIX
    );

    // Recover pubkey from the signed message
    let pubkey = recover_pubkey_from_bitcoin_signed_message(&signature_bytes, &message)
        .expect("Failed to recover pubkey from bitcoin signed message");

    // Decode transaction hex
    let tx_bytes = hex::decode(&tx_hex).expect("Invalid transaction hex");
    let tx: Transaction =
        consensus::encode::deserialize(&tx_bytes).expect("Failed to parse transaction");

    // Assert pubkey is included in an output in the transaction
    let expected_output = pubkey_to_output_script(pubkey, address_type);
    let tx_has_expected_output = tx
        .output
        .iter()
        .any(|output| output.script_pubkey == expected_output);
    assert!(
        tx_has_expected_output,
        "Recovered pubkey is not a P2PKH output in the transaction"
    );

    // Assert tx inclusion proof is valid for block header
    let txid = tx.compute_txid();
    let spv_proof_bytes = hex::decode(&spv_proof_hex).expect("Invalid SPV proof hex");
    let spv_proof: MerkleBlock =
        consensus::encode::deserialize(&spv_proof_bytes).expect("Failed to parse SPV proof");
    let mut matches: Vec<Txid> = vec![];
    let mut _indexes: Vec<u32> = vec![];
    spv_proof
        .extract_matches(&mut matches, &mut _indexes)
        .expect("Failed to extract matches from SPV proof");
    assert!(matches.contains(&txid), "Transaction is not in SPV proof");

    // Verify header inclusion against embedded merkle root over known headers
    let block_hash = spv_proof.header.block_hash().to_byte_array();
    assert!(
        verify_header_merkle_proof(block_hash, &header_proof, block_inclusion_root),
        "Header not included in known header set"
    );

    // Calculate the time of the start of the calender month the tx was confirmed in
    let block_time = spv_proof.header.time;
    let block_month = start_of_month(block_time).to_string();

    // Grab identity from the message
    let identity = message.strip_prefix(OGZKP_MESSAGE_PREFIX).unwrap();

    // Commit to the month and the message
    let output = (block_inclusion_root, block_month, identity);
    env::commit(&output);
}
