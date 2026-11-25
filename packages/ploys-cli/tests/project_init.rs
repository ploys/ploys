use std::process::Command;

use assert_cmd::prelude::*;
use ploys::project::Project;
use ploys::repository::types::git::Git;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn initializes_unborn_git_project_in_current_directory() {
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

    let git_dir = dir.path().join(".git");
    let head = std::fs::read_to_string(git_dir.join("HEAD")).unwrap();
    let branch = head.trim().strip_prefix("ref: ").unwrap();

    assert!(!git_dir.join(branch).exists());
    assert!(!git_dir.join("index").exists());
    assert!(!dir.path().join(".gitignore").exists());

    dir.close().unwrap();
}

#[test]
fn creates_missing_parent_directories_for_absolute_path() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("missing").join("example");

    Command::cargo_bin("ploys")
        .unwrap()
        .current_dir(dir.path())
        .arg("project")
        .arg("init")
        .arg(&path)
        .assert()
        .success();

    let project = Project::fs(&path).unwrap();

    assert_eq!(project.name(), "example");

    dir.close().unwrap();
}

#[test]
fn rejects_existing_git_repository_without_writing_project_config() {
    let dir = tempdir().unwrap();

    Git::init(dir.path()).unwrap();

    let head_path = dir.path().join(".git/HEAD");
    let head = std::fs::read(&head_path).unwrap();

    Command::cargo_bin("ploys")
        .unwrap()
        .args(["project", "init"])
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(contains("Expected an empty directory"));

    assert!(!dir.path().join("Ploys.toml").exists());
    assert_eq!(std::fs::read(head_path).unwrap(), head);

    dir.close().unwrap();
}

#[test]
fn rejects_destination_containing_only_target_directory() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("target");

    std::fs::create_dir(&target).unwrap();

    Command::cargo_bin("ploys")
        .unwrap()
        .args(["project", "init"])
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(contains("Expected an empty directory"));

    assert!(target.is_dir());
    assert!(!dir.path().join(".git").exists());
    assert!(!dir.path().join("Ploys.toml").exists());

    dir.close().unwrap();
}
