use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
#[ignore]
fn test_project_info_command_git_output() {
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
#[ignore]
fn test_project_info_command_github_output() {
    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("project")
        .arg("info")
        .arg("--remote")
        .arg("ploys/ploys")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*github\.com/ploys/ploys"#).unwrap());
}

#[test]
#[ignore]
fn test_project_info_command_github_url_output() {
    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("project")
        .arg("info")
        .arg("--remote")
        .arg("https://github.com/ploys/ploys")
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
        .arg("https://github.com/ploys/ploys.git")
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
