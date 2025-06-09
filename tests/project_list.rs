// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn project_lists_all_projects() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748723008
        project = "project2"
        tags = ["tag3", "tag1"]
        updated_at = 1748723008

        [[frames]]
        start_time = 1748723010
        end_time = 1748723012
        project = "project1"
        tags = ["tag3", "tag4"]
        updated_at = 1748723012

        [[frames]]
        start_time = 1748723050
        end_time = 1748723056
        project = "project2"
        tags = ["tag5", "tag1", "tag2"]
        updated_at = 1748723056
        "#;

    fs::write(&file_path, toml_content.trim())?;

    let expected_output = "\
project1
project2
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("project")
        .arg("list")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    Ok(())
}
