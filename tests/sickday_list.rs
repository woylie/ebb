// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn list_sickdays_displays_all() -> Result<(), Box<dyn std::error::Error>> {
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

    let expected_output = "\
┌────────────┬─────────────┬─────────┐
│ date       │ description │ portion │
├────────────┼─────────────┼─────────┤
│ 2025-05-28 │ headache    │ full    │
│ 2025-05-29 │ fever       │ half    │
└────────────┴─────────────┴─────────┘
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("sickday")
        .arg("list")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}

#[test]
fn list_sickdays_filters_by_year() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("sickdays.toml");
    let toml_content = r#"
        [2024-08-12]
        description = "headache"
        portion = "full"

        [2025-02-05]
        description = "fever"
        portion = "half"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let expected_output = "\
┌────────────┬─────────────┬─────────┐
│ date       │ description │ portion │
├────────────┼─────────────┼─────────┤
│ 2024-08-12 │ headache    │ full    │
└────────────┴─────────────┴─────────┘
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("sickday")
        .arg("list")
        .arg("-y")
        .arg("2024")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}
