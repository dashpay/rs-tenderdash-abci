//! Tenderdash's Merkle tree (RFC 6962).
//!
//! This is the tree behind a block's hash, its `data_hash`, its `results_hash`
//! and a validator set's hash. It is a port of the implementation in
//! Tenderdash's
//! [crypto/merkle](https://github.com/dashpay/tenderdash/tree/v2.0.0-dev.1/crypto/merkle)
//! package:
//!
//! - [merkle_hash] is `HashFromByteSlices`,
//! - [MerkleTree] is `ProofsFromByteSlices`,
//! - [merkle_root_from_path] is `Proof.ComputeRootHash`.

use std::vec::Vec;

const LEAF_PREFIX: u8 = 0;
const INNER_PREFIX: u8 = 1;
const HASH_SIZE: usize = 32;

/// Why a Merkle path leads to no root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MerkleError {
    /// The tree has no leaf at that position.
    #[error("no leaf {index} in a tree of {total} leaves")]
    IndexOutOfRange {
        /// The position that was asked for.
        index: u64,
        /// The number of leaves of the tree.
        total: u64,
    },
    /// The path has another number of hashes than a leaf at that position
    /// of a tree of that size needs, or one of them is not 32 bytes long.
    #[error("malformed path for leaf {index} of a tree of {total} leaves")]
    MalformedPath {
        /// The position of the leaf.
        index: u64,
        /// The number of leaves of the tree.
        total: u64,
    },
}

/// Calculate the Merkle root hash of a list of items.
pub fn merkle_hash<T: AsRef<[u8]>>(items: &[T]) -> [u8; 32] {
    match items.len() {
        0 => empty_hash(),
        1 => leaf(items[0].as_ref()),
        length => {
            let (left, right) = items.split_at(split_point(length as u64) as usize);
            inner(&merkle_hash(left), &merkle_hash(right))
        },
    }
}

/// The tree over some items, with the paths of the selected ones: what proves
/// that those items are under the root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleTree {
    /// The root of the tree, as [merkle_hash] calculates it.
    pub root: [u8; 32],
    /// For each selected item, in the order they were selected in, the
    /// hashes from the sibling of its leaf up to a child of the root. These
    /// are the `aunts` of a Tenderdash `Proof`.
    pub paths: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// Build the tree over `items`, keeping paths only for the positions in
    /// `selected`.
    ///
    /// `selected` must be ascending, without repetitions, and within
    /// `items`. Returns `None` otherwise.
    pub fn new<T: AsRef<[u8]>>(items: &[T], selected: &[usize]) -> Option<Self> {
        let ascending = selected.windows(2).all(|pair| pair[0] < pair[1]);
        let within = selected.last().is_none_or(|&last| last < items.len());
        if !ascending || !within {
            return None;
        }

        Some(Self::build(items, 0, selected))
    }

    /// Build the tree over all `items`, with a path for each.
    pub fn with_all_paths<T: AsRef<[u8]>>(items: &[T]) -> Self {
        let selected: Vec<usize> = (0..items.len()).collect();

        Self::build(items, 0, &selected)
    }

    /// `first` is the position of `items[0]` in the whole tree, which is
    /// what the positions in `selected` count from.
    fn build<T: AsRef<[u8]>>(items: &[T], first: usize, selected: &[usize]) -> Self {
        if selected.is_empty() {
            return Self {
                root: merkle_hash(items),
                paths: Vec::new(),
            };
        }
        if items.len() < 2 {
            return Self {
                root: merkle_hash(items),
                paths: vec![Vec::new(); selected.len()],
            };
        }

        let split = split_point(items.len() as u64) as usize;
        let (left_items, right_items) = items.split_at(split);
        let (left_selected, right_selected) =
            selected.split_at(selected.partition_point(|&position| position < first + split));
        let left = Self::build(left_items, first, left_selected);
        let right = Self::build(right_items, first + split, right_selected);

        let root = inner(&left.root, &right.root);
        let paths = left
            .paths
            .into_iter()
            .map(|mut path| {
                path.push(right.root);
                path
            })
            .chain(right.paths.into_iter().map(|mut path| {
                path.push(left.root);
                path
            }))
            .collect();

        Self { root, paths }
    }
}

