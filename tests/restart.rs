// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use ebb::types::State;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn restart_sets_state_from_last_frame() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748725744
        project = "myproject"
        updated_at = 1748725744
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("restart")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("state.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let state: State = toml::from_str(&contents)?;

    assert_eq!(state.current_frame.unwrap().project, "myproject");

    Ok(())
}

#[test]
fn restart_applies_no_gap_option() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748725744
        project = "myproject"
        updated_at = 1748725744
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("restart")
        .arg("--no-gap")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("state.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let state: State = toml::from_str(&contents)?;
    let current_frame = state.current_frame.unwrap();

    assert_eq!(current_frame.project, "myproject");
    assert_eq!(current_frame.start_time, 1748725744);

    Ok(())
}

#[test]
fn restart_applies_at_option() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748725744
        project = "myproject"
        updated_at = 1748725744
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("restart")
        .arg("--at")
        .arg("1748725750")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("state.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let state: State = toml::from_str(&contents)?;
    let current_frame = state.current_frame.unwrap();

    assert_eq!(current_frame.project, "myproject");
    assert_eq!(current_frame.start_time, 1748725750);

    Ok(())
}

#[test]
fn restart_fails_if_start_time_is_before_last_end_time() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748725744
        project = "myproject"
        updated_at = 1748725744
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("restart")
        .arg("--at")
        .arg("1748725743")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("is before the end"));

    Ok(())
}

#[test]
fn restart_fails_with_no_gap_and_at() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748725744
        project = "myproject"
        updated_at = 1748725744
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("restart")
        .arg("--at")
        .arg("1748725743")
        .arg("--no-gap")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains(
            "the argument '--at <AT>' cannot be used with '--no-gap'",
        ));

    Ok(())
}

#[test]
fn restart_fails_if_there_is_no_previous_frame() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("restart")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("No previous project"));

    Ok(())
}

#[test]
fn restart_fails_if_current_frame_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("state.toml");
    let toml_content = r#"
        [current_frame]
        start_time = 1748723006
        project = "firstproject"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("restart")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("already in progress."));

    Ok(())
}
