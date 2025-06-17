// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::NaiveDate;
use clap::ValueEnum;
use humantime::format_duration;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::time::Duration;
use tabled::Tabled;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(
        default = "default_vacation_days_per_year",
        serialize_with = "serialize_map_keys_as_strings",
        deserialize_with = "deserialize_map_keys_as_i32"
    )]
    pub vacation_days_per_year: HashMap<i32, i32>,
    #[serde(
        default = "default_sick_days_per_year",
        serialize_with = "serialize_map_keys_as_strings",
        deserialize_with = "deserialize_map_keys_as_i32"
    )]
    pub sick_days_per_year: HashMap<i32, i32>,
    #[serde(default = "default_working_hours")]
    pub working_hours: WorkingHours,
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

pub fn default_vacation_days_per_year() -> HashMap<i32, i32> {
    HashMap::from([(2000, 30)])
}

pub fn default_sick_days_per_year() -> HashMap<i32, i32> {
    HashMap::from([(2000, 30)])
}

pub fn default_working_hours() -> WorkingHours {
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

fn deserialize_map_keys_as_i32<'de, D>(deserializer: D) -> Result<HashMap<i32, i32>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MapVisitor;

    impl<'de> Visitor<'de> for MapVisitor {
        type Value = HashMap<i32, i32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with string keys that can be parsed as integers")
        }

        fn visit_map<M>(self, mut access: M) -> Result<HashMap<i32, i32>, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::new();
            while let Some((key, value)) = access.next_entry::<String, i32>()? {
                let parsed_key = key
                    .parse::<i32>()
                    .map_err(|_| de::Error::custom(format!("Invalid integer key: {}", key)))?;
                map.insert(parsed_key, value);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(MapVisitor)
}

pub fn serialize_map_keys_as_strings<S, V>(
    map: &HashMap<i32, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Serialize,
{
    let mut keys: Vec<_> = map.keys().cloned().collect();
    keys.sort();

    let mut ser_map = serializer.serialize_map(Some(map.len()))?;
    for key in keys {
        if let Some(value) = map.get(&key) {
            ser_map.serialize_entry(&key.to_string(), value)?;
        }
    }
    ser_map.end()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CurrentFrame {
    pub start_time: i64,
    pub project: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
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
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
pub struct SickDay {
    pub date: NaiveDate,
    pub description: String,

    #[serde(
        default = "default_portion",
        skip_serializing_if = "is_default_portion"
    )]
    pub portion: DayPortion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SickDayEntry {
    pub description: String,

    #[serde(
        default = "default_portion",
        skip_serializing_if = "is_default_portion"
    )]
    pub portion: DayPortion,
}

pub type SickDays = BTreeMap<NaiveDate, SickDayEntry>;

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
