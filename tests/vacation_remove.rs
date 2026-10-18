// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use ebb::types::VacationEntry;
use predicates::str::contains;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn remove_vacation_removes_entry() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("vacations.toml");
    let toml_content = r#"
        [2025-05-28]
        description = "Mountain Day"
        portion = "full"

        [2025-05-29]
        description = "Ocean Day"
        portion = "half"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("remove")
        .arg("2025-05-28")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("vacations.toml");
    let contents = fs::read_to_string(file)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;

    assert!(
        !parsed.contains_key("2025-05-28"),
        "Unexpected entry found for 2025-05-28"
    );
    assert_eq!(parsed.get("2025-05-29").unwrap().description, "Ocean Day");

    Ok(())
}

#[test]
fn add_vacation_fails_if_date_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("vacations.toml");
    let toml_content = r#"
        [2025-05-29]
        description = "Ocean Day"
        portion = "half"
    "#;
    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("remove")
        .arg("2025-05-28")
        .env("EBB_DATA_DIR", tmp.path());

    cmd.assert()
        .failure()
        .stderr(contains("No vacation found on 2025-05-28"));

    let contents = fs::read_to_string(&file_path)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-29").unwrap().description, "Ocean Day");

    Ok(())
}

#[test]
fn remove_vacation_range_removes_every_vacation_in_it() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let file = tmp.path().join("vacations.toml");
    fs::write(
        &file,
        "[2026-10-05]\ndescription = \"Before\"\n\n\
         [2026-10-06]\ndescription = \"Trip\"\n\n\
         [2026-10-08]\ndescription = \"Trip\"\n\n\
         [2026-10-09]\ndescription = \"After\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("remove")
        .arg("2026-10-06")
        .arg("2026-10-08")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stdout(
            "Removed vacation 'Trip' on 2026-10-06.\n\
             Removed vacation 'Trip' on 2026-10-08.\n",
        );

    let contents = fs::read_to_string(&file)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;
    assert_eq!(
        parsed.keys().collect::<Vec<_>>(),
        ["2026-10-05", "2026-10-09"]
    );

    Ok(())
}

#[test]
fn remove_vacation_range_without_vacations_fails() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let file = tmp.path().join("vacations.toml");
    fs::write(&file, "[2026-10-05]\ndescription = \"Before\"\n")?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("remove")
        .arg("2026-10-06")
        .arg("2026-10-08")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains(
            "No vacations found from 2026-10-06 to 2026-10-08.",
        ));

    let contents = fs::read_to_string(&file)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;
    assert_eq!(parsed.len(), 1);

    Ok(())
}

#[test]
fn remove_vacation_range_rejects_a_range_longer_than_a_year()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
        .arg("remove")
        .arg("2026-01-01")
        .arg("2027-01-01")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains(
            "The range from 2026-01-01 to 2027-01-01 is longer than a year.",
        ));

    Ok(())
}
