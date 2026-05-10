use assert_cmd::prelude::*;
use base64::prelude::*;
use bitcoin::hashes::Hash as _;
use bitcoin::{consensus, MerkleBlock};
use predicates::prelude::*;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// End-to-end tests using fixture transaction/SPV inputs. Uses RISC0_DEV_MODE.
// To keep it fast we rely on dev mode proving which does not produce valid proofs outside of dev mode.

static RISC0_DEV_MODE_LOCK: Mutex<()> = Mutex::new(());

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("{prefix}-{}-{suffix}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct DockerImage(String);

impl Drop for DockerImage {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["image", "rm", "-f", &self.0])
            .output();
    }
}

fn command_output(command: &mut Command, label: &str) -> Output {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("failed to run {label}: {err}"));
    assert!(
        output.status.success(),
        "{label} failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn prove_then_verify(
    message: &str,
    signature: &str,
    address: &str,
    transaction: &str,
    spv_proof: &str,
    expected_identity: &str,
    expected_og_status: &str,
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
        .arg(address)
        .arg("--transaction")
        .arg(transaction)
        .arg("--spv-proof")
        .arg(spv_proof);

    // Run command and grab stdout
    let prove_output = prove_cmd.assert().success().get_output().stdout.clone();

    // Convert to string
    let stdout_str = String::from_utf8(prove_output).expect("utf8 stdout");

    // Grab the receipt from the first line starting with og-zkp1
    let receipt = stdout_str
        .lines()
        .find(|line| line.starts_with("og-zkp1"))
        .expect("receipt line present starting with og-zkp1")
        .trim()
        .to_string();
    assert!(receipt.starts_with("og-zkp1"));

    // Verify the receipt
    let mut verify_cmd = Command::cargo_bin("og-zkp").expect("binary exists");
    verify_cmd
        .env("RISC0_DEV_MODE", "1")
        .arg("verify")
        .arg(&receipt)
        .assert()
        .success()
        // Check for expected identity and OG status in output
        .stdout(predicate::str::contains(format!(
            "Identity:  {expected_identity}"
        )))
        .stdout(predicate::str::contains(format!(
            "OG Status: {expected_og_status}"
        )));
}

#[test]
fn verify_rejects_fake_genesis_root_proof() {
    let mut merkle_block: MerkleBlock =
        consensus::encode::deserialize(&hex::decode(P2PKH_SPV_PROOF).unwrap()).unwrap();
    merkle_block.header.time = 1231006505;
    let fake_block_inclusion_root = merkle_block.header.block_hash().to_byte_array();
    let fake_spv_proof = hex::encode(consensus::encode::serialize(&merkle_block));
    let mut fake_header_proof = Vec::with_capacity(8);
    fake_header_proof.extend_from_slice(&0u32.to_le_bytes());
    fake_header_proof.extend_from_slice(&1u32.to_le_bytes());

    let input = (
        P2PKH_MESSAGE.to_string(),
        BASE64_STANDARD.decode(P2PKH_SIGNATURE).unwrap(),
        og_zkp_core::AddressKind::P2pkh as i32,
        P2PKH_TRANSACTION.to_string(),
        fake_spv_proof,
        fake_header_proof,
        fake_block_inclusion_root,
    );
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let _guard = RISC0_DEV_MODE_LOCK.lock().unwrap();
    let previous_dev_mode = env::var_os("RISC0_DEV_MODE");
    env::set_var("RISC0_DEV_MODE", "1");
    let receipt = default_prover()
        .prove_with_opts(env, methods::OG_ZKP_ELF, &ProverOpts::groth16())
        .unwrap()
        .receipt;
    if let Some(value) = previous_dev_mode {
        env::set_var("RISC0_DEV_MODE", value);
    } else {
        env::remove_var("RISC0_DEV_MODE");
    }

    let serialized_receipt = og_zkp_core::receipt::serialize_receipt(&receipt).unwrap();

    Command::cargo_bin("og-zkp")
        .unwrap()
        .env("RISC0_DEV_MODE", "1")
        .args(["verify", &serialized_receipt])
        .assert()
        .failure();
}

