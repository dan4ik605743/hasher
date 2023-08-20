use std::{
    fs,
    io::{BufReader, Read},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use base64::{engine::general_purpose, Engine as _};
use crc::{Crc, CRC_32_ISCSI};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub size: u64,

    pub hash: String,
    pub blocks_hashes: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub data: Vec<File>,
}

pub fn hashing_file(path: &str, blocks: usize) -> Result<(String, Vec<u32>)> {
    const HASH: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

    let mut reader = BufReader::new(fs::File::open(path)?);
    let mut buffer: Vec<u8> = vec![0; blocks];
    let mut blocks_hashes: Vec<u32> = Vec::new();
    let mut hash = Sha256::new();

    loop {
        let mut digest = HASH.digest();
        let count = reader.read(&mut buffer)?;

        if count == 0 {
            break;
        }

        digest.update(&buffer[..count]);
        hash.update(&buffer[..count]);
        blocks_hashes.push(digest.finalize());
    }
    let hash: String = general_purpose::STANDARD_NO_PAD.encode(hash.finalize());

    Ok((hash, blocks_hashes))
}

pub fn file_verification(lhs: &File, rhs: &File) -> bool {
    tracing::info!("Checking file size and hash: {}...", lhs.name);
    let res = lhs.size == rhs.size && lhs.hash == rhs.hash;

    if !res {
        tracing::warn!("Failed");
        return res;
    }

    tracing::info!("Successfully");
    res
}

pub fn hash_blocks_verification(lhs: &[u32], rhs: &[u32], block_size: usize, file_size: u64) {
    tracing::info!("Checking block hashes...");
    let mut not_changed = true;

    lhs.iter()
        .zip(rhs.iter())
        .enumerate()
        .for_each(|(index, (lhs, rhs))| {
            if lhs != rhs {
                not_changed = false;
                tracing::warn!(
                    "Changed {}..{} bytes",
                    index * block_size,
                    std::cmp::min((index + 1) * block_size, file_size as usize)
                );
            }
        });

    if not_changed {
        tracing::info!("Successfully");
    }
}
