# Data Storage

Ebb keeps all data as TOML files in a single directory. The files can be read
and edited with any text editor.

The default directory is `~/.local/share/ebb`. If `$XDG_DATA_HOME` is set to an
absolute path, the directory is `$XDG_DATA_HOME/ebb`.

You can choose a different directory by passing the `--data-dir` argument or by
setting the `EBB_DATA_DIR` environment variable.

The files:

- `config.toml` - The configuration.
- `frames.toml` - The recorded time frames.
- `frames.toml.bak` - Backup of the time frames. Ebb only keeps a single backup
  that is overwritten with every change.
- `state.toml` - The currently running frame. When you stop a project or start
  a different project, this frame is written to `frames.toml`.
- `holidays.toml`, `sick_days.toml`, `vacations.toml` - The days off.

If you want to back up your files, copy the whole directory:

```bash
cp -r ~/.local/share/ebb ~/backups/ebb-$(date +%Y-%m-%d)
```
