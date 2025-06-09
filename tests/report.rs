// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use chrono::{Duration, Local, TimeZone, Utc};
use ebb::cli::report::{ProjectDuration, ReportOutput};
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
        updated_at = {frame1_end}

        [[frames]]
        start_time = {frame2_start}
        end_time = {frame2_end}
        project = "project2"
        updated_at = {frame2_end}

        [[frames]]
        start_time = {frame3_start}
        end_time = {frame3_end}
        project = "project1"
        updated_at = {frame3_end}
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
    let output: ReportOutput = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(output.total_duration, 9872);

    assert_eq!(output.timespan.from, frame1_start);

    let expected_projects = {
        let mut m = std::collections::HashMap::new();
        m.insert("project1".to_string(), ProjectDuration { duration: 5932 });
        m.insert("project2".to_string(), ProjectDuration { duration: 3940 });
        m
    };

    assert_eq!(output.projects, expected_projects);

    Ok(())
}

#[test]
fn report_without_frames() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    let assert = cmd
        .arg("report")
        .arg("--format")
        .arg("json")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let output: ReportOutput = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(output.total_duration, 0);
    assert_eq!(output.timespan.from, 0);

    let expected_projects = std::collections::HashMap::new();

    assert_eq!(output.projects, expected_projects);

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
    let output: ReportOutput = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert!(output.total_duration >= 6220);
    assert!(output.projects.get("myproject").unwrap().duration >= 6220);

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
        updated_at = {frame1_end}

        [[frames]]
        start_time = {frame2_start}
        end_time = {frame2_end}
        project = "project2"
        updated_at = {frame2_end}

        [[frames]]
        start_time = {frame3_start}
        end_time = {frame3_end}
        project = "project1"
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
    let output: ReportOutput = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(output.total_duration, 6052);
    assert_eq!(output.timespan.from, from);

    let expected_projects = {
        let mut m = std::collections::HashMap::new();
        m.insert("project1".to_string(), ProjectDuration { duration: 2112 });
        m.insert("project2".to_string(), ProjectDuration { duration: 3940 });
        m
    };

    assert_eq!(output.projects, expected_projects);

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
    let output: ReportOutput = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(output.total_duration, 2100);
    assert_eq!(output.timespan.from, from);

    let expected_projects = {
        let mut m = std::collections::HashMap::new();
        m.insert("project".to_string(), ProjectDuration { duration: 2100 });
        m
    };

    assert_eq!(output.projects, expected_projects);

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

        [[frames]]
        start_time = {frame2_start}
        end_time = {frame2_end}
        project = "project2"
        updated_at = {frame2_end}

        [[frames]]
        start_time = {frame3_start}
        end_time = {frame3_end}
        project = "project1"
        updated_at = {frame3_end}
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
    let output: ReportOutput = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(output.total_duration, 7760);
    assert_eq!(output.timespan.to, to);

    let expected_projects = {
        let mut m = std::collections::HashMap::new();
        m.insert("project1".to_string(), ProjectDuration { duration: 3820 });
        m.insert("project2".to_string(), ProjectDuration { duration: 3940 });
        m
    };

    assert_eq!(output.projects, expected_projects);

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
    let output: ReportOutput = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(output.total_duration, 1280);
    assert_eq!(output.timespan.to, to);

    let expected_projects = {
        let mut m = std::collections::HashMap::new();
        m.insert("project".to_string(), ProjectDuration { duration: 1280 });
        m
    };

    assert_eq!(output.projects, expected_projects);

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

        [[frames]]
        start_time = {frame2_start}
        end_time = {frame2_end}
        project = "project2"
        updated_at = {frame2_end}
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
    let output: ReportOutput = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(output.total_duration, 3820);

    let expected_projects = {
        let mut m = std::collections::HashMap::new();
        m.insert("project1".to_string(), ProjectDuration { duration: 3820 });
        m
    };

    assert_eq!(output.projects, expected_projects);

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
    let output: ReportOutput = serde_json::from_str(&stdout).expect("Expected valid JSON output");

    assert_eq!(output.total_duration, 3820);

    let expected_projects = {
        let mut m = std::collections::HashMap::new();
        m.insert("project1".to_string(), ProjectDuration { duration: 3820 });
        m
    };

    assert_eq!(output.projects, expected_projects);

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
    let output: ReportOutput = serde_json::from_str(&stdout)?;

    assert_eq!(output.timespan.from, expected_from);

    Ok(())
}
