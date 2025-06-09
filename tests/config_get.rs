// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn config_get_returns_working_hours() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let expected_output = "\
8h
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("get")
        .arg("working_hours.wednesday")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}

#[test]
fn config_get_returns_sick_days() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let expected_output = "\
30
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("get")
        .arg("sick_days_per_year.2000")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}

#[test]
fn config_get_returns_vacation_days() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let expected_output = "\
30
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("get")
        .arg("vacation_days_per_year.2000")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}
