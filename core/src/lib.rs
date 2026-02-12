mod address_kind;
pub mod block_inclusion_proof;
pub mod guest;

#[cfg(feature = "host")]
pub mod mempool_api;
#[cfg(feature = "host")]
pub mod prove;
#[cfg(feature = "host")]
pub mod receipt;
#[cfg(feature = "host")]
pub mod verify;

pub use address_kind::AddressKind;
