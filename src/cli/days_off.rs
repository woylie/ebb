// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::output::{DisplayOutput, print_output};
use crate::persistence::{load_config, load_sick_days, load_vacations};
use crate::types::DayPortion;
use crate::{DaysOffArgs, Format};
use chrono::Datelike;
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use tabled::settings::{Alignment, Style, object::Columns};
use tabled::{Table, Tabled};

#[derive(Serialize)]
struct Output {
    sick_days_taken: f32,
    sick_days_allowed: i32,
    sick_days_remaining: f32,
    vacation_days_taken: f32,
    vacation_days_allowed: i32,
    vacation_days_remaining: f32,
    year: i32,
}

#[derive(Tabled)]
struct SummaryRow {
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Allowed")]
    allowed: String,
    #[tabled(rename = "Taken")]
    taken: String,
    #[tabled(rename = "Remaining")]
    remaining: String,
}

impl DisplayOutput for Output {
    fn to_text(&self) -> String {
        let rows = vec![
            SummaryRow {
                category: "Vacation".into(),
                taken: format!("{:.1}", self.vacation_days_taken),
                allowed: self.vacation_days_allowed.to_string(),
                remaining: format!("{:.1}", self.vacation_days_remaining),
            },
            SummaryRow {
                category: "Sick".into(),
                taken: format!("{:.1}", self.sick_days_taken),
                allowed: self.sick_days_allowed.to_string(),
                remaining: format!("{:.1}", self.sick_days_remaining),
            },
        ];

        let mut table = Table::new(rows);
        table
            .with(Style::sharp())
            .modify(Columns::new(1..), Alignment::right());

        format!("Year: {}\n\n{}", self.year, table)
    }
}

pub fn run_daysoff(args: &DaysOffArgs, config_path: &Path, format: &Format) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
    let mut sick_days = load_sick_days(config_path)?;
    let mut vacations = load_vacations(config_path)?;

    filter_by_year(&mut sick_days, args.year);
    let sick_days_taken = count_days(sick_days.values().map(|v| &v.portion));
    let sick_days_allowed = config.allowed_sick_days(args.year);

    filter_by_year(&mut vacations, args.year);
    let vacation_days_taken = count_days(vacations.values().map(|v| &v.portion));
    let vacation_days_allowed = config.allowed_vacation_days(args.year);

    let vacation_days_remaining =
        normalize_zero(vacation_days_allowed as f32 - vacation_days_taken);
    let sick_days_remaining = normalize_zero(sick_days_allowed as f32 - sick_days_taken);

    let output = Output {
        year: args.year,
        vacation_days_taken: normalize_zero(vacation_days_taken),
        vacation_days_allowed,
        vacation_days_remaining,
        sick_days_taken: normalize_zero(sick_days_taken),
        sick_days_allowed,
        sick_days_remaining,
    };

    print_output(&output, format)?;

    Ok(())
}

pub fn filter_by_year<T>(map: &mut BTreeMap<NaiveDate, T>, year: i32) {
    map.retain(|date, _| date.year() == year);
}

pub fn count_days<'a, I>(portions: I) -> f32
where
    I: Iterator<Item = &'a DayPortion>,
{
    portions
        .map(|portion| match portion {
            DayPortion::Full => 1.0,
            DayPortion::Half => 0.5,
        })
        .sum()
}

fn normalize_zero(x: f32) -> f32 {
    if x == 0.0 { 0.0 } else { x }
}
