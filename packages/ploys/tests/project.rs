use std::path::PathBuf;

use ploys::project::{Error, Project};

#[test]
#[ignore]
fn test_valid_local_project() -> Result<(), Error> {
    let project = Project::local("../..")?;
    let url = project.get_url()?;

    assert_eq!(project.get_name()?, "ploys");
    assert_eq!(url.domain(), Some("github.com"));
    assert_eq!(url.path().trim_end_matches(".git"), "/ploys/ploys");

    let files = project.get_files()?;

    assert!(files.contains(&PathBuf::from("Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys/Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys-cli/Cargo.toml")));

    Ok(())
}

#[test]
#[ignore]
fn test_valid_remote_project() -> Result<(), Error> {
    let project = match std::env::var("GITHUB_TOKEN").ok() {
        Some(token) => Project::remote_with_authentication_token("ploys/ploys", token)?,
        None => Project::remote("ploys/ploys")?,
    };

    assert_eq!(project.get_name()?, "ploys");
    assert_eq!(
        project.get_url()?,
        "https://github.com/ploys/ploys".parse().unwrap()
    );

    let files = project.get_files()?;

    assert!(files.contains(&PathBuf::from("Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys/Cargo.toml")));
    assert!(files.contains(&PathBuf::from("packages/ploys-cli/Cargo.toml")));

    Ok(())
}
