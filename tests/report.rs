// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use chrono::{Duration, Local, TimeZone, Utc};
use serde_json::{Value, json};
use std::fs;
use tempfile::tempdir;

#[test]
fn report_without_args() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().timestamp();

    let frame1_start = now - Duration::days(6).num_seconds();
    let frame1_end = frame1_start + 3820;

    let frame2_start = now - Duration::days(3).num_seconds();
    let frame2_end = frame2_start + 3940;

    let frame3_start = now - Duration::days(1).num_seconds();
    let frame3_end = frame3_start + 2112;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");

    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {frame1_start}
        end_time = {frame1_end}
        project = "project1"
        tags = ["tag1", "tag2"]
        updated_at = {frame1_end}

        [[frames]]
        start_time = {frame2_start}
        end_time = {frame2_end}
        project = "project2"
        updated_at = {frame2_end}
        tags = ["tag3"]

        [[frames]]
        start_time = {frame3_start}
        end_time = {frame3_end}
        project = "project1"
        updated_at = {frame3_end}
        tags = ["tag2"]
        "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let output: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(output["total_duration"], 9872);
    assert_eq!(output["timespan"]["from"], frame1_start);

    let expected_projects = json!({
        "project1": {
            "duration": 5932,
            "tags": {
                "tag1": 3820,
                "tag2": 5932
            }
        },
        "project2": {
            "duration": 3940,
            "tags": {
                "tag3": 3940
            }
        }
    });

    assert_eq!(output["projects"], expected_projects);

    Ok(())
}

#[test]
fn report_without_frames() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--to")
        .arg("2025-06-18T21:31:43+00:00")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    let expected_json = json!({
        "total_duration": 0,
        "timespan": {
            "from": 1750282303,
            "to": 1750282303
        },
        "projects": {}
    });

    assert_eq!(json, expected_json);

    Ok(())
}

#[test]
fn report_includes_current_frame() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().timestamp();

    let frame1_start = now - Duration::days(6).num_seconds();
    let frame1_end = frame1_start + 3820;
    let frame2_start = now - 2400;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {frame1_start}
        end_time = {frame1_end}
        project = "myproject"
        updated_at = {frame1_end}
        "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let file_path = config_dir.join("state.toml");
    let toml_content = format!(
        r#"
        [current_frame]
        start_time = {frame2_start}
        project = "myproject"
    "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert!(json["total_duration"].as_u64().unwrap() >= 6220);
    assert!(json["projects"]["myproject"]["duration"].as_u64().unwrap() >= 6220);

    Ok(())
}

#[test]
fn report_applies_from_option() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().timestamp();
    let from = now - Duration::days(5).num_seconds();

    let frame1_start = now - Duration::days(6).num_seconds();
    let frame1_end = frame1_start + 3820;

    let frame2_start = now - Duration::days(3).num_seconds();
    let frame2_end = frame2_start + 3940;

    let frame3_start = now - Duration::days(1).num_seconds();
    let frame3_end = frame3_start + 2112;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {frame1_start}
        end_time = {frame1_end}
        project = "project1"
        tags = ["tag1", "tag2"]
        updated_at = {frame1_end}

        [[frames]]
        start_time = {frame2_start}
        end_time = {frame2_end}
        project = "project2"
        tags = ["tag3"]
        updated_at = {frame2_end}

        [[frames]]
        start_time = {frame3_start}
        end_time = {frame3_end}
        project = "project1"
        tags = ["tag2"]
        updated_at = {frame3_end}
        "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--from")
        .arg(Utc.timestamp_opt(from, 0).unwrap().to_rfc3339())
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(json["total_duration"], 6052);
    assert_eq!(json["timespan"]["from"], from);

    assert_eq!(
        json["projects"],
        json!({
            "project1": {
                "duration": 2112,
                "tags": {"tag2": 2112}
            },
            "project2": {
                "duration": 3940,
                "tags": {"tag3": 3940}
            }
        })
    );

    Ok(())
}

