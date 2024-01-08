use core::panic;
use std::{collections::HashMap, fmt, mem};

use debug_tree::{add_branch, add_leaf, defer_print};

use crate::{bit_stream::BitStream, package_merge::package_merge};

/// A huffman-encoded value, containing both the code length and code.
pub type HuffmanCode = (u8, u16);
/// A map mapping input values to their respective huffman encoded version
pub type HuffmanCodeMap = HashMap<u8, HuffmanCode>;

#[derive(PartialEq)]
pub struct HuffmanNode<T: PartialEq> {
    pub chance: u64,
    pub content: Option<T>,
    pub left: Option<Box<HuffmanNode<T>>>,
    pub right: Option<Box<HuffmanNode<T>>>,
}

impl<T: PartialEq> HuffmanNode<T> {
    pub fn content(&self) -> &Option<T> {
        &self.content
    }
    pub fn left(&self) -> &Option<Box<HuffmanNode<T>>> {
        &self.left
    }
    pub fn right(&self) -> &Option<Box<HuffmanNode<T>>> {
        &self.right
    }
}

/// Parse a stream of values, create a huffman tree and encode the values.
/// Returns the stream of encoded data and the map used for encoding.
///
/// # Arguments
///
/// * stream: The stream of data to read.
pub fn encode(stream: &mut BitStream) -> (BitStream, HuffmanCodeMap) {
    let code_map = parse_u8_stream(stream).code_map();
    let mut result = BitStream::open();

    for byte in stream.data() {
        let (len, code) = code_map.get(byte).unwrap();
        result.append_n_bits(*code, *len);
    }

    (result, code_map)
}

/// Parse a stream of u8 values and create a huffman tree for them.
/// The tree grows to the right, meaning no left node ever has a bigger max_depth() than the corresponding
/// right node's max_depth().
/// The tree's height/code length is also restricted to 16 bits.
///
/// # Arguments
///
/// * `stream`: The stream of data to read.
///
/// # Panics
/// * If there are more symbols than can be encoded in 16 bit codes.
pub fn parse_u8_stream(stream: &mut BitStream) -> HuffmanNode<u8> {
    let mut tree = package_merge(stream, 15);

    tree.remove_only_ones_code();

    tree
}

/// Create all huffman leaves for a stream of u8 values.
///
/// # Arguments
///
/// * `stream`: The stream of data to read.
pub(crate) fn get_single_leaves(stream: &mut BitStream) -> Vec<HuffmanNode<u8>> {
    let mut nodes: Vec<HuffmanNode<u8>> = vec![];
    for byte in stream.data() {
        increment_or_append(&mut nodes, *byte);
    }
    nodes
}

/// With a vec of huffman nodes, either increment the chance of the node with the given value
/// or create a new node with the value if none exists yet.
///
/// # Arguments
///
/// * nodes: The vec of nodes to alter.
/// * value: The value to add or increment.
fn increment_or_append(nodes: &mut Vec<HuffmanNode<u8>>, value: u8) {
    if let Some(node) = nodes.iter_mut().find(|n| n.content.unwrap() == value) {
        node.chance += 1;
    } else {
        nodes.push(HuffmanNode {
            chance: 1,
            content: Some(value),
            left: None,
            right: None,
        })
    }
}

/// Build a huffman tree from a vector of leaf nodes.
///
/// # Arguments
///
/// * `nodes`: The leaf nodes to build a tree from.
fn build_huffman_tree(nodes: &mut Vec<HuffmanNode<u8>>) -> HuffmanNode<u8> {
    let mut sort_lambda = |node: &HuffmanNode<u8>| node.chance();
    build_tree(nodes, &mut sort_lambda)
}

/// Build a tree from a vector of leaf nodes, using the provided sort function.
/// The two nodes with the smallest result from the sort function will be combined with a parent node that's put
/// back in the vector, then it again gets the two nodes with the smallest result, etc.
/// Only leaf nodes will contain values.
///
/// # Arguments
///
/// * `nodes`: The leaf nodes to build a tree from.
/// * `sort_lambda`: The function to sort nodes with.
fn build_tree<K, F>(nodes: &mut Vec<HuffmanNode<u8>>, sort_lambda: &mut F) -> HuffmanNode<u8>
where
    F: FnMut(&HuffmanNode<u8>) -> K,
    K: Ord,
{
    while !nodes.len() > 1 {
        nodes.sort_by_key(&mut *sort_lambda);
        // use swap_remove because we don't care how elements are reordered and it's O(1) rather than O(n)
        let mut bigger_node = nodes.swap_remove(1);
        let smaller_node = nodes.swap_remove(0);
        bigger_node = combine_nodes(bigger_node, smaller_node);
        nodes.push(bigger_node);
    }
    if nodes.is_empty() {
        return HuffmanNode::default();
    }
    nodes.remove(0)
}

