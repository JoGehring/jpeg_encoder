use std::{collections::HashMap, mem};
use std::fs;

use criterion::{black_box, Criterion, criterion_group, criterion_main};
use rand::Rng;

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

pub fn code_len_to_tree(nodes: &mut Vec<HuffmanNode<u8>>, map: &mut HashMap<u8, (u8, u16)>) -> HuffmanNode<u8> {
    let mut root = HuffmanNode::default();
    let mut current = &mut root;
    let mut current_height = 0;
    while nodes.len() > 0 {
        let leaf = nodes.remove(0);
        let destination = map.get(&leaf.content().unwrap()).unwrap().0 - 1;
        while current_height < destination {
            if current.right().is_none() && current.left().is_none() {
                current.right = Some(Box::from(HuffmanNode::default()));
                current = current.right.as_mut().unwrap();
            } else if current.right().is_some() && current.right().as_ref().unwrap().has_space_at_depth((destination - current_height - 1) as u16) {
                current = current.right.as_mut().unwrap();
            } else if current.left().is_some() && current.left().as_ref().unwrap().has_space_at_depth((destination - current_height - 1) as u16) {
                current = current.left.as_mut().unwrap();
            } else if current.left().is_none() {
                current.left = Some(Box::from(HuffmanNode::default()));
                current = current.left.as_mut().unwrap();
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
            // println!("{:?}", root);
            panic!("Leaf error");
        }
        current = &mut root;
        current_height = 0;
    }
    root
}

impl HuffmanNode<u8> {
    /// Calculate the chance/frequency for all symbols in this node and its child nodes.
    pub(crate) fn chance(&self) -> u64 {
        let mut result = self.chance;
        if self.left.is_some() {
            result += self.left.as_ref().unwrap().chance();
        }
        if self.right.is_some() {
            result += self.right.as_ref().unwrap().chance();
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
            self.left
                .as_ref()
                .unwrap()
                .append_to_map(map, code << 1, code_len + 1);
        }
        if self.right.is_some() {
            self.right
                .as_ref()
                .unwrap()
                .append_to_map(map, (code << 1) + 1, code_len + 1);
        }
    }

    /// Create a code from this tree. The result is a BitStream
    /// containing the values.
    pub fn create_code(&self) -> BitStream {
        let mut stream = BitStream::open();
        self.append_to_code(&mut stream, 0, 0);
        stream
    }

    /// Append this node's data to the stream. Then recursively call
    /// child nodes to append their data.
    ///
    /// # Arguments
    ///
    /// * `stream`: The stream to append codes to.
    /// * `code`: The code bits for this node.
    /// * `code_len`: The length of the code for this node.
    fn append_to_code(&self, stream: &mut BitStream, code: u16, code_len: u8) {
        if self.content.is_some() {
            stream.append_n_bits(code, code_len);
        }
        if self.left.is_some() {
            self.left
                .as_ref()
                .unwrap()
                .append_to_code(stream, code << 1, code_len + 1);
        }
        if self.right.is_some() {
            self.right
                .as_ref()
                .unwrap()
                .append_to_code(stream, (code << 1) + 1, code_len + 1);
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
            if self.left.as_ref().unwrap().max_depth() > self.right.as_ref().unwrap().min_depth() {
                if self.left.as_ref().unwrap().min_depth()
                    >= self.right.as_ref().unwrap().max_depth()
                {
                    mem::swap(&mut self.right, &mut self.left);
                } else {
                    self.swap_inner_nodes();
                }
            }
        }

        if self.left.is_some() {
            self.left.as_mut().unwrap().ensure_tree_grows_right();
        }
        if self.right.is_some() {
            self.right.as_mut().unwrap().ensure_tree_grows_right();
        }
    }

    /// swap this node's right child's left child with the left child's right child node.
    /// If this node's left child's max depth is greated than this node's right child's, swap them too.
    fn swap_inner_nodes(&mut self) {
        if self.left.as_ref().unwrap().max_depth() > self.right.as_ref().unwrap().max_depth() {
            mem::swap(&mut self.right, &mut self.left);
        }
        mem::swap(
            &mut self.right.as_mut().unwrap().left,
            &mut self.left.as_mut().unwrap().right,
        );
    }


    fn has_space_at_depth(&self, depth: u16) -> bool {
        if self.content.is_some() {
            return false;
        } else if self.right.is_none() || self.left.is_none() {
            return true;
        } else if depth == 0 {
            return false;
        } else {
            return self.left.as_ref().unwrap().has_space_at_depth(depth - 1) || self.right.as_ref().unwrap().has_space_at_depth(depth - 1);
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
    let p: Vec<(u8, u64)> = nodes
        .iter()
        .map(|node| (node.content().unwrap(), node.chance()))
        .collect();
    let mut lookup: HashMap<u8, (u8, u64)> = HashMap::with_capacity(p.len());
    let mut l = vec![0u64; nodes.len()];
    let mut q: Vec<Vec<Vec<(u8, u64)>>> = Vec::with_capacity((height - 1) as usize);
    q.push(vec![]);
    let mut index = 0;

    for i in &p {
        lookup.insert(i.0, (index, i.1));
        q[0].push(vec![*i]);
        index += 1;
    }
    index = 0;
    let mut q_0 = q[0].clone();
    while q[index as usize].len() < (2 * p.len() - 2) {
        let next = package(&mut q[index as usize], &mut q_0);
        q.push(next);
        index += 1;
    }
    for node in &q[q.len() - 1] {
        for entry in node {
            let index = lookup.get(&entry.0).unwrap().0;
            l[index as usize] += 1;
        }
    }
    let mut map: HashMap<u8, (u8, u16)> = HashMap::with_capacity(p.len());
    for (i, el) in p.iter().enumerate() {
        let code_length = l[lookup.get(&el.0).unwrap().0 as usize];
        if code_length > height as u64 {
            panic!("Something went wrong, code length bigger than height");
        }
        map.insert(el.0, (code_length as u8, 0));
        nodes[i].set_chance(u64::MAX - code_length);
    }
    nodes.sort_by_key(|node| node.chance());

    code_len_to_tree(&mut nodes, &mut map)
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
    let mut rng = rand::thread_rng();
    for i in 0..255 {
        for _ in 0..rng.gen_range(2048..1048576) {
            stream.append_byte(i);
        }
    }
    c.bench_function("Benchmark huffman encoding", |b| {
        b.iter(|| {
            package_merge(&mut stream, 8);
        })
    });
}

criterion_group!(benches, criterion_bit_benchmark, criterion_byte_benchmark, criterion_byte_and_write_benchmark, criterion_read_benchmark, criterion_read_and_write_benchmark, criterion_huffman_encoding_benchmark);
criterion_main!(benches);