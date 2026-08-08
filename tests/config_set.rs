// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use ebb::types::Config;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn config_set_sets_working_hour() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let expected_output = "\
Key: working_hours.wednesday
Old value: 8h
New value: 4h
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("set")
        .arg("working_hours.wednesday")
        .arg("4h")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    let file = tmp.path().join("config.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let parsed: Config = toml::from_str(&contents)?;

    assert_eq!(
        parsed.working_hours.wednesday,
        Duration::from_secs(60 * 60 * 4)
    );

    Ok(())
}

#[test]
fn config_set_sets_sick_days_per_year() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let expected_output = "\
Key: sick_days_per_year.2010
Old value: null
New value: 38
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("set")
        .arg("sick_days_per_year.2010")
        .arg("38")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    let file = tmp.path().join("config.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let parsed: Config = toml::from_str(&contents)?;
    let expected_days: i32 = 38;

    assert_eq!(
        parsed.sick_days_per_year.get(&2010).unwrap(),
        &expected_days
    );

    Ok(())
}

#[test]
fn config_set_sets_vacation_days_per_year() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let expected_output = "\
Key: vacation_days_per_year.2010
Old value: null
New value: 38
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("set")
        .arg("vacation_days_per_year.2010")
        .arg("38")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    let file = tmp.path().join("config.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let parsed: Config = toml::from_str(&contents)?;
    let expected_days: i32 = 38;

    assert_eq!(
        parsed.vacation_days_per_year.get(&2010).unwrap(),
        &expected_days
    );

    Ok(())
}

#[test]
fn config_set_rejects_an_unknown_key() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("set")
        .arg("working_hours.mondey")
        .arg("6h")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown field `mondey`"));

    assert!(!tmp.path().join("config.toml").exists());

    Ok(())
}

#[test]
fn config_set_rejects_an_unknown_top_level_key() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("set")
        .arg("workinghours")
        .arg("6h")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown field `workinghours`"));

    Ok(())
}

#[test]
fn config_set_rejects_a_non_year_key_under_the_year_maps() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("set")
        .arg("vacation_days_per_year.foo")
        .arg("25")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid integer key: foo"));

    Ok(())
}

#[test]
fn config_list_rejects_an_unknown_key_in_the_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    fs::write(
        tmp.path().join("config.toml"),
        "[working_hours]\nmonday = \"6h\"\nmondey = \"9h\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("list")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown field `mondey`"));

    Ok(())
}
