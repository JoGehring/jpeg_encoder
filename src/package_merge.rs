use std::collections::HashMap;

use crate::bit_stream::BitStream;
use crate::huffman::{code_len_to_tree, get_single_leaves, HuffmanNode};

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
        println!("value: {}, current code:{:08b}, code length: {}", *val, current_code, code_length);
    }
}

#[cfg(test)]
mod tests {
    use crate::{bit_stream::BitStream, huffman::HuffmanNode};

    use super::{package_merge, package_merge_experimental};

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
        let shortest_code_len = map
            .clone()
            .into_iter()
            .min_by_key(|(_, value)| value.0)
            .unwrap()
            .1
            .0;
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

    #[test]
    #[ignore]
    fn test_experimental() {
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
        let map = tree.code_map();
        let mut expected: Vec<(u8, (u8, u16))> = map.into_iter().map(|(k, v)| (k, v)).collect();
        expected.sort_by_key(|val| val.0);
        let experimental_map = package_merge_experimental(&mut stream, 5);
        let mut test: Vec<(u8, (u8, u16))> = experimental_map.into_iter().map(|(k, v)| (k, v)).collect();
        test.sort_by_key(|val| val.0);
        // 27 is the most likely symbol so it should have the shortest code
        assert_eq!(expected, test);
    }
}
