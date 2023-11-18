use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

#[cfg(test)]
mod usage {
  use super::*;
  #[test]
  fn test_empty_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd.arg("");
    cmd
      .assert()
      .failure()
      .stderr(predicate::str::contains("Usage: tailcall"));

    Ok(())
  }
}

// Check command tests
#[cfg(test)]
mod check {
  use super::*;
  #[test]
  fn test_file_not_specified() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd.arg("check");
    cmd.assert().failure().stderr(predicate::str::contains(
      "error: the following required arguments were not provided",
    ));
    cmd
      .assert()
      .failure()
      .stderr(predicate::str::contains("Usage: tailcall check <FILE_PATH>"));

    Ok(())
  }

  #[test]
  fn test_file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd.arg("check").arg("test.file.doesnt.exist.graphql");
    cmd
      .assert()
      .failure()
      .stderr(predicate::str::contains("Error: No such file or directory"));

    Ok(())
  }

  #[test]
  fn test_file_exists_and_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd
      .arg("check")
      .arg("tests/graphql/errors/test-const-with-inline.graphql");
    cmd
      .assert()
      .failure()
      .stderr(predicate::str::contains("Error: Validation Error"));

    Ok(())
  }

  #[test]
  fn test_file_exists_and_valid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd
      .arg("check")
      .arg("examples/jsonplaceholder.graphql")
      .arg("--n-plus-one-queries")
      .arg("--schema");
    cmd
      .assert()
      .success()
      .stdout(predicate::str::contains("No errors found"));

    Ok(())
  }

  #[test]
  fn test_file_exists_and_valid2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;
    cmd
      .arg("check")
      .arg("examples/jsonplaceholder.graphql")
      .arg("-n")
      .arg("-s");
    cmd
      .assert()
      .success()
      .stdout(predicate::str::contains("No errors found"));

    Ok(())
  }

  #[test]
  fn test_file_exists_and_valid3() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;
    cmd.arg("check").arg("examples/jsonplaceholder.graphql").arg("-s");
    cmd
      .assert()
      .success()
      .stdout(predicate::str::contains("No errors found"));

    Ok(())
  }
}

// Start command tests
#[cfg(test)]
mod start {
  use super::*;
  #[test]
  fn test_file_not_specified() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd.arg("start");
    cmd.assert().failure().stderr(predicate::str::contains(
      "error: the following required arguments were not provided",
    ));
    cmd
      .assert()
      .failure()
      .stderr(predicate::str::contains("Usage: tailcall start <FILE_PATH>"));

    Ok(())
  }

  #[test]
  fn test_file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd.arg("start").arg("test.file.doesnt.exist.graphql");
    cmd
      .assert()
      .failure()
      .stderr(predicate::str::contains("Error: No such file or directory"));

    Ok(())
  }

  #[test]
  fn test_log_level() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd.arg("start").arg("--log_level");
    cmd.assert().failure().stderr(predicate::str::contains(
      "error: unexpected argument '--log_level' found",
    ));
    cmd.assert().failure().stderr(predicate::str::contains(
      "Usage: tailcall start <FILE_PATH|--log-level <LOG_LEVEL>>",
    ));

    Ok(())
  }

  #[test]
  fn test_log_level2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd.arg("start").arg("--log-level");
    cmd.assert().failure().stderr(predicate::str::contains(
      "error: a value is required for '--log-level <LOG_LEVEL>' but none was supplied",
    ));
    cmd
      .assert()
      .failure()
      .stderr(predicate::str::contains("For more information, try '--help'"));

    Ok(())
  }

  #[test]
  fn test_log_level3() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd.arg("start").arg("--log-level").arg("DEBUG");
    cmd.assert().failure().stderr(predicate::str::contains(
      "error: the following required arguments were not provided",
    ));
    cmd.assert().failure().stderr(predicate::str::contains(
      "Usage: tailcall start --log-level <LOG_LEVEL> <FILE_PATH>",
    ));

    Ok(())
  }

  #[test]
  fn test_file_exists_and_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tailcall")?;

    cmd
      .arg("start")
      .arg("tests/graphql/errors/test-const-with-inline.graphql");
    cmd
      .assert()
      .failure()
      .stderr(predicate::str::contains("Error: Invalid Configuration"));

    Ok(())
  }
}

// Init command tests
#[cfg(test)]
mod init {
  use std::path::PathBuf;
  use std::{env, fs};

  use super::*;

  #[test]
  fn test_empty_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("tailcall")?;

    cmd.arg("init");
    cmd
      .assert()
      .failure()
      .stderr(predicates::prelude::predicate::str::contains(
        "error: the following required arguments were not provided:",
      ));
    cmd
      .assert()
      .failure()
      .stderr(predicates::prelude::predicate::str::contains(
        "Usage: tailcall init <FILE_PATH>",
      ));

