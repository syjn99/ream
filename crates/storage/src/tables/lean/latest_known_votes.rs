use std::{collections::HashMap, sync::Arc};

use alloy_primitives::B256;
use ream_consensus_lean::{block::SignedBlock, vote::SignedVote};
use redb::{Database, Durability, ReadableTable, TableDefinition};

use crate::{errors::StoreError, tables::ssz_encoder::SSZEncoding};

/// Table definition for the Latest Known Votes table
///
/// Key: u64 (validator index)
/// Value: [SignedVote]
pub(crate) const LATEST_KNOWN_VOTES_TABLE: TableDefinition<u64, SSZEncoding<SignedVote>> =
    TableDefinition::new("latest_known_votes");

pub struct LatestKnownVotesTable {
    pub db: Arc<Database>,
}

impl LatestKnownVotesTable {
    /// Insert multiple votes with validator id in a single transaction.
    pub fn batch_insert(
        &self,
        values: impl IntoIterator<Item = (u64, SignedVote)>,
    ) -> Result<(), StoreError> {
        let mut write_txn = self.db.begin_write()?;
        write_txn.set_durability(Durability::Immediate);

        let mut table = write_txn.open_table(LATEST_KNOWN_VOTES_TABLE)?;

        for (key, value) in values {
            table.insert(key, value)?;
        }

        drop(table);
        write_txn.commit()?;

        Ok(())
    }

    /// Check if a given vote exists in the append-only array.
    pub fn contains(&self, value: &SignedVote) -> Result<bool, StoreError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(LATEST_KNOWN_VOTES_TABLE)?;

        for entry in table.iter()? {
            let (_, v) = entry?;
            if &v.value() == value {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get all votes.
    pub fn get_all_votes(&self) -> Result<HashMap<u64, SignedVote>, StoreError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(LATEST_KNOWN_VOTES_TABLE)?;

        table
            .iter()?
            .map(|entry| {
                let (k, v) = entry?;
                Ok((k.value(), v.value()))
            })
            .collect()
    }

    /// Get all votes whose `source.root` matches `state.latest_justified.root`
    /// and that are not already in the block's attestations.
    pub fn filter_new_votes_to_add(
        &self,
        justified_root: B256,
        new_block: &SignedBlock,
    ) -> Result<Vec<SignedVote>, StoreError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(LATEST_KNOWN_VOTES_TABLE)?;

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
