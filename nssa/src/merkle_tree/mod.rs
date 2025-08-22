use std::collections::{HashMap, HashSet};

use sha2::{Digest, Sha256};

mod default_values;

type Value = [u8; 32];
type Node = [u8; 32];

/// Compute parent as the hash of two child nodes
fn hash_two(left: &Node, right: &Node) -> Node {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

fn hash_value(value: &Value) -> Node {
    let mut hasher = Sha256::new();
    hasher.update(value);
    hasher.finalize().into()
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct MerkleTree {
    index_map: HashMap<Value, usize>,
    node_map: HashMap<usize, Node>,
    capacity: usize,
}

impl MerkleTree {
    pub fn root(&self) -> Node {
        let root_index = self.root_index();
        *self.get_node(&root_index)
    }

    fn root_index(&self) -> usize {
        let capacity_depth = self.capacity.trailing_zeros() as usize;
        let diff = capacity_depth - self.depth();
        if diff == 0 { 0 } else { (1 << diff) - 1 }
    }

    fn depth(&self) -> usize {
        self.index_map.len().next_power_of_two().trailing_zeros() as usize
    }

    fn get_node(&self, index: &usize) -> &Node {
        self.node_map.get(&index).unwrap_or_else(|| {
            let index_depth = usize::BITS as usize - (index + 1).leading_zeros() as usize - 1;
            let total_levels = self.capacity.trailing_zeros() as usize;
            if total_levels >= index_depth {
                &default_values::DEFAULT_VALUES[total_levels - index_depth]
            } else {
                //TODO: implement error handling
                panic!();
            }
        })
    }

    fn set_node(&mut self, index: usize, node: Node) {
        self.node_map.insert(index, node);
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        Self {
            index_map: HashMap::with_capacity(capacity),
            node_map: HashMap::with_capacity(capacity << 1),
            capacity,
        }
    }

    fn reallocate_to_double_capacity(&mut self) {
        let mut this = Self::with_capacity(self.capacity << 1);
        let mut pairs: Vec<_> = self.index_map.iter().collect();
        pairs.sort_by_key(|&(_, index)| index);
        for (value, _) in pairs {
            this.insert(*value);
        }
        *self = this;
    }

    pub fn insert(&mut self, value: Value) -> bool {
        if self.index_map.contains_key(&value) {
            return false;
        }

        if self.capacity == self.index_map.len() {
            self.reallocate_to_double_capacity();
        }

        let new_index = self.index_map.len();
        self.index_map.insert(value, new_index);

        let base_length = self.capacity;
        let mut layer_node = hash_value(&value);
        let mut layer_index = new_index + base_length - 1;
        self.set_node(layer_index, layer_node);

        let mut layer = 0;
        let mut top_layer = self.depth();
        while layer < top_layer {
            let is_left_child = layer_index & 1 == 1;

            let (parent_index, new_parent_node) = if is_left_child {
                let parent_index = (layer_index - 1) >> 1;
                let sibling = self.get_node(&(layer_index + 1));
                let new_parent_node = hash_two(&layer_node, sibling);
                (parent_index, new_parent_node)
            } else {
                let parent_index = (layer_index - 2) >> 1;
                let sibling = self.get_node(&(layer_index - 1));
                let new_parent_node = hash_two(sibling, &layer_node);
                (parent_index, new_parent_node)
            };

            self.set_node(parent_index, new_parent_node);

            layer += 1;
            layer_index = parent_index;
            layer_node = new_parent_node
        }

        true
    }

    pub fn new(values: Vec<Value>) -> Self {
        let mut deduplicated_values = Vec::with_capacity(values.len());
        let mut seen = HashSet::new();
        for value in values.into_iter() {
            if !seen.contains(&value) {
                deduplicated_values.push(value);
                seen.insert(value);
            }
        }
        let mut this = Self::with_capacity(deduplicated_values.len());
        for value in deduplicated_values.into_iter() {
            this.insert(value);
        }
        this
    }

    pub fn get_authentication_path_for(&self, value: &Value) -> Option<(usize, Vec<Node>)> {
        let mut result = Vec::with_capacity(self.depth());
        let value_index = self.index_map.get(value)?;
        let base_length = self.capacity;
        let mut layer_index = base_length + value_index - 1;
        let mut layer = 0;
        let top_layer = self.depth();
        while layer < top_layer {
            let is_left_child = layer_index & 1 == 1;
            let (sibling, parent_index) = if is_left_child {
                (self.get_node(&(layer_index + 1)), (layer_index - 1) >> 1)
            } else {
                (self.get_node(&(layer_index - 1)), (layer_index - 2) >> 1)
            };
            result.push(*sibling);

            layer += 1;
            layer_index = parent_index;
        }
        Some((*value_index, result))
    }

    pub(crate) fn contains(&self, value: &[u8; 32]) -> bool {
        self.index_map.contains_key(value)
    }
}

// Reference implementation
fn verify_authentication_path(value: &Value, index: usize, path: &[Node], root: &Node) -> bool {
    let mut result = hash_value(value);
    let mut level_index = index;
    for node in path {
        let is_left_child = level_index & 1 == 0;
        if is_left_child {
            result = hash_two(&result, node);
        } else {
            result = hash_two(node, &result);
        }
        level_index >>= 1;
    }
    &result == root
}

#[cfg(test)]
mod tests {
    use nssa_core::account::{Account, NullifierPublicKey};

    use super::*;

    #[test]
    fn test_merkle_tree_1() {
        let values = vec![[1; 32], [2; 32], [3; 32], [4; 32]];
        let tree = MerkleTree::new(values.clone());
        let expected_root = [
            72, 199, 63, 120, 33, 165, 138, 141, 42, 112, 62, 91, 57, 197, 113, 192, 170, 32, 207,
            20, 171, 205, 10, 248, 242, 185, 85, 188, 32, 41, 152, 222,
        ];

        assert_eq!(tree.root(), expected_root);
        assert_eq!(*tree.index_map.get(&[1; 32]).unwrap(), 0);
        assert_eq!(*tree.index_map.get(&[2; 32]).unwrap(), 1);
        assert_eq!(*tree.index_map.get(&[3; 32]).unwrap(), 2);
        assert_eq!(*tree.index_map.get(&[4; 32]).unwrap(), 3);
        assert_eq!(tree.capacity, 4);
    }

    #[test]
    fn test_merkle_tree_2() {
        let values = vec![[1; 32], [2; 32], [3; 32], [0; 32]];
        let tree = MerkleTree::new(values.clone());
        let expected_root = [
            201, 187, 184, 48, 150, 223, 133, 21, 122, 20, 110, 125, 119, 4, 85, 169, 132, 18, 222,
            224, 99, 49, 135, 238, 134, 254, 230, 200, 164, 91, 131, 26,
        ];

        assert_eq!(tree.root(), expected_root);
        assert_eq!(*tree.index_map.get(&[1; 32]).unwrap(), 0);
        assert_eq!(*tree.index_map.get(&[2; 32]).unwrap(), 1);
        assert_eq!(*tree.index_map.get(&[3; 32]).unwrap(), 2);
        assert_eq!(*tree.index_map.get(&[0; 32]).unwrap(), 3);
        assert_eq!(tree.capacity, 4);
    }

    #[test]
    fn test_merkle_tree_3() {
        let values = vec![[1; 32], [2; 32], [3; 32]];
        let tree = MerkleTree::new(values.clone());
        let expected_root = [
            200, 211, 216, 210, 177, 63, 39, 206, 236, 205, 198, 153, 17, 152, 113, 249, 243, 46,
            167, 237, 134, 255, 69, 208, 173, 17, 247, 123, 40, 205, 117, 104,
        ];

        assert_eq!(tree.root(), expected_root);
        assert_eq!(*tree.index_map.get(&[1; 32]).unwrap(), 0);
        assert_eq!(*tree.index_map.get(&[2; 32]).unwrap(), 1);
        assert_eq!(*tree.index_map.get(&[3; 32]).unwrap(), 2);
        assert!(tree.index_map.get(&[0; 32]).is_none());
        assert_eq!(tree.capacity, 4);
    }

    #[test]
    fn test_merkle_tree_4() {
        let values = vec![[11; 32], [12; 32], [13; 32], [14; 32], [15; 32]];
        let tree = MerkleTree::new(values.clone());
        let expected_root = [
            239, 65, 138, 237, 90, 162, 7, 2, 212, 217, 76, 146, 218, 121, 164, 1, 47, 46, 54, 241,
            0, 139, 253, 179, 205, 30, 56, 116, 157, 202, 36, 153,
        ];

        assert_eq!(tree.root(), expected_root);
        assert_eq!(*tree.index_map.get(&[11; 32]).unwrap(), 0);
        assert_eq!(*tree.index_map.get(&[12; 32]).unwrap(), 1);
        assert_eq!(*tree.index_map.get(&[13; 32]).unwrap(), 2);
        assert_eq!(*tree.index_map.get(&[14; 32]).unwrap(), 3);
        assert_eq!(*tree.index_map.get(&[15; 32]).unwrap(), 4);
        assert_eq!(tree.capacity, 8);
    }

    #[test]
    fn test_merkle_tree_5() {
        let values = vec![
            [11; 32], [12; 32], [12; 32], [13; 32], [14; 32], [15; 32], [15; 32], [13; 32],
            [13; 32], [15; 32], [11; 32],
        ];
        let tree = MerkleTree::new(values);
        let expected_root = [
            239, 65, 138, 237, 90, 162, 7, 2, 212, 217, 76, 146, 218, 121, 164, 1, 47, 46, 54, 241,
            0, 139, 253, 179, 205, 30, 56, 116, 157, 202, 36, 153,
        ];

        assert_eq!(tree.root(), expected_root);
        assert_eq!(*tree.index_map.get(&[11; 32]).unwrap(), 0);
        assert_eq!(*tree.index_map.get(&[12; 32]).unwrap(), 1);
        assert_eq!(*tree.index_map.get(&[13; 32]).unwrap(), 2);
        assert_eq!(*tree.index_map.get(&[14; 32]).unwrap(), 3);
        assert_eq!(*tree.index_map.get(&[15; 32]).unwrap(), 4);
        assert_eq!(tree.capacity, 8);
    }

    #[test]
    fn test_merkle_tree_6() {
        let values = vec![[1; 32], [2; 32], [3; 32], [4; 32], [5; 32]];
        let tree = MerkleTree::new(values);
        let expected_root = [
            6, 156, 184, 37, 154, 6, 254, 110, 219, 63, 167, 255, 121, 51, 166, 221, 125, 202, 111,
            202, 41, 147, 20, 55, 151, 148, 166, 136, 146, 108, 55, 146,
        ];

        assert_eq!(tree.root(), expected_root);
    }

    #[test]
    fn test_with_capacity_4() {
        let tree = MerkleTree::with_capacity(4);

        assert!(tree.index_map.is_empty());
        assert!(tree.node_map.is_empty());
        for i in 3..7 {
            assert_eq!(*tree.get_node(&i), default_values::DEFAULT_VALUES[0], "{i}");
        }
        for i in 1..3 {
            assert_eq!(*tree.get_node(&i), default_values::DEFAULT_VALUES[1], "{i}");
        }
        assert_eq!(*tree.get_node(&0), default_values::DEFAULT_VALUES[2]);
    }

    #[test]
    fn test_with_capacity_5() {
        let tree = MerkleTree::with_capacity(5);

        assert!(tree.index_map.is_empty());
        assert!(tree.node_map.is_empty());
        for i in 7..15 {
            assert_eq!(*tree.get_node(&i), default_values::DEFAULT_VALUES[0])
        }
        for i in 3..7 {
            assert_eq!(*tree.get_node(&i), default_values::DEFAULT_VALUES[1])
        }
        for i in 1..3 {
            assert_eq!(*tree.get_node(&i), default_values::DEFAULT_VALUES[2])
        }
        assert_eq!(*tree.get_node(&0), default_values::DEFAULT_VALUES[3])
    }

    #[test]
    fn test_with_capacity_6() {
        let mut tree = MerkleTree::with_capacity(100);

        let values = vec![[1; 32], [2; 32], [3; 32], [4; 32]];

        let expected_root = [
            72, 199, 63, 120, 33, 165, 138, 141, 42, 112, 62, 91, 57, 197, 113, 192, 170, 32, 207,
            20, 171, 205, 10, 248, 242, 185, 85, 188, 32, 41, 152, 222,
        ];

        tree.insert(values[0]);
        tree.insert(values[1]);
        tree.insert(values[2]);
        tree.insert(values[3]);

        assert_eq!(tree.root(), expected_root);
    }

    #[test]
    fn test_with_capacity_7() {
        let mut tree = MerkleTree::with_capacity(599);

        let values = vec![[1; 32], [2; 32], [3; 32]];

        let expected_root = [
            200, 211, 216, 210, 177, 63, 39, 206, 236, 205, 198, 153, 17, 152, 113, 249, 243, 46,
            167, 237, 134, 255, 69, 208, 173, 17, 247, 123, 40, 205, 117, 104,
        ];

        tree.insert(values[0]);
        tree.insert(values[1]);
        tree.insert(values[2]);

        assert_eq!(tree.root(), expected_root);
    }

    #[test]
    fn test_with_capacity_8() {
        let mut tree = MerkleTree::with_capacity(1);

        let values = vec![[1; 32], [2; 32], [3; 32]];

        let expected_root = [
            200, 211, 216, 210, 177, 63, 39, 206, 236, 205, 198, 153, 17, 152, 113, 249, 243, 46,
            167, 237, 134, 255, 69, 208, 173, 17, 247, 123, 40, 205, 117, 104,
        ];

        tree.insert(values[0]);
        tree.insert(values[1]);
        tree.insert(values[2]);

        assert_eq!(tree.root(), expected_root);
    }

    #[test]
    fn test_insert_value_1() {
        let mut tree = MerkleTree::with_capacity(1);

        let values = vec![[1; 32], [2; 32], [3; 32]];
        let expected_tree = MerkleTree::new(values.clone());

        tree.insert(values[0]);
        tree.insert(values[1]);
        tree.insert(values[2]);

        assert_eq!(expected_tree, tree);
    }

    #[test]
    fn test_insert_value_2() {
        let mut tree = MerkleTree::with_capacity(1);

        let values = vec![[1; 32], [2; 32], [3; 32], [4; 32]];
        let expected_tree = MerkleTree::new(values.clone());

        tree.insert(values[0]);
        tree.insert(values[1]);
        tree.insert(values[2]);
        tree.insert(values[3]);

        assert_eq!(expected_tree, tree);
    }

    #[test]
    fn test_insert_value_3() {
        let mut tree = MerkleTree::with_capacity(1);

        let values = vec![[11; 32], [12; 32], [13; 32], [14; 32], [15; 32]];
        let expected_tree = MerkleTree::new(values.clone());

        tree.insert(values[0]);
        tree.insert(values[1]);
        tree.insert(values[2]);
        tree.insert(values[3]);
        tree.insert(values[4]);

        assert_eq!(expected_tree, tree);
    }

    #[test]
    fn test_insert_value_4() {
        let mut tree = MerkleTree::with_capacity(1);

        let values = vec![[11; 32], [12; 32], [13; 32], [14; 32], [15; 32]];
        let expected_tree = MerkleTree::new(values.clone());

        tree.insert(values[0]);
        tree.insert(values[0]);
        tree.insert(values[1]);
        tree.insert(values[1]);
        tree.insert(values[2]);
        tree.insert(values[3]);
        tree.insert(values[2]);
        tree.insert(values[0]);
        tree.insert(values[4]);
        tree.insert(values[2]);
        tree.insert(values[4]);

        assert_eq!(expected_tree, tree);
    }

    #[test]
    fn test_authentication_path_1() {
        let values = vec![[1; 32], [2; 32], [3; 32], [4; 32]];
        let tree = MerkleTree::new(values);
        let expected_authentication_path = (
            2,
            vec![
                [
                    159, 79, 182, 143, 62, 29, 172, 130, 32, 47, 154, 165, 129, 206, 11, 191, 31,
                    118, 93, 240, 233, 172, 60, 140, 87, 226, 15, 104, 90, 186, 184, 237,
                ],
                [
                    80, 162, 125, 71, 70, 243, 87, 203, 112, 12, 190, 157, 72, 131, 183, 127, 182,
                    79, 1, 40, 130, 138, 52, 137, 220, 106, 111, 33, 221, 191, 36, 20,
                ],
            ],
        );

        let authentication_path = tree.get_authentication_path_for(&[3; 32]).unwrap();
        assert_eq!(authentication_path, expected_authentication_path);
    }

    #[test]
    fn test_authentication_path_2() {
        let values = vec![[1; 32], [2; 32], [3; 32]];
        let tree = MerkleTree::new(values);
        let expected_authentication_path = (
            0,
            vec![
                [
                    117, 135, 123, 180, 29, 57, 59, 95, 184, 69, 92, 230, 14, 205, 141, 218, 0, 29,
                    6, 49, 100, 150, 177, 77, 250, 127, 137, 86, 86, 238, 202, 74,
                ],
                [
                    164, 27, 133, 93, 45, 180, 222, 144, 82, 205, 123, 229, 236, 103, 214, 88, 102,
                    41, 203, 159, 110, 50, 70, 164, 175, 165, 186, 49, 63, 7, 169, 197,
                ],
            ],
        );

        let authentication_path = tree.get_authentication_path_for(&[1; 32]).unwrap();
        assert_eq!(authentication_path, expected_authentication_path);
    }

    #[test]
    fn test_authentication_path_3() {
        let values = vec![[1; 32], [2; 32], [3; 32], [4; 32], [5; 32]];
        let tree = MerkleTree::new(values);
        let expected_authentication_path = (
            4,
            vec![
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
                [
                    245, 165, 253, 66, 209, 106, 32, 48, 39, 152, 239, 110, 211, 9, 151, 155, 67,
                    0, 61, 35, 32, 217, 240, 232, 234, 152, 49, 169, 39, 89, 251, 75,
                ],
                [
                    72, 199, 63, 120, 33, 165, 138, 141, 42, 112, 62, 91, 57, 197, 113, 192, 170,
                    32, 207, 20, 171, 205, 10, 248, 242, 185, 85, 188, 32, 41, 152, 222,
                ],
            ],
        );

        let authentication_path = tree.get_authentication_path_for(&[5; 32]).unwrap();
        assert_eq!(authentication_path, expected_authentication_path);
    }

    #[test]
    fn test_authentication_path_4() {
        let values = vec![[1; 32], [2; 32], [3; 32], [4; 32], [5; 32]];
        let tree = MerkleTree::new(values);
        let value = [6; 32];
        assert!(tree.get_authentication_path_for(&value).is_none());
    }

    #[test]
    fn test_authentication_path_5() {
        let values = vec![[1; 32], [2; 32], [3; 32], [4; 32], [5; 32]];
        let tree = MerkleTree::new(values);
        let value = [0; 32];
        assert!(tree.get_authentication_path_for(&value).is_none());
    }

    #[test]
    fn test_authentication_path_6() {
        let values = vec![[1; 32], [2; 32], [3; 32], [4; 32], [5; 32]];
        let tree = MerkleTree::new(values);
        let value = [5; 32];
        let (index, path) = tree.get_authentication_path_for(&value).unwrap();
        assert!(verify_authentication_path(
            &value,
            index,
            &path,
            &tree.root()
        ));
    }
}

//