    Ok(())
  }

  #[test]
  #[serial]
  fn test_folder_nonexistent() -> Result<(), rexpect::error::Error> {
    let mut p = rexpect::spawn("cargo run -- init tmp0", Some(500000))?;
    let mut res = p.exp_regex(r#".*Do you want to add a file to the project\?.*"#)?;
    println!("PROMPT: {:?}", res);

    p.send("N\n")?; // send the newline in tests because it can't do the readline checks that it would in a proper terminal
    res = p.exp_regex(".*No such file or directory.*")?;
    println!("OUTPUT after sending 'N': {:?}", res);

    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  fn test_nonexistent_folder_and_file(
    folder_name: &str,
    answer1: &str,
    answer2: &str,
    answer3: &str,
  ) -> Result<(), rexpect::error::Error> {
    let folder_name = &folder_to_path_cwd(folder_name);
    let mut p = rexpect::spawn(&format!("cargo run -- init {}", folder_name), Some(5000))?;

    let res1 = p.exp_regex(r#".*Do you want to add a file to the project\?.*"#)?;
    println!("PROMPT: {:?}", res1);
    p.send(&format!("{}\n", answer1))?;

    let res2 = p.exp_regex(".*Enter the file name.*")?;
    println!("OUTPUT after sending '{}' to 1st prompt: {:?}", answer1, res2);
    p.send_line(&format!("{}\n", answer2))?;

    let res3 = p.exp_regex(".*Do you want to create the file.*")?;
    p.send_line(&format!("{}\n", answer3))?;
    println!("OUTPUT after sending '{}' to 2nd prompt: {:?}", answer2, res3);

    if !folder_exists(folder_name) {
      let res4 = p.exp_regex(".*No such file or directory.*")?;
      println!("OUTPUT after sending '{}' to 3rd prompt: {:?}", answer3, res4);
    } else {
      let res4 = p.exp_regex(r#".*.*"#)?;
      println!("OUTPUT after sending '{}' to 3rd prompt: {:?}", answer3, res4);

      std::thread::sleep(std::time::Duration::from_secs(2)); // wait for the spawned process to write to disk before proceeding

      if answer3.eq_ignore_ascii_case("y") {
        // assert the other files were created
        assert!(
          file_exists_in_folder(folder_name, answer2),
          "File does not exist in the folder."
        );
        assert!(
          file_exists_in_folder(folder_name, ".graphqlrc.yml"),
          "File does not exist in the folder."
        );
      }
      // assert the file was created
      assert!(folder_exists(folder_name), "Folder does not exist.");
      assert!(
        file_exists_in_folder(folder_name, ".tailcallrc.graphql"),
        "File does not exist in the folder."
      );
    }

    Ok(())
  }

  #[test]
  #[serial]
  fn test_folder_nonexistent2() -> Result<(), rexpect::error::Error> {
    return test_nonexistent_folder_and_file("tmp0", "y", "test", "n");
  }

  #[test]
  #[serial]
  fn test_folder_nonexistent3() -> Result<(), rexpect::error::Error> {
    return test_nonexistent_folder_and_file("tmp0", "y", "test", "y");
  }

  #[test]
  #[serial]
  fn test_folder_exists() -> Result<(), rexpect::error::Error> {
    let folder_name = "tmp01";
    create_folder(folder_name);
    let _ = test_nonexistent_folder_and_file(folder_name, "y", "test1.graphql", "n");
    delete_folder(folder_name);

    let folder_name = "tmp02";
    create_folder(folder_name);
    let _ = test_nonexistent_folder_and_file(folder_name, "y", "test2.graphql", "y");
    delete_folder(folder_name);

    Ok(())
  }

  fn folder_exists(folder_name: &str) -> bool {
    let path = std::path::Path::new(folder_name);
    path.is_dir()
  }

  fn folder_to_path_cwd(folder_name: &str) -> String {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let file_path = current_dir.join(folder_name);

    return file_path.to_string_lossy().into_owned();
  }

  fn file_exists_in_folder(folder_path: &str, file_name: &str) -> bool {
    let file_path = PathBuf::from(folder_path).join(file_name);
    fs::metadata(file_path).is_ok()
  }

  fn create_folder(folder_name: &str) -> bool {
    match fs::create_dir(folder_name) {
      Ok(_) => {
        println!("Folder '{}' created successfully.", folder_name);
        true
      }
      Err(e) => {
        eprintln!("Error creating folder '{}': {}", folder_name, e);
        false
      }
    }
  }

  fn delete_folder(folder_name: &str) -> bool {
    match fs::remove_dir_all(folder_name) {
      Ok(_) => {
        println!("Folder '{}' deleted successfully.", folder_name);
        true
      }
      Err(e) => {
        eprintln!("Error deleting folder '{}': {}", folder_name, e);
        false
      }
    }
  }
}
