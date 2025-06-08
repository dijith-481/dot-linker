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
                print_visual(
                    &format!("set target directory  to {}/.config", path),
                    "target directory set to .config",
                    "aborting",
                    Some(&format!(
                        " XDG_CONFIG_HOME is not set, setting target directory  to {}/.config",
                        path
                    )),
                )?;
                PathBuf::from(path).join(".config")
            }
        }
    };
    match args.files {
        Some(files) => {
            for file in files {
                let path = env::current_dir()?.join(file);
                println!("file is {:?}", path);
                handle_symlink(&path, &target, args.no_symlink)?;
            }
        }
        None => {
            print_verbose("no files provided in --files    checking for directory");
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
                    print_visual(
                        &format!("use current directory {} ", current_dir.display()),
                        "directory set to current dir",
                        "aborting",
                        Some("no dir provided in --dir, using current dir"),
                    )?;
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
                handle_symlink(&path, &target, args.no_symlink)?;
            }
        }
    }
    Ok(())
}

fn handle_symlink(file: &PathBuf, target: &Path, no_symlink: bool) -> anyhow::Result<()> {
    if file.file_name().is_none() {
        println!("skipping '{}'  filename not found", file.display());
        return Ok(());
    }
    let file_name = file.file_name().unwrap();
    let target = target.join(file_name);
    print_visual(
        &format!("link? '{}' ", file_name.to_string_lossy()),
        &format!("symlinking '{}' to '{}'", file.display(), target.display()),
        &format!("skipping '{}' ", file.display()),
        None,
    )?;
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

fn print_visual(
    msg: &str,
    final_msg: &str,
    skip_msg: &str,
    else_msg: Option<&str>,
) -> anyhow::Result<bool> {
    let visual = Args::parse().visual;
    if !visual {
        if let Some(msg) = else_msg {
            print_verbose(msg);
        }
        return Ok(true);
    }
    print!("{}", msg);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !matches!(input.trim(), "y" | "yes" | "") {
        println!("{}", skip_msg);
        return Ok(false);
    }
    print_verbose(final_msg);
    Ok(true)
}
