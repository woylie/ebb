// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use ebb::types::{DayPortion, HolidayEntry};
use predicates::str::contains;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn add_holiday_creates_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("holiday")
        .arg("add")
        .arg("2025-05-28")
        .arg("Mountain Day")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("holidays.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let parsed: BTreeMap<String, HolidayEntry> = toml::from_str(&contents)?;

    assert_eq!(
        parsed.get("2025-05-28").unwrap().description,
        "Mountain Day"
    );
    assert_eq!(parsed.get("2025-05-28").unwrap().portion, DayPortion::Full);

    Ok(())
}

#[test]
fn add_holiday_fails_if_date_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("holidays.toml");
    fs::write(
        &file_path,
        r#"2025-05-28 = {"description" = "fever", "portion" = "half"}"#,
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("holiday")
        .arg("add")
        .arg("2025-05-28")
        .arg("Mountain Day")
        .env("EBB_CONFIG_DIR", tmp.path());

    cmd.assert().failure().stderr(contains("already exists"));

    let contents = fs::read_to_string(&file_path)?;
    let parsed: BTreeMap<String, HolidayEntry> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-28").unwrap().description, "fever");
    assert_eq!(parsed.get("2025-05-28").unwrap().portion, DayPortion::Half);

    Ok(())
}
