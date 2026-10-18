# Time Tracking

The commands related to time tracking are:

- `ebb start <PROJECT> [TAGS]` - Start tracking time for the given project with
  optional tags. Tags are written as plain words, so
  `ebb start website design review` records the project `website` with the tags
  `design` and `review`. A project name or a tag that contains spaces has to be
  quoted, as in `ebb start website "code review"`.
- `ebb stop` - Stop tracking the current project.
- `ebb cancel` - Cancel tracking without saving.
- `ebb restart` - Restart the last project.
- `ebb status` - Show the current time tracking status.
