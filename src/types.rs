// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::serde_utils;
use chrono::NaiveDate;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::time::Duration;
use tabled::Tabled;

const HOUR: u64 = 60 * 60;
const EIGHT_HOURS: Duration = Duration::from_secs(8 * HOUR);
const ZERO_HOURS: Duration = Duration::from_secs(0);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(with = "serde_utils::int_key_map")]
    pub vacation_days_per_year: HashMap<i32, i32>,
    #[serde(with = "serde_utils::int_key_map")]
    pub sick_days_per_year: HashMap<i32, i32>,
    pub working_hours: WorkingHours,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sick_days_per_year: HashMap::from([(2000, 30)]),
            vacation_days_per_year: HashMap::from([(2000, 30)]),
            working_hours: WorkingHours::default(),
        }
    }
}

impl Config {
    pub fn allowed_vacation_days(&self, year: i32) -> i32 {
        find_allowed_for_year(&self.vacation_days_per_year, year)
    }

    pub fn allowed_sick_days(&self, year: i32) -> i32 {
        find_allowed_for_year(&self.sick_days_per_year, year)
    }
}

fn find_allowed_for_year(map: &HashMap<i32, i32>, year: i32) -> i32 {
    map.iter()
        .filter(|entry| {
            let (from_year, _) = *entry;
            *from_year <= year
        })
        .max_by_key(|entry| {
            let (from_year, _) = *entry;
            *from_year
        })
        .map(|(_, &days)| days)
        .unwrap_or(0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkingHours {
    #[serde(with = "serde_utils::human_duration")]
    pub monday: Duration,
    #[serde(with = "serde_utils::human_duration")]
    pub tuesday: Duration,
    #[serde(with = "serde_utils::human_duration")]
    pub wednesday: Duration,
    #[serde(with = "serde_utils::human_duration")]
    pub thursday: Duration,
    #[serde(with = "serde_utils::human_duration")]
    pub friday: Duration,
    #[serde(with = "serde_utils::human_duration")]
    pub saturday: Duration,
    #[serde(with = "serde_utils::human_duration")]
    pub sunday: Duration,
}

impl Default for WorkingHours {
    fn default() -> Self {
        WorkingHours {
            monday: EIGHT_HOURS,
            tuesday: EIGHT_HOURS,
            wednesday: EIGHT_HOURS,
            thursday: EIGHT_HOURS,
            friday: EIGHT_HOURS,
            saturday: ZERO_HOURS,
            sunday: ZERO_HOURS,
        }
    }
}

impl WorkingHours {
    pub fn total_weekly_duration(&self) -> Duration {
        self.monday
            + self.tuesday
            + self.wednesday
            + self.thursday
            + self.friday
            + self.saturday
            + self.sunday
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurrentFrame {
    pub start_time: i64,
    pub project: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DayPortion {
    #[default]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub start_time: i64,
    pub end_time: i64,
    pub project: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Frames {
    pub frames: Vec<Frame>,
}

impl Frames {
    pub fn filter_by_start_time(&mut self, from: i64) -> &mut Self {
        self.frames.retain_mut(|frame| {
            if frame.start_time < from && frame.end_time > from {
                frame.start_time = from;
                true
            } else {
                frame.start_time >= from
            }
        });
        self
    }

    pub fn filter_by_end_time(&mut self, to: i64) -> &mut Self {
        self.frames.retain_mut(|frame| {
            if frame.start_time < to && frame.end_time > to {
                frame.end_time = to;
                true
            } else {
                frame.end_time <= to
            }
        });
        self
    }

    pub fn filter_by_project(&mut self, project: &str) -> &mut Self {
        self.frames.retain(|frame| frame.project == *project);
        self
    }

    pub fn filter_by_tag(&mut self, tag: &str) -> &mut Self {
        self.frames
            .retain(|frame| frame.tags.contains(&tag.to_string()));
        self
    }

    pub fn all_projects(&self) -> Vec<String> {
        let mut project_set: HashSet<String> = HashSet::new();

        for frame in &self.frames {
            project_set.insert(frame.project.clone());
        }

        let mut projects: Vec<String> = project_set.into_iter().collect();
        projects.sort();
        projects
    }

    pub fn all_tags(&self) -> Vec<String> {
        let mut tag_set: HashSet<String> = HashSet::new();

        for frame in &self.frames {
            for tag in &frame.tags {
                tag_set.insert(tag.clone());
            }
        }

        let mut tags: Vec<String> = tag_set.into_iter().collect();
        tags.sort();
        tags
    }

    pub fn rename_project(&mut self, old_name: &str, new_name: &str) {
        for frame in &mut self.frames {
            if frame.project == old_name {
                frame.project = new_name.to_string();
            }
        }
    }

    pub fn rename_tag(&mut self, old_name: &str, new_name: &str) {
        for frame in &mut self.frames {
            let mut changed = false;

            for tag in &mut frame.tags {
                if tag == old_name {
                    *tag = new_name.to_string();
                    changed = true;
                }
            }

            if changed {
                frame.tags.sort();
                frame.tags.dedup();
            }
        }
    }

    pub fn remove_tag(&mut self, tag_to_remove: &str) -> &mut Self {
        for frame in &mut self.frames {
            frame.tags.retain(|tag| tag != tag_to_remove);
        }
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Tabled)]
pub struct Holiday {
    pub date: NaiveDate,
    pub description: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub portion: DayPortion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HolidayEntry {
    pub description: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub portion: DayPortion,
}

pub type Holidays = BTreeMap<NaiveDate, HolidayEntry>;

#[derive(Clone, Debug, Serialize, Deserialize, Tabled)]
pub struct SickDay {
    pub date: NaiveDate,
    pub description: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub portion: DayPortion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SickDayEntry {
    pub description: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub portion: DayPortion,
}

pub type SickDays = BTreeMap<NaiveDate, SickDayEntry>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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

    #[serde(default, skip_serializing_if = "is_default")]
    pub portion: DayPortion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VacationEntry {
    pub description: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub portion: DayPortion,
}

pub type Vacations = BTreeMap<NaiveDate, VacationEntry>;

fn is_default<T>(value: &T) -> bool
where
    T: Default + PartialEq,
{
    *value == T::default()
}
