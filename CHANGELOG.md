# Changelog

All notable changes to this project will be documented in this file.

The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `--ago` to `start`, `restart` and `stop`.
- Add `--exclude-today` to `balance` to count only finished days.
- Allow passing a date as `--from` and `--to` to the `report` command.

### Changed

- Take a date instead of a date time in the `--from` and `--to` options of
  `balance`.

### Fixed

- Count no vacation or sick day in `daysoff` on a date without working hours or
  on a holiday, and only half a day on a half holiday.
- Count two half days off on one date as a whole day off in `balance`, for
  example a half holiday and a half vacation day.

## [0.2.1] - 2026-10-11

### Security

- Make the data directory and its files readable by the owner only, including
  ones that already exist.
- Reject a project or tag name that contains a control character in `start`,
  `restart`, `project rename` and `tag rename`.
- Replace control characters with `�` in text output and error messages, so an
  escape sequence in a name written into a data file cannot act on the
  terminal. JSON output is unchanged.

### Fixed

- Accept a past `--at` in `start` while a frame is running, and stop the
  running frame at that time.
- Reject a start time in the future in `start` and `restart`.
- Reject ending the running frame before its start time in `stop` without
  `--at` and in `start`, for example after the system clock is set back.
- Show an error in `report` and `balance` when `--day`, `--week`, `--month` or
  `--year` starts on a date on which the clock skips midnight.
- Count a running frame that starts after the current time as zero in `report`
  and `balance`, for example after the system clock is set back.

## [0.2.0] - 2026-08-09

### Changed

- Create the data files private (`0600`). A file that already exists keeps the
  mode it has.
- Write tags as plain words. A tag that starts with `+` is rejected, naming the
  tag to write instead.
- Reject an unknown key in `config.toml` and in `config set`.
- Store the files in `~/.local/share/ebb`, or in `$XDG_DATA_HOME/ebb` when that
  variable names an absolute path.
- Rename `--config-dir` to `--data-dir` and `EBB_CONFIG_DIR` to `EBB_DATA_DIR`.
- Reject a bare number as a time. `--at`, `--from` and `--to` used to read one
  as a Unix timestamp, so `--at 1200` silently meant 1970-01-01.

### Deprecated

- `EBB_CONFIG_DIR`. Use `EBB_DATA_DIR` instead. The old name still works and
  warns.

### Fixed

- Keep the running frame open when `start` rejects the new start time. It used
  to be saved as a frame and stay in the state file, so it was counted twice.
- Reject a timespan that runs backwards in `balance` and `report`, instead of
  printing an absurd expected duration or panicking.
- Write the data files through a temporary file, so that an interrupted write no
  longer truncates them.
- Keep the previous frames in `frames.toml.bak`.
- Apply `--at` and `--no-gap` when they follow a tag.
- Report an invalid timestamp in `frames.toml` or `state.toml` instead of
  panicking or showing it as 1970-01-01.
- Report a `frames.toml` that cannot be read when `--no-gap` is given.

### How to upgrade

Ebb keeps its files in the XDG data directory rather than the configuration
directory, because only one of them is configuration. Move the directory once:

```bash
mv ~/.config/ebb ~/.local/share/ebb
```

If you set `EBB_CONFIG_DIR` or pass `--config-dir`, nothing moves. Both still
work, and `--config-dir` is an alias, but rename the variable to `EBB_DATA_DIR`
to stop the warning.

## [0.1.0] - 2025-06-22

Initial release

[unreleased]: https://github.com/woylie/ebb/compare/v0.2.1...HEAD
[0.2.0]: https://github.com/woylie/ebb/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/woylie/ebb/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/woylie/ebb/releases/tag/v0.1.0
