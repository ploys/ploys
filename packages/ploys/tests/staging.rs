use ploys::project::Project;
use ploys::repository::types::staging::Staging;
use tempfile::tempdir;

#[test]
fn test_project_write() -> Result<(), Box<dyn std::error::Error>> {
    let repository = Staging::new()
        .with_file("Ploys.toml", "[project]\nname = \"example\"")
        .with_file("Cargo.toml", "[workspace]\nmembers = [\"packages/*\"]")
        .with_file(
            "packages/example/Cargo.toml",
            "[package]\nname = \"example\"",
        );

    let dir = tempdir()?;
    let project = Project::open(repository)?;
    let project = project.write(dir.path(), false)?;
    let package = project.get_package("example").unwrap();

    assert_eq!(package.name(), "example");

    dir.close()?;

    Ok(())
}
