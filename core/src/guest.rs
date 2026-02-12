use bitcoin::hashes::Hash as _;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::sign_message::{signed_msg_hash, MessageSignature, MessageSignatureError};
use bitcoin::{consensus, script::ScriptBuf, MerkleBlock, PublicKey, Transaction, Txid};

use time::{Date, OffsetDateTime};

use crate::block_inclusion_proof::verify_header_merkle_proof;
use crate::AddressKind;

const OGZKP_MESSAGE_PREFIX: &str = "og-zkp ";

pub type Input = (String, Vec<u8>, i32, String, String, Vec<u8>, [u8; 32]);
pub type Output = ([u8; 32], String, String);

pub fn run(input: Input) -> Output {
    let (
        message,
        signature_bytes,
        address_type_raw,
        tx_hex,
        spv_proof_hex,
        header_proof,
        block_inclusion_root,
    ) = input;

    let address_type = AddressKind::try_from(address_type_raw).expect("Invalid address type");

    // Assert message starts with "og-zkp"
    assert!(
        message.starts_with(OGZKP_MESSAGE_PREFIX),
        "Message does not start with '{OGZKP_MESSAGE_PREFIX}'"
    );

    // Recover pubkey from the signed message
    let pubkey = recover_pubkey_from_bitcoin_signed_message(&signature_bytes, &message)
        .expect("Failed to recover pubkey from bitcoin signed message");

    // Decode transaction hex
    let tx_bytes = hex::decode(tx_hex).expect("Invalid transaction hex");
    let tx: Transaction =
        consensus::encode::deserialize(&tx_bytes).expect("Failed to parse transaction");

    // Assert pubkey is included in an output in the transaction
    let expected_output = pubkey_to_output_script(pubkey, address_type);
    let tx_has_expected_output = tx
        .output
        .iter()
        .any(|output| output.script_pubkey == expected_output);
    assert!(
        tx_has_expected_output,
        "Recovered pubkey is not a P2PKH output in the transaction"
    );

    // Assert tx inclusion proof is valid for block header
    let txid = tx.compute_txid();
    let spv_proof_bytes = hex::decode(spv_proof_hex).expect("Invalid SPV proof hex");
    let spv_proof: MerkleBlock =
        consensus::encode::deserialize(&spv_proof_bytes).expect("Failed to parse SPV proof");
    let mut matches: Vec<Txid> = vec![];
    let mut indexes: Vec<u32> = vec![];
    spv_proof
        .extract_matches(&mut matches, &mut indexes)
        .expect("Failed to extract matches from SPV proof");
    assert!(matches.contains(&txid), "Transaction is not in SPV proof");

    // Verify header inclusion against embedded merkle root over known headers
    let block_hash = spv_proof.header.block_hash().to_byte_array();
    assert!(
        verify_header_merkle_proof(block_hash, &header_proof, block_inclusion_root),
        "Header not included in known header set"
    );

    // Calculate the time of the start of the calender month the tx was confirmed in
    let block_time = spv_proof.header.time;
    let block_month = start_of_month(block_time).to_string();

    // Grab identity from the message
    let identity = message.strip_prefix(OGZKP_MESSAGE_PREFIX).unwrap();

    (block_inclusion_root, block_month, identity.to_string())
}

pub(crate) fn pubkey_to_output_script(pubkey: PublicKey, address_type: AddressKind) -> ScriptBuf {
    match address_type {
        AddressKind::P2pkh => ScriptBuf::new_p2pkh(&pubkey.pubkey_hash()),
        AddressKind::P2sh => {
            let wpkh = pubkey.wpubkey_hash().unwrap();
            let redeem = ScriptBuf::new_p2wpkh(&wpkh);
            ScriptBuf::new_p2sh(&redeem.script_hash())
        }
        AddressKind::P2wpkh => {
            let wpkh = pubkey.wpubkey_hash().unwrap();
            ScriptBuf::new_p2wpkh(&wpkh)
        }
        AddressKind::P2tr => {
            let secp = Secp256k1::verification_only();
            let xonly = bitcoin::XOnlyPublicKey::from(pubkey);
            ScriptBuf::new_p2tr(&secp, xonly, None)
        }
    }
}

pub(crate) fn recover_pubkey_from_bitcoin_signed_message(
    signature_bytes: &[u8],
    message: &str,
) -> Result<PublicKey, MessageSignatureError> {
    let signature = MessageSignature::from_slice(signature_bytes)?;
    let hash = signed_msg_hash(message);
    let secp = Secp256k1::verification_only();
    let pubkey = signature.recover_pubkey(&secp, hash)?;

    Ok(pubkey)
}

