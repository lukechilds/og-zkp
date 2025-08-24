use methods::OGZKP_ID;

use crate::receipt::deserialize_receipt;

pub fn run(receipt_str: &str) {
    let receipt = deserialize_receipt(receipt_str);
    receipt.verify(OGZKP_ID).unwrap();
    let (block_inclusion_root, block_month, identity): ([u8; 32], String, String) =
        receipt.journal.decode().unwrap();
    println!();
    println!(
        "Block inclusion root: {:?}",
        hex::encode(block_inclusion_root)
    );
    println!("Block month: {:?}", block_month);
    println!("Identity: {:?}", identity);
    println!();
}
