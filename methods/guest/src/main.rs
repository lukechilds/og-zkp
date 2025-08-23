use ogzkp_core::{recover_pubkey_from_bitcoin_signed_message, OGZKP_MESSAGE_PREFIX};
use risc0_zkvm::guest::env;

fn main() {
    // Read the input
    let (message, signature_bytes, _tx, _spv_proof): (String, Vec<u8>, String, String) =
        env::read();

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