#[test]
fn report_adjusts_start_time_if_frame_starts_before_from() -> Result<(), Box<dyn std::error::Error>>
{
    let now = Utc::now().timestamp();
    let from = now - Duration::days(5).num_seconds();

    let frame_start = from - 1280;
    let frame_end = from + 2100;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {frame_start}
        end_time = {frame_end}
        project = "project"
        tags = ["tag1", "tag2"]
        updated_at = {frame_end}
        "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--from")
        .arg(Utc.timestamp_opt(from, 0).unwrap().to_rfc3339())
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(json["total_duration"], 2100);
    assert_eq!(json["timespan"]["from"], from);

    let expected_projects = json!({
        "project": {
            "duration": 2100,
            "tags": {
                "tag1": 2100,
                "tag2": 2100
            }
        }
    });

    assert_eq!(json["projects"], expected_projects);

    Ok(())
}

#[test]
fn report_applies_to_option() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().timestamp();
    let to = now - Duration::days(5).num_seconds();

    let frame1_start = now - Duration::days(7).num_seconds();
    let frame1_end = frame1_start + 3820;

    let frame2_start = now - Duration::days(6).num_seconds();
    let frame2_end = frame2_start + 3940;

    let frame3_start = now - Duration::days(4).num_seconds();
    let frame3_end = frame3_start + 2112;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {frame1_start}
        end_time = {frame1_end}
        project = "project1"
        updated_at = {frame1_end}
        tags = ["tag1", "tag2"]

        [[frames]]
        start_time = {frame2_start}
        end_time = {frame2_end}
        project = "project2"
        updated_at = {frame2_end}
        tags = ["tag3"]

        [[frames]]
        start_time = {frame3_start}
        end_time = {frame3_end}
        project = "project1"
        updated_at = {frame3_end}
        tags = ["tag2"]
        "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--to")
        .arg(Utc.timestamp_opt(to, 0).unwrap().to_rfc3339())
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(json["total_duration"], 7760);
    assert_eq!(json["timespan"]["to"], to);

    let expected_projects = json!({
        "project1": {
            "duration": 3820,
            "tags": {
                "tag1": 3820,
                "tag2": 3820
            }
        },
        "project2": {
            "duration": 3940,
            "tags": {
                "tag3": 3940
            }
        }
    });

    assert_eq!(json["projects"], expected_projects);

    Ok(())
}

#[test]
fn report_adjusts_end_time_if_frame_ends_after_to() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().timestamp();
    let to = now - Duration::days(5).num_seconds();

    let frame_start = to - 1280;
    let frame_end = to + 2100;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {frame_start}
        end_time = {frame_end}
        project = "project"
        updated_at = {frame_end}
        tags = ["tag1", "tag2"]
        "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--to")
        .arg(Utc.timestamp_opt(to, 0).unwrap().to_rfc3339())
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(json["total_duration"], 1280);
    assert_eq!(json["timespan"]["to"], to);

    let expected_projects = json!({
        "project": {
            "duration": 1280,
            "tags": {
                "tag1": 1280,
                "tag2": 1280
            }
        }
    });

    assert_eq!(json["projects"], expected_projects);

    Ok(())
}

#[test]
fn report_filters_by_project() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().timestamp();

    let frame1_start = now - Duration::days(6).num_seconds();
    let frame1_end = frame1_start + 3820;

    let frame2_start = now - Duration::days(3).num_seconds();
    let frame2_end = frame2_start + 3940;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {frame1_start}
        end_time = {frame1_end}
        project = "project1"
        updated_at = {frame1_end}
        tags = ["tag1", "tag2"]

        [[frames]]
        start_time = {frame2_start}
        end_time = {frame2_end}
        project = "project2"
        updated_at = {frame2_end}
        tags = ["tag3"]
        "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--project")
        .arg("project1")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(json["total_duration"], 3820);

    let expected_projects = json!({
        "project1": {
            "duration": 3820,
            "tags": {
                "tag1": 3820,
                "tag2": 3820
            }
        }
    });

    assert_eq!(json["projects"], expected_projects);

    Ok(())
}

