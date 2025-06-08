use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The target directory to symlink to
    #[clap(short, long)]
    pub target: Option<PathBuf>,

    /// The directory to symlink from
    #[clap(value_name = "DIR")]
    pub dir: Option<PathBuf>,

    /// The files to symlink if not symlinking from a directory this takes precedence over dir
    #[clap(short, long, value_name = "FILE", num_args=1.. )]
    pub files: Option<Vec<String>>,

    /// The files to ignore if symlinking from a directory
    #[clap(short, long, value_name = "IGNORE", num_args=1.. )]
    pub ignore: Option<Vec<String>>,

    /// donesn't actually symlink but prints the target
    #[clap(short, long, default_value_t = false)]
    pub no_symlink: bool,

    /// asks for confirmation before symlinking
    #[clap(short, long, default_value_t = false)]
    pub visual: bool,

    /// prints verbose output
    #[clap(long, default_value_t = false)]
    pub verbose: bool,

    /// unset symlink
    #[clap(short, long, default_value_t = false)]
    pub unset: bool,

    /// path to config file
    #[clap(short, long)]
    pub config: Option<String>,
}
