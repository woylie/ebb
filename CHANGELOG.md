# Changelog

## Unreleased

### Fixed

- Keep the running frame open when `start` rejects the new start time. It used
  to be saved as a frame and stay in the state file, so it was counted twice.
- Reject a timespan that runs backwards in `balance` and `report`, instead of
  printing an absurd expected duration or panicking.
- Write the data files through a temporary file, so that an interrupted write no
  longer truncates them.
- Keep the previous frames in `frames.toml.bak`.

## [0.1.0] - 2025-06-22

Initial release
