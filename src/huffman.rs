use crate::bit_stream::BitStream;

#[derive(Debug, PartialEq)]
pub struct HuffmanNode<T: PartialEq> {
    chance: u16,
    content: Option<T>,
    left: Option<Box<HuffmanNode<T>>>,
    right: Option<Box<HuffmanNode<T>>>,
}

/// Parse a stream of u8 values and create a huffman tree for them.
/// The tree grows to the right, meaning no left node ever has a bigger max_path() than the corresponding
/// right node's max_path().
///
/// # Arguments
///
/// * stream: The stream of data to read.
pub fn parse_u8_stream(stream: &mut BitStream) -> HuffmanNode<u8> {
    let mut nodes: Vec<HuffmanNode<u8>> = vec![];
    while let Some(byte) = stream.pop_first_byte() {
        increment_or_append(&mut nodes, byte);
    }

    while nodes.len() > 1 {
        nodes.sort_by_key(|node| node.chance());
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
    if higher_chance_node.min_depth() >= lower_chance_node.max_depth() {
        return HuffmanNode {
            left: Some(Box::from(lower_chance_node)),
            right: Some(Box::from(higher_chance_node)),
            ..Default::default()
        };
    } else if higher_chance_node.max_depth() <= lower_chance_node.min_depth() {
        return HuffmanNode {
            left: Some(Box::from(higher_chance_node)),
            right: Some(Box::from(lower_chance_node)),
            ..Default::default()
        };
    } else {
        return combine_and_swap_inner_nodes(higher_chance_node, lower_chance_node);
    }
}

/// Make the given two nodes leaves of a new node.
/// To satisfy the constraint that trees must grow to the right,
/// swap the right node's left child and the left node's right child.
///
/// # Arguments
///
/// * node_1: One of the nodes to combine.
/// * node_2: One of the nodes to combine.
fn combine_and_swap_inner_nodes(
    node_1: HuffmanNode<u8>,
    node_2: HuffmanNode<u8>,
) -> HuffmanNode<u8> {
    let mut right;
    let mut left;
    if node_1.max_depth() > node_2.max_depth() {
        right = node_1;
        left = node_2;
    } else {
        left = node_1;
        right = node_2;
    }

    let right_tree_left_child = right.left;
    right.left = left.right;
    left.right = right_tree_left_child;

    return HuffmanNode {
        left: Some(Box::from(left)),
        right: Some(Box::from(right)),
        ..Default::default()
    };
}

impl HuffmanNode<u8> {
    /// Calculate the chance/frequency for all symbols in this node and its child nodes.
    fn chance(&self) -> u16 {
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

    /// Create a code from this tree. The result is a BitStream
    /// containing the values.
    fn create_code(&self) -> BitStream {
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
            println!("===============================");
            println!("{:?}", code);
            println!("{:?}", code_len);
            stream.append_n_bits(code, code_len);
            println!("{:?}", stream.data());
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

// TODO: remove or improve
// impl fmt::Debug for HuffmanNode<u8> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Content: {:?}\n", self.content).expect("panic message");
//         write!(f, "Chance: {:?}\n", self.chance).expect("panic message");
//         for i in 0..4 {
//             write!(f, "|\n").expect("panic message");
//         }
//         for i in 0..4 {
//             write!(f, "-").expect("panic message");
//         }
//         write!(f, "Left: {:?}", self.left);
//
//         for i in 0..4 {
//             write!(f, "|\n").expect("panic message");
//         }
//         for i in 0..4 {
//             write!(f, "-").expect("panic message");
//         }
//
//         write!(f, "Right: {:?}", self.right)
//     }
// }

#[cfg(test)]
mod tests {
    use crate::bit_stream::BitStream;

    use super::{parse_u8_stream, HuffmanNode};

    //TODO: test append, create_code

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
                chance: 1,
                content: Some(1),
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

        for _ in 1..2 {
            stream.append_byte(1);
            stream.append_byte(2);
        }
        for _ in 1..4 {
            stream.append_byte(3);
            stream.append_byte(4);
        }
        for _ in 1..5 {
            stream.append_byte(5);
        }
        for _ in 1..7 {
            stream.append_byte(6);
        }

        let tree = parse_u8_stream(&mut stream);

        let code = tree.create_code();

        let correct_code: Vec<u8> = vec![0b00_01_100_1, 0b01_110_111];
        assert_eq!(&correct_code, code.data());
    }
}
