use std::path::Path;

use ploys::project::{Error, Project};
use ploys::repository::github::Error as GitHubError;
use ploys::repository::revision::Revision;

#[test]
#[ignore]
fn test_project() -> Result<(), Error<GitHubError>> {
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

    let packages = project.packages().collect::<Vec<_>>();

    let ploys = packages.iter().find(|pkg| pkg.name() == "ploys").unwrap();

    assert_eq!(ploys.name(), "ploys");
    assert_eq!(ploys.path(), Path::new("packages/ploys"));
    assert_eq!(
        ploys.manifest_path(),
        Path::new("packages/ploys/Cargo.toml")
    );

    let ploys_api = packages
        .iter()
        .find(|pkg| pkg.name() == "ploys-api")
        .unwrap();

    assert_eq!(ploys_api.name(), "ploys-api");
    assert_eq!(ploys_api.path(), Path::new("packages/ploys-api"));
    assert_eq!(
        ploys_api.manifest_path(),
        Path::new("packages/ploys-api/Cargo.toml")
    );

    let ploys_cli = packages
        .iter()
        .find(|pkg| pkg.name() == "ploys-cli")
        .unwrap();

    assert_eq!(ploys_cli.name(), "ploys-cli");
    assert_eq!(ploys_cli.path(), Path::new("packages/ploys-cli"));
    assert_eq!(
        ploys_cli.manifest_path(),
        Path::new("packages/ploys-cli/Cargo.toml")
    );

    Ok(())
}
