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

fn main() {
    let proof = std::env::args()
        .nth(1)
        .expect("usage: og-zkp-verifier <proof>");
    let image_id = parse_image_id(include_str!("../../host/expected-image-id"));
    match og_zkp_core::verify::verify_and_extract(&proof, image_id) {
        Ok(result) => {
            let output = serde_json::json!({
                "valid": true,
                "block_inclusion_root": result.block_inclusion_root,
                "block_month": result.block_month,
                "identity": result.identity,
            });
            println!("{}", output);
        }
        Err(e) => {
            let output = serde_json::json!({
                "valid": false,
                "error": e.to_string(),
            });
            println!("{}", output);
            std::process::exit(1);
        }
    }
}
