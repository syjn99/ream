//! https://ethereum.github.io/consensus-specs/ssz/merkle-proofs/#merkle-multiproofs

use std::collections::{BTreeMap, HashMap};

use alloy_primitives::B256;
use anyhow::ensure;

use crate::helper::{
    get_generalized_index, get_generalized_index_parent, get_generalized_index_sibling,
    get_helper_indices, hash,
};

/// ``Index`` is the index of a leaf in the **bottom** layer of the ``tree``.
type Index = u64;

/// ``GeneralizedIndex`` is the index of a node in the ``tree``.
type GeneralizedIndex = u64;

/// Multiproof is a structure that contains the leaves to be verified with
/// their corresponding proofs.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Multiproof {
    /// The leaves to be verified.
    /// Keyed by their generalized indices.
    pub leaves: HashMap<GeneralizedIndex, B256>,
    /// The proof nodes.
    /// Keyed by their generalized indices.
    /// Keys of ``proofs`` will be sorted in descending order when generating a single proof.
    pub proofs: BTreeMap<GeneralizedIndex, B256>,
}

impl Multiproof {
    /// Generate a multiproof for the given tree and indices.
    pub fn generate<const DEPTH: u64>(tree: &[B256], indices: &[Index]) -> anyhow::Result<Self> {
        let bottom_length: u64 = 1 << DEPTH;

        ensure!(!indices.is_empty(), "Indices cannot be empty");
        for &index in indices {
            ensure!(index < bottom_length, "Index out of bounds");
        }

        let generalized_indices: Vec<GeneralizedIndex> = indices
            .iter()
            .map(|&index| get_generalized_index(index, DEPTH))
            .collect();
        let helper_indices: Vec<GeneralizedIndex> = get_helper_indices(&generalized_indices);

        let leaves: HashMap<GeneralizedIndex, B256> = generalized_indices
            .iter()
            .map(|&g| (g, tree[g as usize]))
            .collect::<HashMap<_, _>>();
        let proofs: BTreeMap<GeneralizedIndex, B256> = helper_indices
            .iter()
            .map(|&g| (g, tree[g as usize]))
            .collect();

        Ok(Self { leaves, proofs })
    }

    /// Return the root of the multiproof.
    ///
    /// Most code in this function is borrowed from ssz_rs crate.
    /// https://github.com/ralexstokes/ssz-rs/blob/main/ssz-rs/src/merkleization/multiproofs.rs
    pub fn calculate_root(&self) -> anyhow::Result<B256> {
        let leaves_indices = self.leaves.keys().cloned().collect::<Vec<_>>();
        let helper_indices = get_helper_indices(&leaves_indices);

        ensure!(
            self.proofs.len() == helper_indices.len(),
            "Invalid proof: proof and helper indices length mismatch"
        );

        let mut objects = HashMap::new();

        for (index, node) in self.leaves.iter().chain(self.proofs.iter()) {
            objects.insert(*index, *node);
        }

        let mut keys = objects.keys().cloned().collect::<Vec<_>>();
        keys.sort_by(|a, b| b.cmp(a));

        let mut pos = 0;
        while pos < keys.len() {
            let key = keys.get(pos).unwrap();
            let key_present = objects.contains_key(key);
            let sibling_present = objects.contains_key(&get_generalized_index_sibling(*key));
            let parent_index = get_generalized_index_parent(*key);
            let parent_missing = !objects.contains_key(&parent_index);
            let should_compute = key_present && sibling_present && parent_missing;
            if should_compute {
                let right_index = key | 1;
                let left_index = get_generalized_index_sibling(right_index);
                let left_input = objects.get(&left_index).expect("contains index");
                let right_input = objects.get(&right_index).expect("contains index");

                let value = hash(left_input.as_slice(), right_input.as_slice());

                let parent = objects.entry(parent_index).or_default();
                parent.copy_from_slice(value.as_slice());
                keys.push(parent_index);
            }
            pos += 1;
        }

        let root = *objects.get(&1).expect("contains index");
        Ok(root)
    }

    /// Verify the multiproof against the given root.
    pub fn verify(&self, root: B256) -> anyhow::Result<()> {
        if self.calculate_root()? == root {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid proof"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle_tree;

    const DEPTH: u64 = 3;

    #[test]
    fn test_generate_multiproof() {
        let leaves = vec![
            B256::from_slice(&[0xAA; 32]),
            B256::from_slice(&[0xBB; 32]),
            B256::from_slice(&[0xCC; 32]),
            B256::from_slice(&[0xDD; 32]),
            B256::from_slice(&[0xEE; 32]),
            B256::from_slice(&[0xFF; 32]),
            B256::from_slice(&[0x11; 32]),
            B256::from_slice(&[0x22; 32]),
        ];

        let depth = (leaves.len() as f64).log2().ceil() as u64;
        let tree = merkle_tree(&leaves, depth).unwrap();

        let target_indices = vec![0, 5];
        let multiproof = Multiproof::generate::<DEPTH>(&tree, &target_indices).unwrap();

        assert_eq!(multiproof.leaves.len(), 2);
        assert_eq!(multiproof.proofs.len(), 4);
    }

    #[test]
    fn test_verify_merkle_multiproof() {
        let leaves = vec![
            B256::from_slice(&[0xAA; 32]),
            B256::from_slice(&[0xBB; 32]),
            B256::from_slice(&[0xCC; 32]),
            B256::from_slice(&[0xDD; 32]),
            B256::from_slice(&[0xEE; 32]),
            B256::from_slice(&[0xFF; 32]),
            B256::from_slice(&[0x11; 32]),
            B256::from_slice(&[0x22; 32]),
        ];

        let depth = (leaves.len() as f64).log2().ceil() as u64;
        let tree = merkle_tree(&leaves, depth).unwrap();

        let target_indices = vec![0, 5];
        let multiproof = Multiproof::generate::<DEPTH>(&tree, &target_indices).unwrap();

        let root = tree[1];

        multiproof.verify(root).unwrap();
    }
}
