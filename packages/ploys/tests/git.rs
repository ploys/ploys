use ploys::project::{Error, Project};
use ploys::repository::revision::Revision;
use ploys::repository::types::git::{Error as GitError, Git};
use ploys::repository::{Commit, Repository, Stage};
use tempfile::tempdir;

#[test]
fn test_repository() -> Result<(), GitError> {
    let dir = tempdir()?;
    let mut repo = Git::init(dir.path())?;

    repo.add_file("hello.txt", "Hello World!")?;
    repo.add_file("foo/bar/baz", "Baz")?;
    repo.commit("Initial commit")?;

    assert_eq!(repo.get_file("hello.txt")?, Some("Hello World!".into()));
    assert_eq!(repo.get_file("foo/bar/baz")?, Some("Baz".into()));

    repo.add_file("README.md", "# Readme")?;
    repo.remove_file("hello.txt")?;
    repo.remove_file("foo/bar/baz")?;
    repo.commit("Second commit")?;

    assert_eq!(repo.get_file("README.md")?, Some("# Readme".into()));
    assert_eq!(repo.get_file("hello.txt")?, None);
    assert_eq!(repo.get_file("foo/bar/baz")?, None);

    let mut repo = Git::open(dir.path())?;

    assert_eq!(repo.get_file("README.md")?, Some("# Readme".into()));
    assert_eq!(repo.get_file("hello.txt")?, None);
    assert_eq!(repo.get_file("foo/bar/baz")?, None);

    repo.add_file("Ploys.toml", "[project]\nname = \"example\"")?;
    repo.commit("Third commit")?;

    assert_eq!(
        repo.get_file("Ploys.toml")?,
        Some("[project]\nname = \"example\"".into())
    );

    let branch_name = gix::open(dir.path())?
        .head()?
        .referent_name()
        .map(|name| name.shorten().to_string())
        .unwrap_or_else(|| gix::init::DEFAULT_BRANCH_NAME.to_string());

    let mut repo = Git::open(dir.path())?.with_revision(Revision::branch(branch_name));

    repo.add_file("Cargo.toml", "[package]\nname = \"example\"")?;
    repo.commit("Fourth commit")?;

    assert_eq!(
        repo.get_file("Cargo.toml")?,
        Some("[package]\nname = \"example\"".into())
    );

    let sha = gix::open(dir.path())?
        .head_commit()
        .unwrap()
        .id()
        .to_string();

    let mut repo = Git::open(dir.path())?.with_revision(Revision::sha(sha));

    repo.remove_file("Cargo.toml")?;
    repo.add_file("commit", "5")?;
    repo.commit("Fifth commit")?;

    assert_eq!(repo.get_file("Cargo.toml")?, None);
    assert_eq!(
        repo.get_file("Ploys.toml")?,
        Some("[project]\nname = \"example\"".into())
    );
    assert_eq!(repo.get_file("commit")?, Some("5".into()));

    let repo = Git::open(dir.path())?;

    assert_eq!(
        repo.get_file("Cargo.toml")?,
        Some("[package]\nname = \"example\"".into())
    );
    assert_eq!(
        repo.get_file("Ploys.toml")?,
        Some("[project]\nname = \"example\"".into())
    );
    assert_eq!(repo.get_file("commit")?, None);

    dir.close()?;

    Ok(())
}

#[test]
#[ignore]
fn test_project() -> Result<(), Error<GitError>> {
    let project = Project::git("../..")?;

    assert_eq!(project.name(), "ploys");
    assert_eq!(
        project.repository().unwrap().to_url(),
        "https://github.com/ploys/ploys".parse().unwrap()
    );

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
