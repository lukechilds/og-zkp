use std::collections::HashMap;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = std::path::PathBuf::from(manifest_dir).join("..");

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
}
