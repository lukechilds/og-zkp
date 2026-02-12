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
        /// Output result as JSON
        #[arg(long)]
        json: bool,
    },
    /// Verify a serialized receipt and print committed data
    Verify {
        /// Bech32m-encoded serialized receipt
        receipt: String,
        /// Output result as JSON
        #[arg(long)]
        json: bool,
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
            json,
        } => {
            ogzkp_core::prove::run(
                methods::OGZKP_ELF,
                &message,
                &signature,
                &address,
                &mempool_api,
                transaction,
                spv_proof,
                json,
            )
            .await
        }
        Commands::Verify { receipt, json } => {
            ogzkp_core::verify::run(methods::OGZKP_ID, &receipt, json)
        }
    }
}

#[cfg(test)]
mod tests {
    // Verify the guest program image ID hasn't changed unexpectedly.
    // If this test fails it means the guest binary has changed which will invalidate all existing proofs.
    // Update the expected value only after intentional guest changes.
    #[test]
    fn guest_image_id_unchanged() {
        let expected = "56ca324e6a2e307aee7a30a313859463d0b9154c5a5a4d95aef8d14220491bf9";
        let actual: String = methods::OGZKP_ID.iter().map(|w| format!("{w:08x}")).collect();
        assert_eq!(
            actual,
            expected,
            "Guest image ID has changed! This will break verification of existing proofs. \
             If this change is intentional, update the expected value in this test."
        );
    }
}
