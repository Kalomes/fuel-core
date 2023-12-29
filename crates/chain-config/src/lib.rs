//#![deny(clippy::cast_possible_truncation)]
//#![deny(clippy::arithmetic_side_effects)]
//#![deny(unused_crate_dependencies)]
//#![deny(warnings)]

pub mod config;
pub mod fee_collection_contract;
mod genesis;
mod serialization;

pub use config::*;
use fuel_core_types::fuel_vm::SecretKey;
pub use genesis::GenesisCommitment;

/// A default secret key to use for testing purposes only
pub fn default_consensus_dev_key() -> SecretKey {
    // Derived from:
    //  - Mnemonic phrase: "winner alley monkey elephant sun off boil hope toward boss bronze dish"
    //  - Path: "m/44'/60'/0'/0/0"
    // Equivalent to:
    //  `SecretKey::new_from_mnemonic_phrase_with_path(..)`
    let bytes: [u8; 32] = [
        0xfb, 0xe4, 0x91, 0x78, 0xda, 0xc2, 0xdf, 0x5f, 0xde, 0xa7, 0x4a, 0x11, 0xa9,
        0x0f, 0x99, 0x77, 0x62, 0x5f, 0xe0, 0x23, 0xcd, 0xf6, 0x41, 0x4b, 0xfd, 0x63,
        0x9d, 0x32, 0x7a, 0x2e, 0x9d, 0xdb,
    ];
    SecretKey::try_from(bytes.as_slice()).expect("valid key")
}

#[cfg(test)]
mod tests {
    // These imports are needed to silence unused_crate_dependencies since optional dev
    // dependencies are still not stable in cargo.
    use bytes as _;
    use pretty_assertions as _;
    use strum as _;
    use tempfile as _;
}
