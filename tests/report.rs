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
        .arg("--to")
        .arg("1750282303")
        .arg("--format")
        .arg("json")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let output: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    let expected_json = json!({
        "total_duration": 9872,
        "timespan": {
            "from": frame1_start,
            "to": 1750282303
        },
        "projects": {
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
        }
    });

    assert_eq!(output, expected_json);

    Ok(())
}

#[test]
fn report_without_frames() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--to")
        .arg("1750282303")
        .arg("--format")
        .arg("json")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    let expected_json = json!({
        "total_duration": 0,
        "timespan": {
            "from": 0,
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
        .env("EBB_CONFIG_DIR", tmp.path())
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
        .arg(from.to_string())
        .arg("--format")
        .arg("json")
        .env("EBB_CONFIG_DIR", tmp.path())
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
        .arg(from.to_string())
        .arg("--format")
        .arg("json")
        .env("EBB_CONFIG_DIR", tmp.path())
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
        .arg(to.to_string())
        .arg("--format")
        .arg("json")
        .env("EBB_CONFIG_DIR", tmp.path())
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
        .arg(to.to_string())
        .arg("--format")
        .arg("json")
        .env("EBB_CONFIG_DIR", tmp.path())
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
        .env("EBB_CONFIG_DIR", tmp.path())
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
        .env("EBB_CONFIG_DIR", tmp.path())
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
        .env("EBB_CONFIG_DIR", config_dir)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone())?;
    let json: Value = serde_json::from_str(&stdout)?;

    assert_eq!(json["timespan"]["from"], expected_from);

    Ok(())
}
