use risc0_zkvm::guest::env;

use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

use bitcoin::hashes::Hash;

const OGZKP_MESSAGE_PREFIX: &str = "og-zkp ";

fn recover_pubkey_from_bitcoin_signed_message(
    signature_bytes: &[u8],
    message: &str,
) -> Result<VerifyingKey, Box<dyn std::error::Error>> {
    // Extract recovery id
    let header = signature_bytes[0];
    let recovery_bits = header - 27;
    let recovery_id = RecoveryId::try_from(recovery_bits & 0x03)?;

    // Extract r||s
    let rs: [u8; 64] = signature_bytes[1..65].try_into()?;
    let signature = Signature::from_slice(&rs)?;

    // Compute Bitcoin signed message hash
    let digest = bitcoin::sign_message::signed_msg_hash(message);
    let digest_bytes = *digest.as_byte_array();

    // Recover verifying key from prehashed digest
    let pubkey = VerifyingKey::recover_from_prehash(&digest_bytes, &signature, recovery_id)?;

    Ok(pubkey)
}

fn main() {
    // Read the input
    let (message, signature_bytes): (String, Vec<u8>) = env::read();

    // Assert message starts with "og-zkp"
    assert!(
        message.starts_with(OGZKP_MESSAGE_PREFIX),
        "Message does not start with '{}'",
        OGZKP_MESSAGE_PREFIX
    );

    // Recover pubkey from the signed message
    let _pubkey = recover_pubkey_from_bitcoin_signed_message(&signature_bytes, &message)
        .expect("Failed to recover pubkey from bitcoin signed message");

    // TODO: Assert pubkey is a P2PKH output at expected outpoint

    // TODO: assert tx inclusion proof is valid for block header

    // TODO: assert block inclusion proof is valid for header merkle root

    // TODO: commit time from block header (rounded down to some interval)

    // Strip prefix from message
    let output = message.strip_prefix(OGZKP_MESSAGE_PREFIX).unwrap();

    // Commit output to the journal
    env::commit(&output);
}
