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

use time::{Date, OffsetDateTime};
pub fn start_of_month(unix_ts: u32) -> u32 {
    let ts = OffsetDateTime::from_unix_timestamp(unix_ts.into()).unwrap();

    let date = Date::from_calendar_date(ts.year(), ts.month(), 1).unwrap();

    date.with_hms(0, 0, 0)
        .unwrap()
        .assume_utc()
        .unix_timestamp() as u32
}
