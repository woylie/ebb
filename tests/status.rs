// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn status_returns_current_frame() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("state.toml");
    let toml_content = r#"
        [current_frame]
        start_time = 1748723006
        project = "myproject"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("status")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stdout(contains("Current project 'myproject' started at"));

    Ok(())
}

#[test]
fn status_returns_message_when_status_is_empty() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("status")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .success()
        .stdout(contains("No project started."));

    Ok(())
}

#[test]
fn status_fails_if_the_state_holds_an_invalid_timestamp() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;

    fs::write(
        tmp.path().join("state.toml"),
        "[current_frame]\nstart_time = 900000000000000\nproject = \"x\"\n",
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("status")
        .env("EBB_DATA_DIR", tmp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "state.toml holds an invalid timestamp",
        ));

    Ok(())
}
