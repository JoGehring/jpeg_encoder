use std::collections::HashMap;

use crate::bit_stream::BitStream;
use crate::huffman::{code_len_to_tree, HuffmanNode, get_single_leaves};

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

#[cfg(test)]
mod tests {
    use crate::{bit_stream::BitStream, huffman::HuffmanNode};

    use super::package_merge;

    #[test]
    fn test_package_merge_empty_stream() {
        let mut stream = BitStream::open();
        let tree = package_merge(&mut stream, 16);
        assert_eq!(HuffmanNode::default(), tree)
    }

    #[test]
    fn test_package_merge_single_symbol() {
        let mut stream = BitStream::open();
        stream.append_byte(1);
        stream.append_byte(1);
        stream.append_byte(1);
        let tree = package_merge(&mut stream, 16);
        assert_eq!(
            HuffmanNode {
                left: None,
                content: None,
                right: Some(Box::from(HuffmanNode {
                    content: Some(1),
                    chance: u64::MAX - 1,
                    ..Default::default()
                })),
                ..Default::default()
            },
            tree
        )
    }

    #[test]
    fn test_package_merge_bigger_stream() {
        let mut stream = BitStream::open();
        for _ in 0..2 {
            stream.append_byte(1);
            stream.append_byte(2);
        }
        for _ in 0..3 {
            stream.append_byte(3);
            stream.append_byte(4);
        }
        for _ in 0..4 {
            stream.append_byte(5);
        }
        for _ in 0..5 {
            stream.append_byte(6);
        }

        for _ in 0..6 {
            stream.append_byte(7);
        }

        for _ in 0..7 {
            stream.append_byte(8);
        }
        for _ in 0..7 {
            stream.append_byte(9);
        }
        for _ in 0..7 {
            stream.append_byte(10);
        }
        for _ in 0..7 {
            stream.append_byte(11);
        }
        for _ in 0..7 {
            stream.append_byte(12);
        }
        for _ in 0..7 {
            stream.append_byte(13);
        }

        for _ in 0..7 {
            stream.append_byte(14);
        }
        for _ in 0..17 {
            stream.append_byte(15);
        }
        for _ in 0..71 {
            stream.append_byte(16);
        }
        for _ in 0..74 {
            stream.append_byte(17);
        }
        for _ in 0..17 {
            stream.append_byte(18);
        }
        for _ in 0..71 {
            stream.append_byte(19);
        }
        for _ in 0..74 {
            stream.append_byte(20);
        }
        for _ in 0..7 {
            stream.append_byte(21);
        }
        for _ in 0..7 {
            stream.append_byte(22);
        }
        for _ in 0..7 {
            stream.append_byte(23);
        }

        for _ in 0..7 {
            stream.append_byte(24);
        }
        for _ in 0..17 {
            stream.append_byte(25);
        }
        for _ in 0..71 {
            stream.append_byte(26);
        }
        for _ in 0..74 {
            stream.append_byte(27);
        }

        let tree = package_merge(&mut stream, 5);
        assert_eq!(5, tree.max_depth() - 1);
        assert_eq!(4, tree.min_depth() - 1);
        let map = tree.code_map();
        let shortest_code_len = map.clone().into_iter().min_by_key(|(_, value)| value.0).unwrap().1.0;
        // 27 is the most likely symbol so it should have the shortest code
        assert_eq!(shortest_code_len, map.get(&27u8).unwrap().0)
    }

    #[test]
    #[should_panic]
    fn test_package_merge_too_many_symbols() {
        let mut stream = BitStream::open();
        stream.append_byte(1);
        stream.append_byte(2);
        stream.append_byte(3);
        stream.append_byte(4);
        stream.append_byte(5);
        stream.append_byte(6);
        stream.append_byte(7);
        stream.append_byte(8);
        stream.append_byte(9);
        let _ = package_merge(&mut stream, 3);
    }
}
