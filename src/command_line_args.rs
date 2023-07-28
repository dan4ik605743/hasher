use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArgs {
    /// Path to folder
    #[arg(short, long)]
    pub path: String,

    /// Blocks for buffer hashing
    #[arg(short, long, default_value_t = 1024)]
    pub block_size: usize,

    /// Path to json file to validate data
    #[arg(short, long)]
    pub check: Option<String>,
}
