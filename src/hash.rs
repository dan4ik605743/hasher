use std::{
    fs,
    io::{BufReader, Read},
};

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use crc::{Crc, CRC_32_ISCSI};
use serde::{Deserialize, Serialize};
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

pub fn hashing(path: &str, blocks: usize) -> Result<(String, Vec<u32>)> {
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

pub fn file_verification(lhs: (u64, &str), rhs: (u64, &str)) -> bool {
    let res = lhs == rhs;

    if res {
        tracing::info!("Successfully");
    } else {
        tracing::warn!("Failed");
    }

    res
}

pub fn hash_blocks_verification(lhs: &[u32], rhs: &[u32], block_size: usize) {
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
                    (index + 1) * block_size
                );
            }
        });

    if not_changed {
        tracing::info!("Successfully");
    }
}
