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

## Getting Started

### Configuration

You can list all available configuration options with:

```bash
ebb config list
```

Modify the defaults with:

```bash
ebb config set <KEY> <VALUE>
```

For example: `ebb config set working_hours.monday 6h`

- `sick_days_per_year.<YEAR>` - Defines how many sick days you are allowed to
  take in a year. The value applies to the specified year and all subsequent
  years, until a new value is set for a later year.
- `vacation_days_per_year.<YEAR>` - Specifies the number of vacation days
  allowed per year, starting from the given year. The value applies to the
  specified year and all subsequent years, until a new value is set for a later
  year.
- `working_hours.<WEEKDAY>` - Defines how many hours you are supposed to work on
  a weekday. The value is a duration string, e.g. `"6h"` or `"7h 30m"`.

### Holidays, Vacation Days, and Sick Days

You can manage holidays, vacation days, and sick days with `ebb holiday`,
`ebb vacation`, and `ebb sickday`. Run the commands without further arguments
to see the command line help.

Holidays can be national holidays or other days on which you are not expected to
work or log hours.

You can render an overview with the count of taken and remaining vacation days
and sick days for the current year or other years with `ebb daysoff` or
`ebb daysoff --year 2024`.

### Time Tracking

The commands related to time tracking are:

- `ebb start <PROJECT> [TAGS]` - Start tracking time for the given project with
  optional tags.
- `ebb stop` - Stop tracking the current project.
- `ebb cancel` - Cancel tracking without saving.
- `ebb restart` - Restart the last project.
- `ebb status` - Show the current time tracking status.

### Reporting

- `ebb balance` - Prints the current time balance: expected work hours, actual
  work hours, and the remaining work hours to break even.
  A negative remaining value indicates overtime. This overview requires the
  working hours, vacation days, sick days, and holidays to be configured
  correctly. By default, the regarded time frame starts at first logged entry
  and ends at the end of the current day. Run `ebb balance --help` to view
  options to change the time frame.
- `ebb report` - Print a report listing the recorded time per project and tag.

### Important to Know

- In printed durations, 1d means 24h, _not_ one working day.
- Vacation days, holidays, and sick days can be full or half days. This is
  relative to the expected working hours configured for that day. For example,
  if Monday is configured with 8 expected working hours, a full day off will
  reduce the flex time balance by 8 hours, and a half day off will
  reduce it by 4 hours. If a day is configured with 6 expected working hours,
  a full day off reduces the balance by 6 hours, and a half day off reduces the
  balance by 3 hours. This logic may be improved in the future based on user
  feedback.

### Further Help

There's a [Markdown version of the command-line help](https://github.com/woylie/ebb/blob/main/command_line_help.md) available. You can also view help for any command with
`-h` or `--help` (e.g., `ebb balance --help`).

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
