use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn test_project_info_command_fs_output() {
    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("project")
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*github\.com/ploys/ploys"#).unwrap());
}

#[test]
fn test_project_info_command_git_output() {
    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("project")
        .arg("info")
        .arg("--head")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*github\.com/ploys/ploys"#).unwrap());
}

#[test]
#[ignore]
fn test_project_info_command_github_output() {
    let mut command = Command::cargo_bin("ploys").unwrap();

    command
        .current_dir("../..")
        .arg("project")
        .arg("info")
        .arg("--remote")
        .arg("https://github.com/ploys/ploys");

    if let Ok(sha) = std::env::var("GITHUB_SHA") {
        command.arg("--sha").arg(sha);
    }

    command
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*github\.com/ploys/ploys"#).unwrap());

    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("project")
        .arg("info")
        .arg("--remote")
        .arg("https://github.com/ploys/repo-not-found")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not Found"));
}
