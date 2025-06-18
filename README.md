<!--
SPDX-FileCopyrightText: 2025 Mathias Polligkeit

SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Ebb

CLI for time tracking and flex time balance.

[Command-Line Help](https://github.com/woylie/ebb/blob/main/command_line_help.md)

## Features

- Time tracking
- Projects and tags
- Flex time balance
- Vacation, holiday, and sick day tracking

## Important to know

- In printed durations, 1d means 24h, _not_ one working day.
- Vacation days, holidays and sick days can be full or half. This is relative to
  the working hours on that day. If Monday is configured to have 8 working
  hours, a full day off means the flex time balance is reduced by 8 hours, and a
  half day off means the flex time balance is reduced by 4 hours. With 6 working
  hours on a day, a full day off results in a reduction of 6 hours and a half
  day off means a reduction in 3 hours. This logic might be improved in the
  future based on user needs.

## Installation

### With Nix

```bash
nix profile install github:woylie/ebb
```

If the binary can't be found, ensure your Nix profile is in your `PATH`.

```bash
export PATH="$HOME/.nix-profile/bin:$PATH"
```

### Manual Build

With Cargo:

```bash
git clone https://github.com/woylie/ebb.git
cd ebb
cargo build --release
./target/release/ebb
```

With Nix:

```bash
nix build github:woylie/ebb
./result/bin/ebb
```

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

Generate markdown documentation:

```bash
./gen-docs.sh
```

Generate `THIRD_PARTY` file:

```bash
./gen-third-party.sh
```