pub(crate) fn start_of_month(unix_ts: u32) -> u32 {
    let ts = OffsetDateTime::from_unix_timestamp(unix_ts.into()).unwrap();

    let date = Date::from_calendar_date(ts.year(), ts.month(), 1).unwrap();

    date.with_hms(0, 0, 0)
        .unwrap()
        .assume_utc()
        .unix_timestamp() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::prelude::*;
    use time::Month;

    fn ts(year: i32, month: Month, day: u8, hour: u8, min: u8, sec: u8) -> u32 {
        Date::from_calendar_date(year, month, day)
            .unwrap()
            .with_hms(hour, min, sec)
            .unwrap()
            .assume_utc()
            .unix_timestamp() as u32
    }

    // -- start_of_month tests --

    #[test]
    fn start_of_month_mid_month() {
        let input = ts(2018, Month::October, 15, 12, 0, 0);
        let expected = ts(2018, Month::October, 1, 0, 0, 0);
        assert_eq!(start_of_month(input), expected);
    }

    #[test]
    fn start_of_month_first_of_month() {
        let input = ts(2024, Month::January, 1, 0, 0, 0);
        assert_eq!(start_of_month(input), input);
    }

    #[test]
    fn start_of_month_last_day_of_year() {
        let input = ts(2024, Month::December, 31, 23, 59, 59);
        let expected = ts(2024, Month::December, 1, 0, 0, 0);
        assert_eq!(start_of_month(input), expected);
    }

    #[test]
    fn start_of_month_leap_year_february() {
        let input = ts(2024, Month::February, 29, 15, 0, 0);
        let expected = ts(2024, Month::February, 1, 0, 0, 0);
        assert_eq!(start_of_month(input), expected);
    }

    // -- recover_pubkey tests --

    // P2PKH test vector from e2e
    const P2PKH_MESSAGE: &str = "og-zkp x.com/lukechilds";
    const P2PKH_SIGNATURE_B64: &str =
        "HE6QfyPFmJvCGjWohZYAVa+pbdSRjeQpdNbXp6zNbDCnEN65xmK+WYidKlt6J1E/GDJpmLcatjEazVo5wqOg6wM=";
    const P2PKH_TX_HEX: &str = "01000000000101b0fbbbccf54b14e37ef8709a4410bdd6db580fa7dc628106619e71220068ce630500000017160014b44aa221767d8b6d6e1300973719587b4a70ff78ffffffff02360f000000000000160014a6366f1659517ec2af9001e468f5defade31178510270000000000001976a914da6473ed373e08f46dd8003fca7ba72fbe9c555e88ac0247304402207fbb404bc1d79eaea59fd091f7df0c0cbfc367dec29b945be3f829ed742e01f60220295547047d5cf5ec27ce4bb25ba767cf560046e8c126795ed5e04d4810090fd2012103a66311e6776633e771690bdfa7bfea4a74cfb93b21005b704037671c710d0b9200000000";

    fn p2pkh_signature_bytes() -> Vec<u8> {
        BASE64_STANDARD.decode(P2PKH_SIGNATURE_B64).unwrap()
    }

    fn recover_p2pkh_pubkey() -> PublicKey {
        recover_pubkey_from_bitcoin_signed_message(&p2pkh_signature_bytes(), P2PKH_MESSAGE).unwrap()
    }

    #[test]
    fn recover_pubkey_valid_signature() {
        let pubkey = recover_p2pkh_pubkey();
        // Sanity: the recovered key should produce a P2PKH output that exists in the tx
        let script = pubkey_to_output_script(pubkey, AddressKind::P2pkh);
        let tx_bytes = hex::decode(P2PKH_TX_HEX).unwrap();
        let tx: Transaction = consensus::encode::deserialize(&tx_bytes).unwrap();
        assert!(tx.output.iter().any(|o| o.script_pubkey == script));
    }

    #[test]
    fn recover_pubkey_invalid_signature_bytes() {
        let result =
            recover_pubkey_from_bitcoin_signed_message(&[0xde, 0xad, 0xbe, 0xef], P2PKH_MESSAGE);
        assert!(result.is_err());
    }

    // -- pubkey_to_output_script tests --

    #[test]
    fn pubkey_to_p2pkh_script() {
        let pubkey = recover_p2pkh_pubkey();
        let script = pubkey_to_output_script(pubkey, AddressKind::P2pkh);
        let tx_bytes = hex::decode(P2PKH_TX_HEX).unwrap();
        let tx: Transaction = consensus::encode::deserialize(&tx_bytes).unwrap();
        assert!(
            tx.output.iter().any(|o| o.script_pubkey == script),
            "P2PKH script not found in transaction outputs"
        );
    }

    #[test]
    fn pubkey_to_p2sh_script() {
        // P2SH test vector from e2e
        let sig_bytes = BASE64_STANDARD
            .decode("IEfY9NgVNcWF863TzdsnMBrfC9WQ3S+/Fb29r217ofHZNVwWa/R+JpTluVRuQif+cOAcbcv1qpR2L57n8slmzQU=")
            .unwrap();
        let pubkey =
            recover_pubkey_from_bitcoin_signed_message(&sig_bytes, P2PKH_MESSAGE).unwrap();
        let script = pubkey_to_output_script(pubkey, AddressKind::P2sh);
        let tx_hex = "0100000001c2360d0a190fde0962096ff2b05c5fafea40c1927852096671f2120f72fbb92c000000008a473044022037efa5e6ed145347836edac5cafc6f1c9485d430a608f11cc6f098f6236c3f8a0220070b5ef93b4ebe118d42ee8cb8c052511408943d45e76458f3292dd978a8fac60141040e3a759c33b03e1af8e5d86fb447a40eff244c847a4f8274276db490054e8be076f8801ddc9c5246ee86b6f33cfe38e8b7e57ab9db390eb3ec1ec6ae9eeea113fdffffff02102700000000000017a914d2d309d3d7ed9ee462e3643d7215951e0bd742c68721e61c00000000001976a914da6473ed373e08f46dd8003fca7ba72fbe9c555e88ac73a20800";
        let tx_bytes = hex::decode(tx_hex).unwrap();
        let tx: Transaction = consensus::encode::deserialize(&tx_bytes).unwrap();
        assert!(
            tx.output.iter().any(|o| o.script_pubkey == script),
            "P2SH script not found in transaction outputs"
        );
    }

    #[test]
    fn pubkey_to_p2wpkh_script() {
        // P2WPKH test vector from e2e
        let sig_bytes = BASE64_STANDARD
            .decode("Hy4jk2Sa62j6xN0ud5QqJTlzXuM8sn/g+TevYcBwLiJcAWc/9+4B0ljQgTD+5hp1KMvROkW4k1MYGigbza1K+ZU=")
            .unwrap();
        let pubkey =
            recover_pubkey_from_bitcoin_signed_message(&sig_bytes, P2PKH_MESSAGE).unwrap();
        let script = pubkey_to_output_script(pubkey, AddressKind::P2wpkh);
        let tx_hex = "0100000001b202eaa7a6aab448409199997f246be312dc45cee432024c8b90e0c13236853a010000008a4730440220276fdcb55fe9f28975481f7612584d45feed0901758be48453fd1f6c139d573a022020e12c981bb5ce19f845b57440942df53b002e74c44d50a51a9ee030a1f1ea6e0141040e3a759c33b03e1af8e5d86fb447a40eff244c847a4f8274276db490054e8be076f8801ddc9c5246ee86b6f33cfe38e8b7e57ab9db390eb3ec1ec6ae9eeea113fdffffff021027000000000000160014ff2d9201f8d1585f7210d1d395ab5695b101936d649f1c00000000001976a914da6473ed373e08f46dd8003fca7ba72fbe9c555e88ac73a20800";
        let tx_bytes = hex::decode(tx_hex).unwrap();
        let tx: Transaction = consensus::encode::deserialize(&tx_bytes).unwrap();
        assert!(
            tx.output.iter().any(|o| o.script_pubkey == script),
            "P2WPKH script not found in transaction outputs"
        );
    }

    #[test]
    fn pubkey_to_p2tr_script() {
        // P2TR test vector from e2e
        let sig_bytes = BASE64_STANDARD
            .decode("IOVIcHmzE1koFMStNWd7i3mBWsOjU8Xwy0vSFEk8WU96WPNlrcNlW9lB0Gcx+38MB9UQ1B5IqeYpCEauF8aVxpU=")
            .unwrap();
        let pubkey =
            recover_pubkey_from_bitcoin_signed_message(&sig_bytes, P2PKH_MESSAGE).unwrap();
        let script = pubkey_to_output_script(pubkey, AddressKind::P2tr);
        let tx_hex = "01000000000101d528565e141e0e35e1878e2ea25d347196706dd3b75d85f8a8882f14775870280600000000ffffffff059104000000000000160014798bbdd9e1748f67c0204b0915a03d46c61ab2771413010000000000225120ff2d9c184a9a439c27231e37c5a811e1ee4117c8389144b7bd85bb29672ec8a0599e05000000000017a914fca7c972b1b29359db9496a9ae6f6b3c049d998b878849000000000000160014b12d73d1ba48a7df0571d9de2a546db6f56bb313837d4e5401000000160014eed12e1ba4703f0a6afe1a3dd4a3b1de86803936024730440220553033d4d5019ccfec66cf3fb94bd00fe26f381ecdaa0f2128e79029a2f3cc60022019e044e808e4aa9d415b9f3f161e286719dbab69155ce4bbb5be523c82b995f5012103a876001a753b82a8bec93c5936e65e0bacacebe5de6ca74adadfced8ebb7ad5300000000";
        let tx_bytes = hex::decode(tx_hex).unwrap();
        let tx: Transaction = consensus::encode::deserialize(&tx_bytes).unwrap();
        assert!(
            tx.output.iter().any(|o| o.script_pubkey == script),
            "P2TR script not found in transaction outputs"
        );
    }

    // -- run() integration test (requires host feature for header merkle proof) --

    #[cfg(feature = "host")]
    #[test]
    fn run_valid_input() {
        use crate::block_inclusion_proof::{generate_header_merkle_proof, headers_merkle_root};
        use bitcoin::hashes::Hash as _;

        let message = P2PKH_MESSAGE.to_string();
        let signature_bytes = p2pkh_signature_bytes();
        let address_type_raw = AddressKind::P2pkh as i32;
        let tx_hex = P2PKH_TX_HEX.to_string();
        let spv_proof_hex = "00000020c12df22f6c84f6927e63323d6762077ebc7b6d9e7a1020000000000000000000979f6cab6cd5cd47f6057bd1d49871f5a352b111f47a61c5ed8f9d371b6f8634a7d6c15b91c125176669d50d200b00000d71b32e7ebaecc4f44c637c224b5d54fc23fa37fb4faca57fd9603c5a21f373e20d91ab1288f566a5689d8ed3d2b17c94e8ce3320b03882cef7331309472f28cc3631028ba0a6ac08b7fd623dffb48e084bc587963426927cb7826ba6926ca7c22958835f6f94685952f3d9ff12ac91f6a3349dd000b29cf297220eefd4bf20e3c51eb470a7c94ccecca95db3695c110e9f89502ac4cdf64d6d1ddf0e48b20a819327bbe5c2da1adca6aacbfe378123dc77ae993dd7a9330dd43acf83b2ee6cde4d89f02d01864936146f6070f05fb8409789df8ab7414baadd5763f32ca3fdcb4fef6d7f3c1e5d0bea733b2fd644fa456cdf73f21eb7e8866a2721d79266e9e834282a6acaa3fe1d08aa11d5f0892dc7f9ff629f2efb8a7e190fb7e1e029b4cde8c0b00eea6730d9258aca5208fe5afcc5041e547b9a81f5d54f2ce9bec1e9e3b532622be369a572a783fbaaa2631dc86b7a0f1b412adeb57c51a42f5b8bd8436b31bc8defacd89a6030086fe0abbc4a8be51ed2280d448843b44eb1a28b145981fc4900f9709e02223b25f12baf7c664c15482b0818f538d4e4deb96a1cb3cd04bb550b00".to_string();

        // Extract block hash from SPV proof to generate header merkle proof
        let spv_bytes = hex::decode(&spv_proof_hex).unwrap();
        let merkle_block: MerkleBlock = consensus::encode::deserialize(&spv_bytes).unwrap();
        let block_hash = merkle_block.header.block_hash().to_byte_array();

        let header_proof = generate_header_merkle_proof(block_hash)
            .expect("block hash should be in known header set");
        let block_inclusion_root = headers_merkle_root();

        let input = (
            message,
            signature_bytes,
            address_type_raw,
            tx_hex,
            spv_proof_hex,
            header_proof,
            block_inclusion_root,
        );

        let (root, block_month, identity) = run(input);

        assert_eq!(root, block_inclusion_root);
        assert_eq!(block_month, "1538352000");
        assert_eq!(identity, "x.com/lukechilds");
    }
}
