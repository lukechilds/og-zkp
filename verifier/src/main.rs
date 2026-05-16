use clap::Parser;

#[derive(Parser)]
#[command(name = "og-zkp-verifier", version, about = "og-zkp verifier CLI")]
struct Cli {
    /// Bech32m-encoded serialized receipt
    proof: String,
    /// Output result as JSON
    #[arg(long)]
    json: bool,
}

fn parse_image_id(hex_str: &str) -> [u32; 8] {
    let hex_str = hex_str.trim();
    assert!(hex_str.len() == 64, "image ID must be 64 hex chars");
    let mut id = [0u32; 8];
    for (i, chunk) in hex_str.as_bytes().chunks(8).enumerate() {
        let s = std::str::from_utf8(chunk).unwrap();
        id[i] = u32::from_str_radix(s, 16).unwrap();
    }
    id
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let image_id = parse_image_id(include_str!("../../host/expected-image-id"));
    og_zkp_core::verify::run(image_id, &cli.proof, cli.json)
}
