use risc0_zkvm::{default_prover, ExecutorEnv};

use anyhow::{bail, Context, Result};
use bitcoin::consensus;
use bitcoin::hashes::Hash;
use bitcoin::MerkleBlock;
use bitcoin::{Address, AddressType, Network, Transaction};
use std::io::IsTerminal;
use std::str::FromStr;

use base64::prelude::*;

use crate::animation::Animation;
use crate::block_inclusion_proof::{
    generate_header_merkle_proof, headers_merkle_root, verify_header_merkle_proof,
};
use crate::mempool_api::{fetch_raw_tx, fetch_spv_proof, find_first_seen_txid};
use crate::receipt::serialize_receipt;
use crate::AddressKind;

pub async fn run(
    elf: &[u8],
    message: &str,
    signature: &str,
    address: &str,
    mempool_api: &str,
    transaction: Option<String>,
    spv_proof: Option<String>,
    json: bool,
    no_animation: bool,
) -> Result<()> {
    let seed = format!("{message}{signature}{address}");
    let anim = if !json && !no_animation && std::io::stderr().is_terminal() {
        Some(Animation::start(&seed))
    } else {
        None
    };

    macro_rules! status {
        ($($arg:tt)*) => {
            match &anim {
                Some(a) => a.set_status(&format!($($arg)*)),
                None => eprintln!($($arg)*),
            }
        }
    }

    status!("og-zkp");

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
        Some(AddressType::P2pkh) => AddressKind::P2pkh,
        Some(AddressType::P2sh) => AddressKind::P2sh, // Assume nested P2WPKH
        Some(AddressType::P2wpkh) => AddressKind::P2wpkh,
        Some(AddressType::P2tr) => AddressKind::P2tr,
        _ => bail!("Unsupported address type"),
    };

    // Fetch transaction if not provided
    let transaction = match transaction {
        Some(tx) => tx,
        None => {
            // Lookup first-seen txid for this address
            status!("Looking up first-seen txid for address...");
            let txid = match find_first_seen_txid(mempool_api, address).await {
                Ok(Some(txid)) => txid,
                Ok(None) => bail!("No transactions found for address"),
                Err(e) => bail!("Failed to fetch transactions: {}", e),
            };
            status!("First seen txid: {txid}");
            status!("Fetching raw tx...");
            fetch_raw_tx(mempool_api, &txid)
                .await
                .context("Failed to fetch raw transaction")?
        }
    };

    // Fetch SPV proof if not provided
    let spv_proof = match spv_proof {
        Some(proof) => proof,
        None => {
            status!("Fetching transaction inclusion proof...");
            let tx_bytes = hex::decode(&transaction).context("Invalid transaction hex")?;
            let txid = consensus::encode::deserialize::<Transaction>(&tx_bytes)
                .context("Failed to parse transaction")?
                .compute_txid();
            fetch_spv_proof(mempool_api, &txid.to_string())
                .await
                .context("Failed to fetch SPV proof")?
        }
    };

    status!("Generating block inclusion proof...");
    let spv_proof_bytes = hex::decode(&spv_proof).context("Invalid SPV proof hex")?;
    let merkle_block: MerkleBlock =
        consensus::encode::deserialize(&spv_proof_bytes).context("Failed to parse SPV proof")?;
    let block_hash = merkle_block.header.block_hash().to_byte_array();
    let header_proof =
        generate_header_merkle_proof(block_hash).context("Header not found in known header set")?;
    let block_inclusion_root = headers_merkle_root();
    anyhow::ensure!(
        verify_header_merkle_proof(block_hash, &header_proof, block_inclusion_root),
        "Failed to generate valid block inclusion proof"
    );

    // Bundle input
    let input = (
        message,
        signature_bytes,
        address_type as i32,
        transaction,
        spv_proof,
        header_proof,
        block_inclusion_root,
    );
    let env = ExecutorEnv::builder()
        .write(&input)
        .context("Failed to write prover input")?
        .build()
        .context("Failed to build prover environment")?;

    // Prove the program and obtain the receipt.
    status!("Proving...");
    let prover = default_prover();
    let opts = risc0_zkvm::ProverOpts::groth16();
    let receipt = prover
        .prove_with_opts(env, elf, &opts)
        .context("Proving failed")?
        .receipt;

    // Verify correct data commitments in the receipt journal.
    let (block_inclusion_root, block_month, identity): ([u8; 32], String, String) = receipt
        .journal
        .decode()
        .context("Failed to decode receipt journal")?;

    // Serialize the receipt
    let serialized_receipt = serialize_receipt(&receipt)?;

    status!("Proof generated successfully");
    drop(anim);

    if json {
        let output = serde_json::json!({
            "block_inclusion_root": hex::encode(block_inclusion_root),
            "block_month": block_month,
            "identity": identity,
            "proof": serialized_receipt,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        let month_display = block_month
            .parse::<i64>()
            .ok()
            .and_then(|ts| time::OffsetDateTime::from_unix_timestamp(ts).ok())
            .map(|dt| format!("{} {}", dt.month(), dt.year()))
            .unwrap_or(block_month.clone());
        println!();
        println!("OG Status: {month_display}");
        println!("Identity:  {identity}");
        println!();
        println!("Proof:");
        println!("{serialized_receipt}");
    }

    Ok(())
}
