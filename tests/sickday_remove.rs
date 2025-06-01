// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use ebb::types::SickdayEntry;
use predicates::str::contains;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn remove_sickday_removes_entry() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("sickdays.toml");
    let toml_content = r#"
        [2025-05-28]
        description = "headache"
        portion = "full"

        [2025-05-29]
        description = "fever"
        portion = "half"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("sickday")
        .arg("remove")
        .arg("2025-05-28")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("sickdays.toml");
    let contents = fs::read_to_string(file)?;
    let parsed: BTreeMap<String, SickdayEntry> = toml::from_str(&contents)?;

    assert!(
        !parsed.contains_key("2025-05-28"),
        "Unexpected entry found for 2025-05-28"
    );
    assert_eq!(parsed.get("2025-05-29").unwrap().description, "fever");

    Ok(())
}

#[test]
fn add_sickday_fails_if_date_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("sickdays.toml");
    let toml_content = r#"
        [2025-05-29]
        description = "fever"
        portion = "half"
    "#;
    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("sickday")
        .arg("remove")
        .arg("2025-05-28")
        .env("EBB_CONFIG_DIR", tmp.path());

    cmd.assert()
        .failure()
        .stderr(contains("No sick day found on 2025-05-28"));

    let contents = fs::read_to_string(&file_path)?;
    let parsed: BTreeMap<String, SickdayEntry> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-29").unwrap().description, "fever");

    Ok(())
}
