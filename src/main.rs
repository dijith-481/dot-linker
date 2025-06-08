use clap::Parser;
use std::io::Write;
use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The target directory to symlink to
    #[clap(short, long)]
    target: Option<PathBuf>,

    /// The directory to symlink from
    #[clap(value_name = "DIR")]
    dir: Option<PathBuf>,

    /// The files to symlink if not symlinking from a directory this takes precedence over dir
    #[clap(short, long, value_name = "FILE", num_args=1.. )]
    files: Option<Vec<String>>,

    /// The files to ignore if symlinking from a directory
    #[clap(short, long, value_name = "IGNORE", num_args=1.. )]
    ignore: Option<Vec<String>>,

    /// donesn't actually symlink but prints the target
    #[clap(short, long, default_value_t = false)]
    no_symlink: bool,

    /// asks for confirmation before symlinking
    #[clap(short, long, default_value_t = false)]
    visual: bool,

    /// prints verbose output
    #[clap(long, default_value_t = false)]
    verbose: bool,
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let target = match args.target {
        Some(t) => {
            if !t.exists() {
                anyhow::bail!("target \"{}\" does not exist", t.display());
            }
            if !t.is_dir() {
                anyhow::bail!("target \"{}\" is not a directory", t.display());
            }
            print_verbose(&format!("target directory set to {}", t.display()));
            t
        }
        None => {
            if env::var("XDG_CONFIG_HOME").is_ok() {
                let path = env::var("XDG_CONFIG_HOME")?;
                print_verbose(&format!("target directory  set to {}", path));
                PathBuf::from(path)
            } else {
                let path = env::var("HOME")?;
                if args.visual {
                    print!("set target directory  to {}/.config(y/n)", path);
                    std::io::stdout().flush()?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if !matches!(input.trim(), "y" | "yes" | "") {
                        println!("aborting");
                        return Ok(());
                    }
                    print_verbose(&format!("target directory set to {}/.config", path));
                } else {
                    print_verbose(&format!(
                        " XDG_CONFIG_HOME is not set, setting target directory  to {}/.config",
                        path
                    ));
                }
                PathBuf::from(path).join(".config")
            }
        }
    };
    match args.files {
        Some(files) => {
            for file in files {
                let path = env::current_dir()?.join(file);
                println!("file is {:?}", path);
                handle_symlink(&path, &target, args.no_symlink, args.visual)?;
            }
        }
        None => {
            if args.verbose {
                println!("no files provided in --files    checking for directory");
            }
            let current_dir = match args.dir {
                Some(d) => {
                    if !d.exists() {
                        anyhow::bail!("dir does not exist");
                    }
                    print_verbose(&format!("dir set to {}", d.display()));
                    d
                }
                None => {
                    let current_dir = env::current_dir()?;
                    if args.visual {
                        print!("use current directory {} (y/n)", current_dir.display());
                        std::io::stdout().flush()?;
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        if !matches!(input.trim(), "y" | "yes" | "") {
                            println!("aborting");
                            return Ok(());
                        }
                        print_verbose("directory set to current dir");
                    } else {
                        print_verbose("no dir provided in --dir, using current dir");
                    }
                    current_dir
                }
            };
            let files = current_dir.read_dir()?;
            let ignore = match args.ignore {
                Some(i) => {
                    print_verbose("ignoring files");
                    i.iter().map(|i| current_dir.join(i)).collect()
                }
                None => HashSet::new(),
            };
            for file in files {
                let path = file?.path();
                if ignore.contains(&path) {
                    print_verbose(&format!("ignoring {}", path.display()));
                    continue;
                }
                handle_symlink(&path, &target, args.no_symlink, args.visual)?;
            }
        }
    }
    Ok(())
}

fn handle_symlink(
    file: &PathBuf,
    target: &Path,
    no_symlink: bool,
    visual: bool,
) -> anyhow::Result<()> {
    if file.file_name().is_none() {
        println!("skipping '{}'  filename not found", file.display());
        return Ok(());
    }
    let file_name = file.file_name().unwrap();
    let target = target.join(file_name);
    if visual {
        print!("link? '{}' (y/n)", file_name.to_string_lossy());
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !matches!(input.trim(), "y" | "yes" | "") {
            print_verbose(&format!("skipping '{}' ", file.display()));
            return Ok(());
        }
    }
    if target.exists() {
        println!("target '{}' already exists", target.display());
        return Ok(());
    }
    if no_symlink {
        println!(
            "symlinking '{}' to '{}', no symlink created due to --no-symlink",
            file.display(),
            target.display()
        );
        return Ok(());
    } else {
        println!("symlinking '{}' to {}", file.display(), target.display());
    }
    std::os::unix::fs::symlink(file, target)?;
    Ok(())
}
fn print_verbose(msg: &str) {
    let verbose = Args::parse().verbose;
    if verbose {
        println!("{}", msg);
    }
}
