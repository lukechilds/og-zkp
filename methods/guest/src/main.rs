use risc0_zkvm::guest::env;

fn main() {
    let input: ogzkp_core::guest::Input = env::read();
    let output = ogzkp_core::guest::run(input);
    env::commit(&output);
}
