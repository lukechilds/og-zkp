use risc0_zkvm::guest::env;

fn main() {
    let input: og_zkp_core::guest::Input = env::read();
    let output = og_zkp_core::guest::run(input);
    env::commit(&output);
}