/// Follow `path` up from the item at `index` of a tree of `total` items,
/// returning the root it leads to.
///
/// `path` runs from the sibling of the item's leaf up to a child of the root,
/// as [MerkleTree] builds it. The item is under a root when the returned root
/// equals it.
///
/// The position decides the order in which each pair is hashed, so a path
/// leads to the root of its tree only from the position it was built for.
/// `total` is the caller's claim: a path can also fit a tree of another size,
/// at another position.
pub fn merkle_root_from_path<P: AsRef<[u8]>>(
    index: u64,
    total: u64,
    item: &[u8],
    path: &[P],
) -> Result<[u8; 32], MerkleError> {
    if index >= total {
        return Err(MerkleError::IndexOutOfRange { index, total });
    }

    // Walk down from the root to the leaf, noting at each level which side the
    // leaf is on
    let (mut position, mut size) = (index, total);
    let mut leaf_is_on_the_left = Vec::new();
    while size > 1 {
        let split = split_point(size);
        if position < split {
            leaf_is_on_the_left.push(true);
            size = split;
        } else {
            leaf_is_on_the_left.push(false);
            position -= split;
            size -= split;
        }
    }
    let malformed = path.len() != leaf_is_on_the_left.len()
        || path
            .iter()
            .any(|sibling| sibling.as_ref().len() != HASH_SIZE);
    if malformed {
        return Err(MerkleError::MalformedPath { index, total });
    }

    // Then hash back up, the path giving the other side at each level
    let root = path.iter().zip(leaf_is_on_the_left.iter().rev()).fold(
        leaf(item),
        |hash, (sibling, &on_the_left)| {
            if on_the_left {
                inner(&hash, sibling.as_ref())
            } else {
                inner(sibling.as_ref(), &hash)
            }
        },
    );

    Ok(root)
}

fn leaf(data: &[u8]) -> [u8; 32] {
    let mut buf = vec![LEAF_PREFIX; 1];
    buf.extend(data);
    lhash::sha256(&buf)
}

fn inner(left: &[u8], right: &[u8]) -> [u8; 32] {
    let mut buf = vec![INNER_PREFIX; 1];
    buf.extend(left);
    buf.extend(right);
    lhash::sha256(&buf)
}

fn empty_hash() -> [u8; 32] {
    lhash::sha256(&[])
}

/// The largest power of two below `length`, which must be at least 2.
fn split_point(length: u64) -> u64 {
    1 << (u64::BITS - 1 - (length - 1).leading_zeros())
}

#[cfg(test)]
mod test {
    use std::{string::String, vec::Vec};

    use super::{
        MerkleError, MerkleTree, inner, leaf, merkle_hash, merkle_root_from_path, split_point,
    };

    /// What Tenderdash's `merkle.ProofsFromByteSlices` returns for the items
    /// `leaf-0` to `leaf-6`: the root, and the aunts of each proof.
    const TENDERDASH_ROOT: &str =
        "0b007fb915eb9b2a146f54b1c86ec53b664f8e455b7660b0b6ee13edc0d921c0";
    const TENDERDASH_PATHS: [&[&str]; 7] = [
        &[
            "3145c409f259b7c53e32036090ff76751025a2498ba9823ef718cac50b4e616f",
            "bd45ff28796704d88bdac51b1df553fda59837b616d6d1cb2114dbc3b087ff69",
            "8eae6bd3b3a07f1f75ee72a531629e6eb31e42e62f760e47de52a53c3641ef23",
        ],
        &[
            "305df59f9590c3c9ac63d2b2743c388e3792449078cebf7fb3dbe6471643b2b7",
            "bd45ff28796704d88bdac51b1df553fda59837b616d6d1cb2114dbc3b087ff69",
            "8eae6bd3b3a07f1f75ee72a531629e6eb31e42e62f760e47de52a53c3641ef23",
        ],
        &[
            "f76836325aec5699d8d71f8e42e9d47c5c29b08059ba296384f7ca40ad3a40ae",
            "60a53eed0de87a90c8e59427c59c46253c33a76a09502a51801300927b7e6bdc",
            "8eae6bd3b3a07f1f75ee72a531629e6eb31e42e62f760e47de52a53c3641ef23",
        ],
        &[
            "fca89f57c9f8c8eb4047a7ff9d333acf9e0f3384b20b255bceab0f216dcca267",
            "60a53eed0de87a90c8e59427c59c46253c33a76a09502a51801300927b7e6bdc",
            "8eae6bd3b3a07f1f75ee72a531629e6eb31e42e62f760e47de52a53c3641ef23",
        ],
        &[
            "8f1593cb92f429d9340b9bbc1f0bb122adf8026c42a4a42142e2168931727236",
            "676f3782f5b3a5fb4370ed49572cedc523f4a66322269c85f2af0509d17b0a4d",
            "bdd1c5ff55b19cb6b0e7c761bf9a6ccaa27fbbfc07b74f1fabb6e911a0bd2ab3",
        ],
        &[
            "ea9fc1a1b6e191b460d0d6306e3e870c173f39330f13cda1b70cfc72bdc398ba",
            "676f3782f5b3a5fb4370ed49572cedc523f4a66322269c85f2af0509d17b0a4d",
            "bdd1c5ff55b19cb6b0e7c761bf9a6ccaa27fbbfc07b74f1fabb6e911a0bd2ab3",
        ],
        &[
            "985bb5d36b927800876871da925a7e82abe83a9ddba5882920a007a55ea2b376",
            "bdd1c5ff55b19cb6b0e7c761bf9a6ccaa27fbbfc07b74f1fabb6e911a0bd2ab3",
        ],
    ];

