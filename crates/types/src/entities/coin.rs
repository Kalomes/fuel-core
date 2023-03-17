//! Coin

use crate::{
    blockchain::primitives::BlockHeight,
    fuel_asm::Word,
    fuel_tx::{
        TxPointer,
        UtxoId,
    },
    fuel_types::{
        Address,
        AssetId,
    },
};

/// Represents the user's coin for some asset with `asset_id`.
/// The `Coin` is either `CoinStatus::Spent` or `CoinStatus::Unspent`. If the coin is unspent,
/// it can be used as an input to the transaction and can be spent up to the `amount`.
/// After usage as an input of a transaction, the `Coin` becomes `CoinStatus::Spent`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Coin {
    /// The coin utxo id.
    pub utxo_id: UtxoId,
    /// The address with permission to spend this coin
    pub owner: Address,
    /// Amount of coins
    pub amount: Word,
    /// Different incompatible coins can coexist with different asset ids.
    /// This is the "color" of the coin.
    pub asset_id: AssetId,
    /// This coin cannot be spent until the given height
    pub maturity: BlockHeight,
    /// Indexes the block and transaction this coin originated from
    pub tx_pointer: TxPointer,
}

impl Coin {
    /// Compress the coin to minimize the serialized size.
    pub fn compress(self) -> CompressedCoin {
        CompressedCoin {
            owner: self.owner,
            amount: self.amount,
            asset_id: self.asset_id,
            maturity: self.maturity,
            tx_pointer: self.tx_pointer,
        }
    }
}

/// The compressed version of the `Coin` with minimum fields required for
/// the proper work of the blockchain.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct CompressedCoin {
    /// The address with permission to spend this coin
    pub owner: Address,
    /// Amount of coins
    pub amount: Word,
    /// Different incompatible coins can coexist with different asset ids.
    /// This is the "color" of the coin.
    pub asset_id: AssetId,
    /// This coin cannot be spent until the given height
    pub maturity: BlockHeight,
    /// Indexes the block and transaction this coin originated from
    pub tx_pointer: TxPointer,
}

impl CompressedCoin {
    /// Uncompress the coin.
    pub fn uncompress(self, utxo_id: UtxoId) -> Coin {
        Coin {
            utxo_id,
            owner: self.owner,
            amount: self.amount,
            asset_id: self.asset_id,
            maturity: self.maturity,
            tx_pointer: self.tx_pointer,
        }
    }
}