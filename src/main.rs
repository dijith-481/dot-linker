use anyhow::Result;
use clap::Parser;
use std::{env, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    target: Option<PathBuf>,

    #[clap(value_name = "DIR")]
    dir: Option<PathBuf>,

    #[clap(short, long, value_name = "FILE", num_args=1.. )]
    files: Option<Vec<PathBuf>>,

    #[clap(short, long, value_name = "IGNORE", num_args=1.. )]
    ignore: Option<Vec<PathBuf>>,

    #[clap(short, long)]
    visual: bool,
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let target = match args.target {
        Some(t) => {
            if !t.exists() {
                anyhow::bail!("target does not exist");
            }
            t
        }
        None => {
            if env::var("XDG_CONFIG_HOME").is_ok() {
                PathBuf::from(env::var("XDG_CONFIG_HOME").unwrap())
            } else {
                PathBuf::from(env::var("HOME").unwrap()).join(".config")
            }
        }
    };

    // if args.files.is_some() {
    //     println!("files are {:?}", args.files);
    //     for file in args.files.unwrap() {
    //         let target = target.join(file.file_name().unwrap());
    //         if args.visual {
    //             println!("{}", target.display());
    //         } else {
    //             // std::os::unix::fs::symlink(file, target).unwrap();
    //             println!("symlinking{:?}{:?}", file, target);
    //         }
    //     }
    // } else {
    //     let current_dir = if args.dir.is_none() {
    //         //get current dir
    //         env::current_dir().unwrap()
    //     } else {
    //         if !args.dir.as_ref().unwrap().exists() {
    //             panic!("dir does not exist");
    //         }
    //         args.dir.unwrap()
    //     };
    //     let files = current_dir.read_dir().unwrap();
    //     for file in files {
    //         let file = file.unwrap();
    //         let path = file.path();
    //         if args.ignore.is_some() {
    //             let ignore = args.ignore.as_ref().unwrap();
    //             if ignore.contains(&path) {
    //                 continue;
    //             }
    //         }
    //         if path.is_dir() {
    //             let target = target.join(path.file_name().unwrap());
    //             if args.visual {
    //                 println!("{}", target.display());
    //             } else {
    //                 // std::os::unix::fs::symlink(path, target).unwrap();
    //                 println!("symlinking{:?}{:?}", path, target);
    //             }
    //         }
    //     }
    // }
    Ok(())
}
