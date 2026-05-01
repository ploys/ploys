use ploys::client::{Client, Credentials, Error, Token};

#[test]
#[ignore]
fn test_project() -> Result<(), Error> {
    let client = match std::env::var("GITHUB_TOKEN") {
        Ok(token) => Client::build()
            .with_credentials(Credentials::new().with_access_token(Token::new(token).unwrap()))
            .finished()?,
        Err(_) => Client::build().finished()?,
    };

    let project = client.get_project("ploys/ploys")?;

    assert_eq!(project.name(), "ploys");
    assert_eq!(project.repository().unwrap().full_name(), "ploys/ploys");

    let packages = project.packages().collect::<Vec<_>>();

    let ploys = packages.iter().find(|pkg| pkg.name() == "ploys").unwrap();

    assert_eq!(ploys.name(), "ploys");
    assert_eq!(ploys.path(), "packages/ploys");
    assert_eq!(ploys.manifest_path(), "packages/ploys/Cargo.toml");

    let ploys_api = packages
        .iter()
        .find(|pkg| pkg.name() == "ploys-api")
        .unwrap();

    assert_eq!(ploys_api.name(), "ploys-api");
    assert_eq!(ploys_api.path(), "packages/ploys-api");
    assert_eq!(ploys_api.manifest_path(), "packages/ploys-api/Cargo.toml");

    let ploys_cli = packages
        .iter()
        .find(|pkg| pkg.name() == "ploys-cli")
        .unwrap();

    assert_eq!(ploys_cli.name(), "ploys-cli");
    assert_eq!(ploys_cli.path(), "packages/ploys-cli");
    assert_eq!(ploys_cli.manifest_path(), "packages/ploys-cli/Cargo.toml");

    Ok(())
}
