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

    /// unset symlink
    #[clap(short, long, default_value_t = false)]
    unset: bool,

    /// path to config file
    #[clap(short, long, default_value_t = String::from("dotlinker/dotlinkerignore"))]
    config: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config_path = get_config_path()?;
    let target = match args.target {
        Some(t) => {
            if !t.exists() {
                anyhow::bail!("target '{}' does not exist", t.display());
            }
            if !t.is_dir() {
                anyhow::bail!("target '{}' is not a directory", t.display());
            }
            print_verbose(&format!("target directory set to '{}'", t.display()));
            t
        }
        None => config_path.clone(),
    };

    match args.files {
        Some(files) => {
            for file in files {
                let path = env::current_dir()?.join(file);
                println!("file is {:?}", path);
                handle_symlink(&path, &target, args.no_symlink, args.unset)?;
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
                    if !print_visual(
                        "current directory",
                        &format!("{}", current_dir.display()),
                        Some("no dir provided , using current dir"),
                    )? {
                        println!("aborting");
                        return Ok(());
                    }
                    current_dir
                }
            };
            let files = current_dir.read_dir()?;
            let mut ignore = match args.ignore {
                Some(i) => {
                    print_verbose("ignoring files");
                    i.iter().map(|i| current_dir.join(i)).collect()
                }
                None => HashSet::new(),
            };
            if current_dir.join(&args.config).exists() {
                ignore.extend(convert_ignore_to_globs(
                    &current_dir.join(&args.config),
                    &current_dir,
                )?);
            }
            create_config_file(&config_path)?;
            let dotlinker_path = config_path.join("dotlinker").join("dotlinkerignore");
            ignore.extend(convert_ignore_to_globs(
                &config_path.join(dotlinker_path),
                &current_dir,
            )?);

            for file in files {
                let path = file?.path();
                if ignore.contains(&path) {
                    print_verbose(&format!("ignoring {}", path.display()));
                    continue;
                }
                handle_symlink(&path, &target, args.no_symlink, args.unset)?;
            }
        }
    }
    Ok(())
}

fn handle_symlink(
    file: &PathBuf,
    target: &Path,
    no_symlink: bool,
    unset: bool,
) -> anyhow::Result<()> {
    if file.file_name().is_none() {
        println!("skipping '{}'  filename not found", file.display());
        return Ok(());
    }
    let file_name = file.file_name().unwrap();
    let target = target.join(file_name);
    if unset {
        if !target.exists() {
            println!("target '{}' doesn't exists", target.display());
            return Ok(());
        }
        let metadata = std::fs::symlink_metadata(&target)?;
        if !metadata.file_type().is_symlink() {
            println!("target {} is not a symlink", target.display());
            return Ok(());
        }
        if !print_visual("unlink", &format!("{}", file_name.to_string_lossy()), None)? {
            println!("skipping '{}' ", file.display());
            return Ok(());
        }
        if no_symlink {
            println!(
                "unlinking '{}' from '{}', no unlinking due to --no-symlink",
                file.display(),
                target.display()
            );
            return Ok(());
        } else {
            println!("unlinking '{}' from {}", file.display(), target.display());
            std::fs::remove_file(target)?;
        }
    } else {
        if target.exists() {
            println!("target '{}' already exists", target.display());
            return Ok(());
        }
        if !print_visual("symlink", &format!("{}", file_name.to_string_lossy()), None)? {
            println!("skipping '{}' ", file.display());
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
            std::os::unix::fs::symlink(file, target)?;
        }
    }
    Ok(())
}

fn print_verbose(msg: &str) {
    let verbose = Args::parse().verbose;
    if verbose {
        println!("{}", msg);
    }
}

fn print_visual(item: &str, value: &str, else_msg: Option<&str>) -> anyhow::Result<bool> {
    let visual = Args::parse().visual;
    if !visual {
        if let Some(msg) = else_msg {
            print_verbose(msg);
        }
        return Ok(true);
    }
    print!("set {} to {}(y/n)", item, value);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !matches!(input.trim(), "y" | "yes" | "") {
        return Ok(false);
    }
    print_verbose(&format!("{} set to {}", item, value));
    Ok(true)
}

fn get_config_path() -> anyhow::Result<PathBuf> {
    let xdg_config_home = env::var("XDG_CONFIG_HOME").ok();
    let home = env::var("HOME")?;
    let path = match xdg_config_home {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(home).join(".config"),
    };
    Ok(path)
}

fn convert_ignore_to_globs(
    config_path: &Path,
    current_dir: &Path,
) -> anyhow::Result<HashSet<PathBuf>> {
    let ignorefile = std::fs::read_to_string(config_path)?;
    Ok(ignorefile
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|pattern| {
            let pattern = pattern.trim();

            match pattern {
                "*" => current_dir.join("*"),
                p if p.ends_with('/') => current_dir.join(&p[..p.len() - 1]),
                p => current_dir.join(p),
            }
        })
        .collect())
}

fn create_config_file(config_path: &Path) -> anyhow::Result<()> {
    if config_path
        .join("dotlinker")
        .join("dotlinkerignore")
        .exists()
    {
        return Ok(());
    }
    print_verbose("no dotlinkerignore found creating one");
    let dotlinker_dir = config_path.join("dotlinker");
    std::fs::create_dir_all(&dotlinker_dir)?;
    std::fs::write(
        dotlinker_dir.join("dotlinkerignore"),
        "# This file is used to ignore files when symlinking\n.git",
    )?;
    Ok(())
}
