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

#[test]
fn stop_backs_up_the_previous_frames() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let state_path = config_dir.join("state.toml");
    let backup_path = config_dir.join("frames.toml.bak");

    fs::write(
        &state_path,
        "[current_frame]\nstart_time = 1748723006\nproject = \"firstproject\"",
    )?;

    Command::cargo_bin("ebb")?
        .arg("stop")
        .env("EBB_CONFIG_DIR", config_dir)
        .assert()
        .success();

    assert!(!backup_path.exists());

    fs::write(
        &state_path,
        "[current_frame]\nstart_time = 1748823006\nproject = \"secondproject\"",
    )?;

    Command::cargo_bin("ebb")?
        .arg("stop")
        .env("EBB_CONFIG_DIR", config_dir)
        .assert()
        .success();

    let backup: Frames = toml::from_str(&fs::read_to_string(&backup_path)?)?;
    assert_eq!(backup.frames.len(), 1);
    assert_eq!(backup.frames[0].project, "firstproject");

    let frames: Frames = toml::from_str(&fs::read_to_string(config_dir.join("frames.toml"))?)?;
    assert_eq!(frames.frames.len(), 2);

    Ok(())
}

#[cfg(unix)]
#[test]
fn stop_leaves_the_frames_intact_if_the_write_fails() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let frames_path = config_dir.join("frames.toml");
    let frames_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748725744
        project = "firstproject"
        updated_at = 1748725744
    "#
    .trim();

    fs::write(&frames_path, frames_content)?;
    fs::write(
        config_dir.join("state.toml"),
        "[current_frame]\nstart_time = 1748823006\nproject = \"secondproject\"",
    )?;

    fs::set_permissions(config_dir, fs::Permissions::from_mode(0o555))?;

    let output = Command::cargo_bin("ebb")?
        .arg("stop")
        .env("EBB_CONFIG_DIR", config_dir)
        .output()?;

    // Restore before asserting, so that a failing assertion still leaves a removable
    // temporary directory.
    fs::set_permissions(config_dir, fs::Permissions::from_mode(0o755))?;

    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&frames_path)?, frames_content);

    Ok(())
}

#[cfg(unix)]
#[test]
fn stop_creates_the_frames_private() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempdir()?;
    let config_dir = tmp.path();

    fs::write(
        config_dir.join("state.toml"),
        "[current_frame]\nstart_time = 1748723006\nproject = \"firstproject\"",
    )?;

    Command::cargo_bin("ebb")?
        .arg("stop")
        .env("EBB_CONFIG_DIR", config_dir)
        .assert()
        .success();

    let mode = fs::metadata(config_dir.join("frames.toml"))?
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);

    Ok(())
}

#[cfg(unix)]
#[test]
fn stop_keeps_the_mode_of_an_existing_frames_file() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempdir()?;
    let config_dir = tmp.path();
    let frames_path = config_dir.join("frames.toml");

    fs::write(
        &frames_path,
        "[[frames]]\nstart_time = 1748723006\nend_time = 1748725744\nproject = \"firstproject\"\nupdated_at = 1748725744",
    )?;
    fs::set_permissions(&frames_path, fs::Permissions::from_mode(0o640))?;

    fs::write(
        config_dir.join("state.toml"),
        "[current_frame]\nstart_time = 1748823006\nproject = \"secondproject\"",
    )?;

    Command::cargo_bin("ebb")?
        .arg("stop")
        .env("EBB_CONFIG_DIR", config_dir)
        .assert()
        .success();

    let mode = fs::metadata(&frames_path)?.permissions().mode();
    assert_eq!(mode & 0o777, 0o640);

    Ok(())
}
