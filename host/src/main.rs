// Modules
mod block_inclusion_proof;
mod mempool_api;
mod receipt;

// CLI commands
mod commands;

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
async fn main() {
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
            commands::prove::run(
                &message,
                &signature,
                &address,
                &mempool_api,
                transaction,
                spv_proof,
            )
            .await
        }
        Commands::Verify { receipt } => commands::verify::run(&receipt),
    }
}
