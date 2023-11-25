use std::{collections::HashMap, mem};
use std::fs;

use criterion::{black_box, Criterion, criterion_group, criterion_main};

// Due to limitations with Criterion, we need to copy/paste bit_stream.rs here.
// We can only use code from src/ if we are creating a library :/

pub trait AppendableToBitStream {
    fn append(&self, stream: &mut BitStream);

    fn append_n_bits(&self, _stream: &mut BitStream, _amount: u8) { panic!("Not implemented for this type!") }
}

impl AppendableToBitStream for bool {
    fn append(&self, stream: &mut BitStream) {
        stream.append_bit(*self);
    }
}

impl AppendableToBitStream for Vec<bool> {
    fn append(&self, stream: &mut BitStream) {
        for val in self {
            stream.append_bit(*val);
        }
    }
}

impl AppendableToBitStream for u8 {
    fn append(&self, stream: &mut BitStream) {
        stream.append_byte(*self);
    }
    fn append_n_bits(&self, stream: &mut BitStream, amount: u8) {
        if amount > 8 { panic!("Not enough bits in value to append") }
        for pos in (0..amount).rev() {
            let i = 0b0000_0001 << pos;
            let bit = self & i != 0;
            stream.append_bit(bit);
        }
    }
}

impl AppendableToBitStream for Vec<u8> {
    fn append(&self, stream: &mut BitStream) {
        for val in self {
            stream.append_byte(*val);
        }
    }

    fn append_n_bits(&self, stream: &mut BitStream, amount: u8) {
        if amount > (self.len() * 8) as u8 { panic!("Not enough bits in value to append") }
        for i in 0..amount {
            let current_val = self[(i / 8) as usize];
            let i = 0b1000_0000 >> i % 8;
            let bit = current_val & i != 0;
            stream.append_bit(bit);
        }
    }
}

impl AppendableToBitStream for u16 {
    fn append(&self, stream: &mut BitStream) {
        let bytes = self.to_be_bytes();
        stream.append_byte(bytes[0]);
        stream.append_byte(bytes[1]);
    }

    fn append_n_bits(&self, stream: &mut BitStream, amount: u8) {
        if amount > 16 { panic!("Not enough bits in value to append") }
        for pos in (0..amount).rev() {
            let i = 0b0000_0000_0000_0001 << pos;
            let bit = self & i != 0;
            stream.append_bit(bit);
        }
    }
}

impl AppendableToBitStream for Vec<u16> {
    fn append(&self, stream: &mut BitStream) {
        for val in self {
            stream.append(*val);
        }
    }