#[test]
fn prove_reports_supported_tip_when_block_is_not_in_inclusion_root() {
    let mut merkle_block: MerkleBlock =
        consensus::encode::deserialize(&hex::decode(P2PKH_SPV_PROOF).unwrap()).unwrap();
    merkle_block.header.nonce ^= 1;
    let unknown_tip_spv_proof = hex::encode(consensus::encode::serialize(&merkle_block));

    Command::cargo_bin("og-zkp")
        .unwrap()
        .env("RISC0_DEV_MODE", "1")
        .args([
            "prove",
            "--no-animation",
            "--message",
            P2PKH_MESSAGE,
            "--signature",
            P2PKH_SIGNATURE,
            "--address",
            P2PKH_ADDRESS,
            "--transaction",
            P2PKH_TRANSACTION,
            "--spv-proof",
            &unknown_tip_spv_proof,
            "--mempool-api",
            "http://127.0.0.1:9",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Transaction block is not in the block inclusion root",
        ))
        .stderr(predicate::str::contains(format!(
            "only supports proofs up to block height {EXPECTED_INCLUSION_HEIGHT} / tip {EXPECTED_INCLUSION_TIP}"
        )));
}

#[test]
fn docker_image_prove_then_verify_dev_mode() {
    if !(cfg!(target_os = "linux") && cfg!(target_arch = "x86_64")) {
        eprintln!("skipping Docker e2e: test binary must be linux/amd64 for this image");
        return;
    }

    let root = repo_root();
    let context = TempDir::new("og-zkp-docker-e2e");
    fs::copy(
        root.join("docker/Dockerfile"),
        context.path.join("Dockerfile"),
    )
    .unwrap();
    fs::copy(
        root.join("docker/groth16-shim.sh"),
        context.path.join("groth16-shim.sh"),
    )
    .unwrap();
    fs::copy(
        PathBuf::from(env!("CARGO_BIN_EXE_og-zkp")),
        context.path.join("og-zkp"),
    )
    .unwrap();

    let image = DockerImage(format!(
        "og-zkp:e2e-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    let mut build = Command::new("docker");
    build
        .args(["build", "--platform", "linux/amd64", "-t", &image.0])
        .arg(&context.path);
    command_output(&mut build, "docker build");

    // This keeps CI fast and verifies the Docker image can run the prove/verify CLI flow.
    // It does not exercise the Groth16 Docker shim because dev mode skips real proving,
    // and real Groth16 proving is too slow for this CI test.
    let mut prove = Command::new("docker");
    prove
        .args([
            "run",
            "--rm",
            "--network",
            "none",
            "-e",
            "RISC0_DEV_MODE=1",
            &image.0,
        ])
        .args([
            "prove",
            "--no-animation",
            "--message",
            P2PKH_MESSAGE,
            "--address",
            P2PKH_ADDRESS,
            "--signature",
            P2PKH_SIGNATURE,
            "--transaction",
            P2PKH_TRANSACTION,
            "--spv-proof",
            P2PKH_SPV_PROOF,
        ]);
    let prove_output = command_output(&mut prove, "docker prove");
    let prove_stdout = String::from_utf8(prove_output.stdout).expect("utf8 prove stdout");
    let receipt = prove_stdout
        .lines()
        .find(|line| line.starts_with("og-zkp1"))
        .expect("receipt line present starting with og-zkp1")
        .trim();

    let mut verify = Command::new("docker");
    verify
        .args([
            "run",
            "--rm",
            "--network",
            "none",
            "-e",
            "RISC0_DEV_MODE=1",
            &image.0,
        ])
        .args(["verify", receipt]);
    let verify_output = command_output(&mut verify, "docker verify");
    let verify_stdout = String::from_utf8(verify_output.stdout).expect("utf8 verify stdout");
    assert!(verify_stdout.contains("OG Status: October 2018"));
    assert!(verify_stdout.contains("Identity:  x.com/lukechilds"));
    assert!(verify_stdout.contains("Proof is valid"));
}

#[test]
fn prove_then_verify_p2pkh() {
    // Inputs
    let message = "og-zkp x.com/lukechilds";
    let signature =
        "HE6QfyPFmJvCGjWohZYAVa+pbdSRjeQpdNbXp6zNbDCnEN65xmK+WYidKlt6J1E/GDJpmLcatjEazVo5wqOg6wM=";
    let address = "1LukeQU5jwebXbMLDVydeH4vFSobRV9rkj";
    let transaction = "01000000000101b0fbbbccf54b14e37ef8709a4410bdd6db580fa7dc628106619e71220068ce630500000017160014b44aa221767d8b6d6e1300973719587b4a70ff78ffffffff02360f000000000000160014a6366f1659517ec2af9001e468f5defade31178510270000000000001976a914da6473ed373e08f46dd8003fca7ba72fbe9c555e88ac0247304402207fbb404bc1d79eaea59fd091f7df0c0cbfc367dec29b945be3f829ed742e01f60220295547047d5cf5ec27ce4bb25ba767cf560046e8c126795ed5e04d4810090fd2012103a66311e6776633e771690bdfa7bfea4a74cfb93b21005b704037671c710d0b9200000000";
    let spv_proof = "00000020c12df22f6c84f6927e63323d6762077ebc7b6d9e7a1020000000000000000000979f6cab6cd5cd47f6057bd1d49871f5a352b111f47a61c5ed8f9d371b6f8634a7d6c15b91c125176669d50d200b00000d71b32e7ebaecc4f44c637c224b5d54fc23fa37fb4faca57fd9603c5a21f373e20d91ab1288f566a5689d8ed3d2b17c94e8ce3320b03882cef7331309472f28cc3631028ba0a6ac08b7fd623dffb48e084bc587963426927cb7826ba6926ca7c22958835f6f94685952f3d9ff12ac91f6a3349dd000b29cf297220eefd4bf20e3c51eb470a7c94ccecca95db3695c110e9f89502ac4cdf64d6d1ddf0e48b20a819327bbe5c2da1adca6aacbfe378123dc77ae993dd7a9330dd43acf83b2ee6cde4d89f02d01864936146f6070f05fb8409789df8ab7414baadd5763f32ca3fdcb4fef6d7f3c1e5d0bea733b2fd644fa456cdf73f21eb7e8866a2721d79266e9e834282a6acaa3fe1d08aa11d5f0892dc7f9ff629f2efb8a7e190fb7e1e029b4cde8c0b00eea6730d9258aca5208fe5afcc5041e547b9a81f5d54f2ce9bec1e9e3b532622be369a572a783fbaaa2631dc86b7a0f1b412adeb57c51a42f5b8bd8436b31bc8defacd89a6030086fe0abbc4a8be51ed2280d448843b44eb1a28b145981fc4900f9709e02223b25f12baf7c664c15482b0818f538d4e4deb96a1cb3cd04bb550b00";

    // Expected outputs
    let expected_identity = "x.com/lukechilds";
    let expected_og_status = "October 2018";

    prove_then_verify(
        message,
        signature,
        address,
        transaction,
        spv_proof,
        expected_identity,
        expected_og_status,
    );
}

#[test]
fn prove_then_verify_p2sh_p2wpkh() {
    // Inputs
    let message = "og-zkp x.com/lukechilds";
    let signature =
        "IEfY9NgVNcWF863TzdsnMBrfC9WQ3S+/Fb29r217ofHZNVwWa/R+JpTluVRuQif+cOAcbcv1qpR2L57n8slmzQU=";
    let address = "3Luke2qRn5iLj4NiFrvLBu2jaEj7JeMR6w";
    let transaction = "0100000001c2360d0a190fde0962096ff2b05c5fafea40c1927852096671f2120f72fbb92c000000008a473044022037efa5e6ed145347836edac5cafc6f1c9485d430a608f11cc6f098f6236c3f8a0220070b5ef93b4ebe118d42ee8cb8c052511408943d45e76458f3292dd978a8fac60141040e3a759c33b03e1af8e5d86fb447a40eff244c847a4f8274276db490054e8be076f8801ddc9c5246ee86b6f33cfe38e8b7e57ab9db390eb3ec1ec6ae9eeea113fdffffff02102700000000000017a914d2d309d3d7ed9ee462e3643d7215951e0bd742c68721e61c00000000001976a914da6473ed373e08f46dd8003fca7ba72fbe9c555e88ac73a20800";
    let spv_proof = "00e000203c6f4a4b9f8b40cb0c588e48089f2891592940e5f1f80c000000000000000000337faa613c16a0e85b9c399ec74f2cf51aea696c75d4e4227c822606192b7fcd9a807f5c505b2e176f741a69fc0900000d8d7b6129a8817a41f48fc76441f05359b16ac90b89839dc833329a86a92aa4cfc6a2f0acbf38d29bf36e31b533b77331bd6a49be588ffef3a9709ede56e684e14ed0b643b8a4fe7247b0f9c0d884c5197615ea57cbb506fe56581eef9b92ad4a12b7f5e65fae7bd07fa4f46542d4c33c44b80b5b24b64508f3f8baff54fbe9dbb202eaa7a6aab448409199997f246be312dc45cee432024c8b90e0c13236853a98fb43716e04f6acf9277cc574df782923519b5569a346cfae554c7c9b7b427d066f770edf7870d9458c885898042767a622c43b7b72189c1a237905da96fe72d3d289d26300ee7682dc1aacecfdf7211dcdd8176a21e8f91246739a768a16466f5165ce732615fc004e5eecb7ea976d1c661e297564a0cd0fd7e460d165a5f75f3836c68ff6caf188ee259008942d012a238a4804205b5ea04ea8b4a7961fd53ac5c3ea63ded0c9770c74b393d1296eaddd19e6c2f74c040f812db5f0b219e2f0fa12af6b654a046f091d5f3da1b14858bfd960afdffe6d587096149cb706371f87e5430d0a14ab55cd53a63160e1b42eb28488b983783359c45a0abc276a6804bbde0100";

    // Expected outputs
    let expected_identity = "x.com/lukechilds";
    let expected_og_status = "March 2019";

    prove_then_verify(
        message,
        signature,
        address,
        transaction,
        spv_proof,
        expected_identity,
        expected_og_status,
    );
}

#[test]
fn prove_then_verify_p2wpkh() {
    // Inputs
    let message = "og-zkp x.com/lukechilds";
    let signature =
        "Hy4jk2Sa62j6xN0ud5QqJTlzXuM8sn/g+TevYcBwLiJcAWc/9+4B0ljQgTD+5hp1KMvROkW4k1MYGigbza1K+ZU=";
    let address = "bc1qlukeyq0c69v97uss68fet26kjkcsrymd2kv6d4";
    let transaction = "0100000001b202eaa7a6aab448409199997f246be312dc45cee432024c8b90e0c13236853a010000008a4730440220276fdcb55fe9f28975481f7612584d45feed0901758be48453fd1f6c139d573a022020e12c981bb5ce19f845b57440942df53b002e74c44d50a51a9ee030a1f1ea6e0141040e3a759c33b03e1af8e5d86fb447a40eff244c847a4f8274276db490054e8be076f8801ddc9c5246ee86b6f33cfe38e8b7e57ab9db390eb3ec1ec6ae9eeea113fdffffff021027000000000000160014ff2d9201f8d1585f7210d1d395ab5695b101936d649f1c00000000001976a914da6473ed373e08f46dd8003fca7ba72fbe9c555e88ac73a20800";
    let spv_proof = "00e000203c6f4a4b9f8b40cb0c588e48089f2891592940e5f1f80c000000000000000000337faa613c16a0e85b9c399ec74f2cf51aea696c75d4e4227c822606192b7fcd9a807f5c505b2e176f741a69fc0900000d8d7b6129a8817a41f48fc76441f05359b16ac90b89839dc833329a86a92aa4cfc6a2f0acbf38d29bf36e31b533b77331bd6a49be588ffef3a9709ede56e684e14ed0b643b8a4fe7247b0f9c0d884c5197615ea57cbb506fe56581eef9b92ad4a12b7f5e65fae7bd07fa4f46542d4c33c44b80b5b24b64508f3f8baff54fbe9dbb202eaa7a6aab448409199997f246be312dc45cee432024c8b90e0c13236853a98fb43716e04f6acf9277cc574df782923519b5569a346cfae554c7c9b7b427d066f770edf7870d9458c885898042767a622c43b7b72189c1a237905da96fe72d3d289d26300ee7682dc1aacecfdf7211dcdd8176a21e8f91246739a768a16466f5165ce732615fc004e5eecb7ea976d1c661e297564a0cd0fd7e460d165a5f75f3836c68ff6caf188ee259008942d012a238a4804205b5ea04ea8b4a7961fd53ac5c3ea63ded0c9770c74b393d1296eaddd19e6c2f74c040f812db5f0b219e2f0fa12af6b654a046f091d5f3da1b14858bfd960afdffe6d587096149cb706371f87e5430d0a14ab55cd53a63160e1b42eb28488b983783359c45a0abc276a6804bbde0200";

    // Expected outputs
    let expected_identity = "x.com/lukechilds";
    let expected_og_status = "March 2019";

    prove_then_verify(
        message,
        signature,
        address,
        transaction,
        spv_proof,
        expected_identity,
        expected_og_status,
    );
}

#[test]
fn prove_then_verify_p2tr() {
    // Inputs
    let message = "og-zkp x.com/lukechilds";
    let signature =
        "IOVIcHmzE1koFMStNWd7i3mBWsOjU8Xwy0vSFEk8WU96WPNlrcNlW9lB0Gcx+38MB9UQ1B5IqeYpCEauF8aVxpU=";
    let address = "bc1plukecxz2nfpecferrcmut2q3u8hyz97g8zg5fdaaskajjeewezsqwtjy53";
    let transaction = "01000000000101d528565e141e0e35e1878e2ea25d347196706dd3b75d85f8a8882f14775870280600000000ffffffff059104000000000000160014798bbdd9e1748f67c0204b0915a03d46c61ab2771413010000000000225120ff2d9c184a9a439c27231e37c5a811e1ee4117c8389144b7bd85bb29672ec8a0599e05000000000017a914fca7c972b1b29359db9496a9ae6f6b3c049d998b878849000000000000160014b12d73d1ba48a7df0571d9de2a546db6f56bb313837d4e5401000000160014eed12e1ba4703f0a6afe1a3dd4a3b1de86803936024730440220553033d4d5019ccfec66cf3fb94bd00fe26f381ecdaa0f2128e79029a2f3cc60022019e044e808e4aa9d415b9f3f161e286719dbab69155ce4bbb5be523c82b995f5012103a876001a753b82a8bec93c5936e65e0bacacebe5de6ca74adadfced8ebb7ad5300000000";
    let spv_proof = "00e0c025288cb81df55db460b088a12e67a7a98387d0202c000200000000000000000000e2fe773809008d1ccf052eebb19702b50ce7241bc3ed6e5cf194c55e66f203f7d7b8c367b18b021774e840e85f0700000c48d85971a8ac5b79d5e5ab0179b532255b0769f2c5312f9a8279dea7479fc6835b0bb036fc0de9ee28db50738dd93026919abeceb9468ff77e6f109d64f09e4fc2578ff6dd8ccae1ace558dab0ef3cc6b5259aa01ef5ad39ca039cd42e68bc53c6d86c1b5f3bbc20d57fd4cf2f52c82e4d7686f7fd8ca3b5699e70d0417a480da26153159366d81d43a39329945f0eff2852c0c905a6ebb80ee5425cc2196b96e3e95bb76f5c97b952752267a9cf479a4ddab471d5dcfc045ca0396919aa47b0a326d6f47819d27216c4fb6329835bda507e1443cd0812293bd89ef3c45913d36ef6e0b53f18d9b1a59289556a3259bdd596c1e1174231e2e7192cf51a11830412f3b3f366881e0086a686ce86edcc20fa0c44c88b1a9164403bf543bb8a55259d994e842b221b581b1f369fac48d46a77d589874f4b9b9123d65eb042d3b9a92a82a3861db7974c415ef6f6b0f3185ff0ac780e86112b0eb7a362e7c427583c75a9157af5b988b34961853dcdacbab61003f247e1dccea54c61df3eb935b5f803ab5b05";

    // Expected outputs
    let expected_identity = "x.com/lukechilds";
    let expected_og_status = "March 2025";

    prove_then_verify(
        message,
        signature,
        address,
        transaction,
        spv_proof,
        expected_identity,
        expected_og_status,
    );
}

// P2PKH fixture data shared by JSON tests
const P2PKH_MESSAGE: &str = "og-zkp x.com/lukechilds";
const P2PKH_SIGNATURE: &str =
    "HE6QfyPFmJvCGjWohZYAVa+pbdSRjeQpdNbXp6zNbDCnEN65xmK+WYidKlt6J1E/GDJpmLcatjEazVo5wqOg6wM=";
const P2PKH_ADDRESS: &str = "1LukeQU5jwebXbMLDVydeH4vFSobRV9rkj";
const P2PKH_TRANSACTION: &str = "01000000000101b0fbbbccf54b14e37ef8709a4410bdd6db580fa7dc628106619e71220068ce630500000017160014b44aa221767d8b6d6e1300973719587b4a70ff78ffffffff02360f000000000000160014a6366f1659517ec2af9001e468f5defade31178510270000000000001976a914da6473ed373e08f46dd8003fca7ba72fbe9c555e88ac0247304402207fbb404bc1d79eaea59fd091f7df0c0cbfc367dec29b945be3f829ed742e01f60220295547047d5cf5ec27ce4bb25ba767cf560046e8c126795ed5e04d4810090fd2012103a66311e6776633e771690bdfa7bfea4a74cfb93b21005b704037671c710d0b9200000000";
const P2PKH_SPV_PROOF: &str = "00000020c12df22f6c84f6927e63323d6762077ebc7b6d9e7a1020000000000000000000979f6cab6cd5cd47f6057bd1d49871f5a352b111f47a61c5ed8f9d371b6f8634a7d6c15b91c125176669d50d200b00000d71b32e7ebaecc4f44c637c224b5d54fc23fa37fb4faca57fd9603c5a21f373e20d91ab1288f566a5689d8ed3d2b17c94e8ce3320b03882cef7331309472f28cc3631028ba0a6ac08b7fd623dffb48e084bc587963426927cb7826ba6926ca7c22958835f6f94685952f3d9ff12ac91f6a3349dd000b29cf297220eefd4bf20e3c51eb470a7c94ccecca95db3695c110e9f89502ac4cdf64d6d1ddf0e48b20a819327bbe5c2da1adca6aacbfe378123dc77ae993dd7a9330dd43acf83b2ee6cde4d89f02d01864936146f6070f05fb8409789df8ab7414baadd5763f32ca3fdcb4fef6d7f3c1e5d0bea733b2fd644fa456cdf73f21eb7e8866a2721d79266e9e834282a6acaa3fe1d08aa11d5f0892dc7f9ff629f2efb8a7e190fb7e1e029b4cde8c0b00eea6730d9258aca5208fe5afcc5041e547b9a81f5d54f2ce9bec1e9e3b532622be369a572a783fbaaa2631dc86b7a0f1b412adeb57c51a42f5b8bd8436b31bc8defacd89a6030086fe0abbc4a8be51ed2280d448843b44eb1a28b145981fc4900f9709e02223b25f12baf7c664c15482b0818f538d4e4deb96a1cb3cd04bb550b00";

fn prove_with_json() -> Value {
    let output = Command::cargo_bin("og-zkp")
        .unwrap()
        .env("RISC0_DEV_MODE", "1")
        .args([
            "prove",
            "--json",
            "--message",
            P2PKH_MESSAGE,
            "--signature",
            P2PKH_SIGNATURE,
            "--address",
            P2PKH_ADDRESS,
            "--transaction",
            P2PKH_TRANSACTION,
            "--spv-proof",
            P2PKH_SPV_PROOF,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("stdout is valid JSON")
}

#[test]
fn prove_json_output() {
    let json = prove_with_json();
    assert!(json["block_inclusion_root"].is_string());
    assert_eq!(json["block_month"], "1538352000");
    assert_eq!(json["identity"], "x.com/lukechilds");
    assert!(json["proof"].as_str().unwrap().starts_with("og-zkp1"));
}

#[test]
fn verify_json_output() {
    let prove_json = prove_with_json();
    let receipt = prove_json["proof"].as_str().unwrap();

    let verify_output = Command::cargo_bin("og-zkp")
        .unwrap()
        .env("RISC0_DEV_MODE", "1")
        .args(["verify", "--json", receipt])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&verify_output).expect("stdout is valid JSON");
    assert!(json["block_inclusion_root"].is_string());
    assert_eq!(json["block_month"], "1538352000");
    assert_eq!(json["identity"], "x.com/lukechilds");
    assert!(
        json.get("proof").is_none(),
        "verify should not include proof field"
    );
}

const EXPECTED_IMAGE_ID: &str = include_str!("../expected-image-id");
const EXPECTED_INCLUSION_ROOT: &str =
    "645193f7e45302f503f14d6bdc593a12ee954b5ca844d38affaae51febb77a3e";
const EXPECTED_INCLUSION_TIP: &str =
    "00000000000000000000f97a2e344937318c96249ea6b204b3f1996796bcc72b";
const EXPECTED_INCLUSION_HEIGHT: u64 = 936729;

#[test]
fn info_text_output() {
    Command::cargo_bin("og-zkp")
        .unwrap()
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Version:                  {}",
            env!("CARGO_PKG_VERSION")
        )))
        .stdout(predicate::str::contains(format!(
            "Image ID:                 {EXPECTED_IMAGE_ID}"
        )))
        .stdout(predicate::str::contains(format!(
            "Block inclusion root:     {EXPECTED_INCLUSION_ROOT}"
        )))
        .stdout(predicate::str::contains(format!(
            "Block inclusion tip:      {EXPECTED_INCLUSION_TIP}"
        )))
        .stdout(predicate::str::contains(format!(
            "Block inclusion height:   {EXPECTED_INCLUSION_HEIGHT}"
        )));
}

#[test]
fn info_json_output() {
    let output = Command::cargo_bin("og-zkp")
        .unwrap()
        .args(["info", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("stdout is valid JSON");
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(json["image_id"], EXPECTED_IMAGE_ID);
    assert_eq!(json["block_inclusion_root"], EXPECTED_INCLUSION_ROOT);
    assert_eq!(json["block_inclusion_tip"], EXPECTED_INCLUSION_TIP);
    assert_eq!(json["block_inclusion_height"], EXPECTED_INCLUSION_HEIGHT);
}
