use std::collections::{HashMap, HashSet};

use alloy_primitives::B256;
use anyhow::ensure;

pub fn generate_multiproof(
    tree: &[B256],
    indices: &[u64],
    depth: u64,
) -> anyhow::Result<Vec<(B256, u64)>> {
    let bottom_length: u64 = 1 << depth;

    ensure!(!indices.is_empty(), "Indices cannot be empty");
    for &index in indices {
        ensure!(index < bottom_length, "Index out of bounds");
    }

    let generalized_indices: Vec<u64> = indices
        .iter()
        .map(|index| get_generalized_index(*index, depth))
        .collect();
    let helper = get_helper_indices(&generalized_indices);

    let proof: Vec<(B256, u64)> = helper.iter().map(|&g| (tree[g as usize], g)).collect();
    Ok(proof)
}

pub fn calculate_multi_merkle_root(
    leaves: &[B256],
    proof: &[B256],
    generalized_indices: &[u64],
) -> anyhow::Result<B256> {
    if leaves.len() != generalized_indices.len() {
        return Err(anyhow::anyhow!(
            "Invalid proof: leaves and indices length mismatch"
        ));
    }
    let helper_indices = get_helper_indices(generalized_indices);
    if proof.len() != helper_indices.len() {
        return Err(anyhow::anyhow!(
            "Invalid proof: proof and helper indices length mismatch"
        ));
    }

    let mut objects = HashMap::new();
    for (index, node) in generalized_indices.iter().zip(leaves.iter()) {
        objects.insert(*index, *node);
    }
    for (index, node) in helper_indices.iter().zip(proof.iter()) {
        objects.insert(*index, *node);
    }

    let mut keys = objects.keys().cloned().collect::<Vec<_>>();
    keys.sort_by(|a, b| b.cmp(a));

    let mut pos = 0;
    while pos < keys.len() {
        let key = keys.get(pos).unwrap();
        let key_present = objects.contains_key(key);
        let sibling_present = objects.contains_key(&sibling(*key));
        let parent_index = parent(*key);
        let parent_missing = !objects.contains_key(&parent_index);
        let should_compute = key_present && sibling_present && parent_missing;
        if should_compute {
            let right_index = key | 1;
            let left_index = sibling(right_index);
            let left_input = objects.get(&left_index).expect("contains index");
            let right_input = objects.get(&right_index).expect("contains index");

            let value: B256 =
                ethereum_hashing::hash32_concat(left_input.as_slice(), right_input.as_slice())
                    .into();

            let parent = objects.entry(parent_index).or_default();
            parent.copy_from_slice(&value.as_slice());
            keys.push(parent_index);
        }
        pos += 1;
    }

    let root = *objects.get(&1).expect("contains index");
    Ok(root)
}

pub fn verify_merkle_multiproof(
    leaves: &[B256],
    proof: &[B256],
    generalized_indices: &[u64],
    root: B256,
) -> anyhow::Result<()> {
    if calculate_multi_merkle_root(leaves, proof, generalized_indices)? == root {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid proof"))
    }
}

pub fn get_generalized_index(index: u64, depth: u64) -> u64 {
    let bottom_length = 1 << depth;
    index + bottom_length
}

pub const fn sibling(index: u64) -> u64 {
    index ^ 1
}

pub const fn parent(index: u64) -> u64 {
    index / 2
}

fn get_branch_indices(tree_index: u64) -> Vec<u64> {
    let mut focus = sibling(tree_index);
    let mut result = vec![focus];
    while focus > 1 {
        focus = sibling(parent(focus));
        result.push(focus);
    }
    result.truncate(result.len() - 1);
    result
}

fn get_path_indices(tree_index: u64) -> Vec<u64> {
    let mut focus = tree_index;
    let mut result = vec![focus];
    while focus > 1 {
        focus = parent(focus);
        result.push(focus);
    }
    result.truncate(result.len() - 1);
    result
}

fn get_helper_indices(indices: &[u64]) -> Vec<u64> {
    let mut all_helper_indices = HashSet::new();
    let mut all_path_indices = HashSet::new();

    for index in indices {
        all_helper_indices.extend(get_branch_indices(*index).iter());
        all_path_indices.extend(get_path_indices(*index).iter());
    }

    let mut all_branch_indices = all_helper_indices
        .difference(&all_path_indices)
        .cloned()
        .collect::<Vec<_>>();

    all_branch_indices.sort_by(|a: &u64, b: &u64| b.cmp(a));
    all_branch_indices
}

#[cfg(test)]
mod tests {
    use crate::merkle_tree;

    use super::*;

    const DEPTH: u64 = 3;

    #[test]
    fn test_get_generalized_index() {
        let indices = vec![0, 5];
        let generalized_indices = indices
            .iter()
            .map(|&index| get_generalized_index(index, DEPTH))
            .collect::<Vec<_>>();
        assert_eq!(generalized_indices.len(), 2);
        assert_eq!(generalized_indices[0], 8);
        assert_eq!(generalized_indices[1], 13);
    }

    #[test]
    fn test_get_helper_indices() {
        let indices = vec![0, 5];
        let generalized_indices = indices
            .iter()
            .map(|&index| get_generalized_index(index, DEPTH))
            .collect::<Vec<_>>();
        let helper_indices = get_helper_indices(&generalized_indices);
        assert_eq!(helper_indices.len(), 4);
        assert_eq!(helper_indices[0], 12);
        assert_eq!(helper_indices[1], 9);
        assert_eq!(helper_indices[2], 7);
        assert_eq!(helper_indices[3], 5);
    }

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

        let indices = vec![0, 5];
        let multiproof = generate_multiproof(&tree, &indices, depth).unwrap();

        assert_eq!(multiproof.len(), 4);
        assert_eq!(multiproof[0].1, 12);
        assert_eq!(multiproof[1].1, 9);
        assert_eq!(multiproof[2].1, 7);
        assert_eq!(multiproof[3].1, 5);

        assert_eq!(multiproof[0].0, tree[12]);
        assert_eq!(multiproof[1].0, tree[9]);
        assert_eq!(multiproof[2].0, tree[7]);
        assert_eq!(multiproof[3].0, tree[5]);
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

        let indices = vec![0, 5];
        let multiproof = generate_multiproof(&tree, &indices, depth).unwrap();

        let root = tree[1];

        verify_merkle_multiproof(
            &[leaves[0], leaves[5]],
            &multiproof.iter().map(|(node, _)| *node).collect::<Vec<_>>(),
            &indices
                .iter()
                .map(|&index| get_generalized_index(index, depth))
                .collect::<Vec<_>>(),
            root,
        )
        .unwrap();
    }
}
