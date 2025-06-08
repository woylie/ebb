// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use ebb::types::{Frames, State};
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn start_creates_state_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("start")
        .arg("myproject")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("state.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let state: State = toml::from_str(&contents)?;

    assert_eq!(state.current_frame.unwrap().project, "myproject");

    Ok(())
}

#[test]
fn start_updates_empty_state_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("state.toml");
    fs::write(&file_path, "")?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("start")
        .arg("myproject")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("state.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let state: State = toml::from_str(&contents)?;

    assert_eq!(state.current_frame.unwrap().project, "myproject");

    Ok(())
}

#[test]
fn start_saves_tags() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("start")
        .arg("myproject")
        .arg("+tag1")
        .arg("+tag2")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("state.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let state: State = toml::from_str(&contents)?;

    assert_eq!(state.current_frame.unwrap().tags, vec!["tag1", "tag2"]);

    Ok(())
}

#[test]
fn start_stops_current_frame_and_updates_state_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("state.toml");
    let toml_content = r#"
        [current_frame]
        start_time = 1748723006
        project = "firstproject"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("start")
        .arg("secondproject")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let state_file = tmp.path().join("state.toml");
    assert!(state_file.exists());

    let state_contents = fs::read_to_string(state_file)?;
    let state: State = toml::from_str(&state_contents)?;
    assert_eq!(
        state.current_frame.clone().unwrap().project,
        "secondproject"
    );
    let start_time = state.current_frame.unwrap().start_time;

    let frames_file = tmp.path().join("frames.toml");
    assert!(frames_file.exists());

    let frame_contents = fs::read_to_string(frames_file)?;
    let frames: Frames = toml::from_str(&frame_contents)?;

    let last_frame = frames.frames.last().expect("No frames found");
    assert_eq!(start_time, last_frame.end_time);

    Ok(())
}

#[test]
fn start_applies_no_gap_option() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748725744
        project = "firstproject"
        updated_at = 1748725744
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("start")
        .arg("secondproject")
        .arg("--no-gap")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let state_file = tmp.path().join("state.toml");
    assert!(state_file.exists());

    let state_contents = fs::read_to_string(state_file)?;
    let state: State = toml::from_str(&state_contents)?;
    assert_eq!(state.current_frame.clone().unwrap().start_time, 1748725744);

    Ok(())
}

#[test]
fn start_returns_error_if_start_time_overlaps() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748725744
        project = "firstproject"
        updated_at = 1748725744
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("start")
        .arg("secondproject")
        .arg("--at")
        .arg("1748725743")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("is before the end of the last frame"));

    Ok(())
}

#[test]
fn start_fails_with_both_no_gap_and_at() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let mut cmd = Command::cargo_bin("ebb")?;

    cmd.arg("start")
        .arg("project")
        .arg("--at")
        .arg("1748725743")
        .arg("--no-gap")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("Cannot use --at and --no-gap together."));

    Ok(())
}

fn assert_start_time_at(
    time_str: &str,
    expected_dt: DateTime<Local>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("start")
        .arg("myproject")
        .arg("--at")
        .arg(time_str)
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("state.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let state: State = toml::from_str(&contents)?;

    let saved_start_time = state
        .current_frame
        .as_ref()
        .expect("No current_frame found")
        .start_time;

    let expected_timestamp = expected_dt.timestamp();

    assert_eq!(
        saved_start_time, expected_timestamp,
        "Start time in state.toml did not match --at argument '{}'",
        time_str
    );

    Ok(())
}

#[test]
fn start_applies_at_option_with_hour_minute_second() -> Result<(), Box<dyn std::error::Error>> {
    let today = Local::now().date_naive();
    let naive_time = NaiveTime::from_hms_opt(12, 24, 14).unwrap();
    let naive_dt = NaiveDateTime::new(today, naive_time);
    let expected_dt = Local
        .from_local_datetime(&naive_dt)
        .single()
        .expect("Ambiguous local datetime");

    assert_start_time_at("12:24:14", expected_dt)
}

#[test]
fn start_applies_at_option_with_date_hour_minute() -> Result<(), Box<dyn std::error::Error>> {
    let naive_date = NaiveDate::from_ymd_opt(2013, 1, 5).unwrap();
    let naive_time = NaiveTime::from_hms_opt(12, 23, 0).unwrap();
    let naive_dt = NaiveDateTime::new(naive_date, naive_time);
    let expected_dt = Local
        .from_local_datetime(&naive_dt)
        .single()
        .expect("Ambiguous local datetime");

    assert_start_time_at("2013-01-05 12:23", expected_dt)
}

#[test]
fn start_applies_at_option_with_date_hour_minute_second() -> Result<(), Box<dyn std::error::Error>>
{
    let naive_date = NaiveDate::from_ymd_opt(2013, 1, 5).unwrap();
    let naive_time = NaiveTime::from_hms_opt(12, 23, 45).unwrap();
    let naive_dt = NaiveDateTime::new(naive_date, naive_time);
    let expected_dt = Local
        .from_local_datetime(&naive_dt)
        .single()
        .expect("Ambiguous local datetime");

    assert_start_time_at("2013-01-05 12:23:45", expected_dt)
}

#[test]
fn start_applies_at_option_with_iso8601() -> Result<(), Box<dyn std::error::Error>> {
    let dt_fixed = DateTime::parse_from_rfc3339("2013-01-05T12:23:45+00:00")?;
    let expected_dt = dt_fixed.with_timezone(&Local);

    assert_start_time_at("2013-01-05T12:23:45+00:00", expected_dt)
}

#[test]
fn start_applies_at_option_with_unix_timestamp() -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = 1357388625;

    let expected_dt = chrono::Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .ok_or("Invalid timestamp")?
        .with_timezone(&chrono::Local);

    assert_start_time_at(&timestamp.to_string(), expected_dt)
}
