use chrono::NaiveDate;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

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
pub struct SickdayEntry {
    pub description: String,

    #[serde(
        default = "default_portion",
        skip_serializing_if = "is_default_portion"
    )]
    pub portion: DayPortion,
}

pub type Sickdays = BTreeMap<NaiveDate, SickdayEntry>;
