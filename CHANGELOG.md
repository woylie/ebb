# Changelog

## Unreleased

### Fixed

- Keep the running frame open when `start` rejects the new start time. It used
  to be saved as a frame and stay in the state file, so it was counted twice.
- Reject a timespan that runs backwards in `balance` and `report`, instead of
  printing an absurd expected duration or panicking.

## [0.1.0] - 2025-06-22

Initial release