/// Combine two nodes and make them leaves of a new node.
/// # Arguments
///
/// * node_1: The first node to include.
/// * node_2: The second node to include.
fn combine_nodes(node_1: HuffmanNode<u8>, node_2: HuffmanNode<u8>) -> HuffmanNode<u8> {
    HuffmanNode {
        left: Some(Box::from(node_2)),
        right: Some(Box::from(node_1)),
        ..Default::default()
    }
}

/// Constructs a Huffman tree from a list of Huffman nodes and a mapping of symbols to code lengths.
/// The resulting tree is represented by the root node.
///
/// # Arguments
///
/// * `nodes` - A mutable reference to a vector of Huffman nodes.
/// * `map` - A mutable reference to a hashmap that maps symbols to code lengths and code values.
///
/// # Returns
///
/// The root node of the constructed Huffman tree.
///
/// # Examples
///
/// ```
/// let mut nodes = vec![
///     HuffmanNode::new(Some(0), 2),
///     HuffmanNode::new(Some(1), 3),
///     HuffmanNode::new(Some(2), 3),
///     HuffmanNode::new(Some(3), 4),
/// ];
///
/// let mut map = HashMap::new();
/// map.insert(0, (2, 0b00));
/// map.insert(1, (3, 0b010));
/// map.insert(2, (3, 0b011));
/// map.insert(3, (4, 0b1000));
///
/// let root = code_len_to_tree(&mut nodes, &mut map);
/// ```
pub fn code_len_to_tree(
    nodes: &mut Vec<HuffmanNode<u8>>,
    map: &mut HuffmanCodeMap,
) -> HuffmanNode<u8> {
    let mut root = HuffmanNode::default();
    let mut current = &mut root;
    let mut current_height = 0;
    while !nodes.is_empty() {
        let leaf = nodes.remove(0);
        let destination = map.get(&leaf.content().unwrap()).unwrap().0 - 1;
        while current_height < destination {
            if current.right().is_none() && current.left().is_none() {
                current.right = Some(Box::from(HuffmanNode::default()));
                current = current.right_unchecked_mut();
            } else if current.right().is_some()
                && current
                    .right_unchecked()
                    .has_space_at_depth((destination - current_height - 1) as u16, false)
            {
                current = current.right_unchecked_mut();
            } else if current.left().is_some()
                && current
                    .left_unchecked()
                    .has_space_at_depth((destination - current_height - 1) as u16, false)
            {
                current = current.left_unchecked_mut();
            } else if current.left().is_none() {
                current.left = Some(Box::from(HuffmanNode::default()));
                current = current.left_unchecked_mut();
            } else {
                panic!("Tree path error smth");
            }
            current_height += 1;
        }
        if current.right().is_none() {
            current.right = Some(Box::from(leaf));
        } else if current.left().is_none() {
            current.left = Some(Box::from(leaf));
        } else {
            panic!("Leaf error");
        }
        current = &mut root;
        current_height = 0;
    }
    root
}

impl HuffmanNode<u8> {
    /// get an immutable reference to this node's left child.
    ///
    /// # Panics
    /// * if the left child is None.
    pub fn left_unchecked(&self) -> &HuffmanNode<u8> {
        self.left.as_ref().unwrap()
    }

    /// get an immutable reference to this node's right child.
    ///
    /// # Panics
    /// * if the right child is None.
    pub fn right_unchecked(&self) -> &HuffmanNode<u8> {
        self.right.as_ref().unwrap()
    }

    /// get a mutable reference to this node's left child.
    ///
    /// # Panics
    /// * if the left child is None.
    pub fn left_unchecked_mut(&mut self) -> &mut HuffmanNode<u8> {
        self.left.as_mut().unwrap()
    }

