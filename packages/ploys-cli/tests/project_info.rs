use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
#[ignore]
fn test_project_info_command() {
    Command::cargo_bin("ploys")
        .unwrap()
        .arg("project")
        .arg("info")
        .arg("ploys/ploys")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*ploys/ploys"#).unwrap());

    Command::cargo_bin("ploys")
        .unwrap()
        .arg("project")
        .arg("info")
        .arg("ploys/repo-not-found")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not Found"));
}
