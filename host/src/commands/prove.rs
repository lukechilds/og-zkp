use methods::OGZKP_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv};

use bitcoin::consensus;
use bitcoin::hashes::Hash;
use bitcoin::MerkleBlock;
use bitcoin::{Address, AddressType, Network, Transaction};
use std::str::FromStr;

use base64::prelude::*;

use ogzkp_core::{generate_header_merkle_proof, headers_merkle_root};

use crate::mempool_api::{fetch_raw_tx, fetch_spv_proof, find_first_seen_txid};
use crate::receipt::serialize_receipt;

pub async fn run(
    message: &str,
    signature: &str,
    address: &str,
    mempool_api: &str,
    mut transaction: Option<String>,
    mut spv_proof: Option<String>,
) {
    println!("og-zkp");

    // Decode the base64 signature
    let signature_bytes = BASE64_STANDARD.decode(signature).unwrap();

    // Get address type
    let passed_in_address = Address::from_str(address)
        .unwrap()
        .require_network(Network::Bitcoin)
        .unwrap();
    let address_type = match passed_in_address.address_type() {
        Some(AddressType::P2pkh) => 0,
        Some(AddressType::P2sh) => 1, // Assume nested P2WPKH
        Some(AddressType::P2wpkh) => 2,
        Some(AddressType::P2tr) => 3,
        _ => panic!("Unsupported address type"),
    };

    if transaction.is_none() {
        // Lookup first-seen txid for this address
        println!("Looking up first-seen txid for address...");
        let txid = match find_first_seen_txid(mempool_api, &address.to_string()).await {
            Ok(Some(txid)) => txid,
            Ok(None) => panic!("No transactions found for address! Exiting..."),
            Err(e) => panic!("Failed to fetch transactions: {}", e),
        };
        println!("First seen txid: {}", txid);

        println!("Fetching raw tx...");
        transaction = Some(fetch_raw_tx(mempool_api, &txid).await.unwrap());
    }

    if spv_proof.is_none() {
        println!("Fetching transaction inclusion proof...");
        let tx_bytes =
            hex::decode(&transaction.as_deref().unwrap()).expect("Invalid transaction hex");
        let txid = consensus::encode::deserialize::<Transaction>(&tx_bytes)
            .unwrap()
            .compute_txid();
        spv_proof = Some(
            fetch_spv_proof(mempool_api, &txid.to_string())
                .await
                .unwrap(),
        );
    }

    println!("Generating block inclusion proof...");
    let spv_proof_bytes = hex::decode(&spv_proof.as_deref().unwrap()).unwrap();
    let merkle_block: MerkleBlock = consensus::encode::deserialize(&spv_proof_bytes).unwrap();
    let block_hash = merkle_block.header.block_hash().to_byte_array();
    let header_proof =
        generate_header_merkle_proof(block_hash).expect("Header not found in known header set");
    let block_inclusion_root = headers_merkle_root();

    // Bundle input
    let input = (
        message,
        signature_bytes,
        address_type,
        transaction.unwrap(),
        spv_proof.unwrap(),
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