    /// get a mutable reference to this node's right child.
    ///
    /// # Panics
    /// * if the right child is None.
    pub fn right_unchecked_mut(&mut self) -> &mut HuffmanNode<u8> {
        self.right.as_mut().unwrap()
    }
    /// Calculate the chance/frequency for all symbols in this node and its child nodes.
    pub(crate) fn chance(&self) -> u64 {
        let mut result = self.chance;
        if self.left.is_some() {
            result += self.left_unchecked().chance();
        }
        if self.right.is_some() {
            result += self.right_unchecked().chance();
        }
        result
    }

    /// Set the chance for this node.
    pub fn set_chance(&mut self, chance: u64) {
        self.chance = chance;
    }

    /// Get the maximum depth (i.e. the maximum possible amount of nodes to go through before arriving at a leaf)
    /// of this node.
    /// Leaves are counted too, so if this node is a leaf, this function returns 1.
    pub fn max_depth(&self) -> u16 {
        1 + std::cmp::max(
            match &self.left {
                Some(left) => left.max_depth(),
                None => 0,
            },
            match &self.right {
                Some(right) => right.max_depth(),
                None => 0,
            },
        )
    }

    /// Get the minimum depth (i.e. the minimum possible amount of nodes to go through before arriving at a leaf)
    /// of this node.
    /// Leaves are counted too, so if this node is a leaf, this function returns 1.
    pub fn min_depth(&self) -> u16 {
        let left = self.left.as_ref().map(|left| left.min_depth());
        let right = self.right.as_ref().map(|right| right.min_depth());

        if left.is_none() && right.is_none() {
            return 1;
        }

        1 + std::cmp::min(
            match left {
                Some(value) => value,
                None => u16::MAX,
            },
            match right {
                Some(value) => value,
                None => u16::MAX,
            },
        )
    }

    /// Create a clone of this leaf node.
    /// Only content and chance are kept, left and right child nodes are not included in the clone.
    fn clone_leaf(&self) -> HuffmanNode<u8> {
        HuffmanNode {
            content: self.content,
            chance: self.chance,
            ..Default::default()
        }
    }

    /// Create a code from this tree. The result is a HashMap
    /// with the values as keys and a tuple of code length and code as values.
    pub fn code_map(&self) -> HuffmanCodeMap {
        let mut map = HashMap::with_capacity(2_i32.pow(self.max_depth() as u32) as usize);
        self.append_to_map(&mut map, 0, 0);
        map
    }

    /// Append this node's data to the map. Then recursively call
    /// child nodes to append their data.
    ///
    /// # Arguments
    ///
    /// * `map`: The map to append codes to.
    /// * `code`: The code bits for this node.
    /// * `code_len`: The length of the code for this node.
    fn append_to_map(&self, map: &mut HuffmanCodeMap, code: u16, code_len: u8) {
        if self.content.is_some() {
            map.insert(self.content.unwrap(), (code_len, code));
        }
        if self.left.is_some() {
            self.left_unchecked()
                .append_to_map(map, code << 1, code_len + 1);
        }
        if self.right.is_some() {
            self.right_unchecked()
                .append_to_map(map, (code << 1) + 1, code_len + 1);
        }
    }

    /// Update this tree to ensure it grows to the right, i.e. at any child node, the left child's maximum depth is not larger than
    /// its right child's minimum depth.
    /// Unused but left in to show it.
    fn ensure_tree_grows_right(&mut self) {
        if self.left.is_some() && self.right.is_none() {
            let left = self.left.take();
            self.right = left;
        } else if self.left.is_some()
            && self.left_unchecked().max_depth() > self.right_unchecked().min_depth()
        {
            if self.left_unchecked().min_depth() >= self.right_unchecked().max_depth() {
                mem::swap(&mut self.right, &mut self.left);
            } else {
                self.swap_inner_nodes();
            }
        }

        if self.left.is_some() {
            self.left_unchecked_mut().ensure_tree_grows_right();
        }
        if self.right.is_some() {
            self.right_unchecked_mut().ensure_tree_grows_right();
        }
    }

    /// swap this node's right child's left child with the left child's right child node.
    /// If this node's left child's max depth is greater than this node's right child's, swap them too.
    fn swap_inner_nodes(&mut self) {
        if self.left_unchecked().max_depth() > self.right_unchecked().max_depth() {
            mem::swap(&mut self.right, &mut self.left);
        }
        mem::swap(
            &mut self.right.as_mut().unwrap().left,
            &mut self.left.as_mut().unwrap().right,
        );
    }

