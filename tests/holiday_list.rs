use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn list_holidays_displays_all() -> Result<(), Box<dyn std::error::Error>> {
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

    let expected_output = "\
Holidays:
2025-05-28 — Mountain Day
2025-05-29 — Ocean Day (half)
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("holiday")
       .arg("list")
       .env("EBB_CONFIG_DIR", tmp.path())
       .assert()
       .success()
       .stdout(expected_output);

    Ok(())
}

#[test]
fn list_holidays_filters_by_year() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let config_dir = tmp.path();

    let file_path = config_dir.join("holidays.toml");
    let toml_content = r#"
        [2024-08-12]
        description = "Mountain Day"
        portion = "full"

        [2025-02-05]
        description = "Ocean Day"
        portion = "half"
    "#;

    fs::write(&file_path, toml_content.trim())?;

    let expected_output = "\
Holidays in 2024:
2024-08-12 — Mountain Day
";

    let mut cmd = Command::cargo_bin("ebb")?;
    cmd.arg("holiday")
       .arg("list")
       .arg("-y")
       .arg("2024")
       .env("EBB_CONFIG_DIR", tmp.path())
       .assert()
       .success()
       .stdout(expected_output);

    Ok(())
}
