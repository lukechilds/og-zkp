use anyhow::Result;

use crate::block_inclusion_proof;

pub fn run(version: &str, image_id: &[u32; 8], json: bool) -> Result<()> {
    let image_id: String = image_id.iter().map(|w| format!("{w:08x}")).collect();

    let inclusion_root: String = block_inclusion_proof::headers_merkle_root()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    let hashes = block_inclusion_proof::get_block_hashes();
    let mut tip = *hashes.last().expect("non-empty block list");
    tip.reverse();
    let inclusion_tip = hex::encode(tip);
    let inclusion_height = hashes.len() - 1;

    if json {
        let output = serde_json::json!({
            "version": version,
            "image_id": image_id,
            "block_inclusion_root": inclusion_root,
            "block_inclusion_tip": inclusion_tip,
            "block_inclusion_height": inclusion_height,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("Version:                  {version}");
        println!("Image ID:                 {image_id}");
        println!("Block inclusion root:     {inclusion_root}");
        println!("Block inclusion tip:      {inclusion_tip}");
        println!("Block inclusion height:   {inclusion_height}");
    }
    Ok(())
}
