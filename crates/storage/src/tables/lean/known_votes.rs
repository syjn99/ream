use std::{collections::HashMap, sync::Arc};

use alloy_primitives::B256;
use ream_consensus_lean::{block::SignedBlock, vote::SignedVote};
use redb::{Database, Durability, ReadableTable, ReadableTableMetadata, TableDefinition};

use crate::{
    errors::StoreError,
    tables::{ssz_encoder::SSZEncoding, table::Table},
};

/// Table definition for the Known Votes table
///
/// Key: index (u64, acts like position in an append-only array)
/// Value: [SignedVote]
pub(crate) const KNOWN_VOTES_TABLE: TableDefinition<u64, SSZEncoding<SignedVote>> =
    TableDefinition::new("known_votes");

pub struct KnownVotesTable {
    pub db: Arc<Database>,
}

impl Table for KnownVotesTable {
    type Key = u64;

    type Value = SignedVote;

    fn get(&self, key: Self::Key) -> Result<Option<Self::Value>, StoreError> {
        let read_txn = self.db.begin_read()?;

        let table = read_txn.open_table(KNOWN_VOTES_TABLE)?;
        let result = table.get(key)?;
        Ok(result.map(|res| res.value()))
    }

    fn insert(&self, key: Self::Key, value: Self::Value) -> Result<(), StoreError> {
        let mut write_txn = self.db.begin_write()?;
        write_txn.set_durability(Durability::Immediate);
        let mut table = write_txn.open_table(KNOWN_VOTES_TABLE)?;
        table.insert(key, value)?;
        drop(table);
        write_txn.commit()?;
        Ok(())
    }
}

impl KnownVotesTable {
    /// Append multiple votes in a single transaction.
    pub fn batch_append(
        &self,
        values: impl IntoIterator<Item = (u64, SignedVote)>,
    ) -> Result<(), StoreError> {
        let mut write_txn = self.db.begin_write()?;
        write_txn.set_durability(Durability::Immediate);

        let mut table = write_txn.open_table(KNOWN_VOTES_TABLE)?;

        for (validator_id, signed_vote) in values {
            table.insert(validator_id, signed_vote)?;
        }

        drop(table);
        write_txn.commit()?;

        Ok(())
    }

    /// Check if a given vote exists in the append-only array.
    pub fn contains(&self, value: &SignedVote) -> Result<bool, StoreError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(KNOWN_VOTES_TABLE)?;

        for entry in table.iter()? {
            let (_, v) = entry?;
            if &v.value() == value {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Return number of votes (like `Vec::len`)
    pub fn len(&self) -> Result<u64, StoreError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(KNOWN_VOTES_TABLE)?;
        Ok(table.len()?)
    }

    /// Returns if there are no known votes
    pub fn is_empty(&self) -> Result<bool, StoreError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(KNOWN_VOTES_TABLE)?;
        Ok(table.len()? == 0)
    }

    /// Get all votes.
    pub fn get_all_votes(&self) -> Result<HashMap<u64, SignedVote>, StoreError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(KNOWN_VOTES_TABLE)?;

        let mut votes = HashMap::with_capacity(table.len()? as usize);

        for entry in table.iter()? {
            let (k, v) = entry?;
            votes.insert(k.value(), v.value());
        }

        Ok(votes)
    }

    /// Get all votes whose `source.root` matches `state.latest_justified.root`
    /// and that are not already in the block's attestations.
    pub fn filter_new_votes_to_add(
        &self,
        justified_root: B256,
        new_block: &SignedBlock,
    ) -> Result<Vec<SignedVote>, StoreError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(KNOWN_VOTES_TABLE)?;

        let mut result = Vec::new();

        for entry in table.iter()? {
            let (_, v) = entry?;
            let vote = v.value();

            if vote.message.source.root == justified_root
                && !new_block.message.body.attestations.contains(&vote)
            {
                result.push(vote);
            }
        }

        Ok(result)
    }
}
