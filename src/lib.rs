extern crate chrono;
extern crate crypto;
#[macro_use]
extern crate lazy_static;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use chrono::prelude::*;

pub struct Block {
    index: u32,
    previous_hash: String,
    timestamp: i64,
    data: String,
    hash: String,
}

impl Block {
    pub fn calculate_hash(&self) -> String {
        let &Block {
            index,
            ref previous_hash,
            timestamp,
            ref data,
            ..
        } = self;
        calculate_hash(index, previous_hash, timestamp, data)
    }
}

lazy_static! {
    static ref BLOCKCHAIN: Vec<Block> = vec![
        Block {
            index: 0,
            previous_hash: String::from("0"),
            timestamp: 0,
            data: String::from("Genesis block"),
            hash: String::from("2740aaf9a9a4bb7dfdbcdf12dc1c240f5e1f715330eae639ca745e20df365a0f"),
        },
    ];
}

pub fn calculate_hash(index: u32, previous_hash: &str, timestamp: i64, data: &str) -> String {
    let mut hasher = Sha256::new();
    println!(
        "{}",
        format!("{}{}{}{}", index, previous_hash, timestamp, data)
    );
    hasher.input_str(&format!("{}{}{}{}", index, previous_hash, timestamp, data));
    hasher.result_str()
}

pub fn generate_next_block(data: &str) -> Block {
    let previous_block = get_latest_block();
    let index = previous_block.index + 1;
    let timestamp = Utc::now().timestamp();
    let hash = calculate_hash(index, &previous_block.hash, timestamp, data);
    Block {
        index,
        previous_hash: previous_block.hash.clone(),
        timestamp,
        data: String::from(data),
        hash,
    }
}

pub fn get_latest_block<'a>() -> &'a Block {
    let tmp: &'a Block = BLOCKCHAIN
        .last()
        .expect("There must be at least one element in list");
    tmp
}

pub fn is_valid_new_block(new_block: &Block, prev_block: &Block) -> bool {
    if prev_block.index + 1 != new_block.index {
        println!("Invalid index");
        false
    } else if prev_block.hash != new_block.previous_hash {
        println!("Invalid previous hash");
        false
    } else if new_block.calculate_hash() != new_block.hash {
        println!("Invalid hash for new block");
        false
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_correct() {
        let previous_hash = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        let hash = calculate_hash(123, previous_hash, 10000, "test");
        assert_eq!(
            hash,
            "3d6b22fb1b539fb5073eb1de8cb6b6c30782e51c23d8015d197335801e8c811a"
        );
    }

    #[test]
    fn hash_for_genesis_block_is_correct() {
        let block = &BLOCKCHAIN[0];
        let previous_hash = &block.previous_hash;
        let hash = calculate_hash(block.index, &previous_hash, block.timestamp, &block.data);
        assert_eq!(hash, block.hash);
    }
}
