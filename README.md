# og-zkp

> Prove your OG status in zero-knowledge!

og-zkp is a zero-knowledge proof system to prove the calendar month you received your first Bitcoin transaction without revealing the address that you received to or the specific block the transaction was included in.

The og-zkp prover outputs a Groth16 SNARK comitting to a `month` (timestamp representing a calender month time period) and a `user identity` (any arbitrary string). A valid proof can only be produced by someone who can input a valid signed message of the `user identity` by a pubkey along with an SPV proof proving the same pubkey received Bitcoin within that `month` time period.
