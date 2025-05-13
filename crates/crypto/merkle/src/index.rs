/// ``LeafIndex`` is the index of a leaf in the **bottom** layer of the ``tree``.
pub(crate) type LeafIndex = u64;

/// ``GeneralizedIndex`` is the index of a node in the ``tree``.
pub(crate) type GeneralizedIndex = u64;

/// Return the given bit of a generalized index.
/// Note: It is fine to pass ``LeafIndex`` to this function,
/// as the result will be the same.
pub(crate) fn get_generalized_index_bit(index: GeneralizedIndex, position: u64) -> bool {
    (index & (1 << position)) > 0
}

pub(crate) fn generalized_index_child(
    index: GeneralizedIndex,
    right_side: bool,
) -> GeneralizedIndex {
    index * 2 + right_side as GeneralizedIndex
}

pub(crate) fn get_subtree_index(generalized_index: GeneralizedIndex) -> LeafIndex {
    generalized_index % (1 << (generalized_index as f64).log2().floor() as u64)
}

/// Return the generalized index of the leaf index with ``depth``.
pub(crate) fn generalized_index_from_leaf_index(
    leaf_index: LeafIndex,
    depth: u64,
) -> GeneralizedIndex {
    leaf_index + (1 << depth)
}
