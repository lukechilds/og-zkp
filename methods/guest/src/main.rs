use bitcoin::hashes::Hash as _;
use bitcoin::{consensus, script::ScriptBuf, MerkleBlock, Transaction, Txid};
use ogzkp_core::{
    recover_pubkey_from_bitcoin_signed_message, start_of_month, verify_header_merkle_proof,
    OGZKP_MESSAGE_PREFIX,
};
use risc0_zkvm::guest::env;

fn main() {
    // Read the input
    let (message, signature_bytes, tx_hex, spv_proof_hex, header_proof, block_inclusion_root): (
        String,
        Vec<u8>,
        String,
        String,
        Vec<u8>,
        [u8; 32],
    ) = env::read();

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

    // Assert pubkey is a P2PKH output in the transaction
    // TODO: Support other address types
    let expected_output: ScriptBuf = ScriptBuf::new_p2pkh(&pubkey.pubkey_hash());
    let tx_has_expected_output = tx.output.iter().any(|o| o.script_pubkey == expected_output);
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