#[test]
fn report_filters_by_tag() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().timestamp();

    let frame1_start = now - Duration::days(6).num_seconds();
    let frame1_end = frame1_start + 3820;

    let frame2_start = now - Duration::days(3).num_seconds();
    let frame2_end = frame2_start + 3940;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {frame1_start}
        end_time = {frame1_end}
        project = "project1"
        tags = ["tag1"]
        updated_at = {frame1_end}

        [[frames]]
        start_time = {frame2_start}
        end_time = {frame2_end}
        project = "project2"
        tags = ["tag2"]
        updated_at = {frame2_end}
        "#
    );

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--tag")
        .arg("tag1")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(json["total_duration"], 3820);

    let expected_projects = json!({
        "project1": {
            "duration": 3820,
            "tags": {
                "tag1": 3820
            }
        }
    });

    assert_eq!(json["projects"], expected_projects);

    Ok(())
}

#[test]
fn report_applies_day_option() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let local_now = Local::now();
    let today_start = local_now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let expected_from = Local
        .from_local_datetime(&today_start)
        .unwrap()
        .with_timezone(&Utc)
        .timestamp();

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--day")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", config_dir)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let json: Value = serde_json::from_str(&stdout)?;

    assert_eq!(json["timespan"]["from"], expected_from);

    Ok(())
}

#[test]
fn report_replaces_control_characters_in_text_output() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now().timestamp();
    let start = now - 3600;

    let tmp = tempdir()?;
    let toml_content = format!(
        r#"
        [[frames]]
        start_time = {start}
        end_time = {now}
        project = "work\u001b[2J"
        tags = ["review\u0007"]
        updated_at = {now}
        "#
    );
    fs::write(tmp.path().join("frames.toml"), toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    assert!(stdout.contains("│ work\u{fffd}[2J   │"), "{stdout}");
    assert!(stdout.contains("│   +review\u{fffd} │"), "{stdout}");
    assert!(!stdout.contains(['\u{1b}', '\u{7}']), "{stdout}");

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    assert!(stdout.contains(r#""work\u001b[2J""#), "{stdout}");

    let output: Value = serde_json::from_str(&stdout)?;
    assert_eq!(
        output["projects"]["work\u{1b}[2J"]["tags"]["review\u{7}"],
        3600
    );

    Ok(())
}

#[test]
fn report_counts_a_running_frame_after_the_current_time_as_zero()
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
        .arg("report")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let output: Value = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(output["total_duration"], 0);
    assert_eq!(output["projects"], json!({}));

    Ok(())
}

fn local_timestamp(datetime: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let naive = chrono::NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M")?;
    let local = Local
        .from_local_datetime(&naive)
        .single()
        .ok_or("ambiguous local time")?;
    Ok(local.timestamp())
}

#[test]
fn report_covers_whole_days_with_dates() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let frames = [
        ("2026-10-09 23:00", "2026-10-09 23:30", "before"),
        ("2026-10-10 10:00", "2026-10-10 11:00", "inside"),
        ("2026-10-11 00:30", "2026-10-11 01:00", "after"),
    ];
    let mut toml_content = String::new();
    for (start, end, project) in frames {
        let (start, end) = (local_timestamp(start)?, local_timestamp(end)?);
        toml_content.push_str(&format!(
            "[[frames]]\nstart_time = {start}\nend_time = {end}\nproject = \"{project}\"\nupdated_at = {end}\n\n"
        ));
    }
    fs::write(tmp.path().join("frames.toml"), toml_content)?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--from")
        .arg("2026-10-10")
        .arg("--to")
        .arg("2026-10-10")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success();

    let json: Value = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(
        json["timespan"]["from"],
        local_timestamp("2026-10-10 00:00")?
    );
    assert_eq!(json["timespan"]["to"], local_timestamp("2026-10-11 00:00")?);
    assert_eq!(json["total_duration"], 3600);
    assert_eq!(json["projects"]["inside"]["duration"], 3600);

    Ok(())
}

