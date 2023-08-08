use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn test_inspect_command_output() {
    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir("../..")
        .arg("inspect")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"Name:[ \t]*ploys"#).unwrap())
        .stdout(predicate::str::is_match(r#"Repository:.*github\.com/ploys/ploys"#).unwrap());
}
