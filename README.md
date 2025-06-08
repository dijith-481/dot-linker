

#  dot-linker

symlink's your dots

## Features
A simple, GNU Stow-like utility for managing your configuration files (dotfiles) by creating symbolic links. It allows you to keep your dotfiles in a version-controlled directory and link them to their proper locations.
-   **Link & Unlink:** Easily create and remove symbolic links.
-   **Dry Run Mode:** See what changes will be made without actually performing any actions (`--no-symlink`).
-   **Interactive Confirmation:** Ask for confirmation before creating or removing each link (`--visual`).
-   **Flexible:** Link an entire directory or just specific files.
-   **Ignore Files:** Supports `.gitignore`-style patterns to exclude files from linking.
-   **Customizable:** Specify source and target directories, making it useful for more than just dotfiles.

## Installation

Currently, you need the Rust toolchain to build the project.

```bash
# Clone the repository
git clone <your-repo-url>
cd dot-linker

# Build the release binary
cargo build --release

# The binary will be at target/release/dot-linker
./target/release/dot-linker --help
```

## Usage

The command-line interface provides all the options you need to manage your links.

```bash
Usage: dot-linker [OPTIONS] [DIR]

Arguments:
  [DIR]  The directory to symlink from

Options:
  -t, --target <TARGET>     The directory to symlink to
  -f, --files <FILE>...     The files to symlink highier precedence than dir
  -i, --ignore <IGNORE>...  The files to ignore
  -n, --no-symlink          simulate the  symlink no actual linking
  -v, --visual              asks for confirmation before actions
      --verbose             prints verbose output
  -u, --unset               unset symlink
  -c, --config <CONFIG>     path to config file
  -h, --help                Print help
  -V, --version             Print version
```

## Configuration

`dot-linker` can be configured using an ignore file to prevent certain files or patterns from being linked.

The application looks for an ignore file in the following order:
1.  A path specified with the `-c, --config` option.
2.  `$XDG_CONFIG_HOME/dotlinker/dotignore` (e.g., `~/.config/dotlinker/dotignore`).
3.  The base directory being linked from (e.g., `~/dotfiles/.dotlinker/dotignore`).

If no ignore file is found, a default one will be created at `~/.config/dotlinker/dotignore` with the following content:

```
# This file is used to ignore files when symlinking
.git*
README.md
LICENSE
```

## Examples

Assume your dotfiles are stored in `~/dotfiles`.

#### 1. Link all files to your home directory

This will link files like `~/dotfiles/.zshrc` to `~/.zshrc`.

```bash
# The target directory defaults to $HOME/.config if not provided
# and the source directory defaults to the current directory.
cd ~/dotfiles
dot-linker

# Or, more explicitly:
dot-linker -t ~ ~/dotfiles
```

#### 2. Unlink all files

This removes the symlinks from your home directory. Your original files in `~/dotfiles` are safe.

```bash
dot-linker --unset -t ~ ~/dotfiles
```

#### 3. See what would happen (Dry Run)

Simulate linking to see which files would be created, without making any changes.

```bash
dot-linker --no-symlink -t ~ ~/dotfiles
```

#### 4. Link with interactive confirmation

Prompt for a `(y/n)` confirmation before creating each symlink.

```bash
dot-linker -v -t ~ ~/dotfiles
```

#### 5. Link only specific files

Link only your `niri` and `foot` configurations into `~/.config`.

```bash
cd dotfiles
dot-linker -t ~/.config -f nvim alacritty 
```
#### 5. passing files to be ignored

Link eveything other than  `niri` and `foot` configurations into `~/.config`.

```bash
cd dotfiles
dot-linker -t ~/.config -i nvim alacritty 
```
## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
