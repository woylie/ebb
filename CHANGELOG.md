# Changelog

## Unreleased

### Changed

- Create the data files private (`0600`). A file that already exists keeps the
  mode it has.
- Write tags as plain words. A tag that starts with `+` is rejected, naming the
  tag to write instead.
- Reject an unknown key in `config.toml` and in `config set`.

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

## [0.1.0] - 2025-06-22

Initial release
