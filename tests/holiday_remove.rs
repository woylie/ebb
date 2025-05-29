use assert_cmd::Command;
use ebb::types::HolidayEntry;
use predicates::str::contains;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn remove_holiday_removes_entry() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("holidays.toml");
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
    cmd.arg("holiday")
        .arg("remove")
        .arg("2025-05-28")
        .env("EBB_CONFIG_DIR", tmp.path())
        .assert()
        .success();

    let file = tmp.path().join("holidays.toml");
    let contents = fs::read_to_string(file)?;
    let parsed: BTreeMap<String, HolidayEntry> = toml::from_str(&contents)?;

    assert!(
        !parsed.contains_key("2025-05-28"),
        "Unexpected entry found for 2025-05-28"
    );
    assert_eq!(parsed.get("2025-05-29").unwrap().description, "Ocean Day");

    Ok(())
}

#[test]
fn add_holiday_fails_if_date_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("holidays.toml");
    let toml_content = r#"
        [2025-05-29]
        description = "Ocean Day"
        portion = "half"
    "#;
    fs::write(&file_path, toml_content.trim())?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("holiday")
        .arg("remove")
        .arg("2025-05-28")
        .env("EBB_CONFIG_DIR", tmp.path());

    cmd.assert()
        .failure()
        .stderr(contains("No holiday exists on 2025-05-28"));

    let contents = fs::read_to_string(&file_path)?;
    let parsed: BTreeMap<String, HolidayEntry> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-29").unwrap().description, "Ocean Day");

    Ok(())
}
