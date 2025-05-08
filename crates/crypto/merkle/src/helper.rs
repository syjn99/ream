use std::collections::HashSet;

use alloy_primitives::B256;

pub const fn get_sibling_index(index: u64) -> u64 {
    index ^ 1
}

pub const fn get_parent_index(index: u64) -> u64 {
    index / 2
}

pub(crate) fn get_generalized_index_bit(index: u64, position: u64) -> bool {
    (index & (1 << position)) > 0
}

pub(crate) fn get_generalized_index_child(index: u64, right_side: bool) -> u64 {
    index * 2 + right_side as u64
}

pub(crate) fn get_subtree_index(generalized_index: u64) -> u64 {
    generalized_index % (1 << (generalized_index as f64).log2().floor() as u64)
}

fn get_branch_indices(tree_index: u64) -> Vec<u64> {
    let mut focus = get_sibling_index(tree_index);
    let mut result = vec![focus];
    while focus > 1 {
        focus = get_sibling_index(get_parent_index(focus));
        result.push(focus);
    }
    result.truncate(result.len() - 1);
    result
}

fn get_path_indices(tree_index: u64) -> Vec<u64> {
    let mut focus = tree_index;
    let mut result = vec![focus];
    while focus > 1 {
        focus = get_parent_index(focus);
        result.push(focus);
    }
    result.truncate(result.len() - 1);
    result
}

pub(crate) fn get_helper_indices(indices: &[u64]) -> Vec<u64> {
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

pub(crate) fn get_generalized_index(index: u64, depth: u64) -> u64 {
    let bottom_length = 1 << depth;
    index + bottom_length
}

/// Common hashing function for Merkle trees.
pub(crate) fn hash(h1: &[u8], h2: &[u8]) -> B256 {
    ethereum_hashing::hash32_concat(h1, h2).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEPTH: u64 = 3;

    #[test]
    fn test_get_generalized_index() {
        let indices = [0, 5];
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
        let indices = [0, 5];
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
}
