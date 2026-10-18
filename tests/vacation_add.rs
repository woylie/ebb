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
fn add_vacation_creates_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2025-05-28")
        .arg("Mountain Day")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("vacations.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;

    assert_eq!(
        parsed.get("2025-05-28").unwrap().description,
        "Mountain Day"
    );
    assert_eq!(parsed.get("2025-05-28").unwrap().portion, DayPortion::Full);

    Ok(())
}

#[test]
fn add_vacation_fails_if_date_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("vacations.toml");
    fs::write(
        &file_path,
        r#"2025-05-28 = {"description" = "fever", "portion" = "half"}"#,
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2025-05-28")
        .arg("Mountain Day")
        .env("EBB_DATA_DIR", tmp.path());

    cmd.assert().failure().stderr(contains("already exists"));

    let contents = fs::read_to_string(&file_path)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-28").unwrap().description, "fever");
    assert_eq!(parsed.get("2025-05-28").unwrap().portion, DayPortion::Half);

    Ok(())
}

#[test]
fn add_vacation_prints_the_entry() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2025-05-28")
        .arg("Some day")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stdout("Vacation 'Some day' added on 2025-05-28.\n");

    Ok(())
}

#[test]
fn add_vacation_prints_json() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("vacation")
        .arg("add")
        .arg("2025-05-28")
        .arg("Some day")
        .arg("--portion")
        .arg("half")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let json: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(
        json,
        serde_json::json!({
            "vacation": {"date": "2025-05-28", "description": "Some day", "portion": "half"}
        })
    );

    Ok(())
}

#[test]
fn add_vacation_range_records_only_working_days() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    fs::write(
        tmp.path().join("holidays.toml"),
        "[2027-01-01]\ndescription = \"New Year\"\n\n[2026-12-30]\ndescription = \"Half\"\nportion = \"half\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("vacation")
        .arg("add")
        .arg("2026-12-28")
        .arg("2027-01-03")
        .arg("Winter")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let json: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
    let added: Vec<&str> = json["vacations"]
        .as_array()
        .ok_or("no vacations")?
        .iter()
        .filter_map(|day_off| day_off["date"].as_str())
        .collect();
    assert_eq!(
        added,
        ["2026-12-28", "2026-12-29", "2026-12-30", "2026-12-31"]
    );
    assert_eq!(
        json["skipped"],
        serde_json::json!([
            {"date": "2027-01-01", "reason": "holiday"},
            {"date": "2027-01-02", "reason": "no_working_hours"},
            {"date": "2027-01-03", "reason": "no_working_hours"}
        ])
    );

    let contents = fs::read_to_string(tmp.path().join("vacations.toml"))?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;
    assert_eq!(parsed.len(), 4);
    assert_eq!(parsed["2026-12-31"].description, "Winter");

    Ok(())
}

#[test]
fn add_vacation_range_prints_added_and_skipped_dates() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // Friday to Sunday.
    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-09")
        .arg("2026-10-11")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stdout(
            "Vacation 'Vacation' added on 2026-10-09.\n\
             Skipped 2026-10-10 (no working hours), 2026-10-11 (no working hours).\n",
        );

    Ok(())
}

#[test]
fn add_vacation_range_skips_dates_that_already_have_a_vacation()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let file = tmp.path().join("vacations.toml");
    fs::write(
        &file,
        "[2026-10-07]\ndescription = \"Existing\"\nportion = \"half\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-06")
        .arg("2026-10-08")
        .arg("Trip")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stdout(
            "Vacation 'Trip' added on 2026-10-06, 2026-10-08.\n\
             Skipped 2026-10-07 (already a vacation).\n",
        );

    let contents = fs::read_to_string(&file)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed["2026-10-07"].description, "Existing");
    assert_eq!(parsed["2026-10-07"].portion, DayPortion::Half);

    Ok(())
}

#[test]
fn add_vacation_range_rejects_a_last_day_before_the_first() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-09")
        .arg("2026-10-08")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains(
            "The last day (2026-10-08) is before the first day (2026-10-09).",
        ));

    assert!(!tmp.path().join("vacations.toml").exists());

    Ok(())
}

#[test]
fn add_vacation_range_rejects_a_range_without_working_days()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // A Saturday and a Sunday.
    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-10")
        .arg("2026-10-11")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("No date from 2026-10-10 to 2026-10-11 was added."));

    assert!(!tmp.path().join("vacations.toml").exists());

    Ok(())
}

