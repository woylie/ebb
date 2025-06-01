// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use ebb::types::State;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn cancel_resets_state_file() -> Result<(), Box<dyn std::error::Error>> {
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
    cmd.arg("cancel")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("state.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let state: State = toml::from_str(&contents)?;

    assert_eq!(state.current_frame, None);

    Ok(())
}

#[test]
fn cancel_fails_if_there_is_no_current_frame() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();
    let file_path = config_dir.join("state.toml");
    fs::write(&file_path, "")?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("cancel")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(contains("No project started."));

    Ok(())
}
