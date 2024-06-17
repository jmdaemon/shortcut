use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Top level directory for creating shortcuts within
    #[arg(short, long)]
    pub root: PathBuf,

    /// How many layers of folders should have shortcuts
    /// This will generate shortcuts for folders up to depth
    #[arg(long)]
    pub depth: usize,

    /// Destination file path
    #[arg(short, long)]
    pub dest: PathBuf,

    #[arg(short, long)]
    pub excludes: Option<Vec<String>>,
}
