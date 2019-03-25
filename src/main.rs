#![allow(dead_code)]
#![allow(unused_imports)]
//mod cube;
mod pedersen_hash;
mod hasher;
mod bit_iterator;

fn main() {
    //cube::test_cube_proof();
    pedersen_hash::test_pedersen_proof();
    println!("Done");
}