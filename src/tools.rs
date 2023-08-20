use std::fs;

use anyhow::{bail, Result};
use itertools::Itertools;
use rayon::prelude::*;

use super::hash::{self, File};

pub type Data = Vec<File>;

pub fn hashing_data(path: &str, block_size: usize) -> Result<Data> {
    tracing::info!("Started scanning files...");
    let paths_files: Vec<_> = fs::read_dir(path)?.collect();

    if paths_files.is_empty() {
        tracing::error!("Folder is empty");
        bail!("");
    }

    tracing::info!("Successfully");
    tracing::info!("File hashing started...");
    let mut data: Vec<File> = paths_files
        .into_par_iter()
        .map(|path| {
            let path = path?;
            let (hash, blocks_hashes) =
                hash::hashing_file(path.path().to_str().unwrap(), block_size)?;

            Ok(File {
                name: path.file_name().to_str().unwrap().to_owned(),
                size: filesize::file_real_size(path.path().to_str().unwrap())?,
                hash,
                blocks_hashes,
            })
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;
    data.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    tracing::info!("Successfully");
    Ok(data)
}

pub fn check_data(data: Data, other_data_path: Option<&str>, block_size: usize) -> Result<()> {
    if other_data_path.is_none() {
        let data = serde_json::to_string(&hash::Data { data })?;
        fs::write("data.json", data)?;

        tracing::info!("Data written to data.json");
        return Ok(());
    }

    tracing::info!("File verification started...");
    let other_data = get_other_data(other_data_path.unwrap())?;

    let lost_files = get_difference_data(&data, &other_data);
    if !lost_files.is_empty() {
        tracing::warn!("Missing files: {lost_files}");
    }

    let found_files = get_difference_data(&other_data, &data);
    if !found_files.is_empty() {
        tracing::warn!("Found new files: {found_files}");
    }

    file_verification(get_iter_equal_files_data(&data, &other_data), block_size);

    Ok(())
}

fn get_other_data(path: &str) -> Result<Data> {
    let data: hash::Data = serde_json::from_str(fs::read_to_string(path)?.as_str())?;
    let mut data: Data = data.data;
    data.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    Ok(data)
}

fn get_difference_data(data: &Data, other_data: &Data) -> String {
    iter_set::difference(
        data.iter().map(|file| &file.name),
        other_data.iter().map(|file| &file.name),
    )
    .join(", ")
}

fn get_iter_equal_files_data<'a>(
    data: &'a Data,
    other_data: &'a Data,
) -> impl Iterator<Item = (&'a File, &'a File)> {
    let equal_files_data = iter_set::intersection_by(data.iter(), other_data.iter(), |lhs, rhs| {
        lhs.name.cmp(&rhs.name)
    });
    let equal_files_other_data =
        iter_set::intersection_by(other_data.iter(), data.iter(), |lhs, rhs| {
            lhs.name.cmp(&rhs.name)
        });

    equal_files_data
        .into_iter()
        .zip(equal_files_other_data.into_iter())
}

fn file_verification<'a>(iter: impl Iterator<Item = (&'a File, &'a File)>, block_size: usize) {
    iter.for_each(|(lhs, rhs)| {
        if !hash::file_verification(lhs, rhs) {
            hash::hash_blocks_verification(
                &lhs.blocks_hashes,
                &rhs.blocks_hashes,
                block_size,
                lhs.size,
            );
        }
    });
}