#[test]
fn report_rejects_a_from_that_is_neither_a_date_nor_a_time()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("report")
        .arg("--from")
        .arg("2026-13-01")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "'2026-13-01' is not a date or time",
        ));

    Ok(())
}

// The time zone tests set TZ to a POSIX rule rather than a zone name, so they do not depend
// on the installed time zone data.

#[test]
fn report_starts_a_date_without_a_midnight_when_the_clock_jumps()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // In Africa/Cairo, the clock jumps from 00:00 to 01:00 on 2026-04-24.
    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--from")
        .arg("2026-04-24")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .env("TZ", "EET-2EEST,M4.5.5/0,M10.5.4/24")
        .assert()
        .success();

    let json: Value = serde_json::from_slice(&assert.get_output().stdout)?;
    // 2026-04-24 01:00 local time.
    assert_eq!(json["timespan"]["from"], 1776981600);

    Ok(())
}

#[test]
fn report_starts_a_date_whose_midnight_occurs_twice_at_the_first()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // In America/Havana, the clock goes back from 01:00 to 00:00 on 2026-11-01.
    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
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

    let json: Value = serde_json::from_slice(&assert.get_output().stdout)?;
    // 2026-11-01 00:00 at -04:00, and 2026-11-02 00:00 at -05:00.
    assert_eq!(json["timespan"]["from"], 1793505600);
    assert_eq!(json["timespan"]["to"], 1793595600);

    Ok(())
}

#[test]
fn report_accepts_a_midnight_right_after_the_clock_went_back()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // In Africa/Cairo, the clock goes back from 00:00 to 23:00 on 2026-10-29, so 23:00 to 23:59
    // occur twice and midnight on 2026-10-30 occurs once.
    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--from")
        .arg("2026-10-30")
        .arg("--to")
        .arg("2026-10-30")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .env("TZ", "EET-2EEST,M4.5.5/0,M10.5.4/24")
        .assert()
        .success();

    let json: Value = serde_json::from_slice(&assert.get_output().stdout)?;
    assert_eq!(json["timespan"]["from"], 1793311200);

    Ok(())
}

#[test]
fn report_rejects_a_time_that_occurs_twice() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // In America/Havana, the clock goes back from 01:00 to 00:00 on 2026-11-01.
    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("report")
        .arg("--from")
        .arg("2026-11-01 00:30")
        .env("EBB_DATA_DIR", tmp.path())
        .env("TZ", "CST5CDT,M3.2.0/0,M11.1.0/1")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Ambiguous time. Add the offset, e.g. 2026-11-01T00:30:00-04:00 or 2026-11-01T00:30:00-05:00.",
        ));

    Ok(())
}

#[test]
fn report_accepts_a_time_that_occurs_twice_with_an_offset() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;

    // In America/Havana, the clock goes back from 01:00 to 00:00 on 2026-11-01.
    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--from")
        .arg("2026-11-01T00:30:00-05:00")
        .arg("--to")
        .arg("2026-11-01")
        .arg("--format")
        .arg("json")
        .env("EBB_DATA_DIR", tmp.path())
        .env("TZ", "CST5CDT,M3.2.0/0,M11.1.0/1")
        .assert()
        .success();

    let json: Value = serde_json::from_slice(&assert.get_output().stdout)?;
    // 2026-11-01 05:30 UTC.
    assert_eq!(json["timespan"]["from"], 1793511000);

    Ok(())
}

#[test]
fn report_rejects_a_time_the_clock_skips() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    // In America/Santiago, the clock jumps from 00:00 to 01:00 on 2026-09-06.
    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("report")
        .arg("--from")
        .arg("2026-09-06 00:30")
        .env("EBB_DATA_DIR", tmp.path())
        .env("TZ", "<-04>4<-03>,M9.1.6/24,M4.1.6/24")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Nonexistent time. The clock skips '2026-09-06 00:30' in the local time zone.",
        ));

    Ok(())
}