    /// Remove the lower right leaf (1*) and replace it with a node which has only a leaf on the left.
    /// This will lead to a less optimised code
    fn remove_only_ones_code(&mut self) {
        if self.right.is_none() {
            return;
        }
        let mut current = self;
        while current.right.is_some() {
            current = current.right_unchecked_mut();
        }
        let new_left_node = HuffmanNode {
            chance: current.chance,
            content: current.content,
            left: None,
            right: None,
        };
        current.content = None;
        current.chance = 0;
        current.left = Some(Box::from(new_left_node))
    }

    /// Checks if the Huffman tree has space at a given depth.
    ///
    /// This function checks if the Huffman tree has space at the specified depth. The `depth` parameter
    /// specifies the depth at which to check for space. The `leaves_count_as_space` parameter determines
    /// whether the number of leaves at the specified depth should be considered as space.
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth at which to check for space.
    /// * `leaves_count_as_space` - Determines whether the number of leaves at the specified depth should be considered as space.
    ///
    /// # Returns
    ///
    /// Returns `true` if the Huffman tree has space at the specified depth, otherwise `false`.
    ///
    /// # Example
    ///
    /// ```
    /// let tree = HuffmanTree::new();
    /// let has_space = tree.has_space_at_depth(2, true);
    /// assert_eq!(has_space, true);
    /// ```
    fn has_space_at_depth(&self, depth: u16, leaves_count_as_space: bool) -> bool {
        if self.content.is_some() {
            if leaves_count_as_space {
                depth != 0
            } else {
                false
            }
        } else if self.right.is_none() || self.left.is_none() {
            true
        } else if depth == 0 {
            false
        } else {
            return self
                .left_unchecked()
                .has_space_at_depth(depth - 1, leaves_count_as_space)
                || self
                    .right_unchecked()
                    .has_space_at_depth(depth - 1, leaves_count_as_space);
        }
    }
}

impl Default for HuffmanNode<u8> {
    fn default() -> Self {
        HuffmanNode {
            chance: 0,
            content: None,
            left: None,
            right: None,
        }
    }
}

impl fmt::Debug for HuffmanNode<u8> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        defer_print!();
        build_debug_tree(self, true);
        writeln!(
            f,
            "========================================================="
        )
    }
}

