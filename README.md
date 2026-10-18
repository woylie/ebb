<!--
SPDX-FileCopyrightText: 2025 Mathias Polligkeit

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Ebb

CLI for time tracking and flex time balance.

[Documentation](https://ebb.wlyx.dev)

## Features

- Time tracking
- Projects and tags
- Flex time balance
- Vacation, holiday, and sick day tracking

## Installation

### With Nix

```bash
nix profile install github:woylie/ebb/v0.2.1
```

If the binary can't be found, ensure your Nix profile is in your `PATH`.

```bash
export PATH="$HOME/.nix-profile/bin:$PATH"
```

### Manual Build

With Cargo:

```bash
git clone https://github.com/woylie/ebb.git --branch v0.2.1
cd ebb
cargo build --release
./target/release/ebb
```

With Nix:

```bash
nix build github:woylie/ebb/v0.2.1
./result/bin/ebb
```

### Shell Completions

The Nix package includes completions for bash, fish and zsh. With any other
installation, you can generate shell completions with:

```bash
ebb generate-completions --shell <SHELL>
```

To see a list of available shells, run:

```bash
ebb generate-completions --help
```

## Documentation

The [user guide](https://ebb.wlyx.dev) describes configuration, time tracking,
reporting, days off and data storage, and includes the
[command-line help](https://ebb.wlyx.dev/command_line_help.html).

## Development

The repo contains a `flake.nix`. You can get into a development shell with all
required packages with `nix develop`. If you have `direnv` installed, you can
also run `direnv allow`.

Run all checks via flake:

```bash
nix flake check
```

Run tests:

```bash
cargo test
```

Format Rust code:

```bash
cargo fmt
```

Format Nix code:

```bash
nixfmt flake.nix
```

Format anything else:

```bash
prettier . --write
```

Lint Rust code with Clippy:

```bash
cargo clippy --all-targets --all-features --no-deps
```

Check licenses with cargo-deny:

```bash
cargo deny check licenses
```

Check [Reuse](https://reuse.software) compliance:

```bash
pipx run reuse lint
```

Add SPDX header to file:

```bash
pipx reuse annotate --copyright="YOUR NAME" --license="AGPL-3.0-or-later" <filename>
```

Generate the command-line help in the book:

```bash
./gen-docs.sh
```

Preview the book:

```bash
mdbook serve book
```

Generate `THIRD_PARTY` file:

```bash
./gen-third-party.sh
```
