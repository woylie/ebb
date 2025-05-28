use assert_cmd::Command;
use predicates::str::contains;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn add_sickday_creates_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("sickday")
       .arg("add")
       .arg("2025-05-28")
       .arg("headache")
       .env("EBB_CONFIG_DIR", tmp.path())
       .assert()
       .success();

    let file = tmp.path().join("sickdays.toml");
    assert!(file.exists());

    let contents = fs::read_to_string(file)?;
    let parsed: BTreeMap<String, String> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-28"), Some(&"headache".to_string()));

    Ok(())
}

#[test]
fn add_sickday_fails_if_date_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("sickdays.toml");
    fs::write(
        &file_path,
        r#"2025-05-28 = "fever""#,
    )?;

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("sickday")
       .arg("add")
       .arg("2025-05-28")
       .arg("headache")
       .env("EBB_CONFIG_DIR", tmp.path());

    cmd.assert()
        .failure()
        .stderr(contains("already exists"));

    let contents = fs::read_to_string(&file_path)?;
    let parsed: BTreeMap<String, String> = toml::from_str(&contents)?;

    assert_eq!(parsed.get("2025-05-28"), Some(&"fever".to_string()));

    Ok(())
}
