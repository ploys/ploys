use std::path::PathBuf;

use ploys::project::{Error, Project};

#[test]
#[ignore]
fn test_valid_local_project_gix() -> Result<(), Error> {
    let project = Project::git("../..")?;
    let url = project.get_url()?;

    assert_eq!(project.name(), "ploys");
    assert_eq!(url.domain(), Some("github.com"));
    assert_eq!(url.path().trim_end_matches(".git"), "/ploys/ploys");

    let files = project.get_files()?;

    assert!(files.contains(&PathBuf::from("Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys/Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys-cli/Cargo.toml")));

    let a = String::from_utf8(project.get_file_contents("Cargo.toml")?).unwrap();
    let b = String::from_utf8(project.get_file_contents("packages/ploys/Cargo.toml")?).unwrap();

    assert!(a.contains("[workspace]"));
    assert!(b.contains("[package]"));
    assert!(project.get_file_contents("packages/ploys").is_err());

    let packages = project.packages();

    assert!(packages.iter().any(|pkg| pkg.name() == "ploys"));
    assert!(packages.iter().any(|pkg| pkg.name() == "ploys-cli"));

    Ok(())
}

#[test]
#[ignore]
fn test_valid_local_project_git2() -> Result<(), Error> {
    let mut project = Project::git2("../..")?;

    project.upgrade()?;

    let url = project.get_url()?;

    assert_eq!(project.name(), "ploys");
    assert_eq!(url.domain(), Some("github.com"));
    assert_eq!(url.path().trim_end_matches(".git"), "/ploys/ploys");

    let files = project.get_files()?;

    assert!(files.contains(&PathBuf::from("Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys/Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys-cli/Cargo.toml")));

    let a = String::from_utf8(project.get_file_contents("Cargo.toml")?).unwrap();
    let b = String::from_utf8(project.get_file_contents("packages/ploys/Cargo.toml")?).unwrap();

    assert!(a.contains("[workspace]"));
    assert!(b.contains("[package]"));
    assert!(project.get_file_contents("packages/ploys").is_err());

    let packages = project.packages();

    assert!(packages.iter().any(|pkg| pkg.name() == "ploys"));
    assert!(packages.iter().any(|pkg| pkg.name() == "ploys-cli"));

    Ok(())
}

#[test]
#[ignore]
fn test_valid_remote_project() -> Result<(), Error> {
    let project = match std::env::var("GITHUB_TOKEN").ok() {
        Some(token) => Project::github_with_authentication_token("ploys/ploys", token)?,
        None => Project::github("ploys/ploys")?,
    };

    assert_eq!(project.name(), "ploys");
    assert_eq!(
        project.get_url()?,
        "https://github.com/ploys/ploys".parse().unwrap()
    );

    let files = project.get_files()?;

    assert!(files.contains(&PathBuf::from("Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys/Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys-cli/Cargo.toml")));

    let a = String::from_utf8(project.get_file_contents("Cargo.toml")?).unwrap();
    let b = String::from_utf8(project.get_file_contents("packages/ploys/Cargo.toml")?).unwrap();

    assert!(a.contains("[workspace]"));
    assert!(b.contains("[package]"));
    assert!(project.get_file_contents("packages/ploys").is_err());

    let packages = project.packages();

    assert!(packages.iter().any(|pkg| pkg.name() == "ploys"));
    assert!(packages.iter().any(|pkg| pkg.name() == "ploys-cli"));

    Ok(())
}
