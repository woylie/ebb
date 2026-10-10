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

#[cfg(unix)]
#[test]
fn creates_the_data_directory_and_its_files_private() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempdir()?;
    let data_dir = tmp.path().join("ebb");

    let script = r#"
        umask 022
        "$EBB" config set working_hours.monday 4h
        "$EBB" start first --at "2025-06-02 09:00"
        "$EBB" stop --at "2025-06-02 10:00"
        "$EBB" start second --at "2025-06-02 11:00"
        "$EBB" stop --at "2025-06-02 12:00"
    "#;

    Command::new("sh")
        .arg("-ec")
        .arg(script)
        .env("EBB", assert_cmd::cargo::cargo_bin("ebb"))
        .env("EBB_DATA_DIR", &data_dir)
        .assert()
        .success();

    let mode = |path: &Path| -> std::io::Result<u32> {
        Ok(std::fs::metadata(path)?.permissions().mode() & 0o777)
    };

    assert_eq!(mode(&data_dir)?, 0o700);
    assert_eq!(mode(&data_dir.join("config.toml"))?, 0o600);
    assert_eq!(mode(&data_dir.join("frames.toml.bak"))?, 0o600);

    Ok(())
}

#[cfg(unix)]
#[test]
fn makes_an_existing_data_directory_private() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempdir()?;
    let data_dir = tmp.path();
    fs::set_permissions(data_dir, fs::Permissions::from_mode(0o755))?;

    Command::cargo_bin("ebb")?
        .env("EBB_DATA_DIR", data_dir)
        .arg("start")
        .arg("myproject")
        .assert()
        .success();

    let mode = fs::metadata(data_dir)?.permissions().mode();
    assert_eq!(mode & 0o777, 0o700);

    Ok(())
}
