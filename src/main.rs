use anyhow::{Result, ensure};
use clap::Parser;
use dot_linker::{
    cli::Args,
    config::{determine_config_file, get_config_path},
    handle_link,
    ignore::IgnoreList,
    link::get_link_action,
    prompt_user,
    ui::verbose_println,
};

use dot_linker::UIMode;
use dot_linker::ui::get_ui_mode;
use std::{
    env,
    path::{Path, PathBuf},
};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let is_verbose = args.verbose;
    let config_path = get_config_path()?;
    let target_dir = determine_target_directory(&args.target, &config_path, is_verbose)?;
    let current_dir = env::current_dir()?;
    let base_dir = determine_base_dir(&args.dir, &current_dir, is_verbose)?;
    let ui_mode = get_ui_mode(args.visual);
    let config_file_path =
        determine_config_file(&args.config, &current_dir, &base_dir, &config_path)?;

    let mut ignore_list = IgnoreList::new();
    ignore_list.load_from_file(&config_file_path, &base_dir)?;
    ignore_list.add_literals(args.ignore, &base_dir);

    let action = get_link_action(args.unset);
    let simulate = args.no_symlink;

    match args.files {
        Some(files) => {
            for file in files {
                let path = base_dir.join(file);
                if ignore_list.is_ignored(&path)
                    && prompt_user(
                        &format!(
                            "'{}' is in ignore list, do you  want to  skip it? ",
                            path.file_name().unwrap().to_string_lossy()
                        ),
                        UIMode::Interactive,
                    )?
                {
                    println!(
                        "'{}' in ignore list ,skipping",
                        path.file_name().unwrap().to_string_lossy()
                    );
                    continue;
                }
                handle_link(&path, &target_dir, &action, simulate, ui_mode)?
            }
        }
        None => {
            let files = base_dir.read_dir()?;
            for file in files {
                let path = file?.path();
                if ignore_list.is_ignored(&path) {
                    println!(
                        "'{}' in ignore list ,skipping",
                        path.file_name().unwrap().to_string_lossy()
                    );
                    continue;
                }
                handle_link(&path, &target_dir, &action, simulate, ui_mode)?
            }
        }
    }
    Ok(())
}

fn determine_target_directory(
    target: &Option<PathBuf>,
    config_path: &Path,
    is_verbose: bool,
) -> Result<PathBuf> {
    let dir = match target {
        Some(t) => {
            ensure!(t.exists(), "target '{}' does not exist", t.display());
            ensure!(t.is_dir(), "target '{}' is not a directory", t.display());
            verbose_println(
                &format!("target directory set to '{}'", t.display()),
                is_verbose,
            );
            t.clone()
        }
        None => config_path.to_path_buf(),
    };
    Ok(dir)
}

fn determine_base_dir(
    base_dir: &Option<PathBuf>,
    current_dir: &Path,
    is_verbose: bool,
) -> Result<PathBuf> {
    let dir = match base_dir {
        Some(b) => {
            ensure!(b.exists(), "base dir '{}' does not exist", b.display());
            ensure!(b.is_dir(), "base dir '{}' is not a directory", b.display());
            verbose_println(&format!("base dir set to '{}'", b.display()), is_verbose);
            b.clone()
        }
        None => current_dir.to_path_buf(),
    };
    Ok(dir)
}
