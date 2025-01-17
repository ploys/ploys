use std::process::Command;

use assert_cmd::prelude::*;
use ploys::project::Project;
use tempfile::tempdir;

#[test]
fn test_project_init_cwd() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir(dir.path())
        .arg("project")
        .arg("init")
        .arg("--name")
        .arg("example")
        .assert()
        .success();

    let project = Project::fs(dir.path()).unwrap();

    assert_eq!(project.name(), "example");

    dir.close().unwrap();
}

#[test]
fn test_project_init_path() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir(dir.path())
        .arg("project")
        .arg("init")
        .arg(dir.path())
        .arg("--name")
        .arg("example")
        .assert()
        .success();

    let project = Project::fs(dir.path()).unwrap();

    assert_eq!(project.name(), "example");

    dir.close().unwrap();
}
