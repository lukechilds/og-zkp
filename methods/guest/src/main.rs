use risc0_zkvm::guest::env;

fn main() {
    // read the input
    let input: u32 = env::read();

    // TODO: assert signature is valid for message and pubkey

    // TODO: assert derived address from pubkey is included at expected tx output index

    // TODO: assert tx inclusion proof is valid for block header

    // TODO: assert block inclusion proof is valid for header merkle root

    // TODO: commit time from block header (rounded down to some interval)

    // write public output to the journal
    env::commit(&input);
}
