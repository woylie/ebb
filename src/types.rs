// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::NaiveDate;
use clap::ValueEnum;
use humantime::format_duration;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;
use tabled::Tabled;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_working_hours")]
    pub working_hours: WorkingHours,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkingHours {
    #[serde(
        deserialize_with = "deserialize_duration_human",
        serialize_with = "serialize_duration_human"
    )]
    pub monday: Duration,
    #[serde(
        deserialize_with = "deserialize_duration_human",
        serialize_with = "serialize_duration_human"
    )]
    pub tuesday: Duration,
    #[serde(
        deserialize_with = "deserialize_duration_human",
        serialize_with = "serialize_duration_human"
    )]
    pub wednesday: Duration,
    #[serde(
        deserialize_with = "deserialize_duration_human",
        serialize_with = "serialize_duration_human"
    )]
    pub thursday: Duration,
    #[serde(
        deserialize_with = "deserialize_duration_human",
        serialize_with = "serialize_duration_human"
    )]
    pub friday: Duration,
    #[serde(
        deserialize_with = "deserialize_duration_human",
        serialize_with = "serialize_duration_human"
    )]
    pub saturday: Duration,
    #[serde(
        deserialize_with = "deserialize_duration_human",
        serialize_with = "serialize_duration_human"
    )]
    pub sunday: Duration,
}

fn default_working_hours() -> WorkingHours {
    let eight_hours = Duration::from_secs(60 * 60 * 8);
    let zero_hours = Duration::from_secs(0);

    WorkingHours {
        monday: eight_hours,
        tuesday: eight_hours,
        wednesday: eight_hours,
        thursday: eight_hours,
        friday: eight_hours,
        saturday: zero_hours,
        sunday: zero_hours,
    }
}

fn serialize_duration_human<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format_duration(*duration).to_string();
    serializer.serialize_str(&s)
}

fn deserialize_duration_human<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    humantime::parse_duration(&s).map_err(serde::de::Error::custom)
}

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

#[derive(Clone, Debug, Serialize, Deserialize, Tabled)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Tabled)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Tabled)]
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
