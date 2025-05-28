use chrono::NaiveDate;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DayPortion {
    Full,
    Half,
}

fn default_portion() -> DayPortion {
    DayPortion::Full
}

fn is_default_portion(p: &DayPortion) -> bool {
    *p == DayPortion::Full
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SickdayEntry {
    pub description: String,

    #[serde(default = "default_portion", skip_serializing_if = "is_default_portion")]
    pub portion: DayPortion,
}

pub type Sickdays = BTreeMap<NaiveDate, SickdayEntry>;
