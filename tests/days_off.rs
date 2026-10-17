// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use chrono::{Datelike, Local};
use std::fs;
use tempfile::tempdir;

#[test]
fn daysoff_prints_overview() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let config_dir = tmp.path();

    let file_path = config_dir.join("config.toml");
    let toml_content = r#"
        [sick_days_per_year]
        2004 = 28

        [vacation_days_per_year]
        2004 = 30
    "#;
    fs::write(&file_path, toml_content.trim())?;

    let file_path = config_dir.join("vacations.toml");
    let toml_content = r#"
        [2003-04-01]
        description = "Vacation"
        portion = "full"

        [2004-02-05]
        description = "Vacation"
        portion = "half"

        [2004-08-12]
        description = "Vacation"
        portion = "full"

        [2005-06-01]
        description = "Vacation"
        portion = "full"
    "#;
    fs::write(&file_path, toml_content.trim())?;

    let file_path = config_dir.join("sick_days.toml");
    let toml_content = r#"
        [2003-04-01]
        description = "Sick"
        portion = "full"

        [2004-02-05]
        description = "Sick"
        portion = "half"

        [2004-08-12]
        description = "Sick"
        portion = "full"

        [2005-06-01]
        description = "Sick"
        portion = "full"
    "#;
    fs::write(&file_path, toml_content.trim())?;

    let expected_output = "\
Year: 2004

┌──────────┬─────────┬───────┬───────────┐
│ Category │ Allowed │ Taken │ Remaining │
├──────────┼─────────┼───────┼───────────┤
│ Vacation │      30 │   1.5 │      28.5 │
│ Sick     │      28 │   1.5 │      26.5 │
└──────────┴─────────┴───────┴───────────┘
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("daysoff")
        .arg("--year")
        .arg("2004")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}

#[test]
fn daysoff_defaults_to_the_current_year() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let current_year = Local::now().year();

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("daysoff")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("Year: {current_year}")));

    Ok(())
}

#[test]
fn daysoff_counts_no_day_on_a_weekend_or_holiday() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // A Saturday, and a Tuesday that is a holiday.
    let toml_content = r#"
        [2026-10-10]
        description = "Vacation"

        [2026-10-06]
        description = "Vacation"
        portion = "half"
    "#;
    fs::write(tmp.path().join("vacations.toml"), toml_content.trim())?;

    let toml_content = r#"
        [2026-10-06]
        description = "Holiday"
    "#;
    fs::write(tmp.path().join("holidays.toml"), toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("daysoff")
        .arg("--year")
        .arg("2026")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(output["vacation_days_taken"], 0.0);

    Ok(())
}

#[test]
fn daysoff_counts_half_a_day_on_a_half_holiday() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // A Tuesday that is a half holiday.
    let toml_content = r#"
        [2026-10-06]
        description = "Vacation"
    "#;
    fs::write(tmp.path().join("vacations.toml"), toml_content.trim())?;

    let toml_content = r#"
        [2026-10-06]
        description = "Holiday"
        portion = "half"
    "#;
    fs::write(tmp.path().join("holidays.toml"), toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("daysoff")
        .arg("--year")
        .arg("2026")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(output["vacation_days_taken"], 0.5);

    Ok(())
}
