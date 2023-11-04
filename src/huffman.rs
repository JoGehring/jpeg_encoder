use crate::bit_stream::BitStream;

#[derive(Debug)]
pub struct HuffmanNode<T: PartialEq> {
    chance: u16,
    max_depth: u16,
    content: Option<T>,
    left: Option<Box<HuffmanNode<T>>>,
    right: Option<Box<HuffmanNode<T>>>,
}

//TODO: doc comments

pub fn parse_u8_stream(stream: &mut BitStream) -> HuffmanNode<u8> {
    let mut nodes: Vec<HuffmanNode<u8>> = vec![];
    while let Some(byte) = stream.pop_first_byte() {
        increment_or_append(&mut nodes, byte);
    }
    while nodes.len() > 1 {
        nodes.sort_by_key(|node| node.chance());
        let mut bigger_node = nodes.remove(1);
        let smaller_node = nodes.remove(0);
        bigger_node = combine(Box::from(bigger_node), Box::from(smaller_node));
        nodes.push(bigger_node);
    }
    nodes.remove(0)
}

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

impl HuffmanNode<u8> {
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
}

fn combine(bigger: Box<HuffmanNode<u8>>, smaller: Box<HuffmanNode<u8>>) -> HuffmanNode<u8> {
    // TODO: refactor if possible
    let mut node: HuffmanNode<u8> = HuffmanNode::default();
    if bigger.content.is_some() && smaller.content.is_some() {
        // both single nodes, new tree with bigger left and smaller right
        node.left = Some(bigger);
        node.right = Some(smaller);
        node.max_depth += 1;
    } else if bigger.content.is_none() && smaller.content.is_some() {
        // bigger is tree, smaller only node --> smaller as leaf to the left
        let bigger_depth = bigger.max_depth;
        node.left = Some(smaller);
        node.right = Some(bigger);
        node.max_depth += bigger_depth;
    } else if bigger.content.is_some() && smaller.content.is_none() {
        // smaller is tree, bigger only node --> bigger as leaf to the left
        let smaller_depth = smaller.max_depth;
        node.left = Some(bigger);
        node.right = Some(smaller);
        node.max_depth += smaller_depth;
    } else if bigger.content.is_none() && smaller.content.is_none() {
        // both trees, new tree with smaller depth left and bigger right
        let mut depth: u16 = 0;
        if bigger.max_depth > smaller.max_depth {
            depth = bigger.max_depth;
            node.left = Some(smaller);
            node.right = Some(bigger);
        } else {
            depth = smaller.max_depth;
            node.left = Some(bigger);
            node.right = Some(smaller);
        }
        node.max_depth += depth;
    } else {
        panic!("lol");
    }

    return node;
    //     if self.left.is_some() {
    //         if self.right.is_none() {
    //             let node: HuffmanNode<u8> = HuffmanNode {
    //                 chance: 0,
    //                 content: None,
    //                 left: Some(value),
    //                 right: None,
    //             };
    //             self.right = Some(Box::from(node));
    //         } else {
    //             self.right.as_mut().unwrap().append(value);
    //         }
    //     } else {
    //         self.left = Some(value);
    //     }
}

impl Default for HuffmanNode<u8> {
    fn default() -> Self {
        HuffmanNode { chance: 0, max_depth: 1, content: None, left: None, right: None }
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

    use super::{combine, HuffmanNode, parse_u8_stream};

    //TODO: do a lot of testing
    #[test]
    fn test_append_manually() {
        //TODO: replace this with actual testing
        let mut tree = HuffmanNode::default();
        let node1: HuffmanNode<u8> = HuffmanNode { chance: 0, max_depth: 1, content: Some(1), left: None, right: None };
        tree = combine(Box::from(tree), Box::from(node1));

        let node2: HuffmanNode<u8> = HuffmanNode { chance: 0, max_depth: 1, content: Some(2), left: None, right: None };
        tree = combine(Box::from(tree), Box::from(node2));

        let node3: HuffmanNode<u8> = HuffmanNode { chance: 0, max_depth: 1, content: Some(3), left: None, right: None };
        tree = combine(Box::from(tree), Box::from(node3));

        let node4: HuffmanNode<u8> = HuffmanNode { chance: 0, max_depth: 1, content: Some(4), left: None, right: None };
        tree = combine(Box::from(tree), Box::from(node4));
        println!("{:?}", tree);
        assert!(true);
    }

    #[test]
    fn test_parse_stream_manually() {
        //TODO: replace this with actual testing
        let mut stream = BitStream::open();
        stream.append(1u8);
        stream.append(1u8);
        stream.append(1u8);
        stream.append(1u8);
        stream.append(1u8);

        stream.append(2u8);
        stream.append(2u8);
        stream.append(2u8);

        stream.append(3u8);
        stream.append(4u8);
        stream.append(5u8);
        stream.append(6u8);
        let tree = parse_u8_stream(&mut stream);
        println!("{:?}", tree);
        assert!(true);
    }
}
