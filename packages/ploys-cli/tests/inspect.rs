use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn test_inspect_local_command_output() {
    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("inspect")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*github\.com/ploys/ploys"#).unwrap());
}

#[test]
fn test_inspect_remote_command_output() {
    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("inspect")
        .arg("--remote")
        .arg("ploys/ploys")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*github\.com/ploys/ploys"#).unwrap());
}

#[test]
fn test_inspect_remote_url_command_output() {
    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("inspect")
        .arg("--remote")
        .arg("https://github.com/ploys/ploys")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*github\.com/ploys/ploys"#).unwrap());

    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("inspect")
        .arg("--remote")
        .arg("https://github.com/ploys/ploys.git")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*github\.com/ploys/ploys"#).unwrap());

    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("inspect")
        .arg("--remote")
        .arg("https://github.com/ploys/repo-not-found")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Repository not found"));
}
