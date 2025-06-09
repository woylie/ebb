// SPDX-FileCopyrightText: 2025 Mathias Polligkeit
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use ebb::types::Frames;
use std::fs;
use tempfile::tempdir;

#[test]
fn tag_removes_a_tag() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("frames.toml");
    let toml_content = r#"
        [[frames]]
        start_time = 1748723006
        end_time = 1748723008
        project = "project1"
        tags = ["tag3", "tag1"]
        updated_at = 1748723008

        [[frames]]
        start_time = 1748723010
        end_time = 1748723012
        project = "project2"
        tags = ["tag3", "tag4"]
        updated_at = 1748723012

        [[frames]]
        start_time = 1748723050
        end_time = 1748723056
        project = "project1"
        tags = ["tag5", "tag1", "tag2"]
        updated_at = 1748723056
        "#;

    fs::write(&file_path, toml_content.trim())?;

    let expected_output = "\
Tag 'tag1' removed from all frames.
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("tag")
        .arg("remove")
        .arg("tag1")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success()
        .stdout(expected_output);

    let contents = fs::read_to_string(file_path)?;
    let parsed: Frames = toml::from_str(&contents)?;

    assert_eq!(&parsed.frames[0].tags, &vec!["tag3"]);
    assert_eq!(&parsed.frames[1].tags, &vec!["tag3", "tag4"]);
    assert_eq!(&parsed.frames[2].tags, &vec!["tag5", "tag2"]);

    Ok(())
}
