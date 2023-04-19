# heyps
---

`heyps` is a command-line tool to execute Adobe Photoshop scripts from the terminal.

## Motivation
---

The motivation behind `heyps` is to:

- Practice Rust.
- Learn how to create a CLI tool with Rust.
- Learn how to use git and GitHub.
- As I am an artist, I wanted to have a tool to execute Adobe Photoshop scripts from the terminal by file path.

## Funtcionality
---

Currently, it is a wrapper around the `osascript`, and `open -a` commands to execute Adobe Photoshop `.jsx`, `js`, `.psjs` scripts from the terminal.

## Requirements
---

- [macOS](https://www.apple.com/macos/) (tested with version 11.4)
- [Adobe Photoshop](https://www.adobe.com/products/photoshop.html) (tested with version 22.4.2)

## Usage
---

```sh
heyps --execute <FILE_PATH>
```

### Options

```sh
-e, --execute <FILE_PATH>: The path to the script file to execute in Adobe Photoshop
-h, --help: Print help
-V, --version: Print versioneyps --help
```

### Examples

```sh
heyps --execute /path/to/my_script.jsx
```

## Installation
---

1. Make sure you have Rust installed on your system. You can install Rust from the official Rust website: https://www.rust-lang.org/tools/install
2. Clone the repository from GitHub:

```sh
git clone https://github.com/user/repo.git
```
3. Install the dependencies and build the binary:

```sh
cd repo
cargo build --release
```

4. Install the binary on your system:

```sh
cargo install --path .
```

## Contributing
---
If you want to contribute to heyps, please fork the repository, make your changes, and open a pull request. We welcome all contributions!
