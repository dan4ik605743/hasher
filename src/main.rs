use anyhow::Result;
use clap::Parser;

mod args;
mod hash;
mod logger;
mod tools;

use args::Args;

fn main() -> Result<()> {
    logger::init_logger();
    let args = Args::parse();

    if let Ok(data) = tools::hashing_data(&args.path, args.block_size) {
        tools::check_data(data, args.check.as_deref(), args.block_size)?;
    }

    Ok(())
}