    fn append_n_bits(&self, stream: &mut BitStream, amount: u8) {
        if amount > (self.len() * 16) as u8 { panic!("Not enough bits in value to append") }
        for i in 0..amount {
            let current_val = self[(i / 16) as usize];
            let i = 0b1000_0000_0000_0000 >> i % 16;
            let bit = current_val & i != 0;
            stream.append_bit(bit);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BitStream {
    data: Vec<u8>,
    bits_in_last_byte: u8,
}

impl BitStream {
    /// Open a bit stream.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream::open();
    /// ```
    pub fn open() -> BitStream {
        BitStream {
            ..Default::default()
        }
    }

    /// Create a BitStream object from a file.
    ///
    /// # Arguments
    ///
    /// * filename: The name of the file to write to.
    ///
    /// # Example
    ///
    /// ```
    /// let stream = BitStream::read_bit_stream_from_file(filename);
    /// stream.append_bit(true);
    /// ```
    pub fn read_bit_stream_from_file(filename: &str) -> BitStream {
        let data = fs::read(filename).expect("failed to read file");
        BitStream {
            data,
            bits_in_last_byte: 0,
        }
    }

    /// Append a bit of data to this bit stream.
    ///
    /// # Arguments
    ///
    /// * value: Whether to append a 1 or 0.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream::open();
    /// stream.append_bit(true);
    /// ```
    pub fn append_bit(&mut self, value: bool) {
        if self.bits_in_last_byte == 8 || self.bits_in_last_byte == 0 {
            self.data.push(if value { 0b1000_0000 } else { 0 });
            self.bits_in_last_byte = 1;
            return;
        }
        self.shift_and_add_to_last_byte(u8::from(value), 1);
    }

    /// Append a byte of data to this bit stream.
    /// integers of any size?
    ///
    /// # Arguments
    ///
    /// * value: The data to append.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream.open();
    /// stream.append_byte(244);
    /// ```
    ///
    /// # Explanation
    ///
    /// ## upper_value
    ///
    /// The bits we can append to the last byte in the stream. Cut off the amount of bits already
    ///  occupied in the last byte and move the remaining towards the LSB, then add them to the last byte
    ///
    /// ## lower_value
    ///
    /// The value we have to append as a new byte. Cut off the bits we already appended to the last byte
    /// and move the remaining towards the MSB, then append as a new byte.
    ///
    /// ## General
    ///
    /// * If we have a fully filled byte at the end, we can just push the next to data
    /// * bits_in_last_byte doesn't change as we add a whole byte to the stream. We do need to store and re-set it though,
    ///     as shift_and_add_to_last_byte changes the value of bits_in_last_byte.
    pub fn append_byte(&mut self, value: u8) {
        // if the last byte in the stream is full, we can just append this one
        if self.bits_in_last_byte == 8 || self.bits_in_last_byte == 0 {
            self.data.push(value);
            self.bits_in_last_byte = 8;
            return;
        }

        let previous_bits_in_last_byte = self.bits_in_last_byte;

        let upper_value = value >> self.bits_in_last_byte;
        let bits_still_available_in_last_byte = 8 - self.bits_in_last_byte;
        self.shift_and_add_to_last_byte(upper_value, bits_still_available_in_last_byte);
        let lower_value = value << bits_still_available_in_last_byte;
        self.data.push(lower_value);
        self.bits_in_last_byte = previous_bits_in_last_byte;
    }

    /// Shift the provided value to the correct position, then store it in the last byte.
    /// This should be used to write data to the stream.
    ///
    /// # Arguments
    ///
    /// * `value`: The data to append. Only the first `shift` bits of this should be set.
    /// * `bits_to_occupy`: The amount of bits to add to the last byte.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream::open();
    /// stream.append_byte(0); // necessary because shift_and_add_to_last_byte assumes a byte exists
    /// stream.shift_and_add_to_last_byte(3, 2);
    /// assert_eq!(vec![3], stream.data);
    /// ```
    ///
    /// # Explanation
    /// We shift the value to the correct position given by the available space in the last byte, then add the
    /// resulting byte to the last one and replace it within the vector
    ///
    /// # Panics
    ///
    /// * If more than the last `bits_to_occupy` bits of `value` are set
    fn shift_and_add_to_last_byte(&mut self, mut value: u8, bits_to_occupy: u8) {
        let index = self.data.len() - 1;
        let mut last_byte = self.data[index];
        let bits_available = 8 - bits_to_occupy - self.bits_in_last_byte;
        value = value << bits_available;
        last_byte += value;
        self.data[index] = last_byte;
        self.bits_in_last_byte += bits_to_occupy;
    }

    /// Flush the bit stream to a file.
    ///
    /// # Arguments
    ///
    /// * filename: The name of the file to write to.
    ///
    /// # Example
    ///
    /// ```
    /// let mut stream = BitStream.open();
    /// stream.append_bit(true);
    /// stream.append_bit(false);
    /// stream.flush_to_file("test.bin");
    /// ```
    pub fn flush_to_file(&self, filename: &str) -> std::io::Result<()> {
        fs::write(filename, &self.data)
    }

    pub fn append<T: AppendableToBitStream>(&mut self, value: T) {
        value.append(self);
    }
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
    pub fn bits_in_last_byte(&self) -> u8 {
        self.bits_in_last_byte
    }

    pub fn append_n_bits<T: AppendableToBitStream>(&mut self, value: T, amount: u8) {
        value.append_n_bits(self, amount);
    }
}

impl Default for BitStream {
    fn default() -> BitStream {
        BitStream {
            data: Vec::with_capacity(4096),
            bits_in_last_byte: 0,
        }
    }
}


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
pub fn encode(stream: &mut BitStream) -> (BitStream, HashMap<u8, (u8, u16)>) {
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
    let mut tree = package_merge(stream, 16);

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
    if let Some(node) = nodes.into_iter().find(|n| n.content.unwrap() == value) {
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
    while nodes.len() > 1 {
        nodes.sort_by_key(&mut *sort_lambda);
        let mut bigger_node = nodes.remove(1);
        let smaller_node = nodes.remove(0);
        bigger_node = combine_nodes(bigger_node, smaller_node);
        nodes.push(bigger_node);
    }
    if nodes.len() == 0 {
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
    map: &mut HashMap<u8, (u8, u16)>,
) -> HuffmanNode<u8> {
    let mut root = HuffmanNode::default();
    let mut current = &mut root;
    let mut current_height = 0;
    while nodes.len() > 0 {
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
    pub fn left_unchecked(&self) -> &Box<HuffmanNode<u8>> {
        self.left.as_ref().unwrap()
    }

    /// get an immutable reference to this node's right child.
    ///
    /// # Panics
    /// * if the right child is None.
    pub fn right_unchecked(&self) -> &Box<HuffmanNode<u8>> {
        self.right.as_ref().unwrap()
    }

    /// get a mutable reference to this node's left child.
    ///
    /// # Panics
    /// * if the left child is None.
    pub fn left_unchecked_mut(&mut self) -> &mut Box<HuffmanNode<u8>> {
        self.left.as_mut().unwrap()
    }

    /// get a mutable reference to this node's right child.
    ///
    /// # Panics
    /// * if the right child is None.
    pub fn right_unchecked_mut(&mut self) -> &mut Box<HuffmanNode<u8>> {
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
        let left = match &self.left {
            Some(left) => Some(left.min_depth()),
            None => None,
        };
        let right = match &self.right {
            Some(right) => Some(right.min_depth()),
            None => None,
        };

        if left.is_none() && right.is_none() {
            return 1;
        }

        return 1 + std::cmp::min(
            match left {
                Some(value) => value,
                None => u16::MAX,
            },
            match right {
                Some(value) => value,
                None => u16::MAX,
            },
        );
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
    pub fn code_map(&self) -> HashMap<u8, (u8, u16)> {
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
    fn append_to_map(&self, map: &mut HashMap<u8, (u8, u16)>, code: u16, code_len: u8) {
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
            let left = mem::replace(&mut self.left, None);
            self.right = left;
        } else if self.left.is_some() {
            if self.left_unchecked().max_depth() > self.right_unchecked().min_depth() {
                if self.left_unchecked().min_depth() >= self.right_unchecked().max_depth() {
                    mem::swap(&mut self.right, &mut self.left);
                } else {
                    self.swap_inner_nodes();
                }
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

    /// Remove the lower right leaf (1*) and replace it with a node which has only a leaf on the left
    /// This *CAN* lead to a suboptimal code.
    fn remove_only_ones_code(&mut self) {
        if self.get_or_append_only_ones_code().is_some() {
            panic!("No place found to put the 1* code!")
        }
    }

    /// Recursively iterate through nodes to retrieve the node with the code 1* and put it elsewhere.
    fn get_or_append_only_ones_code(&mut self) -> Option<(HuffmanNode<u8>, u16)> {
        if self.right.is_none() {
            return None;
        }

        if self.right_unchecked().content.is_some() {
            let ones_node = mem::replace(&mut self.right, None).unwrap();
            if self.left.is_none() {
                self.left = Some(Box::from(ones_node.clone_leaf()));
                return None;
            }
            return Some((ones_node.clone_leaf(), 1));
        }

        let ones_node_option = self.right_unchecked_mut().get_or_append_only_ones_code();

        if ones_node_option.is_none() {
            return None;
        }

        let ones_node = ones_node_option.as_ref().unwrap().0.clone_leaf();
        let mut depth = ones_node_option.unwrap().1;
        if self.left.is_none() {
            let _ = mem::replace(&mut self.left.as_mut(), Some(&mut Box::from(ones_node)));
            return None;
        }
        if self.replace_left_leaf_with_root(&ones_node) {
            return None;
        }

        if self.left_unchecked().has_space_at_depth(depth, true) {
            let mut current = self.left_unchecked_mut();
            while depth > 0 {
                if current.right.is_none() {
                    current.right = Some(Box::from(ones_node));
                    return None;
                } else if current.left.is_none() {
                    current.left = Some(Box::from(ones_node));
                    return None;
                } else if depth > 1 {
                    if current.replace_right_leaf_with_root(&ones_node) {
                        return None;
                    } else if current.replace_left_leaf_with_root(&ones_node) {
                        return None;
                    }
                } else if current.right_unchecked().has_space_at_depth(depth, true) {
                    current = current.right_unchecked_mut();
                    // this check is unnecessary as this will always be true, so it's commented out for understandability
                    // } else if current.left_unchecked().has_space_at_depth(depth, true) {
                } else {
                    current = current.left_unchecked_mut();
                }
                depth -= 1;
            }
        }
        return Some((ones_node, depth + 1));
    }

    /// if this node has a right leaf, replace it with a root that in turn has both
    /// the old right leaf and `ones_node` as its left leaf.
    ///
    /// # Arguments
    ///
    /// * `ones_node`: The node to attach.
    fn replace_right_leaf_with_root(&mut self, ones_node: &HuffmanNode<u8>) -> bool {
        if self.right_unchecked().content.is_some() {
            let right = mem::replace(&mut self.right, None);
            self.right = Some(Box::from(HuffmanNode {
                left: Some(Box::from(ones_node.clone_leaf())),
                right: right,
                ..Default::default()
            }));
            return true;
        }
        return false;
    }

    /// if this node has a left leaf, replace it with a root that in turn has both
    /// the old left leaf and `ones_node` as its right leaf.
    ///
    /// # Arguments
    ///
    /// * `ones_node`: The node to attach.
    fn replace_left_leaf_with_root(&mut self, ones_node: &HuffmanNode<u8>) -> bool {
        if self.left_unchecked().content.is_some() {
            let left = mem::replace(&mut self.left, None);
            self.left = Some(Box::from(HuffmanNode {
                right: Some(Box::from(ones_node.clone_leaf())),
                left: left,
                ..Default::default()
            }));
            return true;
        }
        return false;
    }

    /// Checks if the bit stream has space at a given depth.
    ///
    /// This function checks if the bit stream has space at the specified depth. The `depth` parameter
    /// indicates the depth at which the space is being checked. The `leaves_count_as_space` parameter
    /// determines whether the leaves of the bit stream should be considered as space.
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth at which the space is being checked.
    /// * `leaves_count_as_space` - Determines whether the leaves should be considered as space.
    ///
    /// # Returns
    ///
    /// Returns `true` if the bit stream has space at the specified depth, otherwise `false`.
    ///
    /// # Example
    ///
    /// ```
    /// let bit_stream = BitStream::new();
    /// let has_space = bit_stream.has_space_at_depth(2, true);
    /// assert_eq!(has_space, true);
    /// ```
    fn has_space_at_depth(&self, depth: u16, leaves_count_as_space: bool) -> bool {
        if self.content.is_some() {
            return if leaves_count_as_space {
                depth != 0
            } else {
                false
            };
        } else if self.right.is_none() || self.left.is_none() {
            return true;
        } else if depth == 0 {
            return false;
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

pub fn package_merge(stream: &mut BitStream, height: u16) -> HuffmanNode<u8> {
    let mut nodes = get_single_leaves(stream);
    if nodes.len() == 0 {
        return HuffmanNode::default();
    }
    if (nodes.len() as f64).log2().ceil() > height as f64 {
        panic!("Package merge not possible");
    }

    nodes.sort_by_key(|node| node.chance());
    let p = create_p(&mut nodes);

    let mut lookup: HashMap<u8, (u8, u64)> = HashMap::with_capacity(p.len());
    let mut q: Vec<Vec<Vec<(u8, u64)>>> = Vec::with_capacity((height - 1) as usize);
    q.push(vec![]);

    populate_first_q_row(&p, &mut lookup, &mut q);

    calculate_further_q_rows(&mut q, height);

    let l = calculate_code_lengths(q.last().unwrap(), &mut lookup, nodes.len());

    let mut map = map_codes_to_code_length(&p, &l, &lookup, &mut nodes, height);

    nodes.sort_by_key(|node| node.chance());

    code_len_to_tree(&mut nodes, &mut map)
}

//TODO: clean up
pub fn package_merge_experimental(stream: &mut BitStream, height: u16) -> HashMap<u8, (u8, u16)> {
    let mut nodes = get_single_leaves(stream);
    if nodes.len() == 0 {
        panic!("Alarm");
    }
    if (nodes.len() as f64).log2().ceil() > height as f64 {
        panic!("Package merge not possible");
    }

    nodes.sort_by_key(|node| node.chance());
    let p = create_p(&mut nodes);

    let mut lookup: HashMap<u8, (u8, u64)> = HashMap::with_capacity(p.len());
    let mut q: Vec<Vec<Vec<(u8, u64)>>> = Vec::with_capacity((height - 1) as usize);
    q.push(vec![]);

    populate_first_q_row(&p, &mut lookup, &mut q);

    calculate_further_q_rows(&mut q, height);

    let l = calculate_code_lengths(q.last().unwrap(), &mut lookup, nodes.len());

    let mut map = map_codes_to_code_length(&p, &l, &lookup, &mut nodes, height);

    nodes.sort_by_key(|node| node.chance());

    nodes_to_code(&nodes, &mut map, height);
    map
}

fn create_p(nodes: &mut Vec<HuffmanNode<u8>>) -> Vec<(u8, u64)> {
    nodes
        .iter()
        .map(|node| (node.content().unwrap(), node.chance()))
        .collect()
}

fn populate_first_q_row(
    p: &Vec<(u8, u64)>,
    lookup: &mut HashMap<u8, (u8, u64)>,
    q: &mut Vec<Vec<Vec<(u8, u64)>>>,
) {
    let mut index = 0;

    for i in p {
        lookup.insert(i.0, (index, i.1));
        q[0].push(vec![*i]);
        index += 1;
    }
}

fn calculate_further_q_rows(q: &mut Vec<Vec<Vec<(u8, u64)>>>, height: u16) {
    let mut q_0 = q[0].clone();

    for i in 0..(height - 1) as usize {
        let next = package(&mut q[i], &mut q_0);
        q.push(next);
    }
}

fn package(q: &mut Vec<Vec<(u8, u64)>>, q_0: &mut Vec<Vec<(u8, u64)>>) -> Vec<Vec<(u8, u64)>> {
    let mut next_row = q_0.clone();
    for i in (0..q.len() - q.len() % 2).step_by(2) {
        let mut node: Vec<(u8, u64)> = vec![];
        let mut left: Vec<(u8, u64)> = q[i].clone();
        node.append(&mut left);
        let mut right: Vec<(u8, u64)> = q[i + 1].clone();
        node.append(&mut right);
        next_row.push(node);
    }
    next_row.sort_by_key(|nodes| {
        let mut x = 0;
        nodes.iter().for_each(|n| x += n.1);
        x
    });
    next_row
}

fn calculate_code_lengths(
    q: &Vec<Vec<(u8, u64)>>,
    lookup: &mut HashMap<u8, (u8, u64)>,
    number_of_nodes: usize,
) -> Vec<u64> {
    if number_of_nodes == 1 {
        return vec![1u64];
    }
    let mut l = vec![0u64; number_of_nodes];
    for node in &q[0..(2 * number_of_nodes - 2)] {
        for entry in node {
            let index = lookup.get(&entry.0).unwrap().0 as usize;
            l[index] += 1;
        }
    }
    l
}

fn map_codes_to_code_length(
    p: &Vec<(u8, u64)>,
    l: &Vec<u64>,
    lookup: &HashMap<u8, (u8, u64)>,
    nodes: &mut Vec<HuffmanNode<u8>>,
    height: u16,
) -> HashMap<u8, (u8, u16)> {
    let mut map: HashMap<u8, (u8, u16)> = HashMap::with_capacity(p.len());
    for (i, el) in p.iter().enumerate() {
        let code_length = l[lookup.get(&el.0).unwrap().0 as usize];
        if code_length > height as u64 {
            panic!("Something went wrong, code length bigger than height");
        }
        map.insert(el.0, (code_length as u8, 0));
        nodes[i].set_chance(u64::MAX - code_length);
    }
    map
}

fn nodes_to_code(nodes: &Vec<HuffmanNode<u8>>, map: &mut HashMap<u8, (u8, u16)>, height: u16) {
    if 2_i32.pow(height as u32) == nodes.len() as i32 { panic!("Avoiding 1* not possible") }
    let mut current_code = 0;
    let mut start = true;
    // We iterate from shortest to longest code
    for (i, node) in nodes.iter().rev().enumerate() {
        let val = &node.content.unwrap();
        let mut next_node_code_length: u8 = 0;
        let (mut code_length, _) = *map.get(val).unwrap();
        if { i < nodes.len() - 1 } {
            let key = &nodes[nodes.len() - i - 2].content.unwrap();
            next_node_code_length = map.get(key).unwrap().0;
        } else {
            next_node_code_length = 0;
        }
        // If we're on the edge to the next code length, smooth out the transition by incrementing the
        // current code_length and incrementing and shifting the current_code, if not 0
        if code_length != next_node_code_length && next_node_code_length != 0 {
            code_length += 1;
            if !start {
                current_code += 1;
                current_code <<= 1;
            }
            start = false;
            // If the code_length doesn't change, just increment the code
        } else if !start {
            current_code += 1;
        } else {
            start = false;
        }
        // update the map
        map.insert(*val, (code_length, current_code));
        // println!("value: {}, current code:{:08b}, code length: {}", *val, current_code, code_length);
    }
}

pub fn criterion_bit_benchmark(c: &mut Criterion) {
    c.bench_function("Test append_bit", |b| {
        b.iter(|| {
            let mut stream = BitStream::open();
            for _ in 0..10_000_000 {
                stream.append_bit(black_box(true));
            }
        })
    });
}

pub fn criterion_byte_benchmark(c: &mut Criterion) {
    c.bench_function("Test append_byte", |b| {
        b.iter(|| {
            let mut stream = BitStream::open();
            stream.append_bit(black_box(true));
            stream.append_bit(black_box(true));
            for _ in 0..10_000_000 {
                stream.append_byte(black_box(170));
            }
        })
    });
}

pub fn criterion_byte_and_write_benchmark(c: &mut Criterion) {
    c.bench_function("Test append_byte and flush to file", |b| {
        b.iter(|| {
            let mut stream = BitStream::open();
            stream.append_bit(black_box(true));
            stream.append_bit(black_box(true));
            for _ in 0..10_000_000 {
                stream.append_byte(black_box(170));
            }
            stream.flush_to_file(black_box("test/test.bin")).expect("bit stream could not be flushed to file");
        })
    });
    fs::remove_file("test/test.bin").expect("file could not be removed");
}

pub fn criterion_read_benchmark(c: &mut Criterion) {
    let mut stream = BitStream::open();
    for _ in 0..10_000_000 {
        stream.append_byte(170);
    }
    stream.flush_to_file(black_box("test/test.bin")).expect("bit stream could not be flushed to file");
    c.bench_function("Test reading bitstream from file", |b| {
        b.iter(|| {
            let mut read_stream = BitStream::read_bit_stream_from_file(black_box("test/test.bin"));
            read_stream.append(black_box(true));
        })
    });
    fs::remove_file("test/test.bin").expect("file could not be removed");
}

pub fn criterion_read_and_write_benchmark(c: &mut Criterion) {
    let mut stream = BitStream::open();
    for _ in 0..10_000_000 {
        stream.append_byte(170);
    }
    stream.flush_to_file(black_box("test/test.bin")).expect("bit stream could not be flushed to file");
    c.bench_function("Test reading and writing bitstream from/to file", |b| {
        b.iter(|| {
            let mut read_stream = BitStream::read_bit_stream_from_file(black_box("test/test.bin"));
            for _ in 0..10_000_000 {
                read_stream.append_bit(false);
                read_stream.append_byte(black_box(170));
            }
            stream.flush_to_file(black_box("test/test.bin")).expect("bit stream could not be flushed to file");
        })
    });
    fs::remove_file("test/test.bin").expect("file could not be removed");
}

pub fn criterion_huffman_encoding_benchmark(c: &mut Criterion) {
    let mut stream = BitStream::open();
    let mut j = 0;
    for i in 0..240 {
        for _ in 0..(100000 + j) {
            stream.append_byte(i);
        }
        j += 1;
    }
    c.bench_function("Benchmark huffman encoding", |b| {
        b.iter(|| {
            package_merge(&mut stream, 8);
        })
    });
}

pub fn criterion_huffman_encoding_benchmark_experimental(c: &mut Criterion) {
    let mut stream = BitStream::open();
    let mut j = 0;
    for i in 0..240 {
        for _ in 0..(100000 + j) {
            stream.append_byte(i);
        }
        j += 1;
    }
    c.bench_function("Benchmark huffman encoding experimental", |b| {
        b.iter(|| {
            package_merge_experimental(&mut stream, 8);
        })
    });
}

// criterion_group!(benches, criterion_bit_benchmark, criterion_byte_benchmark, criterion_byte_and_write_benchmark, criterion_read_benchmark, criterion_read_and_write_benchmark, criterion_huffman_encoding_benchmark);
criterion_group!(benches,criterion_huffman_encoding_benchmark, criterion_huffman_encoding_benchmark_experimental);
criterion_main!(benches);