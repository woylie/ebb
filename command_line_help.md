# Command-Line Help for `Ebb`

This document contains the help content for the `Ebb` command-line program.

**Command Overview:**

- [`Ebb`↴](#Ebb)
- [`Ebb cancel`↴](#Ebb-cancel)
- [`Ebb holiday`↴](#Ebb-holiday)
- [`Ebb holiday add`↴](#Ebb-holiday-add)
- [`Ebb holiday edit`↴](#Ebb-holiday-edit)
- [`Ebb holiday list`↴](#Ebb-holiday-list)
- [`Ebb holiday remove`↴](#Ebb-holiday-remove)
- [`Ebb restart`↴](#Ebb-restart)
- [`Ebb sickday`↴](#Ebb-sickday)
- [`Ebb sickday add`↴](#Ebb-sickday-add)
- [`Ebb sickday edit`↴](#Ebb-sickday-edit)
- [`Ebb sickday list`↴](#Ebb-sickday-list)
- [`Ebb sickday remove`↴](#Ebb-sickday-remove)
- [`Ebb start`↴](#Ebb-start)
- [`Ebb status`↴](#Ebb-status)
- [`Ebb stop`↴](#Ebb-stop)
- [`Ebb vacation`↴](#Ebb-vacation)
- [`Ebb vacation add`↴](#Ebb-vacation-add)
- [`Ebb vacation edit`↴](#Ebb-vacation-edit)
- [`Ebb vacation list`↴](#Ebb-vacation-list)
- [`Ebb vacation remove`↴](#Ebb-vacation-remove)

## `Ebb`

**Usage:** `Ebb [OPTIONS] <COMMAND>`

###### **Subcommands:**

- `cancel` — Cancel the current time tracking frame
- `holiday` — Manage holidays
- `restart` — Restart the last project
- `sickday` — Manage sick days
- `start` — Start time tracking
- `status` — Show current time tracking status
- `stop` — Stop time tracking
- `vacation` — Manage vacation days

###### **Options:**

- `-c`, `--config-dir <CONFIG_DIR>` — Set the configuration directory

  Default value: `~/.config/ebb`

- `-f`, `--format <FORMAT>` — Set the output format

  Default value: `text`

  Possible values: `text`, `json`

## `Ebb cancel`

Cancel the current time tracking frame

**Usage:** `Ebb cancel`

## `Ebb holiday`

Manage holidays

**Usage:** `Ebb holiday
       holiday <COMMAND>`

###### **Subcommands:**

- `add` — Add a new holiday
- `edit` — Edit the description of an existing holiday
- `list` — List all holidays
- `remove` — Remove a holiday

## `Ebb holiday add`

Add a new holiday

**Usage:** `Ebb holiday add [OPTIONS] <DATE> [DESCRIPTION]`

###### **Arguments:**

- `<DATE>` — Date of the holiday (yyyy-mm-dd, e.g. 2025-08-11)
- `<DESCRIPTION>` — Name of the holiday (e.g. Mountain Day)

  Default value: `Holiday`

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Default value: `full`

  Possible values: `full`, `half`

## `Ebb holiday edit`

Edit the description of an existing holiday

**Usage:** `Ebb holiday edit [OPTIONS] <DATE> <DESCRIPTION>`

###### **Arguments:**

- `<DATE>` — Date of the holiday to edit
- `<DESCRIPTION>` — New name for the holiday

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Possible values: `full`, `half`

## `Ebb holiday list`

List all holidays

**Usage:** `Ebb holiday list [OPTIONS]`

###### **Options:**

- `-y`, `--year <YEAR>` — Filter by year

## `Ebb holiday remove`

Remove a holiday

**Usage:** `Ebb holiday remove <DATE>`

###### **Arguments:**

- `<DATE>` — Date of the holiday to remove

## `Ebb restart`

Restart the last project

**Usage:** `Ebb restart [OPTIONS]`

###### **Options:**

- `--at <AT>` — Time at which the project is restarted (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
- `-G`, `--no-gap` — Set the start time to the end time of the last saved frame

## `Ebb sickday`

Manage sick days

**Usage:** `Ebb sickday
       sickday <COMMAND>`

###### **Subcommands:**

- `add` — Add a new sick day
- `edit` — Edit the description of an existing sick day
- `list` — List all sick days
- `remove` — Remove a sick day

## `Ebb sickday add`

Add a new sick day

**Usage:** `Ebb sickday add [OPTIONS] <DATE> [DESCRIPTION]`

###### **Arguments:**

- `<DATE>` — Day of the sick day
- `<DESCRIPTION>` — Description for the sick day

  Default value: `Sick`

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Default value: `full`

  Possible values: `full`, `half`

## `Ebb sickday edit`

Edit the description of an existing sick day

**Usage:** `Ebb sickday edit [OPTIONS] <DATE> <DESCRIPTION>`

###### **Arguments:**

- `<DATE>` — Date of the sick day to edit
- `<DESCRIPTION>` — New description for the sick day

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Possible values: `full`, `half`

## `Ebb sickday list`

List all sick days

**Usage:** `Ebb sickday list [OPTIONS]`

###### **Options:**

- `-y`, `--year <YEAR>` — Filter by year

## `Ebb sickday remove`

Remove a sick day

**Usage:** `Ebb sickday remove <DATE>`

###### **Arguments:**

- `<DATE>` — Date of the sick day to remove

## `Ebb start`

Start time tracking

**Usage:** `Ebb start [OPTIONS] <PROJECT>`

###### **Arguments:**

- `<PROJECT>` — Name of the project

###### **Options:**

- `--at <AT>` — Time at which the project is started (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used
- `-G`, `--no-gap` — Set the start time to the end time of the last saved frame

## `Ebb status`

Show current time tracking status

**Usage:** `Ebb status`

## `Ebb stop`

Stop time tracking

**Usage:** `Ebb stop [OPTIONS]`

###### **Options:**

- `--at <AT>` — Time at which the project is stopped (hh:mm, hh:mm:ss, yyyy-mm-dd hh:mm, yyyy-mm-dd hh:mm:ss, or ISO 8601); if omitted, the current time is used

## `Ebb vacation`

Manage vacation days

**Usage:** `Ebb vacation
       vacation <COMMAND>`

###### **Subcommands:**

- `add` — Add a new vacation day
- `edit` — Edit the description of an existing vacation day
- `list` — List all vacation days
- `remove` — Remove a vacation day

## `Ebb vacation add`

Add a new vacation day

**Usage:** `Ebb vacation add [OPTIONS] <DATE> [DESCRIPTION]`

###### **Arguments:**

- `<DATE>` — Date of the vacation day
- `<DESCRIPTION>` — Name of the vacation day

  Default value: `Vacation`

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Default value: `full`

  Possible values: `full`, `half`

## `Ebb vacation edit`

Edit the description of an existing vacation day

**Usage:** `Ebb vacation edit [OPTIONS] <DATE> <DESCRIPTION>`

###### **Arguments:**

- `<DATE>` — Date of the vacation day to edit
- `<DESCRIPTION>` — New name for the vacation day

###### **Options:**

- `-p`, `--portion <PORTION>` — Switch between full-day and half-day holiday

  Possible values: `full`, `half`

## `Ebb vacation list`

List all vacation days

**Usage:** `Ebb vacation list [OPTIONS]`

###### **Options:**

- `-y`, `--year <YEAR>` — Filter by year

## `Ebb vacation remove`

Remove a vacation day

**Usage:** `Ebb vacation remove <DATE>`

###### **Arguments:**

- `<DATE>` — Date of the vacation day to remove

<hr/>

<small><i>
This document was generated automatically by
<a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
