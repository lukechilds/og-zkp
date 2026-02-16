use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "og-zkp", version, about = "og-zkp CLI")]
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
        /// Disable terminal animation
        #[arg(long)]
        no_animation: bool,
    },
    /// Verify a serialized receipt and print committed data
    Verify {
        /// Bech32m-encoded serialized receipt
        receipt: String,
        /// Output result as JSON
        #[arg(long)]
        json: bool,
    },
    /// Display build info (image ID, block inclusion data, etc.)
    Info {
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
            no_animation,
        } => {
            og_zkp_core::prove::run(
                methods::OG_ZKP_ELF,
                &message,
                &signature,
                &address,
                &mempool_api,
                transaction,
                spv_proof,
                json,
                no_animation,
            )
            .await
        }
        Commands::Verify { receipt, json } => {
            og_zkp_core::verify::run(methods::OG_ZKP_ID, &receipt, json)
        }
        Commands::Info { json } => {
            og_zkp_core::info::run(env!("CARGO_PKG_VERSION"), &methods::OG_ZKP_ID, json)
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
        let expected = include_str!("../expected-image-id");
        let actual: String = methods::OG_ZKP_ID
            .iter()
            .map(|w| format!("{w:08x}"))
            .collect();
        assert_eq!(
            actual, expected,
            "Guest image ID has changed! This will break verification of existing proofs. \
             If this change is intentional, update the expected value in this test."
        );
    }
}
