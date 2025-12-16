use ploys::project::{Error, Project};
use ploys::repository::types::fs::{Error as FsError, FileSystem};
use ploys::repository::{Commit, Open, Stage};
use tempfile::tempdir;

#[test]
fn test_repository() -> Result<(), FsError> {
    let dir = tempdir()?;
    let mut repo = FileSystem::open(dir.path())?;

    repo.add_file("hello-world.txt", "Hello World")?;
    repo.commit(())?;

    assert_eq!(
        std::fs::read_to_string(repo.path().join("hello-world.txt"))?,
        "Hello World"
    );

    repo.remove_file("hello-world.txt")?;
    repo.commit(())?;

    assert!(!repo.path().join("hello-world.txt").exists());
    assert!(repo.path().read_dir()?.next().is_none());

    repo.add_file("one/two/three.txt", "3")?;
    repo.commit(())?;

    assert_eq!(
        std::fs::read_to_string(repo.path().join("one/two/three.txt"))?,
        "3"
    );

    repo.remove_file("one/two/three.txt")?;
    repo.commit(())?;

    assert!(!repo.path().join("one/two/three.txt").exists());
    assert!(!repo.path().join("one/two").exists());
    assert!(!repo.path().join("one").exists());
    assert!(repo.path().read_dir()?.next().is_none());

    dir.close()?;

    Ok(())
}

#[test]
#[ignore]
fn test_project() -> Result<(), Error<FsError>> {
    let project = Project::fs("../..")?;

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

#[test]
fn test_project_write() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;

    std::fs::write(
        dir.path().join("Ploys.toml"),
        "[project]\nname = \"example\"",
    )?;

    let mut project = Project::fs(dir.path())?;

    project.add_file("file.txt", "New File")?;

    assert!(!dir.path().join("file.txt").exists());

    project.write()?;

    assert_eq!(
        std::fs::read_to_string(dir.path().join("file.txt"))?,
        "New File"
    );

    assert_eq!(project.get_file("file.txt")?, Some("New File".into()));

    dir.close()?;

    Ok(())
}
