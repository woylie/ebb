// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::NaiveDate;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurrentFrame {
    pub start_time: i64,
    pub project: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DayPortion {
    Full,
    Half,
}

impl fmt::Display for DayPortion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DayPortion::Full => "full",
            DayPortion::Half => "half",
        };
        write!(f, "{}", s)
    }
}

fn default_portion() -> DayPortion {
    DayPortion::Full
}

fn is_default_portion(p: &DayPortion) -> bool {
    *p == DayPortion::Full
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub start_time: i64,
    pub end_time: i64,
    pub project: String,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frames {
    pub frames: Vec<Frame>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Holiday {
    pub date: NaiveDate,
    pub description: String,

    #[serde(
        default = "default_portion",
        skip_serializing_if = "is_default_portion"
    )]
    pub portion: DayPortion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HolidayEntry {
    pub description: String,

    #[serde(
        default = "default_portion",
        skip_serializing_if = "is_default_portion"
    )]
    pub portion: DayPortion,
}

pub type Holidays = BTreeMap<NaiveDate, HolidayEntry>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sickday {
    pub date: NaiveDate,
    pub description: String,

    #[serde(
        default = "default_portion",
        skip_serializing_if = "is_default_portion"
    )]
    pub portion: DayPortion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SickdayEntry {
    pub description: String,

    #[serde(
        default = "default_portion",
        skip_serializing_if = "is_default_portion"
    )]
    pub portion: DayPortion,
}

pub type Sickdays = BTreeMap<NaiveDate, SickdayEntry>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub current_frame: Option<CurrentFrame>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Timespan {
    pub from: i64,
    pub to: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vacation {
    pub date: NaiveDate,
    pub description: String,

    #[serde(
        default = "default_portion",
        skip_serializing_if = "is_default_portion"
    )]
    pub portion: DayPortion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VacationEntry {
    pub description: String,

    #[serde(
        default = "default_portion",
        skip_serializing_if = "is_default_portion"
    )]
    pub portion: DayPortion,
}

pub type Vacations = BTreeMap<NaiveDate, VacationEntry>;
