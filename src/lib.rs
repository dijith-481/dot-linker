pub mod cli;

pub use cli::Args;
pub use config::get_config_path;
pub use ignore::IgnoreList;
pub use link::{LinkAction, handle_link};
pub use ui::{UIMode, prompt_user};

pub mod ui {
    use anyhow::Result;
    use std::io::Write;

    #[derive(Clone, Copy)]
    pub enum UIMode {
        Interactive,
        Silent,
    }
    pub fn get_ui_mode(mode: bool) -> UIMode {
        if mode {
            UIMode::Interactive
        } else {
            UIMode::Silent
        }
    }

    pub fn verbose_println(msg: &str, is_verbose: bool) {
        if is_verbose {
            println!("[VERBOSE] {}", msg);
        }
    }

    pub fn prompt_user(prompt: &str, mode: UIMode) -> Result<bool> {
        match mode {
            UIMode::Silent => Ok(true),
            UIMode::Interactive => {
                print!("{} (y/n) ", prompt);
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                Ok(matches!(
                    input.trim().to_lowercase().as_str(),
                    "y" | "yes" | ""
                ))
            }
        }
    }
}

pub mod config {
    use anyhow::{Ok, Result};
    use std::{
        env,
        path::{Path, PathBuf},
    };
    static CONFIG_DIRECTORY: &str = "dotlinker";
    static CONFIG_FILE: &str = "dotlinkerignore";

    pub fn get_config_path() -> Result<PathBuf> {
        Ok(match env::var("XDG_CONFIG_HOME").ok() {
            Some(path) => PathBuf::from(path),
            None => PathBuf::from(env::var("HOME")?).join(".config"),
        })
    }
    pub fn determine_config_file(
        config: &Option<String>,
        curr_dir: &Path,
        base_dir: &Path,
        config_dir: &Path,
    ) -> Result<PathBuf> {
        Ok(match config {
            Some(c) => curr_dir.join(c),
            None => {
                if ignore_file_exists(config_dir) {
                    config_dir.join(CONFIG_DIRECTORY).join(CONFIG_FILE)
                } else if ignore_file_exists(base_dir) {
                    base_dir.join(CONFIG_DIRECTORY).join(CONFIG_FILE)
                } else {
                    create_default_ignore_file(config_dir)?;
                    config_dir.join(CONFIG_DIRECTORY).join(CONFIG_FILE)
                }
            }
        })
    }

    fn create_default_ignore_file(config_path: &Path) -> Result<()> {
        let dotlinker_dir = config_path.join(CONFIG_DIRECTORY);
        std::fs::create_dir_all(&dotlinker_dir)?;
        std::fs::write(
            dotlinker_dir.join(CONFIG_FILE),
            "# This file is used to ignore files when symlinking\n.git*\nREADME.md\nLICENSE",
        )?;
        Ok(())
    }

    fn ignore_file_exists(config_path: &Path) -> bool {
        config_path
            .join(CONFIG_DIRECTORY)
            .join(CONFIG_FILE)
            .exists()
    }
}

pub mod link {

    use anyhow::Result;
    use std::path::Path;

    pub enum LinkAction {
        Link,
        Unlink,
    }

    pub fn get_link_action(action: bool) -> LinkAction {
        if action {
            LinkAction::Unlink
        } else {
            LinkAction::Link
        }
    }

    pub fn handle_link(
        source: &Path,
        target_dir: &Path,
        action: &LinkAction,
        simulate: bool,
    ) -> Result<()> {
        let target_path = match source.file_name() {
            Some(file_name) => target_dir.join(file_name),
            None => {
                println!("skipping '{}'  filename not found", source.display());
                return Ok(());
            }
        };

        match action {
            LinkAction::Link => {
                if target_path.exists() {
                    println!(
                        "target '{}' already exists, skipping.",
                        target_path.display()
                    );
                    return Ok(());
                }
                simulate_println(
                    &format!(
                        "linking '{}' -> '{}'",
                        source.display(),
                        target_path.display()
                    ),
                    simulate,
                );
                std::os::unix::fs::symlink(source, target_path)?;
            }
            LinkAction::Unlink => {
                if !target_path.exists() {
                    println!(
                        "target '{}' doesn't exists, skipping.",
                        target_path.display()
                    );
                    return Ok(());
                }
                if target_path.symlink_metadata()?.file_type().is_symlink() {
                    simulate_println(
                        &format!(
                            "unlinking '{}' -> '{}'",
                            source.display(),
                            target_path.display()
                        ),
                        simulate,
                    );
                    std::fs::remove_file(target_path)?;
                } else {
                    println!(
                        "target '{}' is not a symlink, skipping.",
                        target_path.display()
                    );
                }
            }
        }

        Ok(())
    }

    fn simulate_println(msg: &str, simulate: bool) {
        if simulate {
            println!("[SIMULATE] {}", msg);
        } else {
            println!("{}", msg);
        }
    }
}

pub mod ignore {

    use anyhow::{Result, ensure};
    use glob::Pattern;
    use std::{
        collections::HashSet,
        path::{Path, PathBuf},
    };

    pub struct IgnoreList {
        literals: HashSet<PathBuf>,
        patterns: Vec<Pattern>,
    }
    impl IgnoreList {
        pub fn new() -> Self {
            Self {
                literals: HashSet::new(),
                patterns: Vec::new(),
            }
        }
        pub fn load_from_file(&mut self, file_path: &Path, base_dir: &Path) -> Result<()> {
            ensure!(
                file_path.exists(),
                "config file '{}' does not exist",
                file_path.display()
            );
            let contents = std::fs::read_to_string(file_path)?;
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let mut pattern = line.to_string();
                if pattern.ends_with('/') {
                    pattern = pattern.strip_suffix('/').unwrap().to_string();
                } else if pattern.starts_with('/') {
                    pattern = pattern.strip_prefix('/').unwrap().to_string();
                }
                if is_literal_pattern(&pattern) {
                    self.literals.insert(base_dir.join(pattern));
                } else {
                    self.patterns.push(Pattern::new(&pattern)?);
                }
            }
            Ok(())
        }

        pub fn add_literals(&mut self, paths: Option<Vec<String>>, base_dir: &Path) {
            let paths = match paths {
                Some(p) => p,
                None => return,
            };
            for path_str in paths {
                self.literals.insert(base_dir.join(path_str));
            }
        }

        pub fn is_ignored(&self, path: &Path) -> bool {
            if self.literals.contains(path) {
                return true;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            self.patterns.iter().any(|p| p.matches(file_name))
        }
    }

    impl Default for IgnoreList {
        fn default() -> Self {
            Self::new()
        }
    }

    fn is_literal_pattern(pattern: &str) -> bool {
        !pattern.contains(&['*', '?', '[', ']'][..])
    }
}
