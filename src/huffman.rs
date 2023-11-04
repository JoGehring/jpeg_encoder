use crate::bit_stream::BitStream;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct HuffmanNode<T: PartialEq> {
    chance: u16,
    max_depth: u16,
    content: Option<T>,
    left: Option<Box<HuffmanNode<T>>>,
    right: Option<Box<HuffmanNode<T>>>,
}

/// Parse a stream of u8 values and create a huffman tree for them.
/// Each non-leaf node will always have a leaf on the left and a non-leaf node
/// or none on its right.
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
            max_depth: 1,
            content: Some(value),
            left: None,
            right: None,
        })
    }
}

/// Combine two nodes and make them leaves of a new node (or append one to the other).
/// This also keeps the structure outlined in parse_u8_stream:
/// Each non-leaf node will always have a leaf on the left and a non-leaf node
/// or none on its right.
/// 
/// # Arguments
/// 
/// * bigger: The node with the bigger chance/frequency of symbols in it appearing.
/// * smaller: The node with the smaller chance/frequency of symbols in it appearing.
fn combine_nodes(bigger: HuffmanNode<u8>, smaller: HuffmanNode<u8>) -> HuffmanNode<u8> {
    let mut bigger_tree: HuffmanNode<u8> = HuffmanNode::default();
    if bigger.content.is_some() {
        bigger_tree.left = Some(Box::from(bigger));
    } else {
        bigger_tree = bigger;
    }

    let smaller_tree;
    if smaller.content.is_some() {
        smaller_tree = HuffmanNode {
            left: Some(Box::from(smaller)),
            ..Default::default()
        }
    } else {
        smaller_tree = smaller;
    }

    if bigger_tree.right.is_none() {
        bigger_tree.right = Some(Box::from(smaller_tree));
    } else {
        let mut node_to_append_to = &mut bigger_tree.right;
        while node_to_append_to
            .as_ref()
            .is_some_and(|n| n.right.is_some())
        {
            let new_right_node = node_to_append_to.as_mut().unwrap();
            node_to_append_to = &mut (new_right_node.right);
        }
        node_to_append_to.as_mut().unwrap().right = Some(Box::from(smaller_tree));
    }

    return bigger_tree;
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

    /// Create a code map from this tree. The result is a HashMap
    /// with the values as keys and the codewords as values.
    /// 
    /// This will need to be altered if trees don't strictly follow the
    /// "Leaves to the left, extending to the right" structure.
    fn create_code(&self) -> HashMap<u8, u32> {
        let mut map = HashMap::with_capacity(self.max_depth as usize);
        if self.content.is_some() {
            map.insert(self.content.unwrap(), 0);
            return map;
        }

        let mut node = self;
        if self.left.is_some() {
            map.insert(self.left.as_ref().unwrap().content.unwrap(), 0);
        }

        let mut code = 1;
        while node.right.is_some() {
            node = node.right.as_ref().unwrap();
            if node.left.is_some() {
                map.insert(node.left.as_ref().unwrap().content.unwrap(), code << 1);
            }
            // increment code as depth increments
            code = (code << 1) + 1;
        }

        map
    }
}

impl Default for HuffmanNode<u8> {
    fn default() -> Self {
        HuffmanNode {
            chance: 0,
            max_depth: 1,
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
    use std::collections::HashMap;

    use crate::bit_stream::BitStream;

    use super::{parse_u8_stream, HuffmanNode};

    //TODO: test append, create_code

    #[test]
    fn test_parse_empty_stream() {
        let mut stream = BitStream::open();
        let tree = parse_u8_stream(&mut stream);
        assert_eq!(HuffmanNode::default(), tree);
        let code = tree.create_code();
        let correct_code: HashMap<u8, u32> = HashMap::new();
        assert_eq!(correct_code, code);
    }

    #[test]
    fn test_parse_stream_with_one_byte() {
        let mut stream = BitStream::open();
        stream.append_byte(1);
        let tree = parse_u8_stream(&mut stream);
        assert_eq!(
            HuffmanNode {
                chance: 1,
                max_depth: 1,
                content: Some(1),
                ..Default::default()
            },
            tree
        );
        let code = tree.create_code();
        let mut correct_code: HashMap<u8, u32> = HashMap::new();
        correct_code.insert(1u8, 0b0);
        assert_eq!(correct_code, code);
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

        let mut correct_code: HashMap<u8, u32> = HashMap::new();
        correct_code.insert(5u8, 0b111110);
        correct_code.insert(1u8, 0b11110);
        correct_code.insert(2u8, 0b1110);
        correct_code.insert(6u8, 0b110);
        correct_code.insert(3u8, 0b10);
        correct_code.insert(4u8, 0b0);
        assert_eq!(correct_code, code);
    }
}
