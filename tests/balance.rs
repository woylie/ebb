// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use std::fs;
use tempfile::tempdir;

#[test]
fn balance_prints_overview() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let config_dir = tmp.path();

    let file_path = config_dir.join("config.toml");
    let toml_content = r#"
        [working_hours]
        monday = "4h 5m"
        tuesday = "8h"
        wednesday = "8h"
        thursday = "8h"
        friday = "8h"
        saturday = "0h"
        sunday = "4h 5m 9s"

    "#;
    fs::write(&file_path, toml_content.trim())?;

    let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let time = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
    let naive_dt = NaiveDateTime::new(date, time);
    let utc_dt = Utc.from_utc_datetime(&naive_dt);
    let frame_start = utc_dt.timestamp();
    let frame_end = frame_start + 3820;

    let file_path = config_dir.join("frames.toml");
    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {frame_start}
        end_time = {frame_end}
        project = "project1"
        tags = ["tag1", "tag2"]
        updated_at = {frame_end}
        "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let expected_output = r#"
From: 2024-01-01 00:00:00 (Mon)
To: 2024-12-12 23:59:59 (Thu)

Expected: 83d 4h 22m 21s
Actual: 1h 3m 40s
Remaining: 83d 3h 18m 41s

"#;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--from")
        .arg("2024-01-01 00:00:00")
        .arg("--to")
        .arg("2024-12-12 23:59:59")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}
