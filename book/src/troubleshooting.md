# Troubleshooting

Ebb returns an error if a data file is invalid. The error message includes the
file name and the invalid value. Read the error message and fix the TOML file by
hand.

If a project is counted twice, compare `state.toml` with the last entries in
`frames.toml` and ensure that the entry in `frames.toml` ended before the entry
in `state.toml` started. If both files contain the same time span, delete the
wrong entry. Run `ebb cancel` to cancel the running frame.

If you made a mistake while editing `frames.toml`, you can restore the last
version from the backup:

```bash
cp ~/.local/share/ebb/frames.toml.bak ~/.local/share/ebb/frames.toml
```

Note that Ebb overwrites the backup the next time data is changed.
