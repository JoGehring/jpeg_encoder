use std::collections::HashMap;
use crate::bit_stream::BitStream;
use crate::huffman::{HuffmanNode, increment_or_append};

pub fn package_merge(stream: &mut BitStream, height: u16) {
    let mut nodes: Vec<HuffmanNode<u8>> = vec![];
    for byte in stream.data() {
        increment_or_append(&mut nodes, *byte);
    }
    if (nodes.len() as f64).log2().ceil() > height as f64 {
        panic!("Package merge not possible");
    }
    nodes.sort_by_key(|node| node.chance());
    let p: Vec<(u8, u64)> = nodes.iter().map(|node| (node.content().unwrap(), node.chance())).collect();
    let mut lookup: HashMap<u8, (u8, u64)> = HashMap::with_capacity(p.len());
    let mut l = vec![0; nodes.len()];
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
    for i in &p {
        map.insert(i.0, (l[lookup.get(&i.0).unwrap().0 as usize], 0));
    }
    println!("{:?}", map);
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