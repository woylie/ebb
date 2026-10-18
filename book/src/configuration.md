# Configuration

You can list all available configuration options with:

```bash
ebb config list
```

Modify the defaults with:

```bash
ebb config set <KEY> <VALUE>
```

For example: `ebb config set working_hours.monday 6h`

- `sick_days_per_year.<YEAR>` - Defines how many sick days you are allowed to
  take in a year. The value applies to the specified year and all subsequent
  years, until a new value is set for a later year.
- `vacation_days_per_year.<YEAR>` - Specifies the number of vacation days
  allowed per year, starting from the given year. The value applies to the
  specified year and all subsequent years, until a new value is set for a later
  year.
- `working_hours.<WEEKDAY>` - Defines how many hours you are supposed to work on
  a weekday. The value is a duration string, e.g. `"6h"` or `"7h 30m"`.
