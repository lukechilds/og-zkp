use methods::OGZKP_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv};

use anyhow::{bail, Context, Result};
use bitcoin::consensus;
use bitcoin::hashes::Hash;
use bitcoin::MerkleBlock;
use bitcoin::{Address, AddressType, Network, Transaction};
use std::str::FromStr;

use base64::prelude::*;

use crate::block_inclusion_proof::{
    generate_header_merkle_proof, headers_merkle_root, verify_header_merkle_proof,
};
use crate::mempool_api::{fetch_raw_tx, fetch_spv_proof, find_first_seen_txid};
use crate::receipt::serialize_receipt;

pub async fn run(
    message: &str,
    signature: &str,
    address: &str,
    mempool_api: &str,
    mut transaction: Option<String>,
    mut spv_proof: Option<String>,
) -> Result<()> {
    println!("og-zkp");

    // Decode the base64 signature
    let signature_bytes = BASE64_STANDARD
        .decode(signature)
        .context("Invalid base64 signature")?;

    // Get address type
    let passed_in_address = Address::from_str(address)
        .context("Invalid Bitcoin address")?
        .require_network(Network::Bitcoin)
        .context("Address is not a mainnet address")?;
    let address_type = match passed_in_address.address_type() {
        Some(AddressType::P2pkh) => 0,
        Some(AddressType::P2sh) => 1, // Assume nested P2WPKH
        Some(AddressType::P2wpkh) => 2,
        Some(AddressType::P2tr) => 3,
        _ => bail!("Unsupported address type"),
    };

    if transaction.is_none() {
        // Lookup first-seen txid for this address
        println!("Looking up first-seen txid for address...");
        let txid = match find_first_seen_txid(mempool_api, &address.to_string()).await {
            Ok(Some(txid)) => txid,
            Ok(None) => bail!("No transactions found for address"),
            Err(e) => bail!("Failed to fetch transactions: {}", e),
        };
        println!("First seen txid: {}", txid);

        println!("Fetching raw tx...");
        transaction = Some(
            fetch_raw_tx(mempool_api, &txid)
                .await
                .context("Failed to fetch raw transaction")?,
        );
    }

    if spv_proof.is_none() {
        println!("Fetching transaction inclusion proof...");
        let tx_bytes =
            hex::decode(&transaction.as_deref().unwrap()).context("Invalid transaction hex")?;
        let txid = consensus::encode::deserialize::<Transaction>(&tx_bytes)
            .context("Failed to parse transaction")?
            .compute_txid();
        spv_proof = Some(
            fetch_spv_proof(mempool_api, &txid.to_string())
                .await
                .context("Failed to fetch SPV proof")?,
        );
    }

    println!("Generating block inclusion proof...");
    let spv_proof_bytes =
        hex::decode(&spv_proof.as_deref().unwrap()).context("Invalid SPV proof hex")?;
    let merkle_block: MerkleBlock =
        consensus::encode::deserialize(&spv_proof_bytes).context("Failed to parse SPV proof")?;
    let block_hash = merkle_block.header.block_hash().to_byte_array();
    let header_proof = generate_header_merkle_proof(block_hash)
        .context("Header not found in known header set")?;
    let block_inclusion_root = headers_merkle_root();
    anyhow::ensure!(
        verify_header_merkle_proof(block_hash, &header_proof, block_inclusion_root),
        "Failed to generate valid block inclusion proof"
    );

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
        .context("Failed to write prover input")?
        .build()
        .context("Failed to build prover environment")?;

    // Prove the program and obtain the receipt.
    println!("Proving...");
    let prover = default_prover();
    let opts = risc0_zkvm::ProverOpts::groth16();
    let receipt = prover
        .prove_with_opts(env, OGZKP_ELF, &opts)
        .context("Proving failed")?
        .receipt;

    // Verify correct data commitments in the receipt journal.
    let (block_inclusion_root, block_month, identity): ([u8; 32], String, String) =
        receipt.journal.decode().context("Failed to decode receipt journal")?;
    println!();
    println!("Proof generated successfully!");
    println!(
        "Block inclusion root: {:?}",
        hex::encode(block_inclusion_root)
    );
    println!("Block month: {:?}", block_month);
    println!("Identity: {:?}", identity);

    // Serialize the receipt and print as bech32m
    let serialized_receipt = serialize_receipt(&receipt)?;
    println!();
    println!("Proof:");
    println!("{}", serialized_receipt);

    Ok(())
}
