# NeoNote

A simple, no bullshit, keyboard-first note app
for people who live in the terminal.

![NeoNote Demo](docs/demo.gif)

*NeoNote is currently in a barely enough usable state, do not expect much.*

## Install

### Binaries

Prebuilt binaries are available for Linux and Windows on the [releases page](https://github.com/sol1drat/neonote/releases).

### From source

Requires Rust (edition 2024):

```sh
git clone https://github.com/sol1drat/neonote.git
cd neonote
cargo build --release
```

Or, if you have `cargo` set up:

```sh
cargo install --git https://github.com/sol1drat/neonote.git
```

### NixOs

A `shell.nix` is provided for development and cross-compilation:

```sh
nix-shell             # dev shell
./build/windows.sh    # build a Windows .exe via nix + mingw
```

## Quick start

```sh
neonote               # launch the menu, pick a vault
neonote ~/notes       # skip the menu, open ~/notes as a vault
neonote --help
neonote --version
```

## Build

```sh
cargo build --release
```

A compile script exists for building NeoNote for Windows on NixOS (Linux):

```sh
./build/windows.sh
# output: target/x86_64-pc-windows-gnu/release/neonote.exe
```

## Contributing

The project is in active early development.
Bug reports and suggestions are very welcome.

Would especially appreciate testing on terminals I haven't tried,
like Windows Terminal, WezTerm, Kitty, Alacritty and reporting any rendering bugs.

## License

Apache-2.0. See [LICENSE](LICENSE).
