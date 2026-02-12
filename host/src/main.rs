use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ogzkp", version, about = "og-zkp CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a proof and return the serialized receipt
    Prove {
        /// Message that was signed (must start with "og-zkp ")
        #[arg(long, required = true)]
        message: String,
        /// Base64-encoded signature of the message
        #[arg(long, required = true)]
        signature: String,
        /// Bitcoin address the message was signed by
        #[arg(long, required = true)]
        address: String,
        /// Optional raw transaction hex (skips fetching)
        #[arg(long)]
        transaction: Option<String>,
        /// Optional raw SPV proof (MerkleBlock) hex (skips fetching)
        #[arg(long)]
        spv_proof: Option<String>,
        /// Optional mempool API endpoint
        #[arg(long, default_value = "https://mempool.space/api")]
        mempool_api: String,
    },
    /// Verify a serialized receipt and print committed data
    Verify {
        /// Bech32m-encoded serialized receipt
        receipt: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Prove {
            message,
            signature,
            address,
            transaction,
            spv_proof,
            mempool_api,
        } => {
            ogzkp_core::prove::run(
                methods::OGZKP_ELF,
                &message,
                &signature,
                &address,
                &mempool_api,
                transaction,
                spv_proof,
            )
            .await
        }
        Commands::Verify { receipt } => ogzkp_core::verify::run(methods::OGZKP_ID, &receipt),
    }
}

#[cfg(test)]
mod tests {
    // Verify the guest program image ID hasn't changed unexpectedly.
    // If this test fails it means the guest binary has changed which will invalidate all existing proofs.
    // Update the expected value only after intentional guest changes.
    #[test]
    fn guest_image_id_unchanged() {
        let expected: [u32; 8] = [
            3375371137, 2537482033, 3532430041, 1499898363, 3758672109, 1329636390, 747559332,
            1123749093,
        ];
        assert_eq!(
            methods::OGZKP_ID,
            expected,
            "Guest image ID has changed! This will break verification of existing proofs. \
             If this change is intentional, update the expected value in this test."
        );
    }
}
