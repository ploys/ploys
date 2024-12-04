use ploys::project::{Error, Project};

#[test]
#[ignore]
fn test_valid_local_project() -> Result<(), Error> {
    let project = Project::git("../..")?;
    let url = project.get_url()?;

    assert_eq!(project.name(), "ploys");
    assert_eq!(url.domain(), Some("github.com"));
    assert_eq!(url.path().trim_end_matches(".git"), "/ploys/ploys");

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
        project.get_url()?,
        "https://github.com/ploys/ploys".parse().unwrap()
    );

    assert!(project.packages().any(|pkg| pkg.name() == "ploys"));
    assert!(project.packages().any(|pkg| pkg.name() == "ploys-cli"));

    Ok(())
}