fn build_debug_tree(current: &HuffmanNode<u8>, is_left: bool) {
    if current.content.is_some() {
        if is_left {
            add_leaf!("0: {}", current.content.unwrap());
        } else {
            add_leaf!("1: {}", current.content.unwrap());
        }
    } else {
        add_branch!("{}", u8::from(!is_left));
        if current.left.is_some() {
            build_debug_tree(current.left_unchecked(), true);
        }
        if current.right.is_some() {
            build_debug_tree(current.right_unchecked(), false);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rand::Rng;

    use crate::{bit_stream::BitStream, huffman::increment_or_append};

    use super::{encode, parse_u8_stream, HuffmanNode};

    // TODO: tests zumindest für remove_only_ones_code, code_len_to_tree, has_space_at_depth
    // TODO: tests für parse_u8_stream() müssen auch nach rechtswachsendheit prüfen!

    #[test]
    fn test_parse_empty_stream() {
        let expected_tree = HuffmanNode {
            chance: 0,
            content: None,
            right: None,
            left: None,
        };
        let mut stream = BitStream::open();

        let actual_tree = parse_u8_stream(&mut stream);
        assert_eq!(expected_tree, actual_tree)
    }

    #[test]
    fn test_parse_stream_with_one_byte() {
        let mut stream = BitStream::open();
        stream.append_byte(1);
        let tree = parse_u8_stream(&mut stream);
        assert_eq!(
            HuffmanNode {
                chance: 0,
                content: None,
                right: Some(Box::from(HuffmanNode {
                    chance: 0,
                    content: None,
                    left: Some(Box::from(HuffmanNode {
                        chance: u64::MAX - 1,
                        content: Some(1),
                        ..Default::default()
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            },
            tree
        );
    }

    #[test]
    fn test_encode_example() {
        /*
        test with the following likelihoods:
        - 1: 4
        - 2: 4
        - 3: 6
        - 4: 6
        - 5: 7
        - 6: 9
        */
        let mut stream = BitStream::open();
        stream.append_byte(1);
        stream.append_byte(1);
        stream.append_byte(2);
        stream.append_byte(2);
        stream.append_byte(3);
        stream.append_byte(3);
        stream.append_byte(4);
        stream.append_byte(4);
        stream.append_byte(5);
        stream.append_byte(5);
        stream.append_byte(6);
        stream.append_byte(6);

        for _ in 0..7 {
            stream.append_byte(6);
        }
        for _ in 0..4 {
            stream.append_byte(3);
            stream.append_byte(4);
        }
        for _ in 0..2 {
            stream.append_byte(1);
            stream.append_byte(2);
        }
        for _ in 0..5 {
            stream.append_byte(5);
        }

        let (code, map) = encode(&mut stream);

        // code lengths should be 3 for 1, 3 for 2, 3 for 3, 3 for 4, 3 for 5, 2 for 6
        let mut correct_code_len = 3 * 4 + // 4 1s with code length 3
            4 * 4 + // 4 2s with code length 3, longer than optimal because of remove_only_ones_code()
            3 * 6 + // 6 3s with code length 3
            3 * 6 + // 6 4s with code length 4
            2 * 7 + // 7 5s with code length 2
            2 * 9; // 9 6s with code length 2
        correct_code_len = (correct_code_len as f32 / 8 as f32).ceil() as usize;
        assert_eq!(correct_code_len, code.data().len());

        let shortest_code_len = map
            .clone()
            .into_iter()
            .min_by_key(|(_, value)| value.0)
            .unwrap()
            .1
             .0;
        assert_eq!(shortest_code_len, map.get(&6u8).unwrap().0)
    }

    #[test]
    fn test_append_to_map() {
        let mut map = HashMap::new();
        let node = HuffmanNode {
            chance: 1,
            content: Some(1),
            left: None,
            right: None,
        };
        node.append_to_map(&mut map, 2, 3);

        assert_eq!(map.get(&1), Some(&(3, 2)));
    }

    #[test]
    fn test_code_map() {
        let node = HuffmanNode {
            chance: 1,
            content: Some(1),
            left: None,
            right: None,
        };
        let map = node.code_map();

        assert_eq!(map.get(&1), Some(&(0, 0)));
    }

    #[test]
    fn test_increment_or_append() {
        let mut nodes = vec![
            HuffmanNode {
                chance: 1,
                content: Some(1),
                left: None,
                right: None,
            },
            HuffmanNode {
                chance: 2,
                content: Some(2),
                left: None,
                right: None,
            },
        ];
        increment_or_append(&mut nodes, 1);
        increment_or_append(&mut nodes, 3);

        assert_eq!(nodes[0].chance, 2);
        assert_eq!(nodes[1].chance, 2);
        assert_eq!(nodes[2].chance, 1);
        assert_eq!(nodes[2].content, Some(3));
    }

    #[test]
    #[ignore]
    fn test_huge_bit_stream_six_symbols() {
        let mut stream = BitStream::open();
        let mut rng = rand::thread_rng();
        let six_occurence = rng.gen::<u32>();
        for _ in 0..six_occurence {
            stream.append_byte(6);
        }
        let three_four_occurence = rng.gen::<u32>();
        for _ in 0..three_four_occurence {
            stream.append_byte(3);
            stream.append_byte(4);
        }
        let one_two_occurence = rng.gen::<u32>();
        for _ in 0..one_two_occurence {
            stream.append_byte(1);
            stream.append_byte(2);
        }
        let five_occurence = rng.gen::<u32>();
        for _ in 0..five_occurence {
            stream.append_byte(5);
        }
        let tree = parse_u8_stream(&mut stream);
        let (_, map) = encode(&mut stream);
        println!("ones: {}", one_two_occurence);
        println!("twos: {}", one_two_occurence);
        println!("threes: {}", three_four_occurence);
        println!("fours: {}", three_four_occurence);
        println!("fives: {}", five_occurence);
        println!("sixes: {}", six_occurence);
        println!("{:?}", tree);
        println!("{:?}", map);
    }

    #[test]
    #[ignore]
    fn test_huge_bit_stream() {
        let mut stream = BitStream::open();
        let mut rng = rand::thread_rng();
        let amount_of_symbols = rng.gen::<u8>();
        for _ in 0..amount_of_symbols {
            let symbol = rng.gen::<u8>();
            let amount = rng.gen::<u8>();
            for _ in 0..amount {
                stream.append(symbol);
            }
            println!("Number {}: {}", symbol, amount);
        }
        // let tree = parse_u8_stream(&mut stream, true);
        // let (code, map) = encode(&mut stream);
        println!("Amount of symbols: {}", amount_of_symbols);
        // println!("{:?}", tree);
        // println!("{:?}", map);
    }
}
