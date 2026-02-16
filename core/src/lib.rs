mod address_kind;
pub mod block_inclusion_proof;
pub mod guest;

#[cfg(feature = "host")]
pub mod animation;
#[cfg(feature = "host")]
pub mod mempool_api;
#[cfg(feature = "host")]
pub mod prove;
#[cfg(feature = "host")]
pub mod receipt;
#[cfg(feature = "host")]
pub mod verify;

pub use address_kind::AddressKind;

#[cfg(feature = "host")]
pub mod info;

// For some reason adding new members or comments in the middle of this file breaks the program hash.
// Likely due to some kind of line number debug info making it into the risc0 guest binary.
// Adding new host only members at the end of this file does not break the guest program hash.
