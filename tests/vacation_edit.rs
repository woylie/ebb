// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use ebb::types::{DayPortion, VacationEntry};
use predicates::str::contains;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn edit_vacation_updates_entry() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("vacations.toml");
    let toml_content = r#"
        [2025-05-28]
        description = "Mountain Day"
        portion = "full"

        [2025-05-29]
        description = "Water Day"
        portion = "half"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("edit")
        .arg("2025-05-29")
        .arg("Ocean Day")
        .arg("-p")
        .arg("full")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("vacations.toml");
    let contents = fs::read_to_string(file)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-29").unwrap().description, "Ocean Day");
    assert_eq!(parsed.get("2025-05-29").unwrap().portion, DayPortion::Full);

    Ok(())
}

#[test]
fn edit_vacation_fails_if_not_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("vacations.toml");
    let toml_content = r#"
        [2025-05-29]
        description = "Water Day"
        portion = "half"
    "#;
    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("edit")
        .arg("2025-05-28")
        .arg("Ocean Day")
        .arg("-p")
        .arg("full")
        .env("EBB_DATA_DIR", tmp.path());

    cmd.assert()
        .failure()
        .stderr(contains("No vacation found on 2025-05-28"));

    let contents = fs::read_to_string(&file_path)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-29").unwrap().description, "Water Day");

    Ok(())
}

#[test]
fn edit_vacation_keeps_the_portion_without_portion_option() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;
    let file = tmp.path().join("vacations.toml");
    fs::write(
        &file,
        "[2026-10-06]\ndescription = \"Trip\"\nportion = \"half\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("edit")
        .arg("2026-10-06")
        .arg("Short trip")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let contents = fs::read_to_string(&file)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;
    assert_eq!(parsed["2026-10-06"].description, "Short trip");
    assert_eq!(parsed["2026-10-06"].portion, DayPortion::Half);

    Ok(())
}

#[test]
fn edit_vacation_sets_the_portion_with_portion_option() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let file = tmp.path().join("vacations.toml");
    fs::write(
        &file,
        "[2026-10-06]\ndescription = \"Trip\"\nportion = \"half\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("edit")
        .arg("2026-10-06")
        .arg("Trip")
        .arg("--portion")
        .arg("full")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let contents = fs::read_to_string(&file)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;
    assert_eq!(parsed["2026-10-06"].portion, DayPortion::Full);

    Ok(())
}
