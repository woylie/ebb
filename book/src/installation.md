# Installation

## Prebuilt Binaries

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

## With Nix

```bash
nix profile install github:woylie/ebb/v0.3.0
```

If the binary can't be found, ensure your Nix profile is in your `PATH`.

```bash
export PATH="$HOME/.nix-profile/bin:$PATH"
```

## Manual Build

With Cargo:

```bash
git clone https://github.com/woylie/ebb.git --branch v0.3.0
cd ebb
cargo build --release
./target/release/ebb
```

With Nix:

```bash
nix build github:woylie/ebb/v0.3.0
./result/bin/ebb
```

## Shell Completions

The Nix package includes completions for bash, fish and zsh. With any other
installation, you can generate shell completions with:

```bash
ebb generate-completions --shell <SHELL>
```

To see a list of available shells, run:

```bash
ebb generate-completions --help
```
