// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use std::path::Path;
use tempfile::tempdir;

fn ebb(home: &Path) -> Result<Command, Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ebb")?;

    cmd.env("HOME", home)
        .env_remove("XDG_DATA_HOME")
        .env_remove("EBB_DATA_DIR")
        .env_remove("EBB_CONFIG_DIR");

    Ok(cmd)
}

#[test]
fn defaults_to_the_data_directory_in_the_home_directory() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = tempdir()?;
    let home = tmp.path();

    ebb(home)?.arg("start").arg("myproject").assert().success();

    assert!(home.join(".local/share/ebb/state.toml").exists());

    Ok(())
}

#[test]
fn uses_xdg_data_home_when_it_is_absolute() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let home = tmp.path();

    ebb(home)?
        .env("XDG_DATA_HOME", home.join("xdgdata"))
        .arg("start")
        .arg("myproject")
        .assert()
        .success();

    assert!(home.join("xdgdata/ebb/state.toml").exists());
    assert!(!home.join(".local/share/ebb").exists());

    Ok(())
}

#[test]
fn ignores_a_relative_xdg_data_home() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let home = tmp.path();

    ebb(home)?
        .env("XDG_DATA_HOME", "relative/path")
        .arg("start")
        .arg("myproject")
        .assert()
        .success();

    assert!(home.join(".local/share/ebb/state.toml").exists());

    Ok(())
}

#[test]
fn config_dir_option_takes_precedence() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let home = tmp.path();

    ebb(home)?
        .env("XDG_DATA_HOME", home.join("xdgdata"))
        .arg("--config-dir")
        .arg(home.join("elsewhere"))
        .arg("start")
        .arg("myproject")
        .assert()
        .success();

    assert!(home.join("elsewhere/state.toml").exists());
    assert!(!home.join("xdgdata/ebb").exists());

    Ok(())
}

#[test]
fn config_dir_option_expands_a_tilde() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let home = tmp.path();

    ebb(home)?
        .arg("--config-dir")
        .arg("~/tilde")
        .arg("start")
        .arg("myproject")
        .assert()
        .success();

    assert!(home.join("tilde/state.toml").exists());

    Ok(())
}

#[test]
fn honours_the_deprecated_env_var() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let home = tmp.path();

    ebb(home)?
        .env("EBB_CONFIG_DIR", home.join("legacy"))
        .arg("start")
        .arg("myproject")
        .assert()
        .success()
        .stderr(predicates::str::contains("EBB_CONFIG_DIR is deprecated"));

    assert!(home.join("legacy/state.toml").exists());

    Ok(())
}

#[test]
fn prefers_the_current_env_var_over_the_deprecated_one() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let home = tmp.path();

    ebb(home)?
        .env("EBB_DATA_DIR", home.join("current"))
        .env("EBB_CONFIG_DIR", home.join("legacy"))
        .arg("start")
        .arg("myproject")
        .assert()
        .success();

    assert!(home.join("current/state.toml").exists());
    assert!(!home.join("legacy").exists());

    Ok(())
}
