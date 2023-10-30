# heyps

`heyps` is a command-line tool to execute Adobe Photoshop scripts from the terminal.

## Why?

- As I am an artist, I wanted to have a tool to execute Adobe applications scripts from the terminal.

## Functionality

Currently, it is a wrapper around the `osascript`, and `open -a` commands to execute Adobe Photoshop `.jsx`, `js`, `.psjs` scripts from the terminal.
In the future, I am planning to add more Adobe applications support.

## Requirements

- [macOS](https://www.apple.com/macos/) (tested with version 11.4)
- [Adobe Photoshop](https://www.adobe.com/products/photoshop.html) (tested with version 22.4.2)

## Usage

```sh
heyps --app <APP> --target <TARGET> --execute <FILE_PATH>
```

### Options

```sh
-a, --app <APP>: The app abbreviation to execute the script in. Required.
-t, --target <TARGET>: The target app version to execute the script in. Optional. Default: latest. [possible values: latest, beta, <YEAR>]
-e, --execute <FILE_PATH>: The path to the script file to execute in Adobe Photoshop
-h, --help: Print help
-V, --version: Print versioneyps --help
```

### Examples

```sh
heyps -a ps -t 2023 -e /path/to/my_script.jsx
```

## Installation

1. Make sure you have Rust installed on your system. You can install Rust from the official Rust website: https://www.rust-lang.org/tools/install
2. Clone the repository from GitHub:
```sh
git clone https://github.com/oleksiiluchnikov/heyps.git
```
3. Install the dependencies and build the binary:
```sh
cd heyps
cargo build --release
# Optionally copy the binary to your PATH
cp target/release/heyps /usr/local/bin/heyps
```

## Contributing

If you want to contribute to heyps, please fork the repository, make your changes, and open a pull request. We welcome all contributions!

## License
[MIT](https://choosealicense.com/licenses/mit/)
