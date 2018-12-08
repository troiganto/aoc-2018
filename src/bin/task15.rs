use std::{
    io::{self, Read},
    iter,
};

#[derive(Debug)]
struct Node {
    children: Vec<Node>,
    metadata: Vec<u32>,
}

#[derive(Debug)]
struct NoElement;

fn build_tree(numbers: &mut impl Iterator<Item = u32>) -> Result<Node, NoElement> {
    let num_children = numbers.next().ok_or(NoElement)? as usize;
    let num_metadata = numbers.next().ok_or(NoElement)? as usize;
    let children = iter::repeat_with(|| build_tree(numbers))
        .take(num_children)
        .collect::<Result<_, _>>()?;
    let metadata = numbers.take(num_metadata).collect();
    Ok(Node { children, metadata })
}

fn visit<V: FnMut(&[u32])>(root: &Node, visitor: &mut V) {
    visitor(&root.metadata);
    for child in &root.children {
        visit(child, visitor);
    }
}

fn sum_metadata(root: &Node) -> u32 {
    let mut sum: u32 = 0;
    let mut visitor = |data: &[u32]| {
        sum += data.iter().sum::<u32>();
    };
    visit(root, &mut visitor);
    sum
}

fn get_node_value(node: &Node) -> u32 {
    if node.children.is_empty() {
        node.metadata.iter().sum()
    } else {
        node.metadata
            .iter()
            .filter_map(|&i| node.children.get(i as usize - 1))
            .map(|node| get_node_value(node))
            .sum()
    }
}

fn main() {
    let buffer: String = {
        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.lock().read_to_string(&mut buffer).unwrap();
        buffer
    };
    let tree = build_tree(&mut buffer.split_whitespace().map(|n| n.parse().unwrap())).unwrap();
    println!("sum of metadata: {}", sum_metadata(&tree));
    println!("value of root node: {}", get_node_value(&tree));
}
