# Installation

## With Nix

```bash
nix profile install github:woylie/ebb/v0.2.1
```

If the binary can't be found, ensure your Nix profile is in your `PATH`.

```bash
export PATH="$HOME/.nix-profile/bin:$PATH"
```

## Manual Build

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
