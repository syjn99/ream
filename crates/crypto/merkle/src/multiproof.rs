use std::collections::{BTreeMap, HashMap};

use alloy_primitives::B256;
use anyhow::ensure;

use crate::helper::{
    get_generalized_index, get_helper_indices, get_parent_index, get_sibling_index, hash,
};

type GeneralizedIndex = u64;

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
    pub fn generate(tree: &[B256], indices: &[u64]) -> anyhow::Result<Self> {
        let depth = ((tree.len() as f64).log2().floor() as u64) - 1;
        let bottom_length: u64 = 1 << depth;

        ensure!(!indices.is_empty(), "Indices cannot be empty");
        for &index in indices {
            ensure!(index < bottom_length, "Index out of bounds");
        }

        let generalized_indices: Vec<GeneralizedIndex> = indices
            .iter()
            .map(|&index| get_generalized_index(index, depth))
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
            let sibling_present = objects.contains_key(&get_sibling_index(*key));
            let parent_index = get_parent_index(*key);
            let parent_missing = !objects.contains_key(&parent_index);
            let should_compute = key_present && sibling_present && parent_missing;
            if should_compute {
                let right_index = key | 1;
                let left_index = get_sibling_index(right_index);
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
        let multiproof = Multiproof::generate(&tree, &target_indices).unwrap();

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
        let multiproof = Multiproof::generate(&tree, &target_indices).unwrap();

        let root = tree[1];

        multiproof.verify(root).unwrap();
    }
}
