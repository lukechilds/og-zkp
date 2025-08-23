use bitcoin::secp256k1::Secp256k1;
use bitcoin::sign_message::{signed_msg_hash, MessageSignature, MessageSignatureError};
use bitcoin::PublicKey;

pub const OGZKP_MESSAGE_PREFIX: &str = "og-zkp ";

pub fn recover_pubkey_from_bitcoin_signed_message(
    signature_bytes: &[u8],
    message: &str,
) -> Result<PublicKey, MessageSignatureError> {
    let signature = MessageSignature::from_slice(signature_bytes)?;
    let hash = signed_msg_hash(message);
    let secp = Secp256k1::verification_only();
    let pubkey = signature.recover_pubkey(&secp, hash)?;

    Ok(pubkey)
}
