use serde::{Serialize, Deserialize}; //used to serialise deserialise the data 
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write}; //buffering the data and sending in large chunks rather than sending it small packets
use std::cmp::Ordering; // used to compare the data
extern crate bincode; // bincode crate is specifically designed to serialize and deserialize Rust data structures to and from a compact binary format. 

#[derive(Serialize, Deserialize, Eq, PartialEq)]
struct ByteNode { // huffmann tree
    data: Option<u8>, 
    frequency: i32, // frequency
    left: Option<Box<ByteNode>>, //left branch
    right: Option<Box<ByteNode>>, //right branch
}

impl Ord for ByteNode { //You can use impl to define methods for a struct 
    fn cmp(&self, other: &Self) -> Ordering { //Compares nodes by frequency in reverse order to create a min-heap.
        other.frequency.cmp(&self.frequency)
    }
}

impl PartialOrd for ByteNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { //Provides partial comparison using the Ord implementation.
        Some(self.cmp(other))
    }
}

fn compress(src: &str, dst: &str) {
    let mut in_file = BufReader::new(File::open(src).unwrap()); //Opens the source file and creates a buffered reader.
    let mut buffer = Vec::new();
    in_file.read_to_end(&mut buffer).unwrap(); //reads the entire file content into the buffer.

    let (huffman_bytes, huffmap) = create_zip(&buffer);//  Calls create_zip to get the compressed data and Huffman codes.

    let mut out_file = BufWriter::new(File::create(dst).unwrap()); // Creates a buffered writer for the destination file.
    bincode::serialize_into(&mut out_file, &huffman_bytes).unwrap(); //Serializes and writes the compressed data to the destination file.
    bincode::serialize_into(&mut out_file, &huffmap).unwrap(); //Serializes and writes the Huffman codes to the destination file.
}

fn create_zip(bytes: &[u8]) -> (Vec<u8>, HashMap<u8, String>) {
    let nodes = get_byte_nodes(bytes); // Calls get_byte_nodes to get the byte nodes with their frequencies.
    let root = create_huffman_tree(nodes);//Calls create_huffman_tree to build the Huffman tree and get the root node.
    let huffman_codes = get_huff_codes(&root); //get_huff_codes(&root): Calls get_huff_codes to get the Huffman codes.
    let huffman_code_bytes = zip_bytes_with_codes(bytes, &huffman_codes); // Calls zip_bytes_with_codes to get the compressed bytes.
    (huffman_code_bytes, huffman_codes) // Returns the compressed bytes and Huffman codes.
}

fn get_byte_nodes(bytes: &[u8]) -> BinaryHeap<ByteNode> { // Defines a function get_byte_nodes which takes a byte slice bytes and returns a BinaryHeap of ByteNode.
    let mut freq_map = HashMap::new();
    for &byte in bytes {
        *freq_map.entry(byte).or_insert(0) += 1; // gets in freq map
    }

    let mut nodes = BinaryHeap::new();
    for (&byte, &freq) in &freq_map { //pushed in heap
        nodes.push(ByteNode { data: Some(byte), frequency: freq, left: None, right: None });
    }
    nodes
}

fn create_huffman_tree(mut nodes: BinaryHeap<ByteNode>) -> ByteNode { //takes a binary heap of ByteNode and returns the root of the Huffman tree.
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

fn get_huff_codes(root: &ByteNode) -> HashMap<u8, String> { //takes the root of the Huffman tree and returns a map of Huffman codes.
    let mut huffmap = HashMap::new();
    let mut sb = String::new(); //Creates a new empty string buffer.
    build_huff_codes(&root, &mut sb, &mut huffmap); //Calls build_huff_codes to populate the Huffman code map.
    huffmap //returns huffmap
}

fn build_huff_codes(node: &ByteNode, prefix: &mut String, huffmap: &mut HashMap<u8, String>) {
    if let Some(data) = node.data { //Checks if the node is a leaf node
        huffmap.insert(data, prefix.clone());
    } else {
        if let Some(ref left) = node.left { //If there is a left child, adds '0' to the prefix and recurses.
            prefix.push('0');
            build_huff_codes(left, prefix, huffmap);
            prefix.pop();
        }
        if let Some(ref right) = node.right { //If there is a right child, adds '1' to the prefix and recurses.
            prefix.push('1');
            build_huff_codes(right, prefix, huffmap);
            prefix.pop();
        }
    }
}

fn zip_bytes_with_codes(bytes: &[u8], huff_codes: &HashMap<u8, String>) -> Vec<u8> { // to compress bytes using Huffman codes.
    let mut str_builder = String::new(); //Creates a new empty string builder
    for &byte in bytes {
        str_builder.push_str(huff_codes.get(&byte).unwrap()); //Appends the Huffman code for the byte to the string builder.
    }

    let len = (str_builder.len() + 7) / 8; //Calculates the length of the resulting byte vector.
    let mut huff_code_bytes = Vec::with_capacity(len); //Creates a byte vector with the calculated capacity.
    for chunk in str_builder.as_bytes().chunks(8) { // Iterates over the string builder in 8-bit chunks.
        let byte_str = std::str::from_utf8(chunk).unwrap(); //Converts each chunk to a string.
        let byte = u8::from_str_radix(byte_str, 2).unwrap(); // Converts the binary string to a byte.
        huff_code_bytes.push(byte); //Appends the byte to the byte vector.
    }
    huff_code_bytes //Returns the compressed byte vector
}

fn decompress(src: &str, dst: &str) {
    let mut in_file = BufReader::new(File::open(src).unwrap());
    let huffman_bytes: Vec<u8> = bincode::deserialize_from(&mut in_file).unwrap(); //Deserializes the compressed bytes from the file.
    let huffman_codes: HashMap<u8, String> = bincode::deserialize_from(&mut in_file).unwrap(); //Deserializes the Huffman codes from the file.

    let bytes = decomp(&huffman_codes, &huffman_bytes); //Calls decomp to decompress the bytes.

    let mut out_file = BufWriter::new(File::create(dst).unwrap()); // Creates a buffered writer for the destination file.
    out_file.write_all(&bytes).unwrap(); //Writes the decompressed bytes to the destination file.
}

fn decomp(huffman_codes: &HashMap<u8, String>, huffman_bytes: &[u8]) -> Vec<u8> {
    let mut bit_string = String::new();
    for &byte in huffman_bytes {
        bit_string.push_str(&format!("{:08b}", byte)); //Converts each byte to its binary representation and appends it to the bit string.
    }

    let mut code_to_byte = HashMap::new(); //Creates a new empty map to store Huffman codes and their corresponding bytes.
    for (byte, code) in huffman_codes {
        code_to_byte.insert(code.clone(), *byte); //Inserts the code and byte into the map.
    }

    let mut result = Vec::new();
    let mut current_code = String::new();
    for bit in bit_string.chars() { //Appends the bit to the current code.
        current_code.push(bit); // Iterates over each bit in the bit string.
        if let Some(&byte) = code_to_byte.get(&current_code) { //Checks if the current code matches a Huffman code.
            result.push(byte); // Appends the corresponding byte to the result vector.
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
