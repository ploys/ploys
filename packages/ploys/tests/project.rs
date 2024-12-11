use ploys::project::{Error, Project};

#[test]
#[ignore]
fn test_valid_local_project() -> Result<(), Error> {
    let project = Project::git("../..")?;

    assert_eq!(project.name(), "ploys");
    assert_eq!(
        project.repository().to_url(),
        "https://github.com/ploys/ploys".parse().unwrap()
    );

    assert!(project.packages().any(|pkg| pkg.name() == "ploys"));
    assert!(project.packages().any(|pkg| pkg.name() == "ploys-cli"));

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
        project.repository().to_url(),
        "https://github.com/ploys/ploys".parse().unwrap()
    );

    assert!(project.packages().any(|pkg| pkg.name() == "ploys"));
    assert!(project.packages().any(|pkg| pkg.name() == "ploys-cli"));

    Ok(())
}
