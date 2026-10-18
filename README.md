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

### Prebuilt Binaries

Each release has archives for Linux and macOS:

- `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` for Linux, linked
  statically, so they run on any distribution
- `x86_64-apple-darwin` and `aarch64-apple-darwin` for Intel and Apple Silicon
  Macs

Download the archive for your system from the
[releases page](https://github.com/woylie/ebb/releases), verify that it was
built from this repository, and unpack it:

```bash
version=v0.3.0
target=aarch64-apple-darwin
curl -LO "https://github.com/woylie/ebb/releases/download/$version/ebb-$version-$target.tar.gz"
gh attestation verify "ebb-$version-$target.tar.gz" --repo woylie/ebb
tar -xzf "ebb-$version-$target.tar.gz"
```

Without the GitHub CLI, you can compare the archive with the `SHA256SUMS` file of
the release instead, using `sha256sum -c --ignore-missing SHA256SUMS` on Linux
or `shasum -a 256 -c --ignore-missing SHA256SUMS` on macOS.

Then move the `ebb` binary to a directory on your `PATH`.

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

Release a new version:

1. Bump the version with `./bump.sh major`, `minor`, `patch`, `pre` or an
   explicit version, e.g. `./bump.sh 0.3.0`.
2. Merge the change, tag the merge commit and push the tag, e.g.
   `git tag v0.3.0 && git push origin v0.3.0`.
3. The tag starts the Release workflow, which creates a draft release for the
   tag and attaches the binaries. Once it has finished, add the release notes to
   the draft and publish it, which also publishes the book.
