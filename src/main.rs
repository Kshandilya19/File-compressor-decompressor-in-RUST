use serde::{Serialize, Deserialize};
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::cmp::Ordering;
extern crate bincode; 

#[derive(Serialize, Deserialize, Eq, PartialEq)]
struct ByteNode {
    data: Option<u8>,
    frequency: i32,
    left: Option<Box<ByteNode>>,
    right: Option<Box<ByteNode>>,
}

impl Ord for ByteNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.frequency.cmp(&self.frequency)
    }
}

impl PartialOrd for ByteNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn compress(src: &str, dst: &str) {
    let mut in_file = BufReader::new(File::open(src).unwrap()); 
    let mut buffer = Vec::new();
    in_file.read_to_end(&mut buffer).unwrap(); 

    let (huffman_bytes, huffmap) = create_zip(&buffer);

    let mut out_file = BufWriter::new(File::create(dst).unwrap()); 
    bincode::serialize_into(&mut out_file, &huffman_bytes).unwrap(); 
    bincode::serialize_into(&mut out_file, &huffmap).unwrap(); 
}

fn create_zip(bytes: &[u8]) -> (Vec<u8>, HashMap<u8, String>) {
    let nodes = get_byte_nodes(bytes); 
    let root = create_huffman_tree(nodes);
    let huffman_codes = get_huff_codes(&root); 
    let huffman_code_bytes = zip_bytes_with_codes(bytes, &huffman_codes); 
    (huffman_code_bytes, huffman_codes) 
}

fn get_byte_nodes(bytes: &[u8]) -> BinaryHeap<ByteNode> { 
    let mut freq_map = HashMap::new();
    for &byte in bytes {
        *freq_map.entry(byte).or_insert(0) += 1; 
    }

    let mut nodes = BinaryHeap::new();
    for (&byte, &freq) in &freq_map { 
        nodes.push(ByteNode { data: Some(byte), frequency: freq, left: None, right: None });
    }
    nodes
}

fn create_huffman_tree(mut nodes: BinaryHeap<ByteNode>) -> ByteNode { 
    while nodes.len() > 1 {
        let left = nodes.pop().unwrap();
        let right = nodes.pop().unwrap();
        let parent = ByteNode {
            data: None,
            frequency: left.frequency + right.frequency,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };
        nodes.push(parent);
    }
    nodes.pop().unwrap()
}

fn get_huff_codes(root: &ByteNode) -> HashMap<u8, String> { 
    let mut huffmap = HashMap::new();
    let mut sb = String::new(); 
    build_huff_codes(&root, &mut sb, &mut huffmap); 
    huffmap 
}

fn build_huff_codes(node: &ByteNode, prefix: &mut String, huffmap: &mut HashMap<u8, String>) {
    if let Some(data) = node.data { 
        huffmap.insert(data, prefix.clone());
    } else {
        if let Some(ref left) = node.left { 
            prefix.push('0');
            build_huff_codes(left, prefix, huffmap);
            prefix.pop();
        }
        if let Some(ref right) = node.right { 
            prefix.push('1');
            build_huff_codes(right, prefix, huffmap);
            prefix.pop();
        }
    }
}

fn zip_bytes_with_codes(bytes: &[u8], huff_codes: &HashMap<u8, String>) -> Vec<u8> {
    let mut str_builder = String::new();
    for &byte in bytes {
        str_builder.push_str(huff_codes.get(&byte).unwrap());
    }

    let len = (str_builder.len() + 7) / 8;
    let mut huff_code_bytes = Vec::with_capacity(len);
    for chunk in str_builder.as_bytes().chunks(8) {
        let byte_str = std::str::from_utf8(chunk).unwrap();
        let byte = u8::from_str_radix(byte_str, 2).unwrap();
        huff_code_bytes.push(byte);
    }
    huff_code_bytes
}

fn decompress(src: &str, dst: &str) {
    let mut in_file = BufReader::new(File::open(src).unwrap());
    let huffman_bytes: Vec<u8> = bincode::deserialize_from(&mut in_file).unwrap();
    let huffman_codes: HashMap<u8, String> = bincode::deserialize_from(&mut in_file).unwrap();

    let bytes = decomp(&huffman_codes, &huffman_bytes);

    let mut out_file = BufWriter::new(File::create(dst).unwrap());
    out_file.write_all(&bytes).unwrap();
}

fn decomp(huffman_codes: &HashMap<u8, String>, huffman_bytes: &[u8]) -> Vec<u8> {
    let mut bit_string = String::new();
    for &byte in huffman_bytes {
        bit_string.push_str(&format!("{:08b}", byte));
    }

    let mut code_to_byte = HashMap::new();
    for (byte, code) in huffman_codes {
        code_to_byte.insert(code.clone(), *byte);
    }

    let mut result = Vec::new();
    let mut current_code = String::new();
    for bit in bit_string.chars() {
        current_code.push(bit);
        if let Some(&byte) = code_to_byte.get(&current_code) {
            result.push(byte);
            current_code.clear();
        }
    }
    result
}

fn main() {
    let src = r"C:\PES\CS SEM-4\file_compressor\file.txt";
    let dst_compressed = r"C:\PES\CS SEM-4\file_compressor\compressed_file.huff";
    let dst_decompressed = r"C:\PES\CS SEM-4\file_compressor\decompressed_file.txt";

    compress(src, dst_compressed);
    decompress(dst_compressed, dst_decompressed);
}