    fn tenderdash_items() -> Vec<Vec<u8>> {
        (0..7)
            .map(|index| format!("leaf-{index}").into_bytes())
            .collect()
    }

    fn numbered_items(total: usize) -> Vec<Vec<u8>> {
        (0..total)
            .map(|index| index.to_be_bytes().to_vec())
            .collect()
    }

    fn hex_path(path: &[[u8; 32]]) -> Vec<String> {
        path.iter().map(hex::encode).collect()
    }

    #[test]
    /// Given a set of test vectors that are the same as on Tenderdash tests,
    /// When calculating the Merkle root hash,
    /// Then the result should be the same as the expected value generated in
    /// Tenderdash.
    ///
    /// ## See also
    ///
    /// Tenderdash test [TestHashFromByteSlices](https://github.com/dashpay/tenderdash/blob/feat/statesync-integration/crypto/merkle/tree_test.go#L21)
    fn test_merkle_root() {
        let items = vec![vec![1, 2], vec![3, 4], vec![5, 6], vec![7, 8], vec![9, 10]];
        let root = merkle_hash(&items);
        let expected = "f326493eceab4f2d9ffbc78c59432a0a005d6ea98392045c74df5d14a113be18";
        assert_eq!(hex::encode(root), expected)
    }

    /// The vectors of Tenderdash's `rfc6962_test.go`.
    #[test]
    fn test_rfc_6962_hashes() {
        assert_eq!(
            hex::encode(merkle_hash::<Vec<u8>>(&[])),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex::encode(merkle_hash(&[Vec::new()])),
            "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d"
        );
        assert_eq!(
            hex::encode(merkle_hash(&[b"L123456".to_vec()])),
            "395aa064aa4c29f7010acfe3f25db9485bbd4b91897b6ad7ad547639252b4d56"
        );
        assert_eq!(
            hex::encode(inner(b"N123", b"N456")),
            "aa217fe888e47007fa15edab33c2b492a722cb106c64667fc2b044444de66bbb"
        );
    }

    #[test]
    fn test_split_point() {
        let expected = [
            (2, 1),
            (3, 2),
            (4, 2),
            (5, 4),
            (8, 4),
            (9, 8),
            (u64::MAX, 1 << 63),
        ];
        for (length, split) in expected {
            assert_eq!(split_point(length), split, "length {length}");
        }
    }

    #[test]
    fn test_tree_matches_tenderdash_proofs() {
        let items = tenderdash_items();

        let tree = MerkleTree::with_all_paths(&items);

        assert_eq!(hex::encode(tree.root), TENDERDASH_ROOT);
        assert_eq!(tree.paths.len(), items.len());
        for (path, expected) in tree.paths.iter().zip(TENDERDASH_PATHS) {
            assert_eq!(hex_path(path), expected);
        }
    }

    #[test]
    fn test_tenderdash_paths_lead_to_tenderdash_root() {
        let items = tenderdash_items();

        for (index, (item, path)) in items.iter().zip(TENDERDASH_PATHS).enumerate() {
            let path = path
                .iter()
                .map(|hash| hex::decode(hash).unwrap())
                .collect::<Vec<_>>();

            let root = merkle_root_from_path(index as u64, 7, item, &path).unwrap();

            assert_eq!(hex::encode(root), TENDERDASH_ROOT, "item {index}");
        }
    }

    #[test]
    fn test_tree_root_is_merkle_hash() {
        for total in 0..=33 {
            let items = numbered_items(total);

            assert_eq!(MerkleTree::with_all_paths(&items).root, merkle_hash(&items));
            assert_eq!(
                MerkleTree::new(&items, &[]).unwrap().root,
                merkle_hash(&items)
            );
        }
    }

