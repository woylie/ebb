# Changelog

## Unreleased

### Fixed

- Keep the running frame open when `start` rejects the new start time. It used
  to be saved as a frame and stay in the state file, so it was counted twice.

## [0.1.0] - 2025-06-22

Initial release
