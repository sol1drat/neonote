# NeoNote

Keyboard-first Markdown note taking in your terminal.

![NeoNote Demo](docs/demo.gif)

A small, fast, no-account, no-cloud note app. Point it at a directory, browse
your Markdown files as a tree, edit them with a vim-style editor.

*NeoNote is currently in a barely enough usable state, do not expect much.*

## Why

- **Local first** plain `.md` files on disk. No database, no sync, no server
- **Keyboard first**, every action is one or two keystrokes. No mouse
- **Single binary**, minimal dependencies. Starts instantly, runs anywhere
- **Stays out of the way**, no telemetry, no bloated sidebars. Just notes

## Install

### Prebuilt binaries

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

Inside the app:

1. From the menu, press `v` to browse for a vault directory
2. Navigate the directory tree with `h` `j` `k` `l`
3. Press `Enter` on a directory to confirm opening it as a vault
4. Browse your notes in the explorer. Press `Enter` on a `.md` file to
   open it in the editor
5. Press `Ctrl+s` to save, `Esc` to return to the explorer, `q` to quit

## Keybindings

### Menu

| Key     | Action           |
| ------- | ---------------- |
| `v`     | Open a vault     |
| `q`     | Quit             |

### Vault selection

| Key     | Action                          |
| ------- | ------------------------------- |
| `h`     | Go to parent directory          |
| `j`     | Move down                       |
| `k`     | Move up                         |
| `l`     | Enter selected directory        |
| `c`     | Create a new directory here     |
| `Enter` | Open selected directory as vault |
| `q`     | Quit                            |

### File explorer

| Key     | Action                          |
| ------- | ------------------------------- |
| `j`     | Move down                       |
| `k`     | Move up                         |
| `Enter` | Open file / toggle directory    |
| `Tab`   | Focus the editor                |
| `f`     | Create a new file               |
| `c`     | Create a new directory          |
| `r`     | Rename selected file or dir     |
| `d`     | Delete selected file or dir     |
| `q`     | Quit                            |

### Editor

The editor uses [edtui](https://github.com/preiter10/edtui) a vim-style editor.

| Key      | Action                          |
| -------- | ------------------------------- |
| `Esc`    | Back to explorer (from Normal)  |
| `Tab`    | (from Explorer) focus editor    |
| `Ctrl+s` | Save current file               |
| `Ctrl+q` | Quit                            |

The cursor shape tells you the mode: **block** = Normal, **bar** = Insert,
**underscore** = Visual/Search.

A `*` next to the filename in the editor title means unsaved changes.

### Prompts

| Key     | Action                       |
| ------- | ---------------------------- |
| `y`     | Confirm (in Yes/No prompts)  |
| `n`/`Esc` | Cancel                       |
| `Enter` | Confirm (in text prompts)    |
| `Esc`   | Cancel (in text prompts)     |

## Limitations

- **No undo/redo beyond what edtui provides**, save before risky edits
- **Quitting discards unsaved changes silently**, the confirm prompt doesn't
  check for unsaved buffers yet
- **Hidden files (dotfiles) are not shown** by design
- **No symlink handling**, symlinked directories inside a vault may behave
  unexpectedly

## Build

```sh
cargo build --release
```

Compiling for Windows on NixOS (Linux):

```sh
./build/windows.sh
# output: target/x86_64-pc-windows-gnu/release/neonote.exe
```

## Contributing

The project is in active early development.
Bug reports and suggestions are very welcome.

A few things especially appreciated right now:

- Testing on terminals I haven't tried (Windows Terminal, WezTerm, Kitty,
  Alacritty, , Contour) and reporting rendering bugs.
- Reproductions for the bugs.
- README / help-screen copy edits.

## License

Apache-2.0. See [LICENSE](LICENSE).
