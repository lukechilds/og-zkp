use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// End-to-end tests that intentionally omit transaction/SPV inputs so the CLI
// fetches them from mempool.space. Uses RISC0_DEV_MODE to keep proving fast.

fn prove_then_verify_lookup(
    message: &str,
    signature: &str,
    address: &str,
    expected_identity: &str,
    expected_og_status: &str,
) {
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

    let prove_output = prove_cmd.assert().success().get_output().stdout.clone();
    let stdout_str = String::from_utf8(prove_output).expect("utf8 stdout");
    let receipt = stdout_str
        .lines()
        .find(|line| line.starts_with("og-zkp1"))
        .expect("receipt line present starting with og-zkp1")
        .trim()
        .to_string();

    Command::cargo_bin("og-zkp")
        .expect("binary exists")
        .env("RISC0_DEV_MODE", "1")
        .arg("verify")
        .arg(&receipt)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Identity:    {expected_identity}"
        )))
        .stdout(predicate::str::contains(format!(
            "OG Status:   {expected_og_status}"
        )));
}

#[test]
fn prove_then_verify_p2pkh_lookup() {
    prove_then_verify_lookup(
        "og-zkp x.com/lukechilds",
        "HE6QfyPFmJvCGjWohZYAVa+pbdSRjeQpdNbXp6zNbDCnEN65xmK+WYidKlt6J1E/GDJpmLcatjEazVo5wqOg6wM=",
        "1LukeQU5jwebXbMLDVydeH4vFSobRV9rkj",
        "x.com/lukechilds",
        "October 2018",
    );
}

#[test]
fn prove_then_verify_p2sh_p2wpkh_lookup() {
    prove_then_verify_lookup(
        "og-zkp x.com/lukechilds",
        "IEfY9NgVNcWF863TzdsnMBrfC9WQ3S+/Fb29r217ofHZNVwWa/R+JpTluVRuQif+cOAcbcv1qpR2L57n8slmzQU=",
        "3Luke2qRn5iLj4NiFrvLBu2jaEj7JeMR6w",
        "x.com/lukechilds",
        "March 2019",
    );
}

#[test]
fn prove_then_verify_p2wpkh_lookup() {
    prove_then_verify_lookup(
        "og-zkp x.com/lukechilds",
        "Hy4jk2Sa62j6xN0ud5QqJTlzXuM8sn/g+TevYcBwLiJcAWc/9+4B0ljQgTD+5hp1KMvROkW4k1MYGigbza1K+ZU=",
        "bc1qlukeyq0c69v97uss68fet26kjkcsrymd2kv6d4",
        "x.com/lukechilds",
        "March 2019",
    );
}

#[test]
fn prove_then_verify_p2tr_lookup() {
    prove_then_verify_lookup(
        "og-zkp x.com/lukechilds",
        "IOVIcHmzE1koFMStNWd7i3mBWsOjU8Xwy0vSFEk8WU96WPNlrcNlW9lB0Gcx+38MB9UQ1B5IqeYpCEauF8aVxpU=",
        "bc1plukecxz2nfpecferrcmut2q3u8hyz97g8zg5fdaaskajjeewezsqwtjy53",
        "x.com/lukechilds",
        "March 2025",
    );
}
