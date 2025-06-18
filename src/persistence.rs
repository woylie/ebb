// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::types::{Config, Frames, Holidays, SickDays, State, Vacations};
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::fs;
use std::path::Path;

const CONFIG_FILE: &str = "config.toml";
const FRAME_FILE: &str = "frames.toml";
const HOLIDAY_FILE: &str = "holidays.toml";
const SICK_DAY_FILE: &str = "sick_days.toml";
const STATE_FILE: &str = "state.toml";
const VACATION_FILE: &str = "vacations.toml";

fn load_toml<T: DeserializeOwned>(config_path: &Path, filename: &str, default: T) -> Result<T> {
    let path = config_path.join(filename);
    if !path.exists() {
        return Ok(default);
    }
    let contents = fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

fn save_toml<T: Serialize>(config_path: &Path, filename: &str, value: &T) -> Result<()> {
    let path = config_path.join(filename);
    let toml = toml::to_string(value)?;
    fs::write(path, toml)?;
    Ok(())
}

pub fn load_config(config_path: &Path) -> Result<Config> {
    load_toml(config_path, CONFIG_FILE, Config::default())
}

pub fn save_config(config_path: &Path, config: &Config) -> Result<()> {
    save_toml(config_path, CONFIG_FILE, config)
}

pub fn load_frames(config_path: &Path) -> Result<Frames> {
    load_toml(config_path, FRAME_FILE, Frames::default())
}

pub fn save_frames(config_path: &Path, frames: &Frames) -> Result<()> {
    save_toml(config_path, FRAME_FILE, frames)
}

pub fn load_holidays(config_path: &Path) -> Result<Holidays> {
    load_toml(config_path, HOLIDAY_FILE, Holidays::default())
}

pub fn save_holidays(config_path: &Path, holidays: &Holidays) -> Result<()> {
    save_toml(config_path, HOLIDAY_FILE, holidays)
}

pub fn load_sick_days(config_path: &Path) -> Result<SickDays> {
    load_toml(config_path, SICK_DAY_FILE, SickDays::default())
}

pub fn save_sick_days(config_path: &Path, sick_days: &SickDays) -> Result<()> {
    save_toml(config_path, SICK_DAY_FILE, sick_days)
}

pub fn load_state(config_path: &Path) -> Result<State> {
    load_toml(config_path, STATE_FILE, State::default())
}

pub fn save_state(config_path: &Path, state: &State) -> Result<()> {
    save_toml(config_path, STATE_FILE, state)
}

pub fn load_vacations(config_path: &Path) -> Result<Vacations> {
    load_toml(config_path, VACATION_FILE, Vacations::default())
}

pub fn save_vacations(config_path: &Path, vacations: &Vacations) -> Result<()> {
    save_toml(config_path, VACATION_FILE, vacations)
}