    #[test]
    fn test_tree_builds_only_selected_paths() {
        let items = tenderdash_items();

        for selected in [vec![], vec![0], vec![6], vec![1, 4], vec![2, 3, 5]] {
            let tree = MerkleTree::new(&items, &selected).unwrap();

            assert_eq!(hex::encode(tree.root), TENDERDASH_ROOT);
            assert_eq!(tree.paths.len(), selected.len());
            for (path, position) in tree.paths.iter().zip(&selected) {
                assert_eq!(hex_path(path), TENDERDASH_PATHS[*position]);
            }
        }
    }

    #[test]
    fn test_tree_refuses_a_bad_selection() {
        let items = tenderdash_items();

        // outside the items, repeated, and descending
        for selected in [vec![7], vec![0, 7], vec![3, 3], vec![4, 1]] {
            assert_eq!(MerkleTree::new(&items, &selected), None, "{selected:?}");
        }
        assert_eq!(MerkleTree::new::<Vec<u8>>(&[], &[0]), None);
    }

    #[test]
    fn test_every_item_has_a_path_to_the_root() {
        for total in 1..=33 {
            let items = numbered_items(total);
            let tree = MerkleTree::with_all_paths(&items);

            for (index, item) in items.iter().enumerate() {
                let root =
                    merkle_root_from_path(index as u64, total as u64, item, &tree.paths[index])
                        .unwrap();
                assert_eq!(root, tree.root, "item {index} of {total}");
            }
        }
    }

    #[test]
    fn test_path_leads_to_the_root_from_its_own_position_only() {
        for total in 2..=33 {
            let items = numbered_items(total);
            let tree = MerkleTree::with_all_paths(&items);

            for (index, item) in items.iter().enumerate() {
                for claimed in (0..total).filter(|&claimed| claimed != index) {
                    let root = merkle_root_from_path(
                        claimed as u64,
                        total as u64,
                        item,
                        &tree.paths[index],
                    );
                    assert_ne!(root.ok(), Some(tree.root), "item {index} as {claimed}");
                }
            }
        }
    }

    #[test]
    fn test_path_leads_to_the_root_from_its_own_item_only() {
        let items = tenderdash_items();
        let tree = MerkleTree::new(&items, &[3]).unwrap();

        let root = merkle_root_from_path(3, 7, b"leaf-4", &tree.paths[0]).unwrap();

        assert_ne!(root, tree.root);
    }

    #[test]
    fn test_path_of_a_position_outside_the_tree() {
        let no_path: [[u8; 32]; 0] = [];

        assert_eq!(
            merkle_root_from_path(7, 7, b"leaf", &no_path),
            Err(MerkleError::IndexOutOfRange { index: 7, total: 7 })
        );
        assert_eq!(
            merkle_root_from_path(0, 0, b"leaf", &no_path),
            Err(MerkleError::IndexOutOfRange { index: 0, total: 0 })
        );
    }

    #[test]
    fn test_path_of_the_wrong_shape() {
        let items = tenderdash_items();
        let tree = MerkleTree::new(&items, &[2]).unwrap();
        let path = &tree.paths[0];
        let malformed = Err(MerkleError::MalformedPath { index: 2, total: 7 });

        // too short, too long, and a sibling that is not a hash
        assert_eq!(
            merkle_root_from_path(2, 7, b"leaf-2", &path[1..]),
            malformed
        );
        let mut longer = path.clone();
        longer.push([0; 32]);
        assert_eq!(merkle_root_from_path(2, 7, b"leaf-2", &longer), malformed);
        let mut truncated: Vec<Vec<u8>> = path.iter().map(|hash| hash.to_vec()).collect();
        truncated[0].pop();
        assert_eq!(
            merkle_root_from_path(2, 7, b"leaf-2", &truncated),
            malformed
        );

        // a single item is its own root and has no path
        assert_eq!(
            merkle_root_from_path(0, 1, b"leaf", &[[0u8; 32]]),
            Err(MerkleError::MalformedPath { index: 0, total: 1 })
        );
        let no_path: [[u8; 32]; 0] = [];
        assert_eq!(
            merkle_root_from_path(0, 1, b"leaf", &no_path),
            Ok(leaf(b"leaf"))
        );
    }

    /// The largest tree there can be is as deep as a path can get: following
    /// one must not overflow.
    #[test]
    fn test_path_in_the_largest_tree() {
        let path = [[7u8; 32]; 64];

        // the first leaf is the deepest, the last one is one level up
        assert!(merkle_root_from_path(0, u64::MAX, b"leaf", &path).is_ok());
        assert!(merkle_root_from_path(u64::MAX - 1, u64::MAX, b"leaf", &path[1..]).is_ok());
    }
}
