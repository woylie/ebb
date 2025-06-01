// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use ebb::types::{Frames, State};
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn stop_saves_current_frame() -> Result<(), Box<dyn std::error::Error>> {
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
    cmd.arg("stop")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let state_file = tmp.path().join("state.toml");
    assert!(state_file.exists());

    let state_contents = fs::read_to_string(state_file)?;
    let state: State = toml::from_str(&state_contents)?;
    assert_eq!(state.current_frame, None);

    let frames_file = tmp.path().join("frames.toml");
    assert!(frames_file.exists());

    let frame_contents = fs::read_to_string(frames_file)?;
    let frames: Frames = toml::from_str(&frame_contents)?;

    let last_frame = frames.frames.last().expect("No frames found");
    assert_eq!(last_frame.project, "firstproject");
    assert_eq!(last_frame.start_time, 1748723006);

    Ok(())
}

#[test]
fn stop_applies_at_option() -> Result<(), Box<dyn std::error::Error>> {
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
    cmd.arg("stop")
        .arg("--at")
        .arg("1748723100")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let state_file = tmp.path().join("state.toml");
    assert!(state_file.exists());

    let state_contents = fs::read_to_string(state_file)?;
    let state: State = toml::from_str(&state_contents)?;
    assert_eq!(state.current_frame, None);

    let frames_file = tmp.path().join("frames.toml");
    assert!(frames_file.exists());

    let frame_contents = fs::read_to_string(frames_file)?;
    let frames: Frames = toml::from_str(&frame_contents)?;

    let last_frame = frames.frames.last().expect("No frames found");
    assert_eq!(last_frame.project, "firstproject");
    assert_eq!(last_frame.start_time, 1748723006);
    assert_eq!(last_frame.end_time, 1748723100);

    Ok(())
}

#[test]
fn stop_fails_if_end_time_is_before_start_time() -> Result<(), Box<dyn std::error::Error>> {
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
    cmd.arg("stop")
        .arg("--at")
        .arg("1748723005")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("is before start time"));

    Ok(())
}

#[test]
fn cancel_fails_if_there_is_no_current_frame() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();
    let file_path = config_dir.join("state.toml");
    fs::write(&file_path, "")?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("stop")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("No project started."));

    Ok(())
}
