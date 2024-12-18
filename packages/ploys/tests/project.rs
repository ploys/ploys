use ploys::project::{Error, Project};
use ploys::repository::revision::Revision;

#[test]
#[ignore]
fn test_valid_local_project() -> Result<(), Error> {
    let project = Project::git("../..")?;

    assert_eq!(project.name(), "ploys");
    assert_eq!(
        project.repository().unwrap().to_url(),
        "https://github.com/ploys/ploys".parse().unwrap()
    );

    assert!(project.packages().any(|pkg| pkg.name() == "ploys"));
    assert!(project.packages().any(|pkg| pkg.name() == "ploys-cli"));

    Ok(())
}

#[test]
#[ignore]
fn test_valid_remote_project() -> Result<(), Error> {
    let revision = std::env::var("GITHUB_SHA")
        .map(Revision::Sha)
        .unwrap_or_default();

    let project = match std::env::var("GITHUB_TOKEN").ok() {
        Some(token) => {
            Project::github_with_revision_and_authentication_token("ploys/ploys", revision, token)?
        }
        None => Project::github_with_revision("ploys/ploys", revision)?,
    };

    assert_eq!(project.name(), "ploys");
    assert_eq!(
        project.repository().unwrap().to_url(),
        "https://github.com/ploys/ploys".parse().unwrap()
    );

    assert!(project.packages().any(|pkg| pkg.name() == "ploys"));
    assert!(project.packages().any(|pkg| pkg.name() == "ploys-cli"));

    Ok(())
}
