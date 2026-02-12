use rs_merkle::{algorithms::Sha256, MerkleProof};

// Verify the single-blob proof against an expected root.
pub fn verify_header_merkle_proof(
    leaf_hash_be: [u8; 32],
    blob: &[u8],
    expected_root: [u8; 32],
) -> bool {
    let index = u32::from_le_bytes(blob[0..4].try_into().unwrap()) as usize;
    let total_leaves_count = u32::from_le_bytes(blob[4..8].try_into().unwrap()) as usize;
    let proof_bytes = &blob[8..];
    let proof = MerkleProof::<Sha256>::from_bytes(proof_bytes).unwrap();
    proof.verify(expected_root, &[index], &[leaf_hash_be], total_leaves_count)
}

#[cfg(feature = "host")]
use rs_merkle::MerkleTree;

#[cfg(feature = "host")]
pub fn headers_merkle_root() -> [u8; 32] {
    MerkleTree::<Sha256>::from_leaves(&get_block_hashes())
        .root()
        .expect("non-empty")
}

// Produce a single serialized proof blob:
// 4 byte index + 4 byte total_leaves + proof bytes
#[cfg(feature = "host")]
pub fn generate_header_merkle_proof(target_hash_be: [u8; 32]) -> Option<Vec<u8>> {
    let leaves = get_block_hashes();
    let index = leaves.iter().position(|h| *h == target_hash_be)?; // index in the known set
    let total_leaves = leaves.len() as u32;

    let tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    let proof = tree.proof(&[index]).to_bytes();

    let mut blob = Vec::with_capacity(8 + proof.len());
    blob.extend_from_slice(&(index as u32).to_le_bytes());
    blob.extend_from_slice(&total_leaves.to_le_bytes());
    blob.extend_from_slice(&proof);
    Some(blob)
}

// Statically load blocks from text file at compile time
#[cfg(feature = "host")]
use hex::FromHex;

#[cfg(feature = "host")]
fn get_block_hashes() -> Vec<[u8; 32]> {
    include_str!("blockhashes.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|line| {
            // Parse hex string directly into a [u8; 32]
            let mut bytes: [u8; 32] = <[u8; 32]>::from_hex(line).expect("bad hex or wrong length");
            // Reverse to little-endian
            bytes.reverse();
            bytes
        })
        .collect()
}

#[cfg(test)]
#[cfg(feature = "host")]
mod tests {
    use super::*;

    // Bitcoin genesis block hash (big-endian hex), reversed to little-endian by get_block_hashes()
    fn genesis_block_hash() -> [u8; 32] {
        let mut bytes: [u8; 32] = <[u8; 32]>::from_hex(
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
        )
        .unwrap();
        bytes.reverse();
        bytes
    }

    #[test]
    fn merkle_root_is_deterministic() {
        let root1 = headers_merkle_root();
        let root2 = headers_merkle_root();
        assert_eq!(root1, root2);
    }

    #[test]
    fn generate_proof_for_known_hash() {
        let proof = generate_header_merkle_proof(genesis_block_hash());
        assert!(proof.is_some());
    }

    #[test]
    fn generate_proof_for_unknown_hash_returns_none() {
        let proof = generate_header_merkle_proof([0xff; 32]);
        assert!(proof.is_none());
    }

    #[test]
    fn generate_and_verify_proof_round_trip() {
        let hash = genesis_block_hash();
        let root = headers_merkle_root();
        let proof = generate_header_merkle_proof(hash).unwrap();
        assert!(verify_header_merkle_proof(hash, &proof, root));
    }

    #[test]
    fn verify_proof_fails_with_wrong_hash() {
        let hash = genesis_block_hash();
        let root = headers_merkle_root();
        let proof = generate_header_merkle_proof(hash).unwrap();
        assert!(!verify_header_merkle_proof([0xff; 32], &proof, root));
    }

    #[test]
    fn verify_proof_fails_with_wrong_root() {
        let hash = genesis_block_hash();
        let proof = generate_header_merkle_proof(hash).unwrap();
        assert!(!verify_header_merkle_proof(hash, &proof, [0xff; 32]));
    }
}
