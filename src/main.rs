use std::fs;

use anyhow::Result;
use clap::Parser;
use itertools::Itertools;
use rayon::prelude::*;

mod command_line_args;
mod hash;
mod logger;

use command_line_args::CommandLineArgs;
use hash::{Data, File};

fn main() -> Result<()> {
    let cl_args = CommandLineArgs::parse();

    logger::init_logger();

    let paths_files: Vec<_> = fs::read_dir(cl_args.path)?.collect();
    let mut data: Vec<File> = paths_files
        .into_par_iter()
        .map(|path| {
            let path = path?;
            let (hash, blocks_hashes) =
                hash::hashing(path.path().to_str().unwrap(), cl_args.block_size)?;

            Ok(File {
                name: path.file_name().to_str().unwrap().to_owned(),
                size: filesize::file_real_size(path.path().to_str().unwrap())?,
                hash,
                blocks_hashes,
            })
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;
    data.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    if let Some(path) = cl_args.check {
        let other_data: Data = serde_json::from_str(fs::read_to_string(path)?.as_str())?;
        let mut other_data: Vec<File> = other_data.data;
        other_data.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

        tracing::info!("File verification started");

        let lost_files: String = iter_set::difference(
            data.iter().map(|file| &file.name),
            other_data.iter().map(|file| &file.name),
        )
        .join(", ");

        let found_files = iter_set::difference(
            other_data.iter().map(|file| &file.name),
            data.iter().map(|file| &file.name),
        )
        .join(", ");

        if !lost_files.is_empty() {
            tracing::warn!("Missing files: {lost_files}");
        }
        if !found_files.is_empty() {
            tracing::warn!("Found new files: {found_files}");
        }

        let equal_files_data =
            iter_set::intersection_by(data.iter(), other_data.iter(), |lhs, rhs| {
                lhs.name.cmp(&rhs.name)
            });
        let equal_files_other_data =
            iter_set::intersection_by(other_data.iter(), data.iter(), |lhs, rhs| {
                lhs.name.cmp(&rhs.name)
            });

        let equal_iter = equal_files_data
            .into_iter()
            .zip(equal_files_other_data.into_iter());

        equal_iter.for_each(|(lhs, rhs)| {
            tracing::info!("Checking file size and hash: {}", lhs.name);
            if !hash::file_verification((lhs.size, &lhs.hash), (rhs.size, &rhs.hash)) {
                tracing::info!("Checking block hashes");

                hash::hash_blocks_verification(
                    &lhs.blocks_hashes,
                    &rhs.blocks_hashes,
                    cl_args.block_size,
                );
            }
        });

        Ok(tracing::info!("File verification ended"))
    } else {
        let data = serde_json::to_string(&Data { data })?;
        Ok(fs::write("data.json", data)?)
    }
}
