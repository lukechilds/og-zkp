use methods::OGZKP_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv};

use bitcoin::consensus;
use bitcoin::hashes::Hash;
use bitcoin::MerkleBlock;
use bitcoin::{Address, Network};

use base64::prelude::*;

use ogzkp_core::{
    generate_header_merkle_proof, headers_merkle_root, recover_pubkey_from_bitcoin_signed_message,
};

use crate::mempool_api::{fetch_raw_tx, fetch_spv_proof, find_first_seen_txid};
use crate::receipt::serialize_receipt;

pub async fn run(message: &str, signature: &str, mempool_api: &str) {
    println!("og-zkp");

    // Decode the base64 signature
    let signature_bytes = BASE64_STANDARD.decode(signature).unwrap();

    // Recover pubkey from the signed message
    let pubkey = recover_pubkey_from_bitcoin_signed_message(&signature_bytes, &message)
        .expect("Failed to recover pubkey from bitcoin signed message");

    // Derive P2PKH address from pubkey
    let address = Address::p2pkh(&pubkey, Network::Bitcoin);
    println!("Extracted P2PKH address: {}", address);

    // Lookup first-seen txid for this address
    println!("Looking up first-seen txid for address...");
    let txid = match find_first_seen_txid(mempool_api, &address.to_string()).await {
        Ok(Some(txid)) => txid,
        Ok(None) => panic!("No transactions found for address! Exiting..."),
        Err(e) => panic!("Failed to fetch transactions: {}", e),
    };
    println!("First seen txid: {}", txid);

    println!("Fetching raw tx...");
    let tx = fetch_raw_tx(mempool_api, &txid).await.unwrap();

    println!("Fetching transaction inclusion proof...");
    let spv_proof = fetch_spv_proof(mempool_api, &txid).await.unwrap();

    println!("Generating block inclusion proof...");
    let spv_proof_bytes = hex::decode(&spv_proof).unwrap();
    let merkle_block: MerkleBlock = consensus::encode::deserialize(&spv_proof_bytes).unwrap();
    let block_hash = merkle_block.header.block_hash().to_byte_array();
    let header_proof =
        generate_header_merkle_proof(block_hash).expect("Header not found in known header set");
    let block_inclusion_root = headers_merkle_root();

    // Bundle input
    let input = (
        message,
        signature_bytes,
        tx,
        spv_proof,
        header_proof,
        block_inclusion_root,
    );
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Prove the program and obtain the receipt.
    println!("Proving...");
    let prover = default_prover();
    let opts = risc0_zkvm::ProverOpts::groth16();
    let receipt = prover
        .prove_with_opts(env, OGZKP_ELF, &opts)
        .unwrap()
        .receipt;

    // Verify correct data commitments in the receipt journal.
    let (block_inclusion_root, block_month, identity): ([u8; 32], String, String) =
        receipt.journal.decode().unwrap();
    println!();
    println!("Proof generated successfully!");
    println!(
        "Block inclusion root: {:?}",
        hex::encode(block_inclusion_root)
    );
    println!("Block month: {:?}", block_month);
    println!("Identity: {:?}", identity);

    // Serialize the receipt and print as bech32m
    let serialized_receipt = serialize_receipt(&receipt);
    println!();
    println!("Proof:");
    println!("{}", serialized_receipt);
}
