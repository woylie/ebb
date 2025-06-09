# Command-Line Help for `ebb`

This document contains the help content for the `ebb` command-line program.

**Command Overview:**

- [`ebb`↴](#ebb)
- [`ebb cancel`↴](#ebb-cancel)
- [`ebb config`↴](#ebb-config)
- [`ebb config get`↴](#ebb-config-get)
- [`ebb config list`↴](#ebb-config-list)
- [`ebb config set`↴](#ebb-config-set)
- [`ebb holiday`↴](#ebb-holiday)
- [`ebb holiday add`↴](#ebb-holiday-add)
- [`ebb holiday edit`↴](#ebb-holiday-edit)
- [`ebb holiday list`↴](#ebb-holiday-list)
- [`ebb holiday remove`↴](#ebb-holiday-remove)
- [`ebb project`↴](#ebb-project)
- [`ebb project list`↴](#ebb-project-list)
- [`ebb report`↴](#ebb-report)
- [`ebb restart`↴](#ebb-restart)
- [`ebb sickday`↴](#ebb-sickday)
- [`ebb sickday add`↴](#ebb-sickday-add)
- [`ebb sickday edit`↴](#ebb-sickday-edit)
- [`ebb sickday list`↴](#ebb-sickday-list)
- [`ebb sickday remove`↴](#ebb-sickday-remove)
- [`ebb start`↴](#ebb-start)
- [`ebb status`↴](#ebb-status)
- [`ebb stop`↴](#ebb-stop)
- [`ebb tag`↴](#ebb-tag)
- [`ebb tag list`↴](#ebb-tag-list)
- [`ebb vacation`↴](#ebb-vacation)
- [`ebb vacation add`↴](#ebb-vacation-add)
- [`ebb vacation edit`↴](#ebb-vacation-edit)
- [`ebb vacation list`↴](#ebb-vacation-list)
- [`ebb vacation remove`↴](#ebb-vacation-remove)

## `ebb`

**Usage:** `ebb [OPTIONS] <COMMAND>`

###### **Subcommands:**

- `cancel` — Cancel the current time tracking frame
- `config` — Manage the configuration
- `holiday` — Manage holidays
- `project` — Manage projects
- `report` — Return the total time and time spent per project
- `restart` — Restart the last project
- `sickday` — Manage sick days
- `start` — Start time tracking
- `status` — Show current time tracking status
- `stop` — Stop time tracking
- `tag` — Manage tags
- `vacation` — Manage vacation days

###### **Options:**

- `-c`, `--config-dir <CONFIG_DIR>` — Set the configuration directory

  Default value: `~/.config/ebb`

- `-f`, `--format <FORMAT>` — Set the output format

  Default value: `text`

  Possible values: `text`, `json`

## `ebb cancel`

Cancel the current time tracking frame

**Usage:** `ebb cancel`

## `ebb config`

Manage the configuration

**Usage:** `ebb config
       config <COMMAND>`

###### **Subcommands:**

- `get` — Get a single configuration value
- `list` — List all configuration values
- `set` — Set a configuration value

## `ebb config get`

Get a single configuration value

**Usage:** `ebb config get <KEY>`

###### **Arguments:**

- `<KEY>`

## `ebb config list`

List all configuration values

**Usage:** `ebb config list`

## `ebb config set`

Set a configuration value

**Usage:** `ebb config set <KEY> <VALUE>`

###### **Arguments:**

- `<KEY>` — Configuration key
- `<VALUE>` — Configuration value

## `ebb holiday`

Manage holidays

**Usage:** `ebb holiday
       holiday <COMMAND>`

###### **Subcommands:**

- `add` — Add a new holiday
- `edit` — Edit the description of an existing holiday
- `list` — List all holidays
- `remove` — Remove a holiday

## `ebb holiday add`

Add a new holiday

**Usage:** `ebb holiday add [OPTIONS] <DATE> [DESCRIPTION]`

###### **Arguments:**

- `<DATE>` — Date of the holiday (yyyy-mm-dd, e.g. 2025-08-11)
- `<DESCRIPTION>` — Name of the holiday (e.g. Mountain Day)

  Default value: `Holiday`

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Default value: `full`

  Possible values: `full`, `half`

## `ebb holiday edit`

Edit the description of an existing holiday

**Usage:** `ebb holiday edit [OPTIONS] <DATE> <DESCRIPTION>`

###### **Arguments:**

- `<DATE>` — Date of the holiday to edit
- `<DESCRIPTION>` — New name for the holiday

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Possible values: `full`, `half`

## `ebb holiday list`

List all holidays

**Usage:** `ebb holiday list [OPTIONS]`

###### **Options:**

- `-y`, `--year <YEAR>` — Filter by year

## `ebb holiday remove`

Remove a holiday

**Usage:** `ebb holiday remove <DATE>`

###### **Arguments:**

- `<DATE>` — Date of the holiday to remove

## `ebb project`

Manage projects

**Usage:** `ebb project
       project <COMMAND>`

###### **Subcommands:**

- `list` — List all projects

## `ebb project list`

List all projects

**Usage:** `ebb project list`

## `ebb report`

Return the total time and time spent per project

**Usage:** `ebb report [OPTIONS]`

###### **Options:**

- `--from <FROM>` — Start time (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601)
- `--to <TO>` — End time (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601)
- `-y`, `--year` — Report time spent in the current year
- `-m`, `--month` — Report time spent in the current month
- `-w`, `--week` — Report time spent in the current week
- `-d`, `--day` — Report time spent on the current day
- `-p`, `--project <PROJECT>` — Filter by project

## `ebb restart`

Restart the last project

**Usage:** `ebb restart [OPTIONS]`

###### **Options:**

- `--at <AT>` — Time at which the project is restarted (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
- `-G`, `--no-gap` — Set the start time to the end time of the last saved frame

## `ebb sickday`

Manage sick days

**Usage:** `ebb sickday
       sickday <COMMAND>`

###### **Subcommands:**

- `add` — Add a new sick day
- `edit` — Edit the description of an existing sick day
- `list` — List all sick days
- `remove` — Remove a sick day

## `ebb sickday add`

Add a new sick day

**Usage:** `ebb sickday add [OPTIONS] <DATE> [DESCRIPTION]`

###### **Arguments:**

- `<DATE>` — Day of the sick day
- `<DESCRIPTION>` — Description for the sick day

  Default value: `Sick`

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Default value: `full`

  Possible values: `full`, `half`

## `ebb sickday edit`

Edit the description of an existing sick day

**Usage:** `ebb sickday edit [OPTIONS] <DATE> <DESCRIPTION>`

###### **Arguments:**

- `<DATE>` — Date of the sick day to edit
- `<DESCRIPTION>` — New description for the sick day

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Possible values: `full`, `half`

## `ebb sickday list`

List all sick days

**Usage:** `ebb sickday list [OPTIONS]`

###### **Options:**

- `-y`, `--year <YEAR>` — Filter by year

## `ebb sickday remove`

Remove a sick day

**Usage:** `ebb sickday remove <DATE>`

###### **Arguments:**

- `<DATE>` — Date of the sick day to remove

## `ebb start`

Start time tracking

**Usage:** `ebb start [OPTIONS] <PROJECT> [TAGS]...`

###### **Arguments:**

- `<PROJECT>` — Name of the project
- `<TAGS>` — Any number of additional tags

###### **Options:**

- `--at <AT>` — Time at which the project is started (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
- `-G`, `--no-gap` — Set the start time to the end time of the last saved frame

## `ebb status`

Show current time tracking status

**Usage:** `ebb status`

## `ebb stop`

Stop time tracking

**Usage:** `ebb stop [OPTIONS]`

###### **Options:**

- `--at <AT>` — Time at which the project is stopped (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used

## `ebb tag`

Manage tags

**Usage:** `ebb tag
       tag <COMMAND>`

###### **Subcommands:**

- `list` — List all tags

## `ebb tag list`

List all tags

**Usage:** `ebb tag list`

## `ebb vacation`

Manage vacation days

**Usage:** `ebb vacation
       vacation <COMMAND>`

###### **Subcommands:**

- `add` — Add a new vacation day
- `edit` — Edit the description of an existing vacation day
- `list` — List all vacation days
- `remove` — Remove a vacation day

## `ebb vacation add`

Add a new vacation day

**Usage:** `ebb vacation add [OPTIONS] <DATE> [DESCRIPTION]`

###### **Arguments:**

- `<DATE>` — Date of the vacation day
- `<DESCRIPTION>` — Name of the vacation day

  Default value: `Vacation`

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Default value: `full`

  Possible values: `full`, `half`

## `ebb vacation edit`

Edit the description of an existing vacation day

**Usage:** `ebb vacation edit [OPTIONS] <DATE> <DESCRIPTION>`

###### **Arguments:**

- `<DATE>` — Date of the vacation day to edit
- `<DESCRIPTION>` — New name for the vacation day

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Possible values: `full`, `half`

## `ebb vacation list`

List all vacation days

**Usage:** `ebb vacation list [OPTIONS]`

###### **Options:**

- `-y`, `--year <YEAR>` — Filter by year

## `ebb vacation remove`

Remove a vacation day

**Usage:** `ebb vacation remove <DATE>`

###### **Arguments:**

- `<DATE>` — Date of the vacation day to remove

<hr/>

<small><i>
This document was generated automatically by
<a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
