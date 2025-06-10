// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
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
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}
