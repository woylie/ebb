use assert_cmd::Command;
use ebb::types::{VacationEntry};
use predicates::str::contains;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn remove_vacation_removes_entry() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("vacations.toml");
    let toml_content = r#"
        [2025-05-28]
        description = "Mountain Day"
        portion = "full"

        [2025-05-29]
        description = "Ocean Day"
        portion = "half"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
       .arg("remove")
       .arg("2025-05-28")
       .env("EBB_CONFIG_DIR", tmp.path())
       .assert()
       .success();

    let file = tmp.path().join("vacations.toml");
    let contents = fs::read_to_string(file)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;

    assert!(parsed.get("2025-05-28").is_none(), "Unexpected entry found for 2025-05-28");
    assert_eq!(parsed.get("2025-05-29").unwrap().description, "Ocean Day");

    Ok(())
}

#[test]
fn add_vacation_fails_if_date_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("vacations.toml");
    let toml_content = r#"
        [2025-05-29]
        description = "Ocean Day"
        portion = "half"
    "#;
    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("vacation")
       .arg("remove")
       .arg("2025-05-28")
       .env("EBB_CONFIG_DIR", tmp.path());

    cmd.assert()
        .failure()
        .stderr(contains("No vacation exists on 2025-05-28"));

    let contents = fs::read_to_string(&file_path)?;
    let parsed: BTreeMap<String, VacationEntry> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-29").unwrap().description, "Ocean Day");

    Ok(())
}
