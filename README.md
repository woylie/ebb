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

## Building

```bash
cargo build --release
```

## Development

Run tests:

```bash
cargo test
```

Format Rust code:

```bash
rustfmt **/*.rs
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
