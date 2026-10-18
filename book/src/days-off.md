# Holidays, Vacation Days, and Sick Days

You can manage holidays, vacation days, and sick days with `ebb holiday`,
`ebb vacation`, and `ebb sickday`. Run the commands without further arguments
to see the command line help.

Holidays can be national holidays or other days on which you are not expected to
work or log hours.

You can render an overview with the count of taken and remaining vacation days
and sick days for the current year or other years with `ebb daysoff` or
`ebb daysoff --year 2024`.

## Important to Know

Vacation days, holidays, and sick days can be full or half days. This is
relative to the expected working hours configured for that day. For example,
if Monday is configured with 8 expected working hours, a full day off will
reduce the flex time balance by 8 hours, and a half day off will
reduce it by 4 hours. If a day is configured with 6 expected working hours,
a full day off reduces the balance by 6 hours, and a half day off reduces the
balance by 3 hours. This logic may be improved in the future based on user
feedback.
