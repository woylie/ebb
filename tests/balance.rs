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
To: 2024-12-13 00:00:00 (Fri)

Expected:  1996h 22m 21s
Actual:        1h 3m 40s
Remaining: 1995h 18m 41s

"#;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--from")
        .arg("2024-01-01")
        .arg("--to")
        .arg("2024-12-12")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}

#[test]
fn balance_fails_if_from_is_in_the_future() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--from")
        .arg("2100-01-01")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("'to' must be after 'from'"));

    Ok(())
}

#[test]
fn balance_fails_if_a_frame_holds_an_invalid_timestamp() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    fs::write(
        tmp.path().join("frames.toml"),
        "[[frames]]\nstart_time = 900000000000000\nend_time = 900000000000001\nproject = \"x\"\nupdated_at = 0\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "frames.toml holds an invalid timestamp",
        ))
        .stderr(predicates::str::contains("900000000000000"));

    Ok(())
}

#[test]
fn balance_counts_a_running_frame_after_the_current_time_as_zero()
-> Result<(), Box<dyn std::error::Error>> {
    // The clock was set back by an hour.
    let start_time = Utc::now().timestamp() + 3600;
    let tmp = tempdir()?;
    let toml_content = format!(
        r#"
        [current_frame]
        start_time = {start_time}
        project = "myproject"
        "#
    );
    fs::write(tmp.path().join("state.toml"), toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("balance")
        .arg("--day")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(output["actual_working_seconds"], 0);

    Ok(())
}

#[test]
fn balance_rejects_exclude_today_with_from_after_the_start_of_today()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--exclude-today")
        .arg("--from")
        .arg("2999-01-01")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "'from' must not be after the start of today when --exclude-today is set",
        ));

    Ok(())
}

#[test]
fn balance_rejects_exclude_today_with_to() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--exclude-today")
        .arg("--to")
        .arg("2026-10-07")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "the argument '--exclude-today' cannot be used with '--to <TO>'",
        ));

    Ok(())
}

#[test]
fn balance_rejects_exclude_today_with_day() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--exclude-today")
        .arg("--day")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "the argument '--exclude-today' cannot be used with '--day'",
        ));

    Ok(())
}

#[test]
fn balance_rejects_a_time_in_from() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--from")
        .arg("2026-10-06 14:00")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "'2026-10-06 14:00' is not a date. Write it as yyyy-mm-dd.",
        ));

    Ok(())
}

#[test]
fn balance_rejects_a_to_before_from() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--from")
        .arg("2026-10-06")
        .arg("--to")
        .arg("2026-10-05")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("'to' must not be before 'from'"));

    Ok(())
}

#[test]
fn balance_rejects_the_largest_date_in_to() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--to")
        .arg("+262142-12-31")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "'+262142-12-31' is out of range.",
        ));

    Ok(())
}

#[test]
fn balance_expects_nothing_if_to_is_before_the_first_frame()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    fs::write(
        tmp.path().join("frames.toml"),
        r#"
        [[frames]]
        start_time = 1791367200
        end_time = 1791370800
        project = "project1"
        tags = []
        updated_at = 1791370800
        "#,
    )?;

    // The frame is on 2026-10-07.
    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("balance")
        .arg("--to")
        .arg("2026-10-05")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .env("TZ", "UTC")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            r#""expected_working_seconds": 0,"#,
        ));

    Ok(())
}

// The time zone tests set TZ to a POSIX rule rather than a zone name, so they do not depend
// on the installed time zone data.

#[test]
fn balance_starts_a_date_without_a_midnight_when_the_clock_jumps()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // In America/Santiago, the clock jumps from 00:00 to 01:00 on 2026-09-06.
    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("balance")
        .arg("--from")
        .arg("2026-09-06")
        .arg("--to")
        .arg("2026-09-06")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .env("TZ", "<-04>4<-03>,M9.1.6/24,M4.1.6/24")
        .assert()
        .success();

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
    // 2026-09-06 01:00 and 2026-09-07 00:00 local time.
    assert_eq!(output["timespan"]["from"], 1788667200);
    assert_eq!(output["timespan"]["to"], 1788750000);

    Ok(())
}

#[test]
fn balance_starts_a_date_whose_midnight_occurs_twice_at_the_first()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // In America/Havana, the clock goes back from 01:00 to 00:00 on 2026-11-01.
    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("balance")
        .arg("--from")
        .arg("2026-11-01")
        .arg("--to")
        .arg("2026-11-01")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .env("TZ", "CST5CDT,M3.2.0/0,M11.1.0/1")
        .assert()
        .success();

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
    // 2026-11-01 00:00 at -04:00.
    assert_eq!(output["timespan"]["from"], 1793505600);

    Ok(())
}

#[test]
fn balance_starts_a_date_at_the_end_of_a_gap_that_spans_midnight()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // The clock jumps from 23:30 on 2026-09-06 to 00:30 on 2026-09-07. In America/Toronto, it
    // jumped the same way on 1919-03-31.
    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("balance")
        .arg("--from")
        .arg("2026-09-07")
        .arg("--to")
        .arg("2026-09-07")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .env("TZ", "AAA3BBB,M9.1.0/23:30,M4.1.0/24")
        .assert()
        .success();

    let output: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout)?;
    // 2026-09-07 00:30 at -02:00.
    assert_eq!(output["timespan"]["from"], 1788748200);

    Ok(())
}
