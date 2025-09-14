use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::env;
use std::process::Command;

// End-to-end test: prove then verify. Uses RISC0_DEV_MODE.
// This test exercises the CLI with real network calls unless tx/proof are provided.
// To keep it deterministic and fast, we pass known inputs and rely on dev mode proving.

fn prove_then_verify_case(
    message: &str,
    signature: &str,
    address: &str,
    transaction: Option<&str>,
    spv_proof: Option<&str>,
    expected_identity: &str,
    expected_block_month: &str,
) {
    // Build command
    let mut prove_cmd = Command::cargo_bin("og-zkp").expect("binary exists");
    prove_cmd
        .env("RISC0_DEV_MODE", "1")
        .arg("prove")
        .arg("--message")
        .arg(message)
        .arg("--signature")
        .arg(signature)
        .arg("--address")
        .arg(address);

    // Add optional arguments
    if let Some(tx) = transaction {
        prove_cmd.arg("--transaction").arg(tx);
    }
    if let Some(proof) = spv_proof {
        prove_cmd.arg("--spv-proof").arg(proof);
    }

    // Run command and grab stdout
    let prove_output = prove_cmd.assert().success().get_output().stdout.clone();

    // Convert to string
    let stdout_str = String::from_utf8(prove_output).expect("utf8 stdout");

    // Grab the receipt from the first line starting with ogzkp1
    let receipt = stdout_str
        .lines()
        .find(|line| line.starts_with("ogzkp1"))
        .expect("receipt line present starting with ogzkp1")
        .trim()
        .to_string();
    assert!(receipt.starts_with("ogzkp1"));

    // Verify the receipt
    let mut verify_cmd = Command::cargo_bin("og-zkp").expect("binary exists");
    verify_cmd
        .env("RISC0_DEV_MODE", "1")
        .arg("verify")
        .arg(&receipt)
        .assert()
        .success()
        // Check for expected identity and block month in output
        .stdout(predicate::str::contains(format!(
            "Identity: \"{}\"",
            expected_identity
        )))
        .stdout(predicate::str::contains(format!(
            "Block month: \"{}\"",
            expected_block_month
        )));
}

#[test]
fn prove_then_verify_p2pkh() {
    // Inputs
    let message = "og-zkp x.com/lukechilds";
    let signature =
        "HE6QfyPFmJvCGjWohZYAVa+pbdSRjeQpdNbXp6zNbDCnEN65xmK+WYidKlt6J1E/GDJpmLcatjEazVo5wqOg6wM=";
    let address = "1LukeQU5jwebXbMLDVydeH4vFSobRV9rkj";
    let mut transaction = None;
    let mut spv_proof = None;

    // If OGZKP_TEST_OFFLINE_MODE is set, use local tx and spv proof
    if env::var("OGZKP_TEST_OFFLINE_MODE").is_ok() {
        transaction = Some("01000000000101b0fbbbccf54b14e37ef8709a4410bdd6db580fa7dc628106619e71220068ce630500000017160014b44aa221767d8b6d6e1300973719587b4a70ff78ffffffff02360f000000000000160014a6366f1659517ec2af9001e468f5defade31178510270000000000001976a914da6473ed373e08f46dd8003fca7ba72fbe9c555e88ac0247304402207fbb404bc1d79eaea59fd091f7df0c0cbfc367dec29b945be3f829ed742e01f60220295547047d5cf5ec27ce4bb25ba767cf560046e8c126795ed5e04d4810090fd2012103a66311e6776633e771690bdfa7bfea4a74cfb93b21005b704037671c710d0b9200000000");
        spv_proof = Some("00000020c12df22f6c84f6927e63323d6762077ebc7b6d9e7a1020000000000000000000979f6cab6cd5cd47f6057bd1d49871f5a352b111f47a61c5ed8f9d371b6f8634a7d6c15b91c125176669d50d200b00000d71b32e7ebaecc4f44c637c224b5d54fc23fa37fb4faca57fd9603c5a21f373e20d91ab1288f566a5689d8ed3d2b17c94e8ce3320b03882cef7331309472f28cc3631028ba0a6ac08b7fd623dffb48e084bc587963426927cb7826ba6926ca7c22958835f6f94685952f3d9ff12ac91f6a3349dd000b29cf297220eefd4bf20e3c51eb470a7c94ccecca95db3695c110e9f89502ac4cdf64d6d1ddf0e48b20a819327bbe5c2da1adca6aacbfe378123dc77ae993dd7a9330dd43acf83b2ee6cde4d89f02d01864936146f6070f05fb8409789df8ab7414baadd5763f32ca3fdcb4fef6d7f3c1e5d0bea733b2fd644fa456cdf73f21eb7e8866a2721d79266e9e834282a6acaa3fe1d08aa11d5f0892dc7f9ff629f2efb8a7e190fb7e1e029b4cde8c0b00eea6730d9258aca5208fe5afcc5041e547b9a81f5d54f2ce9bec1e9e3b532622be369a572a783fbaaa2631dc86b7a0f1b412adeb57c51a42f5b8bd8436b31bc8defacd89a6030086fe0abbc4a8be51ed2280d448843b44eb1a28b145981fc4900f9709e02223b25f12baf7c664c15482b0818f538d4e4deb96a1cb3cd04bb550b00");
    }

    // Expected outputs
    let expected_identity = "x.com/lukechilds";
    let expected_block_month = "1538352000"; // 2018-10-01T00:00:00Z

    prove_then_verify_case(
        message,
        signature,
        address,
        transaction,
        spv_proof,
        expected_identity,
        expected_block_month,
    );
}
