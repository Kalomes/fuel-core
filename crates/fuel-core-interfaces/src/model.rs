mod block;
mod block_height;
mod coin;
mod da_block_height;
mod messages;
mod txpool;
mod vote;

use crate::common::fuel_crypto::SecretKey;
pub use block::{
    BlockId,
    ConsensusType,
    Empty,
    FuelApplicationHeader,
    FuelBlock,
    FuelBlockConsensus,
    FuelBlockDb,
    FuelBlockHeader,
    FuelBlockPoAConsensus,
    FuelConsensusHeader,
    Genesis,
    PartialFuelBlock,
    PartialFuelBlockHeader,
    SealedFuelBlock,
    SealedFuelBlockHeader,
};
pub use block_height::BlockHeight;
pub use coin::{
    Coin,
    CoinStatus,
};
pub use da_block_height::DaBlockHeight;
use derive_more::{
    Deref,
    From,
};
pub use messages::*;
use secrecy::{
    zeroize,
    CloneableSecret,
    DebugSecret,
};
pub use txpool::{
    ArcPoolTx,
    TxInfo,
};
pub use vote::ConsensusVote;
use zeroize::Zeroize;

/// Wrapper around [`fuel_crypto::SecretKey`] to implement [`secrecy`] marker traits
#[derive(
    Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroize, Deref, From,
)]
#[repr(transparent)]
pub struct SecretKeyWrapper(SecretKey);

impl CloneableSecret for SecretKeyWrapper {}
impl DebugSecret for SecretKeyWrapper {}