#[test]
fn add_vacation_range_warns_about_sick_days() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    fs::write(
        tmp.path().join("sick_days.toml"),
        "[2026-12-22]\ndescription = \"Sick\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-12-21")
        .arg("2026-12-23")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stderr(contains(
            "Warning: 2026-12-22 already has a sick day. To remove it, run `ebb sickday remove 2026-12-22`.",
        ));

    let contents = fs::read_to_string(tmp.path().join("vacations.toml"))?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;
    assert_eq!(parsed.len(), 3);

    Ok(())
}

#[test]
fn add_vacation_range_of_one_day_without_working_hours_names_the_reason()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // A Saturday.
    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-10")
        .arg("2026-10-10")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("Skipped 2026-10-10 (no working hours)."));

    assert!(!tmp.path().join("vacations.toml").exists());

    Ok(())
}

#[test]
fn add_vacation_range_keeps_json_on_stdout_when_it_warns() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;
    fs::write(
        tmp.path().join("sick_days.toml"),
        "[2026-10-06]\ndescription = \"Sick\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("vacation")
        .arg("add")
        .arg("2026-10-05")
        .arg("2026-10-06")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stderr(contains("Warning: 2026-10-06 already has a sick day."));

    let json: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(json["vacations"].as_array().map(Vec::len), Some(2));

    Ok(())
}

#[test]
fn add_vacation_succeeds_if_the_sick_days_cannot_be_read() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;
    fs::write(tmp.path().join("sick_days.toml"), "garbage[[")?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-05")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stderr(contains(
            "Warning: Could not check the sick days on the same dates",
        ));

    let contents = fs::read_to_string(tmp.path().join("vacations.toml"))?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;
    assert_eq!(parsed.len(), 1);

    Ok(())
}

#[test]
fn add_vacation_rejects_a_day_without_working_hours() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // A Saturday.
    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-10")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains(
            "2026-10-10 has no working hours. Choose another date.",
        ));

    assert!(!tmp.path().join("vacations.toml").exists());

    Ok(())
}

#[test]
fn add_vacation_rejects_a_full_day_holiday() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    fs::write(
        tmp.path().join("holidays.toml"),
        "[2026-10-12]\ndescription = \"Holiday\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-12")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains(
            "2026-10-12 is a full-day holiday. Choose another date.",
        ));

    assert!(!tmp.path().join("vacations.toml").exists());

    Ok(())
}

#[test]
fn add_half_vacation_warns_on_a_half_holiday_with_a_half_sick_day()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    fs::write(
        tmp.path().join("holidays.toml"),
        "[2026-10-06]\ndescription = \"Half\"\nportion = \"half\"\n",
    )?;
    fs::write(
        tmp.path().join("sick_days.toml"),
        "[2026-10-06]\ndescription = \"Sick\"\nportion = \"half\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-06")
        .arg("--portion")
        .arg("half")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stderr(contains("Warning: 2026-10-06 already has a sick day."));

    Ok(())
}

#[test]
fn add_vacation_range_across_the_new_year_counts_in_each_year()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-12-28")
        .arg("2027-01-05")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    for (year, taken) in [("2026", 4.0), ("2027", 3.0)] {
        let mut cmd = Command::cargo_bin("ebb")?;
        let assert = cmd
            .arg("daysoff")
            .arg("--year")
            .arg(year)
            .arg("--format")
            .arg("json")
            .env("EBB_DATA_DIR", tmp.path())
            .assert()
            .success();
        let json: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
        assert_eq!(json["vacation_days_taken"], taken, "for {year}");
    }

    Ok(())
}

#[test]
fn add_vacation_range_rejects_a_range_longer_than_a_year() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-01-01")
        .arg("2027-01-01")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains(
            "The range from 2026-01-01 to 2027-01-01 is longer than a year.",
        ));

    assert!(!tmp.path().join("vacations.toml").exists());

    Ok(())
}

#[test]
fn add_vacation_range_accepts_a_whole_year() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-01-01")
        .arg("2026-12-31")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    Ok(())
}

#[test]
fn add_vacation_rejects_an_invalid_last_day() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-05")
        .arg("2026-13-01")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("'2026-13-01' is not a date"));

    assert!(!tmp.path().join("vacations.toml").exists());

    Ok(())
}

#[test]
fn add_vacation_rejects_a_third_argument_after_a_description()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-05")
        .arg("Trip")
        .arg("Extra")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains(
            "Too many arguments. Quote descriptions that contain spaces.",
        ));

    Ok(())
}

#[test]
fn add_vacation_rejects_a_last_day_after_the_description() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("add")
        .arg("2026-10-05")
        .arg("Trip")
        .arg("2026-10-07")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains(
            "Too many arguments. To specify a range, use the argument order <start-date> <end-date> <description>.",
        ));

    Ok(())
}
