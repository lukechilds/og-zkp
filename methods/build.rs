use std::collections::HashMap;
use std::io::Write;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = std::path::PathBuf::from(manifest_dir).join("..");

    if !std::env::var("RISC0_SKIP_BUILD")
        .unwrap_or_default()
        .is_empty()
    {
        // Two-step build: the guest ELF was already built (e.g. by `cargo build -p methods
        // --release`). Find the pre-built .bin and embed it directly.
        //
        // This works around a risc0-build limitation where RISC0_SKIP_BUILD=1 embeds an
        // empty &[] instead of the pre-built binary.
        let target_dir = std::env::var("CARGO_TARGET_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| root.join("target"));
        let bin_path = target_dir
            .join("riscv-guest/methods/og-zkp/riscv32im-risc0-zkvm-elf/docker/og-zkp.bin");

        if !bin_path.exists() {
            panic!(
                "RISC0_SKIP_BUILD is set but pre-built guest binary not found at: {}. \
                 Run `cargo build -p methods --release` first.",
                bin_path.display()
            );
        }

        let elf_contents = std::fs::read(&bin_path).unwrap();
        let image_id = risc0_binfmt::compute_image_id(&elf_contents).unwrap();

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let methods_path = std::path::PathBuf::from(&out_dir).join("methods.rs");
        let mut f = std::fs::File::create(&methods_path).unwrap();
        writeln!(
            f,
            "pub const OG_ZKP_ELF: &[u8] = include_bytes!({:?});",
            bin_path.to_str().unwrap()
        )
        .unwrap();
        writeln!(
            f,
            "pub const OG_ZKP_PATH: &str = {:?};",
            bin_path.to_str().unwrap()
        )
        .unwrap();
        writeln!(
            f,
            "pub const OG_ZKP_ID: [u32; 8] = {:?};",
            image_id.as_words()
        )
        .unwrap();
        println!("cargo:rerun-if-changed={}", methods_path.display());
    } else if !std::env::var("RISC0_USE_DOCKER")
        .unwrap_or_default()
        .is_empty()
    {
        let docker_opts = risc0_build::DockerOptionsBuilder::default()
            .root_dir(root)
            .build()
            .unwrap();
        let guest_opts = risc0_build::GuestOptionsBuilder::default()
            .use_docker(docker_opts)
            .build()
            .unwrap();

        let mut opts = HashMap::new();
        opts.insert("og-zkp", guest_opts);
        risc0_build::embed_methods_with_options(opts);
    } else {
        risc0_build::embed_methods();
    }
}
