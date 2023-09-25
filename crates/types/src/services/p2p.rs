//! Contains types related to P2P data

use crate::{
    fuel_tx::Transaction,
    fuel_types::BlockHeight,
};
use std::fmt::Debug;

/// Contains types and logic for Peer Reputation
pub mod peer_reputation;

/// List of transactions
pub type Transactions = Vec<Transaction>;

/// Lightweight representation of gossipped data that only includes IDs
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GossipsubMessageInfo {
    /// The message id that corresponds to a message payload (typically a unique hash)
    pub message_id: Vec<u8>,
    /// The ID of the network peer that sent this message
    pub peer_id: PeerId,
}

// TODO: Maybe we can remove most of types from here directly into P2P

/// Reporting levels on the status of a message received via Gossip
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum GossipsubMessageAcceptance {
    /// Report whether the gossiped message is valid and safe to rebroadcast
    Accept,
    /// Ignore the received message and prevent further gossiping
    Reject,
    /// Punish the gossip sender for providing invalid
    /// (or malicious) data and prevent further gossiping
    Ignore,
}

/// A gossipped message from the network containing all relevant data.
#[derive(Debug, Clone)]
pub struct GossipData<T> {
    /// The gossipped message payload
    /// This is meant to be consumed once to avoid cloning. Subsequent attempts to fetch data from
    /// the message should return None.
    pub data: Option<T>,
    /// The ID of the network peer that sent this message
    pub peer_id: PeerId,
    /// The message id that corresponds to a message payload (typically a unique hash)
    pub message_id: Vec<u8>,
}

/// Transactions gossiped by peers for inclusion into a block
pub type TransactionGossipData = GossipData<Transaction>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The source of some network data.
pub struct SourcePeer<T> {
    /// The source of the data.
    pub peer_id: PeerId,
    /// The data.
    pub data: T,
}

impl<T> SourcePeer<T> {
    /// Maps a `SourcePeer<T>` to `SourcePeer<U>` by applying a function to the
    /// contained data. The internal `peer_id` is maintained.
    pub fn map<F, U>(self, mut f: F) -> SourcePeer<U>
    where
        F: FnMut(T) -> U,
    {
        let peer_id = self.peer_id;
        let data = f(self.data);
        SourcePeer { peer_id, data }
    }

    /// Asref
    pub fn as_ref(&self) -> SourcePeer<&T> {
        SourcePeer {
            peer_id: self.peer_id.clone(),
            data: &self.data,
        }
    }
}

impl<T> GossipData<T> {
    /// Construct a new gossip message
    pub fn new(
        data: T,
        peer_id: impl Into<Vec<u8>>,
        message_id: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            data: Some(data),
            peer_id: PeerId::from(peer_id.into()),
            message_id: message_id.into(),
        }
    }
}

/// A generic representation of data that's been gossipped by the network
pub trait NetworkData<T>: Debug + Send {
    /// Consume ownership of data from a gossipped message
    fn take_data(&mut self) -> Option<T>;
}

impl<T: Debug + Send + 'static> NetworkData<T> for GossipData<T> {
    fn take_data(&mut self) -> Option<T> {
        self.data.take()
    }
}
/// Used for relying latest `BlockHeight` info from connected peers
#[derive(Debug, Clone)]
pub struct BlockHeightHeartbeatData {
    /// PeerId as bytes
    pub peer_id: PeerId,
    /// Latest BlockHeight received
    pub block_height: BlockHeight,
}

/// Opaque peer identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PeerId(Vec<u8>);

impl AsRef<[u8]> for PeerId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for PeerId {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<PeerId> for Vec<u8> {
    fn from(peer_id: PeerId) -> Self {
        peer_id.0
    }
}

impl PeerId {
    /// Bind the PeerId and given data of type T together to generate a
    /// SourcePeer<T>
    pub fn bind<T>(self, data: T) -> SourcePeer<T> {
        SourcePeer {
            peer_id: self,
            data,
        }
    }
}
