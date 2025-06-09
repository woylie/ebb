// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn config_list_prints_default_config() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let expected_output = "\
┌─────────────────────────────┬───────┐
│ Key                         │ Value │
├─────────────────────────────┼───────┤
│ sick_days_per_year.2000     │ 30    │
│ vacation_days_per_year.2000 │ 30    │
│ working_hours.monday        │ 8h    │
│ working_hours.tuesday       │ 8h    │
│ working_hours.wednesday     │ 8h    │
│ working_hours.thursday      │ 8h    │
│ working_hours.friday        │ 8h    │
│ working_hours.saturday      │ 0s    │
│ working_hours.sunday        │ 0s    │
└─────────────────────────────┴───────┘
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("list")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}

#[test]
fn config_list_prints_custom_config() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let config_dir = tmp.path();

    let file_path = config_dir.join("config.toml");
    let toml_content = r#"
        [sick_days_per_year]
        2005 = 30
        2004 = 28

        [vacation_days_per_year]
        2005 = 38
        2000 = 30

        [working_hours]
        monday = "4h"
        tuesday = "5h"
        wednesday = "6h 44m"
        thursday = "3h 2m"
        friday = "7s"
        saturday = "0h"
        sunday = "0h"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let expected_output = "\
┌─────────────────────────────┬────────┐
│ Key                         │ Value  │
├─────────────────────────────┼────────┤
│ sick_days_per_year.2004     │ 28     │
│ sick_days_per_year.2005     │ 30     │
│ vacation_days_per_year.2000 │ 30     │
│ vacation_days_per_year.2005 │ 38     │
│ working_hours.monday        │ 4h     │
│ working_hours.tuesday       │ 5h     │
│ working_hours.wednesday     │ 6h 44m │
│ working_hours.thursday      │ 3h 2m  │
│ working_hours.friday        │ 7s     │
│ working_hours.saturday      │ 0s     │
│ working_hours.sunday        │ 0s     │
└─────────────────────────────┴────────┘
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("config")
        .arg("list")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}
