use std::{collections::HashMap, fmt, mem};

use debug_tree::{add_branch, add_leaf, defer_print};

use crate::bit_stream::BitStream;

#[derive(PartialEq)]
pub struct HuffmanNode<T: PartialEq> {
    chance: u64,
    content: Option<T>,
    left: Option<Box<HuffmanNode<T>>>,
    right: Option<Box<HuffmanNode<T>>>,
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
/// The tree grows to the right, meaning no left node ever has a bigger max_path() than the corresponding
/// right node's max_path().
///
/// # Arguments
///
/// * stream: The stream of data to read.
pub fn parse_u8_stream(stream: &mut BitStream) -> HuffmanNode<u8> {
    let mut nodes = get_single_leaves(stream);

    let mut tree = build_huffman_tree(&mut nodes);

    tree.restrict_height(16);
    // TODO right growing constraint here
    tree.ensure_tree_grows_right();
    tree.remove_only_ones_code();

    tree
}

fn get_single_leaves(stream: &mut BitStream) -> Vec<HuffmanNode<u8>> {
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

fn build_huffman_tree(nodes: &mut Vec<HuffmanNode<u8>>) -> HuffmanNode<u8> {
    let mut sort_lambda = |node: &HuffmanNode<u8>| node.chance();
    build_tree(nodes, &mut sort_lambda)
}

fn build_binary_tree(nodes: &mut Vec<HuffmanNode<u8>>) -> HuffmanNode<u8> {
    let mut sort_lambda = |node: &HuffmanNode<u8>| node.min_depth();
    build_tree(nodes, &mut sort_lambda)
}

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
///
/// # Arguments
///
/// * higher_chance_node: The node with the bigger chance/frequency of symbols in it appearing.
/// * lower_chance_node: The node with the smaller chance/frequency of symbols in it appearing.
///
/// # Explanation
///
/// If the condition of "one node's maximum depth <= the other's minimum depth" is already fulfilled,
/// we can simply append the nodes to a new node and return that.
///
/// The only case in which this condition doesn't apply, assuming both incoming nodes are right growing trees,
/// is if the right child of the node with the lower maximum depth is bigger than the left child of the other
/// node. The solution is thus to swap these child nodes and then append the two nodes to a parent node, which
/// is done in combine_and_swap_inner_nodes.
fn combine_nodes(
    higher_chance_node: HuffmanNode<u8>,
    lower_chance_node: HuffmanNode<u8>,
) -> HuffmanNode<u8> {
    HuffmanNode {
        left: Some(Box::from(lower_chance_node)),
        right: Some(Box::from(higher_chance_node)),
        ..Default::default()
    }
}

impl HuffmanNode<u8> {
    /// Calculate the chance/frequency for all symbols in this node and its child nodes.
    fn chance(&self) -> u64 {
        let mut result = self.chance;
        if self.left.is_some() {
            result += self.left.as_ref().unwrap().chance();
        }
        if self.right.is_some() {
            result += self.right.as_ref().unwrap().chance();
        }
        result
    }

    /// Get the maximum depth (i.e. the maximum possible amount of nodes to go through before arriving at a leaf)
    /// of this node.
    /// Leaves are counted too, so if this node is a leaf, this function returns 1.
    fn max_depth(&self) -> u16 {
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
    fn min_depth(&self) -> u16 {
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

    fn swap_inner_nodes(&mut self) {
        if self.left.as_ref().unwrap().max_depth() > self.right.as_ref().unwrap().max_depth() {
            mem::swap(&mut self.right, &mut self.left);
        }
        mem::swap(&mut self.right.as_mut().unwrap().left, &mut self.left.as_mut().unwrap().right);
    }

    /// Remove the lower right leaf (1*) and replace it with a node which has only a leaf on the left
    /// TODO: different implementation that doesn't break height constraint
    fn remove_only_ones_code(&mut self) {
        let mut current = self;
        while current.left.is_some() && current.right.is_some() {
            current = current.right.as_mut().unwrap();
        }
        let new_left_node = HuffmanNode {
            chance: current.chance,
            content: current.content,
            left: None,
            right: None,
        };
        current.content = None;
        current.chance = 0;
        current.left = Some(Box::from(new_left_node));
    }

    pub fn restrict_height(&mut self, height: u16) {
        if self.max_depth() - 1 <= height {
            return;
        }
        if height < ((self.count_leaves() as f64).log2().ceil()) as u16 {
            panic!("Restriction to this height not possible");
        }

        let mut leaves: Vec<HuffmanNode<u8>> = vec![];
        trim_tree(self, &mut leaves, 1, height - 1);

        self.fill_empty_nodes(&mut leaves);
        let t2 = build_binary_tree(&mut leaves);

        let level = height - (t2.max_depth() - 1) - 1;

        if level <= 0 {
            let mut t1 = HuffmanNode::default();
            let left = mem::replace(&mut self.left, None);
            t1.left = left;
            let right = mem::replace(&mut self.right, None);
            t1.right = right;

            self.left = Some(Box::from(t2));
            self.right = Some(Box::from(t1));
            return;
        }

        let mut y_p = self;
        let mut left_chance: u64;
        let mut right_chance: u64;
        let mut y_new = HuffmanNode::default();

        for l in 0..level - 1 {
            left_chance = determine_child_chance(&y_p.left, level, l);
            right_chance = determine_child_chance(&y_p.left, level, l);

            if left_chance <= right_chance {
                y_p = y_p.left.as_mut().unwrap();
            } else {
                y_p = y_p.right.as_mut().unwrap();
            }
        }

        left_chance = determine_child_chance(&y_p.left, 0, 0);
        right_chance = determine_child_chance(&y_p.left, 0, 0);

        if left_chance <= right_chance {
            let y_p_left = mem::replace(&mut y_p.left, None);
            y_new.left = y_p_left;
            y_new.right = Some(Box::from(t2));
            y_p.left = Some(Box::from(y_new));
        } else {
            let y_p_right = mem::replace(&mut y_p.right, None);
            y_new.right = y_p_right;
            y_new.left = Some(Box::from(t2));

            y_p.right = Some(Box::from(y_new));
        }

        // println!("Current: {:?}", y_p);
    }

    pub fn count_leaves(&self) -> usize {
        if self.content.is_some() {
            return 1;
        } else {
            return match &self.left {
                Some(left) => left.count_leaves(),
                None => 0,
            } + match &self.right {
                Some(right) => right.count_leaves(),
                None => 0,
            };
        }
    }

    fn fill_empty_nodes(&mut self, leaves: &mut Vec<HuffmanNode<u8>>) {
        if leaves.len() == 0 {
            return;
        }
        if self.content.is_none() && self.left.is_none() && self.right.is_none() {
            let _ = mem::replace(self, leaves.pop().unwrap());
            return;
        }
        if self.left.as_ref().is_some() {
            self.left.as_mut().unwrap().fill_empty_nodes(leaves);
        }
        if self.right.as_ref().is_some() {
            self.right.as_mut().unwrap().fill_empty_nodes(leaves);
        }
    }
}

fn trim_tree(
    current: &mut HuffmanNode<u8>,
    leaves: &mut Vec<HuffmanNode<u8>>,
    current_height: u16,
    height: u16,
) {
    if current.content.is_some() {
        if current_height > height {
            leaves.push(current.clone_leaf());
        }
        return;
    }

    trim_child_node(&mut current.left, leaves, current_height, height);
    trim_child_node(&mut current.right, leaves, current_height, height);
}

fn trim_child_node(
    child: &mut Option<Box<HuffmanNode<u8>>>,
    leaves: &mut Vec<HuffmanNode<u8>>,
    current_height: u16,
    height: u16,
) {
    if child.is_some() && current_height + 1 > height {
        let partial_tree = mem::replace(&mut child.as_mut(), None).unwrap();
        get_leaves_from_partial_tree(&partial_tree, leaves);
    } else if child.is_some() {
        trim_tree(child.as_mut().unwrap(), leaves, current_height + 1, height);
    }
}

fn get_leaves_from_partial_tree(
    partial_tree: &Box<HuffmanNode<u8>>,
    leaves: &mut Vec<HuffmanNode<u8>>,
) {
    if partial_tree.content.is_some() {
        leaves.push(partial_tree.clone_leaf());
        return;
    }
    if partial_tree.left.is_some() {
        get_leaves_from_partial_tree(partial_tree.left.as_ref().unwrap(), leaves);
    }
    if partial_tree.right.is_some() {
        get_leaves_from_partial_tree(partial_tree.right.as_ref().unwrap(), leaves);
    }
}

fn determine_child_chance(child: &Option<Box<HuffmanNode<u8>>>, level: u16, l: u16) -> u64 {
    if child.is_some() && child.as_ref().unwrap().max_depth() >= level - l {
        return child.as_ref().unwrap().chance();
    }
    u64::MAX
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
        write!(
            f,
            "=========================================================\n"
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
            build_debug_tree(current.left.as_ref().unwrap(), true);
        }
        if current.right.is_some() {
            build_debug_tree(current.right.as_ref().unwrap(), false);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rand::Rng;

    use crate::{bit_stream::BitStream, huffman::increment_or_append};

    use super::{encode, parse_u8_stream, HuffmanNode};

    //TODO: test create_map, encode
    #[test]
    fn test_parse_empty_stream() {
        let mut stream = BitStream::open();
        let tree = parse_u8_stream(&mut stream);
        assert_eq!(HuffmanNode::default(), tree);
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
                left: Some(Box::from(HuffmanNode {
                    chance: 1,
                    content: Some(1),
                    ..Default::default()
                })),
                ..Default::default()
            },
            tree
        );
    }

    #[test]
    fn test_parse_bigger_stream() {
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

        for _ in 0..2 {
            stream.append_byte(1);
            stream.append_byte(2);
        }
        for _ in 0..4 {
            stream.append_byte(3);
            stream.append_byte(4);
        }
        for _ in 0..5 {
            stream.append_byte(5);
        }
        for _ in 0..7 {
            stream.append_byte(6);
        }

        let tree = parse_u8_stream(&mut stream);
        let code = tree.create_code();
        let correct_code: Vec<u8> = vec![0b00_01_100_1, 0b01_110_111, 0b0_0000000];
        assert_eq!(&correct_code, code.data());
    }

    #[test]
    fn test_tree_growing() {
        let expected_tree = HuffmanNode {
            chance: 0,
            content: None,
            left: None,
            right: None,
        };
        let mut stream = BitStream::open();

        let actual_tree = parse_u8_stream(&mut stream);
        assert_eq!(expected_tree, actual_tree)
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
        let correct_code: Vec<u8> = vec![
            0b110_110_11,
            0b10_1110_10,
            0b0_100_101_1,
            0b01_01_01_00,
            0b00_00_00_00,
            0b00_00_00_00,
            0b100_101_10,
            0b0_101_100_1,
            0b01_100_101,
            0b110_1110_1,
            0b10_1110_01,
            0b01_01_01_01,
        ];
        assert_eq!(correct_code, *code.data());
        let mut correct_map: HashMap<u8, (u8, u16)> = HashMap::with_capacity(6);
        correct_map.insert(4, (3, 0b101));
        correct_map.insert(5, (2, 0b01));
        correct_map.insert(2, (4, 0b1110));
        correct_map.insert(6, (2, 0b00));
        correct_map.insert(3, (3, 0b100));
        correct_map.insert(1, (3, 0b110));
        assert_eq!(correct_map, map);
    }

    #[test]
    fn test_append_to_code() {
        let mut stream = BitStream::open();
        let node = HuffmanNode {
            chance: 1,
            content: Some(1),
            left: None,
            right: None,
        };
        node.append_to_code(&mut stream, 2, 3);

        assert_eq!(stream.data(), &vec![64]);
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
    fn test_create_code() {
        let node = HuffmanNode {
            chance: 1,
            content: Some(1),
            left: None,
            right: None,
        };
        let stream = node.create_code();

        let data: Vec<u8> = vec![];
        assert_eq!(stream.data(), &data);
    }

    #[test]
    fn test_create_code_with_left_child() {
        let left_child = Box::new(HuffmanNode {
            chance: 1,
            content: Some(2),
            left: None,
            right: None,
        });
        let node = HuffmanNode {
            chance: 2,
            content: Some(1),
            left: None,
            right: Some(left_child),
        };
        let stream = node.create_code();

        assert_eq!(stream.data(), &vec![128]);
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